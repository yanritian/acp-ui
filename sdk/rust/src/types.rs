//! ACP Worker SDK Types

use serde::{Deserialize, Serialize};

/// Worker 能力配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub worker_id: String,
    pub worker_type: String,
    pub capabilities: Vec<String>,
    pub max_complexity: u32,
    pub max_concurrent: u32,
    pub supports_streaming: bool,
    pub supports_cancel: bool,
    pub default_timeout_ms: u64,
}

/// 任务描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescription {
    pub task_id: String,
    pub prompt: String,
    pub working_dir: Option<String>,
    pub context: std::collections::HashMap<String, String>,
    pub timeout_ms: u64,
    pub priority: u32,
    pub expected_format: String,
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
    pub tokens_used: Option<u64>,
}

/// Worker 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub worker_id: String,
    pub health: String,
    pub current_task: Option<String>,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
}