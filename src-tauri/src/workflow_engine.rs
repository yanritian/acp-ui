//! Workflow Engine - Ultra Workflow Generation & Execution for ACP-UI
//!
//! Generates and executes complex multi-stage workflows with DAG-based
//! dependency resolution, multiple stage strategies, and progress tracking.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;

use crate::AppState;

// ---------------------------------------------------------------------------
// Workflow Definition
// ---------------------------------------------------------------------------

/// Workflow definition - the orchestrator's plan as structured data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub stages: Vec<WorkflowStage>,
    pub max_concurrent_agents: u32,
    pub timeout_ms: u64,
    pub on_failure: FailurePolicy,
    pub created_at: String,
    pub status: WorkflowStatus,
    pub total_tokens_budget: Option<u64>,
}

/// Policy for handling stage failures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailurePolicy {
    /// Stop entire workflow
    StopAll,
    /// Let other stages continue
    ContinueOthers,
    /// Retry failed stage with backoff
    RetryWithBackoff,
    /// Pause and wait for human intervention
    FallbackToManual,
}

impl std::fmt::Display for FailurePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailurePolicy::StopAll => write!(f, "stop_all"),
            FailurePolicy::ContinueOthers => write!(f, "continue_others"),
            FailurePolicy::RetryWithBackoff => write!(f, "retry_with_backoff"),
            FailurePolicy::FallbackToManual => write!(f, "fallback_to_manual"),
        }
    }
}

/// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    Draft,
    Validating,
    Ready,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowStatus::Draft => write!(f, "draft"),
            WorkflowStatus::Validating => write!(f, "validating"),
            WorkflowStatus::Ready => write!(f, "ready"),
            WorkflowStatus::Running => write!(f, "running"),
            WorkflowStatus::Paused => write!(f, "paused"),
            WorkflowStatus::Completed => write!(f, "completed"),
            WorkflowStatus::Failed => write!(f, "failed"),
            WorkflowStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for WorkflowStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(WorkflowStatus::Draft),
            "validating" => Ok(WorkflowStatus::Validating),
            "ready" => Ok(WorkflowStatus::Ready),
            "running" => Ok(WorkflowStatus::Running),
            "paused" => Ok(WorkflowStatus::Paused),
            "completed" => Ok(WorkflowStatus::Completed),
            "failed" => Ok(WorkflowStatus::Failed),
            "cancelled" => Ok(WorkflowStatus::Cancelled),
            other => Err(format!("Unknown workflow status: '{}'", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// Workflow Stage
// ---------------------------------------------------------------------------

/// A stage within a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub strategy: StageStrategy,
    pub agents: Vec<StageAgent>,
    /// Stage IDs this stage depends on
    pub depends_on: Vec<String>,
    pub sync_points: Vec<StageSyncPoint>,
    pub status: WorkflowStatus,
    pub results: Vec<StageResult>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Strategy for executing agents within a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStrategy {
    /// All agents run simultaneously
    Parallel,
    /// Agents run one after another
    Sequential,
    /// Map phase (parallel) then reduce phase (single)
    MapReduce,
    /// Multiple agents compete, best wins
    Competitive,
    /// One agent acts, another reviews
    AdversarialReview,
}

impl std::fmt::Display for StageStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageStrategy::Parallel => write!(f, "parallel"),
            StageStrategy::Sequential => write!(f, "sequential"),
            StageStrategy::MapReduce => write!(f, "map_reduce"),
            StageStrategy::Competitive => write!(f, "competitive"),
            StageStrategy::AdversarialReview => write!(f, "adversarial_review"),
        }
    }
}

/// Agent assignment within a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageAgent {
    pub agent_id: String,
    pub role: String,
    pub prompt_template: String,
    pub max_tokens: Option<u64>,
}

/// Synchronization point within a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSyncPoint {
    /// Agent IDs to wait for
    pub wait_for: Vec<String>,
    pub timeout_ms: u64,
    /// Action on timeout: "skip" | "fail" | "continue"
    pub on_timeout: String,
}

// ---------------------------------------------------------------------------
// Stage Results
// ---------------------------------------------------------------------------

/// Result from a single agent within a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub agent_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub tokens_used: u64,
    pub duration_ms: u64,
    pub review: Option<ReviewResult>,
}

/// Review result from an adversarial reviewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub reviewer_id: String,
    pub approved: bool,
    pub issues: Vec<String>,
    pub score: f64,
}

// ---------------------------------------------------------------------------
// Workflow Progress
// ---------------------------------------------------------------------------

/// Workflow execution progress snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub workflow_id: String,
    pub total_stages: u32,
    pub completed_stages: u32,
    pub running_stages: u32,
    pub failed_stages: u32,
    pub total_agents: u32,
    pub active_agents: u32,
    pub tokens_used: u64,
    pub tokens_budget: Option<u64>,
    pub elapsed_ms: u64,
    pub estimated_remaining_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// The Workflow Engine
// ---------------------------------------------------------------------------

/// The workflow engine - creates, validates, and tracks multi-stage workflows
pub struct WorkflowEngine {
    workflows: HashMap<String, WorkflowDefinition>,
    progress_cache: HashMap<String, WorkflowProgress>,
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            progress_cache: HashMap::new(),
        }
    }

    /// Create a new workflow definition
    pub fn create_workflow(
        &mut self,
        name: String,
        description: String,
        stages: Vec<WorkflowStage>,
        max_concurrent: Option<u32>,
    ) -> Result<WorkflowDefinition, String> {
        if name.is_empty() {
            return Err("Workflow name cannot be empty".to_string());
        }
        if stages.is_empty() {
            return Err("Workflow must have at least one stage".to_string());
        }

        let workflow_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let workflow = WorkflowDefinition {
            id: workflow_id.clone(),
            name,
            description,
            stages,
            max_concurrent_agents: max_concurrent.unwrap_or(4),
            timeout_ms: 300_000, // 5 minutes default
            on_failure: FailurePolicy::ContinueOthers,
            created_at: now,
            status: WorkflowStatus::Draft,
            total_tokens_budget: None,
        };

        println!(
            "[WorkflowEngine] Created workflow '{}' ({})",
            workflow.name, workflow_id
        );
        self.workflows.insert(workflow_id, workflow.clone());
        Ok(workflow)
    }

    /// Validate workflow: DAG acyclicity check and dependency resolution.
    /// Returns Ok(true) if valid, Err with descriptive message if invalid.
    pub fn validate_workflow(&self, workflow_id: &str) -> Result<bool, String> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?;

        let stage_ids: HashSet<String> = workflow.stages.iter().map(|s| s.id.clone()).collect();

        // 1. Check all depends_on references exist
        for stage in &workflow.stages {
            for dep in &stage.depends_on {
                if !stage_ids.contains(dep) {
                    return Err(format!(
                        "Stage '{}' depends on non-existent stage '{}'",
                        stage.id, dep
                    ));
                }
            }
        }

        // 2. Check for self-dependencies
        for stage in &workflow.stages {
            if stage.depends_on.contains(&stage.id) {
                return Err(format!("Stage '{}' depends on itself", stage.id));
            }
        }

        // 3. DFS-based cycle detection
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();

        // Build adjacency map (stage -> stages that depend on it)
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for stage in &workflow.stages {
            adj.entry(stage.id.clone()).or_default();
            for dep in &stage.depends_on {
                adj.entry(dep.clone()).or_default().push(stage.id.clone());
            }
        }

        for stage in &workflow.stages {
            if !visited.contains(&stage.id) {
                if self.has_cycle_dfs(&stage.id, &adj, &mut visited, &mut rec_stack) {
                    return Err(format!(
                        "Cycle detected in workflow '{}' involving stage '{}'",
                        workflow_id, stage.id
                    ));
                }
            }
        }

        // 4. Check for orphan stages (no dependencies and nothing depends on them)
        // Only flag as warning if there are multiple stages
        if workflow.stages.len() > 1 {
            let has_deps: HashSet<String> = workflow
                .stages
                .iter()
                .flat_map(|s| {
                    s.depends_on
                        .iter()
                        .cloned()
                        .chain(std::iter::once(s.id.clone()))
                })
                .collect();

            let orphans: Vec<&str> = workflow
                .stages
                .iter()
                .filter(|s| !has_deps.contains(&s.id) && s.depends_on.is_empty())
                .map(|s| s.id.as_str())
                .collect();

            // Orphans are allowed but logged
            if !orphans.is_empty() {
                println!(
                    "[WorkflowEngine] Warning: workflow '{}' has orphan stages: {:?}",
                    workflow_id, orphans
                );
            }
        }

        println!("[WorkflowEngine] Workflow '{}' validated OK", workflow_id);
        Ok(true)
    }

    /// DFS helper for cycle detection
    fn has_cycle_dfs(
        &self,
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor.as_str()) {
                    if self.has_cycle_dfs(neighbor, adj, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor.as_str()) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Get workflow definition
    pub fn get_workflow(&self, id: &str) -> Option<&WorkflowDefinition> {
        self.workflows.get(id)
    }

    /// List all workflows, sorted by creation time (newest first)
    pub fn list_workflows(&self) -> Vec<&WorkflowDefinition> {
        let mut workflows: Vec<&WorkflowDefinition> = self.workflows.values().collect();
        workflows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        workflows
    }

    /// Update workflow status
    pub fn update_status(
        &mut self,
        id: &str,
        status: WorkflowStatus,
    ) -> Result<(), String> {
        let workflow = self
            .workflows
            .get_mut(id)
            .ok_or_else(|| format!("Workflow '{}' not found", id))?;
        workflow.status = status.clone();
        println!(
            "[WorkflowEngine] Workflow '{}' status -> {}",
            id, status
        );
        Ok(())
    }

    /// Get IDs of stages whose dependencies are all satisfied (completed)
    pub fn get_ready_stages(&self, workflow_id: &str) -> Result<Vec<String>, String> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?;

        let completed_ids: HashSet<String> = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Completed)
            .map(|s| s.id.clone())
            .collect();

        let ready: Vec<String> = workflow
            .stages
            .iter()
            .filter(|s| {
                // Stage must be in Draft or Ready status
                matches!(s.status, WorkflowStatus::Draft | WorkflowStatus::Ready)
                    // All dependencies must be completed
                    && s.depends_on.iter().all(|dep| completed_ids.contains(dep))
            })
            .map(|s| s.id.clone())
            .collect();

        Ok(ready)
    }

    /// Submit a result for a stage agent
    pub fn submit_stage_result(
        &mut self,
        workflow_id: &str,
        stage_id: &str,
        result: StageResult,
    ) -> Result<(), String> {
        let workflow = self
            .workflows
            .get_mut(workflow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?;

        let stage = workflow
            .stages
            .iter_mut()
            .find(|s| s.id == stage_id)
            .ok_or_else(|| {
                format!(
                    "Stage '{}' not found in workflow '{}'",
                    stage_id, workflow_id
                )
            })?;

        // Set started_at if first result
        if stage.started_at.is_none() {
            stage.started_at = Some(chrono::Utc::now().to_rfc3339());
            stage.status = WorkflowStatus::Running;
        }

        stage.results.push(result);

        // Check if stage is complete based on strategy
        let total_agents = stage.agents.len();
        let total_results = stage.results.len();

        let stage_complete = match &stage.strategy {
            StageStrategy::Parallel => total_results >= total_agents,
            StageStrategy::Sequential => total_results >= total_agents,
            StageStrategy::MapReduce => total_results >= total_agents,
            StageStrategy::Competitive => total_results >= 1, // First/best wins
            StageStrategy::AdversarialReview => total_results >= total_agents,
        };

        if stage_complete {
            let all_success = stage.results.iter().any(|r| r.success);
            stage.status = if all_success {
                WorkflowStatus::Completed
            } else {
                WorkflowStatus::Failed
            };
            stage.completed_at = Some(chrono::Utc::now().to_rfc3339());

            println!(
                "[WorkflowEngine] Stage '{}' in workflow '{}' -> {}",
                stage_id, workflow_id, stage.status
            );
        }

        // Check if entire workflow is complete
        self.check_workflow_completion(workflow_id)?;

        // Update progress cache
        self.update_progress_cache(workflow_id)?;

        Ok(())
    }

    /// Check if the entire workflow has completed or failed
    fn check_workflow_completion(&mut self, workflow_id: &str) -> Result<(), String> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?
            .clone();

        let all_completed = workflow
            .stages
            .iter()
            .all(|s| s.status == WorkflowStatus::Completed);

        let any_failed = workflow
            .stages
            .iter()
            .any(|s| s.status == WorkflowStatus::Failed);

        if all_completed {
            let wf = self
                .workflows
                .get_mut(workflow_id)
                .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?;
            wf.status = WorkflowStatus::Completed;
        } else if any_failed {
            match &workflow.on_failure {
                FailurePolicy::StopAll => {
                    let wf = self
                        .workflows
                        .get_mut(workflow_id)
                        .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?;
                    wf.status = WorkflowStatus::Failed;
                }
                FailurePolicy::FallbackToManual => {
                    let wf = self
                        .workflows
                        .get_mut(workflow_id)
                        .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?;
                    wf.status = WorkflowStatus::Paused;
                }
                _ => {
                    // ContinueOthers / RetryWithBackoff - keep running
                }
            }
        }

        Ok(())
    }

    /// Recompute progress cache for a workflow
    fn update_progress_cache(&mut self, workflow_id: &str) -> Result<(), String> {
        // Clone workflow data to avoid holding immutable borrow on self
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_id))?
            .clone();

        let total_stages = workflow.stages.len() as u32;
        let completed_stages = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Completed)
            .count() as u32;
        let running_stages = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Running)
            .count() as u32;
        let failed_stages = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Failed)
            .count() as u32;

        let total_agents: u32 = workflow.stages.iter().map(|s| s.agents.len() as u32).sum();
        let active_agents: u32 = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Running)
            .map(|s| s.agents.len() as u32)
            .sum();

        let tokens_used: u64 = workflow
            .stages
            .iter()
            .flat_map(|s| &s.results)
            .map(|r| r.tokens_used)
            .sum();

        // Estimate remaining time based on completed stage average duration
        let completed_durations: Vec<u64> = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Completed)
            .map(|s| {
                s.results.iter().map(|r| r.duration_ms).sum::<u64>()
            })
            .collect();

        let estimated_remaining_ms = if !completed_durations.is_empty() && completed_stages > 0 {
            let avg_duration =
                completed_durations.iter().sum::<u64>() / completed_stages as u64;
            let remaining_stages = total_stages - completed_stages - failed_stages;
            Some(avg_duration * remaining_stages as u64)
        } else {
            None
        };

        // Calculate elapsed time from created_at
        let elapsed_ms = if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&workflow.created_at) {
            let now = chrono::Utc::now();
            let elapsed = now.signed_duration_since(created.with_timezone(&chrono::Utc));
            elapsed.num_milliseconds().max(0) as u64
        } else {
            0
        };

        let progress = WorkflowProgress {
            workflow_id: workflow_id.to_string(),
            total_stages,
            completed_stages,
            running_stages,
            failed_stages,
            total_agents,
            active_agents,
            tokens_used,
            tokens_budget: workflow.total_tokens_budget,
            elapsed_ms,
            estimated_remaining_ms,
        };

        self.progress_cache
            .insert(workflow_id.to_string(), progress);
        Ok(())
    }

    /// Get workflow progress
    pub fn get_progress(&self, workflow_id: &str) -> Result<WorkflowProgress, String> {
        // Check workflow exists
        if !self.workflows.contains_key(workflow_id) {
            return Err(format!("Workflow '{}' not found", workflow_id));
        }

        // Return cached progress or compute fresh
        if let Some(progress) = self.progress_cache.get(workflow_id) {
            return Ok(progress.clone());
        }

        // Compute fresh progress
        let workflow = self.workflows.get(workflow_id).unwrap();
        let total_stages = workflow.stages.len() as u32;
        let completed_stages = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Completed)
            .count() as u32;
        let running_stages = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Running)
            .count() as u32;
        let failed_stages = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Failed)
            .count() as u32;
        let total_agents: u32 = workflow.stages.iter().map(|s| s.agents.len() as u32).sum();
        let active_agents: u32 = workflow
            .stages
            .iter()
            .filter(|s| s.status == WorkflowStatus::Running)
            .map(|s| s.agents.len() as u32)
            .sum();
        let tokens_used: u64 = workflow
            .stages
            .iter()
            .flat_map(|s| &s.results)
            .map(|r| r.tokens_used)
            .sum();

        let elapsed_ms = if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&workflow.created_at) {
            let now = chrono::Utc::now();
            let elapsed = now.signed_duration_since(created.with_timezone(&chrono::Utc));
            elapsed.num_milliseconds().max(0) as u64
        } else {
            0
        };

        Ok(WorkflowProgress {
            workflow_id: workflow_id.to_string(),
            total_stages,
            completed_stages,
            running_stages,
            failed_stages,
            total_agents,
            active_agents,
            tokens_used,
            tokens_budget: workflow.total_tokens_budget,
            elapsed_ms,
            estimated_remaining_ms: None,
        })
    }

    /// Cancel a workflow and all its running stages
    pub fn cancel_workflow(&mut self, id: &str) -> Result<(), String> {
        let workflow = self
            .workflows
            .get_mut(id)
            .ok_or_else(|| format!("Workflow '{}' not found", id))?;

        if matches!(
            workflow.status,
            WorkflowStatus::Completed | WorkflowStatus::Cancelled
        ) {
            return Err(format!(
                "Workflow '{}' is already in terminal state: {}",
                id, workflow.status
            ));
        }

        // Cancel all running/draft stages
        for stage in &mut workflow.stages {
            if matches!(
                stage.status,
                WorkflowStatus::Running | WorkflowStatus::Draft | WorkflowStatus::Ready
            ) {
                stage.status = WorkflowStatus::Cancelled;
                stage.completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        workflow.status = WorkflowStatus::Cancelled;
        println!("[WorkflowEngine] Cancelled workflow '{}'", id);
        Ok(())
    }

    /// Generate a template-based workflow from a task description and store it.
    /// Inspects the task description and available agents to produce a sensible workflow.
    pub fn generate_and_store_from_task(
        &mut self,
        task_description: &str,
        available_agents: &[String],
    ) -> Result<WorkflowDefinition, String> {
        if available_agents.is_empty() {
            return Err("No available agents to generate workflow".to_string());
        }

        let desc_lower = task_description.to_lowercase();
        let now = chrono::Utc::now().to_rfc3339();
        let workflow_id = uuid::Uuid::new_v4().to_string();

        let stages = if desc_lower.contains("research") || desc_lower.contains("analyze") {
            // Research task: Parallel search + AdversarialReview
            let research_agents: Vec<StageAgent> = available_agents
                .iter()
                .take(2)
                .map(|id| StageAgent {
                    agent_id: id.clone(),
                    role: "researcher".to_string(),
                    prompt_template: format!(
                        "Research and analyze: {}",
                        task_description
                    ),
                    max_tokens: Some(4096),
                })
                .collect();

            let reviewer_agent = available_agents
                .last()
                .cloned()
                .unwrap_or_else(|| available_agents[0].clone());

            vec![
                WorkflowStage {
                    id: "stage-research".to_string(),
                    name: "Research Phase".to_string(),
                    description: "Parallel information gathering".to_string(),
                    strategy: StageStrategy::Parallel,
                    agents: research_agents,
                    depends_on: vec![],
                    sync_points: vec![],
                    status: WorkflowStatus::Draft,
                    results: vec![],
                    started_at: None,
                    completed_at: None,
                },
                WorkflowStage {
                    id: "stage-review".to_string(),
                    name: "Adversarial Review".to_string(),
                    description: "Cross-validate research findings".to_string(),
                    strategy: StageStrategy::AdversarialReview,
                    agents: vec![StageAgent {
                        agent_id: reviewer_agent,
                        role: "reviewer".to_string(),
                        prompt_template: format!(
                            "Review and validate research findings for: {}",
                            task_description
                        ),
                        max_tokens: Some(2048),
                    }],
                    depends_on: vec!["stage-research".to_string()],
                    sync_points: vec![],
                    status: WorkflowStatus::Draft,
                    results: vec![],
                    started_at: None,
                    completed_at: None,
                },
            ]
        } else if desc_lower.contains("code") || desc_lower.contains("implement") || desc_lower.contains("build") {
            // Code task: Competitive (multiple implementations) + review
            let coder_agents: Vec<StageAgent> = available_agents
                .iter()
                .take(3)
                .map(|id| StageAgent {
                    agent_id: id.clone(),
                    role: "coder".to_string(),
                    prompt_template: format!(
                        "Implement the following: {}",
                        task_description
                    ),
                    max_tokens: Some(8192),
                })
                .collect();

            let reviewer_agent = available_agents
                .last()
                .cloned()
                .unwrap_or_else(|| available_agents[0].clone());

            vec![
                WorkflowStage {
                    id: "stage-implement".to_string(),
                    name: "Implementation Phase".to_string(),
                    description: "Competitive code generation".to_string(),
                    strategy: StageStrategy::Competitive,
                    agents: coder_agents,
                    depends_on: vec![],
                    sync_points: vec![],
                    status: WorkflowStatus::Draft,
                    results: vec![],
                    started_at: None,
                    completed_at: None,
                },
                WorkflowStage {
                    id: "stage-review".to_string(),
                    name: "Code Review".to_string(),
                    description: "Review implementation quality".to_string(),
                    strategy: StageStrategy::AdversarialReview,
                    agents: vec![StageAgent {
                        agent_id: reviewer_agent,
                        role: "reviewer".to_string(),
                        prompt_template: format!(
                            "Review code implementation for: {}",
                            task_description
                        ),
                        max_tokens: Some(4096),
                    }],
                    depends_on: vec!["stage-implement".to_string()],
                    sync_points: vec![],
                    status: WorkflowStatus::Draft,
                    results: vec![],
                    started_at: None,
                    completed_at: None,
                },
            ]
        } else if desc_lower.contains("file") || desc_lower.contains("multi-file") || desc_lower.contains("refactor") {
            // Multi-file task: MapReduce (map per file, reduce at end)
            let mapper_agents: Vec<StageAgent> = available_agents
                .iter()
                .take(3)
                .map(|id| StageAgent {
                    agent_id: id.clone(),
                    role: "mapper".to_string(),
                    prompt_template: format!(
                        "Process assigned files for: {}",
                        task_description
                    ),
                    max_tokens: Some(4096),
                })
                .collect();

            let reducer_agent = available_agents
                .first()
                .cloned()
                .unwrap_or_else(|| available_agents[0].clone());

            vec![
                WorkflowStage {
                    id: "stage-map".to_string(),
                    name: "Map Phase".to_string(),
                    description: "Parallel file processing".to_string(),
                    strategy: StageStrategy::MapReduce,
                    agents: mapper_agents,
                    depends_on: vec![],
                    sync_points: vec![],
                    status: WorkflowStatus::Draft,
                    results: vec![],
                    started_at: None,
                    completed_at: None,
                },
                WorkflowStage {
                    id: "stage-reduce".to_string(),
                    name: "Reduce Phase".to_string(),
                    description: "Aggregate and integrate results".to_string(),
                    strategy: StageStrategy::Sequential,
                    agents: vec![StageAgent {
                        agent_id: reducer_agent,
                        role: "reducer".to_string(),
                        prompt_template: format!(
                            "Aggregate results from map phase for: {}",
                            task_description
                        ),
                        max_tokens: Some(4096),
                    }],
                    depends_on: vec!["stage-map".to_string()],
                    sync_points: vec![],
                    status: WorkflowStatus::Draft,
                    results: vec![],
                    started_at: None,
                    completed_at: None,
                },
            ]
        } else {
            // Default: single agent sequential task
            let agent_id = available_agents[0].clone();
            vec![WorkflowStage {
                id: "stage-execute".to_string(),
                name: "Execute Task".to_string(),
                description: task_description.to_string(),
                strategy: StageStrategy::Sequential,
                agents: vec![StageAgent {
                    agent_id,
                    role: "executor".to_string(),
                    prompt_template: task_description.to_string(),
                    max_tokens: Some(8192),
                }],
                depends_on: vec![],
                sync_points: vec![],
                status: WorkflowStatus::Draft,
                results: vec![],
                started_at: None,
                completed_at: None,
            }]
        };

        let workflow = WorkflowDefinition {
            id: workflow_id.clone(),
            name: format!("Generated: {}", &task_description[..task_description.len().min(50)]),
            description: task_description.to_string(),
            stages,
            max_concurrent_agents: available_agents.len() as u32,
            timeout_ms: 600_000, // 10 minutes default for generated workflows
            on_failure: FailurePolicy::ContinueOthers,
            created_at: now,
            status: WorkflowStatus::Draft,
            total_tokens_budget: None,
        };

        println!(
            "[WorkflowEngine] Generated workflow '{}' from task description",
            workflow_id
        );
        self.workflows.insert(workflow_id, workflow.clone());
        Ok(workflow)
    }

    /// Save workflow definition as JSON for replay
    pub fn save_workflow(&self, id: &str) -> Result<serde_json::Value, String> {
        let workflow = self
            .workflows
            .get(id)
            .ok_or_else(|| format!("Workflow '{}' not found", id))?;

        serde_json::to_value(workflow).map_err(|e| format!("Failed to serialize workflow: {}", e))
    }

    /// Load a previously saved workflow from JSON
    pub fn load_workflow(
        &mut self,
        data: serde_json::Value,
    ) -> Result<WorkflowDefinition, String> {
        let workflow: WorkflowDefinition = serde_json::from_value(data)
            .map_err(|e| format!("Failed to deserialize workflow: {}", e))?;

        if workflow.id.is_empty() {
            return Err("Loaded workflow has empty ID".to_string());
        }

        println!(
            "[WorkflowEngine] Loaded workflow '{}' ({})",
            workflow.name, workflow.id
        );
        self.workflows.insert(workflow.id.clone(), workflow.clone());
        Ok(workflow)
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// Create a new workflow
#[tauri::command]
pub fn workflow_create(
    state: State<'_, AppState>,
    name: String,
    description: String,
    stages: Vec<WorkflowStage>,
    max_concurrent: Option<u32>,
) -> Result<WorkflowDefinition, String> {
    let mut engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    engine.create_workflow(name, description, stages, max_concurrent)
}

/// Validate a workflow (DAG acyclicity check, dependency resolution)
#[tauri::command]
pub fn workflow_validate(
    state: State<'_, AppState>,
    workflow_id: String,
) -> Result<bool, String> {
    let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    engine.validate_workflow(&workflow_id)
}

/// List all workflows
#[tauri::command]
pub fn workflow_list(
    state: State<'_, AppState>,
) -> Result<Vec<WorkflowDefinition>, String> {
    let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    Ok(engine
        .list_workflows()
        .into_iter()
        .cloned()
        .collect())
}

/// Get a specific workflow by ID
#[tauri::command]
pub fn workflow_get(
    state: State<'_, AppState>,
    id: String,
) -> Result<WorkflowDefinition, String> {
    let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    engine
        .get_workflow(&id)
        .cloned()
        .ok_or_else(|| format!("Workflow '{}' not found", id))
}

/// Get workflow execution progress
#[tauri::command]
pub fn workflow_get_progress(
    state: State<'_, AppState>,
    id: String,
) -> Result<WorkflowProgress, String> {
    let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    engine.get_progress(&id)
}

/// Update workflow status
#[tauri::command]
pub fn workflow_update_status(
    state: State<'_, AppState>,
    id: String,
    status: String,
) -> Result<(), String> {
    let mut engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    let parsed_status = status.parse::<WorkflowStatus>()?;
    engine.update_status(&id, parsed_status)
}

/// Submit a result for a stage agent
#[tauri::command]
pub fn workflow_submit_result(
    state: State<'_, AppState>,
    workflow_id: String,
    stage_id: String,
    result: StageResult,
) -> Result<(), String> {
    let mut engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    engine.submit_stage_result(&workflow_id, &stage_id, result)
}

/// Generate a workflow from a task description using templates
#[tauri::command]
pub fn workflow_generate_from_task(
    state: State<'_, AppState>,
    task_description: String,
    available_agents: Vec<String>,
) -> Result<WorkflowDefinition, String> {
    let mut engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    engine.generate_and_store_from_task(&task_description, &available_agents)
}

/// Cancel a workflow
#[tauri::command]
pub fn workflow_cancel(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
    engine.cancel_workflow(&id)
}

/// Save a workflow as JSON for replay, also persisting to the database
#[tauri::command]
pub fn workflow_save(
    state: State<'_, AppState>,
    id: String,
) -> Result<serde_json::Value, String> {
    // First serialize the workflow from the engine
    let workflow_json = {
        let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
        engine.save_workflow(&id)?
    };

    // Also persist to database
    let db_guard = state.database.lock().map_err(|e| e.to_string())?;
    if let Some(db) = db_guard.as_ref() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        // Extract name and description from the serialized JSON
        let name = workflow_json.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let description = workflow_json.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let created_at = workflow_json.get("created_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        conn.execute(
            "INSERT OR REPLACE INTO workflows (id, name, description, definition_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            rusqlite::params![
                id,
                name,
                description,
                serde_json::to_string(&workflow_json).unwrap_or_default(),
                created_at,
            ],
        ).map_err(|e| format!("Failed to persist workflow to database: {}", e))?;

        println!("[WorkflowEngine] Workflow '{}' persisted to database", id);
    }

    Ok(workflow_json)
}

/// Load all persisted workflows from the database into the engine
#[tauri::command]
pub fn workflow_load_all(
    state: State<'_, AppState>,
) -> Result<Vec<WorkflowDefinition>, String> {
    // First, collect all definition JSON strings from the database
    let definition_jsons: Vec<String> = {
        let db_guard = state.database.lock().map_err(|e| e.to_string())?;
        let db = db_guard.as_ref().ok_or("Database not initialized")?;
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare(
            "SELECT definition_json FROM workflows ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;

        let workflow_rows = stmt.query_map([], |row| {
            row.get::<_, String>(0)
        }).map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for row in workflow_rows {
            results.push(row.map_err(|e| e.to_string())?);
        }
        results
        // db_guard and conn are dropped here, releasing the locks
    };

    // Now load them into the engine (separate lock scope)
    let mut loaded_workflows = Vec::new();
    let mut engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;

    for definition_json in definition_jsons {
        let json_value: serde_json::Value = serde_json::from_str(&definition_json)
            .map_err(|e| format!("Failed to parse workflow JSON: {}", e))?;

        match engine.load_workflow(json_value) {
            Ok(workflow) => {
                loaded_workflows.push(workflow);
            }
            Err(e) => {
                println!("[WorkflowEngine] Warning: failed to load workflow from database: {}", e);
            }
        }
    }

    println!(
        "[WorkflowEngine] Loaded {} workflows from database",
        loaded_workflows.len()
    );
    Ok(loaded_workflows)
}
