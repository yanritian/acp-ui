// Kubernetes Adapter - K8s 分布式部署支持
//
// 集成 kubectl CLI，支持：
// - Deployment 创建与管理
// - Service 配置
// - Ingress 路由
// - ConfigMap/Secret 管理
// - HPA 自动扩缩容
// - YAML 配置生成

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskInput, TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics,
            AgentStatus, CostEstimate, TokenEstimate},
    health_tracker::HealthTracker,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Kubernetes Adapter
pub struct KubernetesAdapter {
    id: String,
    name: String,
    config: Option<KubernetesConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
}

/// Kubernetes 配置
#[derive(Debug, Clone)]
pub struct KubernetesConfig {
    /// Namespace
    pub namespace: String,
    /// Deployment 名称
    pub deployment_name: String,
    /// 容器镜像
    pub image: String,
    /// 副本数
    pub replicas: u32,
    /// 容器端口
    pub container_port: u32,
    /// Service 端口
    pub service_port: u32,
    /// Service 类型
    pub service_type: ServiceType,
    /// 资源限制
    pub resources: ResourceLimits,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
    /// Ingress 配置
    pub ingress: Option<IngressConfig>,
}

/// Service 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName,
}

impl ServiceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceType::ClusterIP => "ClusterIP",
            ServiceType::NodePort => "NodePort",
            ServiceType::LoadBalancer => "LoadBalancer",
            ServiceType::ExternalName => "ExternalName",
        }
    }
}

/// 资源限制
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_request: "100m".to_string(),
            cpu_limit: "500m".to_string(),
            memory_request: "128Mi".to_string(),
            memory_limit: "512Mi".to_string(),
        }
    }
}

/// Ingress 配置
#[derive(Debug, Clone)]
pub struct IngressConfig {
    pub host: String,
    pub path: String,
    pub path_type: String,
    pub tls_secret: Option<String>,
}

/// K8s 操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KubernetesAction {
    /// 生成 YAML 配置
    GenerateYaml,
    /// 应用配置
    Apply,
    /// 删除资源
    Delete,
    /// 获取资源状态
    Get,
    /// 查看 Pod 日志
    Logs,
    /// 扩缩容
    Scale,
    /// 查看事件
    Events,
    /// 端口转发
    PortForward,
}

impl KubernetesAdapter {
    pub fn new() -> Self {
        Self {
            id: "kubernetes".to_string(),
            name: "Kubernetes 部署引擎".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
        }
    }

    /// 检查 kubectl 是否安装
    fn check_kubectl_installed() -> Result<bool, AgentError> {
        let output = Command::new("kubectl")
            .arg("version")
            .arg("--client")
            .output()
            .map_err(|e| AgentError::ConfigurationError {
                message: format!("kubectl 未安装: {}", e)
            })?;

        Ok(output.status.success())
    }

    /// 执行 kubectl 命令
    async fn execute_kubectl(&self, args: &[String]) -> Result<String, AgentError> {
        let mut cmd = Command::new("kubectl");
        cmd.args(args);

        let output = cmd.output()
            .map_err(|e| AgentError::ExecutionError {
                message: format!("执行 kubectl 命令失败: {}", e),
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

    /// 生成 Deployment YAML
    fn generate_deployment_yaml(&self, config: &KubernetesConfig) -> String {
        format!(
            "apiVersion: apps/v1\n\
             kind: Deployment\n\
             metadata:\n\
               name: {}\n\
               namespace: {}\n\
             spec:\n\
               replicas: {}\n\
               selector:\n\
                 matchLabels:\n\
                   app: {}\n\
               template:\n\
                 metadata:\n\
                   labels:\n\
                     app: {}\n\
                 spec:\n\
                   containers:\n\
                   - name: {}\n\
                     image: {}\n\
                     ports:\n\
                     - containerPort: {}\n\
                     resources:\n\
                       requests:\n\
                         cpu: \"{}\"\n\
                         memory: \"{}\"\n\
                       limits:\n\
                         cpu: \"{}\"\n\
                         memory: \"{}\"\n\
                     env:\n\
                     - name: ENV\n\
                       value: production\n",
            config.deployment_name,
            config.namespace,
            config.replicas,
            config.deployment_name,
            config.deployment_name,
            config.deployment_name,
            config.image,
            config.container_port,
            config.resources.cpu_request,
            config.resources.memory_request,
            config.resources.cpu_limit,
            config.resources.memory_limit
        )
    }

    /// 生成 Service YAML
    fn generate_service_yaml(&self, config: &KubernetesConfig) -> String {
        format!(
            "apiVersion: v1\n\
             kind: Service\n\
             metadata:\n\
               name: {}-service\n\
               namespace: {}\n\
             spec:\n\
               type: {}\n\
               selector:\n\
                 app: {}\n\
               ports:\n\
               - port: {}\n\
                 targetPort: {}\n\
                 protocol: TCP\n",
            config.deployment_name,
            config.namespace,
            config.service_type.as_str(),
            config.deployment_name,
            config.service_port,
            config.container_port
        )
    }

    /// 生成 Ingress YAML
    fn generate_ingress_yaml(&self, config: &KubernetesConfig, ingress: &IngressConfig) -> String {
        let tls_section = if let Some(secret) = &ingress.tls_secret {
            format!(
                "  tls:\n\
                 - hosts:\n\
                   - {}\n\
                   secretName: {}\n",
                ingress.host, secret
            )
        } else {
            "".to_string()
        };

        format!(
            "apiVersion: networking.k8s.io/v1\n\
             kind: Ingress\n\
             metadata:\n\
               name: {}-ingress\n\
               namespace: {}\n\
             spec:\n\
             {}\
               rules:\n\
               - host: {}\n\
                 http:\n\
                   paths:\n\
                   - path: {}\n\
                     pathType: {}\n\
                     backend:\n\
                       service:\n\
                         name: {}-service\n\
                         port:\n\
                           number: {}\n",
            config.deployment_name,
            config.namespace,
            tls_section,
            ingress.host,
            ingress.path,
            ingress.path_type,
            config.deployment_name,
            config.service_port
        )
    }

    /// 生成完整 K8s YAML 配置
    async fn generate_full_yaml(&self, config: &KubernetesConfig) -> Result<AgentResult, AgentError> {
        let deployment_yaml = self.generate_deployment_yaml(config);
        let service_yaml = self.generate_service_yaml(config);

        let full_yaml = if let Some(ingress) = &config.ingress {
            let ingress_yaml = self.generate_ingress_yaml(config, ingress);
            format!("---\n{}\n---\n{}\n---\n{}", deployment_yaml, service_yaml, ingress_yaml)
        } else {
            format!("---\n{}\n---\n{}", deployment_yaml, service_yaml)
        };

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(full_yaml.clone()),
            input_tokens: 0,
            output_tokens: full_yaml.len() as u64 / 4,
            total_tokens: full_yaml.len() as u64 / 4,
            cost: ActualCost {
                amount: 0.0,
                currency: "CNY".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: full_yaml.len() as u64 / 4,
                    total_tokens: full_yaml.len() as u64 / 4,
                },
            },
            duration_ms: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("namespace".to_string(), config.namespace.clone()),
                ("deployment".to_string(), config.deployment_name.clone()),
            ]),
        })
    }

    /// 应用 K8s 配置
    async fn apply_config(&self, yaml_content: &str, namespace: &str) -> Result<AgentResult, AgentError> {
        let args = vec!["apply".to_string(), "-f".to_string(), "-".to_string(), "-n".to_string(), namespace.to_string()];

        // 使用 stdin 传入 YAML
        let mut cmd = Command::new("kubectl");
        cmd.args(&args);
        cmd.stdin(std::process::Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| AgentError::ExecutionError {
                message: format!("执行 kubectl apply 失败: {}", e),
                retryable: false,
            })?;

        // 写入 YAML 到 stdin
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(yaml_content.as_bytes())
                .map_err(|e| AgentError::ExecutionError {
                    message: format!("写入 YAML 失败: {}", e),
                    retryable: false,
                })?;
        }

        let output = child.wait_with_output()
            .map_err(|e| AgentError::ExecutionError {
                message: format!("等待 kubectl 执行失败: {}", e),
                retryable: false,
            })?;

        if output.status.success() {
            Ok(AgentResult {
                task_id: uuid::Uuid::new_v4().to_string(),
                agent_id: self.id.clone(),
                status: ResultStatus::Success,
                output: TaskOutput::Text(String::from_utf8_lossy(&output.stdout).to_string()),
                input_tokens: 0,
                output_tokens: output.stdout.len() as u64 / 4,
                total_tokens: output.stdout.len() as u64 / 4,
                cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.stdout.len() as u64 / 4, total_tokens: output.stdout.len() as u64 / 4 } },
                duration_ms: 0,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                metadata: HashMap::new(),
            })
        } else {
            Err(AgentError::ExecutionError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
                retryable: true,
            })
        }
    }

    /// 获取 Deployment 状态
    async fn get_deployment_status(&self, name: &str, namespace: &str) -> Result<AgentResult, AgentError> {
        let args = vec!["get".to_string(), "deployment".to_string(), name.to_string(), "-n".to_string(), namespace.to_string()];
        let output = self.execute_kubectl(&args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("Deployment 状态:\n{}", output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
            duration_ms: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("deployment".to_string(), name.to_string()),
                ("namespace".to_string(), namespace.to_string()),
            ]),
        })
    }

    /// 扩缩容
    async fn scale_deployment(&self, name: &str, namespace: &str, replicas: u32) -> Result<AgentResult, AgentError> {
        let args = vec![
            "scale".to_string(),
            "deployment".to_string(),
            name.to_string(),
            "-n".to_string(),
            namespace.to_string(),
            "--replicas=".to_string() + &replicas.to_string(),
        ];
        let output = self.execute_kubectl(&args).await?;

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!("扩缩容成功: {} -> {} replicas\n{}", name, replicas, output)),
            input_tokens: 0,
            output_tokens: output.len() as u64 / 4,
            total_tokens: output.len() as u64 / 4,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
            duration_ms: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("deployment".to_string(), name.to_string()),
                ("replicas".to_string(), replicas.to_string()),
            ]),
        })
    }
}

#[async_trait]
impl AgentAdapter for KubernetesAdapter {
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
                name: "k8s_yaml_generate".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 1000,
            },
            Capability {
                name: "k8s_apply".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 5000,
            },
            Capability {
                name: "k8s_deploy".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
            Capability {
                name: "k8s_scale".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 3000,
            },
            Capability {
                name: "k8s_logs".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 2000,
            },
            Capability {
                name: "k8s_ingress".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 5000,
            },
        ]
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        Self::check_kubectl_installed()?;

        let namespace = config.metadata.get("namespace").cloned().unwrap_or("default".to_string());
        let deployment_name = config.metadata.get("deployment_name").cloned().unwrap_or("app".to_string());
        let image = config.metadata.get("image").cloned().unwrap_or("app:latest".to_string());
        let replicas = config.metadata.get("replicas")
            .and_then(|r| r.parse::<u32>().ok())
            .unwrap_or(3);
        let container_port = config.metadata.get("container_port")
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(8080);
        let service_port = config.metadata.get("service_port")
            .and_then(|p| p.parse::<u32>().ok())
            .unwrap_or(80);
        let service_type = config.metadata.get("service_type")
            .map(|t| match t.as_str() {
                "NodePort" => ServiceType::NodePort,
                "LoadBalancer" => ServiceType::LoadBalancer,
                _ => ServiceType::ClusterIP,
            })
            .unwrap_or(ServiceType::ClusterIP);

        let ingress_host = config.metadata.get("ingress_host").cloned();
        let ingress_path = config.metadata.get("ingress_path").cloned().unwrap_or("/".to_string());

        let ingress = ingress_host.map(|host| IngressConfig {
            host,
            path: ingress_path,
            path_type: "Prefix".to_string(),
            tls_secret: None,
        });

        self.config = Some(KubernetesConfig {
            namespace,
            deployment_name,
            image,
            replicas,
            container_port,
            service_port,
            service_type,
            resources: ResourceLimits::default(),
            env_vars: HashMap::new(),
            ingress,
        });

        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        Self::check_kubectl_installed()
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        let action = match task.description.as_str() {
            s if s.contains("生成") || s.contains("yaml") || s.contains("配置") => KubernetesAction::GenerateYaml,
            s if s.contains("应用") || s.contains("apply") || s.contains("部署") => KubernetesAction::Apply,
            s if s.contains("删除") || s.contains("delete") => KubernetesAction::Delete,
            s if s.contains("状态") || s.contains("get") => KubernetesAction::Get,
            s if s.contains("日志") || s.contains("logs") => KubernetesAction::Logs,
            s if s.contains("扩容") || s.contains("scale") => KubernetesAction::Scale,
            s if s.contains("事件") || s.contains("events") => KubernetesAction::Events,
            s if s.contains("端口") || s.contains("forward") => KubernetesAction::PortForward,
            _ => KubernetesAction::GenerateYaml,
        };

        let config = self.config.as_ref()
            .ok_or_else(|| AgentError::ConfigurationError { message: "K8s 配置未初始化".to_string() })?;

        let result = match action {
            KubernetesAction::GenerateYaml => self.generate_full_yaml(config).await?,
            KubernetesAction::Apply => {
                let yaml = self.generate_full_yaml(config).await?.output;
                let yaml_content = match yaml {
                    TaskOutput::Text(content) => content,
                    _ => "".to_string(),
                };
                self.apply_config(&yaml_content, &config.namespace).await?
            }
            KubernetesAction::Delete => {
                let args = vec!["delete".to_string(), "deployment".to_string(), config.deployment_name.clone(), "-n".to_string(), config.namespace.clone()];
                let output = self.execute_kubectl(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("删除成功\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            KubernetesAction::Get => self.get_deployment_status(&config.deployment_name, &config.namespace).await?,
            KubernetesAction::Scale => {
                let new_replicas = match &task.input {
                    TaskInput::Text(text) => text.trim().parse::<u32>().unwrap_or(5),
                    _ => 5,
                };
                self.scale_deployment(&config.deployment_name, &config.namespace, new_replicas).await?
            }
            KubernetesAction::Logs => {
                let args = vec!["logs".to_string(), format!("deployment/{}", config.deployment_name), "-n".to_string(), config.namespace.clone()];
                let output = self.execute_kubectl(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("Pod 日志:\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            KubernetesAction::Events => {
                let args = vec!["get".to_string(), "events".to_string(), "-n".to_string(), config.namespace.clone()];
                let output = self.execute_kubectl(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("集群事件:\n{}", output)),
                    input_tokens: 0,
                    output_tokens: output.len() as u64 / 4,
                    total_tokens: output.len() as u64 / 4,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: output.len() as u64 / 4, total_tokens: output.len() as u64 / 4 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            KubernetesAction::PortForward => {
                let (local_port, pod_port) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split(':').collect();
                        let lp = parts.get(0).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8080);
                        let pp = parts.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(80);
                        (lp, pp)
                    }
                    _ => (8080, 80),
                };
                let args = vec!["port-forward".to_string(), format!("deployment/{}", config.deployment_name), "-n".to_string(), config.namespace.clone(), format!("{}:{}", local_port, pod_port)];
                let output = self.execute_kubectl(&args).await?;
                AgentResult {
                    task_id: task.id.clone(),
                    agent_id: self.id.clone(),
                    status: ResultStatus::Success,
                    output: TaskOutput::Text(format!("端口转发成功: {} -> {}\n{}", local_port, pod_port, output)),
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

impl Default for KubernetesAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubernetes_adapter_creation() {
        let adapter = KubernetesAdapter::new();
        assert_eq!(adapter.id(), "kubernetes");
        assert_eq!(adapter.name(), "Kubernetes 部署引擎");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_kubernetes_capabilities() {
        let adapter = KubernetesAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.len() >= 6);
        assert!(caps.iter().any(|c| c.name == "k8s_yaml_generate"));
        assert!(caps.iter().any(|c| c.name == "k8s_apply"));
        assert!(caps.iter().any(|c| c.name == "k8s_scale"));
    }

    #[test]
    fn test_service_type() {
        assert_eq!(ServiceType::ClusterIP.as_str(), "ClusterIP");
        assert_eq!(ServiceType::NodePort.as_str(), "NodePort");
        assert_eq!(ServiceType::LoadBalancer.as_str(), "LoadBalancer");
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.cpu_request, "100m");
        assert_eq!(limits.memory_request, "128Mi");
    }

    #[test]
    fn test_health_tracker() {
        let adapter = KubernetesAdapter::new();
        let health = adapter.health_tracker.get_metrics();
        assert_eq!(health.health_score, 100.0);
    }
}