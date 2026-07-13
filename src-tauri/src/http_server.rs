// HTTP Server - REST API for Web/Mobile platforms
//
// Phase 7: HTTP Server for multi-platform access
// Provides REST API + WebSocket for Web and Mobile clients

use crate::operator::security::{RemoteAccessDenied, RemoteAccessPolicy, RemoteAccessPolicyConfig};
use crate::operator::{
    self, ApprovalRequest, ApproveRequest, EventLevel, OperatorEvent, OperatorEventType,
    OperatorState, OperatorTask, RedirectRequest, StartTaskRequest, StartTaskResponse, TaskSummary,
};
use axum::body::Body;
use axum::extract::ws::Message;
use axum::{
    extract::{ws::WebSocket, Path as AxumPath, Query, State, WebSocketUpgrade},
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE, ORIGIN, WWW_AUTHENTICATE},
        HeaderName, HeaderValue, Method, Request, StatusCode,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

const REMOTE_AUDIT_CAPACITY: usize = 1000;
const MIN_REMOTE_TOKEN_BYTES: usize = 32;
const OPERATOR_CLIENT_HEADER: &str = "x-acp-operator-client";

/// HTTP Server state
pub struct HttpServerState {
    /// Event broadcaster for WebSocket
    pub event_tx: broadcast::Sender<EventMessage>,
    /// Shared Operator state for remote desktop/web/mobile control
    pub operator_state: Arc<Mutex<OperatorState>>,
    /// Authentication and allowlist policy applied at the HTTP boundary
    pub access_policy: Arc<RemoteAccessPolicy>,
    /// Bounded, body-free audit trail for remote requests
    pub audit_log: Mutex<VecDeque<RemoteAuditRecord>>,
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
        (
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(self),
        )
            .into_response()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemotePlatformCapability {
    id: String,
    name: String,
    platform_type: String,
    transport: Vec<String>,
    status: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EventQuery {
    limit: Option<usize>,
    after_sequence: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RemoteRedirectRequest {
    new_goal: String,
    preserve_completed_work: Option<bool>,
    expected_revision: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct RemoteControlRequest {
    expected_revision: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAuditRecord {
    pub audit_id: String,
    pub request_id: String,
    pub timestamp: String,
    pub principal: String,
    pub client_id: String,
    pub method: String,
    pub path: String,
    pub action: String,
    pub outcome: String,
    pub status_code: u16,
    pub denial_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuditQuery {
    limit: Option<usize>,
}

#[derive(Clone)]
pub struct RemoteOperatorServerConfig {
    pub bind_ip: IpAddr,
    pub port: u16,
    access_policy: RemoteAccessPolicy,
}

impl std::fmt::Debug for RemoteOperatorServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteOperatorServerConfig")
            .field("bind_ip", &self.bind_ip)
            .field("port", &self.port)
            .field("access_policy", &self.access_policy)
            .finish()
    }
}

impl RemoteOperatorServerConfig {
    pub fn new(
        bind_host: &str,
        port: u16,
        policy_config: RemoteAccessPolicyConfig,
    ) -> Result<Self, String> {
        let bind_ip = bind_host
            .trim()
            .parse::<IpAddr>()
            .map_err(|_| format!("Invalid ACP_OPERATOR_HTTP_BIND address: {}", bind_host))?;

        if !bind_ip.is_loopback() {
            let token = policy_config
                .token
                .as_deref()
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .ok_or_else(|| {
                    "ACP_OPERATOR_HTTP_TOKEN is required for non-loopback Remote Operator binds"
                        .to_string()
                })?;
            if token.len() < MIN_REMOTE_TOKEN_BYTES {
                return Err(format!(
                    "ACP_OPERATOR_HTTP_TOKEN must be at least {} bytes for non-loopback binds",
                    MIN_REMOTE_TOKEN_BYTES
                ));
            }
            if policy_config.allowed_project_roots.is_empty() {
                return Err(
                    "ACP_OPERATOR_ALLOWED_PROJECT_ROOTS is required for non-loopback Remote Operator binds"
                        .to_string(),
                );
            }
        }

        let access_policy = RemoteAccessPolicy::new(policy_config)
            .map_err(|error| format!("Invalid Remote Operator security config: {}", error))?;
        Ok(Self {
            bind_ip,
            port,
            access_policy,
        })
    }

    pub fn from_env(default_port: u16) -> Result<Self, String> {
        let bind_host =
            std::env::var("ACP_OPERATOR_HTTP_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = match std::env::var("ACP_OPERATOR_HTTP_PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|_| "ACP_OPERATOR_HTTP_PORT must be a valid u16".to_string())?,
            Err(_) => default_port,
        };
        let mut policy = RemoteAccessPolicyConfig::default();
        policy.token = std::env::var("ACP_OPERATOR_HTTP_TOKEN").ok();

        if let Ok(value) = std::env::var("ACP_OPERATOR_ALLOWED_ORIGINS") {
            policy.allowed_origins = split_csv(&value);
        }
        if let Ok(value) = std::env::var("ACP_OPERATOR_ALLOWED_CLIENTS") {
            policy.allowed_clients = split_csv(&value);
        }
        if let Ok(value) = std::env::var("ACP_OPERATOR_ALLOWED_DOMAINS") {
            policy.allowed_domains = split_csv(&value);
        }
        if let Some(value) = std::env::var_os("ACP_OPERATOR_ALLOWED_PROJECT_ROOTS") {
            policy.allowed_project_roots = std::env::split_paths(&value).collect::<Vec<PathBuf>>();
        }

        Self::new(&bind_host, port, policy)
    }

    fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind_ip, self.port)
    }
}

/// Create HTTP router
pub fn create_router(state: Arc<HttpServerState>) -> Router {
    let cors = create_cors_layer(&state.access_policy);
    Router::new()
        // Agent endpoints
        .route("/api/agents", get(list_agents))
        .route("/api/agents/:id", get(get_agent))
        // Remote Operator endpoints for desktop/web/mobile/IDE clients
        .route("/api/operator/platforms", get(operator_platforms))
        .route(
            "/api/operator/tasks",
            get(operator_list_tasks).post(operator_start_task),
        )
        .route("/api/operator/tasks/:task_id", get(operator_get_task))
        .route(
            "/api/operator/tasks/:task_id/events",
            get(operator_list_events),
        )
        .route(
            "/api/operator/tasks/:task_id/approvals",
            get(operator_get_pending_approvals),
        )
        .route(
            "/api/operator/tasks/:task_id/summary",
            get(operator_get_task_summary),
        )
        .route(
            "/api/operator/tasks/:task_id/pause",
            post(operator_pause_task),
        )
        .route(
            "/api/operator/tasks/:task_id/resume",
            post(operator_resume_task),
        )
        .route(
            "/api/operator/tasks/:task_id/stop",
            post(operator_stop_task),
        )
        .route(
            "/api/operator/tasks/:task_id/redirect",
            post(operator_redirect_task),
        )
        .route("/api/operator/approvals/decision", post(operator_approve))
        .route("/api/operator/audit", get(operator_get_audit))
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
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            enforce_remote_access,
        ))
        .with_state(state)
}

fn create_cors_layer(policy: &RemoteAccessPolicy) -> CorsLayer {
    let origins = policy
        .allowed_origins()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect::<Vec<_>>();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static(OPERATOR_CLIENT_HEADER),
        ])
}

fn event_page_bounds(
    events: &[OperatorEvent],
    limit: usize,
    after_sequence: Option<u64>,
) -> (usize, usize) {
    let limit = limit.min(1000);
    match after_sequence {
        Some(sequence) => {
            let start = events
                .iter()
                .position(|event| event.sequence > sequence)
                .unwrap_or(events.len());
            (start, start.saturating_add(limit).min(events.len()))
        }
        None => {
            let start = events.len().saturating_sub(limit);
            (start, events.len())
        }
    }
}

async fn enforce_remote_access(
    State(state): State<Arc<HttpServerState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let action = remote_action(&method, &path).to_string();
    let client_id = sanitize_client_id(
        request
            .headers()
            .get(OPERATOR_CLIENT_HEADER)
            .and_then(|value| value.to_str().ok()),
    )
    .to_string();
    let origin_header = request.headers().get(ORIGIN);
    let origin = origin_header.and_then(|value| value.to_str().ok());

    if origin_header.is_some() && origin.is_none() {
        return access_denied_response(
            &state,
            &request_id,
            &client_id,
            &method,
            &path,
            &action,
            RemoteAccessDenied::OriginNotAllowed,
            None,
        );
    }

    if let Err(denial) = state.access_policy.authorize_origin(origin) {
        return access_denied_response(
            &state,
            &request_id,
            &client_id,
            &method,
            &path,
            &action,
            denial,
            None,
        );
    }

    if method != Method::OPTIONS {
        let authorization = request
            .headers()
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok());
        if let Err(denial) = state.access_policy.authenticate(authorization) {
            return access_denied_response(
                &state,
                &request_id,
                &client_id,
                &method,
                &path,
                &action,
                denial,
                origin,
            );
        }
        if let Err(denial) = state.access_policy.authorize_client(Some(&client_id)) {
            return access_denied_response(
                &state,
                &request_id,
                &client_id,
                &method,
                &path,
                &action,
                denial,
                origin,
            );
        }
    }

    let should_audit = is_mutating_method(&method);
    let principal = if state.access_policy.requires_authentication() {
        "bearer_token"
    } else {
        "local_trust"
    };
    if should_audit
        && append_audit(
            &state,
            RemoteAuditRecord {
                audit_id: uuid::Uuid::new_v4().to_string(),
                request_id: request_id.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                principal: principal.to_string(),
                client_id: client_id.clone(),
                method: method.to_string(),
                path: path.clone(),
                action: action.clone(),
                outcome: "attempted".to_string(),
                status_code: 0,
                denial_code: None,
            },
        )
        .is_err()
    {
        return api_error(500, "Remote audit log unavailable".to_string()).into_response();
    }

    let response = next.run(request).await;
    if should_audit {
        let status = response.status();
        let _ = append_audit(
            &state,
            RemoteAuditRecord {
                audit_id: uuid::Uuid::new_v4().to_string(),
                request_id,
                timestamp: chrono::Utc::now().to_rfc3339(),
                principal: principal.to_string(),
                client_id,
                method: method.to_string(),
                path,
                action,
                outcome: if status.is_success() {
                    "succeeded".to_string()
                } else {
                    "failed".to_string()
                },
                status_code: status.as_u16(),
                denial_code: if status == StatusCode::FORBIDDEN {
                    Some("request_scope_denied".to_string())
                } else {
                    None
                },
            },
        );
    }
    response
}

fn access_denied_response(
    state: &HttpServerState,
    request_id: &str,
    client_id: &str,
    method: &Method,
    path: &str,
    action: &str,
    denial: RemoteAccessDenied,
    allowed_origin: Option<&str>,
) -> Response {
    let status = match denial {
        RemoteAccessDenied::MissingBearerToken | RemoteAccessDenied::InvalidBearerToken => {
            StatusCode::UNAUTHORIZED
        }
        _ => StatusCode::FORBIDDEN,
    };
    let _ = append_audit(
        state,
        RemoteAuditRecord {
            audit_id: uuid::Uuid::new_v4().to_string(),
            request_id: request_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            principal: "unauthenticated".to_string(),
            client_id: client_id.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            action: action.to_string(),
            outcome: "denied".to_string(),
            status_code: status.as_u16(),
            denial_code: Some(denial.code().to_string()),
        },
    );

    let mut response = api_error(
        status.as_u16(),
        if status == StatusCode::UNAUTHORIZED {
            "Remote Operator authentication required".to_string()
        } else {
            "Remote Operator request forbidden".to_string()
        },
    )
    .into_response();
    if status == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(
            WWW_AUTHENTICATE,
            HeaderValue::from_static("Bearer realm=\"acp-operator\""),
        );
    }
    if let Some(origin) = allowed_origin.and_then(|origin| HeaderValue::from_str(origin).ok()) {
        response.headers_mut().insert(
            HeaderName::from_static("access-control-allow-origin"),
            origin,
        );
        response.headers_mut().insert(
            HeaderName::from_static("access-control-expose-headers"),
            HeaderValue::from_static("www-authenticate"),
        );
        response.headers_mut().insert(
            HeaderName::from_static("vary"),
            HeaderValue::from_static("origin"),
        );
    }
    response
}

fn append_audit(state: &HttpServerState, record: RemoteAuditRecord) -> Result<(), String> {
    let mut audit = state
        .audit_log
        .lock()
        .map_err(|_| "Remote audit log lock failed".to_string())?;
    while audit.len() >= REMOTE_AUDIT_CAPACITY {
        audit.pop_front();
    }
    audit.push_back(record);
    Ok(())
}

fn is_mutating_method(method: &Method) -> bool {
    !matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn remote_action(method: &Method, path: &str) -> &'static str {
    if method == Method::POST && path == "/api/operator/tasks" {
        "start_task"
    } else if method == Method::POST && path == "/api/operator/approvals/decision" {
        "approval_decision"
    } else if method == Method::POST && path.ends_with("/pause") {
        "pause_task"
    } else if method == Method::POST && path.ends_with("/resume") {
        "resume_task"
    } else if method == Method::POST && path.ends_with("/stop") {
        "stop_task"
    } else if method == Method::POST && path.ends_with("/redirect") {
        "redirect_task"
    } else if path == "/api/operator/audit" {
        "read_audit"
    } else {
        "http_request"
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn sanitize_client_id(value: Option<&str>) -> &str {
    value
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .filter(|value| {
            value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':')
            })
        })
        .unwrap_or("unidentified")
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
            capabilities: vec![
                "code_generation".to_string(),
                "code_review".to_string(),
                "bug_fix".to_string(),
                "refactoring".to_string(),
            ],
        },
        AgentSummary {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            adapter_type: "cli".to_string(),
            status: "available".to_string(),
            health_score: 90.0,
            capabilities: vec![
                "code_generation".to_string(),
                "bug_fix".to_string(),
                "testing".to_string(),
            ],
        },
        AgentSummary {
            id: "kimi".to_string(),
            name: "Kimi (Moonshot)".to_string(),
            adapter_type: "api".to_string(),
            status: "available".to_string(),
            health_score: 88.0,
            capabilities: vec![
                "copywriting".to_string(),
                "translation".to_string(),
                "summary".to_string(),
            ],
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
            capabilities: vec![
                "document_generation".to_string(),
                "excel_analysis".to_string(),
                "finance".to_string(),
            ],
        },
        AgentSummary {
            id: "unity".to_string(),
            name: "Unity Game".to_string(),
            adapter_type: "cli".to_string(),
            status: "available".to_string(),
            health_score: 78.0,
            capabilities: vec![
                "game_build".to_string(),
                "asset_management".to_string(),
                "3d".to_string(),
            ],
        },
        AgentSummary {
            id: "godot".to_string(),
            name: "Godot Engine".to_string(),
            adapter_type: "cli".to_string(),
            status: "available".to_string(),
            health_score: 75.0,
            capabilities: vec![
                "game_build".to_string(),
                "asset_management".to_string(),
                "2d_3d".to_string(),
            ],
        },
    ]
}

fn get_remote_platform_capabilities() -> Vec<RemotePlatformCapability> {
    vec![
        RemotePlatformCapability {
            id: "tauri-desktop".to_string(),
            name: "ACP UI Desktop".to_string(),
            platform_type: "desktop_app".to_string(),
            transport: vec![
                "tauri.invoke".to_string(),
                "http".to_string(),
                "websocket".to_string(),
            ],
            status: "active".to_string(),
            capabilities: vec![
                "operator_control".to_string(),
                "godot_project_analysis".to_string(),
                "approval_workflow".to_string(),
                "task_events".to_string(),
            ],
        },
        RemotePlatformCapability {
            id: "web-remote".to_string(),
            name: "Web Remote Console".to_string(),
            platform_type: "web".to_string(),
            transport: vec!["http".to_string(), "websocket".to_string()],
            status: "active_backend".to_string(),
            capabilities: vec![
                "start_task".to_string(),
                "approve".to_string(),
                "pause_resume_stop".to_string(),
                "redirect".to_string(),
                "event_polling".to_string(),
            ],
        },
        RemotePlatformCapability {
            id: "vscode-extension".to_string(),
            name: "Hermes Game VSCode Extension".to_string(),
            platform_type: "ide_extension".to_string(),
            transport: vec!["http".to_string(), "websocket".to_string()],
            status: "bridge_ready".to_string(),
            capabilities: vec![
                "project_context".to_string(),
                "operator_control".to_string(),
                "approval_panel".to_string(),
            ],
        },
        RemotePlatformCapability {
            id: "idea-plugin".to_string(),
            name: "JetBrains IDEA Plugin".to_string(),
            platform_type: "ide_plugin".to_string(),
            transport: vec!["http".to_string()],
            status: "planned_on_same_protocol".to_string(),
            capabilities: vec![
                "operator_control".to_string(),
                "approval_panel".to_string(),
                "task_timeline".to_string(),
            ],
        },
        RemotePlatformCapability {
            id: "mobile-operator".to_string(),
            name: "Mobile Operator Client".to_string(),
            platform_type: "mobile".to_string(),
            transport: vec!["http".to_string(), "websocket".to_string()],
            status: "protocol_ready".to_string(),
            capabilities: vec![
                "remote_pause_stop".to_string(),
                "approval_decision".to_string(),
                "progress_view".to_string(),
            ],
        },
        RemotePlatformCapability {
            id: "godot-domain-pack".to_string(),
            name: "Godot Domain Pack".to_string(),
            platform_type: "game_engine".to_string(),
            transport: vec!["operator_domain".to_string(), "hermes-game".to_string()],
            status: "mvp_active".to_string(),
            capabilities: vec![
                "project_detection".to_string(),
                "project_analysis".to_string(),
                "plan_generation".to_string(),
                "proposal_artifact".to_string(),
            ],
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
async fn get_agent(AxumPath(id): AxumPath<String>) -> Result<Json<AgentSummary>, ApiError> {
    let agents = get_available_agents();
    let agent = agents.iter().find(|a| a.id == id).ok_or_else(|| ApiError {
        error: format!("Agent '{}' not found", id),
        code: 404,
    })?;

    Ok(Json((*agent).clone()))
}

// ============================================================================
// Remote Operator Endpoints
// ============================================================================

async fn operator_platforms() -> Json<Vec<RemotePlatformCapability>> {
    Json(get_remote_platform_capabilities())
}

async fn operator_start_task(
    State(state): State<Arc<HttpServerState>>,
    Json(mut request): Json<StartTaskRequest>,
) -> Result<Json<StartTaskResponse>, ApiError> {
    state
        .access_policy
        .authorize_domain(Some(&request.domain))
        .map_err(remote_scope_error)?;
    let canonical_project_path = state
        .access_policy
        .canonicalize_authorized_project_path(Path::new(&request.project_path))
        .map_err(remote_scope_error)?;
    request.project_path = canonical_project_path.display().to_string();
    let mut operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    let response =
        operator::start_task_in_state(&mut operator_state, request).map_err(operator_error)?;
    Ok(Json(response))
}

async fn operator_get_audit(
    State(state): State<Arc<HttpServerState>>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<RemoteAuditRecord>>, ApiError> {
    let audit = state
        .audit_log
        .lock()
        .map_err(|_| api_error(500, "Remote audit log unavailable".to_string()))?;
    let limit = query.limit.unwrap_or(100).min(REMOTE_AUDIT_CAPACITY);
    let start = audit.len().saturating_sub(limit);
    Ok(Json(audit.iter().skip(start).cloned().collect()))
}

async fn operator_list_tasks(
    State(state): State<Arc<HttpServerState>>,
) -> Result<Json<Vec<OperatorTask>>, ApiError> {
    let operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    Ok(Json(
        operator_state
            .tasks
            .values()
            .filter(|task| task_scope_allowed(&state.access_policy, task))
            .cloned()
            .collect(),
    ))
}

async fn operator_get_task(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
) -> Result<Json<OperatorTask>, ApiError> {
    let operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    Ok(Json(
        authorized_task(&state.access_policy, &operator_state, &task_id)?.clone(),
    ))
}

async fn operator_pause_task(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
    body: Option<Json<RemoteControlRequest>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    authorized_task(&state.access_policy, &operator_state, &task_id)?;
    operator::pause_task_in_state_with_revision(
        &mut operator_state,
        &task_id,
        body.and_then(|body| body.0.expected_revision),
    )
    .map_err(operator_error)?;
    Ok(Json(serde_json::json!({ "ok": true, "task_id": task_id })))
}

async fn operator_resume_task(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
    body: Option<Json<RemoteControlRequest>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    {
        let operator_state = state
            .operator_state
            .lock()
            .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
        authorized_task(&state.access_policy, &operator_state, &task_id)?;
    }
    operator::resume_task_in_shared_state_for_background_with_revision(
        state.operator_state.clone(),
        task_id.clone(),
        body.and_then(|body| body.0.expected_revision),
    )
    .map_err(operator_error)?;
    Ok(Json(serde_json::json!({ "ok": true, "task_id": task_id })))
}

async fn operator_stop_task(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
    body: Option<Json<RemoteControlRequest>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    authorized_task(&state.access_policy, &operator_state, &task_id)?;
    operator::stop_task_in_state_with_revision(
        &mut operator_state,
        &task_id,
        body.and_then(|body| body.0.expected_revision),
    )
    .map_err(operator_error)?;
    Ok(Json(serde_json::json!({ "ok": true, "task_id": task_id })))
}

async fn operator_redirect_task(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
    Json(request): Json<RemoteRedirectRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    authorized_task(&state.access_policy, &operator_state, &task_id)?;
    let redirect = RedirectRequest {
        task_id: task_id.clone(),
        new_goal: request.new_goal,
        preserve_completed_work: request.preserve_completed_work.unwrap_or(true),
        expected_revision: request.expected_revision,
    };
    operator::redirect_task_in_state(&mut operator_state, redirect).map_err(operator_error)?;
    Ok(Json(serde_json::json!({ "ok": true, "task_id": task_id })))
}

async fn operator_approve(
    State(state): State<Arc<HttpServerState>>,
    Json(request): Json<ApproveRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let task_id = request.task_id.clone();
    {
        let operator_state = state
            .operator_state
            .lock()
            .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
        authorized_task(&state.access_policy, &operator_state, &task_id)?;
    }
    operator::approve_task_in_shared_state_for_background(state.operator_state.clone(), request)
        .map_err(operator_error)?;
    Ok(Json(serde_json::json!({ "ok": true, "task_id": task_id })))
}

async fn operator_get_pending_approvals(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
) -> Result<Json<Vec<ApprovalRequest>>, ApiError> {
    let operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    authorized_task(&state.access_policy, &operator_state, &task_id)?;
    let approvals = operator_state
        .approvals
        .get(&task_id)
        .ok_or_else(|| api_error(404, format!("Task not found: {}", task_id)))?;
    Ok(Json(
        approvals
            .iter()
            .filter(|approval| approval.decision.is_none())
            .cloned()
            .collect(),
    ))
}

async fn operator_list_events(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
    Query(query): Query<EventQuery>,
) -> Result<Json<Vec<OperatorEvent>>, ApiError> {
    let operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    authorized_task(&state.access_policy, &operator_state, &task_id)?;
    let events = operator_state
        .events
        .get(&task_id)
        .ok_or_else(|| api_error(404, format!("Task not found: {}", task_id)))?;
    let (start, end) = event_page_bounds(events, query.limit.unwrap_or(100), query.after_sequence);
    Ok(Json(events[start..end].to_vec()))
}

async fn operator_get_task_summary(
    State(state): State<Arc<HttpServerState>>,
    AxumPath(task_id): AxumPath<String>,
) -> Result<Json<TaskSummary>, ApiError> {
    let operator_state = state
        .operator_state
        .lock()
        .map_err(|error| api_error(500, format!("Operator state lock failed: {}", error)))?;
    let task = authorized_task(&state.access_policy, &operator_state, &task_id)?;
    let changes = operator_state
        .file_changes
        .get(&task_id)
        .cloned()
        .unwrap_or_default();

    let files_changed = changes
        .iter()
        .filter(|change| change.change_type == "modified")
        .map(|change| change.path.clone())
        .collect();
    let files_created = changes
        .iter()
        .filter(|change| change.change_type == "created")
        .map(|change| change.path.clone())
        .collect();
    let files_deleted = changes
        .iter()
        .filter(|change| change.change_type == "deleted")
        .map(|change| change.path.clone())
        .collect();
    let errors = task
        .error
        .as_ref()
        .map(|error| vec![error.clone()])
        .unwrap_or_default();
    let summary = task
        .summary
        .clone()
        .unwrap_or_else(|| format!("Task is currently {:?}", task.status));

    Ok(Json(TaskSummary {
        task_id: task.task_id.clone(),
        status: task.status.clone(),
        goal: task.goal.clone(),
        summary,
        files_changed,
        files_created,
        files_deleted,
        duration_seconds: 0,
        iterations: operator_state
            .events
            .get(&task_id)
            .map(|events| events.len() as u32)
            .unwrap_or(0),
        errors,
        warnings: Vec::new(),
    }))
}

fn task_scope_allowed(policy: &RemoteAccessPolicy, task: &OperatorTask) -> bool {
    policy.authorize_domain(Some(&task.domain)).is_ok()
        && policy
            .authorize_project_path(Some(Path::new(&task.project_path)))
            .is_ok()
}

fn authorized_task<'a>(
    policy: &RemoteAccessPolicy,
    operator_state: &'a OperatorState,
    task_id: &str,
) -> Result<&'a OperatorTask, ApiError> {
    let task = operator_state
        .tasks
        .get(task_id)
        .ok_or_else(|| api_error(404, format!("Task not found: {}", task_id)))?;
    if task_scope_allowed(policy, task) {
        Ok(task)
    } else {
        Err(api_error(
            403,
            "Remote Operator request forbidden".to_string(),
        ))
    }
}

// ============================================================================
// One-Shot Endpoint
// ============================================================================

/// Execute one-shot request (mock implementation)
async fn execute_oneshot(Json(request): Json<OneShotApiRequest>) -> Json<OneShotApiResponse> {
    // Detect user role (simple keyword matching)
    let role = detect_role(&request.input);
    let scene = detect_scene(&request.input);

    // Select best agent
    let (agent, reason) = select_agent_for_role(&role, &scene);

    // Generate preview response
    let preview = format!(
        "将使用 {} 处理您的请求: {}",
        agent,
        &request.input[..50.min(request.input.len())]
    );
    let dag_visualization = format!(
        "Input -> RoleDetection({}) -> AgentSelection({})",
        role, agent
    );

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
async fn execute_task(Json(request): Json<ExecuteApiRequest>) -> Json<ExecuteApiResponse> {
    let task_id = uuid::Uuid::new_v4().to_string();

    // Mock execution
    let output = format!(
        "Task '{}' executed by agent '{}'",
        request.description, request.agent_id
    );

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

    if sender
        .send(Message::Text(serde_json::to_string(&initial).unwrap()))
        .await
        .is_err()
    {
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

fn api_error(code: u16, error: String) -> ApiError {
    ApiError { error, code }
}

fn remote_scope_error(_denial: RemoteAccessDenied) -> ApiError {
    api_error(403, "Remote Operator request forbidden".to_string())
}

fn operator_error(error: String) -> ApiError {
    let lower = error.to_ascii_lowercase();
    let code = if lower.contains("not found") {
        404
    } else if lower.contains("cannot")
        || lower.contains("already")
        || lower.contains("invalid state")
    {
        409
    } else {
        400
    };

    api_error(code, error)
}

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
        return (
            agent.to_string(),
            format!("Selected {} for scene: {}", agent, scene),
        );
    }

    // Then use role mapping
    let agent_str = role_mapping.get(role).copied().unwrap_or("claude-code");
    let agent = agent_str.to_string();
    (
        agent.clone(),
        format!("Selected {} for role: {}", agent, role),
    )
}

/// Create new server state
pub fn new_server_state() -> Arc<HttpServerState> {
    new_server_state_with_operator_state(Arc::new(Mutex::new(OperatorState::new())))
}

pub fn new_server_state_with_operator_state(
    operator_state: Arc<Mutex<OperatorState>>,
) -> Arc<HttpServerState> {
    let policy = RemoteAccessPolicy::new(RemoteAccessPolicyConfig::default())
        .expect("default Remote Operator policy must be valid");
    new_server_state_with_security(operator_state, policy)
}

pub fn new_server_state_with_security(
    operator_state: Arc<Mutex<OperatorState>>,
    access_policy: RemoteAccessPolicy,
) -> Arc<HttpServerState> {
    let (event_tx, _) = broadcast::channel(100);
    Arc::new(HttpServerState {
        event_tx,
        operator_state,
        access_policy: Arc::new(access_policy),
        audit_log: Mutex::new(VecDeque::with_capacity(REMOTE_AUDIT_CAPACITY)),
        start_time: chrono::Utc::now(),
    })
}

/// Start HTTP server
pub async fn start_server(port: u16) {
    let state = new_server_state();
    let router = create_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("HTTP Server starting on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), router)
        .await
        .unwrap();
}

pub async fn start_operator_server(
    config: RemoteOperatorServerConfig,
    operator_state: Arc<Mutex<OperatorState>>,
) -> Result<(), String> {
    let addr = config.bind_addr();
    let auth_enabled = config.access_policy.requires_authentication();
    let state = new_server_state_with_security(operator_state, config.access_policy);
    let router = create_router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|error| format!("Failed to bind remote operator server: {}", error))?;

    println!(
        "Remote Operator HTTP Server starting on http://{} (authentication: {})",
        addr,
        if auth_enabled {
            "enabled"
        } else {
            "local-trust"
        }
    );

    axum::serve(listener, router)
        .await
        .map_err(|error| format!("Remote operator server failed: {}", error))
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
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct RemotePersistenceTestDir {
        path: PathBuf,
    }

    impl RemotePersistenceTestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let path = std::env::current_dir()
                .expect("current dir")
                .join(".tmp-tests")
                .join(format!(
                    "remote_persistence_{}_{}",
                    std::process::id(),
                    unique
                ));
            std::fs::create_dir_all(path.join("project/scripts"))
                .expect("create remote persistence project");
            std::fs::write(
                path.join("project/project.godot"),
                "[application]\nconfig/name=\"Remote Persistence\"\n",
            )
            .expect("write project marker");
            std::fs::write(
                path.join("project/scripts/player.gd"),
                "extends Node\nvar jumps = 1\n",
            )
            .expect("write project script");
            Self { path }
        }

        fn project(&self) -> PathBuf {
            self.path.join("project")
        }

        fn db_path(&self) -> PathBuf {
            self.path.join("operator-state.sqlite3")
        }
    }

    impl Drop for RemotePersistenceTestDir {
        fn drop(&mut self) {
            if crate::operator::is_d_drive_path(Path::new(&self.path)) {
                let _ = std::fs::remove_dir_all(&self.path);
            }
        }
    }

    #[test]
    fn event_cursor_pagination_returns_only_the_next_page() {
        let events = (0..6)
            .map(|sequence| OperatorEvent {
                event_id: format!("event_{}", sequence),
                task_id: "task_cursor".to_string(),
                sequence,
                task_revision: sequence,
                timestamp: "2026-01-01T00:00:00Z".to_string(),
                event_type: OperatorEventType::TaskStarted,
                level: EventLevel::Info,
                title: format!("Event {}", sequence),
                message: None,
                source: "test".to_string(),
                payload: None,
                title_key: None,
                message_key: None,
                title_args: None,
                message_args: None,
            })
            .collect::<Vec<_>>();

        let first_page = event_page_bounds(&events, 2, None);
        assert_eq!(
            events[first_page.0..first_page.1]
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            vec![4, 5]
        );

        let next_page = event_page_bounds(&events, 2, Some(1));
        assert_eq!(
            events[next_page.0..next_page.1]
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert_eq!(event_page_bounds(&events, 2, Some(99)), (6, 6));
        assert_eq!(event_page_bounds(&events, 0, Some(1)), (2, 2));
    }

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

    #[test]
    fn test_get_remote_platform_capabilities() {
        let platforms = get_remote_platform_capabilities();
        assert!(platforms.iter().any(|platform| platform.id == "web-remote"));
        assert!(platforms
            .iter()
            .any(|platform| platform.id == "godot-domain-pack"));
        assert!(platforms.iter().any(|platform| platform
            .capabilities
            .contains(&"operator_control".to_string())));
    }

    #[test]
    fn test_create_router_with_operator_routes() {
        let state = new_server_state();
        let _router = create_router(state);
    }

    #[test]
    fn test_new_server_state_shares_operator_state() {
        let state = new_server_state();
        let response = {
            let mut operator_state = state.operator_state.lock().expect("operator state lock");
            operator::start_task_in_state(
                &mut operator_state,
                StartTaskRequest {
                    domain: "remote.test".to_string(),
                    project_path: "D:\\dingsun\\acp-ui".to_string(),
                    goal: "remote control smoke".to_string(),
                    mode: None,
                    approval_policy: None,
                },
            )
            .expect("remote task should start")
        };

        let operator_state = state.operator_state.lock().expect("operator state lock");
        assert!(operator_state.tasks.contains_key(&response.task_id));
    }

    #[tokio::test]
    async fn test_remote_operator_reads_recovered_persistent_state() {
        let temp = RemotePersistenceTestDir::new();
        let task_id = {
            let mut state = OperatorState::new_persistent(temp.db_path())
                .expect("create persistent Operator state");
            operator::start_task_in_state(
                &mut state,
                StartTaskRequest {
                    domain: "game.godot".to_string(),
                    project_path: temp.project().to_string_lossy().to_string(),
                    goal: "recover through remote HTTP".to_string(),
                    mode: None,
                    approval_policy: None,
                },
            )
            .expect("start persistent task")
            .task_id
        };
        let recovered = Arc::new(Mutex::new(
            OperatorState::new_persistent(temp.db_path()).expect("recover Operator state"),
        ));
        let state = new_server_state_with_operator_state(recovered.clone());
        let router = create_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind recovered state server");
        let addr = listener
            .local_addr()
            .expect("read recovered server address");
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve recovered state");
        });
        let base_url = format!("http://{}", addr);
        let client = reqwest::Client::new();

        let task: OperatorTask = client
            .get(format!("{}/api/operator/tasks/{}", base_url, task_id))
            .send()
            .await
            .expect("recovered task response")
            .error_for_status()
            .expect("recovered task status")
            .json()
            .await
            .expect("recovered task JSON");
        let events: Vec<OperatorEvent> = client
            .get(format!(
                "{}/api/operator/tasks/{}/events?limit=100",
                base_url, task_id
            ))
            .send()
            .await
            .expect("recovered event response")
            .error_for_status()
            .expect("recovered event status")
            .json()
            .await
            .expect("recovered event JSON");
        let approvals: Vec<ApprovalRequest> = client
            .get(format!(
                "{}/api/operator/tasks/{}/approvals",
                base_url, task_id
            ))
            .send()
            .await
            .expect("recovered approval response")
            .error_for_status()
            .expect("recovered approval status")
            .json()
            .await
            .expect("recovered approval JSON");

        assert_eq!(
            task.status,
            crate::operator::OperatorTaskStatus::WaitingApproval
        );
        assert_eq!(task.goal, "recover through remote HTTP");
        assert!(!events.is_empty());
        assert_eq!(approvals.len(), 1);
        assert_eq!(
            recovered.lock().expect("shared recovered state").tasks[&task_id].status,
            task.status
        );
        server.abort();
    }

    #[tokio::test]
    async fn test_remote_operator_http_task_roundtrip() {
        let state = new_server_state();
        let router = create_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("read test listener address");
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve test router");
        });

        let client = reqwest::Client::new();
        let base_url = format!("http://{}", addr);

        let platforms: Vec<RemotePlatformCapability> = client
            .get(format!("{}/api/operator/platforms", base_url))
            .send()
            .await
            .expect("platform response")
            .error_for_status()
            .expect("platform status")
            .json()
            .await
            .expect("platform json");
        assert!(platforms
            .iter()
            .any(|platform| platform.id == "vscode-extension"));

        let start_response: StartTaskResponse = client
            .post(format!("{}/api/operator/tasks", base_url))
            .json(&StartTaskRequest {
                domain: "game.godot".to_string(),
                project_path: "D:\\dingsun\\acp-ui".to_string(),
                goal: "remote operator roundtrip".to_string(),
                mode: None,
                approval_policy: None,
            })
            .send()
            .await
            .expect("start task response")
            .error_for_status()
            .expect("start task status")
            .json()
            .await
            .expect("start task json");

        let task: OperatorTask = client
            .get(format!(
                "{}/api/operator/tasks/{}",
                base_url, start_response.task_id
            ))
            .send()
            .await
            .expect("get task response")
            .error_for_status()
            .expect("get task status")
            .json()
            .await
            .expect("get task json");

        assert_eq!(task.task_id, start_response.task_id);
        assert_eq!(task.domain, "game.godot");
        assert_eq!(task.goal, "remote operator roundtrip");

        server.abort();
    }

    #[tokio::test]
    async fn test_remote_operator_reviews_and_applies_structured_patch() {
        let project = tempfile::tempdir().expect("remote patch project");
        std::fs::create_dir_all(project.path().join("scripts")).expect("create scripts");
        std::fs::write(
            project.path().join("project.godot"),
            "[application]\nconfig/name=\"Remote Patch Test\"\n",
        )
        .expect("write project marker");
        let player = project.path().join("scripts/player.gd");
        std::fs::write(&player, "extends Node\nvar jumps = 1\n").expect("write player");

        let operator_state = Arc::new(Mutex::new(OperatorState::new()));
        let started = {
            let mut state = operator_state.lock().expect("operator state");
            operator::start_task_in_state(
                &mut state,
                StartTaskRequest {
                    domain: "game.godot".to_string(),
                    project_path: project.path().display().to_string(),
                    goal: "remote structured patch".to_string(),
                    mode: None,
                    approval_policy: None,
                },
            )
            .expect("start task")
        };
        let artifact = serde_json::json!({
            "version": 1,
            "summary": "Raise the jump count",
            "changes": [{
                "path": "scripts/player.gd",
                "operation": "replace",
                "content": "extends Node\nvar jumps = 2\n"
            }],
            "validation": ["Open the main scene"]
        })
        .to_string();
        let prepared = operator::prepare_structured_patch(&artifact, project.path())
            .expect("prepare remote patch");
        let patch_id = prepared.patch_id.clone();
        let patch_approval_id = format!("approval_{}_http_patch", started.task_id);
        {
            let mut state = operator_state.lock().expect("operator state");
            let task_revision = state
                .tasks
                .get(&started.task_id)
                .map(|task| task.revision)
                .unwrap_or(0);
            if let Some(approvals) = state.approvals.get_mut(&started.task_id) {
                for approval in approvals.iter_mut() {
                    approval.decision = Some(crate::operator::ApprovalDecision::RequestChanges);
                    approval.resolved_at = Some(chrono::Utc::now().to_rfc3339());
                    approval.resolved_by = Some("test_setup".to_string());
                }
                approvals.push(ApprovalRequest {
                    approval_id: patch_approval_id.clone(),
                    task_id: started.task_id.clone(),
                    task_revision,
                    level: crate::operator::ApprovalLevel::Approve,
                    action: "operator.patch.apply".to_string(),
                    title: "Review remote patch".to_string(),
                    reason: "HTTP clients must see the exact diff before apply.".to_string(),
                    risk: Some("replace one text file".to_string()),
                    preview: Some(crate::operator::ApprovalPreview {
                        files: Some(vec!["scripts/player.gd".to_string()]),
                        diff_id: Some(patch_id.clone()),
                        command: None,
                        diffs: Some(vec![crate::operator::FileDiffPreview {
                            path: "scripts/player.gd".to_string(),
                            operation: "replace".to_string(),
                            diff: prepared.changes[0].diff.clone(),
                        }]),
                    }),
                    options: vec![
                        crate::operator::ApprovalDecision::Approve,
                        crate::operator::ApprovalDecision::Reject,
                    ],
                    created_at: chrono::Utc::now().to_rfc3339(),
                    resolved_at: None,
                    decision: None,
                    resolved_by: None,
                });
            }
            state
                .pending_patches
                .insert(started.task_id.clone(), prepared);
        }

        let state = new_server_state_with_operator_state(operator_state.clone());
        let router = create_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("read listener address");
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve test router");
        });
        let client = reqwest::Client::new();
        let base_url = format!("http://{}", addr);

        let approvals: Vec<ApprovalRequest> = client
            .get(format!(
                "{}/api/operator/tasks/{}/approvals",
                base_url, started.task_id
            ))
            .send()
            .await
            .expect("approval response")
            .error_for_status()
            .expect("approval status")
            .json()
            .await
            .expect("approval JSON");
        assert_eq!(approvals.len(), 1);
        assert_eq!(approvals[0].action, "operator.patch.apply");
        assert!(approvals[0]
            .preview
            .as_ref()
            .and_then(|preview| preview.diffs.as_ref())
            .and_then(|diffs| diffs.first())
            .map(|diff| diff.diff.contains("+var jumps = 2"))
            .unwrap_or(false));

        client
            .post(format!("{}/api/operator/approvals/decision", base_url))
            .json(&ApproveRequest {
                task_id: started.task_id.clone(),
                approval_id: patch_approval_id,
                decision: crate::operator::ApprovalDecision::Approve,
                comment: Some("remote review complete".to_string()),
                expected_revision: None,
            })
            .send()
            .await
            .expect("approve response")
            .error_for_status()
            .expect("approve status");

        assert!(std::fs::read_to_string(player)
            .expect("read applied player")
            .contains("jumps = 2"));
        assert_eq!(
            operator_state
                .lock()
                .expect("operator state")
                .tasks
                .get(&started.task_id)
                .map(|task| &task.status),
            Some(&crate::operator::OperatorTaskStatus::Completed)
        );
        let events: Vec<crate::operator::OperatorEvent> = client
            .get(format!(
                "{}/api/operator/tasks/{}/events?limit=100",
                base_url, started.task_id
            ))
            .send()
            .await
            .expect("validation event response")
            .error_for_status()
            .expect("validation event status")
            .json()
            .await
            .expect("validation event JSON");
        assert!(events.iter().any(|event| matches!(
            event.event_type,
            crate::operator::OperatorEventType::ValidationSkipped
        )));
        assert!(!events.iter().any(|event| matches!(
            event.event_type,
            crate::operator::OperatorEventType::ValidationPassed
        )));
        let backup_root = operator_state
            .lock()
            .expect("operator state")
            .events
            .get(&started.task_id)
            .and_then(|events| {
                events.iter().find(|event| {
                    matches!(
                        event.event_type,
                        crate::operator::OperatorEventType::FilePatchApplied
                    )
                })
            })
            .and_then(|event| event.payload.as_ref())
            .and_then(|payload| payload.get("backup_root"))
            .and_then(|value| value.as_str())
            .map(PathBuf::from)
            .expect("backup root event payload");
        assert_eq!(
            std::fs::read_to_string(backup_root.join("scripts/player.gd"))
                .expect("read remote backup"),
            "extends Node\nvar jumps = 1\n"
        );

        server.abort();
        let artifact_root = operator::workspace_root()
            .join(".operator")
            .join("hermes-runs")
            .join(&started.task_id);
        if operator::is_d_drive_path(&artifact_root) {
            let _ = std::fs::remove_dir_all(artifact_root);
        }
    }

    #[test]
    fn test_remote_operator_server_config_rejects_unsafe_remote_bind() {
        let local = RemoteOperatorServerConfig::new(
            "127.0.0.1",
            1422,
            crate::operator::security::RemoteAccessPolicyConfig::default(),
        )
        .expect("loopback defaults should be valid");
        assert!(local.bind_ip.is_loopback());

        let remote_without_token = RemoteOperatorServerConfig::new(
            "0.0.0.0",
            1422,
            crate::operator::security::RemoteAccessPolicyConfig::default(),
        )
        .expect_err("remote bind without token must fail");
        assert!(remote_without_token.contains("ACP_OPERATOR_HTTP_TOKEN"));

        let mut token_without_roots =
            crate::operator::security::RemoteAccessPolicyConfig::default();
        token_without_roots.token = Some("0123456789abcdef0123456789abcdef".to_string());
        let remote_without_roots =
            RemoteOperatorServerConfig::new("0.0.0.0", 1422, token_without_roots)
                .expect_err("remote bind without project roots must fail");
        assert!(remote_without_roots.contains("ACP_OPERATOR_ALLOWED_PROJECT_ROOTS"));

        let root = tempfile::tempdir().expect("remote project root");
        let mut weak_token = crate::operator::security::RemoteAccessPolicyConfig::default();
        weak_token.token = Some("too-short".to_string());
        weak_token.allowed_project_roots = vec![root.path().to_path_buf()];
        let weak_token_error = RemoteOperatorServerConfig::new("0.0.0.0", 1422, weak_token)
            .expect_err("weak remote token must fail");
        assert!(weak_token_error.contains("at least 32 bytes"));
    }

    #[test]
    fn test_remote_operator_client_id_is_sanitized_before_audit() {
        assert_eq!(sanitize_client_id(Some("vscode-main.1")), "vscode-main.1");
        assert_eq!(sanitize_client_id(Some("secret value")), "unidentified");
        assert_eq!(sanitize_client_id(Some("客户端")), "unidentified");
        assert_eq!(sanitize_client_id(None), "unidentified");
    }

    #[test]
    fn test_remote_operator_write_actions_are_classified_for_audit() {
        let cases = [
            ("/api/operator/tasks", "start_task"),
            ("/api/operator/tasks/task-1/pause", "pause_task"),
            ("/api/operator/tasks/task-1/resume", "resume_task"),
            ("/api/operator/tasks/task-1/stop", "stop_task"),
            ("/api/operator/tasks/task-1/redirect", "redirect_task"),
            ("/api/operator/approvals/decision", "approval_decision"),
        ];
        for (path, expected) in cases {
            assert_eq!(remote_action(&Method::POST, path), expected);
        }
    }

    #[tokio::test]
    async fn test_remote_operator_auth_allowlists_and_audit() {
        let allowed_root = tempfile::tempdir().expect("allowed project root");
        std::fs::create_dir(allowed_root.path().join("nested")).expect("nested project directory");
        let denied_root = tempfile::tempdir().expect("denied project root");
        let policy = crate::operator::security::RemoteAccessPolicy::new(
            crate::operator::security::RemoteAccessPolicyConfig {
                token: Some("remote-secret-canary".to_string()),
                allowed_origins: vec!["http://127.0.0.1:1420".to_string()],
                allowed_clients: vec!["vscode-main".to_string()],
                allowed_domains: vec!["game.godot".to_string()],
                allowed_project_roots: vec![allowed_root.path().to_path_buf()],
            },
        )
        .expect("test policy");
        let state =
            new_server_state_with_security(Arc::new(Mutex::new(OperatorState::new())), policy);
        let operator_state = state.operator_state.clone();
        let router = create_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("test listener address");
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve secured router");
        });
        let base_url = format!("http://{}", addr);
        let client = reqwest::Client::new();

        let request_body = StartTaskRequest {
            domain: "game.godot".to_string(),
            project_path: allowed_root
                .path()
                .join("nested")
                .join("..")
                .display()
                .to_string(),
            goal: "goal-secret-canary".to_string(),
            mode: None,
            approval_policy: None,
        };
        let unauthorized = client
            .post(format!("{}/api/operator/tasks", base_url))
            .header("origin", "http://127.0.0.1:1420")
            .header("x-acp-operator-client", "vscode-main")
            .json(&request_body)
            .send()
            .await
            .expect("unauthorized response");
        assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);
        assert_eq!(
            unauthorized
                .headers()
                .get("access-control-allow-origin")
                .and_then(|value| value.to_str().ok()),
            Some("http://127.0.0.1:1420")
        );
        assert!(operator_state
            .lock()
            .expect("operator state")
            .tasks
            .is_empty());

        let forbidden = client
            .post(format!("{}/api/operator/tasks", base_url))
            .bearer_auth("remote-secret-canary")
            .header("x-acp-operator-client", "vscode-main")
            .json(&StartTaskRequest {
                project_path: denied_root.path().display().to_string(),
                ..request_body.clone()
            })
            .send()
            .await
            .expect("forbidden response");
        assert_eq!(forbidden.status(), reqwest::StatusCode::FORBIDDEN);
        assert!(operator_state
            .lock()
            .expect("operator state")
            .tasks
            .is_empty());

        let started: StartTaskResponse = client
            .post(format!("{}/api/operator/tasks", base_url))
            .bearer_auth("remote-secret-canary")
            .header("x-acp-operator-client", "vscode-main")
            .json(&request_body)
            .send()
            .await
            .expect("authorized response")
            .error_for_status()
            .expect("authorized status")
            .json()
            .await
            .expect("authorized response json");
        assert!(operator_state
            .lock()
            .expect("operator state")
            .tasks
            .contains_key(&started.task_id));
        let stored_path = operator_state
            .lock()
            .expect("operator state")
            .tasks
            .get(&started.task_id)
            .expect("started task")
            .project_path
            .clone();
        assert_eq!(
            PathBuf::from(stored_path),
            allowed_root.path().canonicalize().expect("canonical root")
        );

        let audit: Vec<RemoteAuditRecord> = client
            .get(format!("{}/api/operator/audit?limit=100", base_url))
            .bearer_auth("remote-secret-canary")
            .header("x-acp-operator-client", "vscode-main")
            .send()
            .await
            .expect("audit response")
            .error_for_status()
            .expect("audit status")
            .json()
            .await
            .expect("audit json");
        assert!(audit
            .iter()
            .any(|record| { record.action == "start_task" && record.outcome == "succeeded" }));
        assert!(audit.iter().any(|record| record.outcome == "denied"));
        let serialized = serde_json::to_string(&audit).expect("serialize audit");
        assert!(!serialized.contains("remote-secret-canary"));
        assert!(!serialized.contains("goal-secret-canary"));

        server.abort();
    }

    #[tokio::test]
    async fn test_remote_operator_cannot_read_or_control_existing_out_of_scope_task() {
        let allowed_root = tempfile::tempdir().expect("allowed project root");
        let denied_root = tempfile::tempdir().expect("denied project root");
        let operator_state = Arc::new(Mutex::new(OperatorState::new()));
        let local_task = {
            let mut state = operator_state.lock().expect("operator state");
            operator::start_task_in_state(
                &mut state,
                StartTaskRequest {
                    domain: "remote.test".to_string(),
                    project_path: denied_root.path().display().to_string(),
                    goal: "local private task".to_string(),
                    mode: None,
                    approval_policy: None,
                },
            )
            .expect("local task")
        };
        let policy = crate::operator::security::RemoteAccessPolicy::new(
            crate::operator::security::RemoteAccessPolicyConfig {
                token: Some("remote-secret-canary".to_string()),
                allowed_origins: vec!["http://127.0.0.1:1420".to_string()],
                allowed_clients: vec!["vscode-main".to_string()],
                allowed_domains: vec!["remote.test".to_string()],
                allowed_project_roots: vec![allowed_root.path().to_path_buf()],
            },
        )
        .expect("test policy");
        let state = new_server_state_with_security(operator_state.clone(), policy);
        let router = create_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("test listener address");
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve secured router");
        });
        let base_url = format!("http://{}", addr);
        let client = reqwest::Client::new();

        let tasks: Vec<OperatorTask> = client
            .get(format!("{}/api/operator/tasks", base_url))
            .bearer_auth("remote-secret-canary")
            .header("x-acp-operator-client", "vscode-main")
            .send()
            .await
            .expect("task list")
            .error_for_status()
            .expect("task list status")
            .json()
            .await
            .expect("task list json");
        assert!(tasks.is_empty());

        let stop = client
            .post(format!(
                "{}/api/operator/tasks/{}/stop",
                base_url, local_task.task_id
            ))
            .bearer_auth("remote-secret-canary")
            .header("origin", "http://127.0.0.1:1420")
            .header("x-acp-operator-client", "vscode-main")
            .send()
            .await
            .expect("stop response");
        assert_eq!(stop.status(), reqwest::StatusCode::FORBIDDEN);
        assert_eq!(
            stop.headers()
                .get("access-control-allow-origin")
                .and_then(|value| value.to_str().ok()),
            Some("http://127.0.0.1:1420")
        );
        assert_eq!(
            operator_state
                .lock()
                .expect("operator state")
                .tasks
                .get(&local_task.task_id)
                .expect("local task")
                .status,
            crate::operator::OperatorTaskStatus::Planning
        );

        server.abort();
    }

    #[tokio::test]
    async fn test_remote_operator_cors_preflight_uses_origin_allowlist_without_bearer() {
        let state = new_server_state();
        let router = create_router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("test listener address");
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("serve CORS router");
        });
        let base_url = format!("http://{}", addr);
        let client = reqwest::Client::new();

        let allowed = client
            .request(
                reqwest::Method::OPTIONS,
                format!("{}/api/operator/tasks", base_url),
            )
            .header("origin", "http://127.0.0.1:1420")
            .header("access-control-request-method", "POST")
            .header(
                "access-control-request-headers",
                "authorization,x-acp-operator-client,content-type",
            )
            .send()
            .await
            .expect("allowed preflight");
        assert!(allowed.status().is_success());
        assert_eq!(
            allowed
                .headers()
                .get("access-control-allow-origin")
                .and_then(|value| value.to_str().ok()),
            Some("http://127.0.0.1:1420")
        );

        let denied = client
            .request(
                reqwest::Method::OPTIONS,
                format!("{}/api/operator/tasks", base_url),
            )
            .header("origin", "https://attacker.example")
            .header("access-control-request-method", "POST")
            .send()
            .await
            .expect("denied preflight");
        assert_eq!(denied.status(), reqwest::StatusCode::FORBIDDEN);

        server.abort();
    }
}
