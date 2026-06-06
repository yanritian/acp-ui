//! Agent Orchestration Layer - "驾驭层"
//!
//! Core value proposition: ACP-UI doesn't build professional agents (Claude Code,
//! Codex, Cursor). Instead, it *orchestrates* them — knows how to call each one,
//! monitors their execution, and manages the results.
//!
//! This layer sits above the smart_router, workflow_engine, and hermes_flow,
//! providing a unified registry and execution interface for all external agents.

// Debug prints are intentional for development phase
#![allow(clippy::print_stdout)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::State;

use crate::smart_router::{InputType, RouteTarget, TaskAnalyzer, TaskComplexity, TaskType};
use crate::workflow_engine::{
    StageResult, WorkflowDefinition, WorkflowEngine, WorkflowStatus,
};
use crate::AppState;

// ---------------------------------------------------------------------------
// Agent Capability Registry
// ---------------------------------------------------------------------------

/// Output format an agent produces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Structured,
    Markdown,
    Diff,
    Text,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Structured => "structured",
            OutputFormat::Markdown => "markdown",
            OutputFormat::Diff => "diff",
            OutputFormat::Text => "text",
        }
    }
}

/// Task complexity aligned with smart_router's TaskComplexity
pub type TaskComplexityLevel = TaskComplexity;

/// Capability profile for a registered agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapability {
    /// Unique agent identifier
    pub agent_id: String,
    /// Human-readable name
    pub name: String,
    /// CLI command to invoke (e.g., "claude", "codex", "cursor")
    pub cli_command: String,
    /// What this agent is good at
    pub capabilities: Vec<String>,
    /// Maximum complexity this agent can handle
    pub max_complexity: TaskComplexityLevel,
    /// Execution timeout in milliseconds
    pub timeout_ms: u64,
    /// Expected output format
    pub output_format: OutputFormat,
    /// Whether the agent CLI is currently available on PATH
    pub is_available: bool,
    /// Optional working directory for CLI invocation
    pub default_cwd: Option<String>,
}

impl AgentCapability {
    /// Check if this agent can handle the given complexity
    pub fn can_handle(&self, complexity: &TaskComplexityLevel) -> bool {
        use TaskComplexityLevel as TC;
        let agent_max = self.max_complexity;
        let complexity_value = |c: &TC| -> u8 {
            match c {
                TC::Simple => 1,
                TC::Medium => 2,
                TC::Complex => 3,
                TC::VeryComplex => 4,
            }
        };
        complexity_value(complexity) <= complexity_value(&agent_max)
    }

    /// Check if this agent supports a specific capability
    pub fn supports_capability(&self, cap: &str) -> bool {
        self.capabilities.iter().any(|c| c.eq_ignore_ascii_case(cap))
    }
}

// ---------------------------------------------------------------------------
// Active Task Tracking
// ---------------------------------------------------------------------------

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Running,
    Completed,
    Failed,
    NeedsApproval,
    TimedOut,
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::NeedsApproval => write!(f, "needs_approval"),
            TaskStatus::TimedOut => write!(f, "timed_out"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// A task currently being executed by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTask {
    pub task_id: String,
    pub agent_id: String,
    pub description: String,
    pub started_at: DateTime<Utc>,
    pub status: TaskStatus,
    /// Progress percentage 0-100
    pub progress: u8,
    /// Optional workflow stage this task belongs to
    pub workflow_id: Option<String>,
    pub stage_id: Option<String>,
}

/// Historical record of a completed task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub task_id: String,
    pub agent_id: String,
    pub description: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub status: TaskStatus,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tokens_used: Option<u64>,
    pub workflow_id: Option<String>,
    pub stage_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Execution Result
// ---------------------------------------------------------------------------

/// Structured result from executing a task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub agent_id: String,
    pub task_id: String,
}

// ---------------------------------------------------------------------------
// Agent Orchestrator
// ---------------------------------------------------------------------------

/// The Agent Orchestrator — "驾驭层"
///
/// Knows all available agents, routes tasks to the best fit, executes them via CLI,
/// monitors progress, and feeds results back into the workflow engine.
pub struct AgentOrchestrator {
    /// Registry of all known agents and their capabilities
    pub registry: HashMap<String, AgentCapability>,
    /// Currently running tasks
    pub active_tasks: HashMap<String, ActiveTask>,
    /// Historical task records
    pub task_history: Vec<TaskRecord>,
    /// Child process handles for cancellation
    child_processes: HashMap<String, Arc<AtomicU8>>,
    /// Task analyzer for routing decisions
    task_analyzer: TaskAnalyzer,
    /// Workflow engine reference (populated via set_workflow_engine)
    workflow_engine: Option<Arc<std::sync::Mutex<WorkflowEngine>>>,
    /// Task counter for generating unique IDs
    task_counter: u64,
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            active_tasks: HashMap::new(),
            task_history: Vec::new(),
            child_processes: HashMap::new(),
            task_analyzer: TaskAnalyzer::new(),
            workflow_engine: None,
            task_counter: 0,
        }
    }

    /// Register a new agent capability profile
    pub fn register_agent(&mut self, capability: AgentCapability) -> Result<(), String> {
        if capability.agent_id.is_empty() {
            return Err("agent_id cannot be empty".to_string());
        }
        if capability.cli_command.is_empty() {
            return Err("cli_command cannot be empty".to_string());
        }
        if capability.capabilities.is_empty() {
            return Err("capabilities cannot be empty".to_string());
        }

        // Check CLI availability
        let available = Self::check_cli_available(&capability.cli_command);

        let mut cap = capability;
        cap.is_available = available;

        println!(
            "[AgentOrchestrator] Registered agent '{}' ({}) - available: {}",
            cap.name, cap.cli_command, available
        );
        self.registry.insert(cap.agent_id.clone(), cap);
        Ok(())
    }

    /// Unregister an agent by ID
    pub fn unregister_agent(&mut self, agent_id: &str) -> Result<(), String> {
        if self.registry.remove(agent_id).is_some() {
            println!("[AgentOrchestrator] Unregistered agent '{}'", agent_id);
            Ok(())
        } else {
            Err(format!("Agent '{}' not found", agent_id))
        }
    }

    /// Select the best agent for a given task description
    ///
    /// Uses the TaskAnalyzer to determine complexity and type, then picks
    /// the agent that: (1) is available, (2) can handle the complexity,
    /// (3) has the most matching capabilities.
    pub fn select_best_agent(&self, task_description: &str) -> Option<String> {
        let decision = self
            .task_analyzer
            .analyze(task_description, InputType::Text);

        let mut candidates: Vec<(&String, &AgentCapability, u32)> = Vec::new();

        for (agent_id, cap) in &self.registry {
            if !cap.is_available {
                continue;
            }
            if !cap.can_handle(&decision.complexity) {
                continue;
            }

            // Score: count matching capabilities
            let score = self.score_agent_for_task(cap, &decision);
            candidates.push((agent_id, cap, score));
        }

        if candidates.is_empty() {
            println!("[AgentOrchestrator] No available agent for task: {}", task_description);
            return None;
        }

        // Sort by score descending, then by max_complexity descending
        candidates.sort_by(|a, b| {
            b.2.cmp(&a.2).then_with(|| {
                let complexity_value = |c: &TaskComplexityLevel| -> u8 {
                    match c {
                        TaskComplexityLevel::Simple => 1,
                        TaskComplexityLevel::Medium => 2,
                        TaskComplexityLevel::Complex => 3,
                        TaskComplexityLevel::VeryComplex => 4,
                    }
                };
                complexity_value(&b.1.max_complexity).cmp(&complexity_value(&a.1.max_complexity))
            })
        });

        let best = &candidates[0];
        println!(
            "[AgentOrchestrator] Selected agent '{}' (score: {}) for task",
            best.0, best.2
        );
        Some(best.0.clone())
    }

    /// Select a specific agent by ID, validating it exists and is available
    pub fn select_agent_by_id(&self, agent_id: &str) -> Option<String> {
        if let Some(cap) = self.registry.get(agent_id) {
            if cap.is_available {
                Some(agent_id.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Execute a task using the specified agent
    ///
    /// Spawns the agent CLI, captures output, handles timeouts, and returns
    /// structured results.
    pub fn execute_task(
        &mut self,
        task_description: &str,
        agent_id: &str,
        working_dir: Option<&str>,
    ) -> Result<ExecutionResult, String> {
        let cap = self
            .registry
            .get(agent_id)
            .ok_or_else(|| format!("Agent '{}' not registered", agent_id))?
            .clone();

        if !cap.is_available {
            return Err(format!("Agent '{}' ({}) is not available", cap.name, cap.cli_command));
        }

        let task_id = self.generate_task_id();
        let started_at = Utc::now();

        // Register active task
        self.active_tasks.insert(
            task_id.clone(),
            ActiveTask {
                task_id: task_id.clone(),
                agent_id: agent_id.to_string(),
                description: task_description.to_string(),
                started_at,
                status: TaskStatus::Running,
                progress: 0,
                workflow_id: None,
                stage_id: None,
            },
        );

        // Create cancellation flag
        let cancel_flag = Arc::new(AtomicU8::new(0));
        self.child_processes
            .insert(task_id.clone(), cancel_flag.clone());

        // Spawn the CLI command
        let cwd = working_dir
            .or(cap.default_cwd.as_deref())
            .unwrap_or(".");

        let result = self.spawn_cli(
            &cap.cli_command,
            task_description,
            cwd,
            cap.timeout_ms,
            &cancel_flag,
        );

        // Update active task status
        if let Some(task) = self.active_tasks.get_mut(&task_id) {
            task.status = match &result {
                Ok(r) if r.success => TaskStatus::Completed,
                Ok(_) => TaskStatus::Failed,
                Err(e) if e.contains("timeout") => TaskStatus::TimedOut,
                Err(e) if e.contains("cancelled") => TaskStatus::Cancelled,
                Err(_) => TaskStatus::Failed,
            };
            task.progress = 100;
        }

        // Move to history
        if let Some(active) = self.active_tasks.get(&task_id) {
            let completed_at = Utc::now();
            let duration_ms = (completed_at - active.started_at).num_milliseconds() as u64;

            let (output, error, _success) = match &result {
                Ok(r) => (r.output.clone(), r.stderr.clone().into(), r.success),
                Err(e) => (String::new(), Some(e.clone()), false),
            };

            let record = TaskRecord {
                task_id: task_id.clone(),
                agent_id: agent_id.to_string(),
                description: task_description.to_string(),
                started_at: active.started_at,
                completed_at,
                status: active.status,
                output,
                error,
                duration_ms,
                tokens_used: None,
                workflow_id: None,
                stage_id: None,
            };

            self.task_history.push(record);
        }

        // Clean up child process tracking
        self.child_processes.remove(&task_id);

        result
    }

    /// Execute a task and automatically select the best agent
    pub fn execute_task_auto(
        &mut self,
        task_description: &str,
        working_dir: Option<&str>,
    ) -> Result<ExecutionResult, String> {
        let agent_id = self
            .select_best_agent(task_description)
            .ok_or_else(|| {
                format!("No suitable agent available for task: {}", task_description)
            })?;

        self.execute_task(task_description, &agent_id, working_dir)
    }

    /// Get the status of a specific task
    pub fn get_task_status(&self, task_id: &str) -> Option<ActiveTask> {
        // Check active tasks first
        if let Some(task) = self.active_tasks.get(task_id) {
            return Some(task.clone());
        }

        // Check history
        for record in self.task_history.iter().rev() {
            if record.task_id == task_id {
                return Some(ActiveTask {
                    task_id: record.task_id.clone(),
                    agent_id: record.agent_id.clone(),
                    description: record.description.clone(),
                    started_at: record.started_at,
                    status: record.status,
                    progress: 100,
                    workflow_id: record.workflow_id.clone(),
                    stage_id: record.stage_id.clone(),
                });
            }
        }

        None
    }

    /// Cancel a running task
    pub fn cancel_task(&mut self, task_id: &str) -> Result<(), String> {
        // Send cancellation signal
        if let Some(cancel_flag) = self.child_processes.get(task_id) {
            cancel_flag.store(1, Ordering::SeqCst);
        }

        // Also try to kill the process via taskkill on Windows
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID"])
                .arg(task_id)
                .output();
        }

        if let Some(task) = self.active_tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            task.progress = 100;
            println!("[AgentOrchestrator] Cancelled task '{}'", task_id);
            Ok(())
        } else {
            Err(format!("Task '{}' not found or not running", task_id))
        }
    }

    /// List all active tasks
    pub fn get_active_tasks(&self) -> Vec<ActiveTask> {
        self.active_tasks.values().cloned().collect()
    }

    /// List all task history records
    pub fn get_task_history(&self) -> Vec<TaskRecord> {
        self.task_history.clone()
    }

    /// Get all registered agent capabilities
    pub fn get_registered_agents(&self) -> Vec<AgentCapability> {
        self.registry.values().cloned().collect()
    }

    /// Get a specific agent capability
    pub fn get_agent_capability(&self, agent_id: &str) -> Option<AgentCapability> {
        self.registry.get(agent_id).cloned()
    }

    /// Set the workflow engine reference for workflow integration
    pub fn set_workflow_engine(&mut self, engine: Arc<std::sync::Mutex<WorkflowEngine>>) {
        self.workflow_engine = Some(engine);
    }

    /// Execute a workflow stage using the orchestrator
    ///
    /// For each stage in the workflow, selects the appropriate agent and
    /// executes it. Results from one stage feed into the next.
    pub fn execute_workflow_stage(
        &mut self,
        workflow_id: &str,
        stage_id: &str,
    ) -> Result<StageResult, String> {
        // Get workflow definition
        let (stage_name, stage_prompt, stage_agents) = {
            let engine = self
                .workflow_engine
                .as_ref()
                .ok_or("Workflow engine not configured")?;
            let guard = engine.lock().map_err(|e| e.to_string())?;
            let workflow = guard
                .get_workflow(workflow_id)
                .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?;

            let stage = workflow
                .stages
                .iter()
                .find(|s| s.id == stage_id)
                .ok_or_else(|| format!("Stage '{}' not found in workflow", stage_id))?;

            (
                stage.name.clone(),
                stage.agents.first().map(|a| a.prompt_template.clone()).unwrap_or_default(),
                stage.agents.clone(),
            )
        };

        if stage_agents.is_empty() {
            return Err(format!("Stage '{}' has no agents assigned", stage_id));
        }

        // Use the first agent (or select best from the stage's agents)
        let stage_agent = &stage_agents[0];
        let orchestrator_agent_id = self.select_agent_for_stage(&stage_agent.agent_id)?;

        println!(
            "[AgentOrchestrator] Executing workflow stage '{}' ({}) with agent '{}'",
            stage_id, stage_name, orchestrator_agent_id
        );

        let start = std::time::Instant::now();
        let result = self.execute_task(&stage_prompt, &orchestrator_agent_id, None);
        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(exec_result) => {
                let stage_result = StageResult {
                    agent_id: orchestrator_agent_id,
                    success: exec_result.success,
                    output: serde_json::json!({
                        "stdout": exec_result.output,
                        "stderr": exec_result.stderr,
                        "exit_code": exec_result.exit_code,
                    }),
                    tokens_used: 0, // Will be populated from LLM response parsing
                    duration_ms,
                    review: None,
                };

                // Submit result back to workflow engine
                if let Some(ref engine) = self.workflow_engine {
                    let mut guard = engine.lock().map_err(|e| e.to_string())?;
                    let _ = guard.submit_stage_result(workflow_id, stage_id, stage_result.clone());
                }

                Ok(stage_result)
            }
            Err(e) => Err(format!("Stage '{}' execution failed: {}", stage_id, e)),
        }
    }

    /// Execute an entire workflow using the orchestrator
    ///
    /// Resolves dependencies, executes stages in order, and feeds results
    /// from one stage to the next.
    pub fn execute_full_workflow(&mut self, workflow_id: &str) -> Result<WorkflowDefinition, String> {
        // Validate workflow
        {
            let engine = self
                .workflow_engine
                .as_ref()
                .ok_or("Workflow engine not configured")?;
            let mut guard = engine.lock().map_err(|e| e.to_string())?;
            guard.validate_workflow(workflow_id)?;
            guard.update_status(workflow_id, WorkflowStatus::Running)?;
        }

        let max_iterations = 50; // Safety limit
        for _ in 0..max_iterations {
            let ready_stages = {
                let engine = self
                    .workflow_engine
                    .as_ref()
                    .ok_or("Workflow engine not configured")?;
                let guard = engine.lock().map_err(|e| e.to_string())?;
                guard.get_ready_stages(workflow_id)?
            };

            if ready_stages.is_empty() {
                break; // All stages done
            }

            for stage_id in ready_stages {
                match self.execute_workflow_stage(workflow_id, &stage_id) {
                    Ok(_) => {
                        println!(
                            "[AgentOrchestrator] Workflow '{}' stage '{}' completed",
                            workflow_id, stage_id
                        );
                    }
                    Err(e) => {
                        println!(
                            "[AgentOrchestrator] Workflow '{}' stage '{}' failed: {}",
                            workflow_id, stage_id, e
                        );
                        // Update workflow status to failed
                        if let Some(ref engine) = self.workflow_engine {
                            let mut guard = engine.lock().map_err(|e| e.to_string())?;
                            let _ = guard.update_status(workflow_id, WorkflowStatus::Failed);
                        }
                        return Err(e);
                    }
                }
            }
        }

        // Get final workflow definition
        let engine = self
            .workflow_engine
            .as_ref()
            .ok_or("Workflow engine not configured")?;
        let guard = engine.lock().map_err(|e| e.to_string())?;
        let workflow = guard
            .get_workflow(workflow_id)
            .ok_or_else(|| format!("Workflow '{}' not found after execution", workflow_id))?
            .clone();

        println!(
            "[AgentOrchestrator] Workflow '{}' completed with status: {}",
            workflow_id, workflow.status
        );
        Ok(workflow)
    }

    /// Update progress for an active task (called periodically during execution)
    pub fn update_task_progress(&mut self, task_id: &str, progress: u8) {
        if let Some(task) = self.active_tasks.get_mut(task_id) {
            task.progress = progress.min(100);
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn generate_task_id(&mut self) -> String {
        self.task_counter += 1;
        format!("task-{}-{}", self.task_counter, uuid::Uuid::new_v4().to_string()[..8].to_string())
    }

    /// Check if a CLI command is available on PATH
    fn check_cli_available(command: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            Command::new("where")
                .arg(command)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("which")
                .arg(command)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Score an agent for a task based on capability matching and route target alignment
    fn score_agent_for_task(
        &self,
        cap: &AgentCapability,
        decision: &crate::smart_router::RouteDecision,
    ) -> u32 {
        let mut score = 0u32;

        // Capability matching
        let task_keywords = extract_task_keywords(&decision.task_type);
        for keyword in &task_keywords {
            if cap.supports_capability(keyword) {
                score += 10;
            }
        }

        // Route target bonus
        let route_bonus = match decision.route_target {
            RouteTarget::ClaudeCodeHaiku => 5,
            RouteTarget::ClaudeCodeSonnet => 8,
            RouteTarget::CodexHaiku => 5,
            RouteTarget::CodexSonnet => 8,
            RouteTarget::Team => 3,
            RouteTarget::HumanReview => 0,
        };

        if cap.name.to_lowercase().contains("claude") {
            if matches!(
                decision.route_target,
                RouteTarget::ClaudeCodeHaiku | RouteTarget::ClaudeCodeSonnet
            ) {
                score += route_bonus;
            }
        }
        if cap.name.to_lowercase().contains("codex") {
            if matches!(
                decision.route_target,
                RouteTarget::CodexHaiku | RouteTarget::CodexSonnet
            ) {
                score += route_bonus;
            }
        }

        score
    }

    /// Select an orchestrator agent ID that maps to a stage agent's agent_id
    fn select_agent_for_stage(&self, stage_agent_id: &str) -> Result<String, String> {
        // Try exact match first
        if self.registry.contains_key(stage_agent_id) {
            return Ok(stage_agent_id.to_string());
        }

        // Try fuzzy match by name
        for (agent_id, cap) in &self.registry {
            if cap.name.eq_ignore_ascii_case(stage_agent_id)
                || agent_id.eq_ignore_ascii_case(stage_agent_id)
            {
                return Ok(agent_id.clone());
            }
        }

        // Fallback: pick any available agent
        for agent_id in self.registry.keys() {
            return Ok(agent_id.clone());
        }

        Err("No available agent in registry".to_string())
    }

    /// Spawn a CLI process and capture output
    fn spawn_cli(
        &self,
        cli_command: &str,
        prompt: &str,
        cwd: &str,
        timeout_ms: u64,
        cancel_flag: &Arc<AtomicU8>,
    ) -> Result<ExecutionResult, String> {
        let start = std::time::Instant::now();

        // Build the command based on the CLI
        let mut cmd = build_agent_command(cli_command, prompt)?;
        cmd.current_dir(cwd);

        println!(
            "[AgentOrchestrator] Spawning {} in {} with timeout {}ms",
            cli_command, cwd, timeout_ms
        );

        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", cli_command, e))?;

        let child_id = child.id();

        // Spawn a thread to wait for the process
        let (tx, rx) = std::sync::mpsc::channel::<std::process::Output>();

        thread::spawn(move || {
            let output = child.wait_with_output();
            if let Ok(output) = output {
                let _ = tx.send(output);
            }
            // If wait_with_output fails, the cancel_flag will handle it
        });

        // Poll for completion or timeout
        let timeout = Duration::from_millis(timeout_ms);
        loop {
            // Check cancellation
            if cancel_flag.load(Ordering::SeqCst) == 1 {
                let _ = kill_process(child_id);
                return Err("Task cancelled by user".to_string());
            }

            // Check timeout
            if start.elapsed() > timeout {
                let _ = kill_process(child_id);
                return Err(format!(
                    "Task timed out after {}ms",
                    timeout_ms
                ));
            }

            // Try to receive output (non-blocking check)
            match rx.try_recv() {
                Ok(output) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let exit_code = output.status.code();
                    let success = output.status.success();

                    println!(
                        "[AgentOrchestrator] CLI {} completed in {}ms, exit_code: {:?}, success: {}",
                        cli_command, duration_ms, exit_code, success
                    );

                    return Ok(ExecutionResult {
                        success,
                        output: stdout,
                        stderr,
                        exit_code,
                        duration_ms,
                        agent_id: cli_command.to_string(),
                        task_id: format!("{}", child_id),
                    });
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(100));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err("Agent process terminated unexpectedly".to_string());
                }
            }
        }
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// CLI Command Builder
// ---------------------------------------------------------------------------

/// Build a process Command for a specific agent CLI
fn build_agent_command(cli_command: &str, prompt: &str) -> Result<Command, String> {
    match cli_command {
        "claude" => {
            // Claude Code CLI: claude --print -p "prompt"
            let mut cmd = Command::new("claude");
            cmd.args(["--print", "-p", prompt]);
            Ok(cmd)
        }
        "codex" | "codex-cli" => {
            // OpenAI Codex CLI: codex exec -p "prompt"
            let mut cmd = Command::new(cli_command);
            cmd.args(["exec", "-p", prompt]);
            Ok(cmd)
        }
        "cursor" | "cursor-agent" => {
            // Cursor CLI
            let mut cmd = Command::new(cli_command);
            cmd.args(["--agent", "--print", "-p", prompt]);
            Ok(cmd)
        }
        other => {
            // Generic: treat the command as a bash command
            let mut cmd = Command::new("cmd");
            #[cfg(not(target_os = "windows"))]
            {
                cmd = Command::new("sh");
                cmd.args(["-c", &format!("{} '{}'", other, prompt.replace('\'', "'\\''"))]);
            }
            #[cfg(target_os = "windows")]
            {
                cmd.args(["/c", &format!("{} \"{}\"", other, prompt)]);
            }
            Ok(cmd)
        }
    }
}

/// Kill a process by PID (cross-platform)
fn kill_process(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output()
            .map(|_| ())
            .map_err(|e| format!("Failed to kill process {}: {}", pid, e))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
            .map(|_| ())
            .map_err(|e| format!("Failed to kill process {}: {}", pid, e))
    }
}

/// Extract capability keywords from a task type
fn extract_task_keywords(task_type: &TaskType) -> Vec<String> {
    let to_vec = |items: &[&str]| items.iter().map(|s| s.to_string()).collect();
    match task_type {
        TaskType::Frontend => to_vec(&["code_generation", "frontend", "ui"]),
        TaskType::Backend => to_vec(&["code_generation", "backend", "api"]),
        TaskType::Fullstack => to_vec(&["code_generation", "fullstack"]),
        TaskType::DevOps => to_vec(&["devops", "deployment"]),
        TaskType::Testing => to_vec(&["testing", "code_understanding"]),
        TaskType::Documentation => to_vec(&["documentation"]),
        TaskType::Refactoring => to_vec(&["refactoring", "code_understanding"]),
        TaskType::BugFix => to_vec(&["code_understanding", "debugging"]),
    }
}

// ---------------------------------------------------------------------------
// Seed Default Agents
// ---------------------------------------------------------------------------

impl AgentOrchestrator {
    /// Register the default set of agents that ACP-UI can orchestrate
    pub fn seed_default_agents(&mut self) {
        let defaults = vec![
            AgentCapability {
                agent_id: "claude-code".to_string(),
                name: "Claude Code".to_string(),
                cli_command: "claude".to_string(),
                capabilities: vec![
                    "code_understanding".to_string(),
                    "code_generation".to_string(),
                    "refactoring".to_string(),
                    "editor_operations".to_string(),
                    "testing".to_string(),
                    "documentation".to_string(),
                    "frontend".to_string(),
                    "backend".to_string(),
                ],
                max_complexity: TaskComplexityLevel::VeryComplex,
                timeout_ms: 300_000, // 5 minutes
                output_format: OutputFormat::Markdown,
                is_available: true, // Will be checked on register
                default_cwd: None,
            },
            AgentCapability {
                agent_id: "codex".to_string(),
                name: "Codex".to_string(),
                cli_command: "codex".to_string(),
                capabilities: vec![
                    "code_understanding".to_string(),
                    "code_generation".to_string(),
                    "backend".to_string(),
                    "api".to_string(),
                    "devops".to_string(),
                ],
                max_complexity: TaskComplexityLevel::VeryComplex,
                timeout_ms: 300_000,
                output_format: OutputFormat::Markdown,
                is_available: true,
                default_cwd: None,
            },
            AgentCapability {
                agent_id: "cursor".to_string(),
                name: "Cursor Agent".to_string(),
                cli_command: "cursor".to_string(),
                capabilities: vec![
                    "code_understanding".to_string(),
                    "code_generation".to_string(),
                    "refactoring".to_string(),
                    "editor_operations".to_string(),
                    "frontend".to_string(),
                ],
                max_complexity: TaskComplexityLevel::Complex,
                timeout_ms: 180_000, // 3 minutes
                output_format: OutputFormat::Diff,
                is_available: true,
                default_cwd: None,
            },
        ];

        for agent in defaults {
            let _ = self.register_agent(agent);
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Register a new agent capability
#[tauri::command]
pub fn orchestration_register_agent(
    state: State<'_, AppState>,
    capability: AgentCapability,
) -> Result<(), String> {
    let mut orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.register_agent(capability)
}

/// Get all registered agents
#[tauri::command]
pub fn orchestration_list_agents(
    state: State<'_, AppState>,
) -> Result<Vec<AgentCapability>, String> {
    let orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.get_registered_agents())
}

/// Select the best agent for a task
#[tauri::command]
pub fn orchestration_select_agent(
    state: State<'_, AppState>,
    task_description: String,
) -> Result<Option<String>, String> {
    let orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.select_best_agent(&task_description))
}

/// Execute a task with a specific agent
#[tauri::command]
pub fn orchestration_execute_task(
    state: State<'_, AppState>,
    task_description: String,
    agent_id: String,
    working_dir: Option<String>,
) -> Result<ExecutionResult, String> {
    let mut orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.execute_task(&task_description, &agent_id, working_dir.as_deref())
}

/// Execute a task with auto-selected agent
#[tauri::command]
pub fn orchestration_execute_task_auto(
    state: State<'_, AppState>,
    task_description: String,
    working_dir: Option<String>,
) -> Result<ExecutionResult, String> {
    let mut orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.execute_task_auto(&task_description, working_dir.as_deref())
}

/// Get task status
#[tauri::command]
pub fn orchestration_get_task_status(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Option<ActiveTask>, String> {
    let orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.get_task_status(&task_id))
}

/// Cancel a running task
#[tauri::command]
pub fn orchestration_cancel_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<(), String> {
    let mut orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.cancel_task(&task_id)
}

/// List all active tasks
#[tauri::command]
pub fn orchestration_list_active_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<ActiveTask>, String> {
    let orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.get_active_tasks())
}

/// List task history
#[tauri::command]
pub fn orchestration_list_task_history(
    state: State<'_, AppState>,
) -> Result<Vec<TaskRecord>, String> {
    let orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.get_task_history())
}

/// Execute a workflow stage
#[tauri::command]
pub fn orchestration_execute_workflow_stage(
    state: State<'_, AppState>,
    workflow_id: String,
    stage_id: String,
) -> Result<StageResult, String> {
    let mut orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.execute_workflow_stage(&workflow_id, &stage_id)
}

/// Execute a full workflow
#[tauri::command]
pub fn orchestration_execute_full_workflow(
    state: State<'_, AppState>,
    workflow_id: String,
) -> Result<WorkflowDefinition, String> {
    let mut orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.execute_full_workflow(&workflow_id)
}

/// Seed default agents into the orchestrator
#[tauri::command]
pub fn orchestration_seed_default_agents(
    state: State<'_, AppState>,
) -> Result<Vec<AgentCapability>, String> {
    let mut orchestrator = state.agent_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.seed_default_agents();
    Ok(orchestrator.get_registered_agents())
}
