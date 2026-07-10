use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::gateway_config::constant_time_eq;

/// Connected remote client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteClient {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub authenticated: bool,
}

/// Message from remote client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMessage {
    pub client_id: String,
    pub message_type: String,
    pub content: serde_json::Value,
    pub timestamp: chrono::DateTime<Utc>,
}

/// JSON-RPC style request from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRequest {
    pub id: String,
    #[serde(rename = "type")]
    pub request_type: String,
    pub token: Option<String>,
    pub command: String,
    pub payload: Option<serde_json::Value>,
}

/// JSON-RPC style response to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteResponse {
    pub id: String,
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Remote command from client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum RemoteCommand {
    PauseAgent {
        agent_id: String,
    },
    ResumeAgent {
        agent_id: String,
    },
    CancelAgent {
        agent_id: String,
    },
    InjectMessage {
        agent_id: String,
        message: String,
    },
    PausePlanNode {
        task_id: String,
        node_id: String,
    },
    ResumePlanNode {
        task_id: String,
        node_id: String,
    },
    GetAgents {},
    GetPlans {},
    GetStatus {},
    // LogStream commands
    SubscribeLogs {
        agent_id: Option<String>,
    },
    UnsubscribeLogs {
        agent_id: Option<String>,
    },
    GetLogs {
        limit: Option<u64>,
        log_type: Option<String>,
    },
    SearchLogs {
        keyword: String,
        limit: Option<u64>,
    },
    // Executive Agent commands (for Flutter mobile)
    InitExecutiveAgent {
        workspace: String,
    },
    ExecuteDevelopmentTask {
        request: String,
    },
    GetExecutiveAgentStatus {},
    GetGeneratedFiles {},
    ClearExecutiveAgent {},
    // Proxy command - dispatch to any registered backend module
    Proxy {
        cmd: String,
        params: serde_json::Value,
    },
    // Discovery - list all available proxy commands
    ListProxyCommands {},
}

/// Client connection state (stored per-connection)
struct ConnectionState {
    authenticated: bool,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
    subscribed_to_logs: bool, // Whether this client wants log updates
}

/// WebSocket server for remote connections with token authentication
pub struct WebSocketServer {
    pub port: u16,
    clients: Arc<RwLock<HashMap<String, RemoteClient>>>,
    connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
    running: Arc<RwLock<bool>>,
    auth_token: Arc<RwLock<Option<String>>>,
    log_push_interval_ms: u64, // Interval for batch log push (default 100ms)
}

impl WebSocketServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            clients: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            auth_token: Arc::new(RwLock::new(None)),
            log_push_interval_ms: 100,
        }
    }

    /// Set the auth token that clients must present to connect
    pub fn set_auth_token(&self, token: Option<String>) {
        *self.auth_token.write() = token;
    }

    /// Start the WebSocket server
    pub async fn start(
        &self,
        app_handle: AppHandle,
        bind_to_all_interfaces: bool,
    ) -> Result<String, String> {
        if *self.running.read() {
            return Err("Server already running".to_string());
        }

        // Bind address: either localhost (secure) or all interfaces (for external access)
        let bind_addr = if bind_to_all_interfaces {
            "0.0.0.0" // Allow LAN/external connections (requires firewall config)
        } else {
            "127.0.0.1" // Local only, most secure
        };

        let addr: SocketAddr = format!("{}:{}", bind_addr, self.port)
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?;

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind {}: {}", bind_addr, e))?;

        *self.running.write() = true;

        let _ = app_handle.emit("ws-server-started", self.port);

        // Clone all Arcs before spawning tasks
        let clients = Arc::clone(&self.clients);
        let connections = Arc::clone(&self.connections);
        let running = Arc::clone(&self.running);
        let auth_token = Arc::clone(&self.auth_token);
        let log_push_interval = self.log_push_interval_ms;

        // Clone for log batch push thread
        let connections_for_log = Arc::clone(&self.connections);
        let running_for_log = Arc::clone(&self.running);

        // Clone for event forwarding thread
        let connections_for_events = Arc::clone(&self.connections);
        let running_for_events = Arc::clone(&self.running);

        // Clone app_handle before moving into spawn
        let app_handle_for_accept = app_handle.clone();
        let app_handle_for_log = app_handle.clone();
        let app_handle_for_events = app_handle.clone();

        // Accept connections thread
        tokio::spawn(async move {
            while *running.read() {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let clients_clone = Arc::clone(&clients);
                        let connections_clone = Arc::clone(&connections);
                        let handle_clone = app_handle_for_accept.clone();
                        let auth_token_clone = Arc::clone(&auth_token);

                        tokio::spawn(async move {
                            handle_connection(
                                stream,
                                addr,
                                clients_clone,
                                connections_clone,
                                handle_clone,
                                auth_token_clone,
                            )
                            .await;
                        });
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                    }
                }
            }
        });

        // Log batch push thread
        tokio::spawn(async move {
            use crate::log_stream::LogEntry;
            use crate::AppState;

            while *running_for_log.read() {
                tokio::time::sleep(tokio::time::Duration::from_millis(log_push_interval)).await;

                // Get batch from LogStreamManager
                let state = app_handle_for_log.state::<AppState>();
                let log_manager = state.log_stream_manager.lock().ok();

                if let Some(guard) = log_manager {
                    if let Some(manager) = guard.as_ref() {
                        let batch: Vec<LogEntry> = manager.flush_batch();

                        if !batch.is_empty() {
                            // Push batch to all subscribed clients
                            let batch_json = serde_json::json!({
                                "type": "log_batch",
                                "logs": batch
                            });

                            if let Ok(json_str) = serde_json::to_string(&batch_json) {
                                let conns = connections_for_log.read();
                                for (_client_id, cs) in conns.iter() {
                                    if cs.authenticated && cs.subscribed_to_logs {
                                        let _ =
                                            cs.sender.send(Message::Text(json_str.clone().into()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // Event forwarding thread - forward Tauri events to WebSocket clients
        tokio::spawn(async move {
            // Listen to Tauri events and forward to WebSocket clients
            let events_to_forward = [
                "task-started",
                "agent-message",
                "agent-status-update",
                "agent-error",
                "file-created",
                "files-created",
                "task-completed",
            ];

            for event_name in events_to_forward {
                let connections_clone = Arc::clone(&connections_for_events);
                let running_clone = Arc::clone(&running_for_events);
                let event_name_str = event_name.to_string();

                // Use Tauri event listener with move closure
                let _ = app_handle_for_events.listen(event_name_str.clone(), move |event| {
                    if !*running_clone.read() {
                        return;
                    }

                    // Forward event payload to all connected WebSocket clients
                    let payload = event.payload();
                    let forward_json = serde_json::json!({
                        "type": event_name_str,
                        "data": payload,
                    });

                    if let Ok(json_str) = serde_json::to_string(&forward_json) {
                        let conns = connections_clone.read();
                        for (_client_id, cs) in conns.iter() {
                            if cs.authenticated {
                                let _ = cs.sender.send(Message::Text(json_str.clone().into()));
                            }
                        }
                    }
                });
            }

            // Keep thread alive while server is running
            while *running_for_events.read() {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(format!("ws://localhost:{}", self.port))
    }

    /// Stop the server
    pub fn stop(&self) {
        *self.running.write() = false;
        self.clients.write().clear();
        self.connections.write().clear();
    }

    /// Get connected clients
    pub fn get_clients(&self) -> Vec<RemoteClient> {
        self.clients.read().values().cloned().collect()
    }

    pub fn is_running(&self) -> bool {
        *self.running.read()
    }
}

/// Handle individual WebSocket connection with token auth
async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    clients: Arc<RwLock<HashMap<String, RemoteClient>>>,
    connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
    app_handle: AppHandle,
    auth_token: Arc<RwLock<Option<String>>>,
) {
    let ws_stream = tokio_tungstenite::accept_async(stream).await;

    if let Err(e) = ws_stream {
        eprintln!("WebSocket handshake failed: {}", e);
        return;
    }

    let (ws_sender, mut ws_receiver) = ws_stream.unwrap().split();

    // Create a channel for sending responses
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Spawn a task to forward messages from channel to WebSocket
    tokio::spawn(async move {
        let mut sink = ws_sender;
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let client_id = uuid::Uuid::new_v4().to_string();

    let client = RemoteClient {
        id: client_id.clone(),
        name: format!("Client-{}", &client_id[..8]),
        platform: "app".to_string(),
        connected_at: chrono::Utc::now(),
        authenticated: false,
    };

    clients.write().insert(client_id.clone(), client.clone());
    connections.write().insert(
        client_id.clone(),
        ConnectionState {
            authenticated: false,
            sender: tx,
            subscribed_to_logs: false,
        },
    );

    let _ = app_handle.emit("client-connected", client.clone());
    println!("Client {} connected from {}", client_id, addr);

    // Auth phase
    let stored_token = auth_token.read().clone();
    let mut authenticated = false;
    let mut unauth_count = 0u8;

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(request) = serde_json::from_str::<RemoteRequest>(&text) {
                    // Auth check
                    if let Some(ref token) = stored_token {
                        if request.request_type != "auth" && !authenticated {
                            unauth_count += 1;
                            if unauth_count > 5 {
                                println!(
                                    "Client {} disconnected: too many unauthenticated messages",
                                    client_id
                                );
                                break;
                            }
                            send_response(
                                &connections,
                                &client_id,
                                &RemoteResponse {
                                    id: request.id,
                                    ok: false,
                                    data: None,
                                    error: Some(
                                        "Not authenticated. Send {type:'auth', token:'...'} first."
                                            .to_string(),
                                    ),
                                },
                            );
                            continue;
                        }

                        if request.request_type == "auth" {
                            if let Some(ref client_token) = request.token {
                                if constant_time_eq(client_token, token) {
                                    authenticated = true;
                                    if let Some(c) = clients.write().get_mut(&client_id) {
                                        c.authenticated = true;
                                    }
                                    if let Some(cs) = connections.write().get_mut(&client_id) {
                                        cs.authenticated = true;
                                    }
                                    send_response(
                                        &connections,
                                        &client_id,
                                        &RemoteResponse {
                                            id: request.id,
                                            ok: true,
                                            data: Some(
                                                serde_json::json!({ "client_id": client_id }),
                                            ),
                                            error: None,
                                        },
                                    );
                                    continue;
                                }
                            }
                            send_response(
                                &connections,
                                &client_id,
                                &RemoteResponse {
                                    id: request.id,
                                    ok: false,
                                    data: None,
                                    error: Some("Invalid token".to_string()),
                                },
                            );
                            continue;
                        }
                    }

                    // Handle command with real data
                    handle_remote_command(&client_id, &request, &app_handle, &connections).await;
                } else if let Ok(command) = serde_json::from_str::<RemoteCommand>(&text) {
                    handle_legacy_command(&client_id, command, &app_handle);
                } else {
                    let text_str = text.to_string();
                    let remote_msg = RemoteMessage {
                        client_id: client_id.clone(),
                        message_type: "text".to_string(),
                        content: serde_json::from_str(&text_str)
                            .unwrap_or_else(|_| serde_json::json!({ "raw": text_str })),
                        timestamp: chrono::Utc::now(),
                    };
                    let _ = app_handle.emit("remote-message", remote_msg);
                }
            }
            Ok(Message::Binary(data)) => {
                let _ = app_handle.emit(
                    "remote-binary",
                    serde_json::json!({
                        "client_id": client_id,
                        "data_length": data.len(),
                    }),
                );
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    clients.write().remove(&client_id);
    connections.write().remove(&client_id);
    let _ = app_handle.emit("client-disconnected", client_id.clone());
    println!("Client {} disconnected", client_id);
}

fn send_response(
    connections: &RwLock<HashMap<String, ConnectionState>>,
    client_id: &str,
    response: &RemoteResponse,
) {
    if let Some(cs) = connections.read().get(client_id) {
        if let Ok(json) = serde_json::to_string(response) {
            let _ = cs.sender.send(Message::Text(json.into()));
        }
    }
}

// ---------------------------------------------------------------------------
// Proxy Command Infrastructure
// ---------------------------------------------------------------------------

/// Proxy command descriptor for capability discovery
struct ProxyCommandInfo {
    name: &'static str,
    description: &'static str,
    module: &'static str,
}

/// Return the list of all available proxy commands with descriptions
fn get_proxy_command_list() -> Vec<serde_json::Value> {
    let commands = vec![
        // Plugin Registry
        ProxyCommandInfo {
            name: "plugin_list",
            description: "List all plugins, optionally filtered by kind",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_get",
            description: "Get a specific plugin by ID",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_search",
            description: "Search plugins by name or capability",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_get_stats",
            description: "Get aggregated execution statistics for a plugin",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_get_history",
            description: "Get execution history for a plugin",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_register",
            description: "Register a new plugin",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_unregister",
            description: "Unregister a plugin by ID",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_set_enabled",
            description: "Enable or disable a plugin",
            module: "plugin_registry",
        },
        ProxyCommandInfo {
            name: "plugin_update_config",
            description: "Update a plugin configuration",
            module: "plugin_registry",
        },
        // Swarm Orchestrator
        ProxyCommandInfo {
            name: "swarm_list_agents",
            description: "List all agents in the swarm",
            module: "swarm_orchestrator",
        },
        ProxyCommandInfo {
            name: "swarm_get_health",
            description: "Get swarm health summary",
            module: "swarm_orchestrator",
        },
        ProxyCommandInfo {
            name: "swarm_get_task",
            description: "Get task status and progress",
            module: "swarm_orchestrator",
        },
        ProxyCommandInfo {
            name: "swarm_list_tasks",
            description: "List all swarm tasks",
            module: "swarm_orchestrator",
        },
        ProxyCommandInfo {
            name: "swarm_register_agent",
            description: "Register an agent in the swarm",
            module: "swarm_orchestrator",
        },
        ProxyCommandInfo {
            name: "swarm_create_task",
            description: "Create a new swarm task",
            module: "swarm_orchestrator",
        },
        ProxyCommandInfo {
            name: "swarm_cancel_task",
            description: "Cancel a running swarm task",
            module: "swarm_orchestrator",
        },
        // Workflow Engine
        ProxyCommandInfo {
            name: "workflow_list",
            description: "List all workflows",
            module: "workflow_engine",
        },
        ProxyCommandInfo {
            name: "workflow_get",
            description: "Get a specific workflow by ID",
            module: "workflow_engine",
        },
        ProxyCommandInfo {
            name: "workflow_get_progress",
            description: "Get workflow execution progress",
            module: "workflow_engine",
        },
        ProxyCommandInfo {
            name: "workflow_cancel",
            description: "Cancel a workflow",
            module: "workflow_engine",
        },
        // MCP Client
        ProxyCommandInfo {
            name: "mcp_connected_servers",
            description: "Get list of connected MCP server names",
            module: "mcp_client",
        },
        ProxyCommandInfo {
            name: "mcp_list_tools",
            description: "List all MCP tools from all connected servers",
            module: "mcp_client",
        },
        ProxyCommandInfo {
            name: "mcp_get_stats",
            description: "Get MCP client statistics",
            module: "mcp_client",
        },
        ProxyCommandInfo {
            name: "mcp_is_connected",
            description: "Check if a specific MCP server is connected",
            module: "mcp_client",
        },
        // Agent Bus
        ProxyCommandInfo {
            name: "agent_bus_list_agents",
            description: "List all registered agents on the bus",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_list_topics",
            description: "List all active pub/sub topics",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_stats",
            description: "Get agent bus statistics",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_history",
            description: "Get message history for audit",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_inbox_count",
            description: "Get message count for an agent inbox",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_is_registered",
            description: "Check if an agent is registered",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_register",
            description: "Register an agent with the bus",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_unregister",
            description: "Unregister an agent from the bus",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_subscribe",
            description: "Subscribe an agent to a topic",
            module: "agent_bus",
        },
        ProxyCommandInfo {
            name: "agent_bus_publish",
            description: "Publish a message to a topic",
            module: "agent_bus",
        },
        // Healing Executor
        ProxyCommandInfo {
            name: "healing_list_actions",
            description: "List all healing actions",
            module: "self_healing",
        },
        ProxyCommandInfo {
            name: "healing_get_stats",
            description: "Get healing statistics",
            module: "self_healing",
        },
        // Hooks Executor
        ProxyCommandInfo {
            name: "hook_list",
            description: "List all registered hooks",
            module: "hooks_executor",
        },
        ProxyCommandInfo {
            name: "hook_get_agent_hooks",
            description: "Get hooks for a specific agent",
            module: "hooks_executor",
        },
        // Circuit Breaker
        ProxyCommandInfo {
            name: "circuit_breaker_get_all",
            description: "Get all circuit breaker states",
            module: "circuit_breaker",
        },
        ProxyCommandInfo {
            name: "circuit_breaker_get_status",
            description: "Get circuit breaker status for a target",
            module: "circuit_breaker",
        },
        ProxyCommandInfo {
            name: "circuit_breaker_reset",
            description: "Reset a circuit breaker",
            module: "circuit_breaker",
        },
        // DAG Engine
        ProxyCommandInfo {
            name: "dag_get_plan",
            description: "Get a DAG execution plan by ID",
            module: "team_dag",
        },
    ];

    commands
        .iter()
        .map(|cmd| {
            serde_json::json!({
                "name": cmd.name,
                "description": cmd.description,
                "module": cmd.module,
            })
        })
        .collect()
}

/// Handle a proxy command by dispatching to the appropriate backend module
fn handle_proxy_command(
    state: &crate::AppState,
    command: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    match command {
        // =====================================================================
        // Plugin Registry
        // =====================================================================
        "plugin_list" => {
            let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            let kind = params.get("kind").and_then(|v| v.as_str())
                .map(|k| k.parse::<crate::plugin_registry::PluginKind>())
                .transpose()?;
            let plugins: Vec<_> = registry.list(kind).into_iter().cloned().collect();
            serde_json::to_value(plugins).map_err(|e| e.to_string())
        }
        "plugin_get" => {
            let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            let plugin = registry.get(id)
                .cloned()
                .ok_or_else(|| format!("Plugin '{}' not found", id))?;
            serde_json::to_value(plugin).map_err(|e| e.to_string())
        }
        "plugin_search" => {
            let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            let query = params.get("query").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'query'")?;
            let plugins: Vec<_> = registry.search(query).into_iter().cloned().collect();
            serde_json::to_value(plugins).map_err(|e| e.to_string())
        }
        "plugin_get_stats" => {
            let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            if registry.get(id).is_none() {
                return Err(format!("Plugin '{}' not found", id));
            }
            let stats = registry.get_stats(id);
            serde_json::to_value(stats).map_err(|e| e.to_string())
        }
        "plugin_get_history" => {
            let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            if registry.get(id).is_none() {
                return Err(format!("Plugin '{}' not found", id));
            }
            let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            let history: Vec<_> = registry.get_history(id, limit).into_iter().cloned().collect();
            serde_json::to_value(history).map_err(|e| e.to_string())
        }
        "plugin_register" => {
            let meta: crate::plugin_registry::PluginMeta = serde_json::from_value(params.clone())
                .map_err(|e| format!("Invalid PluginMeta: {}", e))?;
            let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            registry.register(meta)?;
            Ok(serde_json::json!({ "registered": true }))
        }
        "plugin_unregister" => {
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            let removed = registry.unregister(id)?;
            serde_json::to_value(removed).map_err(|e| e.to_string())
        }
        "plugin_set_enabled" => {
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            let enabled = params.get("enabled").and_then(|v| v.as_bool())
                .ok_or("Missing required param: 'enabled'")?;
            let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            registry.set_enabled(id, enabled)?;
            Ok(serde_json::json!({ "id": id, "enabled": enabled }))
        }
        "plugin_update_config" => {
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            let config = params.get("config").cloned()
                .ok_or("Missing required param: 'config'")?;
            let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
            registry.update_config(id, config)?;
            Ok(serde_json::json!({ "id": id, "updated": true }))
        }

        // =====================================================================
        // Swarm Orchestrator
        // =====================================================================
        "swarm_list_agents" => {
            let swarm = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
            let agents: Vec<_> = swarm.list_agents().into_iter().cloned().collect();
            serde_json::to_value(agents).map_err(|e| e.to_string())
        }
        "swarm_get_health" => {
            let swarm = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
            let health = swarm.get_health();
            serde_json::to_value(health).map_err(|e| e.to_string())
        }
        "swarm_get_task" => {
            let swarm = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
            let task_id = params.get("task_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'task_id'")?;
            let task = swarm.get_task(task_id)
                .cloned()
                .ok_or_else(|| format!("Task '{}' not found", task_id))?;
            serde_json::to_value(task).map_err(|e| e.to_string())
        }
        "swarm_list_tasks" => {
            let swarm = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
            let tasks: Vec<_> = swarm.list_tasks().into_iter().cloned().collect();
            serde_json::to_value(tasks).map_err(|e| e.to_string())
        }
        "swarm_register_agent" => {
            let agent: crate::swarm_orchestrator::SwarmAgent = serde_json::from_value(params.clone())
                .map_err(|e| format!("Invalid SwarmAgent: {}", e))?;
            let mut swarm = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
            swarm.register_agent(agent)?;
            Ok(serde_json::json!({ "registered": true }))
        }
        "swarm_create_task" => {
            let description = params.get("description").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'description'")?.to_string();
            let topology = params.get("topology").and_then(|v| v.as_str())
                .map(|t| t.parse::<crate::swarm_orchestrator::SwarmTopology>())
                .transpose()?;
            let consensus = params.get("consensus").and_then(|v| v.as_str())
                .map(|c| c.parse::<crate::swarm_orchestrator::ConsensusStrategy>())
                .transpose()?;
            let mut swarm = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
            let task = swarm.create_task(description, topology, consensus)?;
            serde_json::to_value(task).map_err(|e| e.to_string())
        }
        "swarm_cancel_task" => {
            let task_id = params.get("task_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'task_id'")?;
            let mut swarm = state.swarm_orchestrator.lock().map_err(|e| e.to_string())?;
            swarm.cancel_task(task_id)?;
            Ok(serde_json::json!({ "task_id": task_id, "cancelled": true }))
        }

        // =====================================================================
        // Workflow Engine
        // =====================================================================
        "workflow_list" => {
            let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
            let workflows: Vec<_> = engine.list_workflows().into_iter().cloned().collect();
            serde_json::to_value(workflows).map_err(|e| e.to_string())
        }
        "workflow_get" => {
            let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            let workflow = engine.get_workflow(id)
                .cloned()
                .ok_or_else(|| format!("Workflow '{}' not found", id))?;
            serde_json::to_value(workflow).map_err(|e| e.to_string())
        }
        "workflow_get_progress" => {
            let engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            let progress = engine.get_progress(id)?;
            serde_json::to_value(progress).map_err(|e| e.to_string())
        }
        "workflow_cancel" => {
            let id = params.get("id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'id'")?;
            let mut engine = state.workflow_engine.lock().map_err(|e| e.to_string())?;
            engine.cancel_workflow(id)?;
            Ok(serde_json::json!({ "id": id, "cancelled": true }))
        }

        // =====================================================================
        // MCP Client
        // =====================================================================
        "mcp_connected_servers" => {
            let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
            let servers: Vec<String> = client.connected_servers().into_iter().map(|s| s.to_string()).collect();
            serde_json::to_value(servers).map_err(|e| e.to_string())
        }
        "mcp_list_tools" => {
            let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
            let tools: Vec<_> = client.list_tools().into_iter().cloned().collect();
            serde_json::to_value(tools).map_err(|e| e.to_string())
        }
        "mcp_get_stats" => {
            let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
            let stats = client.get_stats();
            serde_json::to_value(stats).map_err(|e| e.to_string())
        }
        "mcp_is_connected" => {
            let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
            let server_name = params.get("server_name").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'server_name'")?;
            Ok(serde_json::json!({ "server_name": server_name, "connected": client.is_connected(server_name) }))
        }

        // =====================================================================
        // Agent Bus
        // =====================================================================
        "agent_bus_list_agents" => {
            let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            let agents = bus.list_agents();
            serde_json::to_value(agents).map_err(|e| e.to_string())
        }
        "agent_bus_list_topics" => {
            let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            let topics = bus.list_topics();
            serde_json::to_value(topics).map_err(|e| e.to_string())
        }
        "agent_bus_stats" => {
            let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            let stats = bus.get_stats();
            serde_json::to_value(stats).map_err(|e| e.to_string())
        }
        "agent_bus_history" => {
            let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            let limit = params.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);
            let history: Vec<_> = bus.get_history(limit).into_iter().cloned().collect();
            serde_json::to_value(history).map_err(|e| e.to_string())
        }
        "agent_bus_inbox_count" => {
            let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            let agent_id = params.get("agent_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'agent_id'")?;
            Ok(serde_json::json!({ "agent_id": agent_id, "count": bus.inbox_count(agent_id) }))
        }
        "agent_bus_is_registered" => {
            let bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            let agent_id = params.get("agent_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'agent_id'")?;
            Ok(serde_json::json!({ "agent_id": agent_id, "registered": bus.is_registered(agent_id) }))
        }
        "agent_bus_register" => {
            let agent_id = params.get("agent_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'agent_id'")?;
            let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            bus.register_agent(agent_id)?;
            Ok(serde_json::json!({ "agent_id": agent_id, "registered": true }))
        }
        "agent_bus_unregister" => {
            let agent_id = params.get("agent_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'agent_id'")?;
            let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            bus.unregister_agent(agent_id)?;
            Ok(serde_json::json!({ "agent_id": agent_id, "unregistered": true }))
        }
        "agent_bus_subscribe" => {
            let agent_id = params.get("agent_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'agent_id'")?;
            let topic = params.get("topic").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'topic'")?;
            let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            bus.subscribe(agent_id, topic)?;
            Ok(serde_json::json!({ "agent_id": agent_id, "topic": topic, "subscribed": true }))
        }
        "agent_bus_publish" => {
            let from_agent = params.get("from_agent").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'from_agent'")?;
            let topic = params.get("topic").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'topic'")?;
            let payload = params.get("payload").cloned().unwrap_or(serde_json::json!({}));
            let mut bus = state.agent_bus.lock().map_err(|e| e.to_string())?;
            let count = bus.publish(from_agent, topic, payload)?;
            Ok(serde_json::json!({ "topic": topic, "recipients": count }))
        }

        // =====================================================================
        // Healing Executor
        // =====================================================================
        "healing_list_actions" => {
            let healer = state.healing_executor.lock().map_err(|e| e.to_string())?;
            let actions = healer.get_action_history();
            serde_json::to_value(actions).map_err(|e| e.to_string())
        }
        "healing_get_stats" => {
            let healer = state.healing_executor.lock().map_err(|e| e.to_string())?;
            Ok(healer.get_stats())
        }

        // =====================================================================
        // Hooks Executor
        // =====================================================================
        "hook_list" => {
            let hooks = state.hooks_executor.lock().map_err(|e| e.to_string())?;
            let hook_list: Vec<_> = hooks.list_hooks().into_iter().cloned().collect();
            serde_json::to_value(hook_list).map_err(|e| e.to_string())
        }
        "hook_get_agent_hooks" => {
            let hooks = state.hooks_executor.lock().map_err(|e| e.to_string())?;
            let agent_id = params.get("agent_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'agent_id'")?;
            let agent_hooks: Vec<_> = hooks.get_agent_hooks(agent_id).into_iter().cloned().collect();
            serde_json::to_value(agent_hooks).map_err(|e| e.to_string())
        }

        // =====================================================================
        // Circuit Breaker
        // =====================================================================
        "circuit_breaker_get_all" => {
            let cb = state.circuit_breaker_manager.lock().map_err(|e| e.to_string())?;
            let all = cb.get_all_states();
            serde_json::to_value(all).map_err(|e| e.to_string())
        }
        "circuit_breaker_get_status" => {
            let cb = state.circuit_breaker_manager.lock().map_err(|e| e.to_string())?;
            let target = params.get("target").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'target'")?;
            let record = cb.get_breaker(target);
            serde_json::to_value(record).map_err(|e| e.to_string())
        }
        "circuit_breaker_reset" => {
            let target = params.get("target").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'target'")?;
            let cb = state.circuit_breaker_manager.lock().map_err(|e| e.to_string())?;
            cb.reset(target);
            Ok(serde_json::json!({ "target": target, "reset": true }))
        }

        // =====================================================================
        // DAG Engine
        // =====================================================================
        "dag_get_plan" => {
            let dag = state.dag_engine.lock().map_err(|e| e.to_string())?;
            let plan_id = params.get("plan_id").and_then(|v| v.as_str())
                .ok_or("Missing required param: 'plan_id'")?;
            let plan = dag.get_plan(plan_id)
                .cloned()
                .ok_or_else(|| format!("Plan '{}' not found", plan_id))?;
            serde_json::to_value(plan).map_err(|e| e.to_string())
        }

        _ => Err(format!("Unknown proxy command: '{}'. Use 'list_proxy_commands' to discover available commands.", command)),
    }
}

/// Handle command with request/response (new format)
async fn handle_remote_command(
    client_id: &str,
    request: &RemoteRequest,
    app_handle: &AppHandle,
    connections: &RwLock<HashMap<String, ConnectionState>>,
) {
    println!("Remote command from {}: {:?}", client_id, request.command);

    // Get state from app_handle for real data
    use crate::AppState;
    use tauri::Manager;

    let state = app_handle.state::<AppState>();

    let response = match request.command.as_str() {
        "get_status" => {
            // Return real status snapshot
            let config_manager = state.config_manager.read();
            let agent_count = config_manager
                .as_ref()
                .map(|cm| cm.get_config().agents.len())
                .unwrap_or(0);

            // Safely get database statistics
            let (running_tasks, total_tasks) = {
                let db_guard = state.database.lock().ok();
                if let Some(guard) = db_guard {
                    if let Some(database) = guard.as_ref() {
                        match database.get_statistics() {
                            Ok(stats) => (stats.running_tasks, stats.total_tasks),
                            Err(_) => (0, 0),
                        }
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                }
            };

            let ws_running = {
                let ws_guard = state.ws_server.lock().ok();
                ws_guard
                    .and_then(|g| g.as_ref().map(|w| w.is_running()))
                    .unwrap_or(false)
            };

            let client_count = {
                let ws_guard = state.ws_server.lock().ok();
                ws_guard
                    .and_then(|g| g.as_ref().map(|w| w.get_clients().len()))
                    .unwrap_or(0)
            };

            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({
                    "agents_configured": agent_count,
                    "running_tasks": running_tasks,
                    "total_tasks": total_tasks,
                    "ws_server_running": ws_running,
                    "connected_clients": client_count
                })),
                error: None,
            }
        }
        "list_agents" => {
            // Return configured agents
            let config_manager = state.config_manager.read();
            let agents = config_manager
                .as_ref()
                .map(|cm| {
                    cm.get_config()
                        .agents
                        .iter()
                        .map(|(name, cfg)| {
                            serde_json::json!({
                                "name": name,
                                "transport": match cfg.transport {
                                    crate::config::AgentTransport::Stdio => {
                                        serde_json::json!({
                                            "type": "stdio",
                                            "command": cfg.command,
                                            "args": cfg.args
                                        })
                                    },
                                    crate::config::AgentTransport::Websocket => {
                                        serde_json::json!({
                                            "type": "websocket",
                                            "url": cfg.url
                                        })
                                    },
                                    crate::config::AgentTransport::Http => {
                                        serde_json::json!({
                                            "type": "http",
                                            "url": cfg.url
                                        })
                                    }
                                }
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "agents": agents })),
                error: None,
            }
        }
        "list_plans" => {
            // Plans are managed by frontend team-runtime, return empty for now
            // Frontend should query team-runtime store directly
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(
                    serde_json::json!({ "plans": [], "note": "Plans managed by frontend team-runtime" }),
                ),
                error: None,
            }
        }
        "pause_agent" | "resume_agent" | "cancel_agent" | "inject_message" => {
            // Forward to frontend for handling (agent management is in frontend)
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": request.command,
                    "request_id": request.id,
                    "client_id": client_id,
                    "payload": request.payload,
                }),
            );
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "forwarded": true })),
                error: None,
            }
        }
        "pause_plan_node" | "resume_plan_node" => {
            // Forward to frontend for handling (plan management is in frontend)
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": request.command,
                    "request_id": request.id,
                    "client_id": client_id,
                    "payload": request.payload,
                }),
            );
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "forwarded": true })),
                error: None,
            }
        }
        "subscribe_logs" => {
            // Subscribe to log stream for an agent
            // Mark this client as subscribed for log batch push
            if let Some(cs) = connections.write().get_mut(client_id) {
                cs.subscribed_to_logs = true;
            }

            let log_manager = state.log_stream_manager.lock().ok();
            if let Some(guard) = log_manager {
                if let Some(manager) = guard.as_ref() {
                    // Subscribe to all logs or specific agent
                    if let Some(payload) = &request.payload {
                        if let Some(agent_id) = payload.get("agent_id").and_then(|v| v.as_str()) {
                            manager.subscribe(agent_id);
                        } else {
                            // Subscribe to all agents (use wildcard)
                            manager.subscribe("*");
                        }
                    } else {
                        manager.subscribe("*");
                    }
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: true,
                        data: Some(serde_json::json!({ "subscribed": true })),
                        error: None,
                    }
                } else {
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: false,
                        data: None,
                        error: Some("LogStreamManager not initialized".to_string()),
                    }
                }
            } else {
                RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Failed to lock LogStreamManager".to_string()),
                }
            }
        }
        "unsubscribe_logs" => {
            // Unsubscribe from log stream
            // Mark this client as unsubscribed
            if let Some(cs) = connections.write().get_mut(client_id) {
                cs.subscribed_to_logs = false;
            }

            let log_manager = state.log_stream_manager.lock().ok();
            if let Some(guard) = log_manager {
                if let Some(manager) = guard.as_ref() {
                    if let Some(payload) = &request.payload {
                        if let Some(agent_id) = payload.get("agent_id").and_then(|v| v.as_str()) {
                            manager.unsubscribe(agent_id);
                        } else {
                            manager.unsubscribe("*");
                        }
                    } else {
                        manager.unsubscribe("*");
                    }
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: true,
                        data: Some(serde_json::json!({ "unsubscribed": true })),
                        error: None,
                    }
                } else {
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: false,
                        data: None,
                        error: Some("LogStreamManager not initialized".to_string()),
                    }
                }
            } else {
                RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Failed to lock LogStreamManager".to_string()),
                }
            }
        }
        "get_logs" => {
            // Get logs with optional filters
            let log_manager = state.log_stream_manager.lock().ok();
            if let Some(guard) = log_manager {
                if let Some(manager) = guard.as_ref() {
                    let limit = request
                        .payload
                        .as_ref()
                        .and_then(|p| p.get("limit").and_then(|v| v.as_u64()));
                    let log_type = request
                        .payload
                        .as_ref()
                        .and_then(|p| p.get("log_type").and_then(|v| v.as_str()));

                    let logs = if let Some(lt) = log_type {
                        manager.get_logs_from_db(
                            None,
                            Some(crate::log_stream::LogType::from_str(lt)),
                            limit,
                        )
                    } else {
                        manager.get_latest_logs(limit.unwrap_or(100) as usize)
                    };

                    RemoteResponse {
                        id: request.id.clone(),
                        ok: true,
                        data: Some(serde_json::json!({ "logs": logs })),
                        error: None,
                    }
                } else {
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: false,
                        data: None,
                        error: Some("LogStreamManager not initialized".to_string()),
                    }
                }
            } else {
                RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Failed to lock LogStreamManager".to_string()),
                }
            }
        }
        "search_logs" => {
            // Search logs by keyword
            let log_manager = state.log_stream_manager.lock().ok();
            if let Some(guard) = log_manager {
                if let Some(manager) = guard.as_ref() {
                    let keyword = request
                        .payload
                        .as_ref()
                        .and_then(|p| p.get("keyword").and_then(|v| v.as_str()))
                        .unwrap_or("");
                    let limit = request
                        .payload
                        .as_ref()
                        .and_then(|p| p.get("limit").and_then(|v| v.as_u64()));

                    let logs = manager.search_logs(keyword, limit);

                    RemoteResponse {
                        id: request.id.clone(),
                        ok: true,
                        data: Some(serde_json::json!({ "logs": logs })),
                        error: None,
                    }
                } else {
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: false,
                        data: None,
                        error: Some("LogStreamManager not initialized".to_string()),
                    }
                }
            } else {
                RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Failed to lock LogStreamManager".to_string()),
                }
            }
        }
        "init_executive_agent" => {
            let workspace = request
                .payload
                .as_ref()
                .and_then(|p| p.get("workspace").and_then(|v| v.as_str()))
                .unwrap_or("D:/dingsun/acp-ui/erp_system");

            let workspace_path = std::path::PathBuf::from(workspace);

            // Create workspace directory
            if !workspace_path.exists() {
                if let Err(e) = std::fs::create_dir_all(&workspace_path) {
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: false,
                        data: None,
                        error: Some(format!("创建工作目录失败: {}", e)),
                    }
                } else {
                    let manager =
                        crate::executive_agent::ExecutiveAgentManager::new(workspace_path);
                    *state.executive_agent_manager.lock().unwrap() = Some(manager);

                    RemoteResponse {
                        id: request.id.clone(),
                        ok: true,
                        data: Some(
                            serde_json::json!({ "workspace": workspace, "initialized": true }),
                        ),
                        error: None,
                    }
                }
            } else {
                let manager = crate::executive_agent::ExecutiveAgentManager::new(workspace_path);
                *state.executive_agent_manager.lock().unwrap() = Some(manager);

                RemoteResponse {
                    id: request.id.clone(),
                    ok: true,
                    data: Some(serde_json::json!({ "workspace": workspace, "initialized": true })),
                    error: None,
                }
            }
        }
        "list_executive_sessions" => {
            // Query executive sessions from database
            let db_guard = state.database.lock().ok();
            if let Some(guard) = db_guard {
                if let Some(database) = guard.as_ref() {
                    let limit = request
                        .payload
                        .as_ref()
                        .and_then(|p| p.get("limit").and_then(|v| v.as_u64()))
                        .map(Some)
                        .unwrap_or(Some(50));

                    match database.load_executive_sessions(limit) {
                        Ok(sessions) => {
                            // Convert to JSON format expected by frontend
                            let session_list: Vec<serde_json::Value> = sessions
                                .iter()
                                .map(|s| {
                                    serde_json::json!({
                                        "id": s.id,
                                        "request": s.request,
                                        "workspace": s.workspace,
                                        "status": s.status,
                                        "summary": s.summary,
                                        "files_json": s.files_json,
                                        "logs_json": s.logs_json,
                                        "created_at": s.created_at,
                                        "completed_at": s.completed_at
                                    })
                                })
                                .collect();

                            RemoteResponse {
                                id: request.id.clone(),
                                ok: true,
                                data: Some(serde_json::json!({
                                    "sessions": session_list,
                                    "count": session_list.len()
                                })),
                                error: None,
                            }
                        }
                        Err(e) => RemoteResponse {
                            id: request.id.clone(),
                            ok: false,
                            data: None,
                            error: Some(format!("Failed to load sessions: {}", e)),
                        },
                    }
                } else {
                    RemoteResponse {
                        id: request.id.clone(),
                        ok: false,
                        data: None,
                        error: Some("Database not initialized".to_string()),
                    }
                }
            } else {
                RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Failed to lock database".to_string()),
                }
            }
        }
        "execute_development_task" => {
            // Directly execute the task using Hermes Native (not just forward to frontend)
            // Convert to owned values for async spawn
            let request_text = request
                .payload
                .as_ref()
                .and_then(|p| p.get("request").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string(); // Convert to owned String

            // Get workspace from manager
            let workspace_path = {
                let manager_guard = state.executive_agent_manager.lock().unwrap();
                match manager_guard.as_ref() {
                    Some(mgr) => mgr.workspace.clone(),
                    None => PathBuf::from("D:/dingsun/test_workspace"),
                }
            };

            // Create a new manager for this execution
            let manager = crate::executive_agent::ExecutiveAgentManager::new(workspace_path);

            // Emit task started event
            let _ = app_handle.emit(
                "remote-task-started",
                serde_json::json!({
                    "request_id": request.id,
                    "client_id": client_id.to_string(),
                    "request": request_text,
                }),
            );

            // Execute asynchronously and send result back
            let app_handle_clone = app_handle.clone();
            let request_id = request.id.clone();
            let client_id_owned = client_id.to_string(); // Convert to owned String

            // Get database reference for saving records
            let db_ref = state.database.clone();

            tokio::spawn(async move {
                let result = manager
                    .execute_workflow(request_text.clone(), app_handle_clone.clone())
                    .await;

                match result {
                    Ok(task_result) => {
                        // Save execution record to database for persistence and traceability
                        let session_record = crate::database::ExecutiveSessionRecord {
                            id: task_result.task_id.clone(),
                            request: task_result.request.clone(),
                            workspace: task_result.workspace.clone(),
                            status: "completed".to_string(),
                            summary: Some(task_result.summary.clone()),
                            files_json: Some(
                                serde_json::to_string(&task_result.files).unwrap_or_default(),
                            ),
                            logs_json: Some(
                                serde_json::to_string(&task_result.logs).unwrap_or_default(),
                            ),
                            created_at: chrono::Utc::now().to_rfc3339(),
                            completed_at: Some(task_result.completed_at.to_rfc3339()),
                        };

                        // Save session record to database
                        if let Some(db) = db_ref.lock().unwrap().as_ref() {
                            if let Err(e) = db.save_executive_session(&session_record) {
                                eprintln!("Failed to save executive session: {}", e);
                            }

                            // Save thinking chunks separately for traceability
                            for chunk in manager.get_thinking_chunks() {
                                if let Err(e) = db.save_thinking_chunk(&chunk) {
                                    eprintln!("Failed to save thinking chunk: {}", e);
                                }
                            }

                            // Save tool calls separately for traceability
                            for tool_call in manager.get_tool_calls() {
                                if let Err(e) = db.save_tool_call(&tool_call) {
                                    eprintln!("Failed to save tool call: {}", e);
                                }
                            }
                        }

                        // Send success response with full result
                        let _ = app_handle_clone.emit(
                            "remote-task-completed",
                            serde_json::json!({
                                "request_id": request_id,
                                "client_id": client_id_owned,
                                "result": task_result,
                                "thinking_chunks": manager.get_thinking_chunks(),
                                "tool_calls": manager.get_tool_calls(),
                                "success": true,
                                "saved_to_db": true,
                            }),
                        );
                    }
                    Err(e) => {
                        // Save error record to database
                        let session_record = crate::database::ExecutiveSessionRecord {
                            id: request_id.clone(),
                            request: request_text,
                            workspace: "unknown".to_string(),
                            status: "error".to_string(),
                            summary: None,
                            files_json: None,
                            logs_json: None,
                            created_at: chrono::Utc::now().to_rfc3339(),
                            completed_at: None,
                        };

                        if let Some(db) = db_ref.lock().unwrap().as_ref() {
                            if let Err(save_err) = db.save_executive_session(&session_record) {
                                eprintln!("Failed to save error session: {}", save_err);
                            }
                        }

                        // Send error response
                        let _ = app_handle_clone.emit(
                            "remote-task-error",
                            serde_json::json!({
                                "request_id": request_id,
                                "client_id": client_id_owned,
                                "error": e,
                                "success": false,
                                "saved_to_db": true,
                            }),
                        );
                    }
                }
            });

            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({
                    "forwarded": true,
                    "executing": true,
                    "message": "任务已开始执行，请监听 remote-task-completed 事件获取结果"
                })),
                error: None,
            }
        }
        "get_executive_agent_status" => {
            let manager = state.executive_agent_manager.lock().unwrap();
            if let Some(mgr) = manager.as_ref() {
                let status = mgr.get_agent_status();
                RemoteResponse {
                    id: request.id.clone(),
                    ok: true,
                    data: Some(serde_json::json!({ "status": status })),
                    error: None,
                }
            } else {
                RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Executive Agent Manager 未初始化".to_string()),
                }
            }
        }
        "get_generated_files" => {
            let manager = state.executive_agent_manager.lock().unwrap();
            if let Some(mgr) = manager.as_ref() {
                let files = mgr.get_generated_files();
                RemoteResponse {
                    id: request.id.clone(),
                    ok: true,
                    data: Some(serde_json::json!({ "files": files })),
                    error: None,
                }
            } else {
                RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some("Executive Agent Manager 未初始化".to_string()),
                }
            }
        }
        "clear_executive_agent" => {
            let manager = state.executive_agent_manager.lock().unwrap();
            if let Some(mgr) = manager.as_ref() {
                mgr.clear();
            }
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "cleared": true })),
                error: None,
            }
        }
        "proxy" => {
            // Generic command proxy - dispatch to any registered backend module
            let proxy_command = request
                .payload
                .as_ref()
                .and_then(|p| p.get("command").and_then(|v| v.as_str()))
                .unwrap_or("");
            let proxy_params = request
                .payload
                .as_ref()
                .and_then(|p| p.get("params").cloned())
                .unwrap_or(serde_json::json!({}));

            match handle_proxy_command(&state, proxy_command, &proxy_params) {
                Ok(data) => RemoteResponse {
                    id: request.id.clone(),
                    ok: true,
                    data: Some(data),
                    error: None,
                },
                Err(e) => RemoteResponse {
                    id: request.id.clone(),
                    ok: false,
                    data: None,
                    error: Some(e),
                },
            }
        }
        "list_proxy_commands" => {
            let commands = get_proxy_command_list();
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "commands": commands })),
                error: None,
            }
        }
        unknown => RemoteResponse {
            id: request.id.clone(),
            ok: false,
            data: None,
            error: Some(format!("Unknown command: {}", unknown)),
        },
    };

    send_response(connections, client_id, &response);
}

/// Handle legacy command format (no response)
fn handle_legacy_command(client_id: &str, command: RemoteCommand, app_handle: &AppHandle) {
    println!("Legacy remote command from {}: {:?}", client_id, command);

    match command {
        RemoteCommand::PauseAgent { agent_id } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "pause_agent", "agent_id": agent_id, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::ResumeAgent { agent_id } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "resume_agent", "agent_id": agent_id, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::CancelAgent { agent_id } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "cancel_agent", "agent_id": agent_id, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::InjectMessage { agent_id, message } => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "inject_message", "agent_id": agent_id, "message": message, "client_id": client_id,
            }));
        }
        RemoteCommand::PausePlanNode { task_id, node_id } => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "pause_plan_node", "task_id": task_id, "node_id": node_id, "client_id": client_id,
            }));
        }
        RemoteCommand::ResumePlanNode { task_id, node_id } => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "resume_plan_node", "task_id": task_id, "node_id": node_id, "client_id": client_id,
            }));
        }
        RemoteCommand::GetAgents {} => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "get_agents", "client_id": client_id,
                }),
            );
        }
        RemoteCommand::GetPlans {} => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "get_plans", "client_id": client_id,
                }),
            );
        }
        RemoteCommand::GetStatus {} => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "get_status", "client_id": client_id,
                }),
            );
        }
        // LogStream commands (legacy format)
        RemoteCommand::SubscribeLogs { agent_id } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "subscribe_logs", "agent_id": agent_id, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::UnsubscribeLogs { agent_id } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "unsubscribe_logs", "agent_id": agent_id, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::GetLogs { limit, log_type } => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "get_logs", "limit": limit, "log_type": log_type, "client_id": client_id,
            }));
        }
        RemoteCommand::SearchLogs { keyword, limit } => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "search_logs", "keyword": keyword, "limit": limit, "client_id": client_id,
            }));
        }
        // Executive Agent commands (legacy format)
        RemoteCommand::InitExecutiveAgent { workspace } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "init_executive_agent", "workspace": workspace, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::ExecuteDevelopmentTask { request } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "execute_development_task", "request": request, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::GetExecutiveAgentStatus {} => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "get_executive_agent_status", "client_id": client_id,
                }),
            );
        }
        RemoteCommand::GetGeneratedFiles {} => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "get_generated_files", "client_id": client_id,
                }),
            );
        }
        RemoteCommand::ClearExecutiveAgent {} => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "clear_executive_agent", "client_id": client_id,
                }),
            );
        }
        RemoteCommand::Proxy { cmd, params } => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "proxy", "command": cmd, "params": params, "client_id": client_id,
                }),
            );
        }
        RemoteCommand::ListProxyCommands {} => {
            let _ = app_handle.emit(
                "remote-command",
                serde_json::json!({
                    "type": "list_proxy_commands", "client_id": client_id,
                }),
            );
        }
    }
}
