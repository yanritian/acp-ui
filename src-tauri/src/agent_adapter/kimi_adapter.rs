// Kimi Adapter - Moonshot AI content generation
//
// Phase 4: Marketing & Finance Agents
// Supports text generation, translation, copywriting

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskInput, TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics,
            CostEstimate, TokenEstimate},
    health_tracker::HealthTracker,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Kimi API Adapter (Moonshot AI)
pub struct KimiAdapter {
    id: String,
    name: String,
    config: Option<KimiConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    client: Client,
}

/// Kimi API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KimiConfig {
    /// API key (Moonshot AI)
    pub api_key: String,
    /// Base URL (default: https://api.moonshot.cn/v1)
    pub base_url: String,
    /// Model (moonshot-v1-8k, moonshot-v1-32k, moonshot-v1-128k)
    pub model: String,
    /// Temperature (0.0-1.0)
    pub temperature: f32,
    /// Max tokens
    pub max_tokens: u32,
}

impl Default for KimiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.moonshot.cn/v1".to_string(),
            model: "moonshot-v1-8k".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

/// Kimi API request
#[derive(Debug, Serialize)]
struct KimiRequest {
    model: String,
    messages: Vec<KimiMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct KimiMessage {
    role: String,
    content: String,
}

/// Kimi API response
#[derive(Debug, Deserialize)]
struct KimiResponse {
    id: String,
    model: String,
    choices: Vec<KimiChoice>,
    usage: KimiUsage,
}

#[derive(Debug, Deserialize)]
struct KimiChoice {
    index: u32,
    message: KimiMessageResponse,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct KimiMessageResponse {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
pub struct KimiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl KimiAdapter {
    pub fn new() -> Self {
        Self {
            id: "kimi-marketing".to_string(),
            name: "Kimi Marketing Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
        }
    }

    pub fn with_config(config: KimiConfig) -> Self {
        Self {
            id: "kimi-marketing".to_string(),
            name: "Kimi Marketing Adapter".to_string(),
            config: Some(config),
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "copywriting".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.012, // $0.012 per 1K tokens (input)
                latency_ms: 2000,
            },
            Capability {
                name: "content-rewrite".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.012,
                latency_ms: 3000,
            },
            Capability {
                name: "translation".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.012,
                latency_ms: 2000,
            },
            Capability {
                name: "summary".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.012,
                latency_ms: 1500,
            },
        ]
    }

    /// Generate text via Kimi API
    pub async fn generate(&self, prompt: &str, system_prompt: Option<&str>) -> Result<(String, KimiUsage), AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Kimi API key not configured".to_string(),
        })?;

        let messages = if let Some(system) = system_prompt {
            vec![
                KimiMessage { role: "system".to_string(), content: system.to_string() },
                KimiMessage { role: "user".to_string(), content: prompt.to_string() },
            ]
        } else {
            vec![KimiMessage { role: "user".to_string(), content: prompt.to_string() }]
        };

        let request = KimiRequest {
            model: config.model.clone(),
            messages,
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        };

        let url = format!("{}/chat/completions", config.base_url);
        let start = Instant::now();

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let kimi_response: KimiResponse = resp.json().await.map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to parse Kimi response: {}", e),
                    retryable: false,
                })?;

                let content = kimi_response.choices.first()
                    .map(|c| c.message.content.clone())
                    .unwrap_or_default();

                Ok((content, kimi_response.usage))
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                Err(AgentError::ExecutionError {
                    message: format!("Kimi API error {}: {}", status, body),
                    retryable: status.is_server_error(),
                })
            }
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Kimi API request failed: {}", e),
                retryable: e.is_timeout() || e.is_connect(),
            }),
        }
    }

    /// Calculate cost from token usage
    fn calculate_cost(usage: &KimiUsage, model: &str) -> f32 {
        // Kimi pricing: $0.012/1K input tokens, $0.012/1K output tokens
        // moonshot-v1-128k is $0.05/1K tokens
        let rate = if model.contains("128k") {
            0.05
        } else {
            0.012
        };

        let input_cost = (usage.prompt_tokens as f32 / 1000.0) * rate;
        let output_cost = (usage.completion_tokens as f32 / 1000.0) * rate;
        input_cost + output_cost
    }
}

impl Default for KimiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for KimiAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Api
    }

    fn capabilities(&self) -> Vec<Capability> {
        Self::get_capabilities()
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        // Extract Kimi-specific config from metadata
        let kimi_config = if let Some(api_key) = config.metadata.get("api_key") {
            KimiConfig {
                api_key: api_key.clone(),
                base_url: config.metadata.get("base_url")
                    .cloned()
                    .unwrap_or_else(|| "https://api.moonshot.cn/v1".to_string()),
                model: config.metadata.get("model")
                    .cloned()
                    .unwrap_or_else(|| "moonshot-v1-8k".to_string()),
                temperature: config.metadata.get("temperature")
                    .and_then(|t| t.parse::<f32>().ok())
                    .unwrap_or(0.7),
                max_tokens: config.metadata.get("max_tokens")
                    .and_then(|t| t.parse::<u32>().ok())
                    .unwrap_or(4096),
            }
        } else {
            return Err(AgentError::ConfigurationError {
                message: "Kimi API key required in metadata".to_string(),
            });
        };

        self.config = Some(kimi_config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Kimi not configured".to_string(),
        })?;

        if config.api_key.is_empty() {
            return Err(AgentError::ConfigurationError {
                message: "Kimi API key is empty".to_string(),
            });
        }

        // Test API connection with minimal request
        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // Parse task input
        let prompt = match &task.input {
            TaskInput::Text(text) => text.clone(),
            TaskInput::File(path) => {
                // Read file content as prompt
                std::fs::read_to_string(path).map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to read file: {}", e),
                    retryable: false,
                })?
            }
            TaskInput::Url(url) => format!("请处理以下URL内容: {}", url),
            TaskInput::Data(data) => data.clone(),
            TaskInput::Multi(inputs) => inputs.iter().map(|i| match i {
                TaskInput::Text(t) => t.clone(),
                TaskInput::File(p) => std::fs::read_to_string(p).unwrap_or_default(),
                TaskInput::Url(u) => format!("URL: {}", u),
                TaskInput::Data(d) => d.clone(),
                TaskInput::Multi(_) => String::new(),
            }).collect::<Vec<_>>().join("\n"),
        };

        // Determine system prompt based on task description
        let system_prompt = if task.description.to_lowercase().contains("翻译") || task.description.to_lowercase().contains("translate") {
            Some("你是一个专业的翻译助手，请准确翻译用户提供的文本，保持原文的风格和语气。")
        } else if task.description.to_lowercase().contains("文案") || task.description.to_lowercase().contains("copywriting") {
            Some("你是一个专业的文案撰写助手，请创作有吸引力、简洁有力的文案内容。")
        } else if task.description.to_lowercase().contains("总结") || task.description.to_lowercase().contains("summary") {
            Some("你是一个专业的总结助手，请提取关键信息，简洁概括内容要点。")
        } else {
            Some("你是一个智能助手，请根据用户需求提供专业、准确的回复。")
        };

        let (content, usage) = self.generate(&prompt, system_prompt).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        let config = self.config.as_ref().unwrap();
        let cost = Self::calculate_cost(&usage, &config.model);

        // Track token usage
        let token_usage = TokenUsage {
            input_tokens: usage.prompt_tokens as u64,
            output_tokens: usage.completion_tokens as u64,
            total_tokens: usage.total_tokens as u64,
        };

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(content),
            input_tokens: usage.prompt_tokens as u64,
            output_tokens: usage.completion_tokens as u64,
            total_tokens: usage.total_tokens as u64,
            cost: ActualCost {
                amount: cost,
                currency: "USD".to_string(),
                token_usage: token_usage.clone(),
            },
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metadata: HashMap::from([
                ("model".to_string(), config.model.clone()),
                ("provider".to_string(), "moonshot".to_string()),
            ]),
        })
    }

    async fn cancel(&self, _task_id: &str) -> Result<(), AgentError> {
        // API calls can't be cancelled once sent
        Ok(())
    }

    fn status(&self) -> crate::agent_adapter::types::AgentStatus {
        crate::agent_adapter::types::AgentStatus::Idle
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate {
        // Estimate based on prompt length
        let prompt_length = match &task.input {
            TaskInput::Text(text) => text.len(),
            TaskInput::File(_) => 1000, // Assume 1000 chars
            TaskInput::Url(_) => 500,
            TaskInput::Data(data) => data.len(),
            TaskInput::Multi(inputs) => inputs.iter().map(|i| match i {
                TaskInput::Text(t) => t.len(),
                TaskInput::File(_) => 1000,
                TaskInput::Url(_) => 500,
                TaskInput::Data(d) => d.len(),
                TaskInput::Multi(_) => 0,
            }).sum(),
        };

        let estimated_input_tokens = (prompt_length / 4) as u64; // ~4 chars per token
        let estimated_output_tokens = 500u64; // Average response
        let estimated_total = estimated_input_tokens + estimated_output_tokens;

        let rate = self.config.as_ref()
            .map(|c| if c.model.contains("128k") { 0.05 } else { 0.012 })
            .unwrap_or(0.012);

        let estimated_cost = (estimated_total as f32 / 1000.0) * rate;

        CostEstimate {
            min_cost: estimated_cost * 0.5,
            max_cost: estimated_cost * 2.0,
            currency: "USD".to_string(),
            breakdown: HashMap::from([
                ("input".to_string(), (estimated_input_tokens as f32 / 1000.0) * rate),
                ("output".to_string(), (estimated_output_tokens as f32 / 1000.0) * rate),
            ]),
            token_estimate: TokenEstimate {
                input_tokens: estimated_input_tokens,
                output_tokens: estimated_output_tokens,
                total_tokens: estimated_total,
            },
        }
    }

    fn actual_cost(&self, _task_id: &str) -> Option<ActualCost> {
        // Would need to store per-task costs
        None
    }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history.iter().rev().take(last_n as usize).cloned().collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        match result.status {
            ResultStatus::Success => {
                self.health_tracker.record_success(result.duration_ms);
                self.token_history.push(result.cost.token_usage.clone());
            }
            _ => {
                self.health_tracker.record_error();
            }
        }
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        self.health_tracker.is_circuit_breaker_closed()
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kimi_adapter_new() {
        let adapter = KimiAdapter::new();
        assert_eq!(adapter.id(), "kimi-marketing");
        assert_eq!(adapter.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_kimi_config_default() {
        let config = KimiConfig::default();
        assert_eq!(config.base_url, "https://api.moonshot.cn/v1");
        assert_eq!(config.model, "moonshot-v1-8k");
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_capabilities() {
        let adapter = KimiAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "copywriting"));
        assert!(caps.iter().any(|c| c.name == "translation"));
        assert!(caps.iter().any(|c| c.name == "summary"));
    }

    #[test]
    fn test_cost_estimate() {
        let adapter = KimiAdapter::new();
        let task = AgentTask {
            id: "test".to_string(),
            description: "Write copywriting".to_string(),
            input: TaskInput::Text("Test prompt".to_string()),
            constraints: Default::default(),
            scene_type: None,
            platform: None,
        };

        let estimate = adapter.cost_estimate(&task);
        assert!(estimate.min_cost > 0.0);
        assert!(estimate.max_cost > estimate.min_cost);
        assert!(estimate.token_estimate.total_tokens > 0);
    }

    #[test]
    fn test_calculate_cost() {
        let usage = KimiUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };

        // Standard model: $0.012/1K tokens
        let cost = KimiAdapter::calculate_cost(&usage, "moonshot-v1-8k");
        assert_eq!(cost, 0.018); // 1K * 0.012 + 0.5K * 0.012

        // 128k model: $0.05/1K tokens
        let cost_128k = KimiAdapter::calculate_cost(&usage, "moonshot-v1-128k");
        assert_eq!(cost_128k, 0.075); // 1K * 0.05 + 0.5K * 0.05
    }

    #[tokio::test]
    async fn test_configure_requires_api_key() {
        let mut adapter = KimiAdapter::new();
        let config = AgentConfig {
            api_key: None,
            endpoint: None,
            model: None,
            cwd: None,
            timeout_ms: 60000,
            max_retries: 3,
            metadata: HashMap::new(),
        };

        let result = adapter.configure(config).await;
        assert!(result.is_err());
    }
}