// Game Export Commands
// Phase 2 Day 6: Complete export implementation

use crate::game_detector::{GameDetector, GameEngine, GameInfo, GameSize};
use crate::game_engine::{GameEngineManager, BuildResult, BuildError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ===== Request/Response Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExportRequest {
    pub cwd: String,
    pub target: String,      // Platform: "windows", "macos", "linux", "android", "ios", "web"
    pub output: Option<String>,  // Output path (optional, auto-generated if not provided)
    pub development: Option<bool>, // Development build (default: false)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExportResponse {
    pub success: bool,
    pub engine: String,
    pub output_path: Option<String>,
    pub build_time_ms: u64,
    pub file_size_mb: Option<f64>,
    pub error: Option<String>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameExportProgress {
    pub stage: String,       // "detecting", "preparing", "building", "finalizing", "complete"
    pub progress: f32,       // 0.0 to 1.0
    pub message: String,
}

// ===== Commands =====

/// Export game project
#[tauri::command]
pub async fn game_export(request: GameExportRequest) -> Result<GameExportResponse, String> {
    let cwd = PathBuf::from(&request.cwd);
    let mut logs = Vec::new();

    logs.push(format!("[INFO] Starting export for project: {}", cwd.display()));

    // Step 1: Detect game engine
    logs.push("[INFO] Step 1: Detecting game engine...".to_string());
    let info = match GameDetector::detect(&cwd) {
        Ok(info) => {
            logs.push(format!("[INFO] Detected engine: {:?}", info.engine));
            logs.push(format!("[INFO] Project: {}", info.project_name));
            logs.push(format!("[INFO] Version: {:?}", info.version));
            logs.push(format!("[INFO] Scenes: {} found", info.scenes.len()));
            logs.push(format!("[INFO] Size: {:?}", info.size));
            info
        }
        Err(e) => {
            logs.push(format!("[ERROR] Detection failed: {}", e));
            return Ok(GameExportResponse {
                success: false,
                engine: "unknown".to_string(),
                output_path: None,
                build_time_ms: 0,
                file_size_mb: None,
                error: Some(format!("Failed to detect game engine: {}", e)),
                logs,
            });
        }
    };

    let engine_name = match info.engine {
        GameEngine::Godot => "Godot",
        GameEngine::Unity => "Unity",
        GameEngine::Unknown => "Unknown",
    };

    // Step 2: Determine output path
    let output_path = if let Some(output) = &request.output {
        PathBuf::from(output)
    } else {
        let extension = match request.target.to_lowercase().as_str() {
            "windows" | "win64" => "exe",
            "macos" | "osx" => "app",
            "linux" | "linux64" => "x86_64",
            "android" => "apk",
            "ios" => "ipa",
            "web" | "webgl" => "html",
            _ => "exe",
        };
        cwd.join(format!("export/game.{}", extension))
    };

    logs.push(format!("[INFO] Step 2: Output path: {}", output_path.display()));

    // Step 3: Create output directory
    if let Some(parent) = output_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            logs.push(format!("[ERROR] Failed to create output directory: {}", e));
            return Ok(GameExportResponse {
                success: false,
                engine: engine_name.to_string(),
                output_path: None,
                build_time_ms: 0,
                file_size_mb: None,
                error: Some(format!("Failed to create output directory: {}", e)),
                logs,
            });
        }
        logs.push("[INFO] Output directory created".to_string());
    }

    // Step 4: Build game
    logs.push("[INFO] Step 3: Building game...".to_string());
    let build_result = GameEngineManager::build_game(&cwd, &request.target, &output_path).await;

    match build_result {
        Ok(result) => {
            if result.success {
                // Get file size
                let file_size = std::fs::metadata(&result.output_path)
                    .map(|m| m.len() as f64 / 1024.0 / 1024.0)
                    .ok();

                logs.push(format!("[INFO] Build successful!"));
                logs.push(format!("[INFO] Output: {}", result.output_path.display()));
                logs.push(format!("[INFO] Build time: {}ms", result.build_time_ms));
                if let Some(size) = file_size {
                    logs.push(format!("[INFO] File size: {:.2} MB", size));
                }

                Ok(GameExportResponse {
                    success: true,
                    engine: engine_name.to_string(),
                    output_path: Some(result.output_path.to_string_lossy().to_string()),
                    build_time_ms: result.build_time_ms,
                    file_size_mb: file_size,
                    error: None,
                    logs,
                })
            } else {
                logs.push(format!("[ERROR] Build failed: {:?}", result.error));
                Ok(GameExportResponse {
                    success: false,
                    engine: engine_name.to_string(),
                    output_path: None,
                    build_time_ms: result.build_time_ms,
                    file_size_mb: None,
                    error: result.error,
                    logs,
                })
            }
        }
        Err(e) => {
            logs.push(format!("[ERROR] Build error: {}", e));
            Ok(GameExportResponse {
                success: false,
                engine: engine_name.to_string(),
                output_path: None,
                build_time_ms: 0,
                file_size_mb: None,
                error: Some(e.to_string()),
                logs,
            })
        }
    }
}

/// Get available export targets for current project
#[tauri::command]
pub async fn game_get_export_targets(cwd: String) -> Result<Vec<String>, String> {
    let cwd_path = PathBuf::from(&cwd);

    let info = GameDetector::detect(&cwd_path)
        .map_err(|e| format!("Failed to detect game: {}", e))?;

    let targets = match info.engine {
        GameEngine::Godot => vec![
            "windows".to_string(),
            "macos".to_string(),
            "linux".to_string(),
            "android".to_string(),
            "ios".to_string(),
            "web".to_string(),
        ],
        GameEngine::Unity => vec![
            "windows".to_string(),
            "macos".to_string(),
            "linux".to_string(),
            "android".to_string(),
            "ios".to_string(),
            "webgl".to_string(),
        ],
        GameEngine::Unknown => vec![],
    };

    Ok(targets)
}

/// Validate export configuration
#[tauri::command]
pub async fn game_validate_export(request: GameExportRequest) -> Result<Vec<String>, String> {
    let cwd = PathBuf::from(&request.cwd);
    let mut warnings = Vec::new();

    // Check if project exists
    if !cwd.exists() {
        return Err("Project directory does not exist".to_string());
    }

    // Detect game
    let info = GameDetector::detect(&cwd)
        .map_err(|e| format!("Failed to detect game: {}", e))?;

    // Engine-specific checks
    match info.engine {
        GameEngine::Godot => {
            // Check if Godot is installed
            if std::env::var("GODOT_PATH").is_err() {
                // Try to run godot --version to check if it's in PATH
                let check = std::process::Command::new("godot")
                    .arg("--version")
                    .output();
                if check.is_err() {
                    warnings.push("Godot not found in PATH. Set GODOT_PATH environment variable.".to_string());
                }
            }

            // Check if export templates are installed
            // This is a simplified check
            warnings.push("Make sure export templates are installed for target platform.".to_string());
        }
        GameEngine::Unity => {
            // Check if Unity is installed
            if std::env::var("UNITY_PATH").is_err() {
                warnings.push("Unity not found. Set UNITY_PATH environment variable.".to_string());
            }

            // Check build target support
            match request.target.to_lowercase().as_str() {
                "android" => {
                    warnings.push("Android build requires JDK and Android SDK.".to_string());
                }
                "ios" => {
                    warnings.push("iOS build requires Xcode (Mac only).".to_string());
                }
                _ => {}
            }
        }
        GameEngine::Unknown => {
            return Err("Unknown game engine".to_string());
        }
    }

    // Check output directory
    if let Some(output) = &request.output {
        let output_path = PathBuf::from(output);
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                warnings.push(format!("Output directory will be created: {}", parent.display()));
            }
        }
    }

    // Check disk space (simplified)
    warnings.push("Ensure sufficient disk space for export.".to_string());

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_export_request() {
        let request = GameExportRequest {
            cwd: "/path/to/project".to_string(),
            target: "windows".to_string(),
            output: None,
            development: Some(false),
        };

        assert_eq!(request.target, "windows");
    }

    #[test]
    fn test_game_export_response() {
        let response = GameExportResponse {
            success: true,
            engine: "Godot".to_string(),
            output_path: Some("/path/to/game.exe".to_string()),
            build_time_ms: 5000,
            file_size_mb: Some(25.5),
            error: None,
            logs: vec!["Build successful".to_string()],
        };

        assert!(response.success);
        assert_eq!(response.engine, "Godot");
    }
}
