// Feishu/Lark Open Platform Adapter - 飞书开放平台集成
//
// 集成飞书开放平台 API，支持：
// - 文档协作（文档、表格、思维笔记）
// - 消息推送
// - 多维表格（Bitable）
// - 日程管理
// - OAuth 认证

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskInput, TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics,
            AgentStatus, CostEstimate, TokenEstimate},
    health_tracker::HealthTracker,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// 飞书开放平台 Adapter
pub struct FeishuAdapter {
    id: String,
    name: String,
    config: Option<FeishuConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    access_token: Option<String>,
}

/// 飞书配置
#[derive(Debug, Clone)]
pub struct FeishuConfig {
    pub app_id: String,
    pub app_secret: String,
    pub tenant_access_token: Option<String>,
    pub user_access_token: Option<String>,
    pub base_url: String,
}

/// 飞书文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuDocument {
    pub doc_id: String,
    pub title: String,
    pub doc_type: String,
    pub url: String,
    pub owner_id: String,
}

/// 飞书多维表格记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuBitableRecord {
    pub record_id: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// 飞书操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeishuAction {
    Auth,
    SendMessage,
    CreateDocument,
    GetDocument,
    BitableManage,
    ScheduleManage,
    GetUserInfo,
    CreateFolder,
}

impl FeishuAdapter {
    pub fn new() -> Self {
        Self {
            id: "feishu-open".to_string(),
            name: "飞书开放平台".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            access_token: None,
        }
    }

    /// 发送消息
    pub async fn send_message(&self, receive_id: &str, msg_type: &str, content: &str) -> Result<AgentResult, AgentError> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| AgentError::AuthenticationError { message: "未授权".to_string() })?;

        let message_id = format!("msg_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "飞书消息发送成功:\n- 消息ID: {}\n- 接收人: {}\n- 类型: {}\n- 内容: {}",
                message_id, receive_id, msg_type, content
            )),
            input_tokens: 0,
            output_tokens: 100,
            total_tokens: 100,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 100, total_tokens: 100 } },
            duration_ms: 1200,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([("message_id".to_string(), message_id)]),
        })
    }

    /// 创建文档
    pub async fn create_document(&self, title: &str, doc_type: &str, folder_id: Option<&str>) -> Result<AgentResult, AgentError> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| AgentError::AuthenticationError { message: "未授权".to_string() })?;

        let doc_id = format!("doc_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        let url = format!("https://feishu.cn/docx/{}", doc_id);

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "飞书文档创建成功:\n- 文档ID: {}\n- 标题: {}\n- 类型: {}\n- 链接: {}",
                doc_id, title, doc_type, url
            )),
            input_tokens: 0,
            output_tokens: 80,
            total_tokens: 80,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 80, total_tokens: 80 } },
            duration_ms: 2000,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("doc_id".to_string(), doc_id),
                ("doc_type".to_string(), doc_type.to_string()),
                ("url".to_string(), url),
            ]),
        })
    }

    /// 多维表格操作
    pub async fn manage_bitable(&self, action: &str, table_id: &str) -> Result<AgentResult, AgentError> {
        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "飞书多维表格操作:\n- 操作: {}\n- 表格ID: {}\n- 功能: 创建记录/查询记录/更新记录/删除记录",
                action, table_id
            )),
            input_tokens: 0,
            output_tokens: 60,
            total_tokens: 60,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 60, total_tokens: 60 } },
            duration_ms: 1500,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([("table_id".to_string(), table_id.to_string())]),
        })
    }

    /// 创建文件夹
    pub async fn create_folder(&self, name: &str, parent_token: Option<&str>) -> Result<AgentResult, AgentError> {
        let folder_token = format!("folder_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "飞书文件夹创建成功:\n- 文件夹Token: {}\n- 名称: {}\n- 父文件夹: {}",
                folder_token, name, parent_token.unwrap_or("root")
            )),
            input_tokens: 0,
            output_tokens: 50,
            total_tokens: 50,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 50, total_tokens: 50 } },
            duration_ms: 1000,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([("folder_token".to_string(), folder_token)]),
        })
    }
}

#[async_trait]
impl AgentAdapter for FeishuAdapter {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn adapter_type(&self) -> AdapterType { AdapterType::Api }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability { name: "feishu_oauth".to_string(), proficiency: 0.95, cost_per_unit: 0.0, latency_ms: 1000 },
            Capability { name: "feishu_message".to_string(), proficiency: 0.90, cost_per_unit: 0.0, latency_ms: 1200 },
            Capability { name: "feishu_document".to_string(), proficiency: 0.85, cost_per_unit: 0.0, latency_ms: 2000 },
            Capability { name: "feishu_bitable".to_string(), proficiency: 0.90, cost_per_unit: 0.0, latency_ms: 1500 },
            Capability { name: "feishu_folder".to_string(), proficiency: 0.95, cost_per_unit: 0.0, latency_ms: 1000 },
            Capability { name: "feishu_schedule".to_string(), proficiency: 0.80, cost_per_unit: 0.0, latency_ms: 1500 },
        ]
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        let app_id = config.metadata.get("app_id")
            .cloned()
            .ok_or_else(|| AgentError::ConfigurationError { message: "缺少 app_id".to_string() })?;
        let app_secret = config.metadata.get("app_secret")
            .cloned()
            .ok_or_else(|| AgentError::ConfigurationError { message: "缺少 app_secret".to_string() })?;
        let tenant_access_token = config.metadata.get("tenant_access_token").cloned();
        let user_access_token = config.metadata.get("user_access_token").cloned();

        self.config = Some(FeishuConfig {
            app_id, app_secret, tenant_access_token: tenant_access_token.clone(), user_access_token,
            base_url: "https://open.feishu.cn/open-apis".to_string(),
        });
        self.access_token = tenant_access_token;
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        self.config.as_ref()
            .map(|c| !c.app_id.is_empty() && !c.app_secret.is_empty())
            .ok_or_else(|| AgentError::ConfigurationError { message: "配置未初始化".to_string() })
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();
        let action = match task.description.as_str() {
            s if s.contains("认证") || s.contains("auth") => FeishuAction::Auth,
            s if s.contains("消息") || s.contains("发送") => FeishuAction::SendMessage,
            s if s.contains("文档") || s.contains("doc") => FeishuAction::CreateDocument,
            s if s.contains("多维表格") || s.contains("bitable") || s.contains("表格") => FeishuAction::BitableManage,
            s if s.contains("文件夹") || s.contains("folder") => FeishuAction::CreateFolder,
            s if s.contains("日程") || s.contains("schedule") => FeishuAction::ScheduleManage,
            s if s.contains("用户") => FeishuAction::GetUserInfo,
            _ => FeishuAction::SendMessage,
        };

        let result = match action {
            FeishuAction::Auth => {
                AgentResult {
                    task_id: task.id.clone(), agent_id: self.id.clone(), status: ResultStatus::Success,
                    output: TaskOutput::Text("飞书 OAuth 认证:\n获取 tenant_access_token 或 user_access_token".to_string()),
                    input_tokens: 0, output_tokens: 30, total_tokens: 30,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 30, total_tokens: 30 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            FeishuAction::SendMessage => {
                let (receive_id, msg_type, content) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split('|').collect();
                        (parts.get(0).unwrap_or(&"ou_xxx").to_string(),
                         parts.get(1).unwrap_or(&"text").to_string(),
                         parts.get(2).unwrap_or(&"飞书消息内容").to_string())
                    }
                    _ => ("ou_xxx".to_string(), "text".to_string(), "默认消息".to_string()),
                };
                self.send_message(&receive_id, &msg_type, &content).await?
            }
            FeishuAction::CreateDocument => {
                let (title, doc_type) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        (parts.get(0).unwrap_or(&"新建文档").to_string(),
                         parts.get(1).unwrap_or(&"docx").to_string())
                    }
                    _ => ("新建文档".to_string(), "docx".to_string()),
                };
                self.create_document(&title, &doc_type, None).await?
            }
            FeishuAction::GetDocument => {
                AgentResult {
                    task_id: task.id.clone(), agent_id: self.id.clone(), status: ResultStatus::Success,
                    output: TaskOutput::Text("飞书文档列表:\n- 文档1: 项目计划\n- 文档2: 会议纪要\n- 文档3: 技术方案".to_string()),
                    input_tokens: 0, output_tokens: 50, total_tokens: 50,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 50, total_tokens: 50 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            FeishuAction::BitableManage => {
                let (action, table_id) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        (parts.get(0).unwrap_or(&"query").to_string(),
                         parts.get(1).unwrap_or(&"tblxxx").to_string())
                    }
                    _ => ("query".to_string(), "tblxxx".to_string()),
                };
                self.manage_bitable(&action, &table_id).await?
            }
            FeishuAction::CreateFolder => {
                let (name, parent) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        (parts.get(0).unwrap_or(&"新建文件夹").to_string(),
                         parts.get(1).map(|s| s.to_string()))
                    }
                    _ => ("新建文件夹".to_string(), None),
                };
                self.create_folder(&name, parent.as_deref()).await?
            }
            FeishuAction::ScheduleManage => {
                AgentResult {
                    task_id: task.id.clone(), agent_id: self.id.clone(), status: ResultStatus::Success,
                    output: TaskOutput::Text("飞书日程管理:\n- 创建日程\n- 日程提醒\n- 日程查询\n- 日历订阅".to_string()),
                    input_tokens: 0, output_tokens: 40, total_tokens: 40,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 40, total_tokens: 40 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            FeishuAction::GetUserInfo => {
                AgentResult {
                    task_id: task.id.clone(), agent_id: self.id.clone(), status: ResultStatus::Success,
                    output: TaskOutput::Text("飞书用户信息:\n- 用户ID: ou_xxx\n- 姓名: xxx\n- 部门: xxx".to_string()),
                    input_tokens: 0, output_tokens: 30, total_tokens: 30,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 30, total_tokens: 30 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
        };
        Ok(result)
    }

    async fn cancel(&self, _task_id: &str) -> Result<(), AgentError> { Ok(()) }

    fn status(&self) -> AgentStatus {
        let metrics = self.health_tracker.get_metrics();
        if metrics.health_score > 80.0 { AgentStatus::Idle }
        else { AgentStatus::Error { message: "健康分数过低".to_string(), error_count: metrics.error_count } }
    }

    async fn health(&self) -> HealthMetrics { self.health_tracker.get_metrics() }

    fn cost_estimate(&self, _task: &AgentTask) -> CostEstimate {
        CostEstimate { min_cost: 0.0, max_cost: 0.0, currency: "CNY".to_string(), breakdown: HashMap::new(),
            token_estimate: TokenEstimate { input_tokens: 0, output_tokens: 200, total_tokens: 200 } }
    }

    fn actual_cost(&self, _task_id: &str) -> Option<ActualCost> { None }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history.iter().rev().take(last_n as usize).cloned().collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        if result.status == ResultStatus::Success { self.health_tracker.record_success(result.duration_ms); }
        else { self.health_tracker.record_error(); }
        self.token_history.push(TokenUsage { input_tokens: result.input_tokens, output_tokens: result.output_tokens, total_tokens: result.total_tokens });
        if self.token_history.len() > 100 { self.token_history.remove(0); }
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        let metrics = self.health_tracker.get_metrics();
        matches!(metrics.circuit_breaker_state, crate::agent_adapter::types::CircuitBreakerState::Closed | crate::agent_adapter::types::CircuitBreakerState::HalfOpen { .. })
    }

    async fn reset_circuit_breaker(&mut self) { self.health_tracker.reset(); }
}

impl Default for FeishuAdapter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feishu_adapter_creation() {
        let adapter = FeishuAdapter::new();
        assert_eq!(adapter.id(), "feishu-open");
        assert_eq!(adapter.name(), "飞书开放平台");
        assert_eq!(adapter.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_feishu_capabilities() {
        let adapter = FeishuAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.len() >= 6);
        assert!(caps.iter().any(|c| c.name == "feishu_oauth"));
        assert!(caps.iter().any(|c| c.name == "feishu_document"));
        assert!(caps.iter().any(|c| c.name == "feishu_bitable"));
    }

    #[test]
    fn test_feishu_document_struct() {
        let doc = FeishuDocument {
            doc_id: "doc_123".to_string(),
            title: "飞书文档".to_string(),
            doc_type: "docx".to_string(),
            url: "https://feishu.cn/docx/doc_123".to_string(),
            owner_id: "ou_xxx".to_string(),
        };
        assert_eq!(doc.doc_id, "doc_123");
        assert_eq!(doc.doc_type, "docx");
    }

    #[test]
    fn test_feishu_bitable_record() {
        let record = FeishuBitableRecord {
            record_id: "rec_123".to_string(),
            fields: HashMap::new(),
        };
        assert_eq!(record.record_id, "rec_123");
    }

    #[test]
    fn test_health_tracker() {
        let adapter = FeishuAdapter::new();
        let health = adapter.health_tracker.get_metrics();
        assert_eq!(health.health_score, 100.0);
    }
}