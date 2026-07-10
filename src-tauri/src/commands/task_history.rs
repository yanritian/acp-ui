use crate::database;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_db_path(state: State<AppState>) -> Result<String, String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    db.as_ref()
        .map(|d| d.get_db_path().to_string_lossy().to_string())
        .ok_or_else(|| "Database not initialized".to_string())
}

#[tauri::command]
pub fn get_task_history(
    status: Option<Vec<String>>,
    source: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<database::TaskRecord>, String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let status_filter = status.map(|s| {
        s.iter()
            .filter_map(|st| database::TaskStatus::from_str(st).ok())
            .collect()
    });

    db.load_tasks(status_filter, source, limit, None)
}

#[tauri::command]
pub fn get_task_statistics(state: State<AppState>) -> Result<database::TaskStatistics, String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.get_statistics()
}

#[tauri::command]
pub fn delete_task_history(task_id: String, state: State<AppState>) -> Result<(), String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.delete_task(&task_id)
}

#[tauri::command]
pub fn search_tasks(
    keyword: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<database::TaskRecord>, String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.search_tasks(&keyword, limit)
}

#[tauri::command]
pub fn get_task_detail(
    task_id: String,
    state: State<AppState>,
) -> Result<Option<database::TaskRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.get_task(&task_id)
}

#[tauri::command]
pub fn save_task_history(task: database::TaskRecord, state: State<AppState>) -> Result<(), String> {
    let db = state
        .database
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    db.save_task(&task)
}
