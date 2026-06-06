// Sync Engine - Multi-platform data synchronization (Phase 3)
// App / Tauri / Web data real-time sync with Local-First strategy

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use chrono::{DateTime, Utc};

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
pub struct SyncEngine {
    /// Configuration
    config: SyncConfig,
    /// In-memory entity cache
    entity_cache: Arc<RwLock<HashMap<String, SyncEntity>>>,
    /// Version tracking per entity type
    version_map: Arc<RwLock<HashMap<String, u64>>>,
    /// Offline queue for pending syncs
    offline_queue: Arc<Mutex<Vec<SyncEvent>>>,
    /// Current sync source (where this engine is running)
    current_source: SyncSource,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(source: SyncSource) -> Self {
        let config = SyncConfig::default();

        Self {
            config,
            entity_cache: Arc::new(RwLock::new(HashMap::new())),
            version_map: Arc::new(RwLock::new(HashMap::new())),
            offline_queue: Arc::new(Mutex::new(Vec::new())),
            current_source: source,
        }
    }

    /// Put an entity into sync (create or update)
    pub fn put_entity(&self, entity_type: SyncEntityType, id: String, data: serde_json::Value) -> Result<SyncEntity, String> {
        // Get current version for this entity type
        let version = {
            let mut versions = self.version_map.write().unwrap();
            let type_key = serde_json::to_string(&entity_type).unwrap_or_default();
            let current = versions.get(&type_key).copied().unwrap_or(0);
            let new_version = current + 1;
            versions.insert(type_key, new_version);
            new_version
        };

        // Calculate checksum
        let checksum = calculate_checksum(&data);

        // Create sync entity
        let entity = SyncEntity {
            id: id.clone(),
            entity_type: entity_type.clone(),
            data,
            version,
            updated_at: Utc::now(),
            source: self.current_source.clone(),
            checksum,
        };

        // Store in cache
        {
            let mut cache = self.entity_cache.write().unwrap();
            cache.insert(id.clone(), entity.clone());
        }

        println!("[SyncEngine] Put entity '{}' type '{}' version {}", id, serde_json::to_string(&entity_type).unwrap_or_default(), version);
        Ok(entity)
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: &str) -> Option<SyncEntity> {
        let cache = self.entity_cache.read().unwrap();
        cache.get(id).cloned()
    }

    /// Delete an entity
    pub fn delete_entity(&self, id: &str) -> Result<(), String> {
        let entity = {
            let mut cache = self.entity_cache.write().unwrap();
            cache.remove(id)
        };

        if let Some(entity) = entity {
            println!("[SyncEngine] Deleted entity '{}'", id);
            Ok(())
        } else {
            Err(format!("Entity '{}' not found", id))
        }
    }

    /// List entities by type
    pub fn list_entities(&self, entity_type: SyncEntityType) -> Vec<SyncEntity> {
        let cache = self.entity_cache.read().unwrap();
        cache.values()
            .filter(|e| e.entity_type == entity_type)
            .cloned()
            .collect()
    }

    /// Receive external sync event (from other platforms)
    pub fn receive_sync_event(&self, event: SyncEvent) -> Result<Option<SyncConflictResolution>, String> {
        // Check for conflicts
        let conflict = {
            let cache = self.entity_cache.read().unwrap();
            if let Some(local) = cache.get(&event.entity.id) {
                // Check if versions conflict
                if local.version >= event.entity.version && local.source != event.entity.source {
                    Some(resolve_conflict(&self.config, local, &event.entity))
                } else {
                    None
                }
            } else {
                None
            }
        };

        // If conflict, use resolution result; otherwise accept incoming
        if let Some(resolution) = &conflict {
            println!("[SyncEngine] Conflict detected for '{}', resolution: {:?}",
                resolution.entity_id, resolution.resolution_strategy);
        } else {
            // Accept incoming entity
            {
                let mut cache = self.entity_cache.write().unwrap();
                cache.insert(event.entity.id.clone(), event.entity.clone());
            }
            println!("[SyncEngine] Received sync event for '{}' type '{:?}'",
                event.entity.id, event.entity.entity_type);
        }

        Ok(conflict)
    }

    /// Push offline queue to sync (when connection restored)
    pub fn flush_offline_queue(&self) -> Result<Vec<SyncEvent>, String> {
        let events = {
            let mut queue = self.offline_queue.lock().unwrap();
            std::mem::take(&mut *queue)
        };

        println!("[SyncEngine] Flushed {} offline sync events", events.len());
        Ok(events)
    }

    /// Add to offline queue (when no connection)
    pub fn queue_offline(&self, event: SyncEvent) {
        println!("[SyncEngine] Queued offline sync event for '{}'", event.entity.id);
        let mut queue = self.offline_queue.lock().unwrap();
        queue.push(event);
    }

    /// Get sync statistics
    pub fn get_stats(&self) -> SyncStats {
        let cache = self.entity_cache.read().unwrap();
        let queue = self.offline_queue.lock().unwrap();

        let by_type: HashMap<String, usize> = cache.values()
            .fold(HashMap::new(), |mut acc, e| {
                let key = serde_json::to_string(&e.entity_type).unwrap_or_default();
                *acc.entry(key).or_insert(0) += 1;
                acc
            });

        SyncStats {
            total_entities: cache.len(),
            entities_by_type: by_type,
            offline_queue_size: queue.len(),
            current_source: self.current_source.clone(),
            last_sync: cache.values().map(|e| e.updated_at).max(),
        }
    }

    /// Clear all entities
    pub fn clear(&self) {
        let mut cache = self.entity_cache.write().unwrap();
        cache.clear();
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
// Helper Functions
// ---------------------------------------------------------------------------

/// Calculate checksum for data integrity
fn calculate_checksum(data: &serde_json::Value) -> String {
    // Simple hash using serde_json serialization
    let serialized = serde_json::to_string(data).unwrap_or_default();
    let hash = serialized.as_bytes().iter().fold(0u64, |acc, b| acc + *b as u64);
    format!("crc-{:016x}", hash)
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
            let local_priority = config.source_priority.iter()
                .position(|s| s == &local.source)
                .unwrap_or(0);
            let incoming_priority = config.source_priority.iter()
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