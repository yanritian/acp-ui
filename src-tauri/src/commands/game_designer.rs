// Game Designer Agent Commands - Tauri command layer for AI game design
//
// Phase 3: Game Development Agents
// Provides frontend commands for AI-powered game design

use crate::agent_adapter::game_designer_agent::{GameDesignerAgent, DesignRequest, DesignConstraints, GameEngine};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

/// Game Designer state for Tauri
pub struct GameDesignerState {
    pub agent: Mutex<GameDesignerAgent>,
}

impl GameDesignerState {
    pub fn new() -> Self {
        Self {
            agent: Mutex::new(GameDesignerAgent::new()),
        }
    }
}

impl Default for GameDesignerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to generate game design document
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateGDDRequest {
    pub user_input: String,
    pub constraints: Option<DesignConstraintsRequest>,
}

/// Design constraints for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct DesignConstraintsRequest {
    pub preferred_engine: Option<String>,  // "RenPy", "Godot", "Unity", "Unreal"
    pub target_platform: Option<String>,
    pub max_development_time: Option<u32>,  // days
    pub team_size: Option<u32>,
}

/// Response for GDD generation
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateGDDResponse {
    pub success: bool,
    pub gdd_json: Option<String>,
    pub error: Option<String>,
}

/// Request to generate game concept only
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateConceptRequest {
    pub user_input: String,
}

/// Response for concept generation
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateConceptResponse {
    pub success: bool,
    pub concept_json: Option<String>,
    pub error: Option<String>,
}

/// Request to recommend engine
#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendEngineRequest {
    pub genre: String,
    pub scope: String,  // "small", "medium", "large"
}

/// Response for engine recommendation
#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendEngineResponse {
    pub success: bool,
    pub engine: Option<String>,
    pub reason: Option<String>,
    pub error: Option<String>,
}

/// Generate complete Game Design Document
#[tauri::command]
pub async fn game_designer_generate_gdd(
    state: State<'_, GameDesignerState>,
    request: GenerateGDDRequest,
) -> Result<GenerateGDDResponse, String> {
    let agent = state.agent.lock().await;

    // Convert constraints
    let constraints = request.constraints.map(|c| {
        let preferred_engine = c.preferred_engine.and_then(|e| {
            match e.to_lowercase().as_str() {
                "renpy" | "ren'py" => Some(GameEngine::RenPy),
                "godot" => Some(GameEngine::Godot),
                "unity" => Some(GameEngine::Unity),
                "unreal" => Some(GameEngine::Unreal),
                _ => None,
            }
        });

        DesignConstraints {
            preferred_engine,
            target_platform: c.target_platform,
            max_development_time: c.max_development_time,
            team_size: c.team_size,
        }
    });

    let design_request = DesignRequest {
        user_input: request.user_input,
        constraints,
    };

    match agent.generate_gdd(&design_request).await {
        Ok(gdd) => {
            match serde_json::to_string_pretty(&gdd) {
                Ok(json) => Ok(GenerateGDDResponse {
                    success: true,
                    gdd_json: Some(json),
                    error: None,
                }),
                Err(e) => Ok(GenerateGDDResponse {
                    success: false,
                    gdd_json: None,
                    error: Some(format!("Failed to serialize GDD: {}", e)),
                }),
            }
        }
        Err(e) => Ok(GenerateGDDResponse {
            success: false,
            gdd_json: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Generate game concept only
#[tauri::command]
pub async fn game_designer_generate_concept(
    state: State<'_, GameDesignerState>,
    request: GenerateConceptRequest,
) -> Result<GenerateConceptResponse, String> {
    let agent = state.agent.lock().await;

    // This is a simplified version - in production, you'd have a separate method
    // For now, generate full GDD and extract concept
    let design_request = DesignRequest {
        user_input: request.user_input,
        constraints: None,
    };

    match agent.generate_gdd(&design_request).await {
        Ok(gdd) => {
            match serde_json::to_string_pretty(&gdd.concept) {
                Ok(json) => Ok(GenerateConceptResponse {
                    success: true,
                    concept_json: Some(json),
                    error: None,
                }),
                Err(e) => Ok(GenerateConceptResponse {
                    success: false,
                    concept_json: None,
                    error: Some(format!("Failed to serialize concept: {}", e)),
                }),
            }
        }
        Err(e) => Ok(GenerateConceptResponse {
            success: false,
            concept_json: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Recommend game engine
#[tauri::command]
pub async fn game_designer_recommend_engine(
    state: State<'_, GameDesignerState>,
    request: RecommendEngineRequest,
) -> Result<RecommendEngineResponse, String> {
    let agent = state.agent.lock().await;

    // Create a minimal concept for engine recommendation
    let concept = crate::agent_adapter::game_designer_agent::GameConcept {
        title: "Unknown".to_string(),
        genre: request.genre,
        core_idea: "User request".to_string(),
        target_audience: "General".to_string(),
        estimated_scope: match request.scope.to_lowercase().as_str() {
            "small" => crate::agent_adapter::game_designer_agent::ScopeLevel::Small,
            "medium" => crate::agent_adapter::game_designer_agent::ScopeLevel::Medium,
            "large" => crate::agent_adapter::game_designer_agent::ScopeLevel::Large,
            _ => crate::agent_adapter::game_designer_agent::ScopeLevel::Medium,
        },
    };

    let engine = agent.recommend_engine(&concept);

    let reason = match engine {
        GameEngine::RenPy => "Best for visual novels and text-heavy games. Simple scripting language, fast development.".to_string(),
        GameEngine::Godot => "Great for 2D and small-to-medium 3D games. Open source, lightweight, GDScript is easy to learn.".to_string(),
        GameEngine::Unity => "Versatile engine for 2D/3D games. Large ecosystem, C# programming, good for mobile and indie games.".to_string(),
        GameEngine::Unreal => "Industry standard for AAA games. High-quality graphics, C++/Blueprint, best for large-scale 3D projects.".to_string(),
    };

    Ok(RecommendEngineResponse {
        success: true,
        engine: Some(format!("{:?}", engine)),
        reason: Some(reason),
        error: None,
    })
}
