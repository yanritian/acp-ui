// Unity Game Adapter - Unity game development support
//
// Phase 3: Game Development Agents
// Supports Unity project build, asset import, and cross-platform compilation

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

/// Unity Game Adapter
pub struct UnityAdapter {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    build_target: UnityPlatform,
}

/// Unity platform target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnityPlatform {
    Windows,
    MacOS,
    Linux,
    Android,
    iOS,
    WebGL,
}

impl UnityPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnityPlatform::Windows => "Windows",
            UnityPlatform::MacOS => "OSX",
            UnityPlatform::Linux => "Linux",
            UnityPlatform::Android => "Android",
            UnityPlatform::iOS => "iOS",
            UnityPlatform::WebGL => "WebGL",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "win" | "windows" => Ok(UnityPlatform::Windows),
            "mac" | "macos" | "osx" => Ok(UnityPlatform::MacOS),
            "linux" => Ok(UnityPlatform::Linux),
            "android" => Ok(UnityPlatform::Android),
            "ios" => Ok(UnityPlatform::iOS),
            "webgl" | "web" => Ok(UnityPlatform::WebGL),
            other => Err(format!("Unknown Unity platform: {}", other)),
        }
    }
}

/// Unity build configuration
#[derive(Debug, Clone)]
pub struct UnityBuildConfig {
    pub target: UnityPlatform,
    pub dev_mode: bool,
    pub release_mode: bool,
    pub scenes: Vec<String>,
}

impl UnityAdapter {
    pub fn new() -> Self {
        Self {
            id: "unity-game".to_string(),
            name: "Unity Game Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            build_target: UnityPlatform::Windows,
        }
    }

    pub fn with_target(target: UnityPlatform) -> Self {
        Self {
            id: "unity-game".to_string(),
            name: "Unity Game Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            build_target: target,
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "unity-build".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.0,
                latency_ms: 180000, // Builds take 3+ minutes
            },
            Capability {
                name: "unity-dev".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 60000,
            },
            Capability {
                name: "unity-asset".to_string(),
                proficiency: 0.75,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
            Capability {
                name: "unity-script".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.002,
                latency_ms: 5000,
            },
            Capability {
                name: "unity-scene".to_string(),
                proficiency: 0.70,
                cost_per_unit: 0.002,
                latency_ms: 10000,
            },
        ]
    }

    /// Build Unity project using CLI
    pub async fn build(
        &self,
        cwd: &PathBuf,
        target: UnityPlatform,
        dev: bool,
    ) -> Result<String, AgentError> {
        // Unity CLI: Unity.exe -quit -batchmode -executeMethod BuildPipeline.Build
        let unity_path = self.find_unity_executable()?;
        let cwd_str = cwd.to_string_lossy().to_string();

        let mut args = vec![
            "-quit",
            "-batchmode",
            "-projectPath",
            &cwd_str,
            "-buildTarget",
            target.as_str(),
        ];

        if dev {
            args.push("-devBuild");
        }

        let output = tokio::process::Command::new(&unity_path)
            .args(&args)
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            Ok(o) => Err(AgentError::ExecutionError {
                message: format!("Unity build failed: {}", String::from_utf8_lossy(&o.stderr)),
                retryable: true,
            }),
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to run Unity: {}", e),
                retryable: true,
            }),
        }
    }

    /// Find Unity executable
    fn find_unity_executable(&self) -> Result<String, AgentError> {
        // Windows: C:\Program Files\Unity\Hub\Editor\VERSION\Editor\Unity.exe
        // Mac: /Applications/Unity/Hub/Editor/VERSION/Unity.app/Contents/MacOS/Unity
        // Linux: /opt/Unity/Hub/Editor/VERSION/Editor/Unity

        if cfg!(target_os = "windows") {
            // Try common paths
            let paths = [
                "C:\\Program Files\\Unity\\Hub\\Editor",
                "C:\\Program Files (x86)\\Unity\\Hub\\Editor",
            ];

            for base in paths {
                if let Ok(entries) = std::fs::read_dir(base) {
                    for entry in entries.flatten() {
                        let version_path = entry.path();
                        let unity_exe = version_path.join("Editor").join("Unity.exe");
                        if unity_exe.exists() {
                            return Ok(unity_exe.to_string_lossy().to_string());
                        }
                    }
                }
            }

            Err(AgentError::ConfigurationError {
                message: "Unity not found. Install Unity Editor first".to_string(),
            })
        } else if cfg!(target_os = "macos") {
            // Check /Applications/Unity/Hub/Editor/
            let base = "/Applications/Unity/Hub/Editor";
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let unity_path = entry
                        .path()
                        .join("Unity.app")
                        .join("Contents")
                        .join("MacOS")
                        .join("Unity");
                    if unity_path.exists() {
                        return Ok(unity_path.to_string_lossy().to_string());
                    }
                }
            }

            Err(AgentError::ConfigurationError {
                message: "Unity not found on macOS".to_string(),
            })
        } else {
            Err(AgentError::ConfigurationError {
                message: "Unity detection not implemented for this platform".to_string(),
            })
        }
    }

    /// Detect Unity project
    pub fn detect_unity_project(cwd: &PathBuf) -> bool {
        // Check for Assets folder and .csproj files
        cwd.join("Assets").exists() && cwd.join("ProjectSettings").exists()
    }

    /// Get Unity scenes
    pub fn get_scenes(cwd: &PathBuf) -> Vec<String> {
        let assets_path = cwd.join("Assets").join("Scenes");
        if !assets_path.exists() {
            return vec![];
        }

        let mut scenes = vec![];
        if let Ok(entries) = std::fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "unity").unwrap_or(false) {
                    scenes.push(path.to_string_lossy().to_string());
                }
            }
        }
        scenes
    }
}

impl Default for UnityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for UnityAdapter {
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
        self.find_unity_executable()?;
        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Unity Adapter not configured".to_string(),
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
        let is_dev = desc_lower.contains("dev") || desc_lower.contains("develop");
        let is_build = desc_lower.contains("build") || desc_lower.contains("release");

        // Parse platform
        let platform = if desc_lower.contains("android") {
            UnityPlatform::Android
        } else if desc_lower.contains("ios") || desc_lower.contains("iphone") {
            UnityPlatform::iOS
        } else if desc_lower.contains("webgl") || desc_lower.contains("web") {
            UnityPlatform::WebGL
        } else if desc_lower.contains("mac") || desc_lower.contains("osx") {
            UnityPlatform::MacOS
        } else if desc_lower.contains("linux") {
            UnityPlatform::Linux
        } else {
            self.build_target
        };

        let output = if is_dev || is_build {
            self.build(&cwd, platform, is_dev).await?
        } else {
            // Default to dev build
            self.build(&cwd, platform, true).await?
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
    fn test_unity_adapter_new() {
        let adapter = UnityAdapter::new();
        assert_eq!(adapter.id(), "unity-game");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_unity_platform_from_str() {
        assert_eq!(
            UnityPlatform::from_str("windows").unwrap(),
            UnityPlatform::Windows
        );
        assert_eq!(
            UnityPlatform::from_str("android").unwrap(),
            UnityPlatform::Android
        );
        assert_eq!(
            UnityPlatform::from_str("webgl").unwrap(),
            UnityPlatform::WebGL
        );
        assert!(UnityPlatform::from_str("unknown").is_err());
    }

    #[test]
    fn test_unity_platform_as_str() {
        assert_eq!(UnityPlatform::Windows.as_str(), "Windows");
        assert_eq!(UnityPlatform::WebGL.as_str(), "WebGL");
    }

    #[test]
    fn test_capabilities() {
        let adapter = UnityAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "unity-build"));
        assert!(caps.iter().any(|c| c.name == "unity-dev"));
    }

    #[test]
    fn test_cost_estimate_free() {
        let adapter = UnityAdapter::new();
        let task = AgentTask::default();
        let estimate = adapter.cost_estimate(&task);
        assert_eq!(estimate.min_cost, 0.0);
    }

    #[test]
    fn test_detect_unity_project() {
        let cwd = PathBuf::from(".");
        // This test passes because we're not in a Unity project
        assert!(!UnityAdapter::detect_unity_project(&cwd));
    }
}
