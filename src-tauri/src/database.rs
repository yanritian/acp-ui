use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

/// Task status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Success => "success",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "running" => Ok(TaskStatus::Running),
            "success" => Ok(TaskStatus::Success),
            "failed" => Ok(TaskStatus::Failed),
            "cancelled" => Ok(TaskStatus::Cancelled),
            other => Err(format!("Unknown status: {}", other)),
        }
    }
}

/// Task record for history storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub agents: Vec<AgentExecution>,
}

/// Agent execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    pub agent_id: String,
    pub agent_name: String,
    pub status: TaskStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
}

/// Statistics for task history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatistics {
    pub total_tasks: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub running_tasks: u64,
    pub pending_tasks: u64,
    pub success_rate: f64,
    pub average_duration_ms: f64,
}

/// SQLite database manager for history storage
pub struct DatabaseManager {
    pub conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl DatabaseManager {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let db_path = get_db_path(app)?;

        // Create database directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

        // Initialize tables
        init_tables(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.db_path.clone()
    }

    /// Save a task record
    pub fn save_task(&self, task: &TaskRecord) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO tasks (
                id, name, status, source, created_at, completed_at, error_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                task.id,
                task.name,
                task.status.as_str(),
                task.source,
                task.created_at.to_rfc3339(),
                task.completed_at.map(|t| t.to_rfc3339()),
                task.error_message,
            ],
        ).map_err(|e| e.to_string())?;

        // Save agent executions
        for agent in &task.agents {
            conn.execute(
                "INSERT OR REPLACE INTO agent_executions (
                    task_id, agent_id, agent_name, status, started_at, completed_at, output
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    task.id,
                    agent.agent_id,
                    agent.agent_name,
                    agent.status.as_str(),
                    agent.started_at.to_rfc3339(),
                    agent.completed_at.map(|t| t.to_rfc3339()),
                    agent.output,
                ],
            ).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Load task records with optional filters
    pub fn load_tasks(
        &self,
        status: Option<Vec<TaskStatus>>,
        source: Option<String>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<TaskRecord>, String> {
        let conn = self.conn.lock().unwrap();

        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
        let offset_clause = offset.map(|o| format!("OFFSET {}", o)).unwrap_or_default();

        let (query, params): (String, Vec<Box<dyn rusqlite::ToSql>>) = match (&status, &source) {
            (Some(statuses), Some(src)) => {
                let placeholders = (1..=statuses.len())
                    .map(|i| format!("?{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                let status_strs: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
                let q = format!(
                    "SELECT id, name, status, source, created_at, completed_at, error_message
                    FROM tasks WHERE status IN ({}) AND source = ?{} ORDER BY created_at DESC {} {}",
                    placeholders, statuses.len() + 1, limit_clause, offset_clause
                );
                let mut p: Vec<Box<dyn rusqlite::ToSql>> = status_strs.iter().map(|s| Box::new(*s) as Box<dyn rusqlite::ToSql>).collect();
                p.push(Box::new(src.clone()));
                (q, p)
            }
            (Some(statuses), None) => {
                let placeholders = (1..=statuses.len())
                    .map(|i| format!("?{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                let status_strs: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
                let q = format!(
                    "SELECT id, name, status, source, created_at, completed_at, error_message
                    FROM tasks WHERE status IN ({}) ORDER BY created_at DESC {} {}",
                    placeholders, limit_clause, offset_clause
                );
                let p: Vec<Box<dyn rusqlite::ToSql>> = status_strs.iter().map(|s| Box::new(*s) as Box<dyn rusqlite::ToSql>).collect();
                (q, p)
            }
            (None, Some(src)) => {
                let q = format!(
                    "SELECT id, name, status, source, created_at, completed_at, error_message
                    FROM tasks WHERE source = ?1 ORDER BY created_at DESC {} {}",
                    limit_clause, offset_clause
                );
                (q, vec![Box::new(src.clone()) as Box<dyn rusqlite::ToSql>])
            }
            (None, None) => {
                let q = format!(
                    "SELECT id, name, status, source, created_at, completed_at, error_message
                    FROM tasks ORDER BY created_at DESC {} {}",
                    limit_clause, offset_clause
                );
                (q, vec![])
            }
        };

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let tasks = stmt
            .query_map(rusqlite::params_from_iter(params_ref.iter()), |row| {
                let status_str: String = row.get(2)?;
                let created_at_str: String = row.get(4)?;
                let completed_at_str: Option<String> = row.get(5)?;

                Ok(TaskRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
                    source: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    completed_at: completed_at_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    error_message: row.get(6)?,
                    agents: vec![], // Loaded separately
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Load agent executions for each task
        let mut result = Vec::new();
        for task in tasks {
            let agents = load_agent_executions_internal(&conn, &task.id)?;
            result.push(TaskRecord { agents, ..task });
        }

        Ok(result)
    }

    /// Get a single task by ID
    pub fn get_task(&self, task_id: &str) -> Result<Option<TaskRecord>, String> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, status, source, created_at, completed_at, error_message
            FROM tasks WHERE id = ?1"
        ).map_err(|e| e.to_string())?;

        let mut rows = stmt.query(params![task_id]).map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let status_str: String = row.get(2).map_err(|e| e.to_string())?;
            let created_at_str: String = row.get(4).map_err(|e| e.to_string())?;
            let completed_at_str: Option<String> = row.get(5).map_err(|e| e.to_string())?;

            let task = TaskRecord {
                id: row.get(0).map_err(|e| e.to_string())?,
                name: row.get(1).map_err(|e| e.to_string())?,
                status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
                source: row.get(3).map_err(|e| e.to_string())?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                completed_at: completed_at_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                error_message: row.get(6).map_err(|e| e.to_string())?,
                agents: load_agent_executions_internal(&conn, task_id)?,
            };

            return Ok(Some(task));
        }

        Ok(None)
    }

    /// Delete a task record
    pub fn delete_task(&self, task_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();

        // Delete agent executions first
        conn.execute(
            "DELETE FROM agent_executions WHERE task_id = ?1",
            params![task_id],
        ).map_err(|e| e.to_string())?;

        // Delete task
        conn.execute(
            "DELETE FROM tasks WHERE id = ?1",
            params![task_id],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Search tasks by keyword (uses SQLite LIKE)
    pub fn search_tasks(&self, keyword: &str, limit: Option<u64>) -> Result<Vec<TaskRecord>, String> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", keyword);

        let query = format!(
            "SELECT id, name, status, source, created_at, completed_at, error_message
            FROM tasks WHERE name LIKE ?1 OR source LIKE ?1 OR error_message LIKE ?1
            ORDER BY created_at DESC {}",
            limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default()
        );

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let tasks = stmt
            .query_map([&pattern], |row| {
                let status_str: String = row.get(2)?;
                let created_at_str: String = row.get(4)?;
                let completed_at_str: Option<String> = row.get(5)?;

                Ok(TaskRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
                    source: row.get(3)?,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    completed_at: completed_at_str.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    error_message: row.get(6)?,
                    agents: vec![],
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for task in tasks {
            let agents = load_agent_executions_internal(&conn, &task.id)?;
            result.push(TaskRecord { agents, ..task });
        }

        Ok(result)
    }

    /// Get task statistics
    pub fn get_statistics(&self) -> Result<TaskStatistics, String> {
        let conn = self.conn.lock().unwrap();

        let total: u64 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let successful: u64 = conn
            .query_row("SELECT COUNT(*) FROM tasks WHERE status = 'success'", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let failed: u64 = conn
            .query_row("SELECT COUNT(*) FROM tasks WHERE status = 'failed'", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let running: u64 = conn
            .query_row("SELECT COUNT(*) FROM tasks WHERE status = 'running'", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        let pending: u64 = conn
            .query_row("SELECT COUNT(*) FROM tasks WHERE status = 'pending'", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        // Calculate average duration for completed tasks
        let avg_duration_ms: f64 = conn
            .query_row(
                "SELECT AVG((strftime('%s', completed_at) - strftime('%s', created_at)) * 1000)
                FROM tasks WHERE completed_at IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(TaskStatistics {
            total_tasks: total,
            successful_tasks: successful,
            failed_tasks: failed,
            running_tasks: running,
            pending_tasks: pending,
            success_rate,
            average_duration_ms: avg_duration_ms,
        })
    }
}

/// Internal function to load agent executions (used when connection is already locked)
fn load_agent_executions_internal(conn: &Connection, task_id: &str) -> Result<Vec<AgentExecution>, String> {
    let mut stmt = conn.prepare(
        "SELECT agent_id, agent_name, status, started_at, completed_at, output
        FROM agent_executions WHERE task_id = ?1 ORDER BY started_at"
    ).map_err(|e| e.to_string())?;

    let agents = stmt
        .query_map(params![task_id], |row| {
            let status_str: String = row.get(2)?;
            let started_at_str: String = row.get(3)?;
            let completed_at_str: Option<String> = row.get(4)?;

            Ok(AgentExecution {
                agent_id: row.get(0)?,
                agent_name: row.get(1)?,
                status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
                started_at: DateTime::parse_from_rfc3339(&started_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                completed_at: completed_at_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                output: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(agents)
}

fn get_db_path(_app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(desktop)]
    {
        dirs::data_local_dir()
            .map(|p| p.join("acp-ui").join("history.db"))
            .ok_or_else(|| "Could not find data directory".to_string())
    }
    #[cfg(not(desktop))]
    {
        _app
            .path()
            .app_data_dir()
            .map_err(|e| format!("Could not resolve app data dir: {}", e))
            .map(|p| p.join("history.db"))
    }
}

fn init_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            status TEXT NOT NULL,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL,
            completed_at TEXT,
            error_message TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_executions (
            task_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            agent_name TEXT NOT NULL,
            status TEXT NOT NULL,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            output TEXT,
            PRIMARY KEY (task_id, agent_id),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_created ON tasks(created_at)",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_source ON tasks(source)",
        [],
    ).map_err(|e| e.to_string())?;

    // Sessions table for multi-session persistence
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            agent_name TEXT NOT NULL,
            acp_session_id TEXT,
            cwd TEXT NOT NULL,
            title TEXT,
            status TEXT NOT NULL DEFAULT 'disconnected',
            created_at TEXT NOT NULL,
            last_active TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Memories table for cross-session memory
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            agent_id TEXT,
            session_id TEXT,
            content TEXT NOT NULL,
            tags TEXT,
            importance REAL DEFAULT 0.5,
            created_at TEXT NOT NULL,
            last_accessed TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_agent ON memories(agent_id)",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_memories_tags ON memories(tags)",
        [],
    ).map_err(|e| e.to_string())?;

    // Enable WAL mode for better concurrent read performance
    conn.execute("PRAGMA journal_mode=WAL", []).map_err(|e| e.to_string())?;

    Ok(())
}