// WeChat Mini Program Adapter - 微信小程序开发工具集成
//
// 集成微信开发者工具 CLI，支持小程序项目的：
// - 项目创建与初始化
// - 编译与预览
// - 上传与发布
// - 代码审查

use crate::agent_adapter::{
    health_tracker::HealthTracker,
    types::{
        ActualCost, AdapterType, AgentConfig, AgentError, AgentResult, AgentStatus, AgentTask,
        Capability, CostEstimate, HealthMetrics, Platform, ResultStatus, SceneType, TaskInput,
        TaskOutput, TokenEstimate, TokenUsage,
    },
    AgentAdapter,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// 微信小程序 Adapter
pub struct WeChatMiniProgramAdapter {
    id: String,
    name: String,
    config: Option<WeChatConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    devtools_path: Option<PathBuf>,
}

/// 微信开发者工具配置
#[derive(Debug, Clone)]
pub struct WeChatConfig {
    /// 开发者工具路径（默认自动检测）
    pub devtools_path: Option<PathBuf>,
    /// AppID（小程序ID）
    pub app_id: String,
    /// 项目路径
    pub project_path: PathBuf,
    /// 编译模式
    pub compile_mode: WeChatCompileMode,
    /// 上传版本号
    pub version: String,
    /// 上传描述
    pub description: String,
}

/// 编译模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeChatCompileMode {
    /// 开发版（默认）
    Development,
    /// 体验版
    Experience,
    /// 正式版
    Production,
}

impl WeChatCompileMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            WeChatCompileMode::Development => "development",
            WeChatCompileMode::Experience => "experience",
            WeChatCompileMode::Production => "production",
        }
    }
}

/// 小程序操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WeChatAction {
    /// 创建项目
    Create,
    /// 编译项目
    Compile,
    /// 预览（生成预览二维码）
    Preview,
    /// 上传代码
    Upload,
    /// 代码审查
    Audit,
    /// 构建npm
    BuildNpm,
}

impl WeChatMiniProgramAdapter {
    pub fn new() -> Self {
        Self {
            id: "wechat-miniprogram".to_string(),
            name: "微信小程序开发工具".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            devtools_path: None,
        }
    }

    /// 自动检测开发者工具路径
    fn detect_devtools_path() -> Option<PathBuf> {
        // Windows 默认路径
        let windows_paths = [
            "C:/Program Files (x86)/Tencent/微信web开发者工具/cli.bat",
            "C:/Program Files/Tencent/微信web开发者工具/cli.bat",
            "D:/Program Files (x86)/Tencent/微信web开发者工具/cli.bat",
            "D:/Program Files/Tencent/微信web开发者工具/cli.bat",
        ];

        for path in windows_paths {
            let pb = PathBuf::from(path);
            if pb.exists() {
                return Some(pb);
            }
        }

        None
    }

    /// 执行微信开发者工具 CLI 命令
    async fn execute_cli(
        &self,
        action: WeChatAction,
        args: &[String],
    ) -> Result<String, AgentError> {
        let cli_path = self
            .devtools_path
            .clone()
            .or_else(|| Self::detect_devtools_path())
            .ok_or_else(|| AgentError::ConfigurationError {
                message: "微信开发者工具未安装或路径未配置".to_string(),
            })?;

        let mut cmd = Command::new(&cli_path);

        // 根据操作类型设置命令
        match action {
            WeChatAction::Create => {
                cmd.arg("create");
                cmd.args(args);
            }
            WeChatAction::Compile => {
                cmd.arg("build");
                cmd.args(args);
            }
            WeChatAction::Preview => {
                cmd.arg("preview");
                cmd.args(args);
            }
            WeChatAction::Upload => {
                cmd.arg("upload");
                cmd.args(args);
            }
            WeChatAction::Audit => {
                cmd.arg("audit");
                cmd.args(args);
            }
            WeChatAction::BuildNpm => {
                cmd.arg("build-npm");
                cmd.args(args);
            }
        }

        let output = cmd.output().map_err(|e| AgentError::ExecutionError {
            message: format!("执行微信开发者工具失败: {}", e),
            retryable: false,
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AgentError::ExecutionError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
                retryable: true,
            })
        }
    }

    /// 创建小程序项目
    async fn create_project(
        &self,
        project_name: &str,
        path: &str,
    ) -> Result<AgentResult, AgentError> {
        let args = vec![
            "--project".to_string(),
            path.to_string(),
            "--name".to_string(),
            project_name.to_string(),
        ];

        let output = self.execute_cli(WeChatAction::Create, &args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("项目创建成功: {}\n{}", project_name, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::new(),
        })
    }

    /// 编译项目
    async fn compile_project(
        &self,
        project_path: &str,
        mode: WeChatCompileMode,
    ) -> Result<AgentResult, AgentError> {
        let args = vec![
            "--project".to_string(),
            project_path.to_string(),
            "--compile-condition".to_string(),
            format!("{{\"mode\": \"{}\"}}", mode.as_str()),
        ];

        let output = self.execute_cli(WeChatAction::Compile, &args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("编译成功\n{}", output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::new(),
        })
    }

    /// 预览项目（生成二维码）
    async fn preview_project(&self, project_path: &str) -> Result<AgentResult, AgentError> {
        let args = vec![
            "--project".to_string(),
            project_path.to_string(),
            "--qr-output".to_string(),
            "terminal".to_string(),
        ];

        let output = self.execute_cli(WeChatAction::Preview, &args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("预览二维码:\n{}", output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([("qr_code".to_string(), "generated".to_string())]),
        })
    }

    /// 上传代码
    async fn upload_project(
        &self,
        project_path: &str,
        version: &str,
        desc: &str,
    ) -> Result<AgentResult, AgentError> {
        let args = vec![
            "--project".to_string(),
            project_path.to_string(),
            "-v".to_string(),
            version.to_string(),
            "-d".to_string(),
            desc.to_string(),
        ];

        let output = self.execute_cli(WeChatAction::Upload, &args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("上传成功: 版本 {}\n{}", version, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::from([
                ("version".to_string(), version.to_string()),
                (
                    "upload_time".to_string(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        .to_string(),
                ),
            ]),
        })
    }
}

#[async_trait]
impl AgentAdapter for WeChatMiniProgramAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Cli
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "miniprogram_create".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 5000,
            },
            Capability {
                name: "miniprogram_compile".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 3000,
            },
            Capability {
                name: "miniprogram_preview".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 5000,
            },
            Capability {
                name: "miniprogram_upload".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 10000,
            },
            Capability {
                name: "miniprogram_audit".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
        ]
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        // 解析微信配置
        let app_id = config.metadata.get("app_id").cloned().unwrap_or_default();

        let project_path = config
            .metadata
            .get("project_path")
            .map(PathBuf::from)
            .unwrap_or_default();

        let devtools_path = config.metadata.get("devtools_path").map(PathBuf::from);

        self.devtools_path = devtools_path.or_else(|| Self::detect_devtools_path());

        self.config = Some(WeChatConfig {
            devtools_path: self.devtools_path.clone(),
            app_id,
            project_path,
            compile_mode: WeChatCompileMode::Development,
            version: "1.0.0".to_string(),
            description: "初始版本".to_string(),
        });

        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        if self.devtools_path.is_none() {
            return Err(AgentError::ConfigurationError {
                message: "微信开发者工具未安装".to_string(),
            });
        }

        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // 解析任务类型
        let action = match task.description.as_str() {
            s if s.contains("创建") || s.contains("create") => WeChatAction::Create,
            s if s.contains("编译") || s.contains("compile") || s.contains("build") => {
                WeChatAction::Compile
            }
            s if s.contains("预览") || s.contains("preview") => WeChatAction::Preview,
            s if s.contains("上传") || s.contains("upload") => WeChatAction::Upload,
            s if s.contains("审查") || s.contains("audit") => WeChatAction::Audit,
            s if s.contains("npm") || s.contains("build-npm") => WeChatAction::BuildNpm,
            _ => WeChatAction::Compile, // 默认编译
        };

        // 从任务输入获取参数
        let (project_path, project_name, version, desc) = match &task.input {
            TaskInput::Text(text) => {
                // 解析文本参数
                let parts: Vec<&str> = text.split_whitespace().collect();
                let path = parts.get(0).unwrap_or(&".").to_string();
                let name = parts.get(1).unwrap_or(&"miniprogram").to_string();
                let ver = parts.get(2).unwrap_or(&"1.0.0").to_string();
                let d = parts.get(3).unwrap_or(&"上传").to_string();
                (path, name, ver, d)
            }
            TaskInput::File(path) => (
                path.clone(),
                "miniprogram".to_string(),
                "1.0.0".to_string(),
                "上传".to_string(),
            ),
            _ => (
                self.config
                    .as_ref()
                    .map(|c| c.project_path.to_string_lossy().to_string())
                    .unwrap_or(".".to_string()),
                "miniprogram".to_string(),
                "1.0.0".to_string(),
                "上传".to_string(),
            ),
        };

        // 执行对应操作
        let result = match action {
            WeChatAction::Create => self.create_project(&project_name, &project_path).await?,
            WeChatAction::Compile => {
                self.compile_project(&project_path, WeChatCompileMode::Development)
                    .await?
            }
            WeChatAction::Preview => self.preview_project(&project_path).await?,
            WeChatAction::Upload => self.upload_project(&project_path, &version, &desc).await?,
            WeChatAction::Audit => {
                let args = vec!["--project".to_string(), project_path.clone()];
                let output = self.execute_cli(WeChatAction::Audit, &args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("审查结果:\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost {
                        amount: 0.0,
                        currency: "CNY".to_string(),
                        token_usage: TokenUsage {
                            input_tokens: 0,
                            output_tokens: output.len() as u64 / 4,
                            total_tokens: output.len() as u64 / 4,
                        },
                    },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                }
            }
            WeChatAction::BuildNpm => {
                let args = vec!["--project".to_string(), project_path.clone()];
                let output = self.execute_cli(WeChatAction::BuildNpm, &args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("npm构建成功:\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost {
                        amount: 0.0,
                        currency: "CNY".to_string(),
                        token_usage: TokenUsage {
                            input_tokens: 0,
                            output_tokens: output.len() as u64 / 4,
                            total_tokens: output.len() as u64 / 4,
                        },
                    },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    metadata: HashMap::new(),
                }
            }
        };

        Ok(result)
    }

    async fn cancel(&self, task_id: &str) -> Result<(), AgentError> {
        // 微信开发者工具 CLI 不支持取消操作
        Ok(())
    }

    fn status(&self) -> crate::agent_adapter::types::AgentStatus {
        let metrics = self.health_tracker.get_metrics();
        if metrics.health_score > 80.0 {
            crate::agent_adapter::types::AgentStatus::Idle
        } else {
            crate::agent_adapter::types::AgentStatus::Error {
                message: "健康分数过低".to_string(),
                error_count: metrics.error_count,
            }
        }
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate {
        // 微信小程序开发工具无 token 成本
        CostEstimate {
            min_cost: 0.0,
            max_cost: 0.0,
            currency: "CNY".to_string(),
            breakdown: HashMap::new(),
            token_estimate: TokenEstimate {
                input_tokens: 0,
                output_tokens: 500,
                total_tokens: 500,
            },
        }
    }

    fn actual_cost(&self, task_id: &str) -> Option<ActualCost> {
        None
    }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history
            .iter()
            .rev()
            .take(last_n as usize)
            .cloned()
            .collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        if result.status == ResultStatus::Success {
            self.health_tracker.record_success(result.duration_ms);
        } else {
            self.health_tracker.record_error();
        }

        self.token_history.push(TokenUsage {
            input_tokens: result.input_tokens,
            output_tokens: result.output_tokens,
            total_tokens: result.total_tokens,
        });

        // 保留最近100条记录
        if self.token_history.len() > 100 {
            self.token_history.remove(0);
        }
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        let metrics = self.health_tracker.get_metrics();
        matches!(
            metrics.circuit_breaker_state,
            crate::agent_adapter::types::CircuitBreakerState::Closed
                | crate::agent_adapter::types::CircuitBreakerState::HalfOpen { .. }
        )
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

impl Default for WeChatMiniProgramAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wechat_adapter_creation() {
        let adapter = WeChatMiniProgramAdapter::new();
        assert_eq!(adapter.id(), "wechat-miniprogram");
        assert_eq!(adapter.name(), "微信小程序开发工具");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_wechat_capabilities() {
        let adapter = WeChatMiniProgramAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.len() >= 5);
        assert!(caps.iter().any(|c| c.name == "miniprogram_create"));
        assert!(caps.iter().any(|c| c.name == "miniprogram_compile"));
    }

    #[test]
    fn test_compile_mode() {
        assert_eq!(WeChatCompileMode::Development.as_str(), "development");
        assert_eq!(WeChatCompileMode::Experience.as_str(), "experience");
        assert_eq!(WeChatCompileMode::Production.as_str(), "production");
    }

    #[test]
    fn test_health_tracker_initial() {
        let adapter = WeChatMiniProgramAdapter::new();
        let health = adapter.health_tracker.get_metrics();
        assert_eq!(health.health_score, 100.0);
    }

    #[tokio::test]
    async fn test_validate_config_without_devtools() {
        let adapter = WeChatMiniProgramAdapter::new();
        // 没有 devtools 配置时会失败
        assert!(adapter.validate_config().await.is_err());
    }
}
