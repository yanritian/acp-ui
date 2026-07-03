// ============================================================================
// ACP-UI Rust Module Organization
// ============================================================================
// Modules are grouped by domain for better navigation and maintainability.
// Future: Each group may become a subdirectory with mod.rs aggregation.
// ============================================================================

// ---- Core Infrastructure ----
mod config;
mod database;
mod log_stream;

// ---- ACP Transport Layer ----
mod websocket;
mod tunnel;

// ---- Agent Management ----
mod agent;
mod agent_registry;
mod agent_config_parser;
mod agent_bus;
mod session_manager;  // Claw Code Session Management
mod agent_adapter;    // Unified Agent Adapter interface (Claude Code, Codex, etc.)

// ---- Game Development ----
mod game_detector;           // Game engine detection (Godot, Unity)
mod game_engine;             // Unified game engine interface
mod game_build_monitor;      // Real-time build progress monitoring
mod game_launcher;           // Game launcher and process management
mod game_process_monitor;    // Process metrics and performance monitoring
mod game_error_handler;      // Comprehensive error handling and UX

// ---- Security & Permissions ----
mod permission_checker;
mod circuit_breaker;  // Three-state failure protection
mod privacy_orchestrator; // Privacy-Preserving Orchestrator (Phase 1 Week 3)

// ---- Workflow & Orchestration ----
mod team_dag;         // Team DAG execution engine
mod workflow_engine;  // Multi-stage orchestrated workflows
mod swarm_orchestrator; // Top-level Codex/Claude Code coordination
mod swarm_adapters;   // Swarm Agent Adapters - Stdio process communication (Day 1)
mod swarm_types;      // Swarm Types - Task shards, replica assignment (Day 2)
mod task_partitioner; // Task Partitioner - ES sharding pattern (Day 2)
mod goal;            // Goal-driven architecture (RFC-001) — replaces TaskPartitioner
mod goal_evaluator;  // ConditionEvaluator — evaluates CompletionConditions
mod reconcile;       // Reconcile Loop — iterative Goal execution with feedback
mod goal_graph;      // GoalGraph — dependency DAG for multi-Goal coordination
mod topology;        // Swarm Topology — Star/Chain/Pipeline execution patterns
mod queen_lease;      // Queen Lease Election - ZK lease pattern (Day 3)
mod worktree;         // Git Worktree Isolation - per-Goal workspace isolation
mod hermes_flow;      // Hermes Flow - Development Flow Orchestration (Phase 2)
mod sync_engine;      // Sync Engine - Multi-platform data sync (Phase 3)
mod agent_orchestration; // Agent Orchestration Layer - "驾驭层"
mod approval_engine;     // Approval protocol for workflow stages

// ---- Smart Routing & Self-Optimizing ----
mod smart_router;     // Three-layer complexity evaluation
mod self_optimizing_router; // Historical data-based intelligent routing (Phase 1 Week 2)
mod project_context;  // Project context tracking (Phase 1 Week 2)
mod self_healing;     // EWMA anomaly detection
mod loop_engine;      // Loop Engine — unified orchestrator for all loops
mod event_router;     // Claw Code Event Router (clawhip layer)
mod event_pusher;     // Tauri emit adapter for EventBus (M-3 WebSocket bridge)
mod game_assets;      // Game Asset Detection and Analysis (Phase 3 Week 4)
mod cost_tracker;     // Cost Tracker - Budget management (Phase 4 Week 4)
mod one_shot;         // One-Shot Interface + User Role Detection (Phase 5)
mod http_server;      // HTTP Server - REST API for Web/Mobile (Phase 7)

// ---- Plugin System ----
mod plugin_registry;  // Unified: Skills/MCP/Hooks/CLI/Adapters
mod hooks_executor;
mod skill_commands;   // Skill System Tauri Commands
mod mcp_manager;
mod mcp_client;       // MCP JSON-RPC Client

// ---- Bot Adapters ----
mod bot_adapters;     // Telegram, Feishu, Discord, App WebSocket
mod feishu_rich_message;

// ---- Executive Agent ----
mod executive_agent;  // Executes actual development tasks

// ---- Pluggable Modules ----
mod gateway_config;

// ---- Tauri Commands ----
mod commands;         // Refactored command handlers organized by domain

// ---- Public exports for integration tests ----
pub use goal::{Goal, GoalGraph, GoalStatus, CompletionCondition, Evaluator, IterationRecord, EvaluationResult, GoalGraphSummary, ConditionResult};
pub use reconcile::ReconcileLoop;
pub use topology::{Topology, execute_star, execute_chain, execute_pipeline};
pub use event_pusher::TauriEventPusher;
pub use swarm_orchestrator::{
    SwarmOrchestrator, SwarmAgent, SwarmTask, SwarmTaskStatus,
    AgentRole, AgentSwarmStatus, AgentResult,
    SwarmTopology, ConsensusStrategy, SwarmHealth,
};

// Re-export from workspace crates (used by tests and complex_executor)
pub use swarm_engine::{GoalOutcome, EchoExecutor};
pub use tool_sandbox::{SandboxConfig, DeniedPatterns, Severity};

// Re-export unified types from acp-core (C-1 integration)
pub use acp_core::{
    GoalSpec, CompletionConditionSpec, EvaluatorSpec,
    GoalStatus as UnifiedGoalStatus,
    IterationRecord as UnifiedIterationRecord,
    EvaluationResult as UnifiedEvaluationResult,
    GoalOutcome as UnifiedGoalOutcome,
    GoalGraphSummary as UnifiedGoalGraphSummary,
    AcpEvent,
};

use agent::{AgentManager};
use config::ConfigManager;
use database::DatabaseManager;
use log_stream::LogStreamManager;
use parking_lot::RwLock;
use std::collections::HashMap;
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
    // Hermes Flow - Development Flow Orchestrator (Phase 2)
    pub hermes_flow: Arc<Mutex<hermes_flow::HermesFlowOrchestrator>>,
    // Sync Engine - Multi-platform data synchronization (Phase 3)
    pub sync_engine: Arc<Mutex<sync_engine::SyncEngine>>,
    // MCP & Agent Communication
    pub mcp_client: Arc<Mutex<mcp_client::McpClient>>,
    pub agent_bus: Arc<Mutex<agent_bus::AgentBus>>,
    // Agent Orchestration Layer
    pub agent_orchestrator: Arc<Mutex<agent_orchestration::AgentOrchestrator>>,
    // Approval Engine
    pub approval_engine: Arc<Mutex<approval_engine::ApprovalEngine>>,
    // Swarm Worker Registry (Day 1 - Real Worker Adapters)
    pub swarm_workers: Arc<Mutex<HashMap<String, Arc<dyn swarm_adapters::SwarmAgentAdapter + Send + Sync>>>>,
    // Goal-Driven Architecture (RFC-001)
    pub goal_graph: Arc<Mutex<goal::GoalGraph>>,
    // Reconcile Loop for Goal execution
    pub reconcile_loop: Arc<Mutex<reconcile::ReconcileLoop>>,
    // Loop Engine — unified orchestrator (Phase 5)
    pub loop_engine: Arc<Mutex<loop_engine::LoopEngine>>,
    // Agent Registry - Self-Optimizing capabilities (Phase 1 Week 2)
    pub agent_registry: Arc<Mutex<agent_registry::AgentRegistry>>,
}

impl AppState {
    fn new(agent_manager: AgentManager) -> Self {
        // Create anomaly_detector Arc first so we can share it with HealingExecutor
        let anomaly_detector = Arc::new(Mutex::new(self_healing::AnomalyDetector::new()));
        let healing_executor = Arc::new(Mutex::new(self_healing::HealingExecutor::new(anomaly_detector.clone())));
        // Create swarm_workers Arc first so we can share it with ReconcileLoop
        let swarm_workers: Arc<Mutex<HashMap<String, Arc<dyn swarm_adapters::SwarmAgentAdapter + Send + Sync>>>> =
            Arc::new(Mutex::new(HashMap::new()));

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
            // Hermes Flow (Phase 2)
            hermes_flow: Arc::new(Mutex::new(hermes_flow::HermesFlowOrchestrator::new())),
            // Sync Engine (Phase 3)
            sync_engine: Arc::new(Mutex::new(
                sync_engine::SyncEngine::new(sync_engine::SyncSource::TauriDesktop)
                    .unwrap_or_else(|e| {
                        eprintln!("[lib] SyncEngine init failed ({}), using fallback in-memory mode", e);
                        sync_engine::SyncEngine::new(sync_engine::SyncSource::TauriDesktop)
                            .expect("SyncEngine fallback also failed")
                    })
            )),
            // MCP & Agent Communication
            mcp_client: Arc::new(Mutex::new(mcp_client::McpClient::new())),
            agent_bus: Arc::new(Mutex::new(agent_bus::AgentBus::new())),
            // Agent Orchestration Layer
            agent_orchestrator: Arc::new(Mutex::new(agent_orchestration::AgentOrchestrator::new())),
            // Approval Engine
            approval_engine: Arc::new(Mutex::new(approval_engine::ApprovalEngine::new())),
            // Swarm Worker Registry (Day 1)
            swarm_workers: swarm_workers.clone(),
            // Goal-Driven Architecture (RFC-001)
            goal_graph: Arc::new(Mutex::new(goal::GoalGraph::new())),
            // Reconcile Loop (initialized with swarm_workers)
            reconcile_loop: Arc::new(Mutex::new(reconcile::ReconcileLoop::new(swarm_workers.clone()))),
            // Loop Engine (unified orchestrator for all loops)
            loop_engine: Arc::new(Mutex::new(loop_engine::LoopEngine::new(swarm_workers))),
            // Agent Registry - Self-Optimizing capabilities (Phase 1 Week 2)
            agent_registry: Arc::new(Mutex::new(agent_registry::AgentRegistry::new())),
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
        .manage(commands::renpy::RenPyState::new())
        .manage(commands::game_designer::GameDesignerState::new())
        .manage(commands::game_developer::GameDeveloperState::new())
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

                    // Wire database into HealingExecutor for auto-learning feedback loop
                    {
                        let mut healer = state.healing_executor.lock().unwrap();
                        healer.set_database(conn_clone.clone());
                    }

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

            // Listen to agent-closed events for bus/hooks cleanup
            // When an agent process exits naturally (stdout thread ends), the
            // AgentManager removes it from its map but doesn't know about bus/hooks.
            // This listener ensures those registrations are cleaned up too.
            let bus_clone = state.agent_bus.clone();
            let hooks_clone = state.hooks_executor.clone();
            app.listen("agent-closed", move |event| {
                let payload = event.payload();
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(payload) {
                    if let Some(agent_id) = msg.get("agent_id").and_then(|v| v.as_str()) {
                        if let Ok(mut bus) = bus_clone.lock() {
                            let _ = bus.unregister_agent(agent_id);
                        }
                        if let Ok(mut hooks) = hooks_clone.lock() {
                            hooks.unregister_agent_hooks(agent_id);
                        }
                    }
                }
            });

            // Auto-start WebSocket server on port 1421 for remote control testing
            let app_handle_for_ws = app.handle().clone();

            // Seed default plugins (16 core skills + 2 MCP servers)
            {
                let mut registry = state.plugin_registry.lock().unwrap();
                registry.seed_defaults();
                println!("✅ Plugin registry seeded with {} plugins", registry.list(None).len());
            }

            // Wire workflow engine into agent orchestrator
            {
                let mut orchestrator = state.agent_orchestrator.lock().unwrap();
                orchestrator.set_workflow_engine(state.workflow_engine.clone());
                println!("✅ Agent orchestrator wired to workflow engine");
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
            // Loop Engine commands
            crate::loop_engine::loop_execute_goal,
            crate::loop_engine::loop_get_state,
            crate::loop_engine::loop_get_stats,
            crate::loop_engine::loop_update_metrics,
            // Self-Optimizing Router commands (Phase 1 Week 2)
            self_optimizing_init,
            self_optimizing_record,
            self_optimizing_route,
            self_optimizing_get_proficiencies,
            self_optimizing_get_history,
            self_optimizing_reset,
            // Project Context commands (Phase 1 Week 2)
            project_context_create,
            project_context_update,
            project_context_get_summary,
            // Privacy Orchestrator commands (Phase 1 Week 3)
            privacy_analyze_task,
            privacy_get_active_zone,
            privacy_set_zone,
            privacy_check_path,
            privacy_list_zones,
            privacy_create_zone,
            // Agent Adapter execution commands (Phase 1 Week 4)
            agent_adapter_init_claude,
            agent_adapter_init_codex,
            agent_adapter_execute_claude,
            agent_adapter_execute_codex,
            agent_adapter_get_health,
            agent_adapter_estimate_cost,
            // Desktop Development commands (Phase 2)
            desktop_detect_project,
            desktop_tauri_dev,
            desktop_tauri_build,
            desktop_electron_dev,
            desktop_electron_build,
            desktop_get_platform,
            desktop_get_targets,
            // Game Development commands (Phase 3)
            game_detect_framework,
            unity_build_dev,
            unity_build_release,
            unity_get_scenes,
            godot_build_dev,
            godot_build_release,
            godot_get_scenes,
            godot_detect,
            game_detect,
            unity_build,
            unity_get_targets,
            game_export,
            game_get_export_targets,
            game_validate_export,
            game_launch,
            game_stop,
            game_get_status,
            game_list_running,
            game_get_platforms,
            // Game Asset commands (Phase 3 Week 4)
            game_detect_assets,
            game_get_asset_stats,
            game_get_optimization_suggestions,
            game_get_asset_types,
            // Ren'Py Visual Novel commands (Phase 3)
            renpy_get_version,
            renpy_create_project,
            renpy_generate_script,
            renpy_run_game,
            renpy_compile_game,
            renpy_lint_game,
            renpy_detect_project,
            // Game Designer AI Agent commands (Phase 3)
            game_designer_generate_gdd,
            game_designer_generate_concept,
            game_designer_recommend_engine,
            // Game Developer AI Agent commands (Phase 3)
            game_developer_generate_code,
            game_developer_generate_file,
            game_developer_get_supported_engines,
            // Marketing commands (Phase 4)
            marketing_init_kimi,
            kimi_generate_text,
            kimi_translate,
            marketing_init_jimeng,
            jimeng_generate_image,
            marketing_init_kling,
            kling_generate_video,
            kling_get_job_status,
            // Office commands (Phase 4 Week 3)
            office_init_wps,
            wps_generate_doc,
            wps_analyze_excel,
            // Budget commands (Phase 4 Week 4)
            budget_set_limit,
            budget_get_usage,
            budget_get_forecast,
            budget_is_allowed,
            // One-Shot Interface commands (Phase 5)
            one_shot_execute,
            one_shot_detect_role,
            one_shot_detect_scene,
            one_shot_get_recommendation,
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
            swarm_orchestrator::swarm_execute_chain,
            swarm_orchestrator::swarm_handle_failure,
            // Goal-Driven Architecture commands (RFC-001)
            goal::goal_submit,
            goal::goal_get_status,
            goal::goal_cancel,
            goal::goal_get_graph_summary,
            goal::goal_list,
            goal::goal_get_ready,
            goal::goal_assign_worker,
            goal::goal_add_iteration,
            goal::goal_execute_once,
            goal::goal_execute_ready,
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
            workflow_engine::workflow_load_all,
            // Hermes Flow commands (Phase 2)
            hermes_flow::hermes_flow_create,
            hermes_flow::hermes_flow_get_context,
            hermes_flow::hermes_flow_get_status,
            hermes_flow::hermes_flow_execute,
            hermes_flow::hermes_flow_cancel,
            hermes_flow::hermes_flow_list,
            hermes_flow::hermes_flow_update_result,
            // Sync Engine commands (Phase 3)
            sync_engine::sync_put_entity,
            sync_engine::sync_get_entity,
            sync_engine::sync_delete_entity,
            sync_engine::sync_list_entities,
            sync_engine::sync_get_stats,
            sync_engine::sync_receive_event,
            sync_engine::sync_flush_offline,
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
            agent_bus::agent_bus_is_registered,
            // Agent Orchestration commands
            agent_orchestration::orchestration_register_agent,
            agent_orchestration::orchestration_list_agents,
            agent_orchestration::orchestration_select_agent,
            agent_orchestration::orchestration_execute_task,
            agent_orchestration::orchestration_execute_task_auto,
            agent_orchestration::orchestration_get_task_status,
            agent_orchestration::orchestration_cancel_task,
            agent_orchestration::orchestration_list_active_tasks,
            agent_orchestration::orchestration_list_task_history,
            agent_orchestration::orchestration_execute_workflow_stage,
            agent_orchestration::orchestration_execute_full_workflow,
            agent_orchestration::orchestration_seed_default_agents,
            // Approval Engine commands
            approval_engine::approval_get_pending,
            approval_engine::approval_get_request,
            approval_engine::approval_decide,
            approval_engine::approval_get_stats,
            // Swarm Worker Adapter commands (Day 1)
            swarm_register_worker,
            swarm_list_workers,
            swarm_send_task,
            swarm_get_worker_status,
            swarm_health_check,
            swarm_worker_cancel_task,
            swarm_get_task_output,
            swarm_shutdown_worker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// Swarm Worker Adapter Commands (Day 1)
// ============================================================================

use swarm_adapters::{SwarmAgentAdapter, TaskDescription, TaskHandle, WorkerStatus, WorkerCapabilities};

/// Register a new swarm worker (Codex or Claude Code)
#[tauri::command]
async fn swarm_register_worker(
    state: State<'_, AppState>,
    worker_type: String,
    worker_id: String,
) -> Result<WorkerCapabilities, String> {
    let adapter: Arc<dyn SwarmAgentAdapter + Send + Sync> = match worker_type.as_str() {
        "codex" => Arc::new(swarm_adapters::CodexAdapter::new(worker_id.clone())),
        "claude_code" => Arc::new(swarm_adapters::ClaudeCodeAdapter::new(worker_id.clone())),
        _ => return Err(format!("Unknown worker type: {}", worker_type)),
    };

    let capabilities = adapter.capabilities();

    // Store in registry
    {
        let mut workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
        workers.insert(worker_id.clone(), adapter);
    }

    println!("✅ Registered swarm worker: {} ({})", worker_id, worker_type);
    Ok(capabilities)
}

/// List all registered swarm workers
#[tauri::command]
async fn swarm_list_workers(
    state: State<'_, AppState>,
) -> Result<Vec<WorkerCapabilities>, String> {
    let workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
    let capabilities: Vec<WorkerCapabilities> = workers
        .values()
        .map(|adapter| adapter.capabilities())
        .collect();
    Ok(capabilities)
}

/// Send a task to a specific worker
#[tauri::command]
async fn swarm_send_task(
    state: State<'_, AppState>,
    worker_id: String,
    task_id: String,
    prompt: String,
    working_dir: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<TaskHandle, String> {
    let adapter = {
        let workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
        workers.get(&worker_id)
            .ok_or_else(|| format!("Worker not found: {}", worker_id))?
            .clone() // Arc clone — Mutex released immediately
    };

    let task = TaskDescription::new(task_id, prompt)
        .with_timeout(timeout_ms.unwrap_or(60000));

    let task = if let Some(dir) = working_dir {
        task.with_working_dir(dir)
    } else {
        task
    };

    adapter.send_task(&task).map_err(|e| e.to_string())
}

/// Get worker status
#[tauri::command]
async fn swarm_get_worker_status(
    state: State<'_, AppState>,
    worker_id: String,
) -> Result<WorkerStatus, String> {
    let adapter = {
        let workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
        workers.get(&worker_id)
            .ok_or_else(|| format!("Worker not found: {}", worker_id))?
            .clone()
    };
    adapter.get_status().map_err(|e| e.to_string())
}

/// Health check for a worker
#[tauri::command]
async fn swarm_health_check(
    state: State<'_, AppState>,
    worker_id: String,
) -> Result<bool, String> {
    let adapter = {
        let workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
        workers.get(&worker_id)
            .ok_or_else(|| format!("Worker not found: {}", worker_id))?
            .clone()
    };
    Ok(adapter.health_check())
}

/// Cancel a running task on a specific worker adapter
#[tauri::command]
async fn swarm_worker_cancel_task(
    state: State<'_, AppState>,
    worker_id: String,
    task_id: String,
) -> Result<(), String> {
    let adapter = {
        let workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
        workers.get(&worker_id)
            .ok_or_else(|| format!("Worker not found: {}", worker_id))?
            .clone()
    };
    adapter.cancel_task(&task_id).map_err(|e| e.to_string())
}

/// Get task output
#[tauri::command]
async fn swarm_get_task_output(
    state: State<'_, AppState>,
    worker_id: String,
    task_id: String,
) -> Result<String, String> {
    let adapter = {
        let workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
        workers.get(&worker_id)
            .ok_or_else(|| format!("Worker not found: {}", worker_id))?
            .clone()
    };
    adapter.get_task_output(&task_id).map_err(|e| e.to_string())
}

/// Shutdown a worker
#[tauri::command]
async fn swarm_shutdown_worker(
    state: State<'_, AppState>,
    worker_id: String,
) -> Result<(), String> {
    let adapter = {
        let mut workers = state.swarm_workers.lock().map_err(|e| e.to_string())?;
        workers.remove(&worker_id)
    }; // Mutex released here

    if let Some(adapter) = adapter {
        adapter.shutdown().map_err(|e| e.to_string())?;
        println!("✅ Shutdown swarm worker: {}", worker_id);
    }

    Ok(())
}
