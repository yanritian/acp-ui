// Sync Engine - Multi-platform data synchronization (Phase 3)
// App / Tauri / Web data real-time sync with Local-First strategy
// SQLite-backed persistence (survives restarts)

// Debug prints are intentional for development phase
#![allow(clippy::print_stdout)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension};

// ---------------------------------------------------------------------------
// Sync Data Types
// ---------------------------------------------------------------------------

/// Syncable data entity types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SyncEntityType {
    AgentConfig,
    TaskHistory,
    SkillVersion,
    EvolutionSuggestion,
    HealingRecord,
    UserPreference,
    SessionData,
    FlowContext,
}

/// Sync entity - a unit of data that can be synchronized
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEntity {
    pub id: String,
    pub entity_type: SyncEntityType,
    pub data: serde_json::Value,
    pub version: u64,
    pub updated_at: DateTime<Utc>,
    pub source: SyncSource,
    pub checksum: String,
}

/// Source of the sync entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyncSource {
    TauriDesktop,
    FlutterApp,
    WebBrowser,
}

/// Sync event for broadcasting changes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEvent {
    pub event_type: SyncEventType,
    pub entity: SyncEntity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncEventType {
    Created,
    Updated,
    Deleted,
}

/// Sync conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflictResolution {
    pub entity_id: String,
    pub winning_version: u64,
    pub winning_source: SyncSource,
    pub resolution_strategy: ConflictStrategy,
    pub merged_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    LastWriteWins,
    SourcePriority,
    MergeFields,
    ManualResolve,
}

// ---------------------------------------------------------------------------
// Sync Configuration
// ---------------------------------------------------------------------------

/// Sync engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfig {
    /// Enable sync broadcasting
    pub broadcast_enabled: bool,
    /// Maximum entities to keep in memory cache
    pub cache_size: usize,
    /// Sync interval in milliseconds (for periodic sync)
    pub sync_interval_ms: u64,
    /// Offline cache retention duration in hours
    pub offline_retention_hours: u64,
    /// Conflict resolution strategy
    pub default_conflict_strategy: ConflictStrategy,
    /// Source priority for conflict resolution (higher index = higher priority)
    pub source_priority: Vec<SyncSource>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            broadcast_enabled: true,
            cache_size: 1000,
            sync_interval_ms: 5000, // 5 seconds
            offline_retention_hours: 24,
            default_conflict_strategy: ConflictStrategy::LastWriteWins,
            source_priority: vec![
                SyncSource::TauriDesktop, // Highest priority
                SyncSource::FlutterApp,
                SyncSource::WebBrowser,
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Sync Engine
// ---------------------------------------------------------------------------

/// Sync Engine - manages multi-platform data synchronization
/// Backed by SQLite for persistence across restarts.
pub struct SyncEngine {
    /// Configuration
    #[allow(dead_code)]
    config: SyncConfig,
    /// SQLite connection
    db: Arc<Mutex<Connection>>,
    /// Current sync source (where this engine is running)
    current_source: SyncSource,
}

impl SyncEngine {
    /// Create a new sync engine with a default SQLite database path.
    ///
    /// Database is stored in the OS data directory under `acp-ui/sync.db`.
    /// Returns an error if the database cannot be opened or initialized.
    pub fn new(source: SyncSource) -> Result<Self, String> {
        let db_path = default_db_path();
        Self::with_db_path(source, db_path)
    }

    /// Create a new sync engine with an explicit database path.
    pub fn with_db_path(source: SyncSource, db_path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create DB directory: {}", e))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open SQLite DB at {}: {}", db_path.display(), e))?;

        let engine = Self {
            config: SyncConfig::default(),
            db: Arc::new(Mutex::new(conn)),
            current_source: source,
        };

        engine.init_schema()?;
        Ok(engine)
    }

    /// Initialize database schema (idempotent).
    fn init_schema(&self) -> Result<(), String> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(|e| e.to_string())?;

        tx.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sync_entities (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                data TEXT NOT NULL,
                version INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                source TEXT NOT NULL,
                checksum TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_versions (
                type_key TEXT PRIMARY KEY,
                version INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_offline_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_data TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| format!("Failed to create schema: {}", e))?;

        tx.commit().map_err(|e| e.to_string())
    }

    /// Put an entity into sync (create or update)
    pub fn put_entity(
        &self,
        entity_type: SyncEntityType,
        id: String,
        data: serde_json::Value,
    ) -> Result<SyncEntity, String> {
        let type_key = serde_json::to_string(&entity_type).unwrap_or_default();

        let checksum = calculate_checksum(&data);
        let now = Utc::now();

        let entity = SyncEntity {
            id: id.clone(),
            entity_type: entity_type.clone(),
            data,
            version: 0, // Will be set to actual version in the database transaction
            updated_at: now,
            source: self.current_source.clone(),
            checksum,
        };

        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(|e| e.to_string())?;

        // Increment version
        let current: Option<i64> = tx
            .query_row(
                "SELECT version FROM sync_versions WHERE type_key = ?",
                [&type_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        let new_version = current.unwrap_or(0) + 1;
        tx.execute(
            "INSERT INTO sync_versions (type_key, version) VALUES (?, ?)
             ON CONFLICT(type_key) DO UPDATE SET version = excluded.version",
            (&type_key, new_version),
        )
        .map_err(|e| e.to_string())?;

        // Upsert entity
        tx.execute(
            "INSERT INTO sync_entities (id, entity_type, data, version, updated_at, source, checksum)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 entity_type = excluded.entity_type,
                 data = excluded.data,
                 version = excluded.version,
                 updated_at = excluded.updated_at,
                 source = excluded.source,
                 checksum = excluded.checksum",
            (
                &id,
                type_key,
                serde_json::to_string(&entity.data).unwrap_or_default(),
                new_version,
                now.to_rfc3339(),
                serialize_source(&self.current_source),
                &entity.checksum,
            ),
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        println!(
            "[SyncEngine] Put entity '{}' type '{}' version {}",
            id,
            serde_json::to_string(&entity_type).unwrap_or_default(),
            new_version
        );

        Ok(SyncEntity {
            version: new_version as u64,
            ..entity
        })
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: &str) -> Option<SyncEntity> {
        let db = self.db.lock().unwrap();
        query_entity(&db, id).ok().flatten()
    }

    /// Delete an entity
    pub fn delete_entity(&self, id: &str) -> Result<(), String> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(|e| e.to_string())?;

        let changed = tx
            .execute("DELETE FROM sync_entities WHERE id = ?", [id])
            .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        if changed > 0 {
            println!("[SyncEngine] Deleted entity '{}'", id);
            Ok(())
        } else {
            Err(format!("Entity '{}' not found", id))
        }
    }

    /// List entities by type
    pub fn list_entities(&self, entity_type: SyncEntityType) -> Vec<SyncEntity> {
        let type_key = serde_json::to_string(&entity_type).unwrap_or_default();
        let db = self.db.lock().unwrap();

        let mut stmt = db
            .prepare(
                "SELECT id, entity_type, data, version, updated_at, source, checksum
                 FROM sync_entities
                 WHERE entity_type = ?",
            )
            .unwrap();

        let rows = stmt
            .query_map([&type_key], |row| row_to_entity(row))
            .unwrap();

        rows.filter_map(|r| r.ok()).collect()
    }

    /// Receive external sync event (from other platforms)
    pub fn receive_sync_event(
        &self,
        event: SyncEvent,
    ) -> Result<Option<SyncConflictResolution>, String> {
        let mut db = self.db.lock().unwrap();

        // Check for conflicts
        let conflict = {
            if let Some(local) = query_entity(&db, &event.entity.id)? {
                // Check if versions conflict
                if local.version >= event.entity.version
                    && local.source != event.entity.source
                {
                    Some(resolve_conflict(&self.config, &local, &event.entity))
                } else {
                    None
                }
            } else {
                None
            }
        };

        // If conflict, use resolution result; otherwise accept incoming
        if let Some(ref resolution) = conflict {
            println!(
                "[SyncEngine] Conflict detected for '{}', resolution: {:?}",
                resolution.entity_id, resolution.resolution_strategy
            );
        } else {
            // Accept incoming entity
            let incoming = &event.entity;
            let type_key = serde_json::to_string(&incoming.entity_type).unwrap_or_default();

            let tx = db.transaction().map_err(|e| e.to_string())?;

            tx.execute(
                "INSERT INTO sync_entities (id, entity_type, data, version, updated_at, source, checksum)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                     entity_type = excluded.entity_type,
                     data = excluded.data,
                     version = excluded.version,
                     updated_at = excluded.updated_at,
                     source = excluded.source,
                     checksum = excluded.checksum",
                (
                    &incoming.id,
                    &type_key,
                    serde_json::to_string(&incoming.data).unwrap_or_default(),
                    incoming.version as i64,
                    incoming.updated_at.to_rfc3339(),
                    serialize_source(&incoming.source),
                    &incoming.checksum,
                ),
            )
            .map_err(|e| e.to_string())?;

            tx.commit().map_err(|e| e.to_string())?;

            println!(
                "[SyncEngine] Received sync event for '{}' type '{:?}'",
                event.entity.id, event.entity.entity_type
            );
        }

        Ok(conflict)
    }

    /// Push offline queue to sync (when connection restored)
    pub fn flush_offline_queue(&self) -> Result<Vec<SyncEvent>, String> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(|e| e.to_string())?;

        // Read all queued events (stmt must be scoped to release borrow on tx before commit)
        let (events, ids_to_delete) = {
            let mut stmt = tx
                .prepare("SELECT id, event_data FROM sync_offline_queue ORDER BY id")
                .map_err(|e| e.to_string())?;

            let mut events = Vec::new();
            let mut ids_to_delete = Vec::new();

            let rows = stmt
                .query_map([], |row| {
                    let id: i64 = row.get(0)?;
                    let data: String = row.get(1)?;
                    Ok((id, data))
                })
                .map_err(|e| e.to_string())?;

            for result in rows {
                let (id, data) = result.map_err(|e| e.to_string())?;
                let event: SyncEvent =
                    serde_json::from_str(&data).map_err(|e| format!("Invalid offline event: {}", e))?;
                events.push(event);
                ids_to_delete.push(id);
            }

            (events, ids_to_delete)
        };
        // stmt is dropped here, releasing the borrow on tx

        // Delete flushed entries
        for id in ids_to_delete {
            tx.execute("DELETE FROM sync_offline_queue WHERE id = ?", [id])
                .map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;

        println!("[SyncEngine] Flushed {} offline sync events", events.len());
        Ok(events)
    }

    /// Add to offline queue (when no connection)
    pub fn queue_offline(&self, event: SyncEvent) {
        let event_data = serde_json::to_string(&event).unwrap_or_default();
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO sync_offline_queue (event_data) VALUES (?)",
            [&event_data],
        )
        .unwrap_or_else(|e| {
            eprintln!("[SyncEngine] Failed to queue offline event: {}", e);
            0
        });

        println!(
            "[SyncEngine] Queued offline sync event for '{}'",
            event.entity.id
        );
    }

    /// Get sync statistics
    pub fn get_stats(&self) -> SyncStats {
        let db = self.db.lock().unwrap();

        let total_entities: usize = db
            .query_row("SELECT COUNT(*) FROM sync_entities", [], |row| row.get(0))
            .unwrap_or(0);

        let offline_queue_size: usize = db
            .query_row("SELECT COUNT(*) FROM sync_offline_queue", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        let mut entities_by_type: HashMap<String, usize> = HashMap::new();
        let mut stmt = db
            .prepare("SELECT entity_type, COUNT(*) FROM sync_entities GROUP BY entity_type")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                let key: String = row.get(0)?;
                let count: usize = row.get(1)?;
                Ok((key, count))
            })
            .unwrap();
        for result in rows {
            if let Ok((key, count)) = result {
                entities_by_type.insert(key, count);
            }
        }

        let last_sync: Option<String> = db
            .query_row(
                "SELECT updated_at FROM sync_entities ORDER BY updated_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten();
        let last_sync = last_sync
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

        SyncStats {
            total_entities,
            entities_by_type,
            offline_queue_size,
            current_source: self.current_source.clone(),
            last_sync,
        }
    }

    /// Clear all entities
    pub fn clear(&self) {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().unwrap();

        tx.execute("DELETE FROM sync_entities", []).unwrap();
        tx.execute("DELETE FROM sync_versions", []).unwrap();

        tx.commit().unwrap();
        println!("[SyncEngine] Cleared all entities");
    }
}

// ---------------------------------------------------------------------------
// Sync Stats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStats {
    pub total_entities: usize,
    pub entities_by_type: HashMap<String, usize>,
    pub offline_queue_size: usize,
    pub current_source: SyncSource,
    pub last_sync: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// SQLite Helper Functions
// ---------------------------------------------------------------------------

/// Get the default database path: OS data directory / acp-ui / sync.db
fn default_db_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("acp-ui");
    path.push("sync.db");
    path
}

/// Serialize SyncSource to a string for SQLite storage
fn serialize_source(source: &SyncSource) -> String {
    serde_json::to_string(source).unwrap_or_default()
}

/// Deserialize SyncSource from string stored in SQLite
fn deserialize_source(s: &str) -> Result<SyncSource, String> {
    serde_json::from_str(s).map_err(|e| format!("Invalid source '{}': {}", s, e))
}

/// Deserialize SyncEntityType from string stored in SQLite
fn deserialize_entity_type(s: &str) -> Result<SyncEntityType, String> {
    serde_json::from_str(s).map_err(|e| format!("Invalid entity type '{}': {}", s, e))
}

/// Parse a DateTime from RFC3339 string stored in SQLite
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Invalid datetime '{}': {}", s, e))
}

/// Convert a rusqlite row into a SyncEntity
fn row_to_entity(row: &rusqlite::Row<'_>) -> Result<SyncEntity, rusqlite::Error> {
    let id: String = row.get(0)?;
    let entity_type_str: String = row.get(1)?;
    let data_str: String = row.get(2)?;
    let version: i64 = row.get(3)?;
    let updated_at_str: String = row.get(4)?;
    let source_str: String = row.get(5)?;
    let checksum: String = row.get(6)?;

    let entity_type = deserialize_entity_type(&entity_type_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    let data: serde_json::Value = serde_json::from_str(&data_str).unwrap_or(serde_json::Value::Null);

    let updated_at = parse_datetime(&updated_at_str).unwrap_or(Utc::now());

    let source = deserialize_source(&source_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    Ok(SyncEntity {
        id,
        entity_type,
        data,
        version: version as u64,
        updated_at,
        source,
        checksum,
    })
}

/// Query a single entity by ID from a Connection reference
fn query_entity(conn: &Connection, id: &str) -> Result<Option<SyncEntity>, String> {
    let result = conn
        .query_row(
            "SELECT id, entity_type, data, version, updated_at, source, checksum
             FROM sync_entities
             WHERE id = ?",
            [id],
            |row| row_to_entity(row),
        )
        .optional();

    match result {
        Ok(entity) => Ok(entity),
        Err(e) => Err(format!("Failed to query entity '{}': {}", id, e)),
    }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Calculate checksum for data integrity
/// Uses FNV-1a hash (64-bit) which is order-sensitive and collision-resistant
/// for JSON data without requiring external crates.
fn calculate_checksum(data: &serde_json::Value) -> String {
    let serialized = serde_json::to_string(data).unwrap_or_default();
    // FNV-1a 64-bit
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in serialized.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x1000000000000043);
    }
    format!("fnv1a-{:016x}", hash)
}

/// Resolve conflict between local and incoming entity
fn resolve_conflict(config: &SyncConfig, local: &SyncEntity, incoming: &SyncEntity) -> SyncConflictResolution {
    match config.default_conflict_strategy {
        ConflictStrategy::LastWriteWins => {
            // Compare timestamps
            if incoming.updated_at > local.updated_at {
                SyncConflictResolution {
                    entity_id: incoming.id.clone(),
                    winning_version: incoming.version,
                    winning_source: incoming.source.clone(),
                    resolution_strategy: ConflictStrategy::LastWriteWins,
                    merged_data: Some(incoming.data.clone()),
                }
            } else {
                SyncConflictResolution {
                    entity_id: local.id.clone(),
                    winning_version: local.version,
                    winning_source: local.source.clone(),
                    resolution_strategy: ConflictStrategy::LastWriteWins,
                    merged_data: Some(local.data.clone()),
                }
            }
        }
        ConflictStrategy::SourcePriority => {
            // Use source priority
            let local_priority = config
                .source_priority
                .iter()
                .position(|s| s == &local.source)
                .unwrap_or(0);
            let incoming_priority = config
                .source_priority
                .iter()
                .position(|s| s == &incoming.source)
                .unwrap_or(0);

            if incoming_priority >= local_priority {
                SyncConflictResolution {
                    entity_id: incoming.id.clone(),
                    winning_version: incoming.version,
                    winning_source: incoming.source.clone(),
                    resolution_strategy: ConflictStrategy::SourcePriority,
                    merged_data: Some(incoming.data.clone()),
                }
            } else {
                SyncConflictResolution {
                    entity_id: local.id.clone(),
                    winning_version: local.version,
                    winning_source: local.source.clone(),
                    resolution_strategy: ConflictStrategy::SourcePriority,
                    merged_data: Some(local.data.clone()),
                }
            }
        }
        ConflictStrategy::MergeFields => {
            // Try to merge data (simple deep merge for JSON objects)
            let merged = if local.data.is_object() && incoming.data.is_object() {
                let mut merged_map = local.data.as_object().cloned().unwrap_or_default();
                if let Some(incoming_obj) = incoming.data.as_object() {
                    for (k, v) in incoming_obj {
                        merged_map.insert(k.clone(), v.clone());
                    }
                }
                Some(serde_json::Value::Object(merged_map))
            } else {
                // Cannot merge non-objects, use incoming
                Some(incoming.data.clone())
            };

            SyncConflictResolution {
                entity_id: local.id.clone(),
                winning_version: local.version.max(incoming.version) + 1,
                winning_source: SyncSource::TauriDesktop, // Merged result
                resolution_strategy: ConflictStrategy::MergeFields,
                merged_data: merged,
            }
        }
        ConflictStrategy::ManualResolve => {
            // Keep both, flag for manual resolution
            SyncConflictResolution {
                entity_id: local.id.clone(),
                winning_version: local.version,
                winning_source: local.source.clone(),
                resolution_strategy: ConflictStrategy::ManualResolve,
                merged_data: None, // Needs manual resolution
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

use tauri::State;
use crate::AppState;

/// Put entity to sync
#[tauri::command]
pub fn sync_put_entity(
    state: State<'_, AppState>,
    entity_type: String,
    id: String,
    data: serde_json::Value,
) -> Result<SyncEntity, String> {
    let engine = state.sync_engine.lock().map_err(|e| e.to_string())?;
    let entity_type: SyncEntityType = serde_json::from_str(&entity_type)
        .map_err(|e| format!("Invalid entity type: {}", e))?;
    engine.put_entity(entity_type, id, data)
}

/// Get entity from sync
#[tauri::command]
pub fn sync_get_entity(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<SyncEntity>, String> {
    let engine = state.sync_engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_entity(&id))
}

/// Delete entity from sync
#[tauri::command]
pub fn sync_delete_entity(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let engine = state.sync_engine.lock().map_err(|e| e.to_string())?;
    engine.delete_entity(&id)
}

/// List entities by type
#[tauri::command]
pub fn sync_list_entities(
    state: State<'_, AppState>,
    entity_type: String,
) -> Result<Vec<SyncEntity>, String> {
    let engine = state.sync_engine.lock().map_err(|e| e.to_string())?;
    let entity_type: SyncEntityType = serde_json::from_str(&entity_type)
        .map_err(|e| format!("Invalid entity type: {}", e))?;
    Ok(engine.list_entities(entity_type))
}

/// Get sync statistics
#[tauri::command]
pub fn sync_get_stats(
    state: State<'_, AppState>,
) -> Result<SyncStats, String> {
    let engine = state.sync_engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.get_stats())
}

/// Receive external sync event
#[tauri::command]
pub fn sync_receive_event(
    state: State<'_, AppState>,
    event: SyncEvent,
) -> Result<Option<SyncConflictResolution>, String> {
    let engine = state.sync_engine.lock().map_err(|e| e.to_string())?;
    engine.receive_sync_event(event)
}

/// Flush offline queue
#[tauri::command]
pub fn sync_flush_offline(
    state: State<'_, AppState>,
) -> Result<Vec<SyncEvent>, String> {
    let engine = state.sync_engine.lock().map_err(|e| e.to_string())?;
    engine.flush_offline_queue()
}
