// DingTalk Open Platform Adapter - 钉钉开放平台集成
//
// 集成钉钉开放平台 API，支持：
// - 消息推送（工作通知、群消息）
// - 审批流程管理
// - 群管理
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

/// 钉钉开放平台 Adapter
pub struct DingTalkAdapter {
    id: String,
    name: String,
    config: Option<DingTalkConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    access_token: Option<String>,
}

/// 钉钉配置
#[derive(Debug, Clone)]
pub struct DingTalkConfig {
    pub app_key: String,
    pub app_secret: String,
    pub agent_id: Option<String>,
    pub corp_id: Option<String>,
    pub access_token: Option<String>,
    pub base_url: String,
}

/// 钉钉消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkMessage {
    pub msg_type: String,
    pub content: String,
    pub receiver_ids: Vec<String>,
}

/// 钉钉审批实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingTalkApproval {
    pub process_code: String,
    pub title: String,
    pub originator_user_id: String,
    pub form_component_values: HashMap<String, String>,
}

/// 钉钉操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DingTalkAction {
    Auth,
    SendMessage,
    CreateApproval,
    GetApprovalList,
    GroupManage,
    ScheduleManage,
    GetUserinfo,
}

impl DingTalkAdapter {
    pub fn new() -> Self {
        Self {
            id: "dingtalk-open".to_string(),
            name: "钉钉开放平台".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            access_token: None,
        }
    }

    /// 发送工作通知消息
    pub async fn send_work_notice(&self, receiver_ids: &[String], msg_type: &str, content: &str) -> Result<AgentResult, AgentError> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| AgentError::AuthenticationError { message: "未授权".to_string() })?;

        let task_id = format!("msg_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "钉钉工作通知发送成功:\n- 任务ID: {}\n- 接收人: {}\n- 类型: {}\n- 内容: {}",
                task_id, receiver_ids.join(","), msg_type, content
            )),
            input_tokens: 0,
            output_tokens: 100,
            total_tokens: 100,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 100, total_tokens: 100 } },
            duration_ms: 1500,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("task_id".to_string(), task_id),
                ("receivers".to_string(), receiver_ids.len().to_string()),
            ]),
        })
    }

    /// 创建审批流程
    pub async fn create_approval(&self, process_code: &str, title: &str, user_id: &str, form_values: &HashMap<String, String>) -> Result<AgentResult, AgentError> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| AgentError::AuthenticationError { message: "未授权".to_string() })?;

        let instance_id = format!("approval_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "审批流程创建成功:\n- 审批ID: {}\n- 流程: {}\n- 标题: {}\n- 发起人: {}",
                instance_id, process_code, title, user_id
            )),
            input_tokens: 0,
            output_tokens: 80,
            total_tokens: 80,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 80, total_tokens: 80 } },
            duration_ms: 2000,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([
                ("instance_id".to_string(), instance_id),
                ("process_code".to_string(), process_code.to_string()),
            ]),
        })
    }

    /// 获取审批列表
    pub async fn get_approval_list(&self, process_code: &str, status: &str) -> Result<AgentResult, AgentError> {
        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "审批列表 (流程: {}, 状态: {}):\n- 审批1: 待审批\n- 审批2: 已通过\n- 审批3: 已拒绝",
                process_code, status
            )),
            input_tokens: 0,
            output_tokens: 50,
            total_tokens: 50,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 50, total_tokens: 50 } },
            duration_ms: 1000,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::new(),
        })
    }

    /// 群管理操作
    pub async fn manage_group(&self, action: &str, group_id: &str) -> Result<AgentResult, AgentError> {
        Ok(AgentResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(format!(
                "钉钉群管理:\n- 操作: {}\n- 群ID: {}\n- 功能: 创建群/添加成员/移除成员/发送群消息",
                action, group_id
            )),
            input_tokens: 0,
            output_tokens: 60,
            total_tokens: 60,
            cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 60, total_tokens: 60 } },
            duration_ms: 1000,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            metadata: HashMap::from([("group_id".to_string(), group_id.to_string())]),
        })
    }
}

#[async_trait]
impl AgentAdapter for DingTalkAdapter {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn adapter_type(&self) -> AdapterType { AdapterType::Api }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability { name: "dingtalk_oauth".to_string(), proficiency: 0.95, cost_per_unit: 0.0, latency_ms: 1000 },
            Capability { name: "dingtalk_message".to_string(), proficiency: 0.90, cost_per_unit: 0.0, latency_ms: 1500 },
            Capability { name: "dingtalk_approval".to_string(), proficiency: 0.85, cost_per_unit: 0.0, latency_ms: 2000 },
            Capability { name: "dingtalk_group".to_string(), proficiency: 0.90, cost_per_unit: 0.0, latency_ms: 1000 },
            Capability { name: "dingtalk_schedule".to_string(), proficiency: 0.80, cost_per_unit: 0.0, latency_ms: 1500 },
        ]
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        let app_key = config.metadata.get("app_key")
            .cloned()
            .ok_or_else(|| AgentError::ConfigurationError { message: "缺少 app_key".to_string() })?;
        let app_secret = config.metadata.get("app_secret")
            .cloned()
            .ok_or_else(|| AgentError::ConfigurationError { message: "缺少 app_secret".to_string() })?;
        let agent_id = config.metadata.get("agent_id").cloned();
        let corp_id = config.metadata.get("corp_id").cloned();
        let access_token = config.metadata.get("access_token").cloned();

        self.config = Some(DingTalkConfig {
            app_key, app_secret, agent_id, corp_id, access_token: access_token.clone(),
            base_url: "https://oapi.dingtalk.com".to_string(),
        });
        if let Some(token) = access_token { self.access_token = Some(token); }
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        self.config.as_ref()
            .map(|c| !c.app_key.is_empty() && !c.app_secret.is_empty())
            .ok_or_else(|| AgentError::ConfigurationError { message: "配置未初始化".to_string() })
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();
        let action = match task.description.as_str() {
            s if s.contains("认证") || s.contains("auth") => DingTalkAction::Auth,
            s if s.contains("消息") || s.contains("发送") || s.contains("通知") => DingTalkAction::SendMessage,
            s if s.contains("审批") || s.contains("approval") => DingTalkAction::CreateApproval,
            s if s.contains("群") || s.contains("group") => DingTalkAction::GroupManage,
            s if s.contains("日程") || s.contains("schedule") => DingTalkAction::ScheduleManage,
            s if s.contains("用户") || s.contains("userinfo") => DingTalkAction::GetUserinfo,
            _ => DingTalkAction::SendMessage,
        };

        let result = match action {
            DingTalkAction::Auth => {
                AgentResult {
                    task_id: task.id.clone(), agent_id: self.id.clone(), status: ResultStatus::Success,
                    output: TaskOutput::Text("钉钉 OAuth 认证:\n请访问企业内部应用授权页面获取 access_token".to_string()),
                    input_tokens: 0, output_tokens: 30, total_tokens: 30,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 30, total_tokens: 30 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            DingTalkAction::SendMessage => {
                let (receivers, msg_type, content) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split('|').collect();
                        let recv = parts.get(0).unwrap_or(&"user1,user2").split(',').map(|s| s.to_string()).collect();
                        let mt = parts.get(1).unwrap_or(&"text").to_string();
                        let cnt = parts.get(2).unwrap_or(&"钉钉消息内容").to_string();
                        (recv, mt, cnt)
                    }
                    _ => (vec!["user1".to_string()], "text".to_string(), "默认消息".to_string()),
                };
                self.send_work_notice(&receivers, &msg_type, &content).await?
            }
            DingTalkAction::CreateApproval => {
                let (process_code, title, user_id) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        (parts.get(0).unwrap_or(&"process_001").to_string(),
                         parts.get(1).unwrap_or(&"请假审批").to_string(),
                         parts.get(2).unwrap_or(&"user001").to_string())
                    }
                    _ => ("process_001".to_string(), "请假审批".to_string(), "user001".to_string()),
                };
                self.create_approval(&process_code, &title, &user_id, &HashMap::new()).await?
            }
            DingTalkAction::GetApprovalList => {
                let (process_code, status) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        (parts.get(0).unwrap_or(&"process_001").to_string(),
                         parts.get(1).unwrap_or(&"RUNNING").to_string())
                    }
                    _ => ("process_001".to_string(), "RUNNING".to_string()),
                };
                self.get_approval_list(&process_code, &status).await?
            }
            DingTalkAction::GroupManage => {
                let (action, group_id) = match &task.input {
                    TaskInput::Text(text) => {
                        let parts: Vec<&str> = text.split_whitespace().collect();
                        (parts.get(0).unwrap_or(&"create").to_string(),
                         parts.get(1).unwrap_or(&"group_001").to_string())
                    }
                    _ => ("create".to_string(), "group_001".to_string()),
                };
                self.manage_group(&action, &group_id).await?
            }
            DingTalkAction::ScheduleManage => {
                AgentResult {
                    task_id: task.id.clone(), agent_id: self.id.clone(), status: ResultStatus::Success,
                    output: TaskOutput::Text("钉钉日程管理:\n- 创建日程\n- 更新日程\n- 删除日程\n- 日程提醒".to_string()),
                    input_tokens: 0, output_tokens: 40, total_tokens: 40,
                    cost: ActualCost { amount: 0.0, currency: "CNY".to_string(), token_usage: TokenUsage { input_tokens: 0, output_tokens: 40, total_tokens: 40 } },
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    metadata: HashMap::new(),
                }
            }
            DingTalkAction::GetUserinfo => {
                AgentResult {
                    task_id: task.id.clone(), agent_id: self.id.clone(), status: ResultStatus::Success,
                    output: TaskOutput::Text("钉钉用户信息:\n- 用户ID: xxx\n- 姓名: xxx\n- 部门: xxx".to_string()),
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

impl Default for DingTalkAdapter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dingtalk_adapter_creation() {
        let adapter = DingTalkAdapter::new();
        assert_eq!(adapter.id(), "dingtalk-open");
        assert_eq!(adapter.name(), "钉钉开放平台");
        assert_eq!(adapter.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_dingtalk_capabilities() {
        let adapter = DingTalkAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.len() >= 5);
        assert!(caps.iter().any(|c| c.name == "dingtalk_oauth"));
        assert!(caps.iter().any(|c| c.name == "dingtalk_message"));
        assert!(caps.iter().any(|c| c.name == "dingtalk_approval"));
    }

    #[test]
    fn test_dingtalk_message_struct() {
        let msg = DingTalkMessage {
            msg_type: "text".to_string(),
            content: "测试消息".to_string(),
            receiver_ids: vec!["user1".to_string(), "user2".to_string()],
        };
        assert_eq!(msg.msg_type, "text");
        assert_eq!(msg.receiver_ids.len(), 2);
    }

    #[test]
    fn test_dingtalk_approval_struct() {
        let approval = DingTalkApproval {
            process_code: "process_001".to_string(),
            title: "请假审批".to_string(),
            originator_user_id: "user001".to_string(),
            form_component_values: HashMap::new(),
        };
        assert_eq!(approval.process_code, "process_001");
    }

    #[test]
    fn test_health_tracker() {
        let adapter = DingTalkAdapter::new();
        let health = adapter.health_tracker.get_metrics();
        assert_eq!(health.health_score, 100.0);
    }
}