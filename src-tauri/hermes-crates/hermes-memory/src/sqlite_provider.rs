//! SQLite memory provider for structured storage

use crate::{MemoryError, MemoryProvider};
use crate::hybrid_search::{SearchResult, SearchMode};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Memory scope levels for hierarchical storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryScope {
    Global,    // Shared across all agents
    Agent,     // Specific to an agent type
    Session,   // Specific to a session
    Task,      // Specific to a task execution
}

impl MemoryScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryScope::Global => "global",
            MemoryScope::Agent => "agent",
            MemoryScope::Session => "session",
            MemoryScope::Task => "task",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, MemoryError> {
        match s {
            "global" => Ok(MemoryScope::Global),
            "agent" => Ok(MemoryScope::Agent),
            "session" => Ok(MemoryScope::Session),
            "task" => Ok(MemoryScope::Task),
            other => Err(MemoryError::InvalidScope(other.to_string())),
        }
    }
}

/// Memory record stored in SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub scope: MemoryScope,
    pub scope_id: Option<String>,
    pub content: String,
    pub tags: Vec<String>,
    pub importance: f64,
    pub access_count: u32,
    pub embedding_id: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// SQLite-based memory provider
pub struct SqliteMemoryProvider {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl SqliteMemoryProvider {
    /// Create a new SQLite memory provider
    pub fn new(db_path: PathBuf) -> Result<Self, MemoryError> {
        // Create database directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MemoryError::Sqlite(e.to_string()))?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        // Initialize tables
        Self::init_tables(&conn)?;

        // Enable WAL mode
        let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .unwrap_or_else(|_| "wal".to_string());

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Initialize memory tables
    fn init_tables(conn: &Connection) -> Result<(), MemoryError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                scope TEXT NOT NULL,
                scope_id TEXT,
                content TEXT NOT NULL,
                tags TEXT,
                importance REAL DEFAULT 0.5,
                access_count INTEGER DEFAULT 0,
                embedding_id TEXT,
                metadata_json TEXT,
                created_at TEXT NOT NULL,
                last_accessed TEXT,
                expires_at TEXT
            )",
            [],
        ).map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        // Create indexes
        conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_scope ON memories(scope)", [])
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_tags ON memories(tags)", [])
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance)", [])
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        // FTS5 for full-text search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                content,
                tags,
                scope_id,
                content='memories'
            )",
            [],
        ).map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        Ok(())
    }

    /// Get database path
    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Full-text search using FTS5
    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, MemoryError> {
        let conn = self.conn.lock().unwrap();

        let sql = format!(
            "SELECT m.id, m.content, m.tags, m.importance, m.created_at
             FROM memories m
             JOIN memories_fts fts ON m.id = fts.rowid
             WHERE memories_fts MATCH ?
             ORDER BY m.importance DESC
             LIMIT {}",
            limit
        );

        let results = conn.prepare(&sql)
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?
            .query_map(params![query], |row| {
                Ok(SearchResult {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    relevance_score: row.get::<_, f64>(3)?,
                    source: "fts".to_string(),
                })
            })
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        Ok(results)
    }
}

#[async_trait]
impl MemoryProvider for SqliteMemoryProvider {
    async fn store(&self, record: &MemoryRecord) -> Result<String, MemoryError> {
        let id = record.id.clone();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO memories (
                id, scope, scope_id, content, tags, importance, access_count,
                embedding_id, metadata_json, created_at, last_accessed, expires_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                id,
                record.scope.as_str(),
                record.scope_id,
                record.content,
                serde_json::to_string(&record.tags).unwrap_or_default(),
                record.importance,
                record.access_count as i32,
                record.embedding_id,
                record.metadata_json,
                record.created_at.to_rfc3339(),
                record.last_accessed.map(|t| t.to_rfc3339()),
                record.expires_at.map(|t| t.to_rfc3339()),
            ],
        ).map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        Ok(id)
    }

    async fn retrieve_by_scope(&self, scope: MemoryScope, limit: usize) -> Result<Vec<MemoryRecord>, MemoryError> {
        let conn = self.conn.lock().unwrap();

        let sql = format!(
            "SELECT id, scope, scope_id, content, tags, importance, access_count,
                    embedding_id, metadata_json, created_at, last_accessed, expires_at
             FROM memories WHERE scope = ?1
             ORDER BY importance DESC, created_at DESC
             LIMIT {}",
            limit
        );

        let records = conn.prepare(&sql)
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?
            .query_map(params![scope.as_str()], |row| {
                let tags_str: String = row.get(4)?;
                let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

                Ok(MemoryRecord {
                    id: row.get(0)?,
                    scope: MemoryScope::from_str(&row.get::<_, String>(1)?).unwrap_or(MemoryScope::Global),
                    scope_id: row.get(2)?,
                    content: row.get(3)?,
                    tags,
                    importance: row.get(5)?,
                    access_count: row.get::<_, i32>(6)? as u32,
                    embedding_id: row.get(7)?,
                    metadata_json: row.get(8)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    last_accessed: row.get::<_, Option<String>>(10)?.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).ok()
                    }),
                    expires_at: row.get::<_, Option<String>>(11)?.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)).ok()
                    }),
                })
            })
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?;

        Ok(records)
    }

    async fn search(&self, query: &str, mode: SearchMode, limit: usize) -> Result<Vec<SearchResult>, MemoryError> {
        match mode {
            SearchMode::Keyword => self.search_fts(query, limit),
            SearchMode::Semantic => {
                // Semantic search requires Chroma, return empty for SQLite-only provider
                Ok(vec![])
            }
            SearchMode::Hybrid => {
                // Combine FTS results with semantic (if available)
                self.search_fts(query, limit)
            }
        }
    }

    async fn delete(&self, id: &str) -> Result<(), MemoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memories WHERE id = ?1", params![id])
            .map_err(|e| MemoryError::Sqlite(e.to_string()))?;
        Ok(())
    }

    async fn touch(&self, id: &str) -> Result<(), MemoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE memories SET access_count = access_count + 1, last_accessed = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), id],
        ).map_err(|e| MemoryError::Sqlite(e.to_string()))?;
        Ok(())
    }
}