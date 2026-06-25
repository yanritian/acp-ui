// Docker Adapter - 容器化部署支持
//
// 集成 Docker CLI，支持：
// - Dockerfile 生成
// - 镜像构建
// - 容器运行与管理
// - 镜像推送
// - compose 编排

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskInput, TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics,
            AgentStatus, CostEstimate, TokenEstimate},
    health_tracker::HealthTracker,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Docker Adapter
pub struct DockerAdapter {
    id: String,
    name: String,
    config: Option<DockerConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
}

/// Docker 配置
#[derive(Debug, Clone)]
pub struct DockerConfig {
    /// 镜像名称
    pub image_name: String,
    /// 镜像标签
    pub tag: String,
    /// Dockerfile 路径
    pub dockerfile_path: PathBuf,
    /// 构建上下文路径
    pub build_context: PathBuf,
    /// 容器名称
    pub container_name: String,
    /// 端口映射
    pub port_mappings: Vec<PortMapping>,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// Docker Registry
    pub registry: Option<String>,
}

/// 端口映射
#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

/// Docker 操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockerAction {
    /// 生成 Dockerfile
    GenerateDockerfile,
    /// 构建镜像
    BuildImage,
    /// 运行容器
    RunContainer,
    /// 停止容器
    StopContainer,
    /// 推送镜像
    PushImage,
    /// 删除镜像
    RemoveImage,
    /// 查看日志
    Logs,
    /// Compose up
    ComposeUp,
    /// Compose down
    ComposeDown,
}

impl DockerAdapter {
    pub fn new() -> Self {
        Self {
            id: "docker".to_string(),
            name: "Docker 容器引擎".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
        }
    }

    /// 检查 Docker 是否安装
    fn check_docker_installed() -> Result<bool, AgentError> {
        let output = Command::new("docker")
            .arg("--version")
            .output()
            .map_err(|e| AgentError::ConfigurationError {
                message: format!("Docker 未安装: {}", e)
            })?;

        Ok(output.status.success())
    }

    /// 执行 Docker CLI 命令
    async fn execute_docker(&self, args: &[String]) -> Result<String, AgentError> {
        let mut cmd = Command::new("docker");
        cmd.args(args);

        let output = cmd.output()
            .map_err(|e| AgentError::ExecutionError {
                message: format!("执行 Docker 命令失败: {}", e),
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

    /// 生成 Dockerfile（根据项目类型）
    async fn generate_dockerfile(&self, project_type: &str, base_image: &str, port: u16) -> Result<AgentResult, AgentError> {
        let dockerfile_content = match project_type.to_lowercase().as_str() {
            "rust" | "tauri" => {
                format!(
                    "# Rust/Tauri Dockerfile\n\
                     FROM {} as builder\n\
                     WORKDIR /app\n\
                     COPY . .\n\
                     RUN cargo build --release\n\
                     \n\
                     FROM debian:bookworm-slim\n\
                     WORKDIR /app\n\
                     COPY --from=builder /app/target/release/app /app/\n\
                     EXPOSE {}\n\
                     CMD [\"./app\"]\n",
                    base_image, port
                )
            }
            "node" | "vue" | "react" | "web" => {
                format!(
                    "# Node.js Web Dockerfile\n\
                     FROM {} as builder\n\
                     WORKDIR /app\n\
                     COPY package*.json ./\n\
                     RUN npm install\n\
                     COPY . .\n\
                     RUN npm run build\n\
                     \n\
                     FROM nginx:alpine\n\
                     COPY --from=builder /app/dist /usr/share/nginx/html\n\
                     EXPOSE {}\n\
                     CMD [\"nginx\", \"-g\", \"daemon off;\"]\n",
                    base_image, port
                )
            }
            "flutter" => {
                format!(
                    "# Flutter Web Dockerfile\n\
                     FROM {} as builder\n\
                     WORKDIR /app\n\
                     COPY . .\n\
                     RUN flutter build web --release\n\
                     \n\
                     FROM nginx:alpine\n\
                     COPY --from=builder /app/build/web /usr/share/nginx/html\n\
                     EXPOSE {}\n\
                     CMD [\"nginx\", \"-g\", \"daemon off;\"]\n",
                    base_image, port
                )
            }
            "python" => {
                format!(
                    "# Python Dockerfile\n\
                     FROM {}\n\
                     WORKDIR /app\n\
                     COPY requirements.txt .\n\
                     RUN pip install --no-cache-dir -r requirements.txt\n\
                     COPY . .\n\
                     EXPOSE {}\n\
                     CMD [\"python\", \"app.py\"]\n",
                    base_image, port
                )
            }
            _ => {
                format!(
                    "# Generic Dockerfile\n\
                     FROM {}\n\
                     WORKDIR /app\n\
                     COPY . .\n\
                     EXPOSE {}\n\
                     CMD [\"./start\"]\n",
                    base_image, port
                )
            }
        };

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(dockerfile_content.clone()),
            input_tokens: 0,
            output_tokens: dockerfile_content.len() as u64 / 4,
            total_tokens: dockerfile_content.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: dockerfile_content.len() as u64 / 4,
                    total_tokens: dockerfile_content.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("project_type".to_string(), project_type.to_string()),
                ("base_image".to_string(), base_image.to_string()),
                ("port".to_string(), port.to_string()),
            ]),
        })
    }

    /// 构建镜像
    async fn build_image(&self, tag: &str, context: &str, dockerfile: Option<&str>) -> Result<AgentResult, AgentError> {
        let mut args = vec!["build".to_string(), "-t".to_string(), tag.to_string()];

        if let Some(df) = dockerfile {
            args.push("-f".to_string());
            args.push(df.to_string());
        }

        args.push(context.to_string());

        let output = self.execute_docker(&args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("镜像构建成功: {}\n{}", tag, output)),
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
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("image_tag".to_string(), tag.to_string()),
                ("build_context".to_string(), context.to_string()),
            ]),
        })
    }

    /// 运行容器
    async fn run_container(&self, image: &str, name: &str, ports: &[PortMapping], env: &HashMap<String, String>) -> Result<AgentResult, AgentError> {
        let mut args = vec!["run".to_string(), "-d".to_string(), "--name".to_string(), name.to_string()];

        // 端口映射
        for pm in ports {
            args.push("-p".to_string());
            args.push(format!("{}:{}{}", pm.host_port, pm.container_port,
                if pm.protocol == Protocol::Udp { "/udp" } else { "" }));
        }

        // 环境变量
        for (key, value) in env {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, value));
        }

        args.push(image.to_string());

        let output = self.execute_docker(&args).await?;

        // 提取容器 ID
        let container_id = output.trim().lines().last().unwrap_or(&output).to_string();

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("容器运行成功: {} (ID: {})\n{}", name, container_id, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 },
            },
            duration_ms: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("container_id".to_string(), container_id),
                ("container_name".to_string(), name.to_string()),
                ("image".to_string(), image.to_string()),
            ]),
        })
    }

    /// 推送镜像
    async fn push_image(&self, image: &str, registry: Option<&str>) -> Result<AgentResult, AgentError> {
        let full_image = if let Some(reg) = registry {
            format!("{}{}", reg, image)
        } else {
            image.to_string()
        };

        let args = vec!["push".to_string(), full_image.clone()];
        let output = self.execute_docker(&args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("镜像推送成功: {}\n{}", full_image, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 },
            },
            duration_ms: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("pushed_image".to_string(), full_image),
            ]),
        })
    }

    /// Docker Compose up
    async fn compose_up(&self, compose_file: &str) -> Result<AgentResult, AgentError> {
        let args = vec!["compose".to_string(), "-f".to_string(), compose_file.to_string(), "up".to_string(), "-d".to_string()];
        let output = self.execute_docker(&args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("Compose 启动成功\n{}", output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 },
            },
            duration_ms: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("compose_file".to_string(), compose_file.to_string()),
            ]),
        })
    }
}

#[async_trait]
impl AgentAdapter for DockerAdapter {
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
                name: "dockerfile_generate".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 1000,
            },
            Capability {
                name: "docker_build".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 120000,
            },
            Capability {
                name: "docker_run".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 5000,
            },
            Capability {
                name: "docker_push".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 60000,
            },
            Capability {
                name: "docker_compose".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
            Capability {
                name: "container_management".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 2000,
            },
        ]
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        Self::check_docker_installed()?;

        let image_name = config.metadata.get("image_name").cloned().unwrap_or("app".to_string());
        let tag = config.metadata.get("tag").cloned().unwrap_or("latest".to_string());
        let dockerfile_path = config.metadata.get("dockerfile_path")
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from("Dockerfile"));
        let build_context = config.metadata.get("build_context")
            .map(PathBuf::from)
            .unwrap_or(PathBuf::from("."));
        let container_name = config.metadata.get("container_name").cloned().unwrap_or("app-container".to_string());
        let registry = config.metadata.get("registry").cloned();

        self.config = Some(DockerConfig {
            image_name,
            tag,
            dockerfile_path,
            build_context,
            container_name,
            port_mappings: vec![PortMapping { host_port: 8080, container_port: 80, protocol: Protocol::Tcp }],
            env_vars: HashMap::new(),
            registry,
        });

        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        Self::check_docker_installed()
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // 解析操作类型
        let action = match task.description.as_str() {
            s if s.contains("dockerfile") || s.contains("生成") => DockerAction::GenerateDockerfile,
            s if s.contains("构建") || s.contains("build") => DockerAction::BuildImage,
            s if s.contains("运行") || s.contains("run") => DockerAction::RunContainer,
            s if s.contains("停止") || s.contains("stop") => DockerAction::StopContainer,
            s if s.contains("推送") || s.contains("push") => DockerAction::PushImage,
            s if s.contains("删除") || s.contains("remove") || s.contains("rm") => DockerAction::RemoveImage,
            s if s.contains("日志") || s.contains("logs") => DockerAction::Logs,
            s if s.contains("compose") && s.contains("up") => DockerAction::ComposeUp,
            s if s.contains("compose") && s.contains("down") => DockerAction::ComposeDown,
            _ => DockerAction::BuildImage,
        };

        // 执行操作
        let result = match action {
            DockerAction::GenerateDockerfile => {
                let (project_type, base_image, port) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        let pt = parts.get(0).unwrap_or(&"rust");
                        let bi = parts.get(1).unwrap_or(&"rust:latest");
                        let p = parts.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8080);
                        (pt.to_string(), bi.to_string(), p)
                    }
                    _ => ("rust".to_string(), "rust:latest".to_string(), 8080),
                };
                self.generate_dockerfile(&project_type, &base_image, port).await?
            }
            DockerAction::BuildImage => {
                let config = self.config.as_ref();
                let tag = config.map(|c| format!("{}:{}", c.image_name, c.tag))
                    .unwrap_or("app:latest".to_string());
                let context = config.map(|c| c.build_context.to_string_lossy().to_string())
                    .unwrap_or(".".to_string());
                let dockerfile = config.map(|c| c.dockerfile_path.to_string_lossy().to_string());
                self.build_image(&tag, &context, dockerfile.as_deref()).await?
            }
            DockerAction::RunContainer => {
                let config = self.config.as_ref();
                let image = config.map(|c| format!("{}:{}", c.image_name, c.tag))
                    .unwrap_or("app:latest".to_string());
                let name = config.map(|c| c.container_name.clone())
                    .unwrap_or("app-container".to_string());
                let ports = config.map(|c| c.port_mappings.clone())
                    .unwrap_or(vec![PortMapping { host_port: 8080, container_port: 80, protocol: Protocol::Tcp }]);
                let env = config.map(|c| c.env_vars.clone())
                    .unwrap_or(HashMap::new());
                self.run_container(&image, &name, &ports, &env).await?
            }
            DockerAction::PushImage => {
                let config = self.config.as_ref();
                let image = config.map(|c| format!("{}:{}", c.image_name, c.tag))
                    .unwrap_or("app:latest".to_string());
                let registry = config.and_then(|c| c.registry.as_deref());
                self.push_image(&image, registry).await?
            }
            DockerAction::ComposeUp => {
                let compose_file = match &task.input {
                    TaskInput::File(path) => path.clone(),
                    TaskInput::Text(text) => text.trim().to_string(),
                    _ => "docker-compose.yml".to_string(),
                };
                self.compose_up(&compose_file).await?
            }
            DockerAction::StopContainer => {
                let name = self.config.as_ref().map(|c| c.container_name.clone())
                    .unwrap_or("app-container".to_string());
                let args = vec!["stop".to_string(), name.clone()];
                let output = self.execute_docker(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("容器停止成功: {}\n{}", name, output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            DockerAction::Logs => {
                let name = self.config.as_ref().map(|c| c.container_name.clone())
                    .unwrap_or("app-container".to_string());
                let args = vec!["logs".to_string(), name.clone()];
                let output = self.execute_docker(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("容器日志:\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            DockerAction::ComposeDown => {
                let args = vec!["compose".to_string(), "down".to_string()];
                let output = self.execute_docker(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("Compose 关闭成功\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            DockerAction::RemoveImage => {
                let tag = self.config.as_ref()
                    .map(|c| format!("{}:{}", c.image_name, c.tag))
                    .unwrap_or("app:latest".to_string());
                let args = vec!["rmi".to_string(), tag.clone()];
                let output = self.execute_docker(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("镜像删除成功: {}\n{}", tag, output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
        };

        Ok(result)
    }

    async fn cancel(&self, task_id: &str) -> Result<(), AgentError> {
        Ok(())
    }

    fn status(&self) -> AgentStatus {
        let metrics = self.health_tracker.get_metrics();
        if metrics.health_score > 80.0 {
            AgentStatus::Idle
        } else {
            AgentStatus::Error {
                message: "健康分数过低".to_string(),
                error_count: metrics.error_count,
            }
        }
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate {
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
        self.token_history.iter().rev().take(last_n as usize).cloned().collect()
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
        if self.token_history.len() > 100 {
            self.token_history.remove(0);
        }
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        let metrics = self.health_tracker.get_metrics();
        matches!(metrics.circuit_breaker_state, crate::agent_adapter::types::CircuitBreakerState::Closed | crate::agent_adapter::types::CircuitBreakerState::HalfOpen { .. })
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

impl Default for DockerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_adapter_creation() {
        let adapter = DockerAdapter::new();
        assert_eq!(adapter.id(), "docker");
        assert_eq!(adapter.name(), "Docker 容器引擎");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_docker_capabilities() {
        let adapter = DockerAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.len() >= 6);
        assert!(caps.iter().any(|c| c.name == "dockerfile_generate"));
        assert!(caps.iter().any(|c| c.name == "docker_build"));
        assert!(caps.iter().any(|c| c.name == "docker_compose"));
    }

    #[test]
    fn test_port_mapping() {
        let pm = PortMapping { host_port: 8080, container_port: 80, protocol: Protocol::Tcp };
        assert_eq!(pm.host_port, 8080);
        assert_eq!(pm.container_port, 80);
    }

    #[test]
    fn test_protocol() {
        assert!(Protocol::Tcp == Protocol::Tcp);
        assert!(Protocol::Udp != Protocol::Tcp);
    }

    #[test]
    fn test_health_tracker() {
        let adapter = DockerAdapter::new();
        let health = adapter.health_tracker.get_metrics();
        assert_eq!(health.health_score, 100.0);
    }
}