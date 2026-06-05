//! Feishu (Lark) Bot Adapter
//!
//! Handles bot commands via Feishu webhook/events.
//! Uses Feishu Open Platform API for message handling.

use crate::bot_adapters::{BotAdapter, BotCommand, BotResponse};
use crate::commands::gateway::FeishuConfig;
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;

/// Feishu message content
#[derive(Debug, Deserialize)]
struct FeishuMessageContent {
    content: String,
    msg_type: String,
}

/// Feishu event payload
#[derive(Debug, Deserialize)]
struct FeishuEvent {
    #[serde(rename = "type")]
    event_type: String,
    chat: Option<FeishuChat>,
    sender: Option<FeishuSender>,
    message: Option<FeishuMessageContent>,
}

/// Feishu chat info
#[derive(Debug, Deserialize)]
struct FeishuChat {
    chat_id: String,
    chat_type: String,
}

/// Feishu sender info
#[derive(Debug, Deserialize)]
struct FeishuSender {
    sender_id: FeishuSenderId,
}

/// Feishu sender ID
#[derive(Debug, Deserialize)]
struct FeishuSenderId {
    open_id: String,
    union_id: String,
    user_id: String,
}

/// Feishu API response
#[derive(Debug, Deserialize)]
struct FeishuResponse<T> {
    code: i32,
    msg: String,
    data: Option<T>,
}

/// Feishu Send Message request
#[derive(Debug, Serialize)]
struct SendMessageRequest {
    receive_id: String,
    receive_id_type: String,
    msg_type: String,
    content: String,
}

// ── Webhook payload types ───────────────────────────────────────────────

/// Feishu URL verification challenge request
/// See: https://open.feishu.cn/document/server-guide/event-subscription-guide/url-verification
#[derive(Debug, Deserialize)]
struct ChallengeRequest {
    challenge: Option<String>,
    token: Option<String>,
    #[serde(rename = "type")]
    event_type: Option<String>,
}

/// Response for URL verification
#[derive(Debug, Serialize)]
struct ChallengeResponse {
    challenge: String,
}

/// Feishu event callback wrapper (v2.0 schema)
#[derive(Debug, Deserialize)]
struct EventCallbackRequest {
    /// Schema version header
    schema: Option<String>,
    /// Event header
    header: Option<EventHeader>,
    /// Event body
    event: Option<EventBody>,
    /// v1.0 compat: challenge field may appear at top-level
    challenge: Option<String>,
    #[serde(rename = "type")]
    top_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventHeader {
    event_id: Option<String>,
    event_type: Option<String>,
    #[serde(rename = "token")]
    verification_token: Option<String>,
    app_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventBody {
    message: Option<EventMessage>,
    sender: Option<EventSender>,
}

#[derive(Debug, Deserialize)]
struct EventMessage {
    message_id: Option<String>,
    chat_id: Option<String>,
    chat_type: Option<String>,
    content: Option<String>,
    message_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventSender {
    sender_id: Option<EventSenderId>,
}

#[derive(Debug, Deserialize)]
struct EventSenderId {
    open_id: Option<String>,
    user_id: Option<String>,
    union_id: Option<String>,
}

/// Shared state passed to axum handlers
#[derive(Clone)]
struct WebhookState {
    adapter: Arc<FeishuAdapterShared>,
    app_handle: AppHandle,
}

/// The subset of adapter state the webhook handlers need (Send + Sync safe)
struct FeishuAdapterShared {
    app_id: String,
    app_secret: String,
    verification_token: Option<String>,
    tenant_token: Arc<RwLock<Option<String>>>,
    client: Client,
}

impl FeishuAdapterShared {
    async fn send_message(&self, chat_id: &str, text: &str) -> Result<(), String> {
        let token = {
            let cached = self.tenant_token.read().await;
            if let Some(t) = cached.as_ref() {
                t.clone()
            } else {
                drop(cached);
                let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
                let resp = self.client
                    .post(url)
                    .json(&serde_json::json!({
                        "app_id": self.app_id,
                        "app_secret": self.app_secret,
                    }))
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;

                let result: FeishuResponse<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;
                if result.code != 0 {
                    return Err(format!("Feishu auth error: {}", result.msg));
                }

                let tok = result.data
                    .as_ref()
                    .and_then(|d| d.get("tenant_access_token"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
                    .ok_or("No token in response")?;

                *self.tenant_token.write().await = Some(tok.clone());
                tok
            }
        };

        let url = "https://open.feishu.cn/open-apis/im/v1/messages";
        let content = serde_json::json!({ "text": text }).to_string();

        let request = SendMessageRequest {
            receive_id: chat_id.to_string(),
            receive_id_type: "chat_id".to_string(),
            msg_type: "text".to_string(),
            content,
        };

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[("receive_id_type", "chat_id")])
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Feishu API error: {}", response.status()));
        }

        Ok(())
    }
}

// ── Axum webhook handlers ──────────────────────────────────────────────

/// POST /webhook/challenge - Feishu URL verification
///
/// When you register a webhook URL, Feishu sends a challenge that must be
/// echoed back to confirm ownership.
async fn handle_challenge(
    State(state): State<WebhookState>,
    Json(body): Json<ChallengeRequest>,
) -> Json<ChallengeResponse> {
    let token = body.challenge.unwrap_or_default();

    // Optionally verify the `token` field matches our verification_token
    if let (Some(expected), Some(received)) = (&state.adapter.verification_token, &body.token) {
        if expected != received {
            tracing::warn!("Feishu challenge: verification_token mismatch");
        }
    }

    Json(ChallengeResponse { challenge: token })
}

/// POST /webhook/event - Feishu event callbacks (v2.0 and v1.0 schemas)
async fn handle_event(
    State(state): State<WebhookState>,
    Json(body): Json<EventCallbackRequest>,
) -> Json<serde_json::Value> {
    // v1.0 compat: if `challenge` is at the top level, echo it back
    if let Some(challenge) = body.challenge {
        return Json(serde_json::json!({ "challenge": challenge }));
    }

    // v2.0 event processing
    if let (Some(header), Some(event)) = (&body.header, &body.event) {
        let event_type = header.event_type.as_deref().unwrap_or("");

        // Verify token if configured
        if let (Some(expected), Some(received)) = (
            &state.adapter.verification_token,
            &header.verification_token,
        ) {
            if expected != received {
                tracing::warn!("Feishu event: verification_token mismatch");
                return Json(serde_json::json!({ "code": 401, "msg": "token mismatch" }));
            }
        }

        // Only handle message receive events
        if event_type == "im.message.receive_v1" {
            if let Some(msg) = &event.message {
                let chat_id = msg.chat_id.as_deref().unwrap_or("");
                let msg_type = msg.message_type.as_deref().unwrap_or("");

                // Only handle text messages
                if msg_type == "text" && !chat_id.is_empty() {
                    // Parse message content - Feishu wraps text in JSON: {"text":"..."}
                    let text = if let Some(content_str) = &msg.content {
                        serde_json::from_str::<serde_json::Value>(content_str)
                            .ok()
                            .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(String::from))
                            .unwrap_or_else(|| content_str.clone())
                    } else {
                        String::new()
                    };

                    if !text.is_empty() {
                        let adapter_shared = state.adapter.clone();
                        let app_handle = state.app_handle.clone();
                        let chat_id = chat_id.to_string();

                        // Process command in background so we don't block the webhook response
                        tokio::spawn(async move {
                            // Try to parse as a bot command
                            let cmd = crate::bot_adapters::parse_bot_text(&text);
                            let response = match cmd {
                                BotCommand::Unknown { .. } => {
                                    BotResponse {
                                        success: false,
                                        message: format!("Unknown command: {text}"),
                                        data: None,
                                    }
                                }
                                _ => {
                                    let app_adapter = crate::bot_adapters::AppWsAdapter::new();
                                    app_adapter.handle_command(cmd, &app_handle).await
                                }
                            };

                            let reply = crate::bot_adapters::format_response(&response, "feishu");
                            if let Err(e) = adapter_shared.send_message(&chat_id, &reply).await {
                                tracing::error!("Feishu send_message error: {e}");
                            }
                        });
                    }
                }
            }
        }
    }

    // Always respond with success to avoid Feishu retries
    Json(serde_json::json!({ "code": 0, "msg": "ok" }))
}

/// Feishu Bot Adapter
pub struct FeishuAdapter {
    app_id: String,
    app_secret: String,
    verification_token: Option<String>,
    webhook_port: u16,
    tenant_token: Arc<RwLock<Option<String>>>,
    client: Client,
    running: Arc<RwLock<bool>>,
}

impl FeishuAdapter {
    pub fn new(config: &FeishuConfig) -> Self {
        Self {
            app_id: config.app_id.clone(),
            app_secret: config.app_secret.clone(),
            verification_token: config.verification_token.clone(),
            webhook_port: 9876,
            tenant_token: Arc::new(RwLock::new(None)),
            client: Client::new(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Get tenant access token
    async fn get_tenant_token(&self) -> Result<String, String> {
        // Check cached token
        {
            let token = self.tenant_token.read().await;
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
            }
        }

        // Fetch new token
        let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
        let response = self.client
            .post(url)
            .json(&serde_json::json!({
                "app_id": self.app_id,
                "app_secret": self.app_secret,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let result: FeishuResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if result.code != 0 {
            return Err(format!("Feishu auth error: {}", result.msg));
        }

        let token = result.data
            .as_ref()
            .and_then(|d| d.get("tenant_access_token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or("No token in response")?;

        // Cache token
        *self.tenant_token.write().await = Some(token.clone());

        Ok(token)
    }

    /// Send message to Feishu chat
    async fn send_message(&self, chat_id: &str, text: &str) -> Result<(), String> {
        let token = self.get_tenant_token().await?;
        let url = "https://open.feishu.cn/open-apis/im/v1/messages";

        let content = serde_json::json!({
            "text": text
        }).to_string();

        let request = SendMessageRequest {
            receive_id: chat_id.to_string(),
            receive_id_type: "chat_id".to_string(),
            msg_type: "text".to_string(),
            content,
        };

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[("receive_id_type", "chat_id")])
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Feishu API error: {}", response.status()));
        }

        Ok(())
    }
}

impl BotAdapter for FeishuAdapter {
    fn platform(&self) -> &str {
        "feishu"
    }

    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }

    async fn start(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        if self.app_id.is_empty() || self.app_secret.is_empty() {
            return Err("Feishu app_id or app_secret not configured".to_string());
        }

        *self.running.write().await = true;

        // Build shared state for axum handlers
        let shared = Arc::new(FeishuAdapterShared {
            app_id: self.app_id.clone(),
            app_secret: self.app_secret.clone(),
            verification_token: self.verification_token.clone(),
            tenant_token: Arc::clone(&self.tenant_token),
            client: self.client.clone(),
        });

        let state = WebhookState {
            adapter: shared,
            app_handle: app_handle.clone(),
        };

        let port = self.webhook_port;

        // Build axum router
        let app = Router::new()
            .route("/webhook/challenge", post(handle_challenge))
            .route("/webhook/event", post(handle_event))
            .with_state(state);

        // Spawn the server in a background tokio task
        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
                .await
                .expect("Failed to bind Feishu webhook port");

            tracing::info!("Feishu webhook server listening on 0.0.0.0:{port}");

            axum::serve(listener, app)
                .await
                .expect("Feishu webhook server error");
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        *self.running.write().await = false;
        Ok(())
    }

    fn parse_command(&self, text: &str) -> Option<BotCommand> {
        let cmd = crate::bot_adapters::parse_bot_text(text);
        match cmd {
            BotCommand::Unknown { .. } => None,
            _ => Some(cmd),
        }
    }

    async fn handle_command(&self, command: BotCommand, app_handle: &AppHandle) -> BotResponse {
        // Use AppWsAdapter logic (same command handling)
        let app_adapter = crate::bot_adapters::AppWsAdapter::new();
        app_adapter.handle_command(command, app_handle).await
    }
}

impl Default for FeishuAdapter {
    fn default() -> Self {
        Self::new(&FeishuConfig {
            app_id: String::new(),
            app_secret: String::new(),
            encrypt_key: None,
            verification_token: None,
            enabled: false,
        })
    }
}