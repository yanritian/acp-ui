mod agent;
mod config;
mod database;
mod gateway_config;
mod websocket;
mod tunnel;
mod log_stream;
mod permission_checker;
mod agent_config_parser;
mod mcp_manager;
mod hooks_executor;
mod session_manager;  // NEW: Claw Code Session Management
mod event_router;     // NEW: Claw Code Event Router (clawhip layer)
mod feishu_rich_message;  // NEW: Feishu rich message support (images, files, cards)
mod executive_agent;  // NEW: Executive Agent - executes actual development tasks
mod agent_registry;   // NEW: Agent Registry - Docker-like Base/Template/Instance system
mod smart_router;     // NEW: Smart Router - three-layer complexity evaluation
mod circuit_breaker;  // NEW: Circuit Breaker - three-state failure protection
mod self_healing;     // NEW: Self-Healing - EWMA anomaly detection
mod team_dag;         // NEW: Team DAG execution engine with SyncPoints

use agent::{AgentInstance, AgentManager, AgentStatus};
use config::{AgentConfig, AgentTransport, AgentsConfig, ConfigManager};
use database::DatabaseManager;
use tunnel::NgrokManager;
use websocket::WebSocketServer;
use log_stream::{LogStreamManager, LogEntry, LogType};
use permission_checker::{PermissionChecker, PermissionConfig, PermissionResult};
use agent_config_parser::{AgentConfigParser, AgentConfigParsed, ParseResult};
use executive_agent::{ExecutiveAgentManager, TaskResult, GeneratedFile};
use agent_registry::{AgentRegistry, AgentBase, AgentTemplate};
use smart_router::{TaskAnalyzer, RouteDecision};
use circuit_breaker::{CircuitBreakerManager, CircuitState};
use self_healing::{AnomalyDetector, AnomalyRecord};
use team_dag::{DAGEngine, ExecutionPlan, DAGNode};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use uuid::Uuid;

/// Application state containing all managers
pub struct AppState {
    pub config_manager: Arc<RwLock<Option<ConfigManager>>>,
    pub agent_manager: AgentManager,
    pub database: Arc<Mutex<Option<DatabaseManager>>>,
    pub ws_server: Arc<Mutex<Option<WebSocketServer>>>,
    pub tunnel_manager: Arc<Mutex<Option<NgrokManager>>>,
    pub log_stream_manager: Arc<Mutex<Option<LogStreamManager>>>,
    pub executive_agent_manager: Arc<Mutex<Option<ExecutiveAgentManager>>>,
}

impl AppState {
    fn new(agent_manager: AgentManager) -> Self {
        Self {
            config_manager: Arc::new(RwLock::new(None)),
            agent_manager,
            database: Arc::new(Mutex::new(None)),
            ws_server: Arc::new(Mutex::new(None)),
            tunnel_manager: Arc::new(Mutex::new(None)),
            log_stream_manager: Arc::new(Mutex::new(None)),
            executive_agent_manager: Arc::new(Mutex::new(None)),
        }
    }
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Result<AgentsConfig, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .map(|cm| cm.get_config())
        .ok_or_else(|| "Config manager not initialized".to_string())
}

#[tauri::command]
fn reload_config(state: State<AppState>) -> Result<AgentsConfig, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .map(|cm| cm.reload())
        .ok_or_else(|| "Config manager not initialized".to_string())?
}

#[tauri::command]
fn get_config_path(state: State<AppState>) -> Result<String, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .map(|cm| cm.get_config_path().to_string_lossy().to_string())
        .ok_or_else(|| "Config manager not initialized".to_string())
}

#[tauri::command]
fn spawn_agent(
    name: String,
    state: State<AppState>,
    app_handle: AppHandle,
) -> Result<AgentInstance, String> {
    let config_manager = state.config_manager.read();
    let config = config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .get_config();

    let agent_config = config
        .agents
        .get(&name)
        .ok_or_else(|| format!("Agent '{}' not found in config", name))?;

    state
        .agent_manager
        .spawn_agent(name, agent_config, app_handle)
}

#[tauri::command]
fn send_to_agent(agent_id: String, message: String, state: State<AppState>) -> Result<(), String> {
    state.agent_manager.send_message(&agent_id, &message)
}

#[tauri::command]
fn kill_agent(agent_id: String, state: State<AppState>) -> Result<(), String> {
    state.agent_manager.kill_agent(&agent_id)
}

#[tauri::command]
fn list_running_agents(state: State<AppState>) -> Vec<String> {
    state.agent_manager.list_running_agents()
}

#[tauri::command]
fn get_running_agents_info(state: State<AppState>) -> Vec<AgentInstance> {
    state.agent_manager.get_running_agents_info()
}

#[tauri::command]
fn get_agent_status(agent_id: String, state: State<AppState>) -> Result<AgentStatus, String> {
    state.agent_manager.get_agent_status(&agent_id)
}

#[tauri::command]
fn pause_agent(agent_id: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    state.agent_manager.pause_agent(&agent_id, &app_handle)
}

#[tauri::command]
fn resume_agent(agent_id: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    state.agent_manager.resume_agent(&agent_id, &app_handle)
}

#[tauri::command]
fn inject_message(agent_id: String, message: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    state.agent_manager.inject_message(&agent_id, &message, &app_handle)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn add_agent(
    name: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
    transport: Option<String>,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
    state: State<AppState>,
) -> Result<AgentsConfig, String> {
    let agent_config = build_agent_config(command, args, env, transport, url, headers)?;
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .add_agent(name, agent_config)
}

#[tauri::command]
fn remove_agent(name: String, state: State<AppState>) -> Result<AgentsConfig, String> {
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .remove_agent(&name)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn update_agent(
    name: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
    transport: Option<String>,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
    state: State<AppState>,
) -> Result<AgentsConfig, String> {
    let agent_config = build_agent_config(command, args, env, transport, url, headers)?;
    let config_manager = state.config_manager.read();
    config_manager
        .as_ref()
        .ok_or_else(|| "Config manager not initialized".to_string())?
        .update_agent(name, agent_config)
}

/// Build an `AgentConfig` from the loosely-typed Tauri command arguments,
/// applying validation rules per transport kind.
fn build_agent_config(
    command: Option<String>,
    args: Option<Vec<String>>,
    env: Option<std::collections::HashMap<String, String>>,
    transport: Option<String>,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
) -> Result<AgentConfig, String> {
    let transport_kind = match transport.as_deref() {
        None | Some("") | Some("stdio") => AgentTransport::Stdio,
        Some("websocket") | Some("ws") | Some("wss") => AgentTransport::Websocket,
        Some("http") | Some("https") => AgentTransport::Http,
        Some(other) => return Err(format!("Unknown transport: {}", other)),
    };

    // Defense in depth: stdio agents can't run on mobile (no subprocess).
    // The frontend already filters them out, but reject here too so a
    // malicious renderer or synced config can't smuggle one through the
    // IPC boundary.
    #[cfg(not(desktop))]
    if matches!(transport_kind, AgentTransport::Stdio) {
        return Err("stdio agents are not supported on this platform".to_string());
    }

    match transport_kind {
        AgentTransport::Stdio => {
            let command = command
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "stdio agent requires a command".to_string())?;
            Ok(AgentConfig {
                transport: AgentTransport::Stdio,
                command: Some(command),
                args: Some(args.unwrap_or_default()),
                env: env.unwrap_or_default(),
                url: None,
                headers: None,
                cwd: None,
                mcp_servers: Vec::new(),
                skills: Vec::new(),
                hooks: Vec::new(),
                capabilities: Vec::new(),
            })
        }
        AgentTransport::Websocket | AgentTransport::Http => {
            let url = url
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "remote agent requires a url".to_string())?;
            // Sanity-check scheme matches transport so users get an early error.
            let lower = url.to_ascii_lowercase();
            let scheme_ok = match transport_kind {
                AgentTransport::Websocket => {
                    lower.starts_with("ws://") || lower.starts_with("wss://")
                }
                AgentTransport::Http => {
                    lower.starts_with("http://") || lower.starts_with("https://")
                }
                _ => true,
            };
            if !scheme_ok {
                return Err(format!(
                    "URL scheme does not match transport '{:?}': {}",
                    transport_kind, url
                ));
            }
            let headers = headers.filter(|h| !h.is_empty());
            Ok(AgentConfig {
                transport: transport_kind,
                command: None,
                args: None,
                env: std::collections::HashMap::new(),
                url: Some(url),
                headers,
                cwd: None,
                mcp_servers: Vec::new(),
                skills: Vec::new(),
                hooks: Vec::new(),
                capabilities: Vec::new(),
            })
        }
    }
}

#[tauri::command]
fn get_machine_id() -> Result<String, String> {
    // `machine-uid` is desktop-only (no support for iOS / Android). Telemetry
    // on mobile falls back to an anonymous id (the frontend handles a failure
    // here by leaving `machineId = null`).
    #[cfg(desktop)]
    {
        machine_uid::get().map_err(|e| format!("Failed to get machine ID: {}", e))
    }
    #[cfg(not(desktop))]
    {
        Err("machine id is not available on this platform".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let agent_manager = AgentManager::new();
    let app_state = AppState::new(agent_manager);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .setup(|app| {
            let app_handle = app.handle().clone();
            let state: State<AppState> = app.state();

            // Initialize config manager
            match ConfigManager::new(&app_handle) {
                Ok(cm) => {
                    *state.config_manager.write() = Some(cm);
                }
                Err(e) => {
                    eprintln!("Failed to initialize config manager: {}", e);
                }
            }

            // Initialize database manager
            match DatabaseManager::new(&app_handle) {
                Ok(db) => {
                    // Clone connection before moving db
                    let conn_clone = db.conn.clone();
                    *state.database.lock().unwrap() = Some(db);

                    // Initialize LogStreamManager with database connection
                    let mut log_manager = LogStreamManager::new(conn_clone, 1000);
                    log_manager.set_app_handle(app.handle().clone());
                    *state.log_stream_manager.lock().unwrap() = Some(log_manager);

                    // Setup event listeners to capture agent stdout/stderr
                    let log_mgr = state.log_stream_manager.clone();

                    // Listen to agent stdout (agent-message event)
                    let log_mgr_clone = log_mgr.clone();
                    app.listen("agent-message", move |event| {
                        let payload = event.payload();
                        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(payload) {
                            if let Some(agent_id) = msg.get("agent_id").and_then(|v| v.as_str()) {
                                if let Some(content) = msg.get("message").and_then(|v| v.as_str()) {
                                    let mgr = log_mgr_clone.lock().unwrap();
                                    if let Some(manager) = mgr.as_ref() {
                                        manager.capture_log(agent_id, content, "stdout");
                                    }
                                }
                            }
                        }
                    });

                    // Listen to agent stderr (agent-stderr event)
                    let log_mgr_clone2 = log_mgr.clone();
                    app.listen("agent-stderr", move |event| {
                        let payload = event.payload();
                        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(payload) {
                            if let Some(agent_id) = msg.get("agent_id").and_then(|v| v.as_str()) {
                                if let Some(content) = msg.get("line").and_then(|v| v.as_str()) {
                                    let mgr = log_mgr_clone2.lock().unwrap();
                                    if let Some(manager) = mgr.as_ref() {
                                        manager.capture_log(agent_id, content, "stderr");
                                    }
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to initialize database manager: {}", e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            reload_config,
            get_config_path,
            spawn_agent,
            send_to_agent,
            kill_agent,
            list_running_agents,
            get_running_agents_info,
            get_agent_status,
            pause_agent,
            resume_agent,
            inject_message,
            add_agent,
            remove_agent,
            update_agent,
            get_machine_id,
            get_task_history,
            get_task_statistics,
            delete_task_history,
            search_tasks,
            get_task_detail,
            save_memory,
            search_memories,
            get_agent_memories,
            get_shared_memories,
            delete_memory,
            // Error/Solution commands (Self-Healing)
            save_error,
            get_errors,
            resolve_error,
            save_solution,
            get_solutions,
            // Evolution/Pattern commands (Self-Evolution)
            save_evolution,
            get_evolutions,
            save_pattern,
            get_patterns,
            update_pattern_usage,
            get_db_path,
            get_gateway_config,
            save_gateway_config,
            start_gateway,
            stop_gateway,
            start_tunnel,
            stop_tunnel,
            generate_app_qrcode,
            start_ws_server,
            stop_ws_server,
            get_connected_clients,
            ws_server_status,
            // LogStream commands
            subscribe_logs,
            unsubscribe_logs,
            get_logs,
            search_logs,
            get_logs_by_agent,
            get_logs_by_type,
            clear_log_buffer,
            flush_log_batch,
            // Permission commands
            check_permission,
            check_bash_permission,
            check_path_permission,
            get_default_permissions,
            // Agent Config Parser commands
            parse_agent_config,
            save_agent_config,
            load_agent_config,
            list_agent_configs,
            delete_agent_config,
            get_example_agent_config,
            // Executive Agent commands
            init_executive_agent,
            execute_development_task,
            get_executive_agent_status,
            get_task_result,
            get_generated_files,
            clear_executive_agent,
            // Executive Session database commands
            load_executive_sessions,
            get_executive_session_by_id,
            delete_executive_session_by_id,
            get_executive_session_stats,
            save_executive_session_record
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_db_path(state: State<AppState>) -> Result<String, String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.as_ref()
        .map(|d| d.get_db_path().to_string_lossy().to_string())
        .ok_or_else(|| "Database not initialized".to_string())
}

#[tauri::command]
fn get_task_history(
    status: Option<Vec<String>>,
    source: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<database::TaskRecord>, String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let status_filter = status.map(|s| {
        s.iter()
            .filter_map(|st| database::TaskStatus::from_str(st).ok())
            .collect()
    });

    db.load_tasks(status_filter, source, limit, None)
}

#[tauri::command]
fn get_task_statistics(state: State<AppState>) -> Result<database::TaskStatistics, String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.get_statistics()
}

#[tauri::command]
fn delete_task_history(task_id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.delete_task(&task_id)
}

#[tauri::command]
fn search_tasks(keyword: String, limit: Option<u64>, state: State<AppState>) -> Result<Vec<database::TaskRecord>, String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.search_tasks(&keyword, limit)
}

#[tauri::command]
fn get_task_detail(task_id: String, state: State<AppState>) -> Result<Option<database::TaskRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.get_task(&task_id)
}

// ===== Memory Commands =====

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

// ===== Error/Solution Commands (Self-Healing System) =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorRecord {
    pub id: String,
    pub category: String,
    pub message: String,
    pub context: Option<String>,
    pub stack_trace: Option<String>,
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
    pub status: String,
    pub solution_id: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionRecord {
    pub id: String,
    pub error_id: Option<String>,
    pub approach: String,
    pub steps: Option<String>,
    pub result: String,
    pub success: Option<bool>,
    pub evidence: Option<String>,
    pub created_at: String,
}

// ===== Evolution/Pattern Commands (Self-Evolution System) =====

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

#[tauri::command]
fn save_memory(
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
fn search_memories(
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
fn get_agent_memories(
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
fn get_shared_memories(
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
fn delete_memory(
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

// ===== Error Commands (Self-Healing System) =====

#[tauri::command]
fn save_error(
    category: String,
    message: String,
    context: Option<String>,
    stack_trace: Option<String>,
    agent_id: Option<String>,
    task_id: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO errors (id, category, message, context, stack_trace, agent_id, task_id, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'open', ?8)",
        rusqlite::params![id, category, message, context, stack_trace, agent_id, task_id, now],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
fn get_errors(
    status: Option<String>,
    category: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<ErrorRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let sql = match (&status, &category) {
        (Some(_s), Some(_c)) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors WHERE status = ?1 AND category = ?2 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (Some(_s), None) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors WHERE status = ?1 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (None, Some(_c)) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors WHERE category = ?1 ORDER BY created_at DESC {}",
            limit_clause
        ),
        (None, None) => format!(
            "SELECT id, category, message, context, stack_trace, agent_id, task_id, status, solution_id, created_at, resolved_at
             FROM errors ORDER BY created_at DESC {}",
            limit_clause
        ),
    };

    let records: Vec<ErrorRecord> = match (&status, &category) {
        (Some(s), Some(c)) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s, c], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (Some(s), None) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![s], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, Some(c)) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![c], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
        (None, None) => conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    message: row.get(2)?,
                    context: row.get(3)?,
                    stack_trace: row.get(4)?,
                    agent_id: row.get(5)?,
                    task_id: row.get(6)?,
                    status: row.get(7)?,
                    solution_id: row.get(8)?,
                    created_at: row.get(9)?,
                    resolved_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    };

    Ok(records)
}

#[tauri::command]
fn resolve_error(
    error_id: String,
    solution_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE errors SET status = 'resolved', solution_id = ?1, resolved_at = ?2 WHERE id = ?3",
        rusqlite::params![solution_id, now, error_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// ===== Solution Commands =====

#[tauri::command]
fn save_solution(
    error_id: Option<String>,
    approach: String,
    steps: Option<String>,
    result: String,
    success: Option<bool>,
    evidence: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO solutions (id, error_id, approach, steps, result, success, evidence, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, error_id, approach, steps, result, success, evidence, now],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
fn get_solutions(
    error_id: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<SolutionRecord>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let conn = db.conn.lock().unwrap();
    let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

    let sql = if let Some(_eid) = &error_id {
        format!(
            "SELECT id, error_id, approach, steps, result, success, evidence, created_at
             FROM solutions WHERE error_id = ?1 ORDER BY created_at DESC {}",
            limit_clause
        )
    } else {
        format!(
            "SELECT id, error_id, approach, steps, result, success, evidence, created_at
             FROM solutions ORDER BY created_at DESC {}",
            limit_clause
        )
    };

    let records: Vec<SolutionRecord> = if let Some(eid) = &error_id {
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map(rusqlite::params![eid], |row| {
                Ok(SolutionRecord {
                    id: row.get(0)?,
                    error_id: row.get(1)?,
                    approach: row.get(2)?,
                    steps: row.get(3)?,
                    result: row.get(4)?,
                    success: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    } else {
        conn.prepare(&sql)
            .map_err(|e| e.to_string())?
            .query_map([], |row| {
                Ok(SolutionRecord {
                    id: row.get(0)?,
                    error_id: row.get(1)?,
                    approach: row.get(2)?,
                    steps: row.get(3)?,
                    result: row.get(4)?,
                    success: row.get(5)?,
                    evidence: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };

    Ok(records)
}

// ===== Evolution Commands (Self-Evolution System) =====

#[tauri::command]
fn save_evolution(
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
fn get_evolutions(
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
fn save_pattern(
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
fn get_patterns(
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
fn update_pattern_usage(
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

    // Update success rate if provided
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

// ===== Gateway Config Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConfig {
    feishu: Option<FeishuConfig>,
    telegram: Option<TelegramConfig>,
    discord: Option<DiscordConfig>,
    app: Option<AppConfig>,
    tunnel: Option<TunnelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeishuConfig {
    #[serde(default)]
    app_id: String,
    #[serde(default)]
    app_secret: String,
    encrypt_key: Option<String>,
    verification_token: Option<String>,
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramConfig {
    #[serde(default)]
    bot_token: String,
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    #[serde(default)]
    bot_token: String,
    guild_id: Option<String>,
    channel_id: Option<String>,
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    websocket_port: u16,
    auth_mode: String,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    enabled: bool,
    provider: String,
    ngrok_token: Option<String>,
    ngrok_region: Option<String>,
    custom_url: Option<String>,
    status: String,
    public_url: Option<String>,
}

#[tauri::command]
fn get_gateway_config(state: State<AppState>) -> Result<GatewayConfig, String> {
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let config = gateway_config::load_gateway_config(db)?;
    Ok(gateway_config::mask_config(&config))
}

#[tauri::command]
fn save_gateway_config(config: GatewayConfig, state: State<AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    gateway_config::save_gateway_config(db, &config)?;
    Ok(())
}

#[tauri::command]
fn start_gateway(config: GatewayConfig, app_handle: AppHandle, state: State<AppState>) -> Result<(), String> {
    println!("Starting gateway with config");

    // Persist config to database
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;
    gateway_config::save_gateway_config(db, &config)?;

    // Generate auth token for app gateway
    if let Some(app_config) = &config.app {
        if app_config.enabled {
            let token = Uuid::new_v4().to_string();
            // Store token in ws_server so the WebSocket server can use it
            let ws = state.ws_server.lock().unwrap();
            if let Some(server) = ws.as_ref() {
                server.set_auth_token(Some(token.clone()));
                println!("Gateway auth token generated");
            }
        }
    }

    // 2. Feishu Bot (placeholder)
    if let Some(feishu) = &config.feishu {
        if feishu.enabled {
            println!("Feishu Bot enabled, App ID: {}", feishu.app_id);
        }
    }

    // 3. Telegram Bot (placeholder)
    if let Some(telegram) = &config.telegram {
        if telegram.enabled {
            println!("Telegram Bot enabled");
        }
    }

    // 4. Discord Bot (placeholder)
    if let Some(discord) = &config.discord {
        if discord.enabled {
            println!("Discord Bot enabled, Guild: {:?}", discord.guild_id);
        }
    }

    let _ = app_handle.emit("gateway-started", config);

    Ok(())
}

#[tauri::command]
fn stop_gateway(app_handle: AppHandle, state: State<AppState>) -> Result<(), String> {
    println!("Stopping gateway");

    // 1. 停止WebSocket服务器
    {
        let ws = state.ws_server.lock().unwrap();
        if let Some(server) = ws.as_ref() {
            server.stop();
            println!("WebSocket server stopped");
        }
    }

    // 2. TODO: 停止飞书Bot
    // 3. TODO: 停止Telegram Bot
    // 4. TODO: 停止Discord Bot

    // Emit gateway stopped event
    let _ = app_handle.emit("gateway-stopped", ());

    Ok(())
}

#[tauri::command]
async fn start_tunnel(
    provider: String,
    token: Option<String>,
    port: u16,
    region: Option<String>,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    println!("Starting tunnel with provider: {}, port: {}", provider, port);

    match provider.as_str() {
        "ngrok" => {
            // ngrok需要token，如果没有则返回错误
            if token.is_none() || token.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                return Err("ngrok token required. Get it from https://ngrok.com".to_string());
            }

            let ngrok_token = token.unwrap();
            let ngrok_region = region.unwrap_or_else(|| "ap".to_string());

            // Create or get NgrokManager
            let manager = {
                let mut tm = state.tunnel_manager.lock().unwrap();
                if tm.is_none() {
                    *tm = Some(NgrokManager::new());
                }
                tm.clone().unwrap()
            };

            // Start ngrok process
            manager.start(&ngrok_token, port, &ngrok_region, app_handle)
        }
        "frp" => {
            Err("frp requires custom server configuration. Please set custom URL in config.".to_string())
        }
        "cloudflare" => {
            // Cloudflare tunnel using cloudflared CLI
            Err("Cloudflare tunnel requires cloudflared CLI. Run: cloudflared tunnel --url http://localhost:PORT".to_string())
        }
        _ => Err(format!("Unknown tunnel provider: {}", provider))
    }
}

#[tauri::command]
fn stop_tunnel(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    println!("Stopping tunnel");

    let manager_guard = state.tunnel_manager.lock().unwrap();

    if let Some(manager) = manager_guard.as_ref() {
        manager.stop(app_handle)?;
    }

    Ok(())
}

#[tauri::command]
fn generate_app_qrcode(state: State<AppState>) -> Result<String, String> {
    let token = Uuid::new_v4().to_string();

    // Sync token to ws_server so the client can actually authenticate
    {
        let ws = state.ws_server.lock().unwrap();
        if let Some(server) = ws.as_ref() {
            server.set_auth_token(Some(token.clone()));
        }
    }

    // Check if tunnel is running and has a public URL
    let tunnel_manager = state.tunnel_manager.lock().unwrap();
    let tunnel_url = tunnel_manager.as_ref().and_then(|tm| tm.get_public_url());

    // Use tunnel URL if available, otherwise use local IP
    let ws_url = if let Some(public_url) = tunnel_url {
        // Convert https URL to wss WebSocket URL
        let wss_url = public_url.replace("https://", "wss://").replace("http://", "ws://");
        format!("{}?token={}", wss_url, token)
    } else {
        let local_ip = get_local_ip().unwrap_or_else(|| "localhost".to_string());
        let ws = state.ws_server.lock().unwrap();
        let port = ws.as_ref().map(|s| s.port).unwrap_or(1420);
        format!("ws://{}:{}?token={}", local_ip, port, token)
    };

    println!("Generated QR code URL: {}", ws_url);
    Ok(ws_url)
}

#[tauri::command]
async fn start_ws_server(
    port: u16,
    bind_external: bool,
    state: State<'_, AppState>,
    app_handle: AppHandle
) -> Result<String, String> {
    // Create server
    let server = websocket::WebSocketServer::new(port);

    // Start the server (bind_external determines if we bind to 0.0.0.0 or 127.0.0.1)
    let url = server.start(app_handle, bind_external).await?;

    // Store in state after async operation completes
    {
        let mut ws = state.ws_server.lock().unwrap();
        *ws = Some(server);
    }

    Ok(url)
}

#[tauri::command]
fn stop_ws_server(state: State<AppState>) -> Result<(), String> {
    let ws = state.ws_server.lock().unwrap();
    if let Some(server) = ws.as_ref() {
        server.stop();
    }
    Ok(())
}

#[tauri::command]
fn get_connected_clients(state: State<AppState>) -> Result<Vec<websocket::RemoteClient>, String> {
    let ws = state.ws_server.lock().unwrap();
    if let Some(server) = ws.as_ref() {
        Ok(server.get_clients())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
fn ws_server_status(state: State<AppState>) -> Result<bool, String> {
    let ws = state.ws_server.lock().unwrap();
    if let Some(server) = ws.as_ref() {
        Ok(server.is_running())
    } else {
        Ok(false)
    }
}

/// Get local IP address
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    // Try to connect to a public address to find local IP
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

// ===== LogStream Commands =====

#[tauri::command]
fn subscribe_logs(agent_id: String, state: State<AppState>) -> Result<(), String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    manager.subscribe(&agent_id);
    Ok(())
}

#[tauri::command]
fn unsubscribe_logs(agent_id: String, state: State<AppState>) -> Result<(), String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    manager.unsubscribe(&agent_id);
    Ok(())
}

#[tauri::command]
fn get_logs(
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;

    let count = limit.unwrap_or(100) as usize;
    Ok(manager.get_latest_logs(count))
}

#[tauri::command]
fn search_logs(
    keyword: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    Ok(manager.search_logs(&keyword, limit))
}

#[tauri::command]
fn get_logs_by_agent(
    agent_id: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    Ok(manager.get_logs_from_db(Some(&agent_id), None, limit))
}

#[tauri::command]
fn get_logs_by_type(
    log_type: String,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    let lt = LogType::from_str(&log_type);
    Ok(manager.get_logs_from_db(None, Some(lt), limit))
}

#[tauri::command]
fn clear_log_buffer(state: State<AppState>) -> Result<(), String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    manager.clear_buffer();
    Ok(())
}

#[tauri::command]
fn flush_log_batch(state: State<AppState>) -> Result<Vec<LogEntry>, String> {
    let log_manager = state.log_stream_manager.lock().unwrap();
    let manager = log_manager.as_ref().ok_or_else(|| "LogStreamManager not initialized".to_string())?;
    Ok(manager.flush_batch())
}

// ===== Permission Commands =====

#[tauri::command]
fn check_permission(
    tool_name: String,
    args: String,
    permission_config: PermissionConfig,
) -> Result<PermissionResult, String> {
    let checker = PermissionChecker::new(permission_config)
        .map_err(|e| format!("Failed to create PermissionChecker: {}", e))?;
    Ok(checker.check_tool(&tool_name, &args))
}

#[tauri::command]
fn check_bash_permission(
    command: String,
    permission_config: PermissionConfig,
) -> Result<PermissionResult, String> {
    let checker = PermissionChecker::new(permission_config)
        .map_err(|e| format!("Failed to create PermissionChecker: {}", e))?;
    Ok(checker.check_bash(&command))
}

#[tauri::command]
fn check_path_permission(
    path: String,
    permission_config: PermissionConfig,
) -> Result<PermissionResult, String> {
    let checker = PermissionChecker::new(permission_config)
        .map_err(|e| format!("Failed to create PermissionChecker: {}", e))?;
    Ok(checker.check_path(&path))
}

#[tauri::command]
fn get_default_permissions(agent_type: String) -> PermissionConfig {
    permission_checker::get_default_permissions(&agent_type)
}

// ===== Agent Config Parser Commands =====

#[tauri::command]
fn parse_agent_config(yaml_content: String) -> ParseResult {
    let parser = AgentConfigParser::new();
    parser.parse_yaml(&yaml_content)
}

#[tauri::command]
fn save_agent_config(config: AgentConfigParsed, state: State<AppState>) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.save_to_db(&config, &conn)
}

#[tauri::command]
fn load_agent_config(name: String, state: State<AppState>) -> Result<Option<AgentConfigParsed>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.load_from_db(&name, &conn)
}

#[tauri::command]
fn list_agent_configs(state: State<AppState>) -> Result<Vec<(String, String)>, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.list_configs(&conn)
}

#[tauri::command]
fn delete_agent_config(name: String, state: State<AppState>) -> Result<(), String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;
    let conn = db.conn.lock().unwrap();

    let parser = AgentConfigParser::new();
    parser.delete_config(&name, &conn)
}

#[tauri::command]
fn get_example_agent_config() -> String {
    agent_config_parser::get_example_config()
}

// ===== Executive Agent Commands =====

/// Initialize Executive Agent Manager with workspace path
#[tauri::command]
fn init_executive_agent(
    workspace: String,
    state: State<AppState>,
) -> Result<(), String> {
    let workspace_path = std::path::PathBuf::from(&workspace);

    // Create workspace directory if not exists
    if !workspace_path.exists() {
        std::fs::create_dir_all(&workspace_path)
            .map_err(|e| format!("创建工作目录失败: {}", e))?;
    }

    let manager = ExecutiveAgentManager::new(workspace_path);
    *state.executive_agent_manager.lock().unwrap() = Some(manager);

    Ok(())
}

/// Execute development task using multi-agent workflow
#[tauri::command]
async fn execute_development_task(
    request: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<TaskResult, String> {
    // Get workspace path from manager (quick operation, no await)
    let workspace_path = {
        let manager_guard = state.executive_agent_manager.lock().unwrap();
        let manager = manager_guard
            .as_ref()
            .ok_or_else(|| "Executive Agent Manager 未初始化，请先调用 init_executive_agent".to_string())?;

        // Clone workspace path only
        manager.workspace.clone()
    };
    // Lock is released here

    // Create a new manager for this execution (ownership, no lock needed)
    let manager = ExecutiveAgentManager::new(workspace_path);

    // Execute workflow (async operation with no locks held)
    let result = manager.execute_workflow(request, app_handle).await?;

    // Update state with the manager that has results
    {
        let mut mgr = state.executive_agent_manager.lock().unwrap();
        *mgr = Some(manager);
    }

    Ok(result)
}

/// Get executive agent status
#[tauri::command]
fn get_executive_agent_status(state: State<AppState>) -> Result<HashMap<String, String>, String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    let manager = manager.as_ref()
        .ok_or_else(|| "Executive Agent Manager 未初始化".to_string())?;

    let status = manager.get_agent_status();
    Ok(status.into_iter().map(|(k, v)| (format!("{:?}", k), format!("{:?}", v))).collect())
}

/// Get task result with generated files
#[tauri::command]
fn get_task_result(state: State<AppState>) -> Result<Vec<GeneratedFile>, String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    let manager = manager.as_ref()
        .ok_or_else(|| "Executive Agent Manager 未初始化".to_string())?;

    Ok(manager.get_generated_files())
}

/// Get all generated files
#[tauri::command]
fn get_generated_files(state: State<AppState>) -> Result<Vec<GeneratedFile>, String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    let manager = manager.as_ref()
        .ok_or_else(|| "Executive Agent Manager 未初始化".to_string())?;

    Ok(manager.get_generated_files())
}

/// Clear executive agent state
#[tauri::command]
fn clear_executive_agent(state: State<AppState>) -> Result<(), String> {
    let manager = state.executive_agent_manager.lock().unwrap();
    if let Some(manager) = manager.as_ref() {
        manager.clear();
    }
    Ok(())
}

// ===== Executive Session Database Commands =====

/// Load executive sessions from database
#[tauri::command]
fn load_executive_sessions(
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<database::ExecutiveSessionRecord>, String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.load_executive_sessions(limit)
}

/// Get single executive session by ID
#[tauri::command]
fn get_executive_session_by_id(
    id: String,
    state: State<AppState>,
) -> Result<Option<database::ExecutiveSessionRecord>, String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.get_executive_session(&id)
}

/// Delete executive session from database
#[tauri::command]
fn delete_executive_session_by_id(
    id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.delete_executive_session(&id)
}

/// Get executive session statistics
#[tauri::command]
fn get_executive_session_stats(state: State<AppState>) -> Result<database::ExecutiveSessionStats, String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.get_executive_session_stats()
}

/// Save executive session record to database
#[tauri::command]
fn save_executive_session_record(
    session: database::ExecutiveSessionRecord,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.database.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.save_executive_session(&session)
}
