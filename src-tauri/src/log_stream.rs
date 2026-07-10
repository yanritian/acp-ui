use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// Log types for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogType {
    Compile,
    Debug,
    Network,
    Error,
    Info,
    Warning,
    System,
}

impl LogType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogType::Compile => "compile",
            LogType::Debug => "debug",
            LogType::Network => "network",
            LogType::Error => "error",
            LogType::Info => "info",
            LogType::Warning => "warning",
            LogType::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "compile" => LogType::Compile,
            "debug" => LogType::Debug,
            "network" => LogType::Network,
            "error" => LogType::Error,
            "warning" => LogType::Warning,
            "system" => LogType::System,
            _ => LogType::Info,
        }
    }
}

/// Single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: String,
    pub agent_id: String,
    pub log_type: LogType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,           // stdout or stderr
    pub metadata: Option<String>, // JSON string for additional metadata
}

/// Circular buffer for recent logs (memory cache)
pub struct CircularBuffer {
    buffer: VecDeque<LogEntry>,
    max_size: usize,
}

impl CircularBuffer {
    pub fn new(max_size: usize) -> Self {
        // Enforce minimum size of 1 to prevent infinite loop in push()
        let safe_size = if max_size == 0 { 1 } else { max_size };
        Self {
            buffer: VecDeque::with_capacity(safe_size),
            max_size: safe_size,
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        // Guard against edge case (already handled in new(), but belt-and-suspenders)
        if self.max_size == 0 {
            return;
        }
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(entry);
    }

    pub fn get_all(&self) -> Vec<LogEntry> {
        self.buffer.iter().cloned().collect()
    }

    pub fn get_by_agent(&self, agent_id: &str) -> Vec<LogEntry> {
        self.buffer
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect()
    }

    pub fn get_by_type(&self, log_type: LogType) -> Vec<LogEntry> {
        self.buffer
            .iter()
            .filter(|e| e.log_type == log_type)
            .cloned()
            .collect()
    }

    pub fn get_latest(&self, count: usize) -> Vec<LogEntry> {
        let start = self.buffer.len().saturating_sub(count);
        self.buffer.iter().skip(start).cloned().collect()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Log classifier - analyzes log content to determine type
pub struct LogClassifier {
    compile_patterns: Vec<String>,
    debug_patterns: Vec<String>,
    network_patterns: Vec<String>,
    error_patterns: Vec<String>,
    warning_patterns: Vec<String>,
}

impl LogClassifier {
    pub fn new() -> Self {
        Self {
            compile_patterns: vec![
                "compiling".to_string(),
                "building".to_string(),
                "tsc".to_string(),
                "webpack".to_string(),
                "vite".to_string(),
                "esbuild".to_string(),
                "error TS".to_string(),
                "error:".to_string(),
                "failed to compile".to_string(),
                "build failed".to_string(),
                "npm run build".to_string(),
                "cargo build".to_string(),
                "gradle".to_string(),
                "make".to_string(),
                ".ts:".to_string(),
                ".tsx:".to_string(),
                ".js:".to_string(),
                ".jsx:".to_string(),
                ".vue:".to_string(),
            ],
            debug_patterns: vec![
                "debug:".to_string(),
                "dbg:".to_string(),
                "breakpoint".to_string(),
                "console.log".to_string(),
                "console.debug".to_string(),
                "inspect".to_string(),
                "debugger".to_string(),
                "step".to_string(),
                "trace".to_string(),
            ],
            network_patterns: vec![
                "http://".to_string(),
                "https://".to_string(),
                "ws://".to_string(),
                "wss://".to_string(),
                "api".to_string(),
                "request".to_string(),
                "response".to_string(),
                "fetch".to_string(),
                "axios".to_string(),
                "xhr".to_string(),
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "status:".to_string(),
                "status code".to_string(),
                "network".to_string(),
                "connection".to_string(),
                "timeout".to_string(),
            ],
            error_patterns: vec![
                "error:".to_string(),
                "exception".to_string(),
                "failed".to_string(),
                "crash".to_string(),
                "fatal".to_string(),
                "panic".to_string(),
                "assertion".to_string(),
                "null".to_string(),
                "undefined is not".to_string(),
                "cannot".to_string(),
                "ENOENT".to_string(),
                "ENOTFOUND".to_string(),
                "ECONNREFUSED".to_string(),
                "ETIMEDOUT".to_string(),
            ],
            warning_patterns: vec![
                "warn:".to_string(),
                "warning:".to_string(),
                "deprecated".to_string(),
                "caution".to_string(),
                "note:".to_string(),
                "consider".to_string(),
                "suggest".to_string(),
                "might".to_string(),
                "potential".to_string(),
            ],
        }
    }

    /// Classify a log line based on content patterns
    pub fn classify(&self, content: &str, source: &str) -> LogType {
        let lower = content.to_lowercase();

        // Check error patterns first (highest priority)
        for pattern in &self.error_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                return LogType::Error;
            }
        }

        // Check warning patterns
        for pattern in &self.warning_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                return LogType::Warning;
            }
        }

        // Check compile patterns
        for pattern in &self.compile_patterns {
            if lower.contains(&pattern.to_lowercase()) || content.contains(pattern) {
                return LogType::Compile;
            }
        }

        // Check network patterns
        for pattern in &self.network_patterns {
            if lower.contains(&pattern.to_lowercase()) || content.contains(pattern) {
                return LogType::Network;
            }
        }

        // Check debug patterns
        for pattern in &self.debug_patterns {
            if lower.contains(&pattern.to_lowercase()) {
                return LogType::Debug;
            }
        }

        // Default classification based on source
        if source == "stderr" {
            LogType::Error
        } else {
            LogType::Info
        }
    }
}

impl Default for LogClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// LogStreamManager - manages real-time log capture, classification, and storage
pub struct LogStreamManager {
    buffer: Arc<RwLock<CircularBuffer>>,
    classifier: LogClassifier,
    db_conn: Arc<std::sync::Mutex<Connection>>,
    subscribers: Arc<RwLock<Vec<String>>>, // agent_ids that want real-time push
    batch_queue: Arc<RwLock<Vec<LogEntry>>>,
    app_handle: Option<AppHandle>,
}

impl LogStreamManager {
    pub fn new(db_conn: Arc<std::sync::Mutex<Connection>>, buffer_size: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(CircularBuffer::new(buffer_size))),
            classifier: LogClassifier::new(),
            db_conn,
            subscribers: Arc::new(RwLock::new(Vec::new())),
            batch_queue: Arc::new(RwLock::new(Vec::new())),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    /// Capture a log line from agent stdout/stderr
    pub fn capture_log(&self, agent_id: &str, content: &str, source: &str) -> LogEntry {
        let log_type = self.classifier.classify(content, source);
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let entry = LogEntry {
            id,
            agent_id: agent_id.to_string(),
            log_type,
            content: content.to_string(),
            timestamp,
            source: source.to_string(),
            metadata: None,
        };

        // Add to circular buffer
        self.buffer.write().push(entry.clone());

        // Add to batch queue for WebSocket push
        self.batch_queue.write().push(entry.clone());

        // Persist to SQLite
        self.save_to_db(&entry);

        entry
    }

    /// Save log entry to SQLite database
    fn save_to_db(&self, entry: &LogEntry) {
        let conn = self.db_conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO logs (id, agent_id, type, content, timestamp, source, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.id,
                entry.agent_id,
                entry.log_type.as_str(),
                entry.content,
                entry.timestamp.to_rfc3339(),
                entry.source,
                entry.metadata,
            ],
        );
    }

    /// Subscribe to real-time log push for an agent
    pub fn subscribe(&self, agent_id: &str) {
        let mut subs = self.subscribers.write();
        if !subs.contains(&agent_id.to_string()) {
            subs.push(agent_id.to_string());
        }
    }

    /// Unsubscribe from real-time log push
    pub fn unsubscribe(&self, agent_id: &str) {
        let mut subs = self.subscribers.write();
        subs.retain(|s| s != agent_id);
    }

    /// Get subscribers list
    pub fn get_subscribers(&self) -> Vec<String> {
        self.subscribers.read().clone()
    }

    /// Flush batch queue and emit WebSocket event
    pub fn flush_batch(&self) -> Vec<LogEntry> {
        let mut queue = self.batch_queue.write();
        let batch: Vec<LogEntry> = queue.drain(..).collect();

        if !batch.is_empty() {
            if let Some(handle) = self.app_handle.as_ref() {
                let _ = handle.emit("log-batch", batch.clone());
            }
        }

        batch
    }

    /// Get all logs from buffer
    pub fn get_all_logs(&self) -> Vec<LogEntry> {
        self.buffer.read().get_all()
    }

    /// Get logs for specific agent
    pub fn get_logs_by_agent(&self, agent_id: &str) -> Vec<LogEntry> {
        self.buffer.read().get_by_agent(agent_id)
    }

    /// Get logs by type
    pub fn get_logs_by_type(&self, log_type: LogType) -> Vec<LogEntry> {
        self.buffer.read().get_by_type(log_type)
    }

    /// Get latest N logs
    pub fn get_latest_logs(&self, count: usize) -> Vec<LogEntry> {
        self.buffer.read().get_latest(count)
    }

    /// Search logs from database (escapes SQL LIKE wildcards)
    pub fn search_logs(&self, keyword: &str, limit: Option<u64>) -> Vec<LogEntry> {
        let conn = self.db_conn.lock().unwrap();

        // Escape SQL LIKE wildcards to prevent unintended matches
        // % matches any sequence, _ matches any single character
        let escaped_keyword = keyword
            .replace('\\', "\\\\") // Escape backslash first
            .replace('%', "\\%") // Escape percent
            .replace('_', "\\_"); // Escape underscore
        let pattern = format!("%{}%", escaped_keyword);

        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let sql = format!(
            "SELECT id, agent_id, type, content, timestamp, source, metadata
             FROM logs WHERE content LIKE ?1 ESCAPE '\\'
             ORDER BY timestamp DESC {}",
            limit_clause
        );

        let entries: Vec<LogEntry> = conn
            .prepare(&sql)
            .ok()
            .and_then(|mut stmt| {
                stmt.query_map([&pattern], |row| {
                    let type_str: String = row.get(2)?;
                    let timestamp_str: String = row.get(4)?;
                    Ok(LogEntry {
                        id: row.get(0)?,
                        agent_id: row.get(1)?,
                        log_type: LogType::from_str(&type_str),
                        content: row.get(3)?,
                        timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                        source: row.get(5)?,
                        metadata: row.get(6)?,
                    })
                })
                .ok()
                .and_then(|rows| rows.collect::<Result<Vec<_>, _>>().ok())
            })
            .unwrap_or_default();

        entries
    }

    /// Get logs from database with filters
    pub fn get_logs_from_db(
        &self,
        agent_id: Option<&str>,
        log_type: Option<LogType>,
        limit: Option<u64>,
    ) -> Vec<LogEntry> {
        let conn = self.db_conn.lock().unwrap();
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let sql = match (&agent_id, &log_type) {
            (Some(_aid), Some(_lt)) => format!(
                "SELECT id, agent_id, type, content, timestamp, source, metadata
                 FROM logs WHERE agent_id = ?1 AND type = ?2
                 ORDER BY timestamp DESC {}",
                limit_clause
            ),
            (Some(_aid), None) => format!(
                "SELECT id, agent_id, type, content, timestamp, source, metadata
                 FROM logs WHERE agent_id = ?1
                 ORDER BY timestamp DESC {}",
                limit_clause
            ),
            (None, Some(_lt)) => format!(
                "SELECT id, agent_id, type, content, timestamp, source, metadata
                 FROM logs WHERE type = ?1
                 ORDER BY timestamp DESC {}",
                limit_clause
            ),
            (None, None) => format!(
                "SELECT id, agent_id, type, content, timestamp, source, metadata
                 FROM logs ORDER BY timestamp DESC {}",
                limit_clause
            ),
        };

        let entries: Vec<LogEntry> = match (&agent_id, &log_type) {
            (Some(aid), Some(lt)) => conn
                .prepare(&sql)
                .ok()
                .and_then(|mut stmt| {
                    stmt.query_map(params![aid, lt.as_str()], |row| {
                        let type_str: String = row.get(2)?;
                        let timestamp_str: String = row.get(4)?;
                        Ok(LogEntry {
                            id: row.get(0)?,
                            agent_id: row.get(1)?,
                            log_type: LogType::from_str(&type_str),
                            content: row.get(3)?,
                            timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                            source: row.get(5)?,
                            metadata: row.get(6)?,
                        })
                    })
                    .ok()
                    .and_then(|rows| rows.collect::<Result<Vec<_>, _>>().ok())
                })
                .unwrap_or_default(),
            (Some(aid), None) => conn
                .prepare(&sql)
                .ok()
                .and_then(|mut stmt| {
                    stmt.query_map(params![aid], |row| {
                        let type_str: String = row.get(2)?;
                        let timestamp_str: String = row.get(4)?;
                        Ok(LogEntry {
                            id: row.get(0)?,
                            agent_id: row.get(1)?,
                            log_type: LogType::from_str(&type_str),
                            content: row.get(3)?,
                            timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                            source: row.get(5)?,
                            metadata: row.get(6)?,
                        })
                    })
                    .ok()
                    .and_then(|rows| rows.collect::<Result<Vec<_>, _>>().ok())
                })
                .unwrap_or_default(),
            (None, Some(lt)) => conn
                .prepare(&sql)
                .ok()
                .and_then(|mut stmt| {
                    stmt.query_map(params![lt.as_str()], |row| {
                        let type_str: String = row.get(2)?;
                        let timestamp_str: String = row.get(4)?;
                        Ok(LogEntry {
                            id: row.get(0)?,
                            agent_id: row.get(1)?,
                            log_type: LogType::from_str(&type_str),
                            content: row.get(3)?,
                            timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                            source: row.get(5)?,
                            metadata: row.get(6)?,
                        })
                    })
                    .ok()
                    .and_then(|rows| rows.collect::<Result<Vec<_>, _>>().ok())
                })
                .unwrap_or_default(),
            (None, None) => conn
                .prepare(&sql)
                .ok()
                .and_then(|mut stmt| {
                    stmt.query_map([], |row| {
                        let type_str: String = row.get(2)?;
                        let timestamp_str: String = row.get(4)?;
                        Ok(LogEntry {
                            id: row.get(0)?,
                            agent_id: row.get(1)?,
                            log_type: LogType::from_str(&type_str),
                            content: row.get(3)?,
                            timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                                .map(|dt| dt.with_timezone(&Utc))
                                .unwrap_or_else(|_| Utc::now()),
                            source: row.get(5)?,
                            metadata: row.get(6)?,
                        })
                    })
                    .ok()
                    .and_then(|rows| rows.collect::<Result<Vec<_>, _>>().ok())
                })
                .unwrap_or_default(),
        };

        entries
    }

    /// Clear logs from buffer (not database)
    pub fn clear_buffer(&self) {
        self.buffer.write().clear();
    }

    /// Delete old logs from database (cleanup)
    pub fn cleanup_old_logs(&self, days_to_keep: u64) -> Result<u64, String> {
        let conn = self.db_conn.lock().unwrap();
        let cutoff = Utc::now() - chrono::Duration::days(days_to_keep as i64);

        let deleted = conn
            .execute(
                "DELETE FROM logs WHERE timestamp < ?1",
                params![cutoff.to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;

        Ok(deleted as u64)
    }
}

/// Initialize logs table in database
pub fn init_logs_table(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            type TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            source TEXT NOT NULL,
            metadata TEXT
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_logs_agent ON logs(agent_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_logs_type ON logs(type)", [])
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
