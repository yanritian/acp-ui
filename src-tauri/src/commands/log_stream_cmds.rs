use crate::AppState;
use crate::log_stream::{LogEntry, LogType};
use tauri::State;

// ===== LogStream Commands =====

#[tauri::command]
pub fn subscribe_logs(agent_id: String, state: State<AppState>) -> Result<(), String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    manager.subscribe(&agent_id);
    Ok(())
}

#[tauri::command]
pub fn unsubscribe_logs(agent_id: String, state: State<AppState>) -> Result<(), String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    manager.unsubscribe(&agent_id);
    Ok(())
}

#[tauri::command]
pub fn get_logs(
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;

    let count = limit.unwrap_or(100) as usize;
    Ok(manager.get_latest_logs(count))
}

#[tauri::command]
pub fn search_logs(
    keyword: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    Ok(manager.search_logs(&keyword, limit))
}

#[tauri::command]
pub fn get_logs_by_agent(
    agent_id: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    Ok(manager.get_logs_from_db(Some(&agent_id), None, limit))
}

#[tauri::command]
pub fn get_logs_by_type(
    log_type: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    let lt = LogType::from_str(&log_type);
    Ok(manager.get_logs_from_db(None, Some(lt), limit))
}

#[tauri::command]
pub fn clear_log_buffer(state: State<AppState>) -> Result<(), String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    manager.clear_buffer();
    Ok(())
}

#[tauri::command]
pub fn flush_log_batch(state: State<AppState>) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    Ok(manager.flush_batch())
}
