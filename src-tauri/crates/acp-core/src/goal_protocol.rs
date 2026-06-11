//! Goal Protocol - Goal definition and submission

use serde::{Deserialize, Serialize};

/// Goal specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSpec {
    /// Goal ID
    pub id: String,
    /// Goal description
    pub description: String,
    /// Completion condition
    pub completion_condition: CompletionConditionSpec,
    /// Evaluator type
    pub evaluator: String,
    /// Executor assignment
    pub executor: Option<String>,
    /// Dependencies
    pub depends_on: Vec<String>,
    /// Token budget
    pub token_budget: u64,
    /// Maximum iterations
    pub max_iterations: u32,
}

/// Completion condition specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionConditionSpec {
    CommandSuccess { command: String },
    FileExists { path: String },
    FileContains { path: String, pattern: String },
    All { conditions: Vec<CompletionConditionSpec> },
    Any { conditions: Vec<CompletionConditionSpec> },
    Custom { evaluator: String },
}