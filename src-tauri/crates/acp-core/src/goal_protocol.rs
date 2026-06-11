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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_spec_new() {
        let spec = GoalSpec {
            id: "goal-001".to_string(),
            description: "Fix compilation errors".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "cargo build".to_string(),
            },
            evaluator: "auto".to_string(),
            executor: Some("worker-001".to_string()),
            depends_on: vec![],
            token_budget: 50_000,
            max_iterations: 5,
        };

        assert_eq!(spec.id, "goal-001");
        assert_eq!(spec.token_budget, 50_000);
    }

    #[test]
    fn goal_spec_serde_roundtrip() {
        let spec = GoalSpec {
            id: "goal-002".to_string(),
            description: "Run tests".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "npm test".to_string(),
            },
            evaluator: "auto".to_string(),
            executor: None,
            depends_on: vec!["goal-001".to_string()],
            token_budget: 30_000,
            max_iterations: 3,
        };

        let json = serde_json::to_string(&spec).unwrap();
        let decoded: GoalSpec = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, spec.id);
        assert_eq!(decoded.depends_on.len(), 1);
    }

    #[test]
    fn completion_condition_command_success() {
        let cc = CompletionConditionSpec::CommandSuccess {
            command: "echo hello".to_string(),
        };

        let json = serde_json::to_string(&cc).unwrap();
        assert!(json.contains("CommandSuccess"));
        assert!(json.contains("echo hello"));

        let decoded: CompletionConditionSpec = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, CompletionConditionSpec::CommandSuccess { .. }));
    }

    #[test]
    fn completion_condition_file_exists() {
        let cc = CompletionConditionSpec::FileExists {
            path: "README.md".to_string(),
        };

        let json = serde_json::to_string(&cc).unwrap();
        let decoded: CompletionConditionSpec = serde_json::from_str(&json).unwrap();

        assert!(matches!(decoded, CompletionConditionSpec::FileExists { .. }));
    }

    #[test]
    fn completion_condition_all() {
        let cc = CompletionConditionSpec::All {
            conditions: vec![
                CompletionConditionSpec::CommandSuccess { command: "cargo test".to_string() },
                CompletionConditionSpec::FileExists { path: "Cargo.toml".to_string() },
            ],
        };

        let json = serde_json::to_string(&cc).unwrap();
        let decoded: CompletionConditionSpec = serde_json::from_str(&json).unwrap();

        if let CompletionConditionSpec::All { conditions } = decoded {
            assert_eq!(conditions.len(), 2);
        } else {
            panic!("Expected All variant");
        }
    }

    #[test]
    fn completion_condition_any() {
        let cc = CompletionConditionSpec::Any {
            conditions: vec![
                CompletionConditionSpec::CommandSuccess { command: "eslint".to_string() },
                CompletionConditionSpec::CommandSuccess { command: "prettier --check".to_string() },
            ],
        };

        let json = serde_json::to_string(&cc).unwrap();
        let decoded: CompletionConditionSpec = serde_json::from_str(&json).unwrap();

        assert!(matches!(decoded, CompletionConditionSpec::Any { .. }));
    }
}