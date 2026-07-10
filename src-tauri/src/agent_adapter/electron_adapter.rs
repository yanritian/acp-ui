// Electron Desktop Adapter - Electron app development support
//
// Phase 2: Desktop Development Agents
// Supports Electron app build, dev, and cross-platform compilation

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

/// Electron Desktop Adapter
pub struct ElectronDesktopAdapter {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    build_target: ElectronPlatform,
}

/// Electron platform target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectronPlatform {
    Windows,
    MacOS,
    Linux,
    CrossPlatform,
}

impl ElectronPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            ElectronPlatform::Windows => "win",
            ElectronPlatform::MacOS => "mac",
            ElectronPlatform::Linux => "linux",
            ElectronPlatform::CrossPlatform => "all",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "win" | "windows" => Ok(ElectronPlatform::Windows),
            "mac" | "macos" | "darwin" => Ok(ElectronPlatform::MacOS),
            "linux" => Ok(ElectronPlatform::Linux),
            "all" | "cross" => Ok(ElectronPlatform::CrossPlatform),
            other => Err(format!("Unknown electron platform: {}", other)),
        }
    }

    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            ElectronPlatform::Windows
        } else if cfg!(target_os = "macos") {
            ElectronPlatform::MacOS
        } else if cfg!(target_os = "linux") {
            ElectronPlatform::Linux
        } else {
            ElectronPlatform::CrossPlatform
        }
    }
}

/// Electron build configuration
#[derive(Debug, Clone)]
pub struct ElectronBuildConfig {
    pub target: ElectronPlatform,
    pub dev_mode: bool,
    pub release_mode: bool,
    pub arch: ElectronArch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectronArch {
    X64,
    Arm64,
    Universal, // Mac only
}

impl ElectronArch {
    pub fn as_str(&self) -> &'static str {
        match self {
            ElectronArch::X64 => "x64",
            ElectronArch::Arm64 => "arm64",
            ElectronArch::Universal => "universal",
        }
    }
}

impl ElectronDesktopAdapter {
    pub fn new() -> Self {
        Self {
            id: "electron-desktop".to_string(),
            name: "Electron Desktop Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            build_target: ElectronPlatform::current(),
        }
    }

    pub fn with_target(target: ElectronPlatform) -> Self {
        Self {
            id: "electron-desktop".to_string(),
            name: "Electron Desktop Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            build_target: target,
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "electron-build".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 90000,
            },
            Capability {
                name: "electron-dev".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
            Capability {
                name: "electron-pack".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.0,
                latency_ms: 120000,
            },
            Capability {
                name: "electron-make".to_string(),
                proficiency: 0.75,
                cost_per_unit: 0.0,
                latency_ms: 150000,
            },
            Capability {
                name: "npm-scripts".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 10000,
            },
        ]
    }

    /// Start Electron in dev mode
    pub async fn start_dev(&self, cwd: &PathBuf) -> Result<String, AgentError> {
        let output = tokio::process::Command::new("npm")
            .args(["run", "dev"])
            .current_dir(cwd)
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            Ok(_) => {
                // Try electron-forge start
                let output2 = tokio::process::Command::new("npm")
                    .args(["run", "start"])
                    .current_dir(cwd)
                    .output()
                    .await;

                match output2 {
                    Ok(o2) if o2.status.success() => {
                        Ok(String::from_utf8_lossy(&o2.stdout).to_string())
                    }
                    Ok(o2) => Err(AgentError::ExecutionError {
                        message: format!(
                            "Electron dev failed: {}",
                            String::from_utf8_lossy(&o2.stderr)
                        ),
                        retryable: true,
                    }),
                    Err(e) => Err(AgentError::ExecutionError {
                        message: format!("Failed to start electron: {}", e),
                        retryable: true,
                    }),
                }
            }
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to run npm dev: {}", e),
                retryable: true,
            }),
        }
    }

    /// Build Electron app for distribution
    pub async fn build(
        &self,
        cwd: &PathBuf,
        platform: ElectronPlatform,
        arch: ElectronArch,
    ) -> Result<String, AgentError> {
        let platform_arg = platform.as_str();
        let arch_arg = arch.as_str();

        // Try electron-builder first
        let output = tokio::process::Command::new("npm")
            .args([
                "run",
                "build",
                "--",
                "--platform",
                platform_arg,
                "--arch",
                arch_arg,
            ])
            .current_dir(cwd)
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            Ok(_) => {
                // Try electron-forge make
                let output2 = tokio::process::Command::new("npm")
                    .args([
                        "run",
                        "make",
                        "--",
                        "--platform",
                        platform_arg,
                        "--arch",
                        arch_arg,
                    ])
                    .current_dir(cwd)
                    .output()
                    .await;

                match output2 {
                    Ok(o2) if o2.status.success() => {
                        Ok(String::from_utf8_lossy(&o2.stdout).to_string())
                    }
                    Ok(o2) => Err(AgentError::ExecutionError {
                        message: format!(
                            "Electron build failed: {}",
                            String::from_utf8_lossy(&o2.stderr)
                        ),
                        retryable: true,
                    }),
                    Err(e) => Err(AgentError::ExecutionError {
                        message: format!("Failed to run electron make: {}", e),
                        retryable: true,
                    }),
                }
            }
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to run npm build: {}", e),
                retryable: true,
            }),
        }
    }

    /// Detect Electron project
    pub fn detect_electron_project(cwd: &PathBuf) -> bool {
        // Check package.json for electron dependency
        if let Ok(content) = std::fs::read_to_string(cwd.join("package.json")) {
            content.contains("\"electron\"")
                || content.contains("\"electron-builder\"")
                || content.contains("\"electron-forge\"")
        } else {
            false
        }
    }

    /// Detect Electron builder config
    pub fn detect_builder_config(cwd: &PathBuf) -> Option<String> {
        // Check electron-builder.yml
        if cwd.join("electron-builder.yml").exists() {
            return Some("electron-builder.yml".to_string());
        }
        // Check electron-builder.json
        if cwd.join("electron-builder.json").exists() {
            return Some("electron-builder.json".to_string());
        }
        // Check forge config
        if cwd.join("forge.config.js").exists() {
            return Some("forge.config.js".to_string());
        }
        None
    }
}

impl Default for ElectronDesktopAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for ElectronDesktopAdapter {
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
        // Check if npm is available
        let output = tokio::process::Command::new("npm")
            .args(["--version"])
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(true),
            _ => Err(AgentError::ConfigurationError {
                message: "npm not installed. Please install Node.js first".to_string(),
            }),
        }
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Electron Desktop not configured".to_string(),
        })?;

        let cwd = PathBuf::from(config.cwd.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string()
        }));

        let start = Instant::now();

        // Parse task for build type
        let desc_lower = task.description.to_lowercase();
        let is_dev = desc_lower.contains("dev") || desc_lower.contains("start");
        let is_build = desc_lower.contains("build")
            || desc_lower.contains("pack")
            || desc_lower.contains("make");

        // Parse platform and arch
        let platform = if desc_lower.contains("win") || desc_lower.contains("windows") {
            ElectronPlatform::Windows
        } else if desc_lower.contains("mac") || desc_lower.contains("macos") {
            ElectronPlatform::MacOS
        } else if desc_lower.contains("linux") {
            ElectronPlatform::Linux
        } else {
            self.build_target
        };

        let arch = if desc_lower.contains("arm64") || desc_lower.contains("aarch64") {
            ElectronArch::Arm64
        } else if desc_lower.contains("x64") || desc_lower.contains("x86_64") {
            ElectronArch::X64
        } else if desc_lower.contains("universal") && platform == ElectronPlatform::MacOS {
            ElectronArch::Universal
        } else {
            ElectronArch::X64
        };

        let output = if is_dev {
            self.start_dev(&cwd).await?
        } else if is_build {
            self.build(&cwd, platform, arch).await?
        } else {
            // Default to dev
            self.start_dev(&cwd).await?
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
                amount: 0.0,
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
                ("platform".to_string(), platform.as_str().to_string()),
                ("arch".to_string(), arch.as_str().to_string()),
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
    fn test_electron_adapter_new() {
        let adapter = ElectronDesktopAdapter::new();
        assert_eq!(adapter.id(), "electron-desktop");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_electron_platform_current() {
        let platform = ElectronPlatform::current();
        if cfg!(target_os = "windows") {
            assert_eq!(platform, ElectronPlatform::Windows);
        }
    }

    #[test]
    fn test_electron_platform_from_str() {
        assert_eq!(
            ElectronPlatform::from_str("win").unwrap(),
            ElectronPlatform::Windows
        );
        assert_eq!(
            ElectronPlatform::from_str("mac").unwrap(),
            ElectronPlatform::MacOS
        );
        assert!(ElectronPlatform::from_str("unknown").is_err());
    }

    #[test]
    fn test_electron_arch_as_str() {
        assert_eq!(ElectronArch::X64.as_str(), "x64");
        assert_eq!(ElectronArch::Arm64.as_str(), "arm64");
        assert_eq!(ElectronArch::Universal.as_str(), "universal");
    }

    #[test]
    fn test_capabilities() {
        let adapter = ElectronDesktopAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "electron-build"));
        assert!(caps.iter().any(|c| c.name == "electron-dev"));
    }

    #[test]
    fn test_cost_estimate_free() {
        let adapter = ElectronDesktopAdapter::new();
        let task = AgentTask::default();
        let estimate = adapter.cost_estimate(&task);
        assert_eq!(estimate.min_cost, 0.0);
    }
}
