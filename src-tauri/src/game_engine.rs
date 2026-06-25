// Unified Game Engine Interface
// Phase 1 Day 5: Abstract interface for all game engines

use crate::game_detector::{GameDetector, GameEngine, GameInfo};
use std::path::PathBuf;

/// Unified game engine interface
pub trait GameEngineInterface {
    /// Build the game project
    fn build(&self, cwd: &PathBuf, output: &PathBuf) -> Result<BuildResult, BuildError>;

    /// Get available build targets
    fn get_targets(&self) -> Vec<String>;

    /// Get engine name
    fn name(&self) -> &str;
}

/// Build result
#[derive(Debug, Clone)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: PathBuf,
    pub build_time_ms: u64,
    pub error: Option<String>,
}

/// Build error
#[derive(Debug)]
pub enum BuildError {
    EngineNotFound(String),
    BuildFailed(String),
    InvalidProject(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::EngineNotFound(e) => write!(f, "Engine not found: {}", e),
            BuildError::BuildFailed(e) => write!(f, "Build failed: {}", e),
            BuildError::InvalidProject(e) => write!(f, "Invalid project: {}", e),
        }
    }
}

/// Game engine manager
pub struct GameEngineManager;

impl GameEngineManager {
    /// Build game using appropriate engine
    pub async fn build_game(
        cwd: &PathBuf,
        target: &str,
        output: &PathBuf,
    ) -> Result<BuildResult, BuildError> {
        // Detect game engine
        let info = GameDetector::detect(cwd)
            .map_err(|e| BuildError::InvalidProject(e.to_string()))?;

        // Route to appropriate engine
        match info.engine {
            GameEngine::Godot => Self::build_godot(cwd, target, output).await,
            GameEngine::Unity => Self::build_unity(cwd, target, output).await,
            GameEngine::Unknown => Err(BuildError::InvalidProject("Unknown game engine".to_string())),
        }
    }

    /// Build Godot project
    async fn build_godot(
        cwd: &PathBuf,
        target: &str,
        output: &PathBuf,
    ) -> Result<BuildResult, BuildError> {
        use crate::agent_adapter::godot_adapter::{GodotAdapter, GodotPlatform};

        let adapter = GodotAdapter::new();

        // Parse target platform
        let platform = match target.to_lowercase().as_str() {
            "windows" | "win64" => GodotPlatform::Windows,
            "macos" | "osx" => GodotPlatform::MacOS,
            "linux" | "linux64" => GodotPlatform::Linux,
            "android" => GodotPlatform::Android,
            "ios" => GodotPlatform::iOS,
            "web" | "webgl" => GodotPlatform::Web,
            _ => GodotPlatform::Windows,
        };

        let start = std::time::Instant::now();

        match adapter.export(cwd, platform, false).await {
            Ok(output_str) => {
                Ok(BuildResult {
                    success: true,
                    output_path: PathBuf::from(output_str),
                    build_time_ms: start.elapsed().as_millis() as u64,
                    error: None,
                })
            }
            Err(e) => {
                Ok(BuildResult {
                    success: false,
                    output_path: output.clone(),
                    build_time_ms: start.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Build Unity project
    async fn build_unity(
        cwd: &PathBuf,
        target: &str,
        output: &PathBuf,
    ) -> Result<BuildResult, BuildError> {
        use std::process::Command;

        // Find Unity executable
        let unity_path = Self::find_unity_executable()?;

        let start = std::time::Instant::now();

        let result = Command::new(&unity_path)
            .arg("-quit")
            .arg("-batchmode")
            .arg("-projectPath")
            .arg(cwd)
            .arg("-buildTarget")
            .arg(target)
            .arg("-buildApp")
            .arg(output)
            .output();

        let build_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(cmd_output) => {
                if cmd_output.status.success() {
                    Ok(BuildResult {
                        success: true,
                        output_path: output.clone(),
                        build_time_ms: build_time,
                        error: None,
                    })
                } else {
                    let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                    Ok(BuildResult {
                        success: false,
                        output_path: output.clone(),
                        build_time_ms: build_time,
                        error: Some(format!("Build failed: {}", stderr)),
                    })
                }
            }
            Err(e) => {
                Ok(BuildResult {
                    success: false,
                    output_path: output.clone(),
                    build_time_ms: build_time,
                    error: Some(format!("Failed to run Unity: {}", e)),
                })
            }
        }
    }

    /// Find Unity executable
    fn find_unity_executable() -> Result<String, BuildError> {
        // 1. Check environment variable
        if let Ok(path) = std::env::var("UNITY_PATH") {
            if PathBuf::from(&path).exists() {
                return Ok(path);
            }
        }

        // 2. Check common locations
        let common_paths = vec![
            "C:\\Program Files\\Unity\\Hub\\Editor\\2021.3.0f1\\Editor\\Unity.exe",
            "C:\\Program Files\\Unity\\Hub\\Editor\\2022.3.0f1\\Editor\\Unity.exe",
            "/Applications/Unity/Hub/Editor/2021.3.0f1/Unity.app/Contents/MacOS/Unity",
            "/Applications/Unity/Hub/Editor/2022.3.0f1/Unity.app/Contents/MacOS/Unity",
            "/opt/Unity/Hub/Editor/2021.3.0f1/Editor/Unity",
        ];

        for path in common_paths {
            if PathBuf::from(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err(BuildError::EngineNotFound("Unity not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_build_result() {
        let result = BuildResult {
            success: true,
            output_path: PathBuf::from("/tmp/game.exe"),
            build_time_ms: 5000,
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.build_time_ms, 5000);
    }

    #[test]
    fn test_build_error_display() {
        let error = BuildError::EngineNotFound("Unity".to_string());
        assert_eq!(error.to_string(), "Engine not found: Unity");
    }
}
