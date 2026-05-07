use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use tauri::{AppHandle, Emitter};

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
    PauseAgent { agent_id: String },
    ResumeAgent { agent_id: String },
    CancelAgent { agent_id: String },
    InjectMessage { agent_id: String, message: String },
    PausePlanNode { task_id: String, node_id: String },
    ResumePlanNode { task_id: String, node_id: String },
    GetAgents {},
    GetPlans {},
    GetStatus {},
}

/// Client connection state (stored per-connection)
struct ConnectionState {
    authenticated: bool,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

/// WebSocket server for remote connections with token authentication
pub struct WebSocketServer {
    port: u16,
    clients: Arc<RwLock<HashMap<String, RemoteClient>>>,
    connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
    running: Arc<RwLock<bool>>,
    auth_token: Arc<RwLock<Option<String>>>,
}

impl WebSocketServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            clients: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            auth_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the auth token that clients must present to connect
    pub fn set_auth_token(&self, token: Option<String>) {
        *self.auth_token.write() = token;
    }

    /// Start the WebSocket server
    pub async fn start(&self, app_handle: AppHandle) -> Result<String, String> {
        if *self.running.read() {
            return Err("Server already running".to_string());
        }

        // Bind to localhost only for security (not all network interfaces)
        // This prevents exposure to LAN/external networks
        let addr: SocketAddr = format!("127.0.0.1:{}", self.port)
            .parse()
            .map_err(|e| format!("Invalid address: {}", e))?;

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind: {}", e))?;

        *self.running.write() = true;

        let _ = app_handle.emit("ws-server-started", self.port);

        let clients = Arc::clone(&self.clients);
        let connections = Arc::clone(&self.connections);
        let running = Arc::clone(&self.running);
        let auth_token = Arc::clone(&self.auth_token);

        tokio::spawn(async move {
            while *running.read() {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let clients_clone = Arc::clone(&clients);
                        let connections_clone = Arc::clone(&connections);
                        let handle_clone = app_handle.clone();
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
    connections.write().insert(client_id.clone(), ConnectionState {
        authenticated: false,
        sender: tx,
    });

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
                                println!("Client {} disconnected: too many unauthenticated messages", client_id);
                                break;
                            }
                            send_response(&connections, &client_id, &RemoteResponse {
                                id: request.id,
                                ok: false,
                                data: None,
                                error: Some("Not authenticated. Send {type:'auth', token:'...'} first.".to_string()),
                            });
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
                                    send_response(&connections, &client_id, &RemoteResponse {
                                        id: request.id,
                                        ok: true,
                                        data: Some(serde_json::json!({ "client_id": client_id })),
                                        error: None,
                                    });
                                    continue;
                                }
                            }
                            send_response(&connections, &client_id, &RemoteResponse {
                                id: request.id,
                                ok: false,
                                data: None,
                                error: Some("Invalid token".to_string()),
                            });
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
                        content: serde_json::from_str(&text_str).unwrap_or_else(|_| {
                            serde_json::json!({ "raw": text_str })
                        }),
                        timestamp: chrono::Utc::now(),
                    };
                    let _ = app_handle.emit("remote-message", remote_msg);
                }
            }
            Ok(Message::Binary(data)) => {
                let _ = app_handle.emit("remote-binary", serde_json::json!({
                    "client_id": client_id,
                    "data_length": data.len(),
                }));
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

fn send_response(connections: &RwLock<HashMap<String, ConnectionState>>, client_id: &str, response: &RemoteResponse) {
    if let Some(cs) = connections.read().get(client_id) {
        if let Ok(json) = serde_json::to_string(response) {
            let _ = cs.sender.send(Message::Text(json.into()));
        }
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
    use tauri::Manager;
    use crate::AppState;

    let state = app_handle.state::<AppState>();

    let response = match request.command.as_str() {
        "get_status" => {
            // Return real status snapshot
            let config_manager = state.config_manager.read();
            let agent_count = config_manager.as_ref()
                .map(|cm| cm.get_config().agents.len())
                .unwrap_or(0);

            // Safely get database statistics
            let (running_tasks, total_tasks) = {
                let db_guard = state.database.lock().ok();
                if let Some(guard) = db_guard {
                    if let Some(database) = guard.as_ref() {
                        match database.get_statistics() {
                            Ok(stats) => (stats.running_tasks, stats.total_tasks),
                            Err(_) => (0, 0)
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
                ws_guard.and_then(|g| g.as_ref().map(|w| w.is_running())).unwrap_or(false)
            };

            let client_count = {
                let ws_guard = state.ws_server.lock().ok();
                ws_guard.and_then(|g| g.as_ref().map(|w| w.get_clients().len())).unwrap_or(0)
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
        },
        "list_agents" => {
            // Return configured agents
            let config_manager = state.config_manager.read();
            let agents = config_manager.as_ref()
                .map(|cm| {
                    cm.get_config().agents.iter().map(|(name, cfg)| {
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
                    }).collect::<Vec<_>>()
                })
                .unwrap_or_default();

            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "agents": agents })),
                error: None,
            }
        },
        "list_plans" => {
            // Plans are managed by frontend team-runtime, return empty for now
            // Frontend should query team-runtime store directly
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "plans": [], "note": "Plans managed by frontend team-runtime" })),
                error: None,
            }
        },
        "pause_agent" | "resume_agent" | "cancel_agent" | "inject_message" => {
            // Forward to frontend for handling (agent management is in frontend)
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": request.command,
                "request_id": request.id,
                "client_id": client_id,
                "payload": request.payload,
            }));
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "forwarded": true })),
                error: None,
            }
        },
        "pause_plan_node" | "resume_plan_node" => {
            // Forward to frontend for handling (plan management is in frontend)
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": request.command,
                "request_id": request.id,
                "client_id": client_id,
                "payload": request.payload,
            }));
            RemoteResponse {
                id: request.id.clone(),
                ok: true,
                data: Some(serde_json::json!({ "forwarded": true })),
                error: None,
            }
        },
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
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "pause_agent", "agent_id": agent_id, "client_id": client_id,
            }));
        }
        RemoteCommand::ResumeAgent { agent_id } => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "resume_agent", "agent_id": agent_id, "client_id": client_id,
            }));
        }
        RemoteCommand::CancelAgent { agent_id } => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "cancel_agent", "agent_id": agent_id, "client_id": client_id,
            }));
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
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "get_agents", "client_id": client_id,
            }));
        }
        RemoteCommand::GetPlans {} => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "get_plans", "client_id": client_id,
            }));
        }
        RemoteCommand::GetStatus {} => {
            let _ = app_handle.emit("remote-command", serde_json::json!({
                "type": "get_status", "client_id": client_id,
            }));
        }
    }
}
