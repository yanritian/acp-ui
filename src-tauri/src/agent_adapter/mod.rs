// Agent Adapter Module - Unified interface for all Agent backends
//
// This module provides a unified trait for wrapping different AI Agent services:
// - CLI Agents (Claude Code, Codex)
// - API Agents (Kimi, Kling, Jimeng, WPS)
// - Hybrid Agents (WebSocket + HTTP)
// - Desktop Agents (Tauri Desktop - Phase 2)
//
// Core features:
// - Unified AgentAdapter trait
// - Capability-based routing
// - Health monitoring (EWMA)
// - Circuit breaker
// - Token tracking
// - Cost estimation

pub mod types;
pub mod claude_adapter;
pub mod codex_adapter;
pub mod health_tracker;
pub mod tauri_adapter;  // Phase 2: Desktop Development
pub mod electron_adapter;  // Phase 2: Electron Desktop
pub mod unity_adapter;  // Phase 3: Game Development
pub mod godot_adapter;  // Phase 3: Game Development
pub mod renpy_adapter;  // Phase 3: Game Development - Visual Novel
pub mod kimi_adapter;   // Phase 4: Marketing & Finance
pub mod jimeng_adapter; // Phase 4: Marketing & Finance
pub mod kling_adapter;  // Phase 4: Marketing & Finance
pub mod wps_adapter;    // Phase 4: Office Automation
pub mod wechat_adapter; // Phase 8: Mini Program Development
pub mod flutter_adapter; // Phase 8: Mobile SDK Integration
pub mod docker_adapter;  // Phase 8: Containerized Deployment
pub mod kubernetes_adapter; // Phase 8: Distributed Orchestration
pub mod douyin_adapter;    // Phase 9: Enterprise Platform - Douyin
pub mod kuaishou_adapter;  // Phase 9: Enterprise Platform - Kuaishou
pub mod dingtalk_adapter;  // Phase 9: Enterprise Platform - DingTalk
pub mod feishu_adapter;    // Phase 9: Enterprise Platform - Feishu/Lark

pub use types::*;
pub use tauri_adapter::{TauriDesktopAdapter, DesktopPlatform, BundleType};
pub use electron_adapter::{ElectronDesktopAdapter, ElectronPlatform, ElectronArch};
pub use unity_adapter::{UnityAdapter, UnityPlatform, UnityBuildConfig};
pub use godot_adapter::{GodotAdapter, GodotPlatform, GodotExportConfig};
pub use renpy_adapter::{RenPyAdapter, RenPyPlatform, StorySpec, CharacterSpec, SceneSpec};
pub use kimi_adapter::{KimiAdapter, KimiConfig};
pub use jimeng_adapter::{JimengAdapter, JimengConfig, JimengStyle, ImageSize};
pub use kling_adapter::{KlingAdapter, KlingConfig, KlingMode, VideoResolution, KlingTaskStatus};
pub use wps_adapter::{WPSAdapter, WPSConfig, DocumentFormat, DocumentType};
pub use wechat_adapter::{WeChatMiniProgramAdapter, WeChatConfig, WeChatCompileMode, WeChatAction};
pub use flutter_adapter::{FlutterAdapter, FlutterConfig, FlutterPlatform, FlutterBuildMode, FlutterAction};
pub use docker_adapter::{DockerAdapter, DockerConfig, DockerAction, PortMapping, Protocol};
pub use kubernetes_adapter::{KubernetesAdapter, KubernetesConfig, KubernetesAction, ServiceType, ResourceLimits, IngressConfig};
pub use douyin_adapter::{DouyinAdapter, DouyinConfig, DouyinVideo, DouyinAnalytics, DouyinAction};
pub use kuaishou_adapter::{KuaishouAdapter, KuaishouConfig, KuaishouVideo, KuaishouAction};
pub use dingtalk_adapter::{DingTalkAdapter, DingTalkConfig, DingTalkMessage, DingTalkApproval, DingTalkAction};
pub use feishu_adapter::{FeishuAdapter, FeishuConfig, FeishuDocument, FeishuBitableRecord, FeishuAction};

use async_trait::async_trait;

/// Agent Adapter unified interface
///
/// All Agent backends must implement this trait to be integrated into the platform.
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Agent ID
    fn id(&self) -> &str;

    /// Agent name
    fn name(&self) -> &str;

    /// Agent type (CLI / API / Hybrid)
    fn adapter_type(&self) -> AdapterType;

    /// Agent capabilities
    fn capabilities(&self) -> Vec<Capability>;

    /// Configure Agent
    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError>;

    /// Validate configuration
    async fn validate_config(&self) -> Result<bool, AgentError>;

    /// Execute task
    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError>;

    /// Cancel task
    async fn cancel(&self, task_id: &str) -> Result<(), AgentError>;

    /// Get current status
    fn status(&self) -> AgentStatus;

    /// Get health metrics
    async fn health(&self) -> HealthMetrics;

    /// Cost estimation
    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate;

    /// Get actual cost (from history)
    fn actual_cost(&self, task_id: &str) -> Option<ActualCost>;

    /// Token usage summary (last N executions)
    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage>;

    /// Update health metrics (EWMA)
    async fn update_health(&mut self, result: &AgentResult);

    /// Check if circuit breaker allows request
    fn is_circuit_breaker_allowed(&self) -> bool;

    /// Reset circuit breaker
    async fn reset_circuit_breaker(&mut self);
}
