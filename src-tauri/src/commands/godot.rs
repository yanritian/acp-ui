// Godot Game Development Commands
// Phase 2: Additional Tauri commands for Godot Adapter
// Note: Basic commands (godot_build_dev, godot_build_release, godot_get_scenes) are in self_optimizing.rs

use crate::agent_adapter::godot_adapter::GodotAdapter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ===== Response Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GodotDetectResponse {
    pub is_godot_project: bool,
    pub scenes: Vec<String>,
}

// ===== Commands =====

/// Detect if current directory is a Godot project
#[tauri::command]
pub async fn godot_detect(cwd: String) -> Result<GodotDetectResponse, String> {
    let cwd_path = PathBuf::from(&cwd);

    let is_godot_project = GodotAdapter::detect_godot_project(&cwd_path);
    let scenes = if is_godot_project {
        GodotAdapter::get_scenes(&cwd_path)
    } else {
        Vec::new()
    };

    Ok(GodotDetectResponse {
        is_godot_project,
        scenes,
    })
}
