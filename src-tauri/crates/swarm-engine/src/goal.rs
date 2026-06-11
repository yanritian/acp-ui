//! Goal - 根据RFC-001定义的Goal数据结构
//!
//! Goal是蜂群编排的核心抽象：包含明确的完成条件和独立的评估器。
//! Worker不是一次性执行，而是进入Reconcile Loop——执行、评估、反馈、再执行——直到完成条件满足。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Goal —— 蜂群编排的核心抽象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    // === 身份 ===
    /// Goal唯一标识
    pub id: String,
    /// Goal描述
    pub description: String,
    /// 所属任务ID
    pub parent_task_id: String,
    /// 父Goal ID（用于Sub-Goal）
    pub parent_goal_id: Option<String>,

    // === 完成条件 ===
    /// 完成条件（8种类型）
    pub completion_condition: CompletionCondition,

    // === 角色分配 ===
    /// 评估器类型
    pub evaluator: Evaluator,
    /// 执行器Worker ID
    pub executor: String,

    // === 依赖关系 ===
    /// 依赖的Goal ID列表
    pub depends_on: Vec<String>,

    // === 预算与限制 ===
    /// Token预算上限
    pub token_budget: u64,
    /// 已消耗Token数
    pub tokens_used: u64,
    /// 最大迭代次数
    pub max_iterations: u32,
    /// 当前迭代次数
    pub current_iteration: u32,
    /// 单次迭代超时（毫秒）
    pub per_iteration_timeout_ms: u64,

    // === 状态 ===
    /// Goal状态
    pub status: GoalStatus,
    /// 迭代记录日志
    pub iteration_log: Vec<IterationRecord>,

    // === 元数据 ===
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 收敛时间
    pub converged_at: Option<DateTime<Utc>>,
    /// 输出文件列表
    pub output_files: Vec<String>,
}

impl Goal {
    /// 创建新Goal
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        completion_condition: CompletionCondition,
        executor: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            parent_task_id: String::new(),
            parent_goal_id: None,
            completion_condition,
            evaluator: Evaluator::Auto,
            executor: executor.into(),
            depends_on: Vec::new(),
            token_budget: 100_000,
            tokens_used: 0,
            max_iterations: 5,
            current_iteration: 0,
            per_iteration_timeout_ms: 60_000,
            status: GoalStatus::Pending,
            iteration_log: Vec::new(),
            created_at: Utc::now(),
            converged_at: None,
            output_files: Vec::new(),
        }
    }

    /// 检查是否可以开始执行（无依赖或所有依赖已收敛）
    pub fn is_ready(&self, converged_goals: &[&str]) -> bool {
        self.status == GoalStatus::Pending
            && self.depends_on.iter().all(|dep| converged_goals.contains(&dep.as_str()))
    }

    /// 检查预算是否耗尽
    pub fn budget_exhausted(&self) -> bool {
        self.tokens_used >= self.token_budget
    }

    /// 检查是否达到最大迭代次数
    pub fn max_iter_reached(&self) -> bool {
        self.current_iteration >= self.max_iterations
    }

    /// 追加迭代反馈
    pub fn append_feedback(&mut self, iteration: u32, feedback: &str, tokens: u64) {
        self.iteration_log.push(IterationRecord {
            iteration,
            started_at: Utc::now(),
            finished_at: Utc::now(),
            execution_output: None,
            evaluation_result: EvaluationResult {
                converged: false,
                feedback: feedback.to_string(),
                tokens_used: tokens,
            },
        });
        self.tokens_used += tokens;
    }
}

/// CompletionCondition（完成条件）—— 8种类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionCondition {
    /// 命令退出码为0
    CommandSuccess {
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
    },
    /// 输出包含文本
    OutputContains {
        command: String,
        pattern: String,
        case_sensitive: bool,
    },
    /// 输出匹配正则
    OutputMatches {
        command: String,
        regex: String,
    },
    /// 文件存在/内容匹配
    FileCheck {
        path: String,
        content_contains: Option<String>,
        max_size_bytes: Option<u64>,
    },
    /// HTTP端点返回预期状态
    HttpHealthCheck {
        url: String,
        method: String,
        expected_status: Option<u16>,
    },
    /// 所有子条件满足（AND）
    All {
        conditions: Vec<CompletionCondition>,
    },
    /// 任一子条件满足（OR）
    Any {
        conditions: Vec<CompletionCondition>,
    },
    /// Queen主观判断
    QueenJudgment {
        criteria: String,
    },
}

impl CompletionCondition {
    /// 创建命令成功条件
    pub fn command_success(command: impl Into<String>) -> Self {
        Self::CommandSuccess {
            command: command.into(),
            args: Vec::new(),
            cwd: None,
        }
    }

    /// 创建文件存在条件
    pub fn file_exists(path: impl Into<String>) -> Self {
        Self::FileCheck {
            path: path.into(),
            content_contains: None,
            max_size_bytes: None,
        }
    }
}

/// Evaluator（评估器）—— 4种类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Evaluator {
    /// 自动运行CompletionCondition
    Auto,
    /// Queen Worker主观判断
    Queen,
    /// 另一个Worker评估
    Adversarial {
        evaluator_worker: String,
    },
    /// 先Auto再Human
    Hybrid {
        auto_condition: CompletionCondition,
        human_evaluator: Box<Evaluator>,
    },
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::Auto
    }
}

/// GoalStatus（状态机）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    /// 等待执行
    Pending,
    /// 正在执行
    Active,
    /// 正在评估
    Evaluating,
    /// 已收敛（成功）
    Converged,
    /// 未收敛，继续迭代
    Iterating,
    /// Token预算耗尽
    BudgetExhausted,
    /// 达到最大迭代次数
    MaxIterReached,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

impl GoalStatus {
    /// 是否为终态（不可继续执行）
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Converged
                | Self::BudgetExhausted
                | Self::MaxIterReached
                | Self::Failed
                | Self::Cancelled
        )
    }

    /// 是否成功
    pub fn is_success(&self) -> bool {
        *self == Self::Converged
    }
}

/// IterationRecord（迭代记录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    /// 迭代编号
    pub iteration: u32,
    /// 开始时间
    pub started_at: DateTime<Utc>,
    /// 结束时间
    pub finished_at: DateTime<Utc>,
    /// 执行输出
    pub execution_output: Option<String>,
    /// 评估结果
    pub evaluation_result: EvaluationResult,
}

/// EvaluationResult（评估结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// 是否收敛
    pub converged: bool,
    /// 反馈信息
    pub feedback: String,
    /// 本次消耗Token数
    pub tokens_used: u64,
}

/// GoalOutcome（Goal执行结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalOutcome {
    /// 收敛成功
    Converged {
        iterations: u32,
        tokens_used: u64,
        final_feedback: String,
    },
    /// 预算耗尽
    BudgetExhausted {
        iterations: u32,
        tokens_used: u64,
        last_feedback: String,
    },
    /// 达到最大迭代
    MaxIterReached {
        iterations: u32,
        tokens_used: u64,
        last_feedback: String,
    },
    /// 失败
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_new_defaults() {
        let goal = Goal::new(
            "test-goal",
            "Test description",
            CompletionCondition::command_success("echo"),
            "worker-001",
        );

        assert_eq!(goal.id, "test-goal");
        assert_eq!(goal.description, "Test description");
        assert_eq!(goal.executor, "worker-001");
        assert_eq!(goal.status, GoalStatus::Pending);
        assert_eq!(goal.current_iteration, 0);
        assert_eq!(goal.tokens_used, 0);
        assert_eq!(goal.token_budget, 100_000);
        assert_eq!(goal.max_iterations, 5);
        assert!(goal.depends_on.is_empty());
        assert!(goal.iteration_log.is_empty());
        assert!(goal.converged_at.is_none());
    }

    #[test]
    fn goal_status_is_terminal() {
        // Terminal states
        assert!(GoalStatus::Converged.is_terminal());
        assert!(GoalStatus::Failed.is_terminal());
        assert!(GoalStatus::Cancelled.is_terminal());
        assert!(GoalStatus::BudgetExhausted.is_terminal());
        assert!(GoalStatus::MaxIterReached.is_terminal());

        // Non-terminal states
        assert!(!GoalStatus::Pending.is_terminal());
        assert!(!GoalStatus::Active.is_terminal());
        assert!(!GoalStatus::Evaluating.is_terminal());
        assert!(!GoalStatus::Iterating.is_terminal());
    }

    #[test]
    fn goal_status_is_success() {
        assert!(GoalStatus::Converged.is_success());
        assert!(!GoalStatus::Failed.is_success());
        assert!(!GoalStatus::Pending.is_success());
        assert!(!GoalStatus::Active.is_success());
    }

    #[test]
    fn goal_budget_exhausted() {
        let mut goal = Goal::new("g1", "desc", CompletionCondition::command_success("echo"), "w1");
        goal.token_budget = 1000;

        assert!(!goal.budget_exhausted());

        goal.tokens_used = 500;
        assert!(!goal.budget_exhausted());

        goal.tokens_used = 1000;
        assert!(goal.budget_exhausted());

        goal.tokens_used = 1500;
        assert!(goal.budget_exhausted());
    }

    #[test]
    fn goal_max_iter_reached() {
        let mut goal = Goal::new("g1", "desc", CompletionCondition::command_success("echo"), "w1");
        goal.max_iterations = 3;

        assert!(!goal.max_iter_reached());

        goal.current_iteration = 2;
        assert!(!goal.max_iter_reached());

        goal.current_iteration = 3;
        assert!(goal.max_iter_reached());
    }

    #[test]
    fn goal_append_feedback() {
        let mut goal = Goal::new("g1", "desc", CompletionCondition::command_success("echo"), "w1");
        assert!(goal.iteration_log.is_empty());

        goal.append_feedback(1, "First iteration feedback", 500);

        assert_eq!(goal.iteration_log.len(), 1);
        assert_eq!(goal.iteration_log[0].iteration, 1);
        assert_eq!(goal.iteration_log[0].evaluation_result.feedback, "First iteration feedback");
        assert_eq!(goal.iteration_log[0].evaluation_result.tokens_used, 500);
        assert_eq!(goal.tokens_used, 500);

        goal.append_feedback(2, "Second iteration feedback", 300);

        assert_eq!(goal.iteration_log.len(), 2);
        assert_eq!(goal.tokens_used, 800);
    }

    #[test]
    fn goal_is_ready_no_deps() {
        let goal = Goal::new("g1", "desc", CompletionCondition::command_success("echo"), "w1");

        assert!(goal.is_ready(&[]));
    }

    #[test]
    fn goal_is_ready_with_deps() {
        let mut goal = Goal::new("g1", "desc", CompletionCondition::command_success("echo"), "w1");
        goal.depends_on = vec!["goal-a".to_string(), "goal-b".to_string()];
        goal.status = GoalStatus::Pending;

        // No deps converged -> not ready
        assert!(!goal.is_ready(&[]));

        // Partial deps converged -> not ready
        assert!(!goal.is_ready(&["goal-a"]));

        // All deps converged -> ready
        assert!(goal.is_ready(&["goal-a", "goal-b"]));

        // Wrong status -> not ready
        goal.status = GoalStatus::Active;
        assert!(!goal.is_ready(&["goal-a", "goal-b"]));
    }

    #[test]
    fn completion_condition_command_success() {
        let cc = CompletionCondition::command_success("npm test");
        assert!(matches!(cc, CompletionCondition::CommandSuccess { .. }));
    }

    #[test]
    fn completion_condition_file_exists() {
        let cc = CompletionCondition::file_exists("README.md");
        assert!(matches!(cc, CompletionCondition::FileCheck { .. }));
    }
}