// Kling Adapter - Kling AI video generation
//
// Phase 4: Marketing & Finance Agents
// Supports video generation, AI avatar, async job polling

use crate::agent_adapter::{
    health_tracker::HealthTracker,
    types::{
        ActualCost, AdapterType, AgentConfig, AgentError, AgentResult, AgentTask, Capability,
        CostEstimate, HealthMetrics, ResultStatus, TaskInput, TaskOutput, TokenEstimate,
        TokenUsage,
    },
    AgentAdapter,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Kling API Adapter (可灵 AI 视频生成)
pub struct KlingAdapter {
    id: String,
    name: String,
    config: Option<KlingConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    client: Client,
    pending_jobs: HashMap<String, KlingJob>,
}

/// Kling API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlingConfig {
    /// API key (快手/Kling)
    pub api_key: String,
    /// Base URL
    pub base_url: String,
    /// Default video duration (seconds)
    pub default_duration: u32,
    /// Default resolution
    pub default_resolution: VideoResolution,
}

impl Default for KlingConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://api.klingai.com/v1".to_string(),
            default_duration: 5,
            default_resolution: VideoResolution::HD1080,
        }
    }
}

/// Video resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoResolution {
    SD480,
    HD720,
    HD1080,
    UHD4K,
}

impl VideoResolution {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            VideoResolution::SD480 => (640, 480),
            VideoResolution::HD720 => (1280, 720),
            VideoResolution::HD1080 => (1920, 1080),
            VideoResolution::UHD4K => (3840, 2160),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            VideoResolution::SD480 => "480p",
            VideoResolution::HD720 => "720p",
            VideoResolution::HD1080 => "1080p",
            VideoResolution::UHD4K => "4k",
        }
    }
}

/// Video generation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KlingMode {
    /// Text-to-video
    TextToVideo,
    /// Image-to-video
    ImageToVideo,
    /// AI Avatar (数字人)
    Avatar,
    /// Video extension (延长视频)
    Extend,
}

/// Kling API request
#[derive(Debug, Serialize)]
struct KlingRequest {
    prompt: String,
    negative_prompt: Option<String>,
    duration: u32,
    resolution: String,
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<String>,
}

/// Kling API response (async job)
#[derive(Debug, Deserialize)]
pub struct KlingResponse {
    pub task_id: String,
    pub status: KlingTaskStatus,
    pub estimated_duration: u32,
    pub cost: f32,
}

/// Kling task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KlingTaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Kling job tracking
#[derive(Debug, Clone)]
pub struct KlingJob {
    pub task_id: String,
    pub original_task_id: String,
    pub status: KlingTaskStatus,
    pub created_at: u64,
    pub video_url: Option<String>,
    pub cost: f32,
}

/// Kling job result
#[derive(Debug, Deserialize)]
pub struct KlingJobResult {
    pub task_id: String,
    pub status: KlingTaskStatus,
    pub video_url: Option<String>,
    pub error: Option<String>,
    pub cost: f32,
}

impl KlingAdapter {
    pub fn new() -> Self {
        Self {
            id: "kling-marketing".to_string(),
            name: "Kling Video Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
            pending_jobs: HashMap::new(),
        }
    }

    pub fn with_config(config: KlingConfig) -> Self {
        Self {
            id: "kling-marketing".to_string(),
            name: "Kling Video Adapter".to_string(),
            config: Some(config),
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            client: Client::new(),
            pending_jobs: HashMap::new(),
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "video-gen".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.50, // $0.50 per 5s video
                latency_ms: 180000,  // Video generation takes ~3 minutes
            },
            Capability {
                name: "avatar-gen".to_string(),
                proficiency: 0.85,
                cost_per_unit: 1.00, // AI avatar is more expensive
                latency_ms: 300000,  // Takes ~5 minutes
            },
            Capability {
                name: "video-extend".to_string(),
                proficiency: 0.75,
                cost_per_unit: 0.30,
                latency_ms: 120000,
            },
            Capability {
                name: "image-to-video".to_string(),
                proficiency: 0.70,
                cost_per_unit: 0.40,
                latency_ms: 150000,
            },
        ]
    }

    /// Create video generation job
    pub async fn create_video_job(
        &self,
        prompt: &str,
        mode: KlingMode,
        image_url: Option<&str>,
    ) -> Result<KlingResponse, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Kling API key not configured".to_string(),
        })?;

        let mode_str = match mode {
            KlingMode::TextToVideo => "text_to_video",
            KlingMode::ImageToVideo => "image_to_video",
            KlingMode::Avatar => "avatar",
            KlingMode::Extend => "extend",
        };

        let request = KlingRequest {
            prompt: prompt.to_string(),
            negative_prompt: None,
            duration: config.default_duration,
            resolution: config.default_resolution.as_str().to_string(),
            mode: mode_str.to_string(),
            image_url: image_url.map(|s| s.to_string()),
        };

        let url = format!("{}/videos/generations", config.base_url);
        let start = Instant::now();

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<KlingResponse>()
                    .await
                    .map_err(|e| AgentError::ExecutionError {
                        message: format!("Failed to parse Kling response: {}", e),
                        retryable: false,
                    })
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                Err(AgentError::ExecutionError {
                    message: format!("Kling API error {}: {}", status, body),
                    retryable: status.is_server_error(),
                })
            }
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Kling API request failed: {}", e),
                retryable: e.is_timeout() || e.is_connect(),
            }),
        }
    }

    /// Get job status
    pub async fn get_job_status(&self, task_id: &str) -> Result<KlingJobResult, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Kling API key not configured".to_string(),
        })?;

        let url = format!("{}/videos/generations/{}/status", config.base_url, task_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<KlingJobResult>()
                    .await
                    .map_err(|e| AgentError::ExecutionError {
                        message: format!("Failed to parse job status: {}", e),
                        retryable: false,
                    })
            }
            Ok(resp) => Err(AgentError::ExecutionError {
                message: format!("Kling API error: {}", resp.status()),
                retryable: resp.status().is_server_error(),
            }),
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to get job status: {}", e),
                retryable: e.is_timeout(),
            }),
        }
    }

    /// Poll job until completion (with timeout)
    pub async fn poll_job_until_complete(
        &self,
        task_id: &str,
        timeout_seconds: u32,
    ) -> Result<(String, f32), AgentError> {
        let start = Instant::now();
        let timeout_ms = timeout_seconds * 1000;

        loop {
            let elapsed = start.elapsed().as_millis();
            if elapsed > timeout_ms as u128 {
                return Err(AgentError::ExecutionError {
                    message: "Video generation timeout".to_string(),
                    retryable: true,
                });
            }

            let result = self.get_job_status(task_id).await?;

            match result.status {
                KlingTaskStatus::Completed => {
                    if let Some(url) = result.video_url {
                        return Ok((url, result.cost));
                    } else {
                        return Err(AgentError::ExecutionError {
                            message: "Video completed but no URL returned".to_string(),
                            retryable: false,
                        });
                    }
                }
                KlingTaskStatus::Failed => {
                    return Err(AgentError::ExecutionError {
                        message: result
                            .error
                            .unwrap_or_else(|| "Video generation failed".to_string()),
                        retryable: false,
                    });
                }
                KlingTaskStatus::Cancelled => {
                    return Err(AgentError::ExecutionError {
                        message: "Video generation cancelled".to_string(),
                        retryable: false,
                    });
                }
                KlingTaskStatus::Pending | KlingTaskStatus::Processing => {
                    // Wait 5 seconds before next poll
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Parse mode from task description
    fn parse_mode_from_description(description: &str) -> KlingMode {
        let desc_lower = description.to_lowercase();

        if desc_lower.contains("数字人")
            || desc_lower.contains("avatar")
            || desc_lower.contains("虚拟主播")
        {
            KlingMode::Avatar
        } else if desc_lower.contains("延长") || desc_lower.contains("extend") {
            KlingMode::Extend
        } else if desc_lower.contains("图生") || desc_lower.contains("image") {
            KlingMode::ImageToVideo
        } else {
            KlingMode::TextToVideo
        }
    }
}

impl Default for KlingAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for KlingAdapter {
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
        let kling_config = if let Some(api_key) = config.metadata.get("api_key") {
            KlingConfig {
                api_key: api_key.clone(),
                base_url: config
                    .metadata
                    .get("base_url")
                    .cloned()
                    .unwrap_or_else(|| "https://api.klingai.com/v1".to_string()),
                default_duration: config
                    .metadata
                    .get("duration")
                    .and_then(|d| d.parse::<u32>().ok())
                    .unwrap_or(5),
                default_resolution: config
                    .metadata
                    .get("resolution")
                    .map(|r| match r.as_str() {
                        "480" | "480p" => VideoResolution::SD480,
                        "720" | "720p" => VideoResolution::HD720,
                        "1080" | "1080p" => VideoResolution::HD1080,
                        "4k" => VideoResolution::UHD4K,
                        _ => VideoResolution::HD1080,
                    })
                    .unwrap_or(VideoResolution::HD1080),
            }
        } else {
            return Err(AgentError::ConfigurationError {
                message: "Kling API key required in metadata".to_string(),
            });
        };

        self.config = Some(kling_config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Kling not configured".to_string(),
        })?;

        if config.api_key.is_empty() {
            return Err(AgentError::ConfigurationError {
                message: "Kling API key is empty".to_string(),
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
            TaskInput::Url(url) => format!("Generate video from: {}", url),
            _ => "Generate a creative video".to_string(),
        };

        // Parse mode from description
        let mode = Self::parse_mode_from_description(&task.description);

        // Check for image URL (for image-to-video mode)
        let image_url = if mode == KlingMode::ImageToVideo {
            match &task.input {
                TaskInput::Url(url) => Some(url.as_str()),
                _ => None,
            }
        } else {
            None
        };

        // Create async job
        let job_response = self.create_video_job(&prompt, mode, image_url).await?;
        let kling_task_id = job_response.task_id.clone();

        // Poll until complete (default 5 minute timeout)
        let (video_url, cost) = self.poll_job_until_complete(&kling_task_id, 300).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Url(video_url),
            input_tokens: 0,
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
                ("mode".to_string(), mode.to_string()),
                ("kling_task_id".to_string(), kling_task_id),
                ("provider".to_string(), "kling".to_string()),
                (
                    "duration_seconds".to_string(),
                    self.config.as_ref().unwrap().default_duration.to_string(),
                ),
            ]),
        })
    }

    async fn cancel(&self, task_id: &str) -> Result<(), AgentError> {
        // Would need to call Kling cancel API
        Ok(())
    }

    fn status(&self) -> crate::agent_adapter::types::AgentStatus {
        crate::agent_adapter::types::AgentStatus::Idle
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate {
        // Video generation has fixed cost per video
        let base_cost = 0.50; // 5s video

        let mode = Self::parse_mode_from_description(&task.description);
        let mode_cost = match mode {
            KlingMode::Avatar => 1.00,
            KlingMode::Extend => 0.30,
            KlingMode::ImageToVideo => 0.40,
            KlingMode::TextToVideo => 0.50,
        };

        CostEstimate {
            min_cost: mode_cost * 0.8,
            max_cost: mode_cost * 1.5,
            currency: "USD".to_string(),
            breakdown: HashMap::from([("video_generation".to_string(), mode_cost)]),
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
        self.token_history
            .iter()
            .rev()
            .take(last_n as usize)
            .cloned()
            .collect()
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

impl std::fmt::Display for KlingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KlingMode::TextToVideo => write!(f, "text_to_video"),
            KlingMode::ImageToVideo => write!(f, "image_to_video"),
            KlingMode::Avatar => write!(f, "avatar"),
            KlingMode::Extend => write!(f, "extend"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kling_adapter_new() {
        let adapter = KlingAdapter::new();
        assert_eq!(adapter.id(), "kling-marketing");
        assert_eq!(adapter.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_kling_config_default() {
        let config = KlingConfig::default();
        assert_eq!(config.default_duration, 5);
        assert_eq!(config.default_resolution, VideoResolution::HD1080);
    }

    #[test]
    fn test_video_resolution_dimensions() {
        assert_eq!(VideoResolution::HD720.dimensions(), (1280, 720));
        assert_eq!(VideoResolution::HD1080.dimensions(), (1920, 1080));
        assert_eq!(VideoResolution::UHD4K.dimensions(), (3840, 2160));
    }

    #[test]
    fn test_video_resolution_as_str() {
        assert_eq!(VideoResolution::SD480.as_str(), "480p");
        assert_eq!(VideoResolution::HD1080.as_str(), "1080p");
    }

    #[test]
    fn test_capabilities() {
        let adapter = KlingAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "video-gen"));
        assert!(caps.iter().any(|c| c.name == "avatar-gen"));
    }

    #[test]
    fn test_cost_estimate() {
        let adapter = KlingAdapter::new();
        let task = AgentTask {
            id: "test".to_string(),
            description: "Generate avatar video".to_string(),
            input: TaskInput::Text("test".to_string()),
            constraints: Default::default(),
            scene_type: None,
            platform: None,
        };

        let estimate = adapter.cost_estimate(&task);
        assert!(estimate.min_cost > 0.0);
        assert!(estimate.max_cost > estimate.min_cost);
    }

    #[test]
    fn test_parse_mode_from_description() {
        assert_eq!(
            KlingAdapter::parse_mode_from_description("数字人视频"),
            KlingMode::Avatar
        );
        assert_eq!(
            KlingAdapter::parse_mode_from_description("avatar generation"),
            KlingMode::Avatar
        );
        assert_eq!(
            KlingAdapter::parse_mode_from_description("延长视频"),
            KlingMode::Extend
        );
        assert_eq!(
            KlingAdapter::parse_mode_from_description("图生视频"),
            KlingMode::ImageToVideo
        );
        assert_eq!(
            KlingAdapter::parse_mode_from_description("normal video"),
            KlingMode::TextToVideo
        );
    }

    #[test]
    fn test_kling_mode_display() {
        assert_eq!(KlingMode::TextToVideo.to_string(), "text_to_video");
        assert_eq!(KlingMode::Avatar.to_string(), "avatar");
    }
}
