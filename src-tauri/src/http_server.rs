// HTTP Server - REST API for Web/Mobile platforms
//
// Phase 7: HTTP Server for multi-platform access
// Provides REST API + WebSocket for Web and Mobile clients

use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use futures_util::{SinkExt, StreamExt};

/// HTTP Server state
pub struct HttpServerState {
    /// Event broadcaster for WebSocket
    pub event_tx: broadcast::Sender<EventMessage>,
    /// Server start time
    pub start_time: chrono::DateTime<chrono::Utc>,
}

/// Event message for WebSocket broadcast
#[derive(Debug, Clone, Serialize)]
pub struct EventMessage {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    error: String,
    code: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(self)).into_response()
    }
}

/// Agent summary for API response
#[derive(Debug, Clone, Serialize)]
pub struct AgentSummary {
    id: String,
    name: String,
    adapter_type: String,
    status: String,
    health_score: f32,
    capabilities: Vec<String>,
}

/// Cost summary for API response
#[derive(Debug, Serialize)]
pub struct CostSummaryResponse {
    daily_total: f32,
    monthly_total: f32,
    currency: String,
    budget_remaining_percent: f32,
}

/// One-shot request
#[derive(Debug, Deserialize)]
pub struct OneShotApiRequest {
    input: String,
    context_hint: Option<String>,
    preferred_agent: Option<String>,
    max_cost: Option<f32>,
    timeout_ms: Option<u64>,
}

/// One-shot response
#[derive(Debug, Serialize)]
pub struct OneShotApiResponse {
    detected_role: String,
    detected_scene: String,
    selected_agent: String,
    selection_reason: String,
    result_preview: String,
    estimated_cost: f32,
    transparency: TransparencyInfoApi,
}

/// Transparency info for API
#[derive(Debug, Serialize)]
pub struct TransparencyInfoApi {
    dag_visualization: String,
    estimated_duration_ms: u64,
    privacy_level: String,
}

/// Execute request
#[derive(Debug, Deserialize)]
pub struct ExecuteApiRequest {
    agent_id: String,
    description: String,
    input: String,
    timeout_ms: Option<u64>,
}

/// Execute response
#[derive(Debug, Serialize)]
pub struct ExecuteApiResponse {
    task_id: String,
    status: String,
    output_preview: String,
    cost: f32,
    duration_ms: u64,
}

/// Create HTTP router
pub fn create_router(state: Arc<HttpServerState>) -> Router {
    Router::new()
        // Agent endpoints
        .route("/api/agents", get(list_agents))
        .route("/api/agents/:id", get(get_agent))

        // One-Shot endpoint
        .route("/api/oneshot", post(execute_oneshot))

        // Cost endpoints
        .route("/api/costs/summary", get(get_cost_summary))

        // Execution endpoint
        .route("/api/execute", post(execute_task))

        // WebSocket endpoint
        .route("/ws/events", get(websocket_handler))

        // Health check
        .route("/api/health", get(health_check))

        // Add CORS for web clients
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
        )
        .with_state(state)
}

/// Available agents (static list)
fn get_available_agents() -> Vec<AgentSummary> {
    vec![
        AgentSummary {
            id: "claude-code".to_string(),
            name: "Claude Code".to_string(),
            adapter_type: "cli".to_string(),
            status: "available".to_string(),
            health_score: 95.0,
            capabilities: vec!["code_generation".to_string(), "code_review".to_string(), "bug_fix".to_string(), "refactoring".to_string()],
        },
        AgentSummary {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            adapter_type: "cli".to_string(),
            status: "available".to_string(),
            health_score: 90.0,
            capabilities: vec!["code_generation".to_string(), "bug_fix".to_string(), "testing".to_string()],
        },
        AgentSummary {
            id: "kimi".to_string(),
            name: "Kimi (Moonshot)".to_string(),
            adapter_type: "api".to_string(),
            status: "available".to_string(),
            health_score: 88.0,
            capabilities: vec!["copywriting".to_string(), "translation".to_string(), "summary".to_string()],
        },
        AgentSummary {
            id: "jimeng".to_string(),
            name: "即梦 (Jimeng)".to_string(),
            adapter_type: "api".to_string(),
            status: "available".to_string(),
            health_score: 85.0,
            capabilities: vec!["image_generation".to_string(), "marketing".to_string()],
        },
        AgentSummary {
            id: "kling".to_string(),
            name: "可灵 (Kling)".to_string(),
            adapter_type: "api".to_string(),
            status: "available".to_string(),
            health_score: 82.0,
            capabilities: vec!["video_generation".to_string(), "marketing".to_string()],
        },
        AgentSummary {
            id: "wps".to_string(),
            name: "WPS Office".to_string(),
            adapter_type: "api".to_string(),
            status: "available".to_string(),
            health_score: 80.0,
            capabilities: vec!["document_generation".to_string(), "excel_analysis".to_string(), "finance".to_string()],
        },
        AgentSummary {
            id: "unity".to_string(),
            name: "Unity Game".to_string(),
            adapter_type: "cli".to_string(),
            status: "available".to_string(),
            health_score: 78.0,
            capabilities: vec!["game_build".to_string(), "asset_management".to_string(), "3d".to_string()],
        },
        AgentSummary {
            id: "godot".to_string(),
            name: "Godot Engine".to_string(),
            adapter_type: "cli".to_string(),
            status: "available".to_string(),
            health_score: 75.0,
            capabilities: vec!["game_build".to_string(), "asset_management".to_string(), "2d_3d".to_string()],
        },
    ]
}

// ============================================================================
// Agent Endpoints
// ============================================================================

/// List all available agents
async fn list_agents() -> Json<Vec<AgentSummary>> {
    Json(get_available_agents())
}

/// Get specific agent details
async fn get_agent(
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<AgentSummary>, ApiError> {
    let agents = get_available_agents();
    let agent = agents.iter()
        .find(|a| a.id == id)
        .ok_or_else(|| ApiError {
            error: format!("Agent '{}' not found", id),
            code: 404,
        })?;

    Ok(Json((*agent).clone()))
}

// ============================================================================
// One-Shot Endpoint
// ============================================================================

/// Execute one-shot request (mock implementation)
async fn execute_oneshot(
    Json(request): Json<OneShotApiRequest>,
) -> Json<OneShotApiResponse> {
    // Detect user role (simple keyword matching)
    let role = detect_role(&request.input);
    let scene = detect_scene(&request.input);

    // Select best agent
    let (agent, reason) = select_agent_for_role(&role, &scene);

    // Generate preview response
    let preview = format!("将使用 {} 处理您的请求: {}", agent, &request.input[..50.min(request.input.len())]);
    let dag_visualization = format!("Input -> RoleDetection({}) -> AgentSelection({})", role, agent);

    Json(OneShotApiResponse {
        detected_role: role,
        detected_scene: scene,
        selected_agent: agent,
        selection_reason: reason,
        result_preview: preview,
        estimated_cost: 0.05,
        transparency: TransparencyInfoApi {
            dag_visualization,
            estimated_duration_ms: 30000,
            privacy_level: "standard".to_string(),
        },
    })
}

// ============================================================================
// Cost Endpoints
// ============================================================================

/// Get cost summary (mock)
async fn get_cost_summary() -> Json<CostSummaryResponse> {
    Json(CostSummaryResponse {
        daily_total: 2.35,
        monthly_total: 45.80,
        currency: "CNY".to_string(),
        budget_remaining_percent: 54.2,
    })
}

// ============================================================================
// Execution Endpoint
// ============================================================================

/// Execute task with specific agent (mock)
async fn execute_task(
    Json(request): Json<ExecuteApiRequest>,
) -> Json<ExecuteApiResponse> {
    let task_id = uuid::Uuid::new_v4().to_string();

    // Mock execution
    let output = format!("Task '{}' executed by agent '{}'", request.description, request.agent_id);

    Json(ExecuteApiResponse {
        task_id,
        status: "success".to_string(),
        output_preview: output,
        cost: 0.02,
        duration_ms: 1500,
    })
}

// ============================================================================
// WebSocket Endpoint
// ============================================================================

/// WebSocket handler for real-time events
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<HttpServerState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, state: Arc<HttpServerState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to events
    let mut event_rx = state.event_tx.subscribe();

    // Send initial connection message
    let initial = EventMessage {
        event_type: "connected".to_string(),
        data: serde_json::json!({
            "server_version": "0.1.14",
            "agents_available": get_available_agents().len(),
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if sender.send(Message::Text(serde_json::to_string(&initial).unwrap())).await.is_err() {
        return;
    }

    // Handle messages in a loop
    loop {
        tokio::select! {
            // Handle incoming messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let cmd: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::json!({}));

                        let response = match cmd.get("type").and_then(|t| t.as_str()) {
                        Some("ping") => EventMessage {
                            event_type: "pong".to_string(),
                            data: serde_json::json!({}),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                        Some("get_agents") => EventMessage {
                            event_type: "agents_list".to_string(),
                            data: serde_json::to_value(get_available_agents()).unwrap(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                        _ => EventMessage {
                            event_type: "unknown".to_string(),
                            data: serde_json::json!({"error": "Unknown command"}),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                    };

                        if sender.send(Message::Text(serde_json::to_string(&response).unwrap())).await.is_err() {
                            break;
                        }
                    },
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }

            // Broadcast events to client
            event = event_rx.recv() => {
                if let Ok(event) = event {
                    if sender.send(Message::Text(serde_json::to_string(&event).unwrap())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}

// ============================================================================
// Health Check
// ============================================================================

/// Health check endpoint
async fn health_check(State(state): State<Arc<HttpServerState>>) -> Json<serde_json::Value> {
    let duration = chrono::Utc::now() - state.start_time;
    Json(serde_json::json!({
        "status": "healthy",
        "version": "0.1.14",
        "uptime_seconds": duration.num_seconds(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Detect user role from input
fn detect_role(input: &str) -> String {
    let keywords: HashMap<&str, &str> = HashMap::from([
        ("代码", "developer"),
        ("编程", "developer"),
        ("bug", "developer"),
        ("API", "developer"),
        ("函数", "developer"),
        ("文案", "marketer"),
        ("营销", "marketer"),
        ("抖音", "marketer"),
        ("设计", "designer"),
        ("UI", "designer"),
        ("财务", "finance"),
        ("Excel", "finance"),
        ("游戏", "gamer"),
        ("Unity", "gamer"),
        ("Godot", "gamer"),
        ("写作", "writer"),
        ("翻译", "writer"),
        ("分析", "analyst"),
        ("数据", "analyst"),
    ]);

    for (keyword, role) in &keywords {
        if input.contains(keyword) {
            return role.to_string();
        }
    }

    "general".to_string()
}

/// Detect scene from input
fn detect_scene(input: &str) -> String {
    let scene_keywords: HashMap<&str, &str> = HashMap::from([
        ("生成代码", "code_generation"),
        ("写代码", "code_generation"),
        ("审查", "code_review"),
        ("修复", "bug_fix"),
        ("生成图片", "image_generation"),
        ("画图", "image_generation"),
        ("生成视频", "video_generation"),
        ("制作视频", "video_generation"),
        ("文案", "copywriting"),
        ("翻译", "translation"),
        ("生成文档", "document_generation"),
        ("Excel", "excel_analysis"),
        ("构建游戏", "game_build"),
    ]);

    for (keyword, scene) in &scene_keywords {
        if input.contains(keyword) {
            return scene.to_string();
        }
    }

    "general_task".to_string()
}

/// Select agent for role
fn select_agent_for_role(role: &str, scene: &str) -> (String, String) {
    let role_mapping: HashMap<&str, &str> = HashMap::from([
        ("developer", "claude-code"),
        ("marketer", "kimi"),
        ("designer", "jimeng"),
        ("finance", "wps"),
        ("gamer", "unity"),
        ("writer", "kimi"),
        ("analyst", "wps"),
        ("general", "claude-code"),
    ]);

    let scene_mapping: HashMap<&str, &str> = HashMap::from([
        ("image_generation", "jimeng"),
        ("video_generation", "kling"),
        ("copywriting", "kimi"),
        ("translation", "kimi"),
        ("document_generation", "wps"),
        ("excel_analysis", "wps"),
        ("game_build", "unity"),
        ("code_generation", "claude-code"),
        ("bug_fix", "codex"),
        ("code_review", "claude-code"),
    ]);

    // First check scene mapping
    if let Some(agent) = scene_mapping.get(scene) {
        return (agent.to_string(), format!("Selected {} for scene: {}", agent, scene));
    }

    // Then use role mapping
    let agent_str = role_mapping.get(role).copied().unwrap_or("claude-code");
    let agent = agent_str.to_string();
    (agent.clone(), format!("Selected {} for role: {}", agent, role))
}

/// Create new server state
pub fn new_server_state() -> Arc<HttpServerState> {
    let (event_tx, _) = broadcast::channel(100);
    Arc::new(HttpServerState {
        event_tx,
        start_time: chrono::Utc::now(),
    })
}

/// Start HTTP server
pub async fn start_server(port: u16) {
    let state = new_server_state();
    let router = create_router(state);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    println!("HTTP Server starting on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), router)
        .await
        .unwrap();
}

/// Broadcast event to all WebSocket clients
pub fn broadcast_event(state: &HttpServerState, event_type: &str, data: serde_json::Value) {
    let event = EventMessage {
        event_type: event_type.to_string(),
        data,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    // Ignore send error (no clients connected)
    let _ = state.event_tx.send(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_role_developer() {
        let role = detect_role("帮我写一个 Rust 函数");
        assert_eq!(role, "developer");
    }

    #[test]
    fn test_detect_role_marketer() {
        let role = detect_role("写一个抖音营销文案");
        assert_eq!(role, "marketer");
    }

    #[test]
    fn test_detect_scene_image() {
        let scene = detect_scene("生成图片");
        assert_eq!(scene, "image_generation");
    }

    #[test]
    fn test_select_agent_for_role() {
        let (agent, reason) = select_agent_for_role("developer", "code_generation");
        assert_eq!(agent, "claude-code");
        assert!(reason.contains("claude-code"));
    }

    #[test]
    fn test_new_server_state() {
        let state = new_server_state();
        assert!(state.start_time < chrono::Utc::now());
    }

    #[test]
    fn test_get_available_agents() {
        let agents = get_available_agents();
        assert_eq!(agents.len(), 8);
        assert!(agents.iter().any(|a| a.id == "claude-code"));
    }
}