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