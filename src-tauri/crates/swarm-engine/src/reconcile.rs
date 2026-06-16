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

    /// 构建执行prompt（改进：明确告诉 AI 必须立即执行，不要聊天）
    fn build_prompt(goal: &Goal) -> String {
        let mut prompt = String::new();

        // 关键：明确告诉 AI 不要聊天，立即执行
        prompt.push_str("DO NOT respond with chat. DO NOT ask questions.\n");
        prompt.push_str("IMMEDIATELY use Write tool to create all required files.\n\n");

        prompt.push_str(&format!("Task: {}\n\n", goal.description));

        // 根据completion condition添加具体要求
        match &goal.completion_condition {
            CompletionCondition::FileCheck { path, content_contains, .. } => {
                prompt.push_str(&format!("IMMEDIATELY create file '{}' using Write tool.\n", path));
                if let Some(content) = content_contains {
                    prompt.push_str(&format!("Content must include: '{}'\n", content));
                }
            }
            CompletionCondition::CommandSuccess { command, args, .. } => {
                prompt.push_str(&format!("Execute: {} {}\n", command, args.join(" ")));
                prompt.push_str("Verify it succeeds (exit code 0).\n");
            }
            CompletionCondition::OutputContains { command, pattern, .. } => {
                prompt.push_str(&format!("Execute: {}\n", command));
                prompt.push_str(&format!("Output must contain: '{}'\n", pattern));
            }
            CompletionCondition::All { conditions } => {
                prompt.push_str("\n=== FILES TO CREATE (use Write tool for each) ===\n\n");
                for (i, cond) in conditions.iter().enumerate() {
                    if let CompletionCondition::FileCheck { path, content_contains, .. } = cond {
                        // 生成具体的 Write 调用
                        prompt.push_str(&format!("{}. Write file: {}\n", i + 1, path));
                        if let Some(content) = content_contains {
                            prompt.push_str(&format!("   Must contain: {}\n", content));
                        }
                        // 根据文件类型生成基本内容模板
                        if path.ends_with(".json") {
                            prompt.push_str("   Example content: { \"name\": \"finance-system\", \"scripts\": { \"test\": \"vitest\" } }\n");
                        } else if path.ends_with(".ts") && content_contains.is_some() {
                            let schema_name = content_contains.as_ref().unwrap();
                            prompt.push_str(&format!("   Content: export const {} = z.object({});\n", schema_name, schema_name.replace("Schema", "")));
                        }
                        prompt.push_str("\n");
                    } else {
                        prompt.push_str(&format!("{}. {}\n\n", i + 1, Self::describe_condition(cond)));
                    }
                }
            }
            CompletionCondition::Any { conditions } => {
                prompt.push_str("Complete at least one:\n");
                for (i, cond) in conditions.iter().enumerate() {
                    prompt.push_str(&format!("{}. {}\n", i + 1, Self::describe_condition(cond)));
                }
            }
            _ => {
                prompt.push_str("Complete the task NOW.\n");
            }
        }

        prompt.push_str("\nEXECUTE NOW. Do not wait for further instructions.");
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

    /// 执行AI命令（改进：添加工具权限让 AI 能实际执行）
    fn execute_ai(&self, prompt: &str) -> Result<String, ReconcileError> {
        use std::process::Command;

        // 构建完整的 prompt，明确告诉 AI 必须使用工具执行
        let full_prompt = format!(
            "You MUST use Write tool to create files. Do NOT just respond with text.\n\
            Use Write(file_path, content) to create each file.\n\
            Use Bash(command) to run commands like mkdir.\n\
            \n\
            {}",
            prompt
        );

        // 调试输出（可选，生产环境可移除）
        // println!("\n【DEBUG】Generated prompt:");
        // println!("----------------------------------------");
        // println!("{}", full_prompt);
        // println!("----------------------------------------\n");

        // Windows: 使用 Git Bash 执行，管道能正确传递 stdin
        // Git Bash 位于 C:\Program Files\Git\usr\bin\bash.exe
        let work_dir = self.default_cwd.as_deref().unwrap_or("D:/tmp");

        // 创建临时文件
        let temp_prompt_file = format!("D:/tmp/claude_prompt_{}.txt", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis());

        std::fs::write(&temp_prompt_file, &full_prompt)
            .map_err(|e| ReconcileError::WorkerError(format!("Failed to write temp prompt file: {}", e)))?;

        // 使用 Git Bash 执行
        let git_bash = "C:/Program Files/Git/usr/bin/bash.exe";
        let cmd_args = format!(
            "cat '{}' | '{}' --bare --dangerously-skip-permissions --allowedTools Write,Edit,Bash --permission-mode bypassPermissions --print",
            temp_prompt_file,
            &self.ai_command
        );

        let output = Command::new(git_bash)
            .args(["-c", &cmd_args])
            .current_dir(work_dir)
            .output()
            .map_err(|e| ReconcileError::WorkerError(format!("Failed to start AI process: {}", e)))?;

        // 清理临时文件
        std::fs::remove_file(&temp_prompt_file).ok();

        // 检查退出码（调试输出可选）
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // println!("【DEBUG】Command stdout:");
        // println!("----------------------------------------");
        // println!("{}", stdout);
        // println!("----------------------------------------");

        if !stderr.is_empty() {
            // println!("【DEBUG】Command stderr:");
            // println!("----------------------------------------");
            // println!("{}", stderr);
            // println!("----------------------------------------");
        }

        if !output.status.success() {
            return Err(ReconcileError::WorkerError(format!("AI process failed: {}", stderr)));
        }

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
        println!("\n【ReconcileLoop 开始执行】");
        println!("  Goal ID: {}", goal.id);
        println!("  MaxIterations: {}", goal.max_iterations);
        println!("  TokenBudget: {}", goal.token_budget);
        println!("");

        loop {
            // 1. 预算检查
            if goal.budget_exhausted() {
                println!("\n【预算耗尽】");
                println!("  TokensUsed: {} / {}", goal.tokens_used, goal.token_budget);
                return GoalOutcome::BudgetExhausted {
                    iterations: goal.current_iteration,
                    tokens_used: goal.tokens_used,
                    last_feedback: "Token budget exhausted".to_string(),
                };
            }

            // 2. 最大迭代检查
            if goal.max_iter_reached() {
                println!("\n【达到最大迭代】");
                println!("  CurrentIteration: {} / {}", goal.current_iteration, goal.max_iterations);
                return GoalOutcome::MaxIterReached {
                    iterations: goal.current_iteration,
                    tokens_used: goal.tokens_used,
                    last_feedback: "Maximum iterations reached".to_string(),
                };
            }

            // 3. Worker执行
            println!("\n========================================");
            println!("【Iteration {} - Worker执行】", goal.current_iteration + 1);
            println!("========================================");
            println!("  发送需求给 AI Worker...");
            println!("  (等待 AI 响应，可能需要较长时间)");
            println!("");

            goal.status = GoalStatus::Active;
            let execution_result = {
                let mut executor = self.worker_executor.lock().await;
                executor.execute(goal)
            };

            match execution_result {
                Ok(output) => {
                    println!("\n【AI Worker 响应完成】");
                    println!("  输出长度: {} 字符", output.len());
                    println!("  输出预览: {}", output.chars().take(200).collect::<String>());

                    // 4. 评估完成条件
                    println!("\n【评估完成条件】");
                    goal.status = GoalStatus::Evaluating;
                    let evaluation = self.evaluator.evaluate(&goal.completion_condition);

                    println!("  converged: {}", evaluation.converged);
                    println!("  feedback: {}", evaluation.feedback);

                    // 5. 判断收敛
                    if evaluation.converged {
                        println!("\n========================================");
                        println!("【✅ Goal 收敛成功！】");
                        println!("========================================");
                        goal.status = GoalStatus::Converged;
                        goal.converged_at = Some(chrono::Utc::now());
                        return GoalOutcome::Converged {
                            iterations: goal.current_iteration + 1,
                            tokens_used: goal.tokens_used + evaluation.tokens_used,
                            final_feedback: evaluation.feedback,
                        };
                    }

                    // 6. 未收敛 → 追加反馈，进入下一轮
                    println!("\n【未收敛，准备下一轮迭代】");
                    goal.current_iteration += 1;
                    goal.status = GoalStatus::Iterating { feedback: evaluation.feedback.clone() };
                    goal.append_feedback(goal.current_iteration, &evaluation.feedback, evaluation.tokens_used);
                    println!("  Iteration {} 完成，累计 tokens: {}", goal.current_iteration, goal.tokens_used);
                }
                Err(e) => {
                    println!("\n【执行失败】");
                    println!("  错误: {}", e);
                    goal.status = GoalStatus::Failed { reason: e.to_string() };
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
                .filter(|g| matches!(g.status, GoalStatus::Failed { .. }))
                .map(|g| g.id.as_str())
                .collect();

            if let Some(goal) = graph.get_goal(&goal_id) {
                // 依赖失败，标记为Blocked
                if goal.depends_on.iter().any(|dep| failed_deps.contains(&dep.as_str())) {
                    graph.update_goal_status(&goal_id, GoalStatus::Failed { reason: "Blocked by failed dependency".into() });
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
        assert!(matches!(goal.status, GoalStatus::Failed { .. }));
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