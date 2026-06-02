//! Executive Agent Module - 使用 Hermes AgentLoop 原生执行
//! Hermes Rust crate 直接集成到 ACP-UI 项目

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tauri::{AppHandle, Emitter};

// Hermes Agent imports
use hermes_agent::{
    AgentLoop, AgentResult, AgentCallbacks,
    agent_builder::{build_agent_config, build_provider, bridge_tool_registry},
};
use hermes_core::MessageRole;
use hermes_config::load_config;
use hermes_tools::ToolRegistry;

use crate::database::{ThinkingChunkRecord, ToolCallRecord};

/// Hermes 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesConfig {
    /// Hermes 配置目录 (HERMES_HOME)
    pub config_dir: Option<String>,
    /// 模型名称
    pub model: String,
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            config_dir: None,
            model: "alibaba-coding-plan:qwen3.6-plus".to_string(),
        }
    }
}

/// 执行模式 - 仅 HermesNative
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ExecutionMode {
    #[default]
    HermesNative,
}

/// 智能体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Coder,
}

impl AgentType {
    pub fn name(&self) -> &'static str {
        "编码智能体"
    }
}

/// 智能体状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Busy,
    Error,
}

/// 执行日志
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLog {
    pub timestamp: DateTime<Utc>,
    pub agent_type: AgentType,
    pub action: String,
    pub file: Option<String>,
    pub content_preview: Option<String>,
    pub error: Option<String>,
}

/// 生成的文件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedFile {
    pub path: String,
    pub relative_path: String,
    pub content: String,
    pub lines: usize,
    pub created_at: DateTime<Utc>,
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResult {
    pub task_id: String,
    pub request: String,
    pub workspace: String,
    pub files: Vec<GeneratedFile>,
    pub logs: Vec<ExecutionLog>,
    pub summary: String,
    pub completed_at: DateTime<Utc>,
}

/// Executive Agent Manager - 使用 Hermes AgentLoop 原生执行
#[derive(Clone)]
pub struct ExecutiveAgentManager {
    pub workspace: PathBuf,
    logs: Arc<RwLock<Vec<ExecutionLog>>>,
    generated_files: Arc<RwLock<HashMap<String, GeneratedFile>>>,
    agent_status: Arc<RwLock<HashMap<AgentType, AgentStatus>>>,
    hermes_config: HermesConfig,
    execution_mode: ExecutionMode,
    /// Thinking chunks collected during execution
    thinking_chunks: Arc<StdMutex<Vec<ThinkingChunkRecord>>>,
    /// Tool calls collected during execution
    tool_calls: Arc<StdMutex<Vec<ToolCallRecord>>>,
    /// Current task ID for callbacks
    current_task_id: Arc<StdMutex<Option<String>>>,
}

impl ExecutiveAgentManager {
    pub fn new(workspace: PathBuf) -> Self {
        Self::with_config(workspace, HermesConfig::default(), ExecutionMode::default())
    }

    pub fn with_config(workspace: PathBuf, hermes_config: HermesConfig, execution_mode: ExecutionMode) -> Self {
        let mut status = HashMap::new();
        status.insert(AgentType::Coder, AgentStatus::Idle);

        Self {
            workspace,
            logs: Arc::new(RwLock::new(Vec::new())),
            generated_files: Arc::new(RwLock::new(HashMap::new())),
            agent_status: Arc::new(RwLock::new(status)),
            hermes_config,
            execution_mode,
            thinking_chunks: Arc::new(StdMutex::new(Vec::new())),
            tool_calls: Arc::new(StdMutex::new(Vec::new())),
            current_task_id: Arc::new(StdMutex::new(None)),
        }
    }

    pub fn get_execution_mode(&self) -> ExecutionMode {
        self.execution_mode.clone()
    }

    pub fn get_agent_status(&self) -> HashMap<AgentType, AgentStatus> {
        self.agent_status.read().clone()
    }

    fn extract_files(&self, content: &str) -> HashMap<String, String> {
        let mut files = HashMap::new();

        // 尝试多种正则模式匹配
        let patterns = vec![
            // 格式1: ### FILE: xxx ```...``
            r"###\s*FILE:\s*[`]?([^`\n]+)[`]?[\s\n]+```[\w]*\s*([\s\S]*?)```",
            // 格式2: ### 📄 FILE: `xxx` ```...``
            r"###\s*📄\s*FILE:\s*[`]?([^`\n]+)[`]?[\s\n]+```[\w]*\s*([\s\S]*?)```",
            // 格式3: ### 📄 `xxx` ```...``
            r"###\s*📄\s*[`]?([^`\n]+\.rs|[^`\n]+\.cs|[^`\n]+\.vue|[^`\n]+\.ts|[^`\n]+\.json|[^`\n]+\.md)[`]?[\s\n]+```[\w]*\s*([\s\S]*?)```",
            // 格式4: 最宽松 - 匹配任何文件名
            r"###\s*(?:FILE:|📄\s*FILE:|📄)\s*[`]?([^`\n]+\.[a-zA-Z]+)[`]?[\s\n]+```[\w]*\s*([\s\S]*?)```",
        ];

        for pattern_str in patterns {
            if let Ok(pattern) = Regex::new(pattern_str) {
                for caps in pattern.captures_iter(content) {
                    let file_path = caps[1].trim();
                    let code = caps[2].trim();

                    // 只添加包含有效文件扩展名的
                    if file_path.contains('.') && !files.contains_key(file_path) {
                        files.insert(file_path.to_string(), code.to_string());
                    }
                }
            }
        }

        files
    }

    async fn write_files(&self, files: HashMap<String, String>, app_handle: &AppHandle) -> Vec<GeneratedFile> {
        let mut generated = Vec::new();

        for (relative_path, content) in files {
            let full_path = self.workspace.join(&relative_path);

            if let Some(parent) = full_path.parent() {
                if !parent.exists() {
                    let _ = fs::create_dir_all(parent);
                }
            }

            if fs::write(&full_path, &content).is_ok() {
                let lines = content.lines().count();
                let file_info = GeneratedFile {
                    path: full_path.to_string_lossy().to_string(),
                    relative_path: relative_path.clone(),
                    content: content.clone(),
                    lines,
                    created_at: Utc::now(),
                };

                generated.push(file_info.clone());
                self.generated_files.write().insert(relative_path.clone(), file_info);

                let _ = app_handle.emit("file-created", serde_json::json!({
                    "path": relative_path,
                    "lines": lines
                }));
            }
        }
        generated
    }

    fn add_log(&self, log: ExecutionLog) {
        self.logs.write().push(log);
    }

    pub async fn execute_workflow(&self, request: String, app_handle: AppHandle) -> Result<TaskResult, String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        self.logs.write().clear();
        self.generated_files.write().clear();

        // 设置当前 task_id 用于 callbacks
        *self.current_task_id.lock().unwrap() = Some(task_id.clone());

        // 清空收集的数据
        self.thinking_chunks.lock().unwrap().clear();
        self.tool_calls.lock().unwrap().clear();

        let _ = app_handle.emit("task-started", serde_json::json!({
            "taskId": task_id,
            "request": request,
            "workspace": self.workspace.to_string_lossy().to_string(),
            "mode": "HermesNative"
        }));

        self.agent_status.write().insert(AgentType::Coder, AgentStatus::Busy);

        let _ = app_handle.emit("agent-status-update", serde_json::json!({
            "agentType": "coder",
            "status": "busy"
        }));

        self.add_log(ExecutionLog {
            timestamp: Utc::now(),
            agent_type: AgentType::Coder,
            action: "started".to_string(),
            file: None,
            content_preview: Some(format!("Processing: {}", request.chars().take(50).collect::<String>())),
            error: None,
        });

        // 构建提示词
        let full_prompt = format!(
            "{}\n\n## 用户需求\n{}\n\n请根据需求生成完整的代码实现。使用 ### FILE: 路径 格式标记每个文件。",
            CODER_PROMPT, request
        );

        // 使用 Hermes AgentLoop 原生执行
        let content = self.execute_with_hermes_native(&full_prompt, &app_handle).await?;

        self.add_log(ExecutionLog {
            timestamp: Utc::now(),
            agent_type: AgentType::Coder,
            action: "response".to_string(),
            file: None,
            content_preview: Some(content.lines().take(10).collect::<Vec<_>>().join("\n")),
            error: None,
        });

        // 提取并写入文件
        let extracted_files = self.extract_files(&content);
        let all_files = self.write_files(extracted_files, &app_handle).await;

        self.agent_status.write().insert(AgentType::Coder, AgentStatus::Idle);

        let summary = self.generate_summary(&request, &all_files);

        let result = TaskResult {
            task_id,
            request,
            workspace: self.workspace.to_string_lossy().to_string(),
            files: all_files,
            logs: self.logs.read().clone(),
            summary,
            completed_at: Utc::now(),
        };

        let _ = app_handle.emit("task-completed", serde_json::json!({
            "taskId": result.task_id,
            "files": result.files.iter().map(|f| f.relative_path.clone()).collect::<Vec<_>>(),
            "summary": result.summary
        }));

        Ok(result)
    }

    /// 使用 Hermes AgentLoop 原生执行任务
    async fn execute_with_hermes_native(&self, prompt: &str, app_handle: &AppHandle) -> Result<String, String> {
        // 获取当前 task_id
        let task_id = self.current_task_id.lock().unwrap().clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // 加载 GatewayConfig
        let gateway_config = load_config(self.hermes_config.config_dir.as_deref())
            .map_err(|e| format!("加载 Hermes 配置失败: {}", e))?;

        // 构建 AgentConfig
        let agent_config = build_agent_config(&gateway_config, &self.hermes_config.model, Some("acp-ui"));

        // 构建 LLM Provider
        let llm_provider = build_provider(&gateway_config, &self.hermes_config.model);

        // 创建 ToolRegistry (使用空 registry，因为我们不需要工具调用)
        let tools = ToolRegistry::new();
        let tool_registry = Arc::new(bridge_tool_registry(&tools));

        // 创建回调 - 捕获 thinking 和 tool calls
        let thinking_chunks_ref = self.thinking_chunks.clone();
        let tool_calls_ref = self.tool_calls.clone();
        let tool_calls_ref_for_complete = self.tool_calls.clone();
        let app_handle_for_thinking = Arc::new(app_handle.clone());
        let app_handle_for_tool_start = Arc::new(app_handle.clone());
        let app_handle_for_tool_complete = Arc::new(app_handle.clone());
        let task_id_for_thinking = task_id.clone();
        let task_id_for_tool_start = task_id.clone();
        let task_id_for_tool_complete = task_id.clone();

        // 清空之前的收集
        self.thinking_chunks.lock().unwrap().clear();
        self.tool_calls.lock().unwrap().clear();

        let callbacks = AgentCallbacks {
            on_thinking: Some(Box::new(move |content: &str| {
                // 保存 thinking chunk
                let chunk = ThinkingChunkRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    task_id: task_id_for_thinking.clone(),
                    content: content.to_string(),
                    depth: 1,
                    duration_ms: 0,
                    created_at: Utc::now(),
                };
                thinking_chunks_ref.lock().unwrap().push(chunk.clone());

                // 发送事件给前端
                let _ = app_handle_for_thinking.emit("thinking-chunk", serde_json::json!({
                    "taskId": task_id_for_thinking,
                    "content": content,
                    "timestamp": Utc::now().to_rfc3339(),
                }));
            })),
            on_tool_start: Some(Box::new(move |tool_name: &str, args: &JsonValue| {
                // 保存 tool call 开始
                let call = ToolCallRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    task_id: task_id_for_tool_start.clone(),
                    tool_name: tool_name.to_string(),
                    arguments_json: Some(serde_json::to_string(args).unwrap_or_default()),
                    result: None,
                    status: "running".to_string(),
                    duration_ms: None,
                    error: None,
                    created_at: Utc::now(),
                    completed_at: None,
                };
                tool_calls_ref.lock().unwrap().push(call.clone());

                // 发送事件给前端
                let _ = app_handle_for_tool_start.emit("tool-start", serde_json::json!({
                    "taskId": task_id_for_tool_start,
                    "toolName": tool_name,
                    "arguments": args,
                }));
            })),
            on_tool_complete: Some(Box::new(move |tool_name: &str, output: &str| {
                // 更新 tool call 完成
                let mut calls = tool_calls_ref_for_complete.lock().unwrap();
                if let Some(call) = calls.iter_mut().rev().find(|c| c.tool_name == tool_name && c.status == "running") {
                    call.status = "completed".to_string();
                    call.result = Some(output.to_string());
                    call.completed_at = Some(Utc::now());
                    call.duration_ms = Some((Utc::now() - call.created_at).num_milliseconds() as u64);
                }

                // 发送事件给前端
                let _ = app_handle_for_tool_complete.emit("tool-complete", serde_json::json!({
                    "taskId": task_id_for_tool_complete,
                    "toolName": tool_name,
                    "result": output,
                }));
            })),
            ..Default::default()
        };

        // 使用 AgentLoop::new 创建 agent loop，并设置 callbacks
        let agent_loop = AgentLoop::new(agent_config, tool_registry, llm_provider)
            .with_callbacks(callbacks);

        // 构建初始消息
        let messages = vec![hermes_core::Message::user(prompt)];

        // 执行 AgentLoop (不传递工具)
        let result: AgentResult = agent_loop.run(messages, None)
            .await
            .map_err(|e| format!("Hermes AgentLoop 执行失败: {}", e))?;

        // 提取最终响应内容
        let content = result.messages.iter()
            .rev()
            .find_map(|m| {
                if m.role == MessageRole::Assistant {
                    m.content.clone()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "无响应内容".to_string());

        Ok(content)
    }

    fn generate_summary(&self, request: &str, files: &[GeneratedFile]) -> String {
        format!(
            "### 任务完成\n\n需求: {}\n\n文件: {} 个\n\n行数: {}",
            request.chars().take(100).collect::<String>(),
            files.len(),
            files.iter().map(|f| f.lines).sum::<usize>()
        )
    }

    pub fn get_generated_files(&self) -> Vec<GeneratedFile> {
        self.generated_files.read().values().cloned().collect()
    }

    pub fn get_logs(&self) -> Vec<ExecutionLog> {
        self.logs.read().clone()
    }

    /// Get collected thinking chunks
    pub fn get_thinking_chunks(&self) -> Vec<ThinkingChunkRecord> {
        self.thinking_chunks.lock().unwrap().clone()
    }

    /// Get collected tool calls
    pub fn get_tool_calls(&self) -> Vec<ToolCallRecord> {
        self.tool_calls.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.logs.write().clear();
        self.generated_files.write().clear();
        self.thinking_chunks.lock().unwrap().clear();
        self.tool_calls.lock().unwrap().clear();
    }

    pub fn stop_hermes(&self) -> Result<(), String> {
        Ok(())
    }
}

const CODER_PROMPT: &str = r#"你是编码智能体。根据需求生成代码。

重要：使用 ### FILE: 路径 格式标记每个文件：
### FILE: src/main.rs
```rust
fn main() {
    println!("Hello");
}
```

用中文注释，提供完整可运行的代码。"#;