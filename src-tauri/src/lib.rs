mod agent;
mod bot;
mod config;
mod database;
mod gateway_config;
mod teams;
mod websocket;

use agent::{AgentInstance, AgentManager, AgentStatus};
use bot::{BotCommand, BotContext, BotManager, BotPlatform, BotResponse, parse_command};
use config::{AgentConfig, AgentTransport, AgentsConfig, ConfigManager};
use database::DatabaseManager;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use teams::{TeamOrchestrator};
use uuid::Uuid;
use websocket::WebSocketServer;

/// Application state containing all managers
pub struct AppState {
    pub config_manager: Arc<RwLock<Option<ConfigManager>>>,
    pub agent_manager: AgentManager,
    pub database: Arc<Mutex<Option<DatabaseManager>>>,
    pub orchestrator: Arc<Mutex<Option<TeamOrchestrator>>>,
    pub ws_server: Arc<Mutex<Option<WebSocketServer>>>,
    pub bot_manager: Arc<Mutex<Option<BotManager>>>,
}

impl AppState {
    fn new(agent_manager: AgentManager) -> Self {
        Self {
            config_manager: Arc::new(RwLock::new(None)),
            agent_manager,
            database: Arc::new(Mutex::new(None)),
            orchestrator: Arc::new(Mutex::new(None)),
            ws_server: Arc::new(Mutex::new(None)),
            bot_manager: Arc::new(Mutex::new(None)),
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
                    // Store database reference in state (for direct access from commands)
                    *state.database.lock().unwrap() = Some(db);

                    // Initialize orchestrator
                    let orchestrator = TeamOrchestrator::new(state.agent_manager.clone());
                    *state.orchestrator.lock().unwrap() = Some(orchestrator.clone());

                    // Initialize bot manager
                    let bot_manager = BotManager::new(orchestrator);
                    *state.bot_manager.lock().unwrap() = Some(bot_manager);
                }
                Err(e) => {
                    eprintln!("Failed to initialize database manager: {}", e);
                }
            }

            // Set AppHandle for bot manager after initialization
            if let Some(bot_mgr) = state.bot_manager.lock().unwrap().as_mut() {
                bot_mgr.set_app_handle(app_handle.clone());

                // Load agent configs
                let config_manager = state.config_manager.read();
                if let Some(cm) = config_manager.as_ref() {
                    let config = cm.get_config();
                    let configs: std::collections::HashMap<String, AgentConfig> = config.agents.into_iter().collect();
                    bot_mgr.set_agent_configs(configs);
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
            create_multi_agent_task,
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
            get_execution_plan,
            list_running_plans,
            pause_plan_node,
            resume_plan_node,
            handle_bot_command,
            parse_bot_text,
            send_bot_message,
            get_bot_help
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_db_path(state: State<AppState>) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    db.as_ref()
        .map(|d| d.get_db_path().to_string_lossy().to_string())
        .ok_or_else(|| "Database not initialized".to_string())
}

#[tauri::command]
fn create_multi_agent_task(
    name: String,
    prompt: String,
    target_agents: Vec<String>,
    routing: Option<String>,
    priority: Option<String>,
    source: Option<String>,
) -> Result<teams::MultiAgentTask, String> {
    let routing_strategy = routing
        .and_then(|r| serde_json::from_str(&r).ok())
        .unwrap_or(teams::RoutingStrategy::Broadcast);

    let task_priority = priority
        .and_then(|p| serde_json::from_str(&p).ok())
        .unwrap_or(teams::TaskPriority::Normal);

    Ok(teams::MultiAgentTask {
        id: Uuid::new_v4().to_string(),
        name,
        prompt,
        routing: routing_strategy,
        priority: task_priority,
        target_agents,
        dependencies: vec![],
        created_at: chrono::Utc::now(),
        source: source.unwrap_or_else(|| "app".to_string()),
    })
}

#[tauri::command]
fn get_task_history(
    status: Option<Vec<String>>,
    source: Option<String>,
    limit: Option<u64>,
    state: State<AppState>,
) -> Result<Vec<database::TaskRecord>, String> {
    let db = state.database.lock().unwrap();
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
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.get_statistics()
}

#[tauri::command]
fn delete_task_history(task_id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    db.delete_task(&task_id)
}

#[tauri::command]
fn search_tasks(keyword: String, limit: Option<u64>, state: State<AppState>) -> Result<Vec<database::TaskRecord>, String> {
    let db = state.database.lock().unwrap();
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
pub struct MemoryRecord {
    pub id: String,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub content: String,
    pub tags: Option<String>,
    pub importance: f64,
    pub created_at: String,
    pub last_accessed: Option<String>,
}

#[tauri::command]
fn save_memory(
    content: String,
    tags: Option<String>,
    agent_id: Option<String>,
    session_id: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.database.lock().unwrap();
    let db = db.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO memories (id, agent_id, session_id, content, tags, importance, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![id, agent_id, session_id, content, tags, 0.5, now],
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

    let (sql, records): (String, Vec<MemoryRecord>) = match &agent_id {
        Some(aid) => {
            let sql = format!(
                "SELECT id, agent_id, session_id, content, tags, importance, created_at, last_accessed
                 FROM memories WHERE content LIKE ?1 AND (agent_id = ?2 OR agent_id IS NULL)
                 ORDER BY importance DESC, created_at DESC {}",
                limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default()
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            let recs: Vec<MemoryRecord> = stmt
                .query_map(rusqlite::params![pattern, aid], |row| {
                    Ok(MemoryRecord {
                        id: row.get(0)?,
                        agent_id: row.get(1)?,
                        session_id: row.get(2)?,
                        content: row.get(3)?,
                        tags: row.get(4)?,
                        importance: row.get(5)?,
                        created_at: row.get(6)?,
                        last_accessed: row.get(7)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            (sql, recs)
        }
        None => {
            let sql = format!(
                "SELECT id, agent_id, session_id, content, tags, importance, created_at, last_accessed
                 FROM memories WHERE content LIKE ?1
                 ORDER BY importance DESC, created_at DESC {}",
                limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default()
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            let recs: Vec<MemoryRecord> = stmt
                .query_map(rusqlite::params![pattern], |row| {
                    Ok(MemoryRecord {
                        id: row.get(0)?,
                        agent_id: row.get(1)?,
                        session_id: row.get(2)?,
                        content: row.get(3)?,
                        tags: row.get(4)?,
                        importance: row.get(5)?,
                        created_at: row.get(6)?,
                        last_accessed: row.get(7)?,
                    })
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            (sql, recs)
        }
    };
    let _ = sql; // suppress unused warning

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
        "SELECT id, agent_id, session_id, content, tags, importance, created_at, last_accessed
         FROM memories WHERE agent_id = ?1 OR agent_id IS NULL
         ORDER BY importance DESC, created_at DESC {}",
        limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default()
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let records: Vec<MemoryRecord> = stmt
        .query_map(rusqlite::params![agent_id], |row| {
            Ok(MemoryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                session_id: row.get(2)?,
                content: row.get(3)?,
                tags: row.get(4)?,
                importance: row.get(5)?,
                created_at: row.get(6)?,
                last_accessed: row.get(7)?,
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
        "SELECT id, agent_id, session_id, content, tags, importance, created_at, last_accessed
         FROM memories WHERE agent_id IS NULL
         ORDER BY importance DESC, created_at DESC {}",
        limit_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let records: Vec<MemoryRecord> = stmt
        .query_map([], |row| {
            Ok(MemoryRecord {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                session_id: row.get(2)?,
                content: row.get(3)?,
                tags: row.get(4)?,
                importance: row.get(5)?,
                created_at: row.get(6)?,
                last_accessed: row.get(7)?,
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

// ===== Bot Commands =====

#[tauri::command]
fn handle_bot_command(
    platform: String,
    user_id: String,
    chat_id: String,
    message_id: String,
    text: String,
    state: State<AppState>,
) -> Result<BotResponse, String> {
    // Parse command
    let command = parse_command(&text)
        .ok_or_else(|| "Invalid command format. Use /help for available commands.".to_string())?;

    // Create context
    let context = BotContext {
        platform: match platform.as_str() {
            "feishu" => BotPlatform::Feishu,
            "telegram" => BotPlatform::Telegram,
            "discord" => BotPlatform::Discord,
            "app" => BotPlatform::App,
            _ => BotPlatform::App,
        },
        user_id,
        chat_id,
        message_id,
        timestamp: chrono::Utc::now(),
    };

    // Handle command
    let bot_manager = state.bot_manager.lock().unwrap();
    if let Some(bot_mgr) = bot_manager.as_ref() {
        bot_mgr.handle_command(command, context)
    } else {
        Err("Bot manager not initialized".to_string())
    }
}

#[tauri::command]
fn parse_bot_text(text: String) -> Result<Option<BotCommand>, String> {
    Ok(parse_command(&text))
}

#[tauri::command]
fn send_bot_message(
    platform: String,
    chat_id: String,
    message: String,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Emit event for frontend to handle actual message sending
    let _ = app_handle.emit("bot-send-message", serde_json::json!({
        "platform": platform,
        "chat_id": chat_id,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }));

    Ok(())
}

#[tauri::command]
fn get_bot_help() -> Result<String, String> {
    Ok(r#"
🤖 Agent Teams Bot 帮助

指令列表:
/agent <name> <prompt> - 启动单个Agent
/team <name> <agents> <prompt> - 启动多Agent任务
/status - 查看运行状态
/pause <agent_id> - 暂停Agent
/resume <agent_id> - 继续Agent
/cancel <agent_id> - 取消Agent
/inject <agent_id> <message> - 注入消息
/history [limit] - 查看历史
/help - 显示帮助

示例:
/agent claude-code 分析当前项目结构
/team 分析 claude-code,gemini 协同完成代码重构
/status
"#.to_string())
}

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
    app_handle: AppHandle,
) -> Result<String, String> {
    println!("Starting tunnel with provider: {}, port: {}", provider, port);

    // 目前返回模拟URL，实际需要集成ngrok/frp/cloudflare SDK
    match provider.as_str() {
        "ngrok" => {
            // TODO: 实际启动ngrok
            // ngrok需要token，如果没有则返回错误
            if token.is_none() || token.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                return Err("ngrok token required. Get it from https://ngrok.com".to_string());
            }
            // 模拟返回ngrok URL
            let public_url = format!("https://random-id.ngrok-free.app");
            let _ = app_handle.emit("tunnel-started", serde_json::json!({
                "provider": provider,
                "public_url": public_url,
            }));
            Ok(public_url)
        }
        "frp" => {
            // TODO: 实际启动frp客户端
            Err("frp requires custom server configuration. Please set custom URL in config.".to_string())
        }
        "cloudflare" => {
            // TODO: 实际启动cloudflare tunnel
            let public_url = format!("https://random-id.trycloudflare.com");
            let _ = app_handle.emit("tunnel-started", serde_json::json!({
                "provider": provider,
                "public_url": public_url,
            }));
            Ok(public_url)
        }
        _ => Err(format!("Unknown tunnel provider: {}", provider))
    }
}

#[tauri::command]
fn stop_tunnel(app_handle: AppHandle) -> Result<(), String> {
    println!("Stopping tunnel");

    // TODO: 实际停止tunnel进程

    let _ = app_handle.emit("tunnel-stopped", ());
    Ok(())
}

#[tauri::command]
fn generate_app_qrcode(state: State<AppState>) -> Result<String, String> {
    // Generate a QR code URL for app connection
    let token = Uuid::new_v4().to_string();

    // Get actual local IP
    let local_ip = get_local_ip().unwrap_or_else(|| "localhost".to_string());

    // Check if WebSocket server is running
    let ws = state.ws_server.lock().unwrap();
    let port = if let Some(server) = ws.as_ref() {
        if server.is_running() {
            1420 // default
        } else {
            1420
        }
    } else {
        1420
    };

    let qr_url = format!("ws://{}:{}?token={}", local_ip, port, token);
    println!("Generated QR code URL: {}", qr_url);
    Ok(qr_url)
}

#[tauri::command]
async fn start_ws_server(port: u16, state: State<'_, AppState>, app_handle: AppHandle) -> Result<String, String> {
    // Create server
    let server = websocket::WebSocketServer::new(port);

    // Start the server with app_handle
    let url = server.start(app_handle).await?;

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

// ===== Plan Commands =====

#[tauri::command]
fn get_execution_plan(task_id: String, state: State<AppState>) -> Result<Option<teams::ExecutionPlan>, String> {
    let orchestrator = state.orchestrator.lock().unwrap();
    if let Some(orch) = orchestrator.as_ref() {
        Ok(orch.get_plan(&task_id))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn list_running_plans(state: State<AppState>) -> Result<Vec<teams::ExecutionPlan>, String> {
    let orchestrator = state.orchestrator.lock().unwrap();
    if let Some(orch) = orchestrator.as_ref() {
        Ok(orch.list_running_plans())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
fn pause_plan_node(task_id: String, node_id: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    let orchestrator = state.orchestrator.lock().unwrap();
    if let Some(orch) = orchestrator.as_ref() {
        // Find the agent_id for this node
        let plan = orch.get_plan(&task_id);
        let agent_id = plan
            .and_then(|p| p.nodes.iter().find(|n| n.id == node_id).map(|n| n.agent_id.clone()));

        if let Some(Some(agent_id)) = agent_id {
            state.agent_manager.pause_agent(&agent_id, &app_handle)?;
            // Update plan node status
            let _ = app_handle.emit("plan-event", teams::PlanEvent::NodeStatusChanged {
                task_id: task_id.clone(),
                node_id: node_id.clone(),
                status: teams::PlanNodeStatus::Paused,
                timestamp: chrono::Utc::now(),
            });
        }
        Ok(())
    } else {
        Err("Orchestrator not initialized".to_string())
    }
}

#[tauri::command]
fn resume_plan_node(task_id: String, node_id: String, state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    let orchestrator = state.orchestrator.lock().unwrap();
    if let Some(orch) = orchestrator.as_ref() {
        // Find the agent_id for this node
        let plan = orch.get_plan(&task_id);
        let agent_id = plan
            .and_then(|p| p.nodes.iter().find(|n| n.id == node_id).map(|n| n.agent_id.clone()));

        if let Some(Some(agent_id)) = agent_id {
            state.agent_manager.resume_agent(&agent_id, &app_handle)?;
            let _ = app_handle.emit("plan-event", teams::PlanEvent::NodeStatusChanged {
                task_id: task_id.clone(),
                node_id: node_id.clone(),
                status: teams::PlanNodeStatus::Running,
                timestamp: chrono::Utc::now(),
            });
        }
        Ok(())
    } else {
        Err("Orchestrator not initialized".to_string())
    }
}
