// Agent Adapter Types - Core type definitions for the Agent Adapter system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdapterType {
    Cli,    // Local CLI Agent (Claude Code, Codex)
    Api,    // HTTP API Agent (Kimi, Kling, WPS)
    Hybrid, // Mixed (WebSocket + HTTP)
}

/// Agent capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub proficiency: f32,   // 0.0 - 1.0, skill level
    pub cost_per_unit: f32, // USD or CNY, unit cost
    pub latency_ms: u64,    // Average response time
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub cwd: Option<String>, // Working directory (CLI Agent)
    pub metadata: HashMap<String, String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            endpoint: None,
            model: None,
            timeout_ms: 300000,
            max_retries: 3,
            cwd: None,
            metadata: HashMap::new(),
        }
    }
}

/// Agent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub description: String,
    pub input: TaskInput,
    pub constraints: TaskConstraints,
    pub scene_type: Option<SceneType>,
    pub platform: Option<Platform>,
}

impl Default for AgentTask {
    fn default() -> Self {
        Self {
            id: String::new(),
            description: String::new(),
            input: TaskInput::Text(String::new()),
            constraints: TaskConstraints::default(),
            scene_type: None,
            platform: None,
        }
    }
}

/// Scene type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SceneType {
    WebDevelopment,
    MiniProgramDevelopment,
    DesktopDevelopment,
    GameDevelopment,
    GameArtGeneration,
    GameCrossPlatform,
    GamePerformanceOptimization,
    Marketing, // Reserved for future
    Finance,   // Reserved for future
    Design,    // Reserved for future
}

/// Platform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Web,
    MiniProgram,
    Desktop,
    Mobile,
    Game,
}

/// Task input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskInput {
    Text(String),
    File(String),
    Url(String),
    Data(String),
    Multi(Vec<TaskInput>),
}

/// Task constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraints {
    pub timeout_ms: u64,
    pub max_tokens: Option<u64>,
    pub budget_limit: Option<f32>,
    pub quality_level: QualityLevel,
}

impl Default for TaskConstraints {
    fn default() -> Self {
        Self {
            timeout_ms: 300000, // 5 minutes
            max_tokens: None,
            budget_limit: None,
            quality_level: QualityLevel::Standard,
        }
    }
}

/// Quality level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityLevel {
    Low,
    Standard,
    High,
    Premium,
}

/// Agent result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: String,
    pub agent_id: String,
    pub status: ResultStatus,
    pub output: TaskOutput,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub cost: ActualCost,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Result status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResultStatus {
    Success,
    Failed { error: String, retryable: bool },
    Cancelled,
    Timeout,
    BudgetExceeded,
}

/// Task output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutput {
    Text(String),
    File(String),
    Url(String),
    Data(String),
    Multi(Vec<TaskOutput>),
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Busy {
        current_task: String,
        started_at: u64,
    },
    Error {
        message: String,
        error_count: u32,
    },
    Maintenance,
    CircuitBreakerOpen {
        reason: String,
    },
}

/// Agent health metrics (EWMA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub health_score: f32,   // EWMA 0-100
    pub success_rate: f32,   // Last 100 success rate
    pub avg_latency_ms: u64, // Average response time
    pub error_count: u32,
    pub last_success_at: Option<u64>,
    pub last_error_at: Option<u64>,
    pub circuit_breaker_state: CircuitBreakerState,
}

/// Circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitBreakerState {
    Closed,                          // Normal
    Open { opened_at: u64 },         // Circuit open (reject requests)
    HalfOpen { test_requests: u32 }, // Half open (testing)
}

/// Cost estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub min_cost: f32,
    pub max_cost: f32,
    pub currency: String,
    pub breakdown: HashMap<String, f32>,
    pub token_estimate: TokenEstimate,
}

/// Token estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimate {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

/// Actual cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualCost {
    pub amount: f32,
    pub currency: String,
    pub token_usage: TokenUsage,
}

/// Token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

/// Agent error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentError {
    ConfigurationError { message: String },
    ExecutionError { message: String, retryable: bool },
    TimeoutError { timeout_ms: u64 },
    CostLimitExceeded { limit: f32, actual: f32 },
    RateLimitError { retry_after_ms: u64 },
    AuthenticationError { message: String },
    NetworkError { message: String },
    CircuitBreakerOpen { reason: String },
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::ConfigurationError { message } => {
                write!(f, "Configuration error: {}", message)
            }
            AgentError::ExecutionError { message, .. } => write!(f, "Execution error: {}", message),
            AgentError::TimeoutError { timeout_ms } => write!(f, "Timeout after {}ms", timeout_ms),
            AgentError::CostLimitExceeded { limit, actual } => {
                write!(f, "Cost limit exceeded: limit={}, actual={}", limit, actual)
            }
            AgentError::RateLimitError { retry_after_ms } => {
                write!(f, "Rate limit, retry after {}ms", retry_after_ms)
            }
            AgentError::AuthenticationError { message } => {
                write!(f, "Authentication error: {}", message)
            }
            AgentError::NetworkError { message } => write!(f, "Network error: {}", message),
            AgentError::CircuitBreakerOpen { reason } => {
                write!(f, "Circuit breaker open: {}", reason)
            }
        }
    }
}

impl std::error::Error for AgentError {}

/// Execution record (for Self-Optimizing Router)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub task_id: String,
    pub scene_type: SceneType,
    pub platform: Platform,
    pub agent_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub cost: f32,
    pub timestamp: u64,
}
