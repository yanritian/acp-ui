use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::operator::commands::{FileChangeRecord, OperatorState, PendingGodotValidation};
use crate::operator::{
    is_d_drive_path, prepare_structured_patch, workspace_root, ApprovalRequest, OperatorEvent,
    OperatorTask, OperatorTaskStatus, PatchOperation, PreparedPatchSet, StructuredFileChange,
    StructuredPatchSet,
};

const SCHEMA_VERSION: u32 = 1;
const MAX_JSON_PAYLOAD_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct OperatorStateStore {
    db_path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct LoadedOperatorState {
    pub tasks: HashMap<String, OperatorTask>,
    pub events: HashMap<String, Vec<OperatorEvent>>,
    pub approvals: HashMap<String, Vec<ApprovalRequest>>,
    pub file_changes: HashMap<String, Vec<FileChangeRecord>>,
    pub pending_patches: HashMap<String, PreparedPatchSet>,
    pub pending_validations: HashMap<String, PendingGodotValidation>,
    pub recovery_origins: HashMap<String, OperatorTaskStatus>,
    pub patch_failures: Vec<PendingPatchRecoveryFailure>,
}

#[derive(Debug)]
pub(crate) struct PendingPatchRecoveryFailure {
    pub task_id: String,
    pub patch_id: String,
    pub error: String,
}

struct PendingPatchSnapshot {
    task_id: String,
    patch_id: String,
    project_root: String,
    artifact_json: String,
}

pub fn default_operator_state_db_path() -> Result<PathBuf, String> {
    if let Ok(value) = std::env::var("ACP_OPERATOR_STATE_DB") {
        let value = value.trim();
        if value.is_empty() {
            return Err("ACP_OPERATOR_STATE_DB cannot be empty".to_string());
        }
        let path = PathBuf::from(value);
        validate_d_drive_database_path(&path)?;
        return Ok(path);
    }

    Ok(workspace_root()
        .join(".operator")
        .join("operator-state.sqlite3"))
}

impl OperatorStateStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        validate_d_drive_database_path(&path)?;

        let parent = path
            .parent()
            .ok_or_else(|| "Operator database path has no parent directory".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Cannot create Operator database directory: {}", error))?;
        let canonical_parent = parent.canonicalize().map_err(|error| {
            format!(
                "Cannot canonicalize Operator database directory '{}': {}",
                parent.display(),
                error
            )
        })?;
        if !is_d_drive_path(&canonical_parent) {
            return Err(format!(
                "Operator database directory must resolve to D:, got '{}'",
                canonical_parent.display()
            ));
        }
        if path.exists() {
            let canonical_file = path.canonicalize().map_err(|error| {
                format!(
                    "Cannot canonicalize Operator database '{}': {}",
                    path.display(),
                    error
                )
            })?;
            if !is_d_drive_path(&canonical_file) {
                return Err(format!(
                    "Operator database must resolve to D:, got '{}'",
                    canonical_file.display()
                ));
            }
        }

        let store = Self { db_path: path };
        let mut connection = store.connection()?;
        migrate(&mut connection)?;
        Ok(store)
    }

    fn connection(&self) -> Result<Connection, String> {
        let connection = Connection::open(&self.db_path).map_err(|error| {
            format!(
                "Cannot open Operator database '{}': {}",
                self.db_path.display(),
                error
            )
        })?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|error| format!("Cannot configure Operator database timeout: {}", error))?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| format!("Cannot enable SQLite foreign keys: {}", error))?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| format!("Cannot enable SQLite WAL mode: {}", error))?;
        Ok(connection)
    }

    pub(crate) fn save(&self, state: &OperatorState) -> Result<(), String> {
        let tasks = serialize_tasks(state)?;
        let events = serialize_events(state)?;
        let approvals = serialize_approvals(state)?;
        let file_changes = serialize_file_changes(state)?;
        let pending_patches = serialize_pending_patches(state)?;
        let pending_validations = serialize_pending_validations(state)?;
        let recovery_origins = serialize_recovery_origins(state)?;

        let mut connection = self.connection()?;
        ensure_supported_schema(&connection)?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| format!("Cannot start Operator save transaction: {}", error))?;

        for (task_id, payload, updated_at) in tasks {
            transaction
                .execute(
                    "INSERT INTO operator_tasks (task_id, payload_json, updated_at)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(task_id) DO UPDATE SET
                       payload_json = excluded.payload_json,
                       updated_at = excluded.updated_at",
                    params![task_id, payload, updated_at],
                )
                .map_err(|error| format!("Cannot save Operator task: {}", error))?;
        }

        for (event_id, task_id, sequence, payload, created_at) in events {
            let inserted = transaction
                .execute(
                    "INSERT OR IGNORE INTO operator_events
                       (event_id, task_id, sequence, payload_json, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![event_id, task_id, sequence, payload, created_at],
                )
                .map_err(|error| format!("Cannot append Operator event: {}", error))?;
            if inserted == 0 {
                let existing = transaction
                    .query_row(
                        "SELECT task_id, sequence, payload_json, created_at
                         FROM operator_events WHERE event_id = ?1",
                        params![event_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, i64>(1)?,
                                row.get::<_, String>(2)?,
                                row.get::<_, String>(3)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|error| format!("Cannot verify existing Operator event: {}", error))?;
                if !matches!(
                    existing.as_ref(),
                    Some((existing_task_id, existing_sequence, existing_payload, existing_created_at))
                        if existing_task_id == &task_id
                            && *existing_sequence == sequence
                            && existing_payload == &payload
                            && existing_created_at == &created_at
                ) {
                    return Err(format!(
                        "Operator event consistency conflict for '{}'",
                        event_id
                    ));
                }
            }
        }

        transaction
            .execute("DELETE FROM operator_approvals", [])
            .map_err(|error| format!("Cannot replace Operator approvals: {}", error))?;
        for (approval_id, task_id, position, payload, updated_at) in approvals {
            transaction
                .execute(
                    "INSERT INTO operator_approvals
                       (approval_id, task_id, position, payload_json, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![approval_id, task_id, position, payload, updated_at],
                )
                .map_err(|error| format!("Cannot save Operator approval: {}", error))?;
        }

        transaction
            .execute("DELETE FROM operator_file_changes", [])
            .map_err(|error| format!("Cannot replace Operator file changes: {}", error))?;
        for (task_id, position, payload) in file_changes {
            transaction
                .execute(
                    "INSERT INTO operator_file_changes (task_id, position, payload_json)
                     VALUES (?1, ?2, ?3)",
                    params![task_id, position, payload],
                )
                .map_err(|error| format!("Cannot save Operator file change: {}", error))?;
        }

        transaction
            .execute("DELETE FROM operator_pending_patches", [])
            .map_err(|error| format!("Cannot replace pending patches: {}", error))?;
        for snapshot in pending_patches {
            transaction
                .execute(
                    "INSERT INTO operator_pending_patches
                       (task_id, patch_id, project_root, artifact_json, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        snapshot.task_id,
                        snapshot.patch_id,
                        snapshot.project_root,
                        snapshot.artifact_json,
                        chrono::Utc::now().to_rfc3339()
                    ],
                )
                .map_err(|error| format!("Cannot save pending patch: {}", error))?;
        }

        transaction
            .execute("DELETE FROM operator_pending_validations", [])
            .map_err(|error| format!("Cannot replace pending validations: {}", error))?;
        for (task_id, payload) in pending_validations {
            transaction
                .execute(
                    "INSERT INTO operator_pending_validations
                       (task_id, payload_json, updated_at)
                     VALUES (?1, ?2, ?3)",
                    params![task_id, payload, chrono::Utc::now().to_rfc3339()],
                )
                .map_err(|error| format!("Cannot save pending validation: {}", error))?;
        }

        transaction
            .execute("DELETE FROM operator_recovery_origins", [])
            .map_err(|error| format!("Cannot replace recovery origins: {}", error))?;
        for (task_id, status_json) in recovery_origins {
            transaction
                .execute(
                    "INSERT INTO operator_recovery_origins (task_id, status_json)
                     VALUES (?1, ?2)",
                    params![task_id, status_json],
                )
                .map_err(|error| format!("Cannot save recovery origin: {}", error))?;
        }

        transaction
            .execute(
                "INSERT INTO operator_meta (key, value) VALUES ('last_saved_at', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![chrono::Utc::now().to_rfc3339()],
            )
            .map_err(|error| format!("Cannot save Operator metadata: {}", error))?;
        transaction
            .commit()
            .map_err(|error| format!("Cannot commit Operator state: {}", error))
    }
}

impl OperatorStateStore {
    pub(crate) fn load(&self) -> Result<LoadedOperatorState, String> {
        let connection = self.connection()?;
        ensure_supported_schema(&connection)?;

        let mut tasks = HashMap::new();
        {
            let mut statement = connection
                .prepare("SELECT task_id, payload_json FROM operator_tasks ORDER BY task_id")
                .map_err(|error| format!("Cannot prepare Operator task recovery: {}", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| format!("Cannot query Operator tasks: {}", error))?;
            for row in rows {
                let (task_id, payload) =
                    row.map_err(|error| format!("Cannot read Operator task row: {}", error))?;
                let task: OperatorTask = deserialize_payload("task", &payload)?;
                if task.task_id != task_id {
                    return Err(format!(
                        "Operator task key mismatch: row '{}' contains '{}'",
                        task_id, task.task_id
                    ));
                }
                validate_task_project_path(&task)?;
                tasks.insert(task_id, task);
            }
        }

        let mut events: HashMap<String, Vec<OperatorEvent>> = HashMap::new();
        {
            let mut statement = connection
                .prepare(
                    "SELECT event_id, task_id, sequence, payload_json, created_at
                     FROM operator_events ORDER BY task_id, sequence",
                )
                .map_err(|error| format!("Cannot prepare Operator event recovery: {}", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                })
                .map_err(|error| format!("Cannot query Operator events: {}", error))?;
            for row in rows {
                let (event_id, task_id, sequence, payload, created_at) =
                    row.map_err(|error| format!("Cannot read Operator event row: {}", error))?;
                if !tasks.contains_key(&task_id) {
                    return Err(format!(
                        "Operator event '{}' references missing task '{}'",
                        event_id, task_id
                    ));
                }
                let expected_sequence = events.get(&task_id).map_or(0, Vec::len) as i64;
                if sequence != expected_sequence {
                    return Err(format!(
                        "Operator task '{}' has non-contiguous event sequence: expected {}, got {} for '{}'",
                        task_id, expected_sequence, sequence, event_id
                    ));
                }
                let mut event: OperatorEvent = deserialize_payload("event", &payload)?;
                if event.event_id != event_id
                    || event.task_id != task_id
                    || event.timestamp != created_at
                {
                    return Err(format!("Operator event key mismatch for '{}'", event_id));
                }
                // Backfill sequence for databases created before the v1 event
                // cursor was serialized into the payload.
                if event.sequence != 0 && event.sequence != expected_sequence as u64 {
                    return Err(format!(
                        "Operator event '{}' payload sequence {} does not match row {}",
                        event_id, event.sequence, expected_sequence
                    ));
                }
                event.sequence = expected_sequence as u64;
                events.entry(task_id).or_default().push(event);
            }
        }

        let mut approvals: HashMap<String, Vec<ApprovalRequest>> = HashMap::new();
        {
            let mut statement = connection
                .prepare(
                    "SELECT approval_id, task_id, payload_json
                     FROM operator_approvals ORDER BY task_id, position",
                )
                .map_err(|error| format!("Cannot prepare approval recovery: {}", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(|error| format!("Cannot query Operator approvals: {}", error))?;
            for row in rows {
                let (approval_id, task_id, payload) =
                    row.map_err(|error| format!("Cannot read approval row: {}", error))?;
                if !tasks.contains_key(&task_id) {
                    return Err(format!(
                        "Operator approval '{}' references missing task '{}'",
                        approval_id, task_id
                    ));
                }
                let approval: ApprovalRequest = deserialize_payload("approval", &payload)?;
                if approval.approval_id != approval_id || approval.task_id != task_id {
                    return Err(format!(
                        "Operator approval key mismatch for '{}'",
                        approval_id
                    ));
                }
                approvals.entry(task_id).or_default().push(approval);
            }
        }

        let mut file_changes: HashMap<String, Vec<FileChangeRecord>> = HashMap::new();
        {
            let mut statement = connection
                .prepare(
                    "SELECT task_id, payload_json
                     FROM operator_file_changes ORDER BY task_id, position",
                )
                .map_err(|error| format!("Cannot prepare file change recovery: {}", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| format!("Cannot query Operator file changes: {}", error))?;
            for row in rows {
                let (task_id, payload) = row
                    .map_err(|error| format!("Cannot read Operator file change row: {}", error))?;
                if !tasks.contains_key(&task_id) {
                    return Err(format!(
                        "Operator file change references missing task '{}'",
                        task_id
                    ));
                }
                let change: FileChangeRecord = deserialize_payload("file change", &payload)?;
                file_changes.entry(task_id).or_default().push(change);
            }
        }

        let mut pending_validations = HashMap::new();
        {
            let mut statement = connection
                .prepare(
                    "SELECT task_id, payload_json
                     FROM operator_pending_validations ORDER BY task_id",
                )
                .map_err(|error| format!("Cannot prepare validation recovery: {}", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| format!("Cannot query pending validations: {}", error))?;
            for row in rows {
                let (task_id, payload) =
                    row.map_err(|error| format!("Cannot read pending validation row: {}", error))?;
                require_existing_task(&tasks, &task_id, "pending validation")?;
                let validation: PendingGodotValidation =
                    deserialize_payload("pending validation", &payload)?;
                validate_pending_validation(&validation)?;
                pending_validations.insert(task_id, validation);
            }
        }

        let mut recovery_origins = HashMap::new();
        {
            let mut statement = connection
                .prepare(
                    "SELECT task_id, status_json
                     FROM operator_recovery_origins ORDER BY task_id",
                )
                .map_err(|error| format!("Cannot prepare recovery origin query: {}", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|error| format!("Cannot query recovery origins: {}", error))?;
            for row in rows {
                let (task_id, payload) =
                    row.map_err(|error| format!("Cannot read recovery origin row: {}", error))?;
                require_existing_task(&tasks, &task_id, "recovery origin")?;
                let status: OperatorTaskStatus = deserialize_payload("recovery origin", &payload)?;
                recovery_origins.insert(task_id, status);
            }
        }

        let mut pending_patches = HashMap::new();
        let mut patch_failures = Vec::new();
        {
            let mut statement = connection
                .prepare(
                    "SELECT task_id, patch_id, project_root, artifact_json
                     FROM operator_pending_patches ORDER BY task_id",
                )
                .map_err(|error| format!("Cannot prepare pending patch recovery: {}", error))?;
            let rows = statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                })
                .map_err(|error| format!("Cannot query pending patches: {}", error))?;
            for row in rows {
                let (task_id, patch_id, project_root, artifact_json) =
                    row.map_err(|error| format!("Cannot read pending patch row: {}", error))?;
                require_existing_task(&tasks, &task_id, "pending patch")?;
                validate_patch_identifier(&patch_id)?;
                ensure_payload_size("pending patch artifact", &artifact_json)?;
                let project_root = PathBuf::from(project_root);
                let recovered = if !is_d_drive_path(&project_root) {
                    Err(format!(
                        "pending patch project must be on D:, got '{}'",
                        project_root.display()
                    ))
                } else {
                    prepare_structured_patch(&artifact_json, &project_root)
                        .map_err(|error| error.to_string())
                };
                match recovered {
                    Ok(mut patch) => {
                        patch.patch_id = patch_id;
                        pending_patches.insert(task_id, patch);
                    }
                    Err(error) => patch_failures.push(PendingPatchRecoveryFailure {
                        task_id,
                        patch_id,
                        error,
                    }),
                }
            }
        }

        for task_id in tasks.keys() {
            events.entry(task_id.clone()).or_default();
            approvals.entry(task_id.clone()).or_default();
            file_changes.entry(task_id.clone()).or_default();
        }

        Ok(LoadedOperatorState {
            tasks,
            events,
            approvals,
            file_changes,
            pending_patches,
            pending_validations,
            recovery_origins,
            patch_failures,
        })
    }
}

fn migrate(connection: &mut Connection) -> Result<(), String> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| format!("Cannot read Operator schema version: {}", error))?;
    if version > SCHEMA_VERSION {
        return Err(format!(
            "Operator database schema {} is newer than supported schema {}",
            version, SCHEMA_VERSION
        ));
    }
    if version == SCHEMA_VERSION {
        return Ok(());
    }

    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Cannot start Operator migration: {}", error))?;
    transaction
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS operator_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS operator_tasks (
                task_id TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS operator_events (
                event_id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(task_id, sequence)
             );
             CREATE INDEX IF NOT EXISTS idx_operator_events_task_sequence
                ON operator_events(task_id, sequence);
             CREATE TRIGGER IF NOT EXISTS operator_events_no_update
                BEFORE UPDATE ON operator_events
                BEGIN SELECT RAISE(ABORT, 'operator events are append-only'); END;
             CREATE TRIGGER IF NOT EXISTS operator_events_no_delete
                BEFORE DELETE ON operator_events
                BEGIN SELECT RAISE(ABORT, 'operator events are append-only'); END;
             CREATE TABLE IF NOT EXISTS operator_approvals (
                approval_id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(task_id, position)
             );
             CREATE INDEX IF NOT EXISTS idx_operator_approvals_task
                ON operator_approvals(task_id, position);
             CREATE TABLE IF NOT EXISTS operator_file_changes (
                task_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                payload_json TEXT NOT NULL,
                PRIMARY KEY(task_id, position)
             );
             CREATE TABLE IF NOT EXISTS operator_pending_patches (
                task_id TEXT PRIMARY KEY,
                patch_id TEXT NOT NULL,
                project_root TEXT NOT NULL,
                artifact_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS operator_pending_validations (
                task_id TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS operator_recovery_origins (
                task_id TEXT PRIMARY KEY,
                status_json TEXT NOT NULL
             );",
        )
        .map_err(|error| format!("Cannot create Operator schema: {}", error))?;
    transaction
        .pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(|error| format!("Cannot set Operator schema version: {}", error))?;
    transaction
        .commit()
        .map_err(|error| format!("Cannot commit Operator migration: {}", error))
}

fn ensure_supported_schema(connection: &Connection) -> Result<(), String> {
    let version: u32 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| format!("Cannot read Operator schema version: {}", error))?;
    if version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(format!(
            "Unsupported Operator database schema {}; expected {}",
            version, SCHEMA_VERSION
        ))
    }
}

fn serialize_tasks(state: &OperatorState) -> Result<Vec<(String, String, String)>, String> {
    state
        .tasks
        .iter()
        .map(|(task_id, task)| {
            if task_id != &task.task_id {
                return Err(format!(
                    "Operator task map key '{}' does not match payload '{}'",
                    task_id, task.task_id
                ));
            }
            validate_task_project_path(task)?;
            Ok((
                task_id.clone(),
                serialize_payload("task", task)?,
                task.updated_at.clone(),
            ))
        })
        .collect()
}

fn serialize_events(
    state: &OperatorState,
) -> Result<Vec<(String, String, i64, String, String)>, String> {
    let mut rows = Vec::new();
    for (task_id, events) in &state.events {
        require_existing_task(&state.tasks, task_id, "event stream")?;
        for (position, event) in events.iter().enumerate() {
            if event.task_id != *task_id {
                return Err(format!(
                    "Operator event '{}' belongs to task '{}' instead of '{}'",
                    event.event_id, event.task_id, task_id
                ));
            }
            rows.push((
                event.event_id.clone(),
                task_id.clone(),
                position as i64,
                serialize_payload("event", event)?,
                event.timestamp.clone(),
            ));
        }
    }
    Ok(rows)
}

fn serialize_approvals(
    state: &OperatorState,
) -> Result<Vec<(String, String, i64, String, String)>, String> {
    let mut rows = Vec::new();
    for (task_id, approvals) in &state.approvals {
        require_existing_task(&state.tasks, task_id, "approval stream")?;
        for (position, approval) in approvals.iter().enumerate() {
            if approval.task_id != *task_id {
                return Err(format!(
                    "Operator approval '{}' belongs to task '{}' instead of '{}'",
                    approval.approval_id, approval.task_id, task_id
                ));
            }
            rows.push((
                approval.approval_id.clone(),
                task_id.clone(),
                position as i64,
                serialize_payload("approval", approval)?,
                approval
                    .resolved_at
                    .clone()
                    .unwrap_or_else(|| approval.created_at.clone()),
            ));
        }
    }
    Ok(rows)
}

fn serialize_file_changes(state: &OperatorState) -> Result<Vec<(String, i64, String)>, String> {
    let mut rows = Vec::new();
    for (task_id, changes) in &state.file_changes {
        require_existing_task(&state.tasks, task_id, "file change stream")?;
        for (position, change) in changes.iter().enumerate() {
            rows.push((
                task_id.clone(),
                position as i64,
                serialize_payload("file change", change)?,
            ));
        }
    }
    Ok(rows)
}

fn serialize_pending_patches(state: &OperatorState) -> Result<Vec<PendingPatchSnapshot>, String> {
    state
        .pending_patches
        .iter()
        .map(|(task_id, patch)| {
            require_existing_task(&state.tasks, task_id, "pending patch")?;
            validate_patch_identifier(&patch.patch_id)?;
            if !is_d_drive_path(&patch.project_root) {
                return Err(format!(
                    "Pending patch project must be on D:, got '{}'",
                    patch.project_root.display()
                ));
            }

            let mut changes = Vec::with_capacity(patch.changes.len());
            for change in &patch.changes {
                let expected_sha256 = match change.operation {
                    PatchOperation::Create => None,
                    PatchOperation::Replace => {
                        Some(change.original_sha256.clone().ok_or_else(|| {
                            format!(
                                "Replace change '{}' is missing its preview hash",
                                change.path
                            )
                        })?)
                    }
                };
                changes.push(StructuredFileChange {
                    path: change.path.clone(),
                    operation: change.operation.clone(),
                    content: change.new_content.clone(),
                    expected_sha256,
                });
            }
            let artifact_json = serde_json::to_string(&StructuredPatchSet {
                version: 1,
                summary: patch.summary.clone(),
                changes,
                validation: patch.validation.clone(),
            })
            .map_err(|error| format!("Cannot serialize pending patch: {}", error))?;
            ensure_payload_size("pending patch artifact", &artifact_json)?;

            Ok(PendingPatchSnapshot {
                task_id: task_id.clone(),
                patch_id: patch.patch_id.clone(),
                project_root: patch.project_root.to_string_lossy().to_string(),
                artifact_json,
            })
        })
        .collect()
}

fn serialize_pending_validations(state: &OperatorState) -> Result<Vec<(String, String)>, String> {
    state
        .pending_validations
        .iter()
        .map(|(task_id, validation)| {
            require_existing_task(&state.tasks, task_id, "pending validation")?;
            validate_pending_validation(validation)?;
            Ok((
                task_id.clone(),
                serialize_payload("pending validation", validation)?,
            ))
        })
        .collect()
}

fn serialize_recovery_origins(state: &OperatorState) -> Result<Vec<(String, String)>, String> {
    state
        .recovery_origins
        .iter()
        .map(|(task_id, status)| {
            require_existing_task(&state.tasks, task_id, "recovery origin")?;
            Ok((
                task_id.clone(),
                serialize_payload("recovery origin", status)?,
            ))
        })
        .collect()
}

fn serialize_payload<T: serde::Serialize>(kind: &str, value: &T) -> Result<String, String> {
    let payload = serde_json::to_string(value)
        .map_err(|error| format!("Cannot serialize Operator {}: {}", kind, error))?;
    ensure_payload_size(kind, &payload)?;
    Ok(payload)
}

fn deserialize_payload<T: DeserializeOwned>(kind: &str, payload: &str) -> Result<T, String> {
    ensure_payload_size(kind, payload)?;
    serde_json::from_str(payload)
        .map_err(|error| format!("Cannot deserialize Operator {}: {}", kind, error))
}

fn ensure_payload_size(kind: &str, payload: &str) -> Result<(), String> {
    if payload.len() > MAX_JSON_PAYLOAD_BYTES {
        Err(format!(
            "Operator {} exceeds {} bytes",
            kind, MAX_JSON_PAYLOAD_BYTES
        ))
    } else {
        Ok(())
    }
}

fn validate_d_drive_database_path(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || !is_d_drive_path(path) {
        return Err(format!(
            "Operator database must be an absolute D: path, got '{}'",
            path.display()
        ));
    }
    if path.file_name().is_none() {
        return Err("Operator database path must include a file name".to_string());
    }
    Ok(())
}

fn validate_task_project_path(task: &OperatorTask) -> Result<(), String> {
    let project_path = PathBuf::from(&task.project_path);
    if !project_path.is_absolute() || !is_d_drive_path(&project_path) {
        return Err(format!(
            "Operator task '{}' project must be an absolute D: path, got '{}'",
            task.task_id, task.project_path
        ));
    }
    Ok(())
}

fn validate_pending_validation(validation: &PendingGodotValidation) -> Result<(), String> {
    if !is_d_drive_path(&validation.project_root) {
        return Err(format!(
            "Pending validation project must be on D:, got '{}'",
            validation.project_root.display()
        ));
    }
    let backup_root = PathBuf::from(&validation.backup_root);
    if !is_d_drive_path(&backup_root) {
        return Err(format!(
            "Pending validation backup must be on D:, got '{}'",
            validation.backup_root
        ));
    }
    Ok(())
}

fn validate_patch_identifier(patch_id: &str) -> Result<(), String> {
    if patch_id.len() > 128
        || !patch_id.starts_with("patch_")
        || !patch_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(format!("Invalid pending patch identifier '{}'", patch_id));
    }
    Ok(())
}

fn require_existing_task<T>(
    tasks: &HashMap<String, T>,
    task_id: &str,
    kind: &str,
) -> Result<(), String> {
    if tasks.contains_key(task_id) {
        Ok(())
    } else {
        Err(format!(
            "Operator {} references missing task '{}'",
            kind, task_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::commands::start_task_in_state;
    use crate::operator::{ApprovalPolicy, StartTaskRequest, TaskMode};
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let path = std::env::current_dir()
                .expect("current dir")
                .join(".tmp-tests")
                .join(format!(
                    "persistence_{}_{}_{}",
                    name,
                    std::process::id(),
                    unique
                ));
            fs::create_dir_all(&path).expect("create persistence test dir");
            Self { path }
        }

        fn db_path(&self) -> PathBuf {
            self.path.join("operator-state.sqlite3")
        }

        fn project_path(&self) -> PathBuf {
            let project = self.path.join("godot-project");
            fs::create_dir_all(project.join("scripts")).expect("create scripts");
            fs::write(
                project.join("project.godot"),
                "[application]\nconfig/name=\"Persistence Test\"\n",
            )
            .expect("write project marker");
            fs::write(
                project.join("scripts/player.gd"),
                "extends Node\nvar jumps = 1\n",
            )
            .expect("write player");
            project
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            if is_d_drive_path(&self.path) {
                let _ = fs::remove_dir_all(&self.path);
            }
        }
    }

    fn state_with_waiting_approval(project: &Path) -> (OperatorState, String) {
        let mut state = OperatorState::new();
        let response = start_task_in_state(
            &mut state,
            StartTaskRequest {
                domain: "game.godot".to_string(),
                project_path: project.to_string_lossy().to_string(),
                goal: "Persist this plan".to_string(),
                mode: Some(TaskMode::ProposeThenApply),
                approval_policy: Some(ApprovalPolicy::SafeDefault),
            },
        )
        .expect("start persistent task");
        (state, response.task_id)
    }

    fn replace_patch(project: &Path) -> PreparedPatchSet {
        let artifact = serde_json::to_string(&json!({
            "version": 1,
            "summary": "Increase jumps",
            "changes": [{
                "path": "scripts/player.gd",
                "operation": "replace",
                "content": "extends Node\nvar jumps = 2\n"
            }],
            "validation": ["Open the main scene"]
        }))
        .expect("serialize patch");
        prepare_structured_patch(&artifact, project).expect("prepare patch")
    }

    #[test]
    fn rejects_non_d_database_path_before_opening_it() {
        let error = OperatorStateStore::open("E:/forbidden/operator-state.sqlite3")
            .expect_err("non-D database must fail");

        assert!(error.contains("absolute D:"));
    }

    #[test]
    fn initializes_schema_v1_idempotently() {
        let dir = TestDir::new("schema");
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        let connection = store.connection().expect("open connection");
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read schema version");
        assert_eq!(version, SCHEMA_VERSION);

        OperatorStateStore::open(dir.db_path()).expect("reopen schema");
    }

    #[test]
    fn roundtrips_tasks_events_approvals_and_file_changes() {
        let dir = TestDir::new("roundtrip");
        let project = dir.project_path();
        let (mut state, task_id) = state_with_waiting_approval(&project);
        state
            .file_changes
            .entry(task_id.clone())
            .or_default()
            .push(FileChangeRecord {
                path: "scripts/player.gd".to_string(),
                change_type: "modified".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        store.save(&state).expect("save state");

        let loaded = store.load().expect("load state");
        assert_eq!(loaded.tasks.len(), 1);
        assert_eq!(
            loaded.tasks[&task_id].status,
            OperatorTaskStatus::WaitingApproval
        );
        assert_eq!(loaded.events[&task_id].len(), state.events[&task_id].len());
        assert_eq!(loaded.approvals[&task_id].len(), 1);
        assert_eq!(loaded.file_changes[&task_id].len(), 1);
    }

    #[test]
    fn persisted_events_are_append_only() {
        let dir = TestDir::new("append_only");
        let project = dir.project_path();
        let (state, _) = state_with_waiting_approval(&project);
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        store.save(&state).expect("save state");
        let connection = store.connection().expect("open connection");

        assert!(connection
            .execute(
                "UPDATE operator_events SET payload_json = '{}' WHERE 1 = 1",
                []
            )
            .is_err());
        assert!(connection
            .execute("DELETE FROM operator_events", [])
            .is_err());
    }

    #[test]
    fn rejects_reordered_existing_events_on_save() {
        let dir = TestDir::new("event_reorder");
        let project = dir.project_path();
        let (mut state, task_id) = state_with_waiting_approval(&project);
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        store.save(&state).expect("save original event stream");

        let events = state.events.get_mut(&task_id).expect("event stream");
        assert!(events.len() >= 2, "test requires at least two events");
        let original_first_id = events[0].event_id.clone();
        events.swap(0, 1);

        let error = store
            .save(&state)
            .expect_err("reordered persisted events must fail");
        assert!(error.contains("event consistency conflict"));

        let loaded = store.load().expect("load original event stream");
        assert_eq!(loaded.events[&task_id][0].event_id, original_first_id);
    }

    #[test]
    fn rejects_non_contiguous_event_sequences_on_load() {
        let dir = TestDir::new("event_gap");
        let project = dir.project_path();
        let (state, task_id) = state_with_waiting_approval(&project);
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        store.save(&state).expect("save original event stream");

        let mut extra_event = state.events[&task_id]
            .last()
            .expect("existing event")
            .clone();
        extra_event.event_id = format!("evt_gap_{}", uuid::Uuid::new_v4());
        let payload = serialize_payload("event", &extra_event).expect("serialize gap event");
        let gap_sequence = state.events[&task_id].len() as i64 + 10;
        let connection = store.connection().expect("open connection");
        connection
            .execute(
                "INSERT INTO operator_events
                   (event_id, task_id, sequence, payload_json, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    extra_event.event_id,
                    task_id,
                    gap_sequence,
                    payload,
                    extra_event.timestamp
                ],
            )
            .expect("inject event gap");
        drop(connection);

        let error = store.load().expect_err("event sequence gap must fail");
        assert!(error.contains("non-contiguous event sequence"));
    }

    #[test]
    fn failed_snapshot_validation_keeps_last_committed_state() {
        let dir = TestDir::new("rollback");
        let project = dir.project_path();
        let (mut state, task_id) = state_with_waiting_approval(&project);
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        store.save(&state).expect("save valid state");

        state.tasks.get_mut(&task_id).expect("task").project_path =
            "E:/forbidden/project".to_string();
        assert!(store.save(&state).is_err());

        let loaded = store.load().expect("load previous snapshot");
        assert_eq!(
            loaded.tasks[&task_id].project_path,
            project.to_string_lossy()
        );
    }

    #[test]
    fn pending_patch_is_revalidated_and_stale_changes_are_reported() {
        let dir = TestDir::new("pending_patch");
        let project = dir.project_path();
        let (mut state, task_id) = state_with_waiting_approval(&project);
        let patch = replace_patch(&project);
        let patch_id = patch.patch_id.clone();
        state.pending_patches.insert(task_id.clone(), patch);
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        store.save(&state).expect("save pending patch");

        let loaded = store.load().expect("load valid pending patch");
        assert_eq!(loaded.pending_patches[&task_id].patch_id, patch_id);
        assert!(loaded.patch_failures.is_empty());

        fs::write(
            project.join("scripts/player.gd"),
            "extends Node\nvar jumps = 99\n",
        )
        .expect("operator edit");
        let stale = store.load().expect("load stale state");
        assert!(!stale.pending_patches.contains_key(&task_id));
        assert_eq!(stale.patch_failures.len(), 1);
        assert!(stale.patch_failures[0].error.contains("stale"));
    }

    #[test]
    fn rejects_database_schema_newer_than_the_binary() {
        let dir = TestDir::new("future_schema");
        let store = OperatorStateStore::open(dir.db_path()).expect("open store");
        let connection = store.connection().expect("open connection");
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .expect("set future schema");
        drop(connection);
        drop(store);

        let error = OperatorStateStore::open(dir.db_path()).expect_err("future schema must fail");
        assert!(error.contains("newer than supported"));
    }
}
