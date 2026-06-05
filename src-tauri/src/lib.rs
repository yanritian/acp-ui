mod agent;
mod config;
mod database;
mod gateway_config;
mod websocket;
mod finance;  // ERP Finance Module
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
mod skill_commands;   // NEW: Skill System Tauri Commands (OpenClacky pattern)
mod bot_adapters;     // NEW: Bot Adapters - Telegram, Feishu, Discord, App WebSocket
mod commands;         // Refactored Tauri command handlers organized by domain
mod plugin_registry;  // NEW: Unified Plugin Registry - Skills/MCP/Hooks/CLI/Adapters
mod swarm_orchestrator; // NEW: Agent Swarm Orchestrator - top-level Codex/Claude Code coordination
mod workflow_engine;  // NEW: Ultra Workflow Engine - multi-stage orchestrated workflows
mod mcp_client;       // NEW: MCP JSON-RPC Client - tool discovery and invocation
mod agent_bus;        // NEW: Agent Communication Bus - agent-to-agent messaging + pub/sub

use agent::{AgentManager};
use config::ConfigManager;
use database::DatabaseManager;
use log_stream::LogStreamManager;
use parking_lot::RwLock;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager, State};

/// Application state containing all managers
pub struct AppState {
    pub config_manager: Arc<RwLock<Option<ConfigManager>>>,
    pub agent_manager: AgentManager,
    pub database: Arc<Mutex<Option<DatabaseManager>>>,
    pub ws_server: Arc<Mutex<Option<websocket::WebSocketServer>>>,
    pub tunnel_manager: Arc<Mutex<Option<tunnel::NgrokManager>>>,
    pub log_stream_manager: Arc<Mutex<Option<LogStreamManager>>>,
    pub executive_agent_manager: Arc<Mutex<Option<executive_agent::ExecutiveAgentManager>>>,
    // Agent Teams Platform (wrapped in Mutex for mutability in Tauri commands)
    pub circuit_breaker_manager: Arc<Mutex<circuit_breaker::CircuitBreakerManager>>,
    pub dag_engine: Arc<Mutex<team_dag::DAGEngine>>,
    pub anomaly_detector: Arc<Mutex<self_healing::AnomalyDetector>>,
    pub healing_executor: Arc<Mutex<self_healing::HealingExecutor>>,
    // Bot Adapters
    pub telegram_adapter: Arc<Mutex<Option<bot_adapters::TelegramAdapter>>>,
    pub feishu_adapter: Arc<Mutex<Option<bot_adapters::FeishuAdapter>>>,
    // Pluggable Architecture
    pub plugin_registry: Arc<Mutex<plugin_registry::PluginRegistry>>,
    pub swarm_orchestrator: Arc<Mutex<swarm_orchestrator::SwarmOrchestrator>>,
    pub workflow_engine: Arc<Mutex<workflow_engine::WorkflowEngine>>,
    pub hooks_executor: Arc<Mutex<hooks_executor::HooksExecutor>>,
    // MCP & Agent Communication
    pub mcp_client: Arc<Mutex<mcp_client::McpClient>>,
    pub agent_bus: Arc<Mutex<agent_bus::AgentBus>>,
}

impl AppState {
    fn new(agent_manager: AgentManager) -> Self {
        // Create anomaly_detector Arc first so we can share it with HealingExecutor
        let anomaly_detector = Arc::new(Mutex::new(self_healing::AnomalyDetector::new()));
        let healing_executor = Arc::new(Mutex::new(self_healing::HealingExecutor::new(anomaly_detector.clone())));

        Self {
            config_manager: Arc::new(RwLock::new(None)),
            agent_manager,
            database: Arc::new(Mutex::new(None)),
            ws_server: Arc::new(Mutex::new(None)),
            tunnel_manager: Arc::new(Mutex::new(None)),
            log_stream_manager: Arc::new(Mutex::new(None)),
            executive_agent_manager: Arc::new(Mutex::new(None)),
            // Agent Teams Platform
            circuit_breaker_manager: Arc::new(Mutex::new(circuit_breaker::CircuitBreakerManager::new())),
            dag_engine: Arc::new(Mutex::new(team_dag::DAGEngine::new())),
            anomaly_detector,
            healing_executor,
            // Bot Adapters
            telegram_adapter: Arc::new(Mutex::new(None)),
            feishu_adapter: Arc::new(Mutex::new(None)),
            // Pluggable Architecture
            plugin_registry: Arc::new(Mutex::new(plugin_registry::PluginRegistry::new())),
            swarm_orchestrator: Arc::new(Mutex::new(swarm_orchestrator::SwarmOrchestrator::new())),
            workflow_engine: Arc::new(Mutex::new(workflow_engine::WorkflowEngine::new())),
            hooks_executor: Arc::new(Mutex::new(hooks_executor::HooksExecutor::new())),
            // MCP & Agent Communication
            mcp_client: Arc::new(Mutex::new(mcp_client::McpClient::new())),
            agent_bus: Arc::new(Mutex::new(agent_bus::AgentBus::new())),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use commands::*;

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

            // Auto-start WebSocket server on port 1421 for remote control testing
            let app_handle_for_ws = app.handle().clone();

            // Seed default plugins (16 core skills + 2 MCP servers)
            {
                let mut registry = state.plugin_registry.lock().unwrap();
                registry.seed_defaults();
                println!("✅ Plugin registry seeded with {} plugins", registry.list(None).len());
            }

            tauri::async_runtime::spawn(async move {
                let server = websocket::WebSocketServer::new(1421);
                match server.start(app_handle_for_ws.clone(), false).await {
                    Ok(url) => {
                        println!("✅ WebSocket server auto-started: {}", url);

                        // Get state from app_handle and store server
                        let state = app_handle_for_ws.state::<AppState>();
                        let mut ws = state.ws_server.lock().unwrap();
                        *ws = Some(server);

                        // Emit event to notify frontend
                        let _ = app_handle_for_ws.emit("ws-server-auto-started", serde_json::json!({
                            "url": url,
                            "port": 1421
                        }));
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to auto-start WebSocket server: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config commands
            get_config,
            reload_config,
            get_config_path,
            get_machine_id,
            // Agent lifecycle commands
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
            // Task history commands
            get_db_path,
            get_task_history,
            get_task_statistics,
            delete_task_history,
            search_tasks,
            get_task_detail,
            save_task_history,
            // Memory commands
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
            // Gateway commands
            get_gateway_config,
            save_gateway_config,
            start_gateway,
            stop_gateway,
            start_tunnel,
            stop_tunnel,
            // WebSocket commands
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
            save_executive_session_record,
            // Agent Teams Platform commands
            analyze_task_complexity,
            get_circuit_breaker_status,
            is_circuit_breaker_allowed,
            reset_circuit_breaker,
            get_all_circuit_breakers,
            create_dag_plan,
            get_dag_plan_progress,
            check_anomaly,
            update_anomaly_baseline,
            // Self-Healing Executor commands
            crate::self_healing::healing_execute,
            crate::self_healing::healing_list_actions,
            crate::self_healing::healing_resolve_action,
            crate::self_healing::healing_get_stats,
            // Skill System Commands (OpenClacky pattern)
            skill_commands::skills_list,
            skill_commands::skill_view,
            skill_commands::invoke_skill,
            skill_commands::create_skill,
            skill_commands::skill_manage,
            skill_commands::skill_rate,
            // Plugin Registry commands
            plugin_registry::plugin_list,
            plugin_registry::plugin_register,
            plugin_registry::plugin_unregister,
            plugin_registry::plugin_get,
            plugin_registry::plugin_set_enabled,
            plugin_registry::plugin_update_config,
            plugin_registry::plugin_search,
            plugin_registry::plugin_get_stats,
            plugin_registry::plugin_get_history,
            // Swarm Orchestrator commands
            swarm_orchestrator::swarm_register_agent,
            swarm_orchestrator::swarm_list_agents,
            swarm_orchestrator::swarm_create_task,
            swarm_orchestrator::swarm_submit_result,
            swarm_orchestrator::swarm_get_task,
            swarm_orchestrator::swarm_get_health,
            swarm_orchestrator::swarm_cancel_task,
            // Workflow Engine commands
            workflow_engine::workflow_create,
            workflow_engine::workflow_validate,
            workflow_engine::workflow_list,
            workflow_engine::workflow_get,
            workflow_engine::workflow_get_progress,
            workflow_engine::workflow_update_status,
            workflow_engine::workflow_submit_result,
            workflow_engine::workflow_generate_from_task,
            workflow_engine::workflow_cancel,
            workflow_engine::workflow_save,
            // Hooks Executor commands
            hooks_executor::hook_register,
            hooks_executor::hook_register_for_agent,
            hooks_executor::hook_unregister_agent,
            hooks_executor::hook_execute_pre_tool,
            hooks_executor::hook_execute_post_tool,
            hooks_executor::hook_execute_post_tool_failure,
            hooks_executor::hook_list,
            hooks_executor::hook_get_agent_hooks,
            // MCP Client commands
            mcp_client::mcp_list_tools,
            mcp_client::mcp_list_server_tools,
            mcp_client::mcp_call_tool,
            mcp_client::mcp_connected_servers,
            mcp_client::mcp_is_connected,
            mcp_client::mcp_connect,
            mcp_client::mcp_disconnect,
            mcp_client::mcp_get_stats,
            // Agent Bus commands
            agent_bus::agent_bus_register,
            agent_bus::agent_bus_unregister,
            agent_bus::agent_bus_send,
            agent_bus::agent_bus_receive,
            agent_bus::agent_bus_peek,
            agent_bus::agent_bus_inbox_count,
            agent_bus::agent_bus_subscribe,
            agent_bus::agent_bus_publish,
            agent_bus::agent_bus_stats,
            agent_bus::agent_bus_history,
            agent_bus::agent_bus_list_agents,
            agent_bus::agent_bus_list_topics,
            agent_bus::agent_bus_is_registered
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
