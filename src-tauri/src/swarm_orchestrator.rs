//! Swarm Orchestrator - Agent Swarm Coordination Layer for ACP-UI
//!
//! Coordinates multiple agents (Codex, Claude Code, etc.) as a unified swarm.
//! Supports different topologies (hierarchical, mesh, pipeline, star, adaptive)
//! and consensus strategies for multi-agent agreement.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use crate::AppState;

// ---------------------------------------------------------------------------
// Swarm Topology & Agent Roles
// ---------------------------------------------------------------------------

/// Swarm topology types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmTopology {
    /// Leader + workers
    Hierarchical,
    /// All-to-all
    Mesh,
    /// Sequential chain
    Pipeline,
    /// Central coordinator + peripherals
    Star,
    /// Dynamic selection based on task
    Adaptive,
}

impl std::fmt::Display for SwarmTopology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwarmTopology::Hierarchical => write!(f, "hierarchical"),
            SwarmTopology::Mesh => write!(f, "mesh"),
            SwarmTopology::Pipeline => write!(f, "pipeline"),
            SwarmTopology::Star => write!(f, "star"),
            SwarmTopology::Adaptive => write!(f, "adaptive"),
        }
    }
}

impl std::str::FromStr for SwarmTopology {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hierarchical" => Ok(SwarmTopology::Hierarchical),
            "mesh" => Ok(SwarmTopology::Mesh),
            "pipeline" => Ok(SwarmTopology::Pipeline),
            "star" => Ok(SwarmTopology::Star),
            "adaptive" => Ok(SwarmTopology::Adaptive),
            other => Err(format!(
                "Unknown topology: '{}'. Valid: hierarchical, mesh, pipeline, star, adaptive",
                other
            )),
        }
    }
}

/// Agent role in the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    /// Coordinates, plans, reviews (e.g., Claude Code)
    Orchestrator,
    /// Writes code, runs commands (e.g., Codex)
    Executor,
    /// Domain-specific expertise
    Specialist,
    /// Adversarial review
    Reviewer,
    /// Merges results
    Aggregator,
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::Orchestrator => write!(f, "orchestrator"),
            AgentRole::Executor => write!(f, "executor"),
            AgentRole::Specialist => write!(f, "specialist"),
            AgentRole::Reviewer => write!(f, "reviewer"),
            AgentRole::Aggregator => write!(f, "aggregator"),
        }
    }
}

// ---------------------------------------------------------------------------
// Swarm Agent Descriptor
// ---------------------------------------------------------------------------

/// Swarm agent descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmAgent {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub capabilities: Vec<String>,
    /// References an agent in agents.json
    pub agent_config_name: String,
    pub status: AgentSwarmStatus,
    pub current_task: Option<String>,
    pub results: Vec<AgentResult>,
    pub token_budget: Option<u64>,
    pub tokens_used: u64,
}

/// Agent status within the swarm
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentSwarmStatus {
    Idle,
    Assigned,
    Running,
    WaitingSync,
    Completed,
    Failed,
    Evicted,
}

impl std::fmt::Display for AgentSwarmStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentSwarmStatus::Idle => write!(f, "idle"),
            AgentSwarmStatus::Assigned => write!(f, "assigned"),
            AgentSwarmStatus::Running => write!(f, "running"),
            AgentSwarmStatus::WaitingSync => write!(f, "waiting_sync"),
            AgentSwarmStatus::Completed => write!(f, "completed"),
            AgentSwarmStatus::Failed => write!(f, "failed"),
            AgentSwarmStatus::Evicted => write!(f, "evicted"),
        }
    }
}

/// Result from a single agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub tokens_used: u64,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// Consensus Strategy
// ---------------------------------------------------------------------------

/// Consensus strategy for multi-agent agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusStrategy {
    /// First agent's result accepted
    FirstWins,
    /// Democratic voting
    Majority,
    /// Highest quality score wins
    BestScore,
    /// Combine all results
    Merge,
    /// Cross-validate with adversarial review
    Adversarial,
}

impl std::fmt::Display for ConsensusStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusStrategy::FirstWins => write!(f, "first_wins"),
            ConsensusStrategy::Majority => write!(f, "majority"),
            ConsensusStrategy::BestScore => write!(f, "best_score"),
            ConsensusStrategy::Merge => write!(f, "merge"),
            ConsensusStrategy::Adversarial => write!(f, "adversarial"),
        }
    }
}

impl std::str::FromStr for ConsensusStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "first_wins" | "firstwins" => Ok(ConsensusStrategy::FirstWins),
            "majority" => Ok(ConsensusStrategy::Majority),
            "best_score" | "bestscore" => Ok(ConsensusStrategy::BestScore),
            "merge" => Ok(ConsensusStrategy::Merge),
            "adversarial" => Ok(ConsensusStrategy::Adversarial),
            other => Err(format!(
                "Unknown consensus strategy: '{}'. Valid: first_wins, majority, best_score, merge, adversarial",
                other
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Swarm Task
// ---------------------------------------------------------------------------

/// Swarm task assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTask {
    pub id: String,
    pub description: String,
    pub assigned_agents: Vec<String>,
    pub topology: SwarmTopology,
    pub consensus: ConsensusStrategy,
    pub status: SwarmTaskStatus,
    pub results: Vec<AgentResult>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SwarmTaskStatus {
    Pending,
    InProgress,
    WaitingConsensus,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for SwarmTaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwarmTaskStatus::Pending => write!(f, "pending"),
            SwarmTaskStatus::InProgress => write!(f, "in_progress"),
            SwarmTaskStatus::WaitingConsensus => write!(f, "waiting_consensus"),
            SwarmTaskStatus::Completed => write!(f, "completed"),
            SwarmTaskStatus::Failed => write!(f, "failed"),
            SwarmTaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

// ---------------------------------------------------------------------------
// Swarm Health Summary
// ---------------------------------------------------------------------------

/// Swarm health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmHealth {
    pub total_agents: u32,
    pub active_agents: u32,
    pub idle_agents: u32,
    pub failed_agents: u32,
    pub pending_tasks: u32,
    pub running_tasks: u32,
    pub completed_tasks: u32,
    pub total_tokens_used: u64,
}

// ---------------------------------------------------------------------------
// The Swarm Orchestrator
// ---------------------------------------------------------------------------

/// The swarm orchestrator - coordinates multiple agents as a unified swarm
pub struct SwarmOrchestrator {
    agents: HashMap<String, SwarmAgent>,
    tasks: HashMap<String, SwarmTask>,
    default_topology: SwarmTopology,
    default_consensus: ConsensusStrategy,
    max_concurrent_agents: u32,
}

impl SwarmOrchestrator {
    /// Create a new orchestrator with sensible defaults
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            tasks: HashMap::new(),
            default_topology: SwarmTopology::Hierarchical,
            default_consensus: ConsensusStrategy::FirstWins,
            max_concurrent_agents: 8,
        }
    }

    /// Register an agent in the swarm
    pub fn register_agent(&mut self, agent: SwarmAgent) -> Result<(), String> {
        if agent.id.is_empty() {
            return Err("Agent ID cannot be empty".to_string());
        }
        if self.agents.contains_key(&agent.id) {
            return Err(format!("Agent '{}' is already registered", agent.id));
        }
        println!(
            "[SwarmOrchestrator] Registered agent: {} (role: {})",
            agent.id, agent.role
        );
        self.agents.insert(agent.id.clone(), agent);
        Ok(())
    }

    /// Remove an agent from the swarm
    pub fn remove_agent(&mut self, agent_id: &str) -> Result<SwarmAgent, String> {
        self.agents
            .remove(agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))
            .inspect(|a| {
                println!("[SwarmOrchestrator] Removed agent: {}", a.id);
            })
    }

    /// List all agents in the swarm
    pub fn list_agents(&self) -> Vec<&SwarmAgent> {
        let mut agents: Vec<&SwarmAgent> = self.agents.values().collect();
        agents.sort_by(|a, b| a.name.cmp(&b.name));
        agents
    }

    /// Get available agents: idle status, within token budget, and matching required capabilities
    pub fn get_available_agents(&self, required_capabilities: &[String]) -> Vec<&SwarmAgent> {
        self.agents
            .values()
            .filter(|a| {
                // Must be idle
                if a.status != AgentSwarmStatus::Idle {
                    return false;
                }
                // Must be within token budget
                if let Some(budget) = a.token_budget {
                    if a.tokens_used >= budget {
                        return false;
                    }
                }
                // Must have at least one of the required capabilities (if any specified)
                if !required_capabilities.is_empty() {
                    return a
                        .capabilities
                        .iter()
                        .any(|cap| required_capabilities.contains(cap));
                }
                true
            })
            .collect()
    }

    /// Create a swarm task - route to appropriate agents based on topology
    pub fn create_task(
        &mut self,
        description: String,
        topology: Option<SwarmTopology>,
        consensus: Option<ConsensusStrategy>,
    ) -> Result<SwarmTask, String> {
        let topo = topology.unwrap_or(self.default_topology.clone());
        let cons = consensus.unwrap_or(self.default_consensus.clone());

        // Select agents based on topology
        let selected_ids: Vec<String> = match &topo {
            SwarmTopology::Hierarchical | SwarmTopology::Star => {
                // Need one orchestrator + executors
                let orchestrators: Vec<&SwarmAgent> = self
                    .agents
                    .values()
                    .filter(|a| {
                        matches!(a.role, AgentRole::Orchestrator)
                            && a.status == AgentSwarmStatus::Idle
                    })
                    .collect();
                let executors: Vec<&SwarmAgent> = self
                    .agents
                    .values()
                    .filter(|a| {
                        matches!(a.role, AgentRole::Executor | AgentRole::Specialist)
                            && a.status == AgentSwarmStatus::Idle
                    })
                    .collect();

                let mut ids = Vec::new();
                if let Some(orch) = orchestrators.first() {
                    ids.push(orch.id.clone());
                }
                for exec in executors
                    .iter()
                    .take(self.max_concurrent_agents as usize - 1)
                {
                    ids.push(exec.id.clone());
                }
                ids
            }
            SwarmTopology::Pipeline => {
                // All idle agents in order (orchestrator first, then executors, then reviewers)
                let mut sorted: Vec<&SwarmAgent> = self
                    .agents
                    .values()
                    .filter(|a| a.status == AgentSwarmStatus::Idle)
                    .collect();
                sorted.sort_by_key(|a| match a.role {
                    AgentRole::Orchestrator => 0,
                    AgentRole::Executor => 1,
                    AgentRole::Specialist => 2,
                    AgentRole::Reviewer => 3,
                    AgentRole::Aggregator => 4,
                });
                sorted
                    .iter()
                    .take(self.max_concurrent_agents as usize)
                    .map(|a| a.id.clone())
                    .collect()
            }
            SwarmTopology::Mesh | SwarmTopology::Adaptive => {
                // All idle agents participate
                self.agents
                    .values()
                    .filter(|a| a.status == AgentSwarmStatus::Idle)
                    .take(self.max_concurrent_agents as usize)
                    .map(|a| a.id.clone())
                    .collect()
            }
        };

        if selected_ids.is_empty() {
            return Err(
                "No available agents for task. Register agents or wait for running ones to complete."
                    .to_string(),
            );
        }

        let task_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Mark selected agents as assigned
        for agent_id in &selected_ids {
            if let Some(agent) = self.agents.get_mut(agent_id) {
                agent.status = AgentSwarmStatus::Assigned;
                agent.current_task = Some(task_id.clone());
            }
        }

        let task = SwarmTask {
            id: task_id.clone(),
            description,
            assigned_agents: selected_ids,
            topology: topo,
            consensus: cons,
            status: SwarmTaskStatus::InProgress,
            results: Vec::new(),
            created_at: now,
            completed_at: None,
        };

        println!(
            "[SwarmOrchestrator] Created task {} with {} agents",
            task_id,
            task.assigned_agents.len()
        );

        self.tasks.insert(task_id, task.clone());
        Ok(task)
    }

    /// Update task with agent result and check consensus
    pub fn submit_result(
        &mut self,
        task_id: &str,
        agent_id: &str,
        result: AgentResult,
    ) -> Result<SwarmTaskStatus, String> {
        // Verify agent is assigned to this task (scoped to release immutable borrow early)
        {
            let task = self
                .tasks
                .get(task_id)
                .ok_or_else(|| format!("Task '{}' not found", task_id))?;

            if !task.assigned_agents.contains(&agent_id.to_string()) {
                return Err(format!(
                    "Agent '{}' is not assigned to task '{}'",
                    agent_id, task_id
                ));
            }
        }

        // Update agent status
        if let Some(agent) = self.agents.get_mut(agent_id) {
            agent.status = AgentSwarmStatus::Completed;
            agent.current_task = None;
            agent.tokens_used += result.tokens_used;
            agent.results.push(result.clone());
        }

        // Get task mutably and record result
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;
        task.results.push(result);

        // Check consensus
        let new_status = self.evaluate_consensus(task_id)?;

        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;
        task.status = new_status.clone();

        if matches!(
            new_status,
            SwarmTaskStatus::Completed | SwarmTaskStatus::Failed
        ) {
            task.completed_at = Some(chrono::Utc::now().to_rfc3339());
        }

        Ok(new_status)
    }

    /// Evaluate whether consensus has been reached for a task
    fn evaluate_consensus(&self, task_id: &str) -> Result<SwarmTaskStatus, String> {
        let task = self
            .tasks
            .get(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;

        let total_assigned = task.assigned_agents.len();
        let total_results = task.results.len();
        let success_count = task.results.iter().filter(|r| r.success).count();

        // If not all agents have reported, still in progress
        if total_results < total_assigned {
            // Check if remaining agents can still change the outcome
            match &task.consensus {
                ConsensusStrategy::FirstWins => {
                    // First result wins - already have at least one
                    if total_results >= 1 {
                        return Ok(SwarmTaskStatus::Completed);
                    }
                }
                _ => {
                    return Ok(SwarmTaskStatus::InProgress);
                }
            }
        }

        // All agents have reported - evaluate based on strategy
        match &task.consensus {
            ConsensusStrategy::FirstWins => {
                // First result decides
                if task.results.first().is_some_and(|r| r.success) {
                    Ok(SwarmTaskStatus::Completed)
                } else {
                    Ok(SwarmTaskStatus::Failed)
                }
            }
            ConsensusStrategy::Majority => {
                // Majority vote
                if success_count * 2 > total_results {
                    Ok(SwarmTaskStatus::Completed)
                } else {
                    Ok(SwarmTaskStatus::Failed)
                }
            }
            ConsensusStrategy::BestScore => {
                // Best quality score wins (look for "score" in output JSON)
                let has_any_success = task.results.iter().any(|r| r.success);
                if has_any_success {
                    Ok(SwarmTaskStatus::Completed)
                } else {
                    Ok(SwarmTaskStatus::Failed)
                }
            }
            ConsensusStrategy::Merge => {
                // All results are merged - considered complete if at least one succeeded
                let has_any_success = task.results.iter().any(|r| r.success);
                if has_any_success {
                    Ok(SwarmTaskStatus::Completed)
                } else {
                    Ok(SwarmTaskStatus::Failed)
                }
            }
            ConsensusStrategy::Adversarial => {
                // Require all agents to succeed (cross-validation)
                if success_count == total_results {
                    Ok(SwarmTaskStatus::Completed)
                } else {
                    Ok(SwarmTaskStatus::WaitingConsensus)
                }
            }
        }
    }

    /// Get task status and progress
    pub fn get_task(&self, task_id: &str) -> Option<&SwarmTask> {
        self.tasks.get(task_id)
    }

    /// List all tasks, sorted by creation time (newest first)
    pub fn list_tasks(&self) -> Vec<&SwarmTask> {
        let mut tasks: Vec<&SwarmTask> = self.tasks.values().collect();
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        tasks
    }

    /// Cancel a running task and release assigned agents
    pub fn cancel_task(&mut self, task_id: &str) -> Result<(), String> {
        let task = self
            .tasks
            .get(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?
            .clone();

        if matches!(
            task.status,
            SwarmTaskStatus::Completed | SwarmTaskStatus::Cancelled
        ) {
            return Err(format!(
                "Task '{}' is already in terminal state: {}",
                task_id, task.status
            ));
        }

        // Release assigned agents back to idle
        for agent_id in &task.assigned_agents {
            if let Some(agent) = self.agents.get_mut(agent_id) {
                if agent.current_task.as_deref() == Some(task_id) {
                    agent.status = AgentSwarmStatus::Idle;
                    agent.current_task = None;
                }
            }
        }

        // Update task status
        let task_mut = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task '{}' not found", task_id))?;
        task_mut.status = SwarmTaskStatus::Cancelled;
        task_mut.completed_at = Some(chrono::Utc::now().to_rfc3339());

        println!("[SwarmOrchestrator] Cancelled task {}", task_id);
        Ok(())
    }

    /// Get swarm health summary
    pub fn get_health(&self) -> SwarmHealth {
        let total_agents = self.agents.len() as u32;
        let active_agents = self
            .agents
            .values()
            .filter(|a| {
                matches!(
                    a.status,
                    AgentSwarmStatus::Running
                        | AgentSwarmStatus::Assigned
                        | AgentSwarmStatus::WaitingSync
                )
            })
            .count() as u32;
        let idle_agents = self
            .agents
            .values()
            .filter(|a| a.status == AgentSwarmStatus::Idle)
            .count() as u32;
        let failed_agents = self
            .agents
            .values()
            .filter(|a| {
                matches!(
                    a.status,
                    AgentSwarmStatus::Failed | AgentSwarmStatus::Evicted
                )
            })
            .count() as u32;

        let pending_tasks = self
            .tasks
            .values()
            .filter(|t| t.status == SwarmTaskStatus::Pending)
            .count() as u32;
        let running_tasks = self
            .tasks
            .values()
            .filter(|t| {
                matches!(
                    t.status,
                    SwarmTaskStatus::InProgress | SwarmTaskStatus::WaitingConsensus
                )
            })
            .count() as u32;
        let completed_tasks = self
            .tasks
            .values()
            .filter(|t| t.status == SwarmTaskStatus::Completed)
            .count() as u32;

        let total_tokens_used: u64 = self.agents.values().map(|a| a.tokens_used).sum();

        SwarmHealth {
            total_agents,
            active_agents,
            idle_agents,
            failed_agents,
            pending_tasks,
            running_tasks,
            completed_tasks,
            total_tokens_used,
        }
    }

    /// Evict underperforming agents (success rate below threshold).
    /// Returns the IDs of evicted agents.
    pub fn check_and_evict(&mut self, failure_threshold: f64) -> Vec<String> {
        let mut evicted = Vec::new();

        let agent_ids: Vec<String> = self.agents.keys().cloned().collect();
        for agent_id in agent_ids {
            if let Some(agent) = self.agents.get(&agent_id) {
                let total_results = agent.results.len();
                if total_results < 3 {
                    // Not enough data to evict
                    continue;
                }
                let failures = agent.results.iter().filter(|r| !r.success).count();
                let failure_rate = failures as f64 / total_results as f64;

                if failure_rate >= failure_threshold {
                    if let Some(agent) = self.agents.get_mut(&agent_id) {
                        agent.status = AgentSwarmStatus::Evicted;
                        println!(
                            "[SwarmOrchestrator] Evicted agent '{}' (failure rate: {:.1}%)",
                            agent_id,
                            failure_rate * 100.0
                        );
                        evicted.push(agent_id);
                    }
                }
            }
        }

        evicted
    }
}

impl Default for SwarmOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Register an agent in the swarm
#[tauri::command]
pub fn swarm_register_agent(state: State<'_, AppState>, agent: SwarmAgent) -> Result<(), String> {
    let mut orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.register_agent(agent)
}

/// List all agents in the swarm
#[tauri::command]
pub fn swarm_list_agents(state: State<'_, AppState>) -> Result<Vec<SwarmAgent>, String> {
    let orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.list_agents().into_iter().cloned().collect())
}

/// Create a new swarm task
#[tauri::command]
pub fn swarm_create_task(
    state: State<'_, AppState>,
    description: String,
    topology: Option<String>,
    consensus: Option<String>,
) -> Result<SwarmTask, String> {
    let mut orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;

    let topo = match topology {
        Some(t) => Some(t.parse::<SwarmTopology>()?),
        None => None,
    };
    let cons = match consensus {
        Some(c) => Some(c.parse::<ConsensusStrategy>()?),
        None => None,
    };

    orchestrator.create_task(description, topo, cons)
}

/// Submit a result from an agent for a task. Returns the new task status.
#[tauri::command]
pub fn swarm_submit_result(
    state: State<'_, AppState>,
    task_id: String,
    agent_id: String,
    result: AgentResult,
) -> Result<String, String> {
    let mut orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    let status = orchestrator.submit_result(&task_id, &agent_id, result)?;
    Ok(status.to_string())
}

/// Get task status and progress
#[tauri::command]
pub fn swarm_get_task(state: State<'_, AppState>, task_id: String) -> Result<SwarmTask, String> {
    let orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator
        .get_task(&task_id)
        .cloned()
        .ok_or_else(|| format!("Task '{}' not found", task_id))
}

/// Get swarm health summary
#[tauri::command]
pub fn swarm_get_health(state: State<'_, AppState>) -> Result<SwarmHealth, String> {
    let orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    Ok(orchestrator.get_health())
}

/// Cancel a running task in swarm orchestrator
#[tauri::command]
pub fn swarm_cancel_task(state: State<'_, AppState>, task_id: String) -> Result<(), String> {
    let mut orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.cancel_task(&task_id)
}

// ---------------------------------------------------------------------------
// Chain Topology Execution (Phase 2)
// ---------------------------------------------------------------------------

impl SwarmOrchestrator {
    /// Execute chain topology - sequential pipeline execution
    ///
    /// Worker A → Worker B → Worker C
    /// Each worker's output becomes the next worker's input
    pub fn execute_chain(
        &mut self,
        task_description: &str,
        worker_ids: &[String],
    ) -> Result<SwarmResult, String> {
        if worker_ids.is_empty() {
            return Err("Chain requires at least one worker".to_string());
        }

        let mut current_input = task_description.to_string();
        let mut stage_results: Vec<StageResult> = Vec::new();

        for (i, worker_id) in worker_ids.iter().enumerate() {
            let agent = self
                .agents
                .get(worker_id)
                .ok_or_else(|| format!("Worker '{}' not found", worker_id))?;

            // Check worker availability
            if agent.status != AgentSwarmStatus::Idle {
                return Err(format!(
                    "Worker '{}' is not idle (status: {})",
                    worker_id, agent.status
                ));
            }

            // Build stage prompt
            let stage_prompt = format!(
                "[Chain Pipeline Stage {}/{}]\n\n\
                Original task: {}\n\n\
                Previous stage output:\n{}\n\n\
                Please process this input and produce your stage output.",
                i + 1,
                worker_ids.len(),
                task_description,
                current_input
            );

            // Create stage task
            let stage_task_id = format!("chain-stage-{}-{}", i, uuid::Uuid::new_v4());
            let stage_task = SwarmTask {
                id: stage_task_id.clone(),
                description: stage_prompt.clone(),
                status: SwarmTaskStatus::Pending,
                assigned_agents: vec![worker_id.clone()],
                results: Vec::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                completed_at: None,
                topology: SwarmTopology::Pipeline,
                consensus: ConsensusStrategy::FirstWins,
            };

            // Store the stage task for tracking
            self.tasks.insert(stage_task_id.clone(), stage_task);

            // NOTE: Real adapter execution is handled by the topology module
            // (topology::execute_chain via GoalGraph + ReconcileLoop).
            // This legacy path records the stage intent; actual worker dispatch
            // requires the ReconcileLoop which is not accessible from SwarmOrchestrator.
            let stage_output = format!(
                "[Stage {}/{}] Assigned to worker '{}'. \
                 Real execution is routed through topology::execute_chain (GoalGraph + ReconcileLoop).",
                i + 1,
                worker_ids.len(),
                worker_id,
            );
            let stage_result = StageResult {
                worker_id: worker_id.clone(),
                stage_index: i,
                input: current_input.clone(),
                output: stage_output.clone(),
                success: false,
                error: Some(
                    "Legacy execute_chain does not dispatch to real adapters; \
                     use topology::execute_chain for production execution."
                        .into(),
                ),
            };

            stage_results.push(stage_result);
            current_input = stage_output;

            // Update stored task status
            if let Some(task) = self.tasks.get_mut(&stage_task_id) {
                task.status = SwarmTaskStatus::InProgress;
            }

            // Update agent status
            if let Some(agent) = self.agents.get_mut(worker_id) {
                agent.status = AgentSwarmStatus::Running;
                agent.current_task = Some(stage_task_id);
            }
        }

        // Mark all workers as idle again (chain completed)
        for worker_id in worker_ids {
            if let Some(agent) = self.agents.get_mut(worker_id) {
                agent.status = AgentSwarmStatus::Idle;
                agent.current_task = None;
            }
        }

        Ok(SwarmResult {
            final_output: current_input,
            stages: stage_results,
            total_stages: worker_ids.len(),
        })
    }

    /// Handle worker failure - retry or find replacement
    pub fn handle_worker_failure(
        &mut self,
        failed_worker_id: &str,
        _task_description: &str,
        required_capabilities: &[String],
    ) -> Result<String, String> {
        // Mark failed worker as unhealthy
        if let Some(agent) = self.agents.get_mut(failed_worker_id) {
            agent.status = AgentSwarmStatus::Failed;
            println!("[SwarmOrchestrator] Worker '{}' failed", failed_worker_id);
        }

        // Find replacement worker with matching capabilities
        let replacement = self.find_replacement_worker(required_capabilities)?;

        // Assign task to replacement
        if let Some(agent) = self.agents.get_mut(&replacement) {
            agent.status = AgentSwarmStatus::Running;
            println!(
                "[SwarmOrchestrator] Reassigned task to replacement worker '{}'",
                replacement
            );
        }

        Ok(replacement)
    }

    /// Find a replacement worker based on capabilities
    fn find_replacement_worker(&self, required_capabilities: &[String]) -> Result<String, String> {
        let available = self.get_available_agents(required_capabilities);

        if available.is_empty() {
            return Err("No replacement worker available with required capabilities".to_string());
        }

        // Select first available worker
        Ok(available[0].id.clone())
    }
}

/// Stage result for chain topology
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageResult {
    pub worker_id: String,
    pub stage_index: usize,
    pub input: String,
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Swarm execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwarmResult {
    pub final_output: String,
    pub stages: Vec<StageResult>,
    pub total_stages: usize,
}

/// Tauri command for chain execution
#[tauri::command]
pub fn swarm_execute_chain(
    state: State<'_, AppState>,
    task_description: String,
    worker_ids: Vec<String>,
) -> Result<SwarmResult, String> {
    let mut orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.execute_chain(&task_description, &worker_ids)
}

/// Tauri command for handling worker failure
#[tauri::command]
pub fn swarm_handle_failure(
    state: State<'_, AppState>,
    failed_worker_id: String,
    task_description: String,
    required_capabilities: Vec<String>,
) -> Result<String, String> {
    let mut orchestrator = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
    orchestrator.handle_worker_failure(&failed_worker_id, &task_description, &required_capabilities)
}
