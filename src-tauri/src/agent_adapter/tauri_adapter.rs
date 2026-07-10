// Tauri Desktop Adapter - Desktop app development support
//
// Phase 2: Desktop Development Agents
// Supports Tauri app build, dev, and cross-platform compilation

use crate::agent_adapter::{
    health_tracker::HealthTracker,
    types::{
        ActualCost, AdapterType, AgentConfig, AgentError, AgentResult, AgentTask, Capability,
        HealthMetrics, ResultStatus, TaskInput, TaskOutput, TokenUsage,
    },
    AgentAdapter,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Tauri Desktop Adapter
pub struct TauriDesktopAdapter {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    build_target: DesktopPlatform,
}

/// Desktop platform target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopPlatform {
    Windows,
    MacOS,
    Linux,
    CrossPlatform,
}

impl DesktopPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            DesktopPlatform::Windows => "windows",
            DesktopPlatform::MacOS => "macos",
            DesktopPlatform::Linux => "linux",
            DesktopPlatform::CrossPlatform => "cross",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "windows" | "win" => Ok(DesktopPlatform::Windows),
            "macos" | "mac" | "darwin" => Ok(DesktopPlatform::MacOS),
            "linux" => Ok(DesktopPlatform::Linux),
            "cross" | "cross-platform" => Ok(DesktopPlatform::CrossPlatform),
            other => Err(format!("Unknown desktop platform: {}", other)),
        }
    }

    /// Get current platform from OS
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            DesktopPlatform::Windows
        } else if cfg!(target_os = "macos") {
            DesktopPlatform::MacOS
        } else if cfg!(target_os = "linux") {
            DesktopPlatform::Linux
        } else {
            DesktopPlatform::CrossPlatform
        }
    }

    /// Get build targets for cross-compilation
    pub fn build_targets(&self) -> Vec<String> {
        match self {
            DesktopPlatform::Windows => vec!["x86_64-pc-windows-msvc".to_string()],
            DesktopPlatform::MacOS => vec![
                "x86_64-apple-darwin".to_string(),
                "aarch64-apple-darwin".to_string(),
            ],
            DesktopPlatform::Linux => vec!["x86_64-unknown-linux-gnu".to_string()],
            DesktopPlatform::CrossPlatform => vec![
                "x86_64-pc-windows-msvc".to_string(),
                "x86_64-apple-darwin".to_string(),
                "aarch64-apple-darwin".to_string(),
                "x86_64-unknown-linux-gnu".to_string(),
            ],
        }
    }
}

/// Desktop build configuration
#[derive(Debug, Clone)]
pub struct DesktopBuildConfig {
    pub target: DesktopPlatform,
    pub dev_mode: bool,
    pub hot_reload: bool,
    pub release_mode: bool,
    pub bundle_type: BundleType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleType {
    None,     // Just build, no bundle
    Msi,      // Windows MSI
    Dmg,      // macOS DMG
    AppImage, // Linux AppImage
    Deb,      // Linux DEB
    Rpm,      // Linux RPM
    Nsis,     // Windows NSIS
}

impl BundleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BundleType::None => "none",
            BundleType::Msi => "msi",
            BundleType::Dmg => "dmg",
            BundleType::AppImage => "appimage",
            BundleType::Deb => "deb",
            BundleType::Rpm => "rpm",
            BundleType::Nsis => "nsis",
        }
    }

    pub fn for_platform(platform: DesktopPlatform) -> Vec<BundleType> {
        match platform {
            DesktopPlatform::Windows => vec![BundleType::Msi, BundleType::Nsis],
            DesktopPlatform::MacOS => vec![BundleType::Dmg],
            DesktopPlatform::Linux => vec![BundleType::AppImage, BundleType::Deb, BundleType::Rpm],
            DesktopPlatform::CrossPlatform => {
                vec![BundleType::Msi, BundleType::Dmg, BundleType::AppImage]
            }
        }
    }
}

impl TauriDesktopAdapter {
    pub fn new() -> Self {
        Self {
            id: "tauri-desktop".to_string(),
            name: "Tauri Desktop Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            build_target: DesktopPlatform::current(),
        }
    }

    pub fn with_target(target: DesktopPlatform) -> Self {
        Self {
            id: "tauri-desktop".to_string(),
            name: "Tauri Desktop Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            build_target: target,
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "tauri-build".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 60000, // Builds take time
            },
            Capability {
                name: "tauri-dev".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.0,
                latency_ms: 30000, // Dev mode faster
            },
            Capability {
                name: "tauri-release".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 120000, // Release builds longer
            },
            Capability {
                name: "cross-compile".to_string(),
                proficiency: 0.70,
                cost_per_unit: 0.0,
                latency_ms: 180000, // Cross-compile longest
            },
            Capability {
                name: "ui-component".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.002,
                latency_ms: 5000,
            },
        ]
    }

    /// Build Tauri app in dev mode
    pub async fn build_dev(&self, cwd: &PathBuf) -> Result<String, AgentError> {
        let output = tokio::process::Command::new("cargo")
            .args(["tauri", "dev"])
            .current_dir(cwd)
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            Ok(o) => Err(AgentError::ExecutionError {
                message: format!("Tauri dev failed: {}", String::from_utf8_lossy(&o.stderr)),
                retryable: true,
            }),
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to start tauri dev: {}", e),
                retryable: true,
            }),
        }
    }

    /// Build Tauri app in release mode
    pub async fn build_release(
        &self,
        cwd: &PathBuf,
        bundle: Option<BundleType>,
    ) -> Result<String, AgentError> {
        let mut args = vec!["tauri", "build"];

        if let Some(b) = bundle {
            args.push("--bundles");
            args.push(b.as_str());
        }

        let output = tokio::process::Command::new("cargo")
            .args(&args)
            .current_dir(cwd)
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            Ok(o) => Err(AgentError::ExecutionError {
                message: format!("Tauri build failed: {}", String::from_utf8_lossy(&o.stderr)),
                retryable: true,
            }),
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to start tauri build: {}", e),
                retryable: true,
            }),
        }
    }

    /// Detect Tauri project
    pub fn detect_tauri_project(cwd: &PathBuf) -> bool {
        cwd.join("src-tauri/tauri.conf.json").exists() || cwd.join("tauri.conf.json").exists()
    }

    /// Get build output directory
    pub fn get_output_dir(cwd: &PathBuf, release: bool) -> PathBuf {
        if release {
            cwd.join("src-tauri/target/release/bundle")
        } else {
            cwd.join("src-tauri/target/debug")
        }
    }
}

impl Default for TauriDesktopAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for TauriDesktopAdapter {
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
        Self::get_capabilities()
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        self.config = Some(config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        // Check if cargo-tauri is installed
        let output = tokio::process::Command::new("cargo")
            .args(["tauri", "--version"])
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(true),
            _ => Err(AgentError::ConfigurationError {
                message: "cargo-tauri not installed. Run: cargo install tauri-cli".to_string(),
            }),
        }
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Tauri Desktop not configured".to_string(),
        })?;

        let cwd = PathBuf::from(config.cwd.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string()
        }));

        let start = Instant::now();

        // Parse task description for build type
        let desc_lower = task.description.to_lowercase();
        let is_dev = desc_lower.contains("dev") || desc_lower.contains("develop");
        let is_release = desc_lower.contains("release") || desc_lower.contains("build");

        let output = if is_dev {
            self.build_dev(&cwd).await?
        } else if is_release {
            let bundle = if desc_lower.contains("msi") {
                Some(BundleType::Msi)
            } else if desc_lower.contains("dmg") {
                Some(BundleType::Dmg)
            } else if desc_lower.contains("appimage") {
                Some(BundleType::AppImage)
            } else {
                None
            };
            self.build_release(&cwd, bundle).await?
        } else {
            // Default to dev
            self.build_dev(&cwd).await?
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(output),
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cost: ActualCost {
                amount: 0.0, // Local builds are free
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
                (
                    "platform".to_string(),
                    self.build_target.as_str().to_string(),
                ),
                ("cwd".to_string(), cwd.to_string_lossy().to_string()),
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

    fn cost_estimate(&self, _task: &AgentTask) -> crate::agent_adapter::types::CostEstimate {
        // Local builds are free
        crate::agent_adapter::types::CostEstimate {
            min_cost: 0.0,
            max_cost: 0.0,
            currency: "USD".to_string(),
            breakdown: HashMap::new(),
            token_estimate: crate::agent_adapter::types::TokenEstimate {
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
        true
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tauri_adapter_new() {
        let adapter = TauriDesktopAdapter::new();
        assert_eq!(adapter.id(), "tauri-desktop");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_desktop_platform_current() {
        let platform = DesktopPlatform::current();
        if cfg!(target_os = "windows") {
            assert_eq!(platform, DesktopPlatform::Windows);
        }
    }

    #[test]
    fn test_desktop_platform_from_str() {
        assert_eq!(
            DesktopPlatform::from_str("windows").unwrap(),
            DesktopPlatform::Windows
        );
        assert_eq!(
            DesktopPlatform::from_str("macos").unwrap(),
            DesktopPlatform::MacOS
        );
        assert!(DesktopPlatform::from_str("unknown").is_err());
    }

    #[test]
    fn test_build_targets() {
        let targets = DesktopPlatform::CrossPlatform.build_targets();
        assert!(targets.contains(&"x86_64-pc-windows-msvc".to_string()));
        assert!(targets.contains(&"x86_64-apple-darwin".to_string()));
    }

    #[test]
    fn test_bundle_type_for_platform() {
        let bundles = BundleType::for_platform(DesktopPlatform::Windows);
        assert!(bundles.contains(&BundleType::Msi));
        assert!(bundles.contains(&BundleType::Nsis));
    }

    #[test]
    fn test_capabilities() {
        let adapter = TauriDesktopAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "tauri-build"));
        assert!(caps.iter().any(|c| c.name == "tauri-dev"));
    }

    #[test]
    fn test_cost_estimate_free() {
        let adapter = TauriDesktopAdapter::new();
        let task = AgentTask::default();
        let estimate = adapter.cost_estimate(&task);
        assert_eq!(estimate.min_cost, 0.0);
        assert_eq!(estimate.max_cost, 0.0);
    }
}
