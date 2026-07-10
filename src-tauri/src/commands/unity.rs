// Unity Game Development Commands
// Phase 1 Day 4: Unity integration

use crate::game_detector::GameDetector;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

// ===== Request/Response Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityBuildRequest {
    pub cwd: String,
    pub target: String, // "Win64", "OSX", "Linux64", "Android", "iOS", "WebGL"
    pub output: String,
    pub development: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityBuildResponse {
    pub success: bool,
    pub output_path: Option<String>,
    pub build_time_ms: u64,
    pub error: Option<String>,
}

// ===== Commands =====

/// Build Unity project
#[tauri::command]
pub async fn unity_build(request: UnityBuildRequest) -> Result<UnityBuildResponse, String> {
    let cwd = PathBuf::from(&request.cwd);
    let output = PathBuf::from(&request.output);

    // Validate Unity project
    let info =
        GameDetector::detect(&cwd).map_err(|e| format!("Not a valid game project: {}", e))?;

    if info.engine != crate::game_detector::GameEngine::Unity {
        return Err("Not a Unity project".to_string());
    }

    // Find Unity executable
    let unity_path = find_unity_executable().map_err(|e| format!("Unity not found: {}", e))?;

    let start = std::time::Instant::now();

    // Build command
    let mut cmd = Command::new(&unity_path);
    cmd.arg("-quit")
        .arg("-batchmode")
        .arg("-projectPath")
        .arg(&cwd)
        .arg("-buildTarget")
        .arg(&request.target)
        .arg("-buildApp")
        .arg(&output);

    // Add development flag if requested
    if request.development.unwrap_or(false) {
        cmd.arg("-development");
    }

    // Execute build
    let result = cmd.output();

    let build_time = start.elapsed().as_millis() as u64;

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(UnityBuildResponse {
                    success: true,
                    output_path: Some(request.output),
                    build_time_ms: build_time,
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(UnityBuildResponse {
                    success: false,
                    output_path: None,
                    build_time_ms: build_time,
                    error: Some(format!("Build failed: {}", stderr)),
                })
            }
        }
        Err(e) => Ok(UnityBuildResponse {
            success: false,
            output_path: None,
            build_time_ms: build_time,
            error: Some(format!("Failed to run Unity: {}", e)),
        }),
    }
}

/// Find Unity executable
fn find_unity_executable() -> Result<String, String> {
    // 1. Check environment variable
    if let Ok(path) = std::env::var("UNITY_PATH") {
        if PathBuf::from(&path).exists() {
            return Ok(path);
        }
    }

    // 2. Check common locations
    let common_paths = vec![
        // Windows
        "C:\\Program Files\\Unity\\Hub\\Editor\\2021.3.0f1\\Editor\\Unity.exe",
        "C:\\Program Files\\Unity\\Hub\\Editor\\2022.3.0f1\\Editor\\Unity.exe",
        "C:\\Program Files\\Unity\\Hub\\Editor\\2023.1.0f1\\Editor\\Unity.exe",
        // MacOS
        "/Applications/Unity/Hub/Editor/2021.3.0f1/Unity.app/Contents/MacOS/Unity",
        "/Applications/Unity/Hub/Editor/2022.3.0f1/Unity.app/Contents/MacOS/Unity",
        "/Applications/Unity/Hub/Editor/2023.1.0f1/Unity.app/Contents/MacOS/Unity",
        // Linux
        "/opt/Unity/Hub/Editor/2021.3.0f1/Editor/Unity",
        "/opt/Unity/Hub/Editor/2022.3.0f1/Editor/Unity",
    ];

    for path in common_paths {
        if PathBuf::from(path).exists() {
            return Ok(path.to_string());
        }
    }

    Err("Unity not found. Please install Unity or set UNITY_PATH environment variable.".to_string())
}

/// Get available Unity build targets
#[tauri::command]
pub async fn unity_get_targets() -> Result<Vec<String>, String> {
    Ok(vec![
        "Win64".to_string(),
        "OSX".to_string(),
        "Linux64".to_string(),
        "Android".to_string(),
        "iOS".to_string(),
        "WebGL".to_string(),
    ])
}
