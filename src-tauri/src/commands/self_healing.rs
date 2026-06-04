use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// ===== Error/Solution Records (Self-Healing System) =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRecord {
    pub id: String,
    pub category: String,
    pub message: String,
    pub context: Option<String>,
    pub stack_trace: Option<String>,
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
    pub status: String,
    pub solution_id: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionRecord {
    pub id: String,
    pub error_id: Option<String>,
    pub approach: String,
    pub steps: Option<String>,
    pub result: String,
    pub success: Option<bool>,
    pub evidence: Option<String>,
    pub created_at: String,
}

// ===== Error Commands =====

#[tauri::command]
pub fn save_error(
    category: String,
    message: String,
    context: Option<String>,
    stack_trace: Option<String>,
    agent_id: Option<String>,
    task_id: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO errors (id, category, message, context, stack_trace, agent_id, task_id, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'open', ?8)",
        rusqlite::params![id, category, message, context, stack_trace, agent_id, task_id, now],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub fn get_errors(
    status: Option<String>,
    category: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<ErrorRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let sql = match (&status, &category) {
        (Some(_s), Some(_c)) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors WHERE status = ?1 AND category = ?2 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (Some(_s), None) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors WHERE status = ?1 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (None, Some(_c)) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors WHERE category = ?1 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (None, None) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors ORDER BY created_at DESC {}",
            limit_clause
        ),
    };

    let records: Vec<ErrorRecord> = match (&status, &category) {
        (Some(s), Some(c)) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s, c], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (Some(s), None) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, Some(c)) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![c], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, None) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    };

    Ok(records)
}

#[tauri::command]
pub fn resolve_error(
    error_id: String,
    solution_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE errors SET status = 'resolved', solution_id = ?1, resolved_at = ?2 WHERE id = ?3",
        rusqlite::params![solution_id, now, error_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// ===== Solution Commands =====

#[tauri::command]
pub fn save_solution(
    error_id: Option<String>,
    approach: String,
    steps: Option<String>,
    result: String,
    success: Option<bool>,
    evidence: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO solutions (id, error_id, approach, steps, result, success, evidence, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, error_id, approach, steps, result, success, evidence, now],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub fn get_solutions(
    error_id: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<SolutionRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let sql = if let Some(_eid) = &error_id {
        format!(
            "SELECT id, error_id, approach, steps, result, success, evidence, created_at
             FROM solutions WHERE error_id = ?1 ORDER BY created_at DESC {}",
            limit_clause
        )
    } else {
        format!(
            "SELECT id, error_id, approach, steps, result, success, evidence, created_at
             FROM solutions ORDER BY created_at DESC {}",
            limit_clause
        )
    };

    let records: Vec<SolutionRecord> = if let Some(eid) = &error_id {
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![eid], |row| {
                Ok(SolutionRecord {
                    id: row.get(0)?,
                    error_id: row.get(1)?,
                    approach: row.get(2)?,
                    steps: row.get(3)?,
                    result: row.get(4)?,
                    success: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(SolutionRecord {
                    id: row.get(0)?,
                    error_id: row.get(1)?,
                    approach: row.get(2)?,
                    steps: row.get(3)?,
                    result: row.get(4)?,
                    success: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(records)
}
