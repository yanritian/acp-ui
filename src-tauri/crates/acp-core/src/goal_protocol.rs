//! Goal Protocol - Goal definition and submission
//!
//! Unified Goal types for both src-tauri/src/ and swarm-engine crate.

use serde::{Deserialize, Serialize};

/// Goal specification (用于提交 Goal 的规格)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalSpec {
    /// Goal ID
    pub id: String,
    /// Goal description
    pub description: String,
    /// Completion condition
    pub completion_condition: CompletionConditionSpec,
    /// Evaluator type
    pub evaluator: EvaluatorSpec,
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

/// Evaluator specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvaluatorSpec {
    Auto,
    Queen { queen_worker_id: String },
    Adversarial { primary: String, adversary: String },
}

// ============================================================================
// 执行状态类型（用于 Goal 执行过程中）
// ============================================================================

/// Goal 执行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalStatus {
    Pending,
    Active,
    Evaluating,
    Converged,
    Iterating { feedback: String },
    Failed { reason: String },
    BudgetExhausted,
    MaxIterReached,
    Cancelled,
}

impl GoalStatus {
    /// 检查是否为终止状态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            GoalStatus::Converged
                | GoalStatus::Failed { .. }
                | GoalStatus::BudgetExhausted
                | GoalStatus::MaxIterReached
                | GoalStatus::Cancelled
        )
    }

    /// 检查是否为成功状态
    pub fn is_success(&self) -> bool {
        matches!(self, GoalStatus::Converged)
    }
}

/// 迭代记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    /// 迭代编号
    pub iteration: u32,
    /// Worker 输出
    pub worker_output: String,
    /// 评估结果
    pub evaluation: EvaluationResult,
    /// 时间戳（UNIX 毫秒）
    pub timestamp: u64,
    /// 消耗的 token
    pub tokens_used: u64,
}

/// 评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// 是否通过
    pub passed: bool,
    /// 解释说明
    pub explanation: String,
    /// 详细条件结果
    pub details: Vec<ConditionResult>,
}

/// 单个条件结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionResult {
    /// 条件描述
    pub description: String,
    /// 是否通过
    pub passed: bool,
    /// 证据/输出
    pub evidence: String,
}

/// Goal 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalOutcome {
    Converged { iterations: u32, tokens_used: u64 },
    Failed { reason: String, iterations: u32 },
    BudgetExhausted { tokens_used: u64 },
    MaxIterReached { iterations: u32 },
    Cancelled,
}

/// Goal 图摘要
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

// ============================================================================
// AcpEvent - Event Protocol 实现
// ============================================================================

/// ACP 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpEvent {
    /// 事件类型
    pub event_type: String,
    /// Goal ID
    pub goal_id: Option<String>,
    /// Worker ID
    pub worker_id: Option<String>,
    /// 事件数据
    pub data: serde_json::Value,
    /// 时间戳
    pub timestamp: u64,
}

impl AcpEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            goal_id: None,
            worker_id: None,
            data: serde_json::Value::Null,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    pub fn with_goal(mut self, goal_id: impl Into<String>) -> Self {
        self.goal_id = Some(goal_id.into());
        self
    }

    pub fn with_worker(mut self, worker_id: impl Into<String>) -> Self {
        self.worker_id = Some(worker_id.into());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }
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
            evaluator: EvaluatorSpec::Auto,
            executor: Some("worker-001".to_string()),
            depends_on: vec![],
            token_budget: 50_000,
            max_iterations: 5,
        };

        assert_eq!(spec.id, "goal-001");
        assert_eq!(spec.token_budget, 50_000);
    }

    #[test]
    fn goal_status_is_terminal() {
        assert!(GoalStatus::Converged.is_terminal());
        assert!(GoalStatus::Failed { reason: "test".into() }.is_terminal());
        assert!(!GoalStatus::Pending.is_terminal());
        assert!(!GoalStatus::Active.is_terminal());
    }

    #[test]
    fn goal_status_is_success() {
        assert!(GoalStatus::Converged.is_success());
        assert!(!GoalStatus::Failed { reason: "test".into() }.is_success());
    }

    #[test]
    fn acp_event_builder() {
        let event = AcpEvent::new("goal.created")
            .with_goal("goal-001")
            .with_data(serde_json::json!({"description": "Test"}));

        assert_eq!(event.event_type, "goal.created");
        assert_eq!(event.goal_id, Some("goal-001".to_string()));
    }

    #[test]
    fn goal_spec_serde_roundtrip() {
        let spec = GoalSpec {
            id: "goal-002".to_string(),
            description: "Run tests".to_string(),
            completion_condition: CompletionConditionSpec::CommandSuccess {
                command: "npm test".to_string(),
            },
            evaluator: EvaluatorSpec::Auto,
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
    fn evaluator_spec_serde() {
        let eval = EvaluatorSpec::Queen { queen_worker_id: "queen-001".into() };
        let json = serde_json::to_string(&eval).unwrap();
        let decoded: EvaluatorSpec = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, EvaluatorSpec::Queen { .. }));
    }

    #[test]
    fn iteration_record() {
        let record = IterationRecord {
            iteration: 1,
            worker_output: "output".into(),
            evaluation: EvaluationResult {
                passed: true,
                explanation: "success".into(),
                details: vec![],
            },
            timestamp: 1000,
            tokens_used: 100,
        };

        let json = serde_json::to_string(&record).unwrap();
        let decoded: IterationRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.iteration, 1);
    }

    #[test]
    fn goal_outcome_converged() {
        let outcome = GoalOutcome::Converged { iterations: 3, tokens_used: 1000 };
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(json.contains("Converged"));
    }

    #[test]
    fn goal_graph_summary_default() {
        let summary = GoalGraphSummary::default();
        assert_eq!(summary.total, 0);
        assert_eq!(summary.pending, 0);
    }
}