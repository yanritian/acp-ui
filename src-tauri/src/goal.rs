//! Goal-Driven Architecture — Core Data Structures (RFC-001)
//!
//! Replaces the TaskShard/TaskPartitioner system with Goals that have:
//! - Explicit completion conditions (auto-evaluated or Queen-judged)
//! - Iterative reconcile loop (dispatch → evaluate → feedback → retry)
//! - Token budget tracking
//! - Parent-child dependency graph

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::swarm_adapters::{WorkerId, now_ms};

// ---------------------------------------------------------------------------
// Goal
// ---------------------------------------------------------------------------

/// A Goal represents a unit of work with explicit success criteria.
///
/// Unlike the old TaskShard system, Goals have:
/// - A `CompletionCondition` that determines when the goal is "done"
/// - An `Evaluator` that checks the condition (auto, Queen, adversarial)
/// - An iteration history tracking each attempt
/// - A token budget to prevent runaway execution
///
/// ## Unified Field Names (C-1 compatibility)
/// The following fields have aliases for backward compatibility:
/// - `dependencies` also accepts `depends_on`
/// - `parent_id` also accepts `parent_goal_id`
/// - `token_used` also accepts `tokens_used`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Goal {
    /// Unique identifier
    pub id: String,
    /// Human-readable description of what to achieve
    pub description: String,
    /// Condition that must be true for the goal to be considered converged
    pub completion_condition: CompletionCondition,
    /// How the completion condition is evaluated
    pub evaluator: Evaluator,
    /// Worker assigned to execute this goal
    pub assigned_worker: Option<WorkerId>,
    /// Current status
    pub status: GoalStatus,
    /// History of each iteration attempt
    pub iterations: Vec<IterationRecord>,
    /// Maximum number of iterations before failing
    pub max_iterations: u32,
    /// Token budget (None = unlimited)
    pub token_budget: Option<u64>,
    /// Tokens used so far (alias: tokens_used for compatibility)
    #[serde(alias = "tokens_used")]
    pub token_used: u64,
    /// Creation timestamp (UNIX ms)
    pub created_at: u64,
    /// When execution started (UNIX ms)
    pub started_at: Option<u64>,
    /// When the goal converged (UNIX ms)
    pub converged_at: Option<u64>,
    /// Parent goal ID (alias: parent_goal_id for compatibility)
    #[serde(alias = "parent_goal_id")]
    pub parent_id: Option<String>,
    /// Goal IDs that must converge before this goal can start (alias: depends_on)
    #[serde(alias = "depends_on")]
    pub dependencies: Vec<String>,
}

impl Goal {
    /// Create a new goal with sensible defaults
    pub fn new(
        id: String,
        description: String,
        completion_condition: CompletionCondition,
    ) -> Self {
        Self {
            id,
            description,
            completion_condition,
            evaluator: Evaluator::Auto,
            assigned_worker: None,
            status: GoalStatus::Pending,
            iterations: Vec::new(),
            max_iterations: 5,
            token_budget: None,
            token_used: 0,
            created_at: now_ms(),
            started_at: None,
            converged_at: None,
            parent_id: None,
            dependencies: Vec::new(),
        }
    }

    /// Set the evaluator
    pub fn with_evaluator(mut self, evaluator: Evaluator) -> Self {
        self.evaluator = evaluator;
        self
    }

    /// Set the assigned worker
    pub fn with_worker(mut self, worker_id: WorkerId) -> Self {
        self.assigned_worker = Some(worker_id);
        self
    }

    /// Set max iterations
    pub fn with_max_iterations(mut self, max: u32) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set token budget
    pub fn with_token_budget(mut self, budget: u64) -> Self {
        self.token_budget = Some(budget);
        self
    }

    /// Set parent goal
    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dep_id: String) -> Self {
        self.dependencies.push(dep_id);
        self
    }

    // =========================================================================
    // Unified Field Names (C-1 compatibility aliases)
    // =========================================================================

    /// Get dependencies (alias: depends_on)
    pub fn depends_on(&self) -> &[String] {
        &self.dependencies
    }

    /// Set dependencies (alias for compatibility)
    pub fn set_depends_on(&mut self, deps: Vec<String>) {
        self.dependencies = deps;
    }

    /// Get parent goal ID (alias: parent_goal_id)
    pub fn parent_goal_id(&self) -> Option<&String> {
        self.parent_id.as_ref()
    }

    /// Set parent goal ID (alias for compatibility)
    pub fn set_parent_goal_id(&mut self, id: Option<String>) {
        self.parent_id = id;
    }

    /// Get tokens used (alias: tokens_used)
    pub fn tokens_used(&self) -> u64 {
        self.token_used
    }

    /// Set tokens used (alias for compatibility)
    pub fn set_tokens_used(&mut self, tokens: u64) {
        self.token_used = tokens;
    }

    /// Check if the goal has exceeded its budget
    pub fn is_budget_exhausted(&self) -> bool {
        self.token_budget
            .map(|budget| self.token_used >= budget)
            .unwrap_or(false)
    }

    /// Check if max iterations have been reached
    pub fn is_max_iterations_reached(&self) -> bool {
        self.iterations.len() as u32 >= self.max_iterations
    }

    /// Get the current iteration number (1-based)
    pub fn current_iteration(&self) -> u32 {
        self.iterations.len() as u32 + 1
    }

    /// Get the latest feedback from the evaluator (if any)
    pub fn latest_feedback(&self) -> Option<&str> {
        self.iterations.last().and_then(|i| i.feedback.as_deref())
    }
}

// ---------------------------------------------------------------------------
// CompletionCondition
// ---------------------------------------------------------------------------

/// Defines when a goal is considered "done".
///
/// Each variant represents a different way to check completion.
/// The `ConditionEvaluator` evaluates these against execution results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompletionCondition {
    /// A shell command exits with the expected code
    CommandSuccess {
        command: String,
        expected_exit_code: i32,
    },
    /// The output contains a specific text
    OutputContains {
        text: String,
        case_sensitive: bool,
    },
    /// The output matches a regex pattern
    OutputMatches {
        pattern: String,
    },
    /// All sub-conditions must be true
    All {
        conditions: Vec<CompletionCondition>,
    },
    /// At least one sub-condition must be true
    Any {
        conditions: Vec<CompletionCondition>,
    },
    /// A file exists (and optionally contains text)
    FileCheck {
        path: String,
        must_exist: bool,
        content_contains: Option<String>,
    },
    /// An HTTP endpoint returns the expected status
    HttpHealthCheck {
        url: String,
        expected_status: u16,
    },
    /// The Queen (Claude Code) judges the result
    QueenJudgment {
        criteria: String,
    },
}

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

/// Determines how the completion condition is evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Evaluator {
    /// Automatic evaluation using CompletionCondition (no human/Queen needed)
    Auto,
    /// Queen (Claude Code) judges the result
    Queen {
        queen_worker_id: WorkerId,
    },
    /// Two workers: primary does the work, adversary verifies
    Adversarial {
        primary: WorkerId,
        adversary: WorkerId,
    },
    /// Combination: auto conditions + Queen judgment
    Hybrid {
        auto_conditions: Vec<CompletionCondition>,
        queen_criteria: String,
    },
}

// ---------------------------------------------------------------------------
// GoalStatus
// ---------------------------------------------------------------------------

/// Current state of a goal in the reconcile loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GoalStatus {
    /// Not yet started
    Pending,
    /// Currently being executed by a worker
    Active,
    /// Execution complete, awaiting evaluation
    Evaluating,
    /// Successfully converged — completion condition met
    Converged,
    /// Failed evaluation, retrying with feedback
    Iterating {
        feedback: String,
    },
    /// Permanently failed
    Failed {
        reason: String,
    },
    /// Token budget exhausted
    BudgetExhausted,
    /// Cancelled by user
    Cancelled,
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoalStatus::Pending => write!(f, "pending"),
            GoalStatus::Active => write!(f, "active"),
            GoalStatus::Evaluating => write!(f, "evaluating"),
            GoalStatus::Converged => write!(f, "converged"),
            GoalStatus::Iterating { .. } => write!(f, "iterating"),
            GoalStatus::Failed { .. } => write!(f, "failed"),
            GoalStatus::BudgetExhausted => write!(f, "budget_exhausted"),
            GoalStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl GoalStatus {
    /// Check if this is a terminal state (no further changes)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            GoalStatus::Converged
                | GoalStatus::Failed { .. }
                | GoalStatus::BudgetExhausted
                | GoalStatus::Cancelled
        )
    }

    /// Check if this is a successful terminal state
    pub fn is_success(&self) -> bool {
        matches!(self, GoalStatus::Converged)
    }
}

// ---------------------------------------------------------------------------
// IterationRecord
// ---------------------------------------------------------------------------

/// Record of a single iteration attempt in the reconcile loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IterationRecord {
    /// Iteration number (1-based)
    pub iteration: u32,
    /// Worker output from this attempt
    pub worker_output: String,
    /// Evaluation result
    pub evaluation: EvaluationResult,
    /// Feedback for the next iteration (if failed)
    pub feedback: Option<String>,
    /// Tokens consumed in this iteration
    pub tokens_used: u64,
    /// Timestamp (UNIX ms)
    pub timestamp: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Result of evaluating a completion condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EvaluationResult {
    /// Whether the condition passed
    pub passed: bool,
    /// Human-readable explanation
    pub explanation: String,
    /// Per-condition results (for compound conditions)
    pub details: Vec<ConditionResult>,
}

/// Result of evaluating a single condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConditionResult {
    /// Condition description
    pub description: String,
    /// Whether this condition passed
    pub passed: bool,
    /// Details / evidence
    pub evidence: String,
}

// ---------------------------------------------------------------------------
// GoalGraph - Goal Dependency Graph Manager
// ---------------------------------------------------------------------------

use std::sync::Mutex;

/// Graph of all goals with dependency tracking and state management.
pub struct GoalGraph {
    goals: Mutex<HashMap<String, Goal>>,
}

impl GoalGraph {
    pub fn new() -> Self {
        Self {
            goals: Mutex::new(HashMap::new()),
        }
    }

    /// Submit a new goal to the graph
    pub fn submit(&self, goal: Goal) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;

        // Check for duplicate ID
        if goals.contains_key(&goal.id) {
            return Err(format!("Goal '{}' already exists", goal.id));
        }

        // Validate dependencies exist
        for dep_id in &goal.dependencies {
            if !goals.contains_key(dep_id) {
                return Err(format!("Dependency '{}' not found", dep_id));
            }
        }

        goals.insert(goal.id.clone(), goal);
        Ok(())
    }

    /// Get a goal by ID
    pub fn get(&self, id: &str) -> Option<Goal> {
        let goals = self.goals.lock().ok()?;
        goals.get(id).cloned()
    }

    /// Update goal status
    pub fn update_status(&self, id: &str, status: GoalStatus) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;

        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;

        goal.status = status;
        Ok(())
    }

    /// Add an iteration record to a goal
    pub fn add_iteration(&self, id: &str, record: IterationRecord) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;

        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;

        goal.iterations.push(record);
        Ok(())
    }

    /// Cancel a goal
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        self.update_status(id, GoalStatus::Cancelled)
    }

    /// Get all goals
    pub fn list(&self) -> Vec<Goal> {
        let goals = self.goals.lock().ok();
        match goals {
            Some(g) => g.values().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> GoalGraphSummary {
        let goals = match self.goals.lock() {
            Ok(g) => g,
            Err(_) => return GoalGraphSummary::default(),
        };

        let mut summary = GoalGraphSummary::default();
        for goal in goals.values() {
            match &goal.status {
                GoalStatus::Pending => summary.pending += 1,
                GoalStatus::Active => summary.active += 1,
                GoalStatus::Evaluating => summary.evaluating += 1,
                GoalStatus::Converged => summary.converged += 1,
                GoalStatus::Iterating { .. } => summary.iterating += 1,
                GoalStatus::Failed { .. } => summary.failed += 1,
                GoalStatus::BudgetExhausted => summary.budget_exhausted += 1,
                GoalStatus::Cancelled => summary.cancelled += 1,
            }
        }
        summary.total = goals.len() as u32;
        summary
    }

    /// Get goals ready to execute (all dependencies converged)
    pub fn get_ready_goals(&self) -> Vec<Goal> {
        let goals = match self.goals.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };

        goals
            .values()
            .filter(|goal| {
                if goal.status != GoalStatus::Pending {
                    return false;
                }
                // Check all dependencies are converged
                goal.dependencies.iter().all(|dep_id| {
                    goals
                        .get(dep_id)
                        .map(|dep| dep.status == GoalStatus::Converged)
                        .unwrap_or(false)
                })
            })
            .cloned()
            .collect()
    }

    /// Assign a worker to a goal
    pub fn assign_worker(&self, id: &str, worker_id: WorkerId) -> Result<(), String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;

        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;

        goal.assigned_worker = Some(worker_id);
        goal.status = GoalStatus::Active;
        goal.started_at = Some(now_ms());
        Ok(())
    }

    /// Execute one reconcile iteration for a goal
    pub fn execute_once(&self, id: &str, reconcile: &crate::reconcile::ReconcileLoop) -> Result<GoalStatus, String> {
        let mut goals = self.goals.lock().map_err(|e| e.to_string())?;

        let goal = goals
            .get_mut(id)
            .ok_or_else(|| format!("Goal '{}' not found", id))?;

        // Check if already terminal
        if goal.status.is_terminal() {
            return Ok(goal.status.clone());
        }

        // Execute reconcile iteration
        reconcile.reconcile_once(goal);

        Ok(goal.status.clone())
    }

    /// Get IDs of goals ready to execute (pending, all dependencies converged)
    pub fn get_ready_ids(&self) -> Vec<String> {
        let goals = match self.goals.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };

        let converged_ids: Vec<&str> = goals
            .values()
            .filter(|g| g.status.is_success())
            .map(|g| g.id.as_str())
            .collect();

        goals
            .values()
            .filter(|g| g.status == GoalStatus::Pending)
            .filter(|g| g.dependencies.iter().all(|dep| converged_ids.contains(&dep.as_str())))
            .map(|g| g.id.clone())
            .collect()
    }
}

impl Default for GoalGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics for the goal graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GoalGraphSummary {
    pub total: u32,
    pub pending: u32,
    pub active: u32,
    pub evaluating: u32,
    pub converged: u32,
    pub iterating: u32,
    pub failed: u32,
    pub budget_exhausted: u32,
    pub cancelled: u32,
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

use tauri::State;
use crate::AppState;

/// Submit a new goal
#[tauri::command]
pub fn goal_submit(
    state: State<'_, AppState>,
    goal: Goal,
) -> Result<(), String> {
    state.goal_graph.lock().map_err(|e| e.to_string())?.submit(goal)
}

/// Get goal status
#[tauri::command]
pub fn goal_get_status(
    state: State<'_, AppState>,
    id: String,
) -> Result<Goal, String> {
    state.goal_graph.lock().map_err(|e| e.to_string())?
        .get(&id)
        .ok_or_else(|| format!("Goal '{}' not found", id))
}

/// Cancel a goal
#[tauri::command]
pub fn goal_cancel(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state.goal_graph.lock().map_err(|e| e.to_string())?.cancel(&id)
}

/// Get graph summary
#[tauri::command]
pub fn goal_get_graph_summary(
    state: State<'_, AppState>,
) -> Result<GoalGraphSummary, String> {
    Ok(state.goal_graph.lock().map_err(|e| e.to_string())?.summary())
}

/// List all goals
#[tauri::command]
pub fn goal_list(
    state: State<'_, AppState>,
) -> Result<Vec<Goal>, String> {
    Ok(state.goal_graph.lock().map_err(|e| e.to_string())?.list())
}

/// Get goals ready to execute
#[tauri::command]
pub fn goal_get_ready(
    state: State<'_, AppState>,
) -> Result<Vec<Goal>, String> {
    Ok(state.goal_graph.lock().map_err(|e| e.to_string())?.get_ready_goals())
}

/// Assign a worker to a goal
#[tauri::command]
pub fn goal_assign_worker(
    state: State<'_, AppState>,
    id: String,
    worker_id: String,
) -> Result<(), String> {
    state.goal_graph.lock().map_err(|e| e.to_string())?
        .assign_worker(&id, worker_id)
}

/// Add iteration record to a goal
#[tauri::command]
pub fn goal_add_iteration(
    state: State<'_, AppState>,
    id: String,
    record: IterationRecord,
) -> Result<(), String> {
    state.goal_graph.lock().map_err(|e| e.to_string())?
        .add_iteration(&id, record)
}

/// Execute a single iteration for a goal.
///
/// This triggers ReconcileLoop.reconcile_once() for the goal.
/// The goal must have an assigned worker.
/// Returns the updated GoalStatus after this iteration.
#[tauri::command]
pub fn goal_execute_once(
    state: State<'_, AppState>,
    id: String,
) -> Result<GoalStatus, String> {
    // Get reconcile loop from state
    let reconcile = state.reconcile_loop.lock().map_err(|e| e.to_string())?;

    // Get goal graph
    let graph = state.goal_graph.lock().map_err(|e| e.to_string())?;

    // Execute via GoalGraph method
    graph.execute_once(&id, &reconcile)
}

/// Execute all ready goals in the graph.
///
/// Iterates through all pending goals with no pending dependencies,
/// executes one iteration for each, and returns the statuses.
#[tauri::command]
pub fn goal_execute_ready(
    state: State<'_, AppState>,
) -> Result<Vec<(String, GoalStatus)>, String> {
    let reconcile = state.reconcile_loop.lock().map_err(|e| e.to_string())?;
    let graph = state.goal_graph.lock().map_err(|e| e.to_string())?;

    // Get ready goal IDs
    let ready_ids = graph.get_ready_ids();

    let mut results = Vec::new();

    for id in ready_ids {
        let status = graph.execute_once(&id, &reconcile)?;
        results.push((id, status));
    }

    Ok(results)
}
