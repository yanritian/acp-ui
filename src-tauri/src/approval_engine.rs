// Approval Engine - Async approval protocol for workflow stages
// Manages human-in-the-loop approval queue across devices (desktop, mobile, web)

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

/// Status of an approval request
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

impl ApprovalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalStatus::Pending => "pending",
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::Rejected => "rejected",
            ApprovalStatus::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "pending" => Ok(ApprovalStatus::Pending),
            "approved" => Ok(ApprovalStatus::Approved),
            "rejected" => Ok(ApprovalStatus::Rejected),
            "expired" => Ok(ApprovalStatus::Expired),
            other => Err(format!("Unknown approval status: {}", other)),
        }
    }
}

/// Action associated with an approval option
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalAction {
    Approve,
    Reject,
    Feedback,
}

impl ApprovalAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalAction::Approve => "approve",
            ApprovalAction::Reject => "reject",
            ApprovalAction::Feedback => "feedback",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "approve" => Ok(ApprovalAction::Approve),
            "reject" => Ok(ApprovalAction::Reject),
            "feedback" => Ok(ApprovalAction::Feedback),
            other => Err(format!("Unknown approval action: {}", other)),
        }
    }
}

/// An option presented to the approver
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalOption {
    pub id: String,
    pub label: String,
    pub action: ApprovalAction,
}

/// A decision made by an approver
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDecision {
    pub option_id: String,
    pub feedback: Option<String>,
    pub decided_at: DateTime<Utc>,
    pub source: String,
}

/// An approval request that needs human review
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    pub id: String,
    pub workflow_id: String,
    pub stage_id: String,
    pub title: String,
    pub description: String,
    pub options: Vec<ApprovalOption>,
    pub created_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub status: ApprovalStatus,
    pub response: Option<ApprovalDecision>,
}

/// Statistics about the approval system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalStats {
    pub total: u64,
    pub pending: u64,
    pub approved: u64,
    pub rejected: u64,
    pub expired: u64,
}

// ---------------------------------------------------------------------------
// Callback / Notification
// ---------------------------------------------------------------------------

/// Callback type invoked when an approval request is decided.
/// Signature: Fn(&ApprovalRequest)
type ApprovalCallback = Box<dyn Fn(&ApprovalRequest) + Send + Sync>;

// ---------------------------------------------------------------------------
// Approval Engine
// ---------------------------------------------------------------------------

pub struct ApprovalEngine {
    /// In-memory cache of all requests keyed by ID.
    requests: Arc<RwLock<HashMap<String, ApprovalRequest>>>,
    /// Registered callbacks notified on decision events.
    callbacks: Arc<Mutex<Vec<ApprovalCallback>>>,
}

impl ApprovalEngine {
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // ----- Public API -----

    /// Register a callback to be invoked when any approval is decided.
    pub fn on_decided<F>(&self, callback: F)
    where
        F: Fn(&ApprovalRequest) + Send + Sync + 'static,
    {
        let mut cbs = self.callbacks.lock().unwrap();
        cbs.push(Box::new(callback));
    }

    /// Create a new approval request and persist it.
    pub fn create_request(
        &self,
        conn: &Connection,
        mut request: ApprovalRequest,
    ) -> Result<String, String> {
        if request.id.is_empty() {
            request.id = Uuid::new_v4().to_string();
        }
        if request.status == ApprovalStatus::Pending && request.created_at > Utc::now() {
            // default: set created_at to now if not set to a meaningful value
        }

        let options_json =
            serde_json::to_string(&request.options).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO approval_requests (
                id, workflow_id, stage_id, title, description, options_json,
                status, deadline, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                request.id,
                request.workflow_id,
                request.stage_id,
                request.title,
                request.description,
                options_json,
                request.status.as_str(),
                request.deadline.to_rfc3339(),
                request.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| e.to_string())?;

        {
            let mut map = self.requests.write().unwrap();
            map.insert(request.id.clone(), request.clone());
        }

        Ok(request.id)
    }

    /// Return all pending approval requests.
    pub fn get_pending(&self, conn: &Connection) -> Result<Vec<ApprovalRequest>, String> {
        // First try to load from DB (source of truth), merge with in-memory.
        let db_requests = self.load_by_status(conn, ApprovalStatus::Pending)?;

        let map = self.requests.read().unwrap();
        let mut merged: HashMap<String, ApprovalRequest> = db_requests
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect();

        // In-memory entries that may not yet be in DB take precedence
        for (id, req) in map.iter() {
            if req.status == ApprovalStatus::Pending {
                merged.entry(id.clone()).or_insert_with(|| req.clone());
            }
        }

        Ok(merged.into_values().collect())
    }

    /// Get a single approval request by ID.
    pub fn get_request(
        &self,
        conn: &Connection,
        id: &str,
    ) -> Result<Option<ApprovalRequest>, String> {
        // Check in-memory cache first
        {
            let map = self.requests.read().unwrap();
            if let Some(req) = map.get(id) {
                return Ok(Some(req.clone()));
            }
        }

        // Fall back to DB
        let mut stmt = conn
            .prepare(
                "SELECT id, workflow_id, stage_id, title, description, options_json,
                        status, deadline, response_json, created_at, decided_at, source
                 FROM approval_requests WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![id], |row| {
            let status_str: String = row.get(6)?;
            let deadline_str: String = row.get(7)?;
            let options_json: String = row.get(5)?;
            let response_json: Option<String> = row.get(8)?;
            let created_at_str: String = row.get(9)?;
            let decided_at_str: Option<String> = row.get(10)?;
            let source: Option<String> = row.get(11)?;

            let options: Vec<ApprovalOption> =
                serde_json::from_str(&options_json).unwrap_or_default();

            let response = match (response_json, decided_at_str, source) {
                (Some(resp_json), Some(decided_at_str), Some(src)) => {
                    let feedback: Option<String> =
                        serde_json::from_str(&resp_json).unwrap_or(None);
                    let decided_at = DateTime::parse_from_rfc3339(&decided_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok();
                    Some(ApprovalDecision {
                        option_id: "".to_string(),
                        feedback,
                        decided_at: decided_at.unwrap_or_else(Utc::now),
                        source: src,
                    })
                }
                _ => None,
            };

            Ok(ApprovalRequest {
                id: row.get(0)?,
                workflow_id: row.get(1)?,
                stage_id: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                options,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                deadline: DateTime::parse_from_rfc3339(&deadline_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                status: ApprovalStatus::from_str(&status_str).unwrap_or(ApprovalStatus::Pending),
                response,
            })
        });

        match result {
            Ok(req) => {
                // Cache the result
                let mut map = self.requests.write().unwrap();
                map.insert(req.id.clone(), req.clone());
                Ok(Some(req))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Decide (approve / reject / feedback) on a request.
    pub fn decide(
        &self,
        conn: &Connection,
        request_id: &str,
        decision: ApprovalDecision,
    ) -> Result<(), String> {
        // Resolve the option to determine the resulting status
        let option_action = {
            let map = self.requests.read().unwrap();
            if let Some(req) = map.get(request_id) {
                req.options
                    .iter()
                    .find(|o| o.id == decision.option_id)
                    .map(|o| o.action)
            } else {
                None
            }
        };

        // If not found in cache, try to load options from DB
        let option_action = option_action.or_else(|| {
            let mut stmt = conn
                .prepare("SELECT options_json FROM approval_requests WHERE id = ?1")
                .ok()?;
            let options_json: String = stmt.query_row(params![request_id], |row| row.get(0)).ok()?;
            let options: Vec<ApprovalOption> = serde_json::from_str(&options_json).ok()?;
            options
                .iter()
                .find(|o| o.id == decision.option_id)
                .map(|o| o.action)
        });

        let new_status = match option_action {
            Some(ApprovalAction::Approve) => ApprovalStatus::Approved,
            Some(ApprovalAction::Reject) => ApprovalStatus::Rejected,
            Some(ApprovalAction::Feedback) => ApprovalStatus::Pending,
            None => {
                return Err(format!("Option '{}' not found in request '{}'", decision.option_id, request_id));
            }
        };

        // Store the full decision as JSON to preserve option_id
        let response_json = serde_json::to_string(&decision).map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE approval_requests
             SET status = ?1, response_json = ?2, decided_at = ?3, source = ?4
             WHERE id = ?5",
            params![
                new_status.as_str(),
                response_json,
                decision.decided_at.to_rfc3339(),
                decision.source,
                request_id,
            ],
        )
        .map_err(|e| e.to_string())?;

        // Update in-memory cache
        {
            let mut map = self.requests.write().unwrap();
            if let Some(req) = map.get_mut(request_id) {
                req.status = new_status;
                req.response = Some(decision.clone());
            }
        }

        // Fire callbacks with the updated request
        let decided_request = {
            let map = self.requests.read().unwrap();
            map.get(request_id).cloned()
        };

        if let Some(ref req) = decided_request {
            let cbs = self.callbacks.lock().unwrap();
            for cb in cbs.iter() {
                cb(req);
            }
        }

        Ok(())
    }

    /// Expire requests whose deadline has passed and are still pending.
    /// Returns the number of expired requests.
    pub fn expire_old_requests(&self, conn: &Connection) -> Result<usize, String> {
        let now = Utc::now().to_rfc3339();

        let count = conn
            .execute(
                "UPDATE approval_requests
                 SET status = 'expired'
                 WHERE status = 'pending' AND deadline < ?1",
                params![now],
            )
            .map_err(|e| e.to_string())?;

        if count > 0 {
            // Remove expired from cache
            let mut map = self.requests.write().unwrap();
            map.retain(|_, v| v.status != ApprovalStatus::Expired);
        }

        Ok(count)
    }

    /// Get aggregate statistics.
    pub fn get_stats(&self, conn: &Connection) -> Result<ApprovalStats, String> {
        let total: u64 = conn
            .query_row("SELECT COUNT(*) FROM approval_requests", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let pending: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM approval_requests WHERE status = 'pending'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let approved: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM approval_requests WHERE status = 'approved'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let rejected: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM approval_requests WHERE status = 'rejected'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let expired: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM approval_requests WHERE status = 'expired'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        Ok(ApprovalStats {
            total,
            pending,
            approved,
            rejected,
            expired,
        })
    }

    // ----- Internal helpers -----

    /// Load all requests with a given status from the database.
    fn load_by_status(
        &self,
        conn: &Connection,
        status: ApprovalStatus,
    ) -> Result<Vec<ApprovalRequest>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT id, workflow_id, stage_id, title, description, options_json,
                        status, deadline, response_json, created_at, decided_at, source
                 FROM approval_requests WHERE status = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![status.as_str()], |row| {
                parse_approval_row(row)
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows)
    }
}

/// Parse a single approval row from the database.
fn parse_approval_row(
    row: &rusqlite::Row<'_>,
) -> Result<ApprovalRequest, rusqlite::Error> {
    let status_str: String = row.get(6)?;
    let deadline_str: String = row.get(7)?;
    let created_at_str: String = row.get(9)?;
    let options_json: String = row.get(5)?;
    let response_json: Option<String> = row.get(8)?;
    let decided_at_str: Option<String> = row.get(10)?;
    let source: Option<String> = row.get(11)?;

    let options: Vec<ApprovalOption> =
        serde_json::from_str(&options_json).unwrap_or_default();

    let response = match (response_json, decided_at_str, source) {
        (Some(resp_json), Some(decided_at_str), Some(src)) => {
            let decided_at = DateTime::parse_from_rfc3339(&decided_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .ok();

            // Try to deserialize the full ApprovalDecision from JSON
            
            serde_json::from_str::<ApprovalDecision>(&resp_json)
                .ok()
                .or_else(|| {
                    // Fallback: construct from partial data
                    Some(ApprovalDecision {
                        option_id: "".to_string(),
                        feedback: None,
                        decided_at: decided_at.unwrap_or_else(Utc::now),
                        source: src,
                    })
                })
        }
        _ => None,
    };

    Ok(ApprovalRequest {
        id: row.get(0)?,
        workflow_id: row.get(1)?,
        stage_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        options,
        created_at: DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        deadline: DateTime::parse_from_rfc3339(&deadline_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        status: ApprovalStatus::from_str(&status_str).unwrap_or(ApprovalStatus::Pending),
        response,
    })
}

// ---------------------------------------------------------------------------
// Table initialization (called from database.rs init_tables)
// ---------------------------------------------------------------------------

/// Create the approval_requests table if it does not exist.
pub fn init_approval_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS approval_requests (
            id TEXT PRIMARY KEY,
            workflow_id TEXT NOT NULL,
            stage_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            options_json TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            deadline TEXT NOT NULL,
            response_json TEXT,
            created_at TEXT NOT NULL,
            decided_at TEXT,
            source TEXT
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_approval_status ON approval_requests(status)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_approval_workflow ON approval_requests(workflow_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_approval_deadline ON approval_requests(deadline)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

use crate::AppState;
use tauri::State;

/// Get all pending approval requests.
#[tauri::command]
pub fn approval_get_pending(
    state: State<AppState>,
) -> Result<Vec<ApprovalRequest>, String> {
    let engine = state.approval_engine.lock().map_err(|e| e.to_string())?;
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let conn = db
        .as_ref()
        .ok_or("Database not initialized")?
        .conn
        .lock()
        .map_err(|e| e.to_string())?;
    engine.get_pending(&conn)
}

/// Get a single approval request by ID.
#[tauri::command]
pub fn approval_get_request(
    state: State<AppState>,
    id: String,
) -> Result<Option<ApprovalRequest>, String> {
    let engine = state.approval_engine.lock().map_err(|e| e.to_string())?;
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let conn = db
        .as_ref()
        .ok_or("Database not initialized")?
        .conn
        .lock()
        .map_err(|e| e.to_string())?;
    engine.get_request(&conn, &id)
}

/// Submit a decision for an approval request.
#[tauri::command]
pub fn approval_decide(
    state: State<AppState>,
    request_id: String,
    decision: ApprovalDecision,
) -> Result<(), String> {
    let engine = state.approval_engine.lock().map_err(|e| e.to_string())?;
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let conn = db
        .as_ref()
        .ok_or("Database not initialized")?
        .conn
        .lock()
        .map_err(|e| e.to_string())?;
    engine.decide(&conn, &request_id, decision)
}

/// Get approval system statistics.
#[tauri::command]
pub fn approval_get_stats(
    state: State<AppState>,
) -> Result<ApprovalStats, String> {
    let engine = state.approval_engine.lock().map_err(|e| e.to_string())?;
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let conn = db
        .as_ref()
        .ok_or("Database not initialized")?
        .conn
        .lock()
        .map_err(|e| e.to_string())?;
    engine.get_stats(&conn)
}
