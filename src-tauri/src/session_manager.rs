//! Session Management Module (Claw Code inspired)
//!
//! Provides session persistence, compaction, and branch lock collision detection.
//! NOTE: Module is imported but commands not yet registered in lib.rs - marked for future use.

#![allow(dead_code)] // Reserved for future session persistence feature

use chrono::{DateTime, Utc};
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Session record for an agent conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub messages: Vec<SessionMessage>,
    pub branch_lock: Option<String>,
    pub status: SessionStatus,
    pub compaction_count: i64,
}

/// Message within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub compressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    Active,
    Paused,
    Completed,
    Compacted,
}

/// Session Manager (Claw Code inspired)
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    branch_locks: HashMap<String, String>, // branch -> session_id
    db: Arc<Mutex<Connection>>,
    max_messages: usize,
    compaction_threshold: usize,
}

impl SessionManager {
    pub fn new(db: Arc<Mutex<Connection>>, max_messages: usize) -> Self {
        Self {
            sessions: HashMap::new(),
            branch_locks: HashMap::new(),
            db,
            max_messages,
            compaction_threshold: max_messages / 2,
        }
    }

    /// Create a new session for an agent
    pub fn create_session(&mut self, agent_id: &str, branch: Option<&str>) -> Result<Session, String> {
        // Check for branch lock collision (Claw Code)
        if let Some(b) = branch {
            if self.branch_locks.contains_key(b) {
                let existing_session = self.branch_locks.get(b).unwrap();
                return Err(format!("Branch '{}' is locked by session '{}'", b, existing_session));
            }
        }

        let session = Session {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            created_at: Utc::now(),
            last_active: Utc::now(),
            messages: Vec::new(),
            branch_lock: branch.map(|b| b.to_string()),
            status: SessionStatus::Active,
            compaction_count: 0,
        };

        // Register branch lock
        if let Some(b) = branch {
            self.branch_locks.insert(b.to_string(), session.id.clone());
        }

        // Save to database
        self.save_session(&session)?;

        self.sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    /// Add a message to a session
    pub fn add_message(&mut self, session_id: &str, role: MessageRole, content: &str) -> Result<(), String> {
        // Get session first to check if it exists
        if !self.sessions.contains_key(session_id) {
            return Err(format!("Session '{}' not found", session_id));
        }

        let message = SessionMessage {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content: content.to_string(),
            timestamp: Utc::now(),
            compressed: false,
        };

        // Get mutable session and add message
        let needs_compaction = {
            let session = self.sessions.get_mut(session_id).unwrap();
            session.messages.push(message);
            session.last_active = Utc::now();
            session.messages.len() > self.compaction_threshold
        };

        // Compact if needed (separate borrow)
        if needs_compaction {
            self.compact_session_internal(session_id)?;
        }

        // Save to database (separate borrow)
        let session = self.sessions.get(session_id).unwrap();
        self.save_session(session)?;
        Ok(())
    }

    /// Compact a session (Claw Code inspired)
    /// Compresses older messages to reduce context window usage
    pub fn compact_session(&mut self, session_id: &str) -> Result<(), String> {
        self.compact_session_internal(session_id)?;

        // Save to database
        let session = self.sessions.get(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;
        self.save_session(session)?;
        Ok(())
    }

    /// Internal compaction logic
    fn compact_session_internal(&mut self, session_id: &str) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| format!("Session '{}' not found", session_id))?;

        if session.messages.len() <= self.compaction_threshold {
            return Ok(());
        }

        // Keep the last N messages uncompressed
        let keep_count = self.compaction_threshold / 2;
        let total = session.messages.len();

        // Compress older messages
        for i in 0..(total - keep_count) {
            session.messages[i].compressed = true;
            // In a real implementation, we would compress the content here
            // For now, just mark as compressed
        }

        session.compaction_count += 1;
        session.status = SessionStatus::Compacted;

        println!("Session '{}' compacted (count: {})", session_id, session.compaction_count);
        Ok(())
    }

    /// Save session to database
    fn save_session(&self, session: &Session) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let messages_json = serde_json::to_string(&session.messages)
            .map_err(|e| format!("Failed to serialize messages: {}", e))?;

        db.execute(
            "INSERT OR REPLACE INTO claw_sessions (id, agent_id, created_at, last_active, messages, branch_lock, status, compaction_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session.id,
                session.agent_id,
                session.created_at.to_rfc3339(),
                session.last_active.to_rfc3339(),
                messages_json,
                session.branch_lock,
                serde_json::to_string(&session.status).map_err(|e| e.to_string())?,
                session.compaction_count,
            ],
        ).map_err(|e| format!("Failed to save session: {}", e))?;

        Ok(())
    }

    /// Load session from database
    pub fn load_session(&mut self, session_id: &str) -> Result<Option<Session>, String> {
        // Check in-memory cache first
        if let Some(session) = self.sessions.get(session_id) {
            return Ok(Some(session.clone()));
        }

        let db = self.db.lock().map_err(|e| e.to_string())?;

        let session = db.query_row(
            "SELECT id, agent_id, created_at, last_active, messages, branch_lock, status, compaction_count
             FROM claw_sessions WHERE id = ?1",
            params![session_id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_active: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    messages: serde_json::from_str(&row.get::<_, String>(4)?)
                        .unwrap_or_else(|_| Vec::new()),
                    branch_lock: row.get(5)?,
                    status: serde_json::from_str(&row.get::<_, String>(6)?)
                        .unwrap_or(SessionStatus::Active),
                    compaction_count: row.get(7)?,
                })
            },
        ).optional().map_err(|e| format!("Failed to load session: {}", e))?;

        if let Some(s) = &session {
            self.sessions.insert(s.id.clone(), s.clone());
        }

        Ok(session)
    }

    /// End a session and release branch lock
    pub fn end_session(&mut self, session_id: &str) -> Result<(), String> {
        // Get branch lock before modifying
        let branch_lock = self.sessions.get(session_id)
            .and_then(|s| s.branch_lock.clone());

        // Update session status
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.status = SessionStatus::Completed;
        } else {
            return Err(format!("Session '{}' not found", session_id));
        }

        // Release branch lock (separate operation)
        if let Some(branch) = branch_lock {
            self.branch_locks.remove(&branch);
            println!("Branch lock '{}' released", branch);
        }

        // Save to database
        let session = self.sessions.get(session_id).unwrap();
        self.save_session(session)?;
        Ok(())
    }

    /// Check if a branch has collision
    pub fn detect_collision(&self, branch: &str) -> bool {
        self.branch_locks.contains_key(branch)
    }

    /// Get active sessions for an agent
    pub fn get_agent_sessions(&self, agent_id: &str) -> Vec<&Session> {
        self.sessions.values()
            .filter(|s| s.agent_id == agent_id && s.status == SessionStatus::Active)
            .collect()
    }

    /// Get all sessions
    pub fn list_sessions(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }

    /// Initialize database tables for sessions (renamed to claw_sessions to avoid collision with database.rs)
    pub fn init_database(db: &Connection) -> Result<(), String> {
        db.execute(
            "CREATE TABLE IF NOT EXISTS claw_sessions (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_active TEXT NOT NULL,
                messages TEXT NOT NULL,
                branch_lock TEXT,
                status TEXT NOT NULL,
                compaction_count INTEGER DEFAULT 0
            )",
            [],
        ).map_err(|e| format!("Failed to create claw_sessions table: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_lock_collision() {
        // Create in-memory database
        let conn = Connection::open_in_memory().unwrap();
        SessionManager::init_database(&conn).unwrap();
        let db = Arc::new(Mutex::new(conn));

        let mut manager = SessionManager::new(db, 100);

        // Create session with branch lock
        let session1 = manager.create_session("agent-1", Some("branch-1")).unwrap();
        assert!(session1.branch_lock.is_some());

        // Attempt to create another session with same branch
        let result = manager.create_session("agent-2", Some("branch-1"));
        assert!(result.is_err());

        // End first session
        manager.end_session(&session1.id).unwrap();

        // Now should be able to create session with same branch
        let session2 = manager.create_session("agent-3", Some("branch-1")).unwrap();
        assert!(session2.branch_lock.is_some());
    }

    #[test]
    fn test_session_compaction() {
        let conn = Connection::open_in_memory().unwrap();
        SessionManager::init_database(&conn).unwrap();
        let db = Arc::new(Mutex::new(conn));

        let mut manager = SessionManager::new(db, 10);

        let session = manager.create_session("agent-1", None).unwrap();

        // Add messages beyond threshold
        for i in 0..15 {
            manager.add_message(&session.id, MessageRole::User, &format!("Message {}", i)).unwrap();
        }

        let loaded = manager.sessions.get(&session.id).unwrap();
        assert!(loaded.compaction_count > 0);
    }
}