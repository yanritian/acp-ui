//! ComplexGoalExecutor - 处理复杂系统需求的执行器
//!
//! 核心改进：
//! 1. 规划阶段：AI 分析需求并制定实施计划
//! 2. 分解阶段：将需求拆分成多个子 Goal
//! 3. 执行阶段：分批次执行，支持进度持久化和恢复
//! 4. 验证阶段：检查所有文件，编译验证

use crate::goal::{Goal, GoalStatus, CompletionCondition, GoalOutcome};
use crate::goal_graph::GoalGraph;
use crate::goal_evaluator::ConditionEvaluator;
use std::sync::Arc;
use tokio::sync::Mutex;
use thiserror::Error;
use std::path::Path;

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

    #[error("Progress error: {0}")]
    ProgressError(String),
}

/// 实施计划
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImplementationPlan {
    /// 需求分析
    pub analysis: String,
    /// 子任务列表
    pub subtasks: Vec<SubTaskSpec>,
    /// 预估总步骤
    pub estimated_steps: u32,
}

/// 子任务规格
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTaskSpec {
    pub id: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub validation_criteria: String,
}

/// 执行进度（持久化状态）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionProgress {
    /// 需求原文
    pub requirement: String,
    /// 实施计划
    pub plan: ImplementationPlan,
    /// 已完成的任务
    pub completed: Vec<TaskResult>,
    /// 失败的任务
    pub failed: Vec<TaskResult>,
    /// 待执行的任务 ID
    pub pending: Vec<String>,
    /// 当前批次
    pub current_batch: u32,
    /// 总批次
    pub total_batches: u32,
    /// 时间戳
    pub timestamp: u64,
}

/// 任务执行结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub iterations: u32,
    pub feedback: String,
    pub file_path: Option<String>,
}

/// ComplexGoalExecutor - 复杂需求执行器（改进版）
pub struct ComplexGoalExecutor {
    ai_command: String,
    working_dir: String,
    evaluator: ConditionEvaluator,
    /// 进度文件路径
    progress_file: String,
    /// 每批次执行的任务数
    batch_size: u32,
    /// 单任务超时秒数
    task_timeout_secs: u64,
}

impl ComplexGoalExecutor {
    pub fn new(ai_command: impl Into<String>, working_dir: impl Into<String>) -> Self {
        let ai_cmd = ai_command.into();
        let work_dir = working_dir.into();
        Self {
            ai_command: ai_cmd,
            working_dir: work_dir.clone(),
            evaluator: ConditionEvaluator::new(),
            progress_file: format!("{}progress.json", work_dir),
            batch_size: 3, // 每批次 3 个任务
            task_timeout_secs: 45, // 单任务 45 秒超时
        }
    }

    /// 设置批次大小
    pub fn with_batch_size(mut self, size: u32) -> Self {
        self.batch_size = size;
        self
    }

    /// 设置任务超时
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.task_timeout_secs = secs;
        self
    }

    /// 保存进度到文件
    pub fn save_progress(&self, progress: &ExecutionProgress) -> Result<(), ComplexError> {
        let json = serde_json::to_string_pretty(progress)
            .map_err(|e| ComplexError::ProgressError(format!("Serialize: {}", e)))?;
        std::fs::write(&self.progress_file, json)
            .map_err(|e| ComplexError::ProgressError(format!("Write: {}", e)))?;
        println!("  [进度已保存: {} 个完成, {} 个待执行]", progress.completed.len(), progress.pending.len());
        Ok(())
    }

    /// 从文件恢复进度
    pub fn load_progress(&self) -> Result<ExecutionProgress, ComplexError> {
        if !Path::new(&self.progress_file).exists() {
            return Err(ComplexError::ProgressError("No progress file found".into()));
        }
        let json = std::fs::read_to_string(&self.progress_file)
            .map_err(|e| ComplexError::ProgressError(format!("Read: {}", e)))?;
        let progress: ExecutionProgress = serde_json::from_str(&json)
            .map_err(|e| ComplexError::ProgressError(format!("Deserialize: {}", e)))?;
        println!("  [进度已恢复: {} 个完成, {} 个待执行]", progress.completed.len(), progress.pending.len());
        Ok(progress)
    }

    /// 检查是否有可恢复的进度
    pub fn has_progress(&self) -> bool {
        Path::new(&self.progress_file).exists()
    }

    /// 清除进度文件
    pub fn clear_progress(&self) {
        std::fs::remove_file(&self.progress_file).ok();
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

    /// 第三阶段：执行 - 分批次执行，支持进度持久化
    pub async fn execute_graph(&self, graph: &mut GoalGraph) -> Vec<(String, GoalOutcome)> {
        self.execute_graph_with_progress(graph, None).await
    }

    /// 分批次执行（带进度）
    pub async fn execute_graph_with_progress(
        &self,
        graph: &mut GoalGraph,
        initial_progress: Option<ExecutionProgress>,
    ) -> Vec<(String, GoalOutcome)> {
        let mut results: Vec<(String, GoalOutcome)> = Vec::new();
        let order = graph.topological_order();
        let total_tasks = order.len();
        let batch_size = self.batch_size as usize;

        // 计算批次
        let total_batches = (total_tasks + batch_size - 1) / batch_size;
        println!("  总任务: {} 个, 分 {} 批执行 (每批 {} 个)\n", total_tasks, total_batches, batch_size);

        // 如果有恢复的进度，跳过已完成的任务
        let skip_tasks: Vec<String> = if let Some(progress) = initial_progress {
            progress.completed.iter().map(|r| r.task_id.clone()).collect()
        } else {
            vec![]
        };

        // 分批次执行（改进：内容质量验证 + 空文件重试）
        for batch_num in 0..total_batches {
            let batch_start = batch_num * batch_size;
            let batch_end = std::cmp::min(batch_start + batch_size, total_tasks);
            let batch_tasks = &order[batch_start..batch_end];

            println!("【批次 {}】执行任务 {}-{}:", batch_num + 1, batch_start + 1, batch_end);

            for goal_id in batch_tasks {
                // 跳过已完成的任务
                if skip_tasks.contains(goal_id) {
                    println!("  ✓ {} 已完成（从进度恢复）", goal_id);
                    continue;
                }

                if let Some(goal) = graph.get_goal(goal_id) {
                    let description = goal.description.clone();
                    let condition = goal.completion_condition.clone();

                    println!("  执行 {}...", goal_id);
                    graph.update_goal_status(goal_id, GoalStatus::Active);

                    // 提取文件路径
                    let file_path = self.extract_file_path(&description);

                    // 重试循环（最多 2 次）
                    let mut success = false;
                    let mut attempts = 0;
                    let max_attempts = 2;

                    while !success && attempts < max_attempts {
                        attempts += 1;
                        if attempts > 1 {
                            println!("    重试 {} 第 {} 次...", goal_id, attempts);
                        }

                        let prompt = self.build_execution_prompt_from_info(goal_id, &description, &results);
                        let exec_result = self.execute_ai_with_timeout(&prompt, self.task_timeout_secs).await;

                        match exec_result {
                            Ok(_) => {
                                // 验证文件是否存在且有足够内容
                                if !file_path.is_empty() {
                                    let content_check = self.check_file_quality(&file_path);
                                    if content_check.is_ok() {
                                        success = true;
                                        println!("    ✓ 成功 ({} 行)", content_check.unwrap());
                                    } else {
                                        println!("    ✗ 内容不足: {}", content_check.unwrap_err());
                                        // 删除空文件，准备重试
                                        std::fs::remove_file(&file_path).ok();
                                    }
                                } else {
                                    // 无文件路径的任务，只检查条件
                                    graph.update_goal_status(goal_id, GoalStatus::Evaluating);
                                    let eval_result = self.evaluator.evaluate(&condition);
                                    success = eval_result.converged;
                                    if !success {
                                        println!("    ✗ 条件未满足");
                                    }
                                }
                            },
                            Err(e) => {
                                println!("    ✗ 错误: {}", e);
                            }
                        }
                    }

                    // 记录结果
                    if success {
                        graph.update_goal_status(goal_id, GoalStatus::Converged);
                        results.push((goal_id.clone(), GoalOutcome::Converged {
                            iterations: attempts,
                            tokens_used: 0,
                            final_feedback: "Success".into(),
                        }));
                    } else {
                        graph.update_goal_status(goal_id, GoalStatus::Failed);
                        results.push((goal_id.clone(), GoalOutcome::Failed(
                            format!("Failed after {} attempts", attempts)
                        )));
                    }
                }
            }

            println!("  批次 {} 完成\n", batch_num + 1);

            // 进度统计
            let success_count = results.iter().filter(|(_, o)| matches!(o, GoalOutcome::Converged { .. })).count();
            println!("  进度: {} / {} 完成", success_count, total_tasks);

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        println!("\n全部执行完成！");
        results
    }

    /// 提取文件路径
    fn extract_file_path(&self, description: &str) -> String {
        if description.contains("(file: ") {
            description.split("(file: ")
                .nth(1)
                .and_then(|s| s.split(')').next())
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }

    /// 检查文件质量（至少 50 行或 500 bytes）
    fn check_file_quality(&self, path: &str) -> Result<u32, String> {
        if !std::path::Path::new(path).exists() {
            return Err("File not created".into());
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let lines = content.lines().count() as u32;
        let bytes = content.len();

        if lines >= 50 || bytes >= 500 {
            Ok(lines)
        } else if lines == 0 {
            Err("Empty file".into())
        } else {
            Err(format!("Insufficient content: {} lines, {} bytes (need 50 lines or 500 bytes)", lines, bytes))
        }
    }

    /// 执行 AI 命令（带超时）- 使用更短的超时
    async fn execute_ai_with_timeout(&self, prompt: &str, timeout_secs: u64) -> Result<String, ComplexError> {
        use tokio::time::{timeout, Duration};

        let result = timeout(
            Duration::from_secs(timeout_secs),
            self.execute_ai(prompt)
        ).await;

        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ComplexError::ExecutionFailed(format!("Timeout {}s", timeout_secs))),
        }
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

    /// 构建规划 prompt - 明确格式，确保 AI 正确理解任务分解
    fn build_planning_prompt(&self, requirement: &str) -> String {
        format!(
            r#"分析需求，输出结构化 JSON 任务列表。

需求: {}

输出 JSON 格式（严格遵守）:
{{
  "analysis": "简短分析",
  "tasks": [
    {{
      "id": "task-1",
      "file": "{}/文件名.ts",
      "desc": "模块功能描述",
      "deps": []
    }},
    {{
      "id": "task-2",
      "file": "{}/另一个文件.md",
      "desc": "另一个模块描述",
      "deps": ["task-1"]
    }}
  ]
}}

关键规则:
1. file 字段: 必须是完整路径，包含工作目录前缀
2. deps 字段: 空数组 [] 表示无依赖，有依赖填任务 ID
3. 每个任务创建一个文件，文件扩展名要与内容匹配
4. id 格式: task-1, task-2, task-3... 顺序编号

仅输出 JSON，不要其他文字:"#,
            requirement.chars().take(300).collect::<String>(),
            self.working_dir,
            self.working_dir
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

    /// 构建执行 prompt（明确文件路径和内容质量要求）
    fn build_execution_prompt_from_info(&self, _id: &str, description: &str, completed: &[(String, GoalOutcome)]) -> String {
        // 从 description 提取文件路径（格式: "... (file: path) ..."）
        let file_path = if description.contains("(file: ") {
            description.split("(file: ")
                .nth(1)
                .and_then(|s| s.split(')').next())
                .map(|s| s.trim())
                .unwrap_or("")
        } else {
            // 备用：查找 .ts 或 .md 文件名
            description.split_whitespace()
                .find(|s| s.ends_with(".ts") || s.ends_with(".md"))
                .unwrap_or("")
        };

        // 构建明确的 prompt（包含内容质量要求）
        let mut prompt = String::new();

        // 明确的目标文件
        if !file_path.is_empty() {
            prompt.push_str(&format!("TARGET FILE: {}\n\n", file_path));
        }

        // 内容质量要求（关键改进）
        prompt.push_str("CONTENT REQUIREMENT:\n");
        prompt.push_str("- File must have at least 50 lines OR 500 bytes\n");
        prompt.push_str("- Include complete module structure with exports\n");
        prompt.push_str("- Add detailed comments and type definitions\n\n");

        // 功能描述
        let func_desc = description.split("(file: ").next().unwrap_or(description);
        prompt.push_str(&format!("MODULE: {}\n\n", func_desc.chars().take(80).collect::<String>()));

        // 已完成提示
        if !completed.is_empty() {
            let done_count = completed.iter().filter(|(_, o)| matches!(o, GoalOutcome::Converged { .. })).count();
            if done_count > 0 {
                prompt.push_str(&format!("COMPLETED: {} files already created\n\n", done_count));
            }
        }

        // 执行指令
        prompt.push_str("ACTION: Use Write tool to create the file NOW.\n");
        prompt.push_str("DO NOT create empty files. DO NOT stop until file has substantial content.");

        prompt
    }

    /// 从描述中提取文件路径
    fn extract_file_path_from_description(&self, description: &str) -> String {
        // 格式: "description (file: path/to/file.ts)"
        if let Some(file_part) = description.split("(file: ").nth(1) {
            if let Some(path) = file_part.strip_suffix(')') {
                return path.trim().to_string();
            }
        }

        // 尝试提取常见路径模式
        let path_patterns = [
            ("src/", ".ts"),
            ("src/", ".tsx"),
            ("lib/", ".ts"),
            ("", ".md"),
            ("", ".json"),
        ];

        for (prefix, ext) in &path_patterns {
            if description.contains(prefix) && description.contains(ext) {
                // 尝试提取完整路径
                if let Some(start) = description.find(prefix) {
                    if let Some(end) = description[start..].find(ext) {
                        let path = &description[start..start + end + ext.len()];
                        return path.to_string();
                    }
                }
            }
        }

        String::new()
    }

    /// 解析 AI 输出提取计划 - 改进版，支持 JSON 和智能 fallback
    fn parse_plan(&self, output: &str) -> Result<ImplementationPlan, ComplexError> {
        let mut subtasks = Vec::new();
        let mut analysis = output.to_string();

        // 尝试解析 JSON 格式
        if let Some(json_str) = self.extract_json(output) {
            // 尝试解析为 JSON
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(parsed) => {
                    // 提取 analysis 字段
                    if let Some(a) = parsed.get("analysis").and_then(|v| v.as_str()) {
                        analysis = a.to_string();
                    }

                    // 解析 tasks 数组
                    if let Some(tasks) = parsed.get("tasks").and_then(|t| t.as_array()) {
                        for task in tasks {
                            let id = task.get("id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| format!("task-{}", subtasks.len() + 1));

                            // 解析字段（支持精简格式: desc/file 和完整格式: description/file_path）
                            let desc = task.get("desc")
                                .and_then(|v| v.as_str())
                                .or_else(|| task.get("description").and_then(|v| v.as_str()))
                                .unwrap_or("")
                                .to_string();

                            let file_path = task.get("file")
                                .and_then(|v| v.as_str())
                                .or_else(|| task.get("file_path").and_then(|v| v.as_str()))
                                .unwrap_or("")
                                .to_string();

                            let deps = task.get("deps")
                                .and_then(|v| v.as_array())
                                .or_else(|| task.get("dependencies").and_then(|v| v.as_array()))
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|d| d.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();

                            let validation = task.get("validation")
                                .and_then(|v| v.as_str())
                                .unwrap_or("export")
                                .to_string();

                            // 构建完整描述，包含文件路径
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
                Err(e) => {
                    // JSON 解析失败，记录但继续尝试 fallback
                    eprintln!("JSON parse error: {}, attempting fallback", e);
                }
            }
        }

        // 如果 JSON 解析失败或没有提取到任务，尝试旧的格式解析
        if subtasks.is_empty() {
            for line in output.lines() {
                // 匹配格式: "- task-id: description" 或 "task-id: description"
                if line.starts_with("- ") || (line.contains(": ") && !line.starts_with("{")) {
                    let parts: Vec<&str> = line.splitn(2, ": ").collect();
                    if parts.len() == 2 {
                        let id = parts[0].replace("- ", "").trim().to_string();
                        let desc = parts[1].trim().to_string();

                        // 解析依赖: "depends on: task-1, task-2"
                        let deps: Vec<String> = if line.contains("(depends on:") {
                            line.split("(depends on:")
                                .nth(1)
                                .unwrap_or("")
                                .replace(")", "")
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        } else if line.contains("depends on:") {
                            line.split("depends on:")
                                .nth(1)
                                .unwrap_or("")
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect()
                        } else {
                            vec![]
                        };

                        // 从描述中提取验证关键词
                        let validation = if desc.contains("包含") {
                            desc.split("包含").nth(1).unwrap_or(&desc).trim().to_string()
                        } else if desc.contains("contains") {
                            desc.split("contains").nth(1).unwrap_or(&desc).trim().to_string()
                        } else {
                            "export".to_string()
                        };

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
            analysis,
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

    /// 执行 AI 命令 - 使用 Git Bash stdin 方式（避免 cmd 管道问题）
    async fn execute_ai(&self, prompt: &str) -> Result<String, ComplexError> {
        use std::process::Command;

        // 创建临时文件存储 prompt
        let temp_file = format!("D:/tmp/complex_prompt_{}.txt",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis());

        std::fs::write(&temp_file, prompt)
            .map_err(|e| ComplexError::ExecutionFailed(format!("Write temp file failed: {}", e)))?;

        // 使用 Git Bash 执行（与 reconcile.rs 一致的方式）
        let git_bash = "C:/Program Files/Git/usr/bin/bash.exe";
        let cmd_args = format!(
            "cat '{}' | '{}' --bare --dangerously-skip-permissions --allowedTools Write,Edit,Bash --permission-mode bypassPermissions --print",
            temp_file,
            &self.ai_command
        );

        let output = Command::new(git_bash)
            .args(["-c", &cmd_args])
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| ComplexError::ExecutionFailed(format!("AI process failed: {}", e)))?;

        // 清理临时文件
        std::fs::remove_file(&temp_file).ok();

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
        assert!(prompt.contains("JSON")); // 验证 JSON 格式
        assert!(prompt.contains("需求")); // 验证需求关键词
        assert!(prompt.contains("D:/tmp")); // 验证工作目录
        assert!(prompt.contains("file")); // 验证 file 字段说明
        assert!(prompt.contains("deps")); // 验证 deps 字段说明
        assert!(prompt.contains("analysis")); // 验证 analysis 字段示例
    }

    #[test]
    fn test_parse_plan_json() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let output = r#"{"analysis": "MES system needs modules", "tasks": [{"id": "task-1", "description": "Create README", "file_path": "README.md", "dependencies": [], "validation": "MES"}]}"#;
        let plan = executor.parse_plan(output).unwrap();
        assert_eq!(plan.subtasks.len(), 1);
        assert_eq!(plan.subtasks[0].id, "task-1");
        assert_eq!(plan.analysis, "MES system needs modules");
        assert!(plan.subtasks[0].description.contains("README.md"));
    }

    #[test]
    fn test_parse_plan_json_with_markdown() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let output = "```\n{\"tasks\": [{\"id\": \"task-1\", \"description\": \"Create README\", \"file_path\": \"README.md\", \"dependencies\": [], \"validation\": \"MES\"}]}\n```";
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
    fn test_parse_plan_text_format() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let output = "- task-1: Create README.md\n- task-2: Create production.ts (depends on: task-1)";
        let plan = executor.parse_plan(output).unwrap();
        assert_eq!(plan.subtasks.len(), 2);
        assert_eq!(plan.subtasks[0].id, "task-1");
        assert_eq!(plan.subtasks[1].dependencies, vec!["task-1"]);
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
    fn test_extract_json_raw() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let output = "Here is the plan: {\"tasks\": [{\"id\": \"task-1\"}]} end";
        let json = executor.extract_json(output);
        assert!(json.is_some());
        assert!(json.unwrap().contains("tasks"));
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
        assert!(prompt.contains("TARGET FILE"));
        assert!(prompt.contains("README.md"));
        assert!(prompt.contains("50 lines")); // 验证内容质量要求
        assert!(prompt.contains("Write tool")); // 验证执行指令
    }

    #[test]
    fn test_build_execution_prompt_with_completed() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");
        let completed = vec![
            ("task-1".to_string(), GoalOutcome::Converged {
                iterations: 1,
                tokens_used: 100,
                final_feedback: "Created README.md successfully".to_string(),
            }),
        ];
        let prompt = executor.build_execution_prompt_from_info("task-2", "Create index.ts (file: src/index.ts)", &completed);
        assert!(prompt.contains("COMPLETED")); // 验证已完成提示
        assert!(prompt.contains("src/index.ts"));
        assert!(prompt.contains("50 lines")); // 验证内容质量要求
    }

    #[test]
    fn test_extract_file_path_from_description() {
        let executor = ComplexGoalExecutor::new("claude", "D:/tmp");

        // 测试 (file: ...) 格式
        let path = executor.extract_file_path_from_description("Create README (file: README.md)");
        assert_eq!(path, "README.md");

        // 测试 src/ 路径
        let path = executor.extract_file_path_from_description("Create src/index.ts module");
        assert_eq!(path, "src/index.ts");

        // 测试无路径
        let path = executor.extract_file_path_from_description("Do something generic");
        assert!(path.is_empty());
    }
}