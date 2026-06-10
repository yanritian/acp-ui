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
    /// Tokens used so far
    pub token_used: u64,
    /// Creation timestamp (UNIX ms)
    pub created_at: u64,
    /// When execution started (UNIX ms)
    pub started_at: Option<u64>,
    /// When the goal converged (UNIX ms)
    pub converged_at: Option<u64>,
    /// Parent goal ID (for sub-goals)
    pub parent_id: Option<String>,
    /// Goal IDs that must converge before this goal can start
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
