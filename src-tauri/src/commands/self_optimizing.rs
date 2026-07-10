// Self-Optimizing Router Commands - Tauri commands for intelligent routing

use crate::agent_adapter::types::{
    AgentConfig, AgentResult, AgentTask, ExecutionRecord, Platform, SceneType, TaskConstraints,
    TaskInput,
};
use crate::privacy_orchestrator::{
    PrivacyDecision, PrivacyLevel, PrivacyOrchestrator, PrivacyZone,
};
use crate::project_context::{ContextSummary, FileInfo, ProjectContext};
use crate::self_optimizing_router::{HistorySummary, ProficiencyScore, SelfOptimizingConfig};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

/// Initialize Self-Optimizing Router with config
#[tauri::command]
pub async fn self_optimizing_init(
    state: State<'_, AppState>,
    min_history_size: Option<u32>,
    ewma_alpha: Option<f32>,
    cost_weight: Option<f32>,
    performance_weight: Option<f32>,
    privacy_weight: Option<f32>,
) -> Result<(), String> {
    let config = SelfOptimizingConfig {
        min_history_size: min_history_size.unwrap_or(10),
        ewma_alpha: ewma_alpha.unwrap_or(0.15),
        cost_weight: cost_weight.unwrap_or(0.3),
        performance_weight: performance_weight.unwrap_or(0.4),
        privacy_weight: privacy_weight.unwrap_or(0.3),
    };

    // In real implementation, we'd store the router in AppState
    // For now, just return success
    Ok(())
}

/// Record execution for self-optimization learning
#[tauri::command]
pub async fn self_optimizing_record(
    state: State<'_, AppState>,
    task_id: String,
    scene_type: String,
    platform: String,
    agent_id: String,
    success: bool,
    duration_ms: u64,
    cost: f32,
) -> Result<(), String> {
    let scene = parse_scene(&scene_type)?;
    let plat = parse_platform(&platform)?;

    let record = ExecutionRecord {
        task_id,
        scene_type: scene,
        platform: plat,
        agent_id,
        success,
        duration_ms,
        cost,
        timestamp: 0, // Will be set by router
    };

    // Record in agent registry for proficiency tracking
    {
        let mut registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
        registry.record_execution(
            &record.agent_id,
            record.scene_type.clone(),
            record.platform.clone(),
            record.success,
            record.duration_ms,
            record.cost,
        );
    }

    Ok(())
}

/// Get optimal agent recommendation
#[tauri::command]
pub async fn self_optimizing_route(
    state: State<'_, AppState>,
    input: String,
    input_type: String,
    scene_type: Option<String>,
    platform: Option<String>,
) -> Result<SelfOptimizingRouteResult, String> {
    // Get available agents from registry
    let available_agents: Vec<String> = {
        let registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
        registry.list_bases().iter().map(|b| b.id.clone()).collect()
    };

    // If no agents, return default
    if available_agents.is_empty() {
        return Ok(SelfOptimizingRouteResult {
            selected_agent: "claude-code".to_string(),
            optimization_applied: false,
            proficiency_scores: vec![],
            reason: "No registered agents".to_string(),
        });
    }

    let scene = scene_type.map(|s| parse_scene(&s)).transpose()?;
    let plat = platform.map(|p| parse_platform(&p)).transpose()?;

    // Get proficiencies from registry
    let proficiencies: Vec<ProficiencyScore> = {
        let registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
        registry.get_all_proficiencies()
    };

    let proficiencies_len = proficiencies.len();

    // Find best agent
    let best_agent = {
        let registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
        if let (Some(s), Some(p)) = (&scene, &plat) {
            registry.get_best_agent_for(s.clone(), p.clone())
        } else {
            None
        }
    };

    let selected_agent = best_agent.unwrap_or_else(|| {
        // Default to first available
        available_agents
            .first()
            .cloned()
            .unwrap_or_else(|| "claude-code".to_string())
    });

    Ok(SelfOptimizingRouteResult {
        selected_agent: selected_agent.clone(),
        optimization_applied: proficiencies_len >= 10,
        proficiency_scores: proficiencies,
        reason: if proficiencies_len >= 10 {
            format!(
                "Selected {} based on historical performance",
                selected_agent
            )
        } else {
            format!("Selected {} (insufficient history)", selected_agent)
        },
    })
}

/// Get agent proficiency scores
#[tauri::command]
pub async fn self_optimizing_get_proficiencies(
    state: State<'_, AppState>,
    agent_id: Option<String>,
) -> Result<Vec<ProficiencyScore>, String> {
    let registry = state.agent_registry.lock().map_err(|e| e.to_string())?;

    if let Some(id) = agent_id {
        Ok(registry.get_agent_proficiencies(&id).unwrap_or_default())
    } else {
        Ok(registry.get_all_proficiencies())
    }
}

/// Get history summary
#[tauri::command]
pub async fn self_optimizing_get_history(
    state: State<'_, AppState>,
) -> Result<HistorySummary, String> {
    // In real implementation, we'd get from router
    // For now, calculate from proficiencies
    let registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
    let proficiencies = registry.get_all_proficiencies();

    let total = proficiencies
        .iter()
        .map(|p| p.success_count + p.failure_count)
        .sum::<u32>() as usize;
    let successes = proficiencies.iter().map(|p| p.success_count).sum::<u32>() as usize;
    let avg_latency = if total > 0 {
        proficiencies.iter().map(|p| p.avg_latency_ms).sum::<u64>() / proficiencies.len() as u64
    } else {
        0
    };
    let total_cost = proficiencies
        .iter()
        .map(|p| p.avg_cost * (p.success_count + p.failure_count) as f32)
        .sum();

    Ok(HistorySummary {
        total_executions: total,
        success_count: successes,
        failure_count: total - successes,
        success_rate: if total > 0 {
            successes as f32 / total as f32
        } else {
            0.0
        },
        avg_latency_ms: avg_latency,
        total_cost,
    })
}

/// Reset proficiency scores
#[tauri::command]
pub async fn self_optimizing_reset(state: State<'_, AppState>) -> Result<(), String> {
    let mut registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
    registry.reset_proficiencies();
    Ok(())
}

// === Project Context Commands ===

/// Create a new project context
#[tauri::command]
pub async fn project_context_create(
    state: State<'_, AppState>,
    id: String,
    name: String,
    root_path: String,
) -> Result<ProjectContext, String> {
    // In real implementation, we'd store in AppState
    Ok(ProjectContext::new(
        id,
        name,
        std::path::PathBuf::from(root_path),
    ))
}

/// Update project context with file info
#[tauri::command]
pub async fn project_context_update(
    state: State<'_, AppState>,
    id: String,
    files: Vec<FileInfoInput>,
) -> Result<ContextSummary, String> {
    let files: Vec<FileInfo> = files
        .iter()
        .map(|f| FileInfo {
            path: f.path.clone(),
            line_count: f.line_count,
            is_source: f.is_source,
            is_test: f.is_test,
            is_config: f.is_config,
            is_doc: f.is_doc,
        })
        .collect();

    // Create context and detect from files
    let mut ctx = ProjectContext::new(
        id.clone(),
        "Project".to_string(),
        std::path::PathBuf::from("."),
    );
    ctx.detect_from_files(&files);

    Ok(ctx.get_summary())
}

/// Get context summary for routing
#[tauri::command]
pub async fn project_context_get_summary(
    state: State<'_, AppState>,
    id: String,
) -> Result<ContextSummary, String> {
    // In real implementation, we'd get from stored context
    Ok(ContextSummary {
        language: crate::project_context::ProjectLanguage::Unknown,
        framework: None,
        scene: crate::project_context::Scene::Unknown,
        platform: crate::project_context::Platform::Unknown,
        file_count: 0,
        recent_change_count: 0,
        pattern_count: 0,
    })
}

// === Result Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfOptimizingRouteResult {
    pub selected_agent: String,
    pub optimization_applied: bool,
    pub proficiency_scores: Vec<ProficiencyScore>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoInput {
    pub path: String,
    pub line_count: usize,
    pub is_source: bool,
    pub is_test: bool,
    pub is_config: bool,
    pub is_doc: bool,
}

// === Helper Functions ===

fn parse_scene(s: &str) -> Result<SceneType, String> {
    match s.to_lowercase().as_str() {
        "web" | "webdevelopment" => Ok(SceneType::WebDevelopment),
        "mini" | "miniprogram" => Ok(SceneType::MiniProgramDevelopment),
        "desktop" => Ok(SceneType::DesktopDevelopment),
        "game" => Ok(SceneType::GameDevelopment),
        "game_art" => Ok(SceneType::GameArtGeneration),
        "game_cross" => Ok(SceneType::GameCrossPlatform),
        "game_perf" => Ok(SceneType::GamePerformanceOptimization),
        "marketing" => Ok(SceneType::Marketing),
        "finance" => Ok(SceneType::Finance),
        "design" => Ok(SceneType::Design),
        other => Err(format!("Unknown scene type: {}", other)),
    }
}

fn parse_platform(s: &str) -> Result<Platform, String> {
    match s.to_lowercase().as_str() {
        "web" => Ok(Platform::Web),
        "mini" | "miniprogram" => Ok(Platform::MiniProgram),
        "desktop" => Ok(Platform::Desktop),
        "mobile" => Ok(Platform::Mobile),
        "game" => Ok(Platform::Game),
        other => Err(format!("Unknown platform: {}", other)),
    }
}

// === Privacy Orchestrator Commands (Phase 1 Week 3) ===

/// Analyze task for privacy requirements
#[tauri::command]
pub async fn privacy_analyze_task(
    state: State<'_, AppState>,
    task_id: String,
    content: String,
    file_paths: Vec<String>,
    command: Option<String>,
) -> Result<PrivacyDecision, String> {
    let orchestrator = PrivacyOrchestrator::new();
    let decision = orchestrator.analyze_task(&task_id, &content, &file_paths, command.as_deref());
    Ok(decision)
}

/// Get active privacy zone
#[tauri::command]
pub async fn privacy_get_active_zone(
    state: State<'_, AppState>,
) -> Result<Option<PrivacyZone>, String> {
    let orchestrator = PrivacyOrchestrator::new();
    Ok(orchestrator.get_active_zone().cloned())
}

/// Set active privacy zone
#[tauri::command]
pub async fn privacy_set_zone(state: State<'_, AppState>, zone_id: String) -> Result<(), String> {
    // In real implementation, we'd store orchestrator in AppState
    Ok(())
}

/// Check if path is safe for external processing
#[tauri::command]
pub async fn privacy_check_path(state: State<'_, AppState>, path: String) -> Result<bool, String> {
    let orchestrator = PrivacyOrchestrator::new();
    Ok(orchestrator.is_path_safe(&path))
}

/// List all privacy zones
#[tauri::command]
pub async fn privacy_list_zones(state: State<'_, AppState>) -> Result<Vec<PrivacyZone>, String> {
    let orchestrator = PrivacyOrchestrator::new();
    Ok(orchestrator.list_zones())
}

/// Create custom privacy zone
#[tauri::command]
pub async fn privacy_create_zone(
    state: State<'_, AppState>,
    id: String,
    name: String,
    level: String,
) -> Result<PrivacyZone, String> {
    let privacy_level = match level.to_lowercase().as_str() {
        "strict" | "strictlocal" => PrivacyLevel::StrictLocal,
        "sanitized" => PrivacyLevel::Sanitized,
        "standard" => PrivacyLevel::Standard,
        "external" => PrivacyLevel::External,
        other => return Err(format!("Unknown privacy level: {}", other)),
    };

    let zone = PrivacyZone::new(id, name, privacy_level);
    Ok(zone)
}

// === Agent Adapter Execution Commands (Phase 1 Week 4) ===

use crate::agent_adapter::{
    claude_adapter::ClaudeCodeAdapter,
    codex_adapter::CodexAdapter,
    electron_adapter::{ElectronArch, ElectronDesktopAdapter, ElectronPlatform},
    tauri_adapter::{BundleType, DesktopPlatform, TauriDesktopAdapter},
    AgentAdapter,
};

/// Initialize Claude Code adapter
#[tauri::command]
pub async fn agent_adapter_init_claude(
    state: State<'_, AppState>,
    cwd: Option<String>,
) -> Result<String, String> {
    let mut adapter = ClaudeCodeAdapter::new();

    let config = AgentConfig {
        cwd,
        timeout_ms: 300000,
        max_retries: 3,
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;
    Ok(adapter.id().to_string())
}

/// Initialize Codex adapter
#[tauri::command]
pub async fn agent_adapter_init_codex(
    state: State<'_, AppState>,
    cwd: Option<String>,
) -> Result<String, String> {
    let mut adapter = CodexAdapter::new();

    let config = AgentConfig {
        cwd,
        timeout_ms: 300000,
        max_retries: 3,
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;
    Ok(adapter.id().to_string())
}

/// Execute task with Claude Code
#[tauri::command]
pub async fn agent_adapter_execute_claude(
    state: State<'_, AppState>,
    task_id: String,
    description: String,
    cwd: Option<String>,
    scene_type: Option<String>,
    platform: Option<String>,
) -> Result<AgentResult, String> {
    let mut adapter = ClaudeCodeAdapter::new();

    let config = AgentConfig {
        cwd,
        timeout_ms: 300000,
        max_retries: 3,
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;

    let scene = scene_type.map(|s| parse_scene(&s)).transpose()?;
    let plat = platform.map(|p| parse_platform(&p)).transpose()?;

    let desc_clone = description.clone();
    let task = AgentTask {
        id: task_id,
        description,
        input: TaskInput::Text(desc_clone),
        constraints: TaskConstraints::default(),
        scene_type: scene,
        platform: plat,
    };

    let result = adapter.execute(task).await.map_err(|e| e.to_string())?;

    // Update health and record in registry
    adapter.update_health(&result).await;

    // Record for self-optimization
    {
        let mut registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
        registry.record_execution(
            &result.agent_id,
            crate::agent_adapter::types::SceneType::WebDevelopment,
            crate::agent_adapter::types::Platform::Web,
            result.status == crate::agent_adapter::types::ResultStatus::Success,
            result.duration_ms,
            result.cost.amount,
        );
    }

    Ok(result)
}

/// Execute task with Codex
#[tauri::command]
pub async fn agent_adapter_execute_codex(
    state: State<'_, AppState>,
    task_id: String,
    description: String,
    cwd: Option<String>,
    scene_type: Option<String>,
    platform: Option<String>,
) -> Result<AgentResult, String> {
    let mut adapter = CodexAdapter::new();

    let config = AgentConfig {
        cwd,
        timeout_ms: 300000,
        max_retries: 3,
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;

    let scene = scene_type.map(|s| parse_scene(&s)).transpose()?;
    let plat = platform.map(|p| parse_platform(&p)).transpose()?;

    let desc_clone = description.clone();
    let task = AgentTask {
        id: task_id,
        description,
        input: TaskInput::Text(desc_clone),
        constraints: TaskConstraints::default(),
        scene_type: scene,
        platform: plat,
    };

    let result = adapter.execute(task).await.map_err(|e| e.to_string())?;

    // Update health
    adapter.update_health(&result).await;

    // Record for self-optimization
    {
        let mut registry = state.agent_registry.lock().map_err(|e| e.to_string())?;
        registry.record_execution(
            &result.agent_id,
            crate::agent_adapter::types::SceneType::WebDevelopment,
            crate::agent_adapter::types::Platform::Web,
            result.status == crate::agent_adapter::types::ResultStatus::Success,
            result.duration_ms,
            result.cost.amount,
        );
    }

    Ok(result)
}

/// Get adapter health metrics
#[tauri::command]
pub async fn agent_adapter_get_health(
    state: State<'_, AppState>,
    adapter_id: String,
) -> Result<crate::agent_adapter::types::HealthMetrics, String> {
    if adapter_id.contains("claude") {
        let adapter = ClaudeCodeAdapter::new();
        Ok(adapter.health().await)
    } else if adapter_id.contains("codex") {
        let adapter = CodexAdapter::new();
        Ok(adapter.health().await)
    } else {
        Err(format!("Unknown adapter: {}", adapter_id))
    }
}

/// Get cost estimate for task
#[tauri::command]
pub async fn agent_adapter_estimate_cost(
    state: State<'_, AppState>,
    adapter_id: String,
    description: String,
) -> Result<crate::agent_adapter::types::CostEstimate, String> {
    let desc_clone = description.clone();
    let task = AgentTask {
        id: "estimate".to_string(),
        description,
        input: TaskInput::Text(desc_clone),
        constraints: TaskConstraints::default(),
        scene_type: None,
        platform: None,
    };

    if adapter_id.contains("claude") {
        let adapter = ClaudeCodeAdapter::new();
        Ok(adapter.cost_estimate(&task))
    } else if adapter_id.contains("codex") {
        let adapter = CodexAdapter::new();
        Ok(adapter.cost_estimate(&task))
    } else {
        Err(format!("Unknown adapter: {}", adapter_id))
    }
}

// === Desktop Development Commands (Phase 2) ===

/// Detect desktop framework (Tauri or Electron)
#[tauri::command]
pub async fn desktop_detect_project(
    state: State<'_, AppState>,
    cwd: String,
) -> Result<DesktopDetectionResult, String> {
    let path = std::path::PathBuf::from(&cwd);

    let is_tauri = TauriDesktopAdapter::detect_tauri_project(&path);
    let is_electron = ElectronDesktopAdapter::detect_electron_project(&path);

    let framework = if is_tauri && is_electron {
        "hybrid"
    } else if is_tauri {
        "tauri"
    } else if is_electron {
        "electron"
    } else {
        "unknown"
    };

    let platform = DesktopPlatform::current();

    let recommended_bundles = if is_tauri {
        BundleType::for_platform(platform)
    } else if is_electron {
        // Electron uses different output formats
        vec![]
    } else {
        vec![]
    };

    Ok(DesktopDetectionResult {
        framework: framework.to_string(),
        platform: platform.as_str().to_string(),
        recommended_bundles: recommended_bundles
            .iter()
            .map(|b| b.as_str().to_string())
            .collect(),
        cwd,
    })
}

/// Build Tauri app in dev mode
#[tauri::command]
pub async fn desktop_tauri_dev(state: State<'_, AppState>, cwd: String) -> Result<String, String> {
    let adapter = TauriDesktopAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    adapter.build_dev(&path).await.map_err(|e| e.to_string())
}

/// Build Tauri app in release mode
#[tauri::command]
pub async fn desktop_tauri_build(
    state: State<'_, AppState>,
    cwd: String,
    bundle: Option<String>,
) -> Result<String, String> {
    let adapter = TauriDesktopAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    let bundle_type = bundle.map(|b| match b.to_lowercase().as_str() {
        "msi" => BundleType::Msi,
        "dmg" => BundleType::Dmg,
        "appimage" => BundleType::AppImage,
        "deb" => BundleType::Deb,
        "rpm" => BundleType::Rpm,
        "nsis" => BundleType::Nsis,
        _ => BundleType::None,
    });

    adapter
        .build_release(&path, bundle_type)
        .await
        .map_err(|e| e.to_string())
}

/// Start Electron in dev mode
#[tauri::command]
pub async fn desktop_electron_dev(
    state: State<'_, AppState>,
    cwd: String,
) -> Result<String, String> {
    let adapter = ElectronDesktopAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    adapter.start_dev(&path).await.map_err(|e| e.to_string())
}

/// Build Electron app
#[tauri::command]
pub async fn desktop_electron_build(
    state: State<'_, AppState>,
    cwd: String,
    platform: Option<String>,
    arch: Option<String>,
) -> Result<String, String> {
    let adapter = ElectronDesktopAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    let target_platform = platform
        .map(|p| ElectronPlatform::from_str(&p))
        .transpose()?
        .unwrap_or(ElectronPlatform::current());

    let target_arch = arch
        .map(|a| match a.to_lowercase().as_str() {
            "x64" => ElectronArch::X64,
            "arm64" => ElectronArch::Arm64,
            "universal" => ElectronArch::Universal,
            _ => ElectronArch::X64,
        })
        .unwrap_or(ElectronArch::X64);

    adapter
        .build(&path, target_platform, target_arch)
        .await
        .map_err(|e| e.to_string())
}

/// Get current desktop platform
#[tauri::command]
pub async fn desktop_get_platform() -> Result<String, String> {
    Ok(DesktopPlatform::current().as_str().to_string())
}

/// Get available build targets for platform
#[tauri::command]
pub async fn desktop_get_targets(platform: String) -> Result<Vec<String>, String> {
    let plat = DesktopPlatform::from_str(&platform)?;
    Ok(plat.build_targets())
}

// === Desktop Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopDetectionResult {
    pub framework: String,
    pub platform: String,
    pub recommended_bundles: Vec<String>,
    pub cwd: String,
}

// === Game Development Commands (Phase 3) ===

use crate::agent_adapter::godot_adapter::{GodotAdapter, GodotPlatform};
use crate::agent_adapter::unity_adapter::{UnityAdapter, UnityPlatform};

/// Detect game framework (Unity or Godot)
#[tauri::command]
pub async fn game_detect_framework(
    state: State<'_, AppState>,
    cwd: String,
) -> Result<GameDetectionResult, String> {
    let path = std::path::PathBuf::from(&cwd);

    let is_unity = UnityAdapter::detect_unity_project(&path);
    let is_godot = GodotAdapter::detect_godot_project(&path);

    let framework = if is_unity && is_godot {
        "hybrid"
    } else if is_unity {
        "unity"
    } else if is_godot {
        "godot"
    } else {
        "unknown"
    };

    let scenes = if is_unity {
        UnityAdapter::get_scenes(&path)
    } else if is_godot {
        GodotAdapter::get_scenes(&path)
    } else {
        vec![]
    };

    Ok(GameDetectionResult {
        framework: framework.to_string(),
        scenes,
        cwd,
    })
}

/// Build Unity project (development mode)
#[tauri::command]
pub async fn unity_build_dev(
    state: State<'_, AppState>,
    cwd: String,
    platform: Option<String>,
) -> Result<String, String> {
    let adapter = UnityAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    let target = platform
        .map(|p| UnityPlatform::from_str(&p))
        .transpose()?
        .unwrap_or(UnityPlatform::Windows);

    adapter
        .build(&path, target, true)
        .await
        .map_err(|e| e.to_string())
}

/// Build Unity project (release mode)
#[tauri::command]
pub async fn unity_build_release(
    state: State<'_, AppState>,
    cwd: String,
    platform: Option<String>,
) -> Result<String, String> {
    let adapter = UnityAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    let target = platform
        .map(|p| UnityPlatform::from_str(&p))
        .transpose()?
        .unwrap_or(UnityPlatform::Windows);

    adapter
        .build(&path, target, false)
        .await
        .map_err(|e| e.to_string())
}

/// Get Unity scenes in project
#[tauri::command]
pub async fn unity_get_scenes(cwd: String) -> Result<Vec<String>, String> {
    let path = std::path::PathBuf::from(&cwd);
    Ok(UnityAdapter::get_scenes(&path))
}

/// Export Godot project (development mode)
#[tauri::command]
pub async fn godot_build_dev(
    state: State<'_, AppState>,
    cwd: String,
    platform: Option<String>,
) -> Result<String, String> {
    let adapter = GodotAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    let target = platform
        .map(|p| GodotPlatform::from_str(&p))
        .transpose()?
        .unwrap_or(GodotPlatform::Windows);

    adapter
        .export(&path, target, true)
        .await
        .map_err(|e| e.to_string())
}

/// Export Godot project (release mode)
#[tauri::command]
pub async fn godot_build_release(
    state: State<'_, AppState>,
    cwd: String,
    platform: Option<String>,
) -> Result<String, String> {
    let adapter = GodotAdapter::new();
    let path = std::path::PathBuf::from(&cwd);

    let target = platform
        .map(|p| GodotPlatform::from_str(&p))
        .transpose()?
        .unwrap_or(GodotPlatform::Windows);

    adapter
        .export(&path, target, false)
        .await
        .map_err(|e| e.to_string())
}

/// Get Godot scenes in project
#[tauri::command]
pub async fn godot_get_scenes(cwd: String) -> Result<Vec<String>, String> {
    let path = std::path::PathBuf::from(&cwd);
    Ok(GodotAdapter::get_scenes(&path))
}

/// Get available Unity platforms
#[tauri::command]
pub async fn game_get_platforms() -> Result<Vec<String>, String> {
    Ok(vec![
        "Windows".to_string(),
        "MacOS".to_string(),
        "Linux".to_string(),
        "Android".to_string(),
        "iOS".to_string(),
        "WebGL".to_string(),
    ])
}

// === Game Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDetectionResult {
    pub framework: String,
    pub scenes: Vec<String>,
    pub cwd: String,
}

// === Game Asset Commands (Phase 3 Week 4) ===

use crate::game_assets::{
    AssetOptimization, AssetStats, GameAsset, GameAssetDetector, GameAssetType,
};

/// Detect game assets in project
#[tauri::command]
pub async fn game_detect_assets(cwd: String, framework: String) -> Result<Vec<GameAsset>, String> {
    let detector = GameAssetDetector::new();
    let path = std::path::PathBuf::from(&cwd);
    Ok(detector.detect_assets(&path, &framework))
}

/// Get game asset statistics
#[tauri::command]
pub async fn game_get_asset_stats(cwd: String, framework: String) -> Result<AssetStats, String> {
    let detector = GameAssetDetector::new();
    let path = std::path::PathBuf::from(&cwd);
    let assets = detector.detect_assets(&path, &framework);
    Ok(detector.get_asset_stats(&assets))
}

/// Get optimization suggestions for assets
#[tauri::command]
pub async fn game_get_optimization_suggestions(
    asset_type: String,
    size_bytes: u64,
    format: String,
) -> Result<Vec<AssetOptimization>, String> {
    let detector = GameAssetDetector::new();
    let asset_type_enum = GameAssetType::from_str(&asset_type);
    Ok(detector.get_optimization_suggestions(asset_type_enum, size_bytes, &format))
}

/// Get all supported asset types
#[tauri::command]
pub async fn game_get_asset_types() -> Result<Vec<String>, String> {
    Ok(vec![
        "sprite2d".to_string(),
        "texture".to_string(),
        "model3d".to_string(),
        "animation".to_string(),
        "audio".to_string(),
        "shader".to_string(),
        "level".to_string(),
        "font".to_string(),
        "video".to_string(),
        "script".to_string(),
    ])
}

// === Marketing Commands (Phase 4) ===

use crate::agent_adapter::jimeng_adapter::{ImageSize, JimengAdapter, JimengConfig, JimengStyle};
use crate::agent_adapter::kimi_adapter::{KimiAdapter, KimiConfig, KimiUsage};
use crate::agent_adapter::kling_adapter::{KlingAdapter, KlingConfig, KlingMode};

/// Initialize Kimi adapter with API key
#[tauri::command]
pub async fn marketing_init_kimi(api_key: String, model: Option<String>) -> Result<String, String> {
    let mut adapter = KimiAdapter::new();

    let config = AgentConfig {
        cwd: None,
        timeout_ms: 60000,
        max_retries: 3,
        metadata: HashMap::from([
            ("api_key".to_string(), api_key),
            (
                "model".to_string(),
                model.unwrap_or_else(|| "moonshot-v1-8k".to_string()),
            ),
        ]),
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;
    Ok(adapter.id().to_string())
}

/// Generate text with Kimi
#[tauri::command]
pub async fn kimi_generate_text(
    api_key: String,
    prompt: String,
    system_prompt: Option<String>,
    model: Option<String>,
) -> Result<KimiGenerateResult, String> {
    let adapter = KimiAdapter::with_config(KimiConfig {
        api_key,
        base_url: "https://api.moonshot.cn/v1".to_string(),
        model: model.unwrap_or_else(|| "moonshot-v1-8k".to_string()),
        temperature: 0.7,
        max_tokens: 4096,
    });

    let (content, usage) = adapter
        .generate(&prompt, system_prompt.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(KimiGenerateResult {
        content,
        input_tokens: usage.prompt_tokens,
        output_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
    })
}

/// Translate text with Kimi
#[tauri::command]
pub async fn kimi_translate(
    api_key: String,
    text: String,
    target_language: String,
) -> Result<String, String> {
    let adapter = KimiAdapter::with_config(KimiConfig {
        api_key,
        base_url: "https://api.moonshot.cn/v1".to_string(),
        model: "moonshot-v1-8k".to_string(),
        temperature: 0.3, // Lower temperature for translation
        max_tokens: 4096,
    });

    let system_prompt = format!(
        "你是一个翻译助手，请将文本翻译成{}，保持原文风格。",
        target_language
    );
    let (content, _) = adapter
        .generate(&text, Some(&system_prompt))
        .await
        .map_err(|e| e.to_string())?;

    Ok(content)
}

/// Initialize Jimeng adapter with API key
#[tauri::command]
pub async fn marketing_init_jimeng(
    api_key: String,
    default_style: Option<String>,
) -> Result<String, String> {
    let mut adapter = JimengAdapter::new();

    let config = AgentConfig {
        cwd: None,
        timeout_ms: 60000,
        max_retries: 3,
        metadata: HashMap::from([
            ("api_key".to_string(), api_key),
            (
                "style".to_string(),
                default_style.unwrap_or_else(|| "realistic".to_string()),
            ),
        ]),
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;
    Ok(adapter.id().to_string())
}

/// Generate image with Jimeng
#[tauri::command]
pub async fn jimeng_generate_image(
    api_key: String,
    prompt: String,
    style: Option<String>,
    size: Option<String>,
) -> Result<JimengGenerateResult, String> {
    let jimeng_style = style
        .as_ref()
        .map(|s| JimengStyle::from_str(s))
        .unwrap_or(JimengStyle::Realistic);
    let jimeng_size = size
        .as_ref()
        .map(|s| match s.as_str() {
            "512" => ImageSize::Square512,
            "portrait" => ImageSize::Portrait768x1024,
            "landscape" => ImageSize::Landscape1024x768,
            "wide" => ImageSize::Wide1920x1080,
            _ => ImageSize::Square1024,
        })
        .unwrap_or(ImageSize::Square1024);

    let adapter = JimengAdapter::with_config(crate::agent_adapter::jimeng_adapter::JimengConfig {
        api_key,
        base_url: "https://api.volcengine.com/api/jimeng".to_string(),
        default_style: jimeng_style,
        default_size: jimeng_size,
    });

    let (urls, cost) = adapter
        .generate_image(&prompt, None, jimeng_style, jimeng_size)
        .await
        .map_err(|e| e.to_string())?;

    Ok(JimengGenerateResult {
        image_urls: urls,
        cost,
        style: jimeng_style.as_str().to_string(),
    })
}

/// Initialize Kling adapter with API key
#[tauri::command]
pub async fn marketing_init_kling(
    api_key: String,
    default_duration: Option<u32>,
) -> Result<String, String> {
    let mut adapter = KlingAdapter::new();

    let config = AgentConfig {
        cwd: None,
        timeout_ms: 300000, // 5 minutes for video
        max_retries: 3,
        metadata: HashMap::from([
            ("api_key".to_string(), api_key),
            (
                "duration".to_string(),
                default_duration.unwrap_or(5).to_string(),
            ),
        ]),
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;
    Ok(adapter.id().to_string())
}

/// Generate video with Kling (async)
#[tauri::command]
pub async fn kling_generate_video(
    api_key: String,
    prompt: String,
    mode: Option<String>,
) -> Result<KlingGenerateResult, String> {
    let adapter = KlingAdapter::with_config(crate::agent_adapter::kling_adapter::KlingConfig {
        api_key,
        base_url: "https://api.klingai.com/v1".to_string(),
        default_duration: 5,
        default_resolution: crate::agent_adapter::kling_adapter::VideoResolution::HD1080,
    });

    let kling_mode = mode
        .map(|m| match m.to_lowercase().as_str() {
            "avatar" => KlingMode::Avatar,
            "extend" => KlingMode::Extend,
            "image" => KlingMode::ImageToVideo,
            _ => KlingMode::TextToVideo,
        })
        .unwrap_or(KlingMode::TextToVideo);

    // Create job
    let job_response = adapter
        .create_video_job(&prompt, kling_mode, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(KlingGenerateResult {
        task_id: job_response.task_id,
        estimated_duration: job_response.estimated_duration,
        cost: job_response.cost,
        mode: kling_mode.to_string(),
    })
}

/// Get Kling video job status
#[tauri::command]
pub async fn kling_get_job_status(
    api_key: String,
    task_id: String,
) -> Result<KlingJobStatusResult, String> {
    let adapter = KlingAdapter::with_config(crate::agent_adapter::kling_adapter::KlingConfig {
        api_key,
        base_url: "https://api.klingai.com/v1".to_string(),
        default_duration: 5,
        default_resolution: crate::agent_adapter::kling_adapter::VideoResolution::HD1080,
    });

    let result = adapter
        .get_job_status(&task_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(KlingJobStatusResult {
        task_id: result.task_id,
        status: match result.status {
            crate::agent_adapter::kling_adapter::KlingTaskStatus::Pending => "pending",
            crate::agent_adapter::kling_adapter::KlingTaskStatus::Processing => "processing",
            crate::agent_adapter::kling_adapter::KlingTaskStatus::Completed => "completed",
            crate::agent_adapter::kling_adapter::KlingTaskStatus::Failed => "failed",
            crate::agent_adapter::kling_adapter::KlingTaskStatus::Cancelled => "cancelled",
        }
        .to_string(),
        video_url: result.video_url,
        error: result.error,
    })
}

// === Marketing Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KimiGenerateResult {
    pub content: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JimengGenerateResult {
    pub image_urls: Vec<String>,
    pub cost: f32,
    pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlingGenerateResult {
    pub task_id: String,
    pub estimated_duration: u32,
    pub cost: f32,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlingJobStatusResult {
    pub task_id: String,
    pub status: String,
    pub video_url: Option<String>,
    pub error: Option<String>,
}

// === Office Commands (Phase 4 Week 3) ===

use crate::agent_adapter::wps_adapter::{DocumentFormat, DocumentType, WPSAdapter};

/// Initialize WPS adapter
#[tauri::command]
pub async fn office_init_wps(
    api_key: String,
    default_format: Option<String>,
) -> Result<String, String> {
    let mut adapter = WPSAdapter::new();

    let config = crate::agent_adapter::types::AgentConfig {
        cwd: None,
        timeout_ms: 60000,
        max_retries: 3,
        metadata: HashMap::from([
            ("api_key".to_string(), api_key),
            (
                "format".to_string(),
                default_format.unwrap_or_else(|| "docx".to_string()),
            ),
        ]),
        ..Default::default()
    };

    adapter.configure(config).await.map_err(|e| e.to_string())?;
    Ok(adapter.id().to_string())
}

/// Generate document with WPS
#[tauri::command]
pub async fn wps_generate_doc(
    api_key: String,
    prompt: String,
    document_type: Option<String>,
    format: Option<String>,
) -> Result<WPSGenerateResult, String> {
    let adapter = WPSAdapter::with_config(crate::agent_adapter::wps_adapter::WPSConfig {
        api_key,
        base_url: "https://api.wps.cn/v1".to_string(),
        default_format: format
            .map(|f| DocumentFormat::from_str(&f))
            .unwrap_or(DocumentFormat::Docx),
    });

    let doc_type = document_type
        .map(|d| match d.to_lowercase().as_str() {
            "report" | "报告" => DocumentType::Report,
            "contract" | "合同" => DocumentType::Contract,
            "proposal" | "提案" => DocumentType::Proposal,
            "resume" | "简历" => DocumentType::Resume,
            "summary" | "总结" => DocumentType::Summary,
            "plan" | "计划" => DocumentType::Plan,
            _ => DocumentType::Article,
        })
        .unwrap_or(DocumentType::Article);

    let (url, cost) = adapter
        .generate_document(&prompt, doc_type, DocumentFormat::Docx)
        .await
        .map_err(|e| e.to_string())?;

    Ok(WPSGenerateResult {
        document_url: url,
        cost,
        document_type: doc_type.as_str().to_string(),
        format: "docx".to_string(),
    })
}

/// Analyze Excel with WPS
#[tauri::command]
pub async fn wps_analyze_excel(
    api_key: String,
    file_url: String,
) -> Result<ExcelAnalysisResult, String> {
    let adapter = WPSAdapter::with_config(crate::agent_adapter::wps_adapter::WPSConfig {
        api_key,
        base_url: "https://api.wps.cn/v1".to_string(),
        default_format: DocumentFormat::Xlsx,
    });

    let (summary, charts, insights, cost) = adapter
        .analyze_excel(&file_url, "comprehensive")
        .await
        .map_err(|e| e.to_string())?;

    Ok(ExcelAnalysisResult {
        summary,
        charts,
        insights,
        cost,
    })
}

// === Office Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WPSGenerateResult {
    pub document_url: String,
    pub cost: f32,
    pub document_type: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelAnalysisResult {
    pub summary: String,
    pub charts: Vec<String>,
    pub insights: Vec<String>,
    pub cost: f32,
}

// === Budget Commands (Phase 4 Week 4) ===

use crate::cost_tracker::{CostAlert, CostBudget, CostForecast, CostTracker, CostUsage};

/// Set budget limits
#[tauri::command]
pub async fn budget_set_limit(
    state: State<'_, AppState>,
    daily_limit: f32,
    monthly_limit: f32,
    hard_stop: Option<bool>,
) -> Result<(), String> {
    let budget = CostBudget {
        daily_limit,
        monthly_limit,
        currency: "USD".to_string(),
        hard_stop: hard_stop.unwrap_or(false),
        enable_alerts: true,
    };

    // In real implementation, we'd store CostTracker in AppState
    Ok(())
}

/// Get current budget usage
#[tauri::command]
pub async fn budget_get_usage() -> Result<CostUsage, String> {
    // In real implementation, we'd get from stored CostTracker
    Ok(CostUsage {
        daily_total: 0.0,
        monthly_total: 0.0,
        total_all_time: 0.0,
        adapter_totals: HashMap::new(),
        scene_totals: HashMap::new(),
        token_totals: crate::cost_tracker::TokenTotals::default(),
    })
}

/// Get cost forecast
#[tauri::command]
pub async fn budget_get_forecast() -> Result<CostForecast, String> {
    // In real implementation, we'd calculate from tracker
    Ok(CostForecast {
        predicted_daily: 0.0,
        predicted_monthly: 0.0,
        days_remaining: 30,
        budget_remaining_percent: 100.0,
        will_exceed: false,
    })
}

/// Check if budget allows execution
#[tauri::command]
pub async fn budget_is_allowed() -> Result<bool, String> {
    Ok(true)
}

// === One-Shot Interface Commands (Phase 5) ===

use crate::one_shot::{OneShotInterface, OneShotRequest, OneShotResponse, UserRole};

/// One-shot unified entry point
#[tauri::command]
pub async fn one_shot_execute(
    input: String,
    context_hint: Option<String>,
    preferred_agent: Option<String>,
    max_cost: Option<f32>,
    timeout_ms: Option<u64>,
) -> Result<OneShotResponse, String> {
    let interface = OneShotInterface::new();
    let request = OneShotRequest {
        input,
        context_hint,
        preferred_agent,
        max_cost,
        timeout_ms,
    };

    Ok(interface.process(request))
}

/// Detect user role from input
#[tauri::command]
pub async fn one_shot_detect_role(input: String) -> Result<UserRoleResult, String> {
    let detector = crate::one_shot::UserRoleDetector::new();
    let role = detector.detect(&input);
    let confidence = detector.confidence(&input, role);

    Ok(UserRoleResult {
        role: role.as_str().to_string(),
        confidence,
    })
}

/// Detect scene from input
#[tauri::command]
pub async fn one_shot_detect_scene(input: String) -> Result<String, String> {
    let detector = crate::one_shot::SceneDetector::new();
    Ok(detector.detect(&input))
}

/// Get recommended agent for role and scene
#[tauri::command]
pub async fn one_shot_get_recommendation(
    role: String,
    scene: String,
) -> Result<AgentRecommendation, String> {
    let selector = crate::one_shot::AgentSelector::new();
    let user_role = crate::one_shot::UserRole::from_str(&role);
    let (agent, reason) = selector.select(user_role, &scene);

    Ok(AgentRecommendation {
        agent,
        reason,
        alternatives: vec![], // Would include other viable agents
    })
}

// === One-Shot Types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleResult {
    pub role: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecommendation {
    pub agent: String,
    pub reason: String,
    pub alternatives: Vec<String>,
}
