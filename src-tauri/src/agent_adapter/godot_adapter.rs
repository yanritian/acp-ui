// Godot Game Adapter - Godot game development support
//
// Phase 3: Game Development Agents
// supports Godot project build, GDScript, and export templates

use crate::agent_adapter::{
    health_tracker::HealthTracker,
    types::{
        ActualCost, AdapterType, AgentConfig, AgentError, AgentResult, AgentTask, Capability,
        HealthMetrics, ResultStatus, TaskOutput, TokenUsage,
    },
    AgentAdapter,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Godot Game Adapter
pub struct GodotAdapter {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    export_target: GodotPlatform,
}

/// Godot platform target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GodotPlatform {
    Windows,
    MacOS,
    Linux,
    Android,
    iOS,
    Web,
}

impl GodotPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            GodotPlatform::Windows => "Windows Desktop",
            GodotPlatform::MacOS => "Mac OSX",
            GodotPlatform::Linux => "Linux/X11",
            GodotPlatform::Android => "Android",
            GodotPlatform::iOS => "iOS",
            GodotPlatform::Web => "Web",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "win" | "windows" => Ok(GodotPlatform::Windows),
            "mac" | "macos" => Ok(GodotPlatform::MacOS),
            "linux" => Ok(GodotPlatform::Linux),
            "android" => Ok(GodotPlatform::Android),
            "ios" => Ok(GodotPlatform::iOS),
            "web" | "webgl" => Ok(GodotPlatform::Web),
            other => Err(format!("Unknown Godot platform: {}", other)),
        }
    }
}

/// Godot export configuration
#[derive(Debug, Clone)]
pub struct GodotExportConfig {
    pub target: GodotPlatform,
    pub dev_mode: bool,
    pub export_path: String,
}

impl GodotAdapter {
    pub fn new() -> Self {
        Self {
            id: "godot-game".to_string(),
            name: "Godot Game Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            export_target: GodotPlatform::Windows,
        }
    }

    pub fn with_target(target: GodotPlatform) -> Self {
        Self {
            id: "godot-game".to_string(),
            name: "Godot Game Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            export_target: target,
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "godot-build".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 120000, // Godot builds faster than Unity
            },
            Capability {
                name: "godot-dev".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
            Capability {
                name: "godot-script".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.002,
                latency_ms: 5000,
            },
            Capability {
                name: "godot-scene".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.002,
                latency_ms: 8000,
            },
        ]
    }

    /// Export Godot project using headless mode
    pub async fn export(
        &self,
        cwd: &PathBuf,
        target: GodotPlatform,
        dev: bool,
    ) -> Result<String, AgentError> {
        // Godot CLI: godot --headless --export-release "Windows Desktop" output.exe
        let godot_path = self.find_godot_executable()?;

        let export_mode = if dev {
            "--export-debug"
        } else {
            "--export-release"
        };
        let output_name = self.get_output_name(target, dev);
        let output_path = cwd.join(&output_name);
        let output_str = output_path.to_string_lossy().to_string();

        let cwd_str = cwd.to_string_lossy().to_string();
        let args = vec![
            "--headless",
            "--path",
            &cwd_str,
            export_mode,
            target.as_str(),
            &output_str,
        ];

        let output = tokio::process::Command::new(&godot_path)
            .args(&args)
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => Ok(format!("Exported to: {}", output_str)),
            Ok(o) => Err(AgentError::ExecutionError {
                message: format!(
                    "Godot export failed: {}",
                    String::from_utf8_lossy(&o.stderr)
                ),
                retryable: true,
            }),
            Err(e) => Err(AgentError::ExecutionError {
                message: format!("Failed to run Godot: {}", e),
                retryable: true,
            }),
        }
    }

    /// Get output filename for platform
    fn get_output_name(&self, target: GodotPlatform, dev: bool) -> String {
        let suffix = if dev { "_dev" } else { "" };
        match target {
            GodotPlatform::Windows => format!("game{}.exe", suffix),
            GodotPlatform::MacOS => format!("game{}.app", suffix),
            GodotPlatform::Linux => format!("game{}.x86_64", suffix),
            GodotPlatform::Android => format!("game{}.apk", suffix),
            GodotPlatform::iOS => format!("game{}.ipa", suffix),
            GodotPlatform::Web => format!("game{}{}", suffix, "/index.html"), // Web is a folder
        }
    }

    /// Find Godot executable
    fn find_godot_executable(&self) -> Result<String, AgentError> {
        // Windows: C:\Program Files\Godot\Godot_v4.exe
        // Mac: /Applications/Godot.app/Contents/MacOS/Godot
        // Linux: /usr/bin/godot4 or ~/.local/bin/godot4

        if cfg!(target_os = "windows") {
            let paths = ["C:\\Program Files\\Godot", "C:\\Godot"];

            for base in paths {
                if let Ok(entries) = std::fs::read_dir(base) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map(|e| e == "exe").unwrap_or(false) {
                            if path
                                .file_name()
                                .map(|n| n.to_string_lossy().contains("Godot"))
                                .unwrap_or(false)
                            {
                                return Ok(path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }

            Err(AgentError::ConfigurationError {
                message: "Godot not found. Install Godot Editor first".to_string(),
            })
        } else if cfg!(target_os = "macos") {
            let app_path = "/Applications/Godot.app/Contents/MacOS/Godot";
            if std::path::Path::new(app_path).exists() {
                Ok(app_path.to_string())
            } else {
                Err(AgentError::ConfigurationError {
                    message: "Godot not found on macOS".to_string(),
                })
            }
        } else if cfg!(target_os = "linux") {
            // Check common paths
            let paths = [
                "/usr/bin/godot4",
                "/usr/local/bin/godot4",
                "~/.local/bin/godot4",
            ];

            for p in paths {
                let expanded = if p.starts_with("~") {
                    let home = std::env::var("HOME").unwrap_or("/home".to_string());
                    p.replace("~", &home)
                } else {
                    p.to_string()
                };
                if std::path::Path::new(&expanded).exists() {
                    return Ok(expanded);
                }
            }

            Err(AgentError::ConfigurationError {
                message: "Godot not found on Linux".to_string(),
            })
        } else {
            Err(AgentError::ConfigurationError {
                message: "Godot detection not implemented for this platform".to_string(),
            })
        }
    }

    /// Detect Godot project
    pub fn detect_godot_project(cwd: &PathBuf) -> bool {
        // Check for project.godot file
        cwd.join("project.godot").exists()
    }

    /// Get Godot scenes
    pub fn get_scenes(cwd: &PathBuf) -> Vec<String> {
        // Godot scenes are .tscn files
        let mut scenes = vec![];

        // Recursively search for .tscn files
        if let Ok(entries) = std::fs::read_dir(cwd) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Search subdirectories
                    Self::find_scenes_recursive(&path, &mut scenes);
                } else if path.extension().map(|e| e == "tscn").unwrap_or(false) {
                    scenes.push(path.to_string_lossy().to_string());
                }
            }
        }
        scenes
    }

    fn find_scenes_recursive(dir: &PathBuf, scenes: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::find_scenes_recursive(&path, scenes);
                } else if path.extension().map(|e| e == "tscn").unwrap_or(false) {
                    scenes.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
}

impl Default for GodotAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for GodotAdapter {
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
        self.find_godot_executable()?;
        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Godot Adapter not configured".to_string(),
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
        let is_dev = desc_lower.contains("dev") || desc_lower.contains("debug");
        let is_export = desc_lower.contains("export")
            || desc_lower.contains("build")
            || desc_lower.contains("release");

        // Parse platform
        let platform = if desc_lower.contains("android") {
            GodotPlatform::Android
        } else if desc_lower.contains("ios") || desc_lower.contains("iphone") {
            GodotPlatform::iOS
        } else if desc_lower.contains("web") || desc_lower.contains("webgl") {
            GodotPlatform::Web
        } else if desc_lower.contains("mac") || desc_lower.contains("osx") {
            GodotPlatform::MacOS
        } else if desc_lower.contains("linux") {
            GodotPlatform::Linux
        } else {
            self.export_target
        };

        let output = if is_dev || is_export {
            self.export(&cwd, platform, is_dev).await?
        } else {
            // Default to dev export
            self.export(&cwd, platform, true).await?
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
    fn test_godot_adapter_new() {
        let adapter = GodotAdapter::new();
        assert_eq!(adapter.id(), "godot-game");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_godot_platform_from_str() {
        assert_eq!(
            GodotPlatform::from_str("windows").unwrap(),
            GodotPlatform::Windows
        );
        assert_eq!(
            GodotPlatform::from_str("android").unwrap(),
            GodotPlatform::Android
        );
        assert_eq!(GodotPlatform::from_str("web").unwrap(), GodotPlatform::Web);
        assert!(GodotPlatform::from_str("unknown").is_err());
    }

    #[test]
    fn test_godot_platform_as_str() {
        assert_eq!(GodotPlatform::Windows.as_str(), "Windows Desktop");
        assert_eq!(GodotPlatform::Web.as_str(), "Web");
    }

    #[test]
    fn test_capabilities() {
        let adapter = GodotAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "godot-build"));
        assert!(caps.iter().any(|c| c.name == "godot-dev"));
    }

    #[test]
    fn test_cost_estimate_free() {
        let adapter = GodotAdapter::new();
        let task = AgentTask::default();
        let estimate = adapter.cost_estimate(&task);
        assert_eq!(estimate.min_cost, 0.0);
    }

    #[test]
    fn test_detect_godot_project() {
        let cwd = PathBuf::from(".");
        // This test passes because we're not in a Godot project
        assert!(!GodotAdapter::detect_godot_project(&cwd));
    }

    #[test]
    fn test_output_name() {
        let adapter = GodotAdapter::new();
        assert_eq!(
            adapter.get_output_name(GodotPlatform::Windows, false),
            "game.exe"
        );
        assert_eq!(
            adapter.get_output_name(GodotPlatform::Windows, true),
            "game_dev.exe"
        );
        assert_eq!(
            adapter.get_output_name(GodotPlatform::Android, false),
            "game.apk"
        );
    }
}
