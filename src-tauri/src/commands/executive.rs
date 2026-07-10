use crate::database;
use crate::executive_agent::{ExecutiveAgentManager, GeneratedFile, TaskResult};
use crate::AppState;
use std::collections::HashMap;
use tauri::{AppHandle, State};

// ===== Executive Agent Commands =====

/// Initialize Executive Agent Manager with workspace path
#[tauri::command]
pub fn init_executive_agent(workspace: String, state: State<AppState>) -> Result<(), String> {
    let workspace_path = std::path::PathBuf::from(&workspace);

    // Create workspace directory if not exists
    if !workspace_path.exists() {
        std::fs::create_dir_all(&workspace_path).map_err(|e| format!("创建工作目录失败: {}", e))?;
    }

    let manager = ExecutiveAgentManager::new(workspace_path);
    *state.executive_agent_manager.lock().unwrap() = Some(manager);

    Ok(())
}

/// Execute development task using multi-agent workflow
#[tauri::command]
pub async fn execute_development_task(
    request: String,
    mode: Option<String>,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<TaskResult, String> {
    // Get workspace path from manager (quick operation, no await)
    let workspace_path = {
        let manager_guard = state.executive_agent_manager.lock().unwrap();
        let manager = manager_guard.as_ref().ok_or_else(|| {
            "Executive Agent Manager 未初始化，请先调用 init_executive_agent".to_string()
        })?;

        // Clone workspace path only
        manager.workspace.clone()
    };
    // Lock is released here

    // Create a new manager for this execution (ownership, no lock needed)
    let manager = ExecutiveAgentManager::new(workspace_path);

    // Execute workflow (async operation with no locks held)
    let result = if let Some(mode_str) = mode {
        manager
            .execute_workflow_with_mode(request, mode_str, app_handle)
            .await?
    } else {
        manager.execute_workflow(request, app_handle).await?
    };

    // Update state with the manager that has results
    {
        let mut mgr = state.executive_agent_manager.lock().unwrap();
        *mgr = Some(manager);
    }

    Ok(result)
}

/// Get executive agent status
#[tauri::command]
pub fn get_executive_agent_status(
    state: State<AppState>,
) -> Result<HashMap<String, String>, String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Executive Agent Manager 未初始化".to_string())?;

    let status = manager.get_agent_status();
    Ok(status
        .into_iter()
        .map(|(k, v)| (format!("{:?}", k), format!("{:?}", v)))
        .collect())
}

/// Get task result with generated files
#[tauri::command]
pub fn get_task_result(state: State<AppState>) -> Result<Vec<GeneratedFile>, String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Executive Agent Manager 未初始化".to_string())?;

    Ok(manager.get_generated_files())
}

/// Get all generated files
#[tauri::command]
pub fn get_generated_files(state: State<AppState>) -> Result<Vec<GeneratedFile>, String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    let manager = manager
        .as_ref()
        .ok_or_else(|| "Executive Agent Manager 未初始化".to_string())?;

    Ok(manager.get_generated_files())
}

/// Clear executive agent state
#[tauri::command]
pub fn clear_executive_agent(state: State<AppState>) -> Result<(), String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    if let Some(manager) = manager.as_ref() {
        manager.clear();
    }
    Ok(())
}

// ===== Executive Session Database Commands =====

/// Load executive sessions from database
#[tauri::command]
pub fn load_executive_sessions(
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<database::ExecutiveSessionRecord>, String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.load_executive_sessions(limit)
}

/// Get single executive session by ID
#[tauri::command]
pub fn get_executive_session_by_id(
    id: String,
    state: State<AppState>,
) -> Result<Option<database::ExecutiveSessionRecord>, String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.get_executive_session(&id)
}

/// Delete executive session from database
#[tauri::command]
pub fn delete_executive_session_by_id(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.delete_executive_session(&id)
}

/// Get executive session statistics
#[tauri::command]
pub fn get_executive_session_stats(
    state: State<AppState>,
) -> Result<database::ExecutiveSessionStats, String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.get_executive_session_stats()
}

/// Save executive session record to database
#[tauri::command]
pub fn save_executive_session_record(
    session: database::ExecutiveSessionRecord,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.save_executive_session(&session)
}
