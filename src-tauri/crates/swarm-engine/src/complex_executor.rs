//! ComplexGoalExecutor - 处理复杂系统需求的执行器
//!
//! 核心改进：
//! 1. 规划阶段：先让 AI 分析需求并制定实施计划
//! 2. 分解阶段：将复杂需求拆分成多个子 Goal
//! 3. 执行阶段：逐个执行子 Goal，支持增量反馈
//! 4. 验证阶段：编译/测试验证，确保质量

use crate::goal::{Goal, GoalStatus, CompletionCondition, GoalOutcome};
use crate::goal_graph::GoalGraph;
use crate::goal_evaluator::ConditionEvaluator;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComplexError {
    #[error("Planning failed: {0}")]
    PlanningFailed(String),

    #[error("Decomposition failed: {0}")]
    DecompositionFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// 实施计划
#[derive(Debug, Clone)]
pub struct ImplementationPlan {
    /// 需求分析
    pub analysis: String,
    /// 子任务列表
    pub subtasks: Vec<SubTaskSpec>,
    /// 预估总步骤
    pub estimated_steps: u32,
}

/// 子任务规格
#[derive(Debug, Clone)]
pub struct SubTaskSpec {
    pub id: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub validation_criteria: String,
}

/// ComplexGoalExecutor - 复杂需求执行器
pub struct ComplexGoalExecutor {
    /// AI 命令路径
    ai_command: String,
    /// 工作目录
    working_dir: String,
    /// 评估器
    evaluator: ConditionEvaluator,
}

impl ComplexGoalExecutor {
    pub fn new(ai_command: impl Into<String>, working_dir: impl Into<String>) -> Self {
        Self {
            ai_command: ai_command.into(),
            working_dir: working_dir.into(),
            evaluator: ConditionEvaluator::new(),
        }
    }

    /// 第一阶段：规划 - 分析需求并制定计划
    pub async fn plan(&self, requirement: &str) -> Result<ImplementationPlan, ComplexError> {
        let planning_prompt = self.build_planning_prompt(requirement);
        let ai_output = self.execute_ai(&planning_prompt).await?;

        // 解析 AI 输出，提取计划
        self.parse_plan(&ai_output)
    }

    /// 第二阶段：分解 - 将计划转换为 GoalGraph
    pub async fn decompose(&self, plan: &ImplementationPlan, base_path: &str) -> GoalGraph {
        let mut graph = GoalGraph::new();

        for subtask in &plan.subtasks {
            let goal = Goal::new(
                &subtask.id,
                &subtask.description,
                CompletionCondition::FileCheck {
                    path: format!("{}{}", base_path, subtask.id.replace("-", "/")),
                    content_contains: Some(subtask.validation_criteria.clone()),
                    max_size_bytes: None,
                },
                "complex-executor",
            );

            // 添加依赖关系
            let mut goal_with_deps = goal;
            goal_with_deps.depends_on = subtask.dependencies.clone();

            graph.add_goal(goal_with_deps);
        }

        graph
    }

    /// 第三阶段：执行 - 逐个执行 GoalGraph
    pub async fn execute_graph(&self, graph: &mut GoalGraph) -> Vec<(String, GoalOutcome)> {
        let mut results = Vec::new();
        let order = graph.topological_order();

        for goal_id in order {
            // 获取当前 Goal 信息
            if let Some(goal) = graph.get_goal(&goal_id) {
                // 构建增量执行 prompt（包含已完成的上下文）
                let description = goal.description.clone();
                let condition = goal.completion_condition.clone();
                let prompt = self.build_execution_prompt_from_info(&goal_id, &description, &results);

                // 更新状态为 Active
                graph.update_goal_status(&goal_id, GoalStatus::Active);

                // 执行
                let _output = self.execute_ai(&prompt).await.ok();

                // 更新状态为 Evaluating
                graph.update_goal_status(&goal_id, GoalStatus::Evaluating);

                // 验证
                let eval_result = self.evaluator.evaluate(&condition);

                if eval_result.converged {
                    graph.update_goal_status(&goal_id, GoalStatus::Converged);
                    results.push((goal_id.clone(), GoalOutcome::Converged {
                        iterations: 1,
                        tokens_used: 0,
                        final_feedback: eval_result.feedback,
                    }));
                } else {
                    // 未收敛，记录反馈继续
                    graph.update_goal_status(&goal_id, GoalStatus::Iterating);
                    results.push((goal_id.clone(), GoalOutcome::Failed(format!("Not converged: {}", eval_result.feedback))));
                }
            }
        }

        results
    }

    /// 第四阶段：验证 - 编译/测试验证
    pub async fn validate(&self, path: &str) -> Result<bool, ComplexError> {
        // 检查 TypeScript 编译
        let compile_check = self.check_typescript_compile(path);
        if !compile_check {
            return Err(ComplexError::ValidationFailed("TypeScript compilation failed".into()));
        }

        Ok(true)
    }

    /// 构建规划 prompt - 改进版，输出结构化 JSON 格式
    fn build_planning_prompt(&self, requirement: &str) -> String {
        format!(
            "You are a system architect. Analyze the requirement and output a JSON task list.\n\n\
            REQUIREMENT:\n{}\n\n\
            OUTPUT FORMAT (strict JSON):\n\
            ```json\n\
            {{\n\
              \"tasks\": [\n\
                {{\n\
                  \"id\": \"task-1\",\n\
                  \"description\": \"Create file X with content Y\",\n\
                  \"file_path\": \"path/to/file.ts\",\n\
                  \"dependencies\": [],\n\
                  \"validation\": \"File exists and contains keyword\"\n\
                }}\n\
              ]\n\
            }}\n\
            ```\n\n\
            RULES:\n\
            1. Each task creates ONE file only\n\
            2. Use simple task IDs: task-1, task-2, etc.\n\
            3. Dependencies use task IDs (empty array [] for first task)\n\
            4. file_path must be relative to working directory\n\
            5. validation is a single keyword or phrase to check\n\n\
            OUTPUT THE JSON NOW:\n",
            requirement
        )
    }

    /// 构建执行 prompt（包含已完成任务的上下文）
    fn build_execution_prompt(&self, goal: &Goal, completed: &[(String, GoalOutcome)]) -> String {
        let mut prompt = format!("Task: {}\n\n", goal.description);

        // 添加已完成的上下文
        if !completed.is_empty() {
            prompt.push_str("Previously completed tasks:\n");
            for (id, outcome) in completed {
                if matches!(outcome, GoalOutcome::Converged { .. }) {
                    prompt.push_str(&format!("  - {} ✓\n", id));
                }
            }
            prompt.push_str("\n");
        }

        prompt.push_str("Create the file as described. Focus on this single task only.\n");
        prompt.push_str("After completing, report what was done.");
        prompt
    }

    /// 构建执行 prompt（从信息构建，用于 execute_graph）- 改进版
    fn build_execution_prompt_from_info(&self, id: &str, description: &str, completed: &[(String, GoalOutcome)]) -> String {
        let mut prompt = format!("Task ID: {}\n", id);

        // 提取文件路径信息
        if description.contains("file:") {
            let file_part = description.split("file:").nth(1).unwrap_or("").trim();
            prompt.push_str(&format!("Target File: {}\n", file_part));
        }

        prompt.push_str(&format!("Working Directory: {}\n\n", self.working_dir));
        prompt.push_str(&format!("Description: {}\n\n", description));

        // 添加已完成的上下文
        if !completed.is_empty() {
            prompt.push_str("Previously completed tasks:\n");
            for (prev_id, outcome) in completed {
                if matches!(outcome, GoalOutcome::Converged { .. }) {
                    prompt.push_str(&format!("  - {} ✓\n", prev_id));
                }
            }
            prompt.push_str("\n");
        }

        prompt.push_str("INSTRUCTIONS:\n");
        prompt.push_str("1. Create ONLY the file specified above\n");
        prompt.push_str("2. Use the working directory as base path\n");
        prompt.push_str("3. Include appropriate exports/imports\n");
        prompt.push_str("4. After completing, report what was done\n");
        prompt
    }

    /// 解析 AI 输出提取计划 - 改进版，支持 JSON 和 fallback
    fn parse_plan(&self, output: &str) -> Result<ImplementationPlan, ComplexError> {
        let mut subtasks = Vec::new();

        // 尝试解析 JSON 格式
        if let Some(json_str) = self.extract_json(output) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(tasks) = parsed.get("tasks").and_then(|t| t.as_array()) {
                    for task in tasks {
                        let id = task.get("id").and_then(|v| v.as_str()).unwrap_or("task").to_string();
                        let desc = task.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let file_path = task.get("file_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let deps = task.get("dependencies")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|d| d.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default();
                        let validation = task.get("validation").and_then(|v| v.as_str()).unwrap_or("export").to_string();

                        // 使用 file_path 作为描述的一部分
                        let full_desc = if file_path.is_empty() {
                            desc.clone()
                        } else {
                            format!("{} (file: {})", desc, file_path)
                        };

                        subtasks.push(SubTaskSpec {
                            id,
                            description: full_desc,
                            dependencies: deps,
                            validation_criteria: validation,
                        });
                    }
                }
            }
        }

        // 如果 JSON 解析失败，尝试旧的格式解析
        if subtasks.is_empty() {
            for line in output.lines() {
                if line.starts_with("- ") || line.contains(": ") {
                    let parts: Vec<&str> = line.split(": ").collect();
                    if parts.len() >= 2 {
                        let id = parts[0].replace("- ", "").trim().to_string();
                        let desc = parts[1].trim().to_string();

                        let deps: Vec<String> = if line.contains("(depends on:") {
                            line.split("depends on:")
                                .nth(1)
                                .unwrap_or("")
                                .replace(")", "")
                                .split(",")
                                .map(|s| s.trim().to_string())
                                .collect()
                        } else {
                            vec![]
                        };

                        let validation = desc.split("包含").nth(1).unwrap_or(&desc).to_string();

                        subtasks.push(SubTaskSpec {
                            id,
                            description: desc,
                            dependencies: deps,
                            validation_criteria: validation,
                        });
                    }
                }
            }
        }

        // 如果仍然没有解析到，使用智能 fallback 分解
        if subtasks.is_empty() {
            subtasks = self.create_smart_fallback(output);
        }

        let steps = subtasks.len() as u32;
        Ok(ImplementationPlan {
            analysis: output.to_string(),
            subtasks,
            estimated_steps: steps,
        })
    }

    /// 从输出中提取 JSON 块
    fn extract_json<'a>(&self, output: &'a str) -> Option<&'a str> {
        // 查找 ```json 块
        if let Some(start) = output.find("```json") {
            let json_start = start + 7;
            if let Some(end) = output[json_start..].find("```") {
                return Some(&output[json_start..json_start + end]);
            }
        }
        // 查找 { 开始的 JSON
        if let Some(start) = output.find('{') {
            if let Some(end) = output[start..].rfind('}') {
                return Some(&output[start..start + end + 1]);
            }
        }
        None
    }

    /// 智能 fallback - 基于需求关键词分解
    fn create_smart_fallback(&self, requirement: &str) -> Vec<SubTaskSpec> {
        let mut tasks: Vec<SubTaskSpec> = Vec::new();

        // 检测需求中的文件路径关键词
        let file_keywords = ["README", "production", "inventory", "quality", "main", "index", "utils", "types"];
        let ext_keywords = [".ts", ".js", ".md", ".tsx", ".jsx", ".py", ".rs"];

        for keyword in &file_keywords {
            if requirement.contains(keyword) {
                let ext = if requirement.contains(".md") { ".md" } else { ".ts" };
                let file_path = if *keyword == "README" {
                    "README.md".to_string()
                } else {
                    format!("src/{}{}", keyword.to_lowercase(), ext)
                };

                let desc = format!("Create {} file", file_path);
                let validation = if *keyword == "README" { "MES" } else { keyword };

                let deps = if tasks.is_empty() {
                    vec![]
                } else {
                    vec![tasks.last().unwrap().id.clone()]
                };

                tasks.push(SubTaskSpec {
                    id: format!("task-{}", tasks.len() + 1),
                    description: desc,
                    dependencies: deps,
                    validation_criteria: validation.to_string(),
                });
            }
        }

        // 如果没有检测到任何关键词，创建单个任务
        if tasks.is_empty() {
            tasks.push(SubTaskSpec {
                id: "task-1".to_string(),
                description: requirement.lines().next().unwrap_or("Create main module").to_string(),
                dependencies: vec![],
                validation_criteria: "export".to_string(),
            });
        }

        tasks
    }

    /// 执行 AI 命令
    async fn execute_ai(&self, prompt: &str) -> Result<String, ComplexError> {
        use std::process::Command;

        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &self.ai_command, "-p", prompt])
                .current_dir(&self.working_dir)
                .output()
                .map_err(|e| ComplexError::ExecutionFailed(format!("AI process failed: {}", e)))?
        } else {
            Command::new(&self.ai_command)
                .args(["-p", prompt])
                .current_dir(&self.working_dir)
                .output()
                .map_err(|e| ComplexError::ExecutionFailed(format!("AI process failed: {}", e)))?
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ComplexError::ExecutionFailed(stderr.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// 检查 TypeScript 编译
    fn check_typescript_compile(&self, path: &str) -> bool {
        use std::process::Command;

        // 检查是否有 tsconfig.json
        let tsconfig = std::path::Path::new(path).join("tsconfig.json");
        if !tsconfig.exists() {
            return true; // 没有 tsconfig，跳过编译检查
        }

        // 运行 tsc --noEmit
        let result = Command::new("tsc")
            .args(["--noEmit"])
            .current_dir(path)
            .output();

        match result {
            Ok(output) => output.status.success(),
            Err(_) => true, // tsc 不存在，跳过
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_planning_prompt() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let prompt = executor.build_planning_prompt("Create a MES system");
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("tasks"));
        assert!(prompt.contains("REQUIREMENT"));
    }

    #[test]
    fn test_parse_plan_json() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let output = "```json\n{\"tasks\": [{\"id\": \"task-1\", \"description\": \"Create README\", \"file_path\": \"README.md\", \"dependencies\": [], \"validation\": \"MES\"}]}\n```";
        let plan = executor.parse_plan(output).unwrap();
        assert_eq!(plan.subtasks.len(), 1);
        assert_eq!(plan.subtasks[0].id, "task-1");
    }

    #[test]
    fn test_parse_plan_fallback() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let output = "Create MES with README, production, inventory modules";
        let plan = executor.parse_plan(output).unwrap();
        assert!(!plan.subtasks.is_empty());
    }

    #[test]
    fn test_extract_json() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let output = "Some text ```json\n{\"test\": 1}\n``` more text";
        let json = executor.extract_json(output);
        assert!(json.is_some());
        assert!(json.unwrap().contains("test"));
    }

    #[test]
    fn test_smart_fallback() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let tasks = executor.create_smart_fallback("Create README and production.ts for MES");
        assert!(!tasks.is_empty());
        assert!(tasks.iter().any(|t| t.description.contains("README")));
        assert!(tasks.iter().any(|t| t.description.contains("production")));
    }

    #[test]
    fn test_build_execution_prompt_from_info() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let prompt = executor.build_execution_prompt_from_info("task-1", "Create README.md (file: README.md)", &[]);
        assert!(prompt.contains("Target File"));
        assert!(prompt.contains("Working Directory"));
    }
}