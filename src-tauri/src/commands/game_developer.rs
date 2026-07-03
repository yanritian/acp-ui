// Game Developer Agent Commands - Tauri command layer for AI game code generation
//
// Phase 3: Game Development Agents
// Provides frontend commands for AI-powered game code generation

use crate::agent_adapter::game_developer_agent::{GameDeveloperAgent, CodeGenRequest, CodeOutputFormat, GeneratedCode};
use crate::agent_adapter::game_designer_agent::{GameDesignDoc, GameEngine};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

/// Game Developer state for Tauri
pub struct GameDeveloperState {
    pub agent: Mutex<GameDeveloperAgent>,
}

impl GameDeveloperState {
    pub fn new() -> Self {
        Self {
            agent: Mutex::new(GameDeveloperAgent::new()),
        }
    }
}

impl Default for GameDeveloperState {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to generate code
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateCodeRequest {
    pub gdd_json: String,
    pub output_format: String,  // "SingleFile", "MultipleFiles", "ProjectStructure"
    pub target_files: Option<Vec<String>>,
}

/// Response for code generation
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateCodeResponse {
    pub success: bool,
    pub code_json: Option<String>,
    pub error: Option<String>,
}

/// Request to generate specific file
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateFileRequest {
    pub gdd_json: String,
    pub file_type: String,  // "script", "main", "player", etc.
}

/// Response for file generation
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateFileResponse {
    pub success: bool,
    pub file_content: Option<String>,
    pub file_path: Option<String>,
    pub error: Option<String>,
}

/// Request to get supported engines
#[derive(Debug, Serialize, Deserialize)]
pub struct GetSupportedEnginesRequest {}

/// Response for supported engines
#[derive(Debug, Serialize, Deserialize)]
pub struct GetSupportedEnginesResponse {
    pub success: bool,
    pub engines: Option<Vec<EngineInfo>>,
    pub error: Option<String>,
}

/// Engine information
#[derive(Debug, Serialize, Deserialize)]
pub struct EngineInfo {
    pub name: String,
    pub display_name: String,
    pub language: String,
    pub description: String,
    pub best_for: Vec<String>,
}

/// Generate complete code from GDD
#[tauri::command]
pub async fn game_developer_generate_code(
    state: State<'_, GameDeveloperState>,
    request: GenerateCodeRequest,
) -> Result<GenerateCodeResponse, String> {
    let agent = state.agent.lock().await;

    // Parse GDD from JSON
    let gdd: GameDesignDoc = match serde_json::from_str(&request.gdd_json) {
        Ok(g) => g,
        Err(e) => {
            return Ok(GenerateCodeResponse {
                success: false,
                code_json: None,
                error: Some(format!("Failed to parse GDD: {}", e)),
            });
        }
    };

    // Parse output format
    let output_format = match request.output_format.as_str() {
        "SingleFile" => CodeOutputFormat::SingleFile,
        "MultipleFiles" => CodeOutputFormat::MultipleFiles,
        "ProjectStructure" => CodeOutputFormat::ProjectStructure,
        _ => CodeOutputFormat::MultipleFiles,
    };

    let code_request = CodeGenRequest {
        gdd,
        output_format,
        target_files: request.target_files,
    };

    match agent.generate_code(&code_request).await {
        Ok(generated_code) => {
            match serde_json::to_string_pretty(&generated_code) {
                Ok(json) => Ok(GenerateCodeResponse {
                    success: true,
                    code_json: Some(json),
                    error: None,
                }),
                Err(e) => Ok(GenerateCodeResponse {
                    success: false,
                    code_json: None,
                    error: Some(format!("Failed to serialize code: {}", e)),
                }),
            }
        }
        Err(e) => Ok(GenerateCodeResponse {
            success: false,
            code_json: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Generate specific file
#[tauri::command]
pub async fn game_developer_generate_file(
    state: State<'_, GameDeveloperState>,
    request: GenerateFileRequest,
) -> Result<GenerateFileResponse, String> {
    let agent = state.agent.lock().await;

    // Parse GDD from JSON
    let gdd: GameDesignDoc = match serde_json::from_str(&request.gdd_json) {
        Ok(g) => g,
        Err(e) => {
            return Ok(GenerateFileResponse {
                success: false,
                file_content: None,
                file_path: None,
                error: Some(format!("Failed to parse GDD: {}", e)),
            });
        }
    };

    // For now, generate all code and return the requested file
    let code_request = CodeGenRequest {
        gdd,
        output_format: CodeOutputFormat::MultipleFiles,
        target_files: None,
    };

    match agent.generate_code(&code_request).await {
        Ok(generated_code) => {
            // Find the requested file
            let file = generated_code.files.iter().find(|f| {
                f.path.to_lowercase().contains(&request.file_type.to_lowercase())
            });

            if let Some(f) = file {
                Ok(GenerateFileResponse {
                    success: true,
                    file_content: Some(f.content.clone()),
                    file_path: Some(f.path.clone()),
                    error: None,
                })
            } else {
                Ok(GenerateFileResponse {
                    success: false,
                    file_content: None,
                    file_path: None,
                    error: Some(format!("File type '{}' not found", request.file_type)),
                })
            }
        }
        Err(e) => Ok(GenerateFileResponse {
            success: false,
            file_content: None,
            file_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Get supported engines
#[tauri::command]
pub async fn game_developer_get_supported_engines(
    _state: State<'_, GameDeveloperState>,
    _request: GetSupportedEnginesRequest,
) -> Result<GetSupportedEnginesResponse, String> {
    let engines = vec![
        EngineInfo {
            name: "RenPy".to_string(),
            display_name: "Ren'Py".to_string(),
            language: "Ren'Py Script".to_string(),
            description: "Visual Novel Engine - Perfect for text-heavy games with branching narratives".to_string(),
            best_for: vec![
                "Visual Novels".to_string(),
                "Dating Sims".to_string(),
                "Interactive Stories".to_string(),
            ],
        },
        EngineInfo {
            name: "Godot".to_string(),
            display_name: "Godot".to_string(),
            language: "GDScript".to_string(),
            description: "Open source 2D/3D engine with intuitive scene system".to_string(),
            best_for: vec![
                "2D Games".to_string(),
                "Indie Games".to_string(),
                "Mobile Games".to_string(),
            ],
        },
        EngineInfo {
            name: "Unity".to_string(),
            display_name: "Unity".to_string(),
            language: "C#".to_string(),
            description: "Versatile engine with large ecosystem and asset store".to_string(),
            best_for: vec![
                "2D/3D Games".to_string(),
                "Mobile Games".to_string(),
                "VR/AR".to_string(),
            ],
        },
        EngineInfo {
            name: "Unreal".to_string(),
            display_name: "Unreal Engine".to_string(),
            language: "C++ / Blueprint".to_string(),
            description: "Industry standard for AAA games with cutting-edge graphics".to_string(),
            best_for: vec![
                "AAA Games".to_string(),
                "FPS Games".to_string(),
                "High-End 3D".to_string(),
            ],
        },
    ];

    Ok(GetSupportedEnginesResponse {
        success: true,
        engines: Some(engines),
        error: None,
    })
}
