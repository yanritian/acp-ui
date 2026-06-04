use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecord {
    pub id: String,
    pub scope: String,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub memory_type: String,
    pub content: String,
    pub tags: Option<String>,
    pub importance: f64,
    pub created_at: String,
    pub last_accessed: Option<String>,
    pub access_count: i64,
    pub expires_at: Option<String>,
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn save_memory(
    content: String,
    tags: Option<String>,
    scope: Option<String>,
    memory_type: Option<String>,
    agent_id: Option<String>,
    session_id: Option<String>,
    task_id: Option<String>,
    importance: Option<f64>,
    expires_at: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let scope_val = scope.unwrap_or_else(|| "global".to_string());
    let type_val = memory_type.unwrap_or_else(|| "fact".to_string());
    let importance_val = importance.unwrap_or(0.5);

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO memories (id, scope, agent_id, session_id, task_id, type, content, tags, importance, created_at, access_count, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11)",
        rusqlite::params![id, scope_val, agent_id, session_id, task_id, type_val, content, tags, importance_val, now, expires_at],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub fn search_memories(
    query: String,
    agent_id: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<MemoryRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let pattern = format!("%{}%", query);

    // Helper function to map row to MemoryRecord
    fn map_row(row: &rusqlite::Row) -> Result<MemoryRecord, rusqlite::Error> {
        Ok(MemoryRecord {
            id: row.get(0)?,
            scope: row.get(1)?,
            agent_id: row.get(2)?,
            session_id: row.get(3)?,
            task_id: row.get(4)?,
            memory_type: row.get(5)?,
            content: row.get(6)?,
            tags: row.get(7)?,
            importance: row.get(8)?,
            created_at: row.get(9)?,
            last_accessed: row.get(10)?,
            access_count: row.get(11)?,
            expires_at: row.get(12)?,
        })
    }

    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let records: Vec<MemoryRecord> = if let Some(aid) = &agent_id {
        let sql = format!(
            "SELECT id, scope, agent_id, session_id, task_id, type, content, tags, importance, created_at, last_accessed, access_count, expires_at
             FROM memories WHERE content LIKE ?1 AND (agent_id = ?2 OR agent_id IS NULL)
             ORDER BY importance DESC, created_at DESC {}",
            limit_clause
        );
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![&pattern, aid], map_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        let sql = format!(
            "SELECT id, scope, agent_id, session_id, task_id, type, content, tags, importance, created_at, last_accessed, access_count, expires_at
             FROM memories WHERE content LIKE ?1
             ORDER BY importance DESC, created_at DESC {}",
            limit_clause
        );
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![&pattern], map_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(records)
}

#[tauri::command]
pub fn get_agent_memories(
    agent_id: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<MemoryRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();

    let sql = format!(
        "SELECT id, scope, agent_id, session_id, task_id, type, content, tags, importance, created_at, last_accessed, access_count, expires_at
         FROM memories WHERE agent_id = ?1 OR agent_id IS NULL
         ORDER BY importance DESC, created_at DESC {}",
        limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default()
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let records: Vec<MemoryRecord> = stmt
        .query_map(rusqlite::params![agent_id], |row| {
            Ok(MemoryRecord {
                id: row.get(0)?,
                scope: row.get(1)?,
                agent_id: row.get(2)?,
                session_id: row.get(3)?,
                task_id: row.get(4)?,
                memory_type: row.get(5)?,
                content: row.get(6)?,
                tags: row.get(7)?,
                importance: row.get(8)?,
                created_at: row.get(9)?,
                last_accessed: row.get(10)?,
                access_count: row.get(11)?,
                expires_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(records)
}

#[tauri::command]
pub fn get_shared_memories(
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<MemoryRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let sql = format!(
        "SELECT id, scope, agent_id, session_id, task_id, type, content, tags, importance, created_at, last_accessed, access_count, expires_at
         FROM memories WHERE agent_id IS NULL
         ORDER BY importance DESC, created_at DESC {}",
        limit_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let records: Vec<MemoryRecord> = stmt
        .query_map([], |row| {
            Ok(MemoryRecord {
                id: row.get(0)?,
                scope: row.get(1)?,
                agent_id: row.get(2)?,
                session_id: row.get(3)?,
                task_id: row.get(4)?,
                memory_type: row.get(5)?,
                content: row.get(6)?,
                tags: row.get(7)?,
                importance: row.get(8)?,
                created_at: row.get(9)?,
                last_accessed: row.get(10)?,
                access_count: row.get(11)?,
                expires_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(records)
}

#[tauri::command]
pub fn delete_memory(
    memory_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM memories WHERE id = ?1",
        rusqlite::params![memory_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}
