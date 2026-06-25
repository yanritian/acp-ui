// Jimeng Adapter - Jimeng AI image generation
//
// Phase 4: Marketing & Finance Agents
// Supports image generation, style transfer

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

/// Jimeng API Adapter (即梦 AI 图像生成)
pub struct JimengAdapter {
    id: String,
    name: String,
    config: Option<JimengConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    client: Client,
}

/// Jimeng API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JimengConfig {
    /// API key (Volcengine/Jimeng)
    pub api_key: String,
    /// Base URL
    pub base_url: String,
    /// Default style
    pub default_style: JimengStyle,
    /// Default size
    pub default_size: ImageSize,
}

impl Default for JimengConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.volcengine.com/api/jimeng".to_string(),
            default_style: JimengStyle::Realistic,
            default_size: ImageSize::Square1024,
        }
    }
}

/// Image generation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JimengStyle {
    Realistic,
    Anime,
    Abstract,
    Cartoon,
    OilPainting,
    Watercolor,
    Sketch,
    Cyberpunk,
    Fantasy,
    Minimalist,
}

impl JimengStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            JimengStyle::Realistic => "realistic",
            JimengStyle::Anime => "anime",
            JimengStyle::Abstract => "abstract",
            JimengStyle::Cartoon => "cartoon",
            JimengStyle::OilPainting => "oil_painting",
            JimengStyle::Watercolor => "watercolor",
            JimengStyle::Sketch => "sketch",
            JimengStyle::Cyberpunk => "cyberpunk",
            JimengStyle::Fantasy => "fantasy",
            JimengStyle::Minimalist => "minimalist",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "realistic" | "写实" => JimengStyle::Realistic,
            "anime" | "动漫" => JimengStyle::Anime,
            "abstract" | "抽象" => JimengStyle::Abstract,
            "cartoon" | "卡通" => JimengStyle::Cartoon,
            "oil_painting" | "油画" => JimengStyle::OilPainting,
            "watercolor" | "水彩" => JimengStyle::Watercolor,
            "sketch" | "素描" => JimengStyle::Sketch,
            "cyberpunk" | "赛博朋克" => JimengStyle::Cyberpunk,
            "fantasy" | "奇幻" => JimengStyle::Fantasy,
            "minimalist" | "极简" => JimengStyle::Minimalist,
            _ => JimengStyle::Realistic,
        }
    }
}

/// Image size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageSize {
    Square512,
    Square1024,
    Portrait768x1024,
    Landscape1024x768,
    Wide1920x1080,
}

impl ImageSize {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            ImageSize::Square512 => (512, 512),
            ImageSize::Square1024 => (1024, 1024),
            ImageSize::Portrait768x1024 => (768, 1024),
            ImageSize::Landscape1024x768 => (1024, 768),
            ImageSize::Wide1920x1080 => (1920, 1080),
        }
    }
}

/// Jimeng API request
#[derive(Debug, Serialize)]
struct JimengRequest {
    prompt: String,
    negative_prompt: Option<String>,
    style: String,
    width: u32,
    height: u32,
    num_images: u32,
}

/// Jimeng API response
#[derive(Debug, Deserialize)]
struct JimengResponse {
    task_id: String,
    status: JimengTaskStatus,
    images: Vec<JimengImage>,
    cost: f32,
}

#[derive(Debug, Deserialize)]
pub enum JimengTaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Deserialize)]
struct JimengImage {
    url: String,
    seed: u32,
}

/// Jimeng async job result
#[derive(Debug, Deserialize)]
struct JimengJobResult {
    task_id: String,
    status: JimengTaskStatus,
    images: Vec<String>,
    error: Option<String>,
}

impl JimengAdapter {
    pub fn new() -> Self {
        Self {
            id: "jimeng-marketing".to_string(),
            name: "Jimeng Image Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
        }
    }

    pub fn with_config(config: JimengConfig) -> Self {
        Self {
            id: "jimeng-marketing".to_string(),
            name: "Jimeng Image Adapter".to_string(),
            config: Some(config),
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "image-gen".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.05, // $0.05 per image
                latency_ms: 15000, // Image generation takes ~15 seconds
            },
            Capability {
                name: "style-transfer".to_string(),
                proficiency: 0.75,
                cost_per_unit: 0.08,
                latency_ms: 20000,
            },
            Capability {
                name: "variation".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.03,
                latency_ms: 10000,
            },
        ]
    }

    /// Generate image via Jimeng API
    pub async fn generate_image(
        &self,
        prompt: &str,
        negative_prompt: Option<&str>,
        style: JimengStyle,
        size: ImageSize,
    ) -> Result<(Vec<String>, f32), AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Jimeng API key not configured".to_string(),
        })?;

        let (width, height) = size.dimensions();
        let request = JimengRequest {
            prompt: prompt.to_string(),
            negative_prompt: negative_prompt.map(|s| s.to_string()),
            style: style.as_str().to_string(),
            width,
            height,
            num_images: 1,
        };

        let url = format!("{}/generate", config.base_url);
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
                let jimeng_response: JimengResponse = resp.json().await.map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to parse Jimeng response: {}", e),
                    retryable: false,
                })?;

                let urls = jimeng_response.images.iter()
                    .map(|i| i.url.clone())
                    .collect();

                Ok((urls, jimeng_response.cost))
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                Err(AgentError::ExecutionError {
                    message: format!("Jimeng API error {}: {}", status, body),
                    retryable: status.is_server_error(),
                })
            }
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Jimeng API request failed: {}", e),
                retryable: e.is_timeout() || e.is_connect(),
            }),
        }
    }

    /// Get job status (for async generation)
    pub async fn get_job_status(&self, task_id: &str) -> Result<JimengJobResult, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Jimeng API key not configured".to_string(),
        })?;

        let url = format!("{}/task/{}/status", config.base_url, task_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<JimengJobResult>().await.map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to parse job status: {}", e),
                    retryable: false,
                })
            }
            Ok(resp) => Err(AgentError::ExecutionError {
                message: format!("Jimeng API error: {}", resp.status()),
                retryable: resp.status().is_server_error(),
            }),
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to get job status: {}", e),
                retryable: e.is_timeout(),
            }),
        }
    }

    /// Parse style from task description
    fn parse_style_from_description(description: &str) -> JimengStyle {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("动漫") || desc_lower.contains("anime") {
            JimengStyle::Anime
        } else if desc_lower.contains("油画") || desc_lower.contains("oil") {
            JimengStyle::OilPainting
        } else if desc_lower.contains("水彩") || desc_lower.contains("watercolor") {
            JimengStyle::Watercolor
        } else if desc_lower.contains("赛博") || desc_lower.contains("cyberpunk") {
            JimengStyle::Cyberpunk
        } else if desc_lower.contains("奇幻") || desc_lower.contains("fantasy") {
            JimengStyle::Fantasy
        } else if desc_lower.contains("卡通") || desc_lower.contains("cartoon") {
            JimengStyle::Cartoon
        } else if desc_lower.contains("抽象") || desc_lower.contains("abstract") {
            JimengStyle::Abstract
        } else if desc_lower.contains("素描") || desc_lower.contains("sketch") {
            JimengStyle::Sketch
        } else if desc_lower.contains("极简") || desc_lower.contains("minimalist") {
            JimengStyle::Minimalist
        } else {
            JimengStyle::Realistic
        }
    }
}

impl Default for JimengAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for JimengAdapter {
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
        let jimeng_config = if let Some(api_key) = config.metadata.get("api_key") {
            JimengConfig {
                api_key: api_key.clone(),
                base_url: config.metadata.get("base_url")
                    .cloned()
                    .unwrap_or_else(|| "https://api.volcengine.com/api/jimeng".to_string()),
                default_style: config.metadata.get("style")
                    .map(|s| JimengStyle::from_str(s))
                    .unwrap_or(JimengStyle::Realistic),
                default_size: config.metadata.get("size")
                    .and_then(|s| match s.as_str() {
                        "512" => Some(ImageSize::Square512),
                        "1024" => Some(ImageSize::Square1024),
                        "portrait" => Some(ImageSize::Portrait768x1024),
                        "landscape" => Some(ImageSize::Landscape1024x768),
                        "wide" => Some(ImageSize::Wide1920x1080),
                        _ => None,
                    })
                    .unwrap_or(ImageSize::Square1024),
            }
        } else {
            return Err(AgentError::ConfigurationError {
                message: "Jimeng API key required in metadata".to_string(),
            });
        };

        self.config = Some(jimeng_config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Jimeng not configured".to_string(),
        })?;

        if config.api_key.is_empty() {
            return Err(AgentError::ConfigurationError {
                message: "Jimeng API key is empty".to_string(),
            });
        }

        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // Parse prompt from task input
        let prompt = match &task.input {
            TaskInput::Text(text) => text.clone(),
            TaskInput::Data(data) => data.clone(),
            _ => "Generate a beautiful image".to_string(),
        };

        // Parse style from description or use default
        let style = Self::parse_style_from_description(&task.description);
        let config = self.config.as_ref().unwrap();
        let effective_style = if task.description.to_lowercase().contains("风格") {
            style
        } else {
            config.default_style
        };

        let (urls, cost) = self.generate_image(&prompt, None, effective_style, config.default_size).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Output first image URL
        let output = urls.first().cloned().unwrap_or_default();

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Url(output),
            input_tokens: 0, // Image generation doesn't use tokens
            output_tokens: 0,
            total_tokens: 0,
            cost: ActualCost {
                amount: cost,
                currency: "USD".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                },
            },
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metadata: HashMap::from([
                ("style".to_string(), effective_style.as_str().to_string()),
                ("provider".to_string(), "jimeng".to_string()),
                ("images_count".to_string(), urls.len().to_string()),
            ]),
        })
    }

    async fn cancel(&self, _task_id: &str) -> Result<(), AgentError> {
        Ok(())
    }

    fn status(&self) -> crate::agent_adapter::types::AgentStatus {
        crate::agent_adapter::types::AgentStatus::Idle
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, _task: &AgentTask) -> CostEstimate {
        // Image generation has fixed cost per image
        CostEstimate {
            min_cost: 0.03, // Variation
            max_cost: 0.10, // Style transfer
            currency: "USD".to_string(),
            breakdown: HashMap::from([
                ("base".to_string(), 0.05),
            ]),
            token_estimate: TokenEstimate {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            },
        }
    }

    fn actual_cost(&self, _task_id: &str) -> Option<ActualCost> {
        None
    }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history.iter().rev().take(last_n as usize).cloned().collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        match result.status {
            ResultStatus::Success => {
                self.health_tracker.record_success(result.duration_ms);
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
    fn test_jimeng_adapter_new() {
        let adapter = JimengAdapter::new();
        assert_eq!(adapter.id(), "jimeng-marketing");
        assert_eq!(adapter.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_jimeng_config_default() {
        let config = JimengConfig::default();
        assert_eq!(config.default_style, JimengStyle::Realistic);
        assert_eq!(config.default_size, ImageSize::Square1024);
    }

    #[test]
    fn test_jimeng_style_as_str() {
        assert_eq!(JimengStyle::Anime.as_str(), "anime");
        assert_eq!(JimengStyle::Cyberpunk.as_str(), "cyberpunk");
        assert_eq!(JimengStyle::OilPainting.as_str(), "oil_painting");
    }

    #[test]
    fn test_jimeng_style_from_str() {
        assert_eq!(JimengStyle::from_str("anime"), JimengStyle::Anime);
        assert_eq!(JimengStyle::from_str("写实"), JimengStyle::Realistic);
        assert_eq!(JimengStyle::from_str("unknown"), JimengStyle::Realistic);
    }

    #[test]
    fn test_image_size_dimensions() {
        assert_eq!(ImageSize::Square512.dimensions(), (512, 512));
        assert_eq!(ImageSize::Square1024.dimensions(), (1024, 1024));
        assert_eq!(ImageSize::Wide1920x1080.dimensions(), (1920, 1080));
    }

    #[test]
    fn test_capabilities() {
        let adapter = JimengAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "image-gen"));
        assert!(caps.iter().any(|c| c.name == "style-transfer"));
    }

    #[test]
    fn test_cost_estimate() {
        let adapter = JimengAdapter::new();
        let task = AgentTask::default();
        let estimate = adapter.cost_estimate(&task);
        assert!(estimate.min_cost > 0.0);
        assert!(estimate.max_cost > estimate.min_cost);
    }

    #[test]
    fn test_parse_style_from_description() {
        assert_eq!(JimengAdapter::parse_style_from_description("动漫风格"), JimengStyle::Anime);
        assert_eq!(JimengAdapter::parse_style_from_description("oil painting"), JimengStyle::OilPainting);
        assert_eq!(JimengAdapter::parse_style_from_description("cyberpunk theme"), JimengStyle::Cyberpunk);
        assert_eq!(JimengAdapter::parse_style_from_description("normal image"), JimengStyle::Realistic);
    }
}