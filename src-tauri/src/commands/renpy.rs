// Ren'Py Game Commands - Tauri command layer for Ren'Py Adapter
//
// Phase 3: Game Development Agents
// Provides frontend commands for Ren'Py game development

use crate::agent_adapter::renpy_adapter::{RenPyAdapter, StorySpec, CharacterSpec, SceneSpec};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tokio::sync::Mutex;

/// Ren'Py state for Tauri
pub struct RenPyState {
    pub adapter: Mutex<RenPyAdapter>,
}

impl RenPyState {
    pub fn new() -> Self {
        Self {
            adapter: Mutex::new(RenPyAdapter::new()),
        }
    }
}

impl Default for RenPyState {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to get Ren'Py version
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyVersionRequest {}

/// Response for Ren'Py version
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyVersionResponse {
    pub success: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Request to create a Ren'Py project
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyCreateProjectRequest {
    pub name: String,
    pub path: String,
}

/// Response for creating project
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyCreateProjectResponse {
    pub success: bool,
    pub project_path: Option<String>,
    pub error: Option<String>,
}

/// Request to generate script
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyGenerateScriptRequest {
    pub project_path: String,
    pub story_spec: StorySpecRequest,
}

/// Story specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct StorySpecRequest {
    pub title: String,
    pub characters: Vec<CharacterSpecRequest>,
    pub scenes: Vec<SceneSpecRequest>,
    pub backgrounds: Vec<BackgroundSpecRequest>,
}

/// Character specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterSpecRequest {
    pub id: String,
    pub name: String,
    pub color: String,
    pub expressions: Vec<ExpressionSpecRequest>,
}

/// Expression specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ExpressionSpecRequest {
    pub name: String,
    pub image_filename: String,
}

/// Scene specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct SceneSpecRequest {
    pub label: String,
    pub background: Option<String>,
    pub dialogues: Vec<DialogueSpecRequest>,
    pub menu: Option<MenuSpecRequest>,
    pub next_label: Option<String>,
}

/// Dialogue specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct DialogueSpecRequest {
    pub speaker: Option<String>,
    pub text: String,
    pub character_shows: Vec<CharacterShowSpecRequest>,
}

/// Character show specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterShowSpecRequest {
    pub character_id: String,
    pub expression: String,
    pub position: String,
}

/// Menu specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct MenuSpecRequest {
    pub prompt: Option<String>,
    pub choices: Vec<ChoiceSpecRequest>,
}

/// Choice specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct ChoiceSpecRequest {
    pub text: String,
    pub target_label: String,
}

/// Background specification for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct BackgroundSpecRequest {
    pub id: String,
    pub image_filename: String,
}

/// Response for generating script
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyGenerateScriptResponse {
    pub success: bool,
    pub script_path: Option<String>,
    pub error: Option<String>,
}

/// Request to run game
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyRunGameRequest {
    pub project_path: String,
}

/// Response for running game
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyRunGameResponse {
    pub success: bool,
    pub process_id: Option<u32>,
    pub error: Option<String>,
}

/// Request to compile game
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyCompileGameRequest {
    pub project_path: String,
}

/// Response for compiling game
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyCompileGameResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Request to lint game
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyLintGameRequest {
    pub project_path: String,
}

/// Response for linting game
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyLintGameResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Request to detect project
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyDetectProjectRequest {
    pub path: String,
}

/// Response for detecting project
#[derive(Debug, Serialize, Deserialize)]
pub struct RenPyDetectProjectResponse {
    pub success: bool,
    pub is_project: bool,
    pub project_name: Option<String>,
    pub error: Option<String>,
}

/// Get Ren'Py version
#[tauri::command]
pub async fn renpy_get_version(
    state: State<'_, RenPyState>,
    _request: RenPyVersionRequest,
) -> Result<RenPyVersionResponse, String> {
    let adapter = state.adapter.lock().await;

    match adapter.get_version().await {
        Ok(version) => Ok(RenPyVersionResponse {
            success: true,
            version: Some(version),
            error: None,
        }),
        Err(e) => Ok(RenPyVersionResponse {
            success: false,
            version: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Create a new Ren'Py project
#[tauri::command]
pub async fn renpy_create_project(
    state: State<'_, RenPyState>,
    request: RenPyCreateProjectRequest,
) -> Result<RenPyCreateProjectResponse, String> {
    let adapter = state.adapter.lock().await;
    let base_path = PathBuf::from(&request.path);

    match adapter.create_project(&request.name, &base_path).await {
        Ok(project_path) => Ok(RenPyCreateProjectResponse {
            success: true,
            project_path: Some(project_path.to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(RenPyCreateProjectResponse {
            success: false,
            project_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Generate Ren'Py script from story specification
#[tauri::command]
pub async fn renpy_generate_script(
    state: State<'_, RenPyState>,
    request: RenPyGenerateScriptRequest,
) -> Result<RenPyGenerateScriptResponse, String> {
    let adapter = state.adapter.lock().await;
    let project_path = PathBuf::from(&request.project_path);

    // Convert request to internal types
    let story_spec = StorySpec {
        title: request.story_spec.title,
        characters: request.story_spec.characters
            .into_iter()
            .map(|c| CharacterSpec {
                id: c.id,
                name: c.name,
                color: c.color,
                expressions: c.expressions
                    .into_iter()
                    .map(|e| crate::agent_adapter::renpy_adapter::ExpressionSpec {
                        name: e.name,
                        image_filename: e.image_filename,
                    })
                    .collect(),
            })
            .collect(),
        scenes: request.story_spec.scenes
            .into_iter()
            .map(|s| SceneSpec {
                label: s.label,
                background: s.background,
                dialogues: s.dialogues
                    .into_iter()
                    .map(|d| crate::agent_adapter::renpy_adapter::DialogueSpec {
                        speaker: d.speaker,
                        text: d.text,
                        character_shows: d.character_shows
                            .into_iter()
                            .map(|cs| crate::agent_adapter::renpy_adapter::CharacterShowSpec {
                                character_id: cs.character_id,
                                expression: cs.expression,
                                position: cs.position,
                            })
                            .collect(),
                    })
                    .collect(),
                menu: s.menu.map(|m| crate::agent_adapter::renpy_adapter::MenuSpec {
                    prompt: m.prompt,
                    choices: m.choices
                        .into_iter()
                        .map(|c| crate::agent_adapter::renpy_adapter::ChoiceSpec {
                            text: c.text,
                            target_label: c.target_label,
                        })
                        .collect(),
                }),
                next_label: s.next_label,
            })
            .collect(),
        backgrounds: request.story_spec.backgrounds
            .into_iter()
            .map(|b| crate::agent_adapter::renpy_adapter::BackgroundSpec {
                id: b.id,
                image_filename: b.image_filename,
            })
            .collect(),
    };

    match adapter.generate_script(&story_spec, &project_path).await {
        Ok(_) => Ok(RenPyGenerateScriptResponse {
            success: true,
            script_path: Some(project_path.join("game/script.rpy").to_string_lossy().to_string()),
            error: None,
        }),
        Err(e) => Ok(RenPyGenerateScriptResponse {
            success: false,
            script_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Run the game
#[tauri::command]
pub async fn renpy_run_game(
    state: State<'_, RenPyState>,
    request: RenPyRunGameRequest,
) -> Result<RenPyRunGameResponse, String> {
    let adapter = state.adapter.lock().await;
    let project_path = PathBuf::from(&request.project_path);

    match adapter.run_game(&project_path).await {
        Ok(pid) => Ok(RenPyRunGameResponse {
            success: true,
            process_id: Some(pid),
            error: None,
        }),
        Err(e) => Ok(RenPyRunGameResponse {
            success: false,
            process_id: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Compile the game
#[tauri::command]
pub async fn renpy_compile_game(
    state: State<'_, RenPyState>,
    request: RenPyCompileGameRequest,
) -> Result<RenPyCompileGameResponse, String> {
    let adapter = state.adapter.lock().await;
    let project_path = PathBuf::from(&request.project_path);

    match adapter.compile_game(&project_path).await {
        Ok(output) => Ok(RenPyCompileGameResponse {
            success: true,
            output: Some(output),
            error: None,
        }),
        Err(e) => Ok(RenPyCompileGameResponse {
            success: false,
            output: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Lint the game
#[tauri::command]
pub async fn renpy_lint_game(
    state: State<'_, RenPyState>,
    request: RenPyLintGameRequest,
) -> Result<RenPyLintGameResponse, String> {
    let adapter = state.adapter.lock().await;
    let project_path = PathBuf::from(&request.project_path);

    match adapter.lint_game(&project_path).await {
        Ok(output) => Ok(RenPyLintGameResponse {
            success: true,
            output: Some(output),
            error: None,
        }),
        Err(e) => Ok(RenPyLintGameResponse {
            success: false,
            output: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Detect if a directory is a Ren'Py project
#[tauri::command]
pub async fn renpy_detect_project(
    state: State<'_, RenPyState>,
    request: RenPyDetectProjectRequest,
) -> Result<RenPyDetectProjectResponse, String> {
    let path = PathBuf::from(&request.path);
    let is_project = RenPyAdapter::detect_renpy_project(&path);

    // Try to get project name from project.json
    let project_name = if is_project {
        let project_json_path = path.join("project.json");
        if project_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&project_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json.get("display_name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(RenPyDetectProjectResponse {
        success: true,
        is_project,
        project_name,
        error: None,
    })
}
