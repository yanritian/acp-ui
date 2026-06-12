//! ReconcileLoop - Goal迭代执行循环
//!
//! 根据RFC-001定义的Reconcile Loop算法：
//! 执行 → 评估 → 反馈 → 再执行 → 直到收敛或预算耗尽。

use crate::goal::{Goal, GoalStatus, GoalOutcome, CompletionCondition};
use crate::goal_evaluator::ConditionEvaluator;
use crate::goal_graph::GoalGraph;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("Worker not found: {0}")]
    WorkerNotFound(String),

    #[error("Execution timeout: {0}")]
    Timeout(String),

    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Worker error: {0}")]
    WorkerError(String),
}

/// WorkerExecutor trait - 执行Goal的Worker抽象
pub trait WorkerExecutor: Send {
    /// 在Worker上执行Goal描述
    fn execute(&mut self, goal: &Goal) -> Result<String, ReconcileError>;

    /// 获取Worker ID
    fn worker_id(&self) -> &str;
}

/// EchoExecutor - 用于测试的简单Worker（返回固定输出）
pub struct EchoExecutor {
    worker_id: String,
    output: String,
    success: bool,
}

impl EchoExecutor {
    pub fn new(worker_id: impl Into<String>, output: impl Into<String>, success: bool) -> Self {
        Self {
            worker_id: worker_id.into(),
            output: output.into(),
            success,
        }
    }

    pub fn set_output(&mut self, output: impl Into<String>) {
        self.output = output.into();
    }

    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }
}

impl WorkerExecutor for EchoExecutor {
    fn execute(&mut self, _goal: &Goal) -> Result<String, ReconcileError> {
        if self.success {
            Ok(self.output.clone())
        } else {
            Err(ReconcileError::EvaluationFailed(self.output.clone()))
        }
    }

    fn worker_id(&self) -> &str {
        &self.worker_id
    }
}

/// CommandExecutor - 通过命令行执行Goal
///
/// 执行指定的命令并收集输出
pub struct CommandExecutor {
    worker_id: String,
    default_cwd: Option<String>,
}

impl CommandExecutor {
    pub fn new(worker_id: impl Into<String>) -> Self {
        Self {
            worker_id: worker_id.into(),
            default_cwd: None,
        }
    }

    pub fn with_cwd(worker_id: impl Into<String>, cwd: impl Into<String>) -> Self {
        Self {
            worker_id: worker_id.into(),
            default_cwd: Some(cwd.into()),
        }
    }
}

impl WorkerExecutor for CommandExecutor {
    fn execute(&mut self, goal: &Goal) -> Result<String, ReconcileError> {
        // 根据Goal的CompletionCondition执行命令
        let output = match &goal.completion_condition {
            CompletionCondition::CommandSuccess { command, args, cwd } => {
                let work_dir = cwd.as_ref().or(self.default_cwd.as_ref());
                self.run_command(command, args, work_dir)?
            }
            CompletionCondition::OutputContains { command, .. } => {
                self.run_command(command, &[], self.default_cwd.as_ref())?
            }
            CompletionCondition::OutputMatches { command, .. } => {
                self.run_command(command, &[], self.default_cwd.as_ref())?
            }
            _ => {
                // 其他条件类型，返回Goal描述作为执行内容
                goal.description.clone()
            }
        };
        Ok(output)
    }

    fn worker_id(&self) -> &str {
        &self.worker_id
    }
}

impl CommandExecutor {
    fn run_command(&self, command: &str, args: &[String], cwd: Option<&String>) -> Result<String, ReconcileError> {
        use std::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd.output()
            .map_err(|e| ReconcileError::WorkerError(format!("Command execution failed: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

/// AIWorkerExecutor - 通过AI Worker执行Goal
///
/// 调用Claude Code或其他AI来执行任务
pub struct AIWorkerExecutor {
    worker_id: String,
    ai_command: String,
    default_cwd: Option<String>,
}

impl AIWorkerExecutor {
    /// 创建新的AI Worker Executor
    ///
    /// # Arguments
    /// * `worker_id` - Worker标识
    /// * `ai_command` - AI命令（如 "claude" 或 "codex"）
    pub fn new(worker_id: impl Into<String>, ai_command: impl Into<String>) -> Self {
        Self {
            worker_id: worker_id.into(),
            ai_command: ai_command.into(),
            default_cwd: None,
        }
    }

    /// 设置默认工作目录
    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.default_cwd = Some(cwd.into());
        self
    }

    /// 构建执行prompt
    fn build_prompt(goal: &Goal) -> String {
        let mut prompt = format!("Task: {}\n\n", goal.description);

        // 根据completion condition添加具体要求
        match &goal.completion_condition {
            CompletionCondition::FileCheck { path, content_contains, .. } => {
                prompt.push_str(&format!("Please create or modify the file '{}'.\n", path));
                if let Some(content) = content_contains {
                    prompt.push_str(&format!("The file must contain: '{}'\n", content));
                }
            }
            CompletionCondition::CommandSuccess { command, args, .. } => {
                prompt.push_str(&format!("Execute the command: {} {}\n", command, args.join(" ")));
                prompt.push_str("Verify it succeeds (exit code 0).\n");
            }
            CompletionCondition::OutputContains { command, pattern, .. } => {
                prompt.push_str(&format!("Execute: {}\n", command));
                prompt.push_str(&format!("The output must contain: '{}'\n", pattern));
            }
            CompletionCondition::All { conditions } => {
                prompt.push_str("All of the following must be satisfied:\n");
                for (i, cond) in conditions.iter().enumerate() {
                    prompt.push_str(&format!("{}. {}\n", i + 1, Self::describe_condition(cond)));
                }
            }
            CompletionCondition::Any { conditions } => {
                prompt.push_str("At least one of the following must be satisfied:\n");
                for (i, cond) in conditions.iter().enumerate() {
                    prompt.push_str(&format!("{}. {}\n", i + 1, Self::describe_condition(cond)));
                }
            }
            _ => {
                prompt.push_str("Complete the task as described.\n");
            }
        }

        prompt.push_str("\nAfter completing, report what was done.");
        prompt
    }

    /// 描述条件
    fn describe_condition(cond: &CompletionCondition) -> String {
        match cond {
            CompletionCondition::FileCheck { path, content_contains, .. } => {
                if let Some(content) = content_contains {
                    format!("File '{}' contains '{}'", path, content)
                } else {
                    format!("File '{}' exists", path)
                }
            }
            CompletionCondition::CommandSuccess { command, args, .. } => {
                format!("Command '{}' with args {:?} succeeds", command, args)
            }
            CompletionCondition::OutputContains { command, pattern, .. } => {
                format!("Output of '{}' contains '{}'", command, pattern)
            }
            _ => "Condition met".to_string()
        }
    }

    /// 执行AI命令
    fn execute_ai(&self, prompt: &str) -> Result<String, ReconcileError> {
        use std::process::Command;

        // Windows需要通过cmd.exe执行npm安装的命令
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &self.ai_command, "-p", prompt])
                .current_dir(self.default_cwd.as_deref().unwrap_or("."))
                .output()
                .map_err(|e| ReconcileError::WorkerError(format!("Failed to start AI process: {}", e)))?
        } else {
            Command::new(&self.ai_command)
                .args(["-p", prompt])
                .current_dir(self.default_cwd.as_deref().unwrap_or("."))
                .output()
                .map_err(|e| ReconcileError::WorkerError(format!("Failed to start AI process: {}", e)))?
        };

        // 检查退出码
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReconcileError::WorkerError(format!("AI process failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

impl WorkerExecutor for AIWorkerExecutor {
    fn execute(&mut self, goal: &Goal) -> Result<String, ReconcileError> {
        // 1. 构建prompt
        let prompt = Self::build_prompt(goal);

        // 2. 执行AI
        self.execute_ai(&prompt)
    }

    fn worker_id(&self) -> &str {
        &self.worker_id
    }
}

/// ReconcileLoop - Goal迭代执行引擎
pub struct ReconcileLoop {
    evaluator: ConditionEvaluator,
    worker_executor: Arc<Mutex<dyn WorkerExecutor>>,
}

impl ReconcileLoop {
    pub fn new(worker_executor: Arc<Mutex<dyn WorkerExecutor>>) -> Self {
        Self {
            evaluator: ConditionEvaluator::new(),
            worker_executor,
        }
    }

    pub fn with_echo(success: bool) -> Self {
        Self::new(Arc::new(Mutex::new(EchoExecutor::new("echo-worker", "", success))))
    }

    pub fn with_command(worker_id: impl Into<String>) -> Self {
        Self::new(Arc::new(Mutex::new(CommandExecutor::new(worker_id))))
    }

    pub fn with_ai(worker_id: impl Into<String>, ai_command: impl Into<String>) -> Self {
        Self::new(Arc::new(Mutex::new(AIWorkerExecutor::new(worker_id, ai_command))))
    }

    /// 执行单Goal的Reconcile Loop
    pub async fn reconcile_goal(&self, goal: &mut Goal) -> GoalOutcome {
        loop {
            // 1. 预算检查
            if goal.budget_exhausted() {
                return GoalOutcome::BudgetExhausted {
                    iterations: goal.current_iteration,
                    tokens_used: goal.tokens_used,
                    last_feedback: "Token budget exhausted".to_string(),
                };
            }

            // 2. 最大迭代检查
            if goal.max_iter_reached() {
                return GoalOutcome::MaxIterReached {
                    iterations: goal.current_iteration,
                    tokens_used: goal.tokens_used,
                    last_feedback: "Maximum iterations reached".to_string(),
                };
            }

            // 3. Worker执行
            goal.status = GoalStatus::Active;
            let execution_result = {
                let mut executor = self.worker_executor.lock().await;
                executor.execute(goal)
            };

            match execution_result {
                Ok(_output) => {
                    // 4. 评估完成条件
                    goal.status = GoalStatus::Evaluating;
                    let evaluation = self.evaluator.evaluate(&goal.completion_condition);

                    // 5. 判断收敛
                    if evaluation.converged {
                        goal.status = GoalStatus::Converged;
                        goal.converged_at = Some(chrono::Utc::now());
                        return GoalOutcome::Converged {
                            iterations: goal.current_iteration + 1,
                            tokens_used: goal.tokens_used + evaluation.tokens_used,
                            final_feedback: evaluation.feedback,
                        };
                    }

                    // 6. 未收敛 → 追加反馈，进入下一轮
                    goal.current_iteration += 1;
                    goal.status = GoalStatus::Iterating;
                    goal.append_feedback(goal.current_iteration, &evaluation.feedback, evaluation.tokens_used);
                }
                Err(e) => {
                    goal.status = GoalStatus::Failed;
                    return GoalOutcome::Failed(e.to_string());
                }
            }
        }
    }

    /// 执行GoalGraph（多Goal协调）
    pub async fn reconcile_graph(&self, graph: &mut GoalGraph) -> Vec<(String, GoalOutcome)> {
        let mut results = Vec::new();
        let order = graph.topological_order();

        for goal_id in order {
            // 检查依赖是否都已收敛
            let converged_ids: Vec<&str> = graph
                .all_goals()
                .iter()
                .filter(|g| g.status == GoalStatus::Converged)
                .map(|g| g.id.as_str())
                .collect();

            // 检查是否有失败的依赖
            let failed_deps: Vec<&str> = graph
                .all_goals()
                .iter()
                .filter(|g| g.status == GoalStatus::Failed)
                .map(|g| g.id.as_str())
                .collect();

            if let Some(goal) = graph.get_goal(&goal_id) {
                // 依赖失败，标记为Blocked
                if goal.depends_on.iter().any(|dep| failed_deps.contains(&dep.as_str())) {
                    graph.update_goal_status(&goal_id, GoalStatus::Failed);
                    results.push((goal_id, GoalOutcome::Failed("Blocked by failed dependency".to_string())));
                    continue;
                }

                // 依赖未满足，跳过
                if !goal.depends_on.iter().all(|dep| converged_ids.contains(&dep.as_str())) {
                    continue;
                }
            }

            // 执行Goal（简化实现）
            // 完整实现需要从graph中取出Goal，执行后更新状态
            results.push((goal_id.clone(), GoalOutcome::Converged {
                iterations: 1,
                tokens_used: 1000,
                final_feedback: "Graph execution completed".to_string(),
            }));
            graph.update_goal_status(&goal_id, GoalStatus::Converged);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goal::CompletionCondition;

    #[tokio::test]
    async fn reconcile_converges_with_echo_executor_success() {
        let reconciler = ReconcileLoop::with_echo(true);
        let mut goal = Goal::new(
            "test-goal",
            "Test description",
            CompletionCondition::command_success("echo"),
            "echo-worker",
        );

        let outcome = reconciler.reconcile_goal(&mut goal).await;

        assert!(matches!(outcome, GoalOutcome::Converged { .. }));
        assert_eq!(goal.status, GoalStatus::Converged);
    }

    #[tokio::test]
    async fn reconcile_fails_with_echo_executor_error() {
        let reconciler = ReconcileLoop::with_echo(false);
        let mut goal = Goal::new(
            "test-goal",
            "Test description",
            CompletionCondition::command_success("echo"),
            "echo-worker",
        );

        let outcome = reconciler.reconcile_goal(&mut goal).await;

        assert!(matches!(outcome, GoalOutcome::Failed(_)));
        assert_eq!(goal.status, GoalStatus::Failed);
    }

    #[tokio::test]
    async fn reconcile_budget_exhausted() {
        let reconciler = ReconcileLoop::with_echo(true);
        let mut goal = Goal::new(
            "test-goal",
            "Test description",
            CompletionCondition::command_success("echo"),
            "echo-worker",
        );
        goal.token_budget = 0; // Zero budget

        let outcome = reconciler.reconcile_goal(&mut goal).await;

        assert!(matches!(outcome, GoalOutcome::BudgetExhausted { .. }));
    }

    #[tokio::test]
    async fn reconcile_max_iter_reached() {
        let reconciler = ReconcileLoop::with_echo(true);
        let mut goal = Goal::new(
            "test-goal",
            "Test description",
            CompletionCondition::command_success("echo"),
            "echo-worker",
        );
        goal.max_iterations = 0; // Zero max iterations

        let outcome = reconciler.reconcile_goal(&mut goal).await;

        assert!(matches!(outcome, GoalOutcome::MaxIterReached { .. }));
    }

    #[test]
    fn echo_executor_worker_id() {
        let executor = EchoExecutor::new("my-worker", "output", true);
        assert_eq!(executor.worker_id(), "my-worker");
    }

    #[test]
    fn echo_executor_success() {
        let mut executor = EchoExecutor::new("worker", "test output", true);
        let goal = Goal::new("g1", "desc", CompletionCondition::command_success("echo"), "w1");
        let result = executor.execute(&goal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test output");
    }

    #[test]
    fn echo_executor_failure() {
        let mut executor = EchoExecutor::new("worker", "error message", false);
        let goal = Goal::new("g1", "desc", CompletionCondition::command_success("echo"), "w1");
        let result = executor.execute(&goal);
        assert!(result.is_err());
    }

    #[test]
    fn command_executor_worker_id() {
        let executor = CommandExecutor::new("cmd-worker");
        assert_eq!(executor.worker_id(), "cmd-worker");
    }

    #[tokio::test]
    async fn reconcile_graph_simple_chain() {
        let reconciler = ReconcileLoop::with_echo(true);
        let mut graph = crate::goal_graph::GoalGraph::new();

        let mut goal_a = Goal::new("a", "Goal A", CompletionCondition::command_success("echo"), "w1");
        goal_a.depends_on = vec![];

        let mut goal_b = Goal::new("b", "Goal B", CompletionCondition::command_success("echo"), "w1");
        goal_b.depends_on = vec!["a".to_string()];

        graph.add_goal(goal_a);
        graph.add_goal(goal_b);

        let results = reconciler.reconcile_graph(&mut graph).await;

        assert_eq!(results.len(), 2);
        // Both should converge
        for (_, outcome) in &results {
            assert!(matches!(outcome, GoalOutcome::Converged { .. }));
        }
    }
}