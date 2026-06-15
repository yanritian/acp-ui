//! ACP Worker SDK - Rust
//!
//! 使用此 SDK 实现 ACP-UI 的 Worker。

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

pub mod types;
pub use types::*;

/// ACP Worker SDK 错误类型
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    #[error("Heartbeat failed: {0}")]
    HeartbeatFailed(String),

    #[error("Shutdown failed: {0}")]
    ShutdownFailed(String),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

/// ACP Worker
pub struct AcpWorker {
    worker_id: String,
    worker_type: String,
    capabilities: Vec<String>,
    api_base: String,
    task_handler: Arc<Mutex<Option<Box<dyn Fn(TaskDescription) -> TaskResult + Send + Sync>>>>,
    heartbeat_running: Arc<Mutex<bool>>,
}

impl AcpWorker {
    /// 创建新 Worker
    pub fn new(
        worker_id: impl Into<String>,
        worker_type: impl Into<String>,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            worker_id: worker_id.into(),
            worker_type: worker_type.into(),
            capabilities,
            api_base: "http://localhost:3000".to_string(),
            task_handler: Arc::new(Mutex::new(None)),
            heartbeat_running: Arc::new(Mutex::new(false)),
        }
    }

    /// 设置 API 基础 URL
    pub fn with_api_base(mut self, base: impl Into<String>) -> Self {
        self.api_base = base.into();
        self
    }

    /// 注册 Worker
    pub async fn register(&self) -> Result<WorkerCapabilities, WorkerError> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/api/swarm/register", self.api_base))
            .json(&RegisterRequest {
                worker_type: &self.worker_type,
                worker_id: &self.worker_id,
                capabilities: &self.capabilities,
            })
            .send()
            .await?;

        let caps: WorkerCapabilities = resp.json().await?;

        // 启动心跳
        self.start_heartbeat();

        Ok(caps)
    }

    /// 设置任务处理回调
    pub async fn set_task_handler<F>(&self, handler: F)
    where
        F: Fn(TaskDescription) -> TaskResult + Send + Sync + 'static,
    {
        let mut h = self.task_handler.lock().await;
        *h = Some(Box::new(handler));
    }

    /// 执行任务
    pub async fn execute_task(&self, task: TaskDescription) -> Result<TaskResult, WorkerError> {
        let handler = self.task_handler.lock().await;

        if let Some(h) = handler.as_ref() {
            let result = h(task.clone());

            // 上报结果
            self.report_result(&result).await?;

            Ok(result)
        } else {
            Err(WorkerError::TaskExecutionFailed("No task handler set".into()))
        }
    }

    /// 上报任务结果
    async fn report_result(&self, result: &TaskResult) -> Result<(), WorkerError> {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/api/swarm/report", self.api_base))
            .json(&ReportRequest {
                worker_id: &self.worker_id,
                task_id: &result.task_id,
                success: result.success,
                output: &result.output,
                error: result.error.as_deref(),
            })
            .send()
            .await?;

        Ok(())
    }

    /// 发送心跳
    pub async fn send_heartbeat(&self) -> Result<(), WorkerError> {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/api/swarm/heartbeat", self.api_base))
            .json(&HeartbeatRequest { worker_id: &self.worker_id })
            .send()
            .await?;

        Ok(())
    }

    /// 启动心跳
    fn start_heartbeat(&self) {
        let running = self.heartbeat_running.clone();
        let worker_id = self.worker_id.clone();
        let api_base = self.api_base.clone();

        tokio::spawn(async move {
            *running.lock().await = true;
            while *running.lock().await {
                let client = reqwest::Client::new();
                if let Err(e) = client
                    .post(format!("{}/api/swarm/heartbeat", api_base))
                    .json(&HeartbeatRequest { worker_id: &worker_id })
                    .send()
                    .await
                {
                    eprintln!("Heartbeat error: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
    }

    /// 关闭 Worker
    pub async fn shutdown(&self) -> Result<(), WorkerError> {
        // 停止心跳
        *self.heartbeat_running.lock().await = false;

        let client = reqwest::Client::new();
        client
            .post(format!("{}/api/swarm/shutdown", self.api_base))
            .json(&ShutdownRequest { worker_id: &self.worker_id })
            .send()
            .await?;

        Ok(())
    }
}

/// 快速创建 Worker
pub async fn create_worker<F>(
    worker_type: impl Into<String>,
    capabilities: Vec<String>,
    handler: F,
) -> Result<AcpWorker, WorkerError>
where
    F: Fn(TaskDescription) -> TaskResult + Send + Sync + 'static,
{
    let worker_id = format!("worker-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs());

    let worker = AcpWorker::new(worker_id, worker_type, capabilities);
    worker.register().await?;
    worker.set_task_handler(handler).await;

    Ok(worker)
}

// 内部请求类型
#[derive(Serialize)]
struct RegisterRequest<'a> {
    worker_type: &'a str,
    worker_id: &'a str,
    capabilities: &'a Vec<String>,
}

#[derive(Serialize)]
struct ReportRequest<'a> {
    worker_id: &'a str,
    task_id: &'a str,
    success: bool,
    output: &'a str,
    error: Option<&'a str>,
}

#[derive(Serialize)]
struct HeartbeatRequest<'a> {
    worker_id: &'a str,
}

#[derive(Serialize)]
struct ShutdownRequest<'a> {
    worker_id: &'a str,
}