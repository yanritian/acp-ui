use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// ===== Evolution/Pattern Records (Self-Evolution System) =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionRecord {
    pub id: String,
    pub evolution_type: String,
    pub domain: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub reason: String,
    pub evidence: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub examples: Option<String>,
    pub success_rate: Option<f64>,
    pub usage_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

// ===== Evolution Commands =====

#[tauri::command]
pub fn save_evolution(
    evolution_type: String,
    domain: String,
    before: Option<String>,
    after: Option<String>,
    reason: String,
    evidence: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO evolutions (id, type, domain, before, after, reason, evidence, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, evolution_type, domain, before, after, reason, evidence, now],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub fn get_evolutions(
    evolution_type: Option<String>,
    domain: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<EvolutionRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let sql = match (&evolution_type, &domain) {
        (Some(_t), Some(_d)) => format!(
            "SELECT id, type, domain, before, after, reason, evidence, created_at
             FROM evolutions WHERE type = ?1 AND domain = ?2 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (Some(_t), None) => format!(
            "SELECT id, type, domain, before, after, reason, evidence, created_at
             FROM evolutions WHERE type = ?1 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (None, Some(_d)) => format!(
            "SELECT id, type, domain, before, after, reason, evidence, created_at
             FROM evolutions WHERE domain = ?1 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (None, None) => format!(
            "SELECT id, type, domain, before, after, reason, evidence, created_at
             FROM evolutions ORDER BY created_at DESC {}",
            limit_clause
        ),
    };

    let records: Vec<EvolutionRecord> = match (&evolution_type, &domain) {
        (Some(t), Some(d)) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![t, d], |row| {
                Ok(EvolutionRecord {
                    id: row.get(0)?,
                    evolution_type: row.get(1)?,
                    domain: row.get(2)?,
                    before: row.get(3)?,
                    after: row.get(4)?,
                    reason: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (Some(t), None) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![t], |row| {
                Ok(EvolutionRecord {
                    id: row.get(0)?,
                    evolution_type: row.get(1)?,
                    domain: row.get(2)?,
                    before: row.get(3)?,
                    after: row.get(4)?,
                    reason: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, Some(d)) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![d], |row| {
                Ok(EvolutionRecord {
                    id: row.get(0)?,
                    evolution_type: row.get(1)?,
                    domain: row.get(2)?,
                    before: row.get(3)?,
                    after: row.get(4)?,
                    reason: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, None) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(EvolutionRecord {
                    id: row.get(0)?,
                    evolution_type: row.get(1)?,
                    domain: row.get(2)?,
                    before: row.get(3)?,
                    after: row.get(4)?,
                    reason: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    };

    Ok(records)
}

// ===== Pattern Commands =====

#[tauri::command]
pub fn save_pattern(
    name: String,
    description: String,
    category: String,
    examples: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO patterns (id, name, description, category, examples, usage_count, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        rusqlite::params![id, name, description, category, examples, now, now],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub fn get_patterns(
    category: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<PatternRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let sql = if let Some(_cat) = &category {
        format!(
            "SELECT id, name, description, category, examples, success_rate, usage_count, created_at, updated_at
             FROM patterns WHERE category = ?1 ORDER BY usage_count DESC {}",
            limit_clause
        )
    } else {
        format!(
            "SELECT id, name, description, category, examples, success_rate, usage_count, created_at, updated_at
             FROM patterns ORDER BY usage_count DESC {}",
            limit_clause
        )
    };

    let records: Vec<PatternRecord> = if let Some(cat) = &category {
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![cat], |row| {
                Ok(PatternRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    examples: row.get(4)?,
                    success_rate: row.get(5)?,
                    usage_count: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(PatternRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    examples: row.get(4)?,
                    success_rate: row.get(5)?,
                    usage_count: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(records)
}

#[tauri::command]
pub fn update_pattern_usage(
    pattern_id: String,
    success: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.conn.lock().unwrap();

    // Increment usage count
    conn.execute(
        "UPDATE patterns SET usage_count = usage_count + 1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, pattern_id],
    ).map_err(|e| e.to_string())?;

    // Update success rate
    if success {
        conn.execute(
            "UPDATE patterns SET success_rate = (success_rate * usage_count + 1.0) / (usage_count + 1) WHERE id = ?1",
            rusqlite::params![pattern_id],
        ).map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE patterns SET success_rate = (success_rate * usage_count) / (usage_count + 1) WHERE id = ?1",
            rusqlite::params![pattern_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}
