// Game Development Commands
// Phase 1: Game engine detection and integration

use crate::game_detector::{GameDetector, GameEngine, GameInfo, GameSize};
use crate::agent_adapter::godot_adapter::GodotAdapter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ===== Response Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetectResponse {
    pub is_game: bool,
    pub engine: Option<String>,
    pub size: Option<String>,
    pub scenes: Vec<String>,
    pub project_name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GodotDetectResponse {
    pub is_godot_project: bool,
    pub scenes: Vec<String>,
}

// ===== Commands =====

/// Detect game engine and project details
#[tauri::command]
pub async fn game_detect(cwd: String) -> Result<GameDetectResponse, String> {
    let cwd_path = PathBuf::from(&cwd);

    match GameDetector::detect(&cwd_path) {
        Ok(info) => {
            let engine = match info.engine {
                GameEngine::Godot => Some("Godot".to_string()),
                GameEngine::Unity => Some("Unity".to_string()),
                GameEngine::Unknown => None,
            };

            let size = match info.size {
                GameSize::Small => Some("small".to_string()),
                GameSize::Large => Some("large".to_string()),
            };

            Ok(GameDetectResponse {
                is_game: engine.is_some(),
                engine,
                size,
                scenes: info.scenes,
                project_name: info.project_name,
                version: info.version,
            })
        }
        Err(_) => {
            Ok(GameDetectResponse {
                is_game: false,
                engine: None,
                size: None,
                scenes: Vec::new(),
                project_name: "Unknown".to_string(),
                version: None,
            })
        }
    }
}

/// Detect if current directory is a Godot project (legacy)
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
