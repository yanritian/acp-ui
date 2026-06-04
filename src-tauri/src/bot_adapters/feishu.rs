//! Feishu (Lark) Bot Adapter
//!
//! Handles bot commands via Feishu webhook/events.
//! Uses Feishu Open Platform API for message handling.

use crate::bot_adapters::{BotAdapter, BotCommand, BotResponse};
use crate::commands::gateway::FeishuConfig;
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

/// Feishu Bot Adapter
pub struct FeishuAdapter {
    app_id: String,
    app_secret: String,
    tenant_token: Arc<RwLock<Option<String>>>,
    client: Client,
    running: Arc<RwLock<bool>>,
}

impl FeishuAdapter {
    pub fn new(config: &FeishuConfig) -> Self {
        Self {
            app_id: config.app_id.clone(),
            app_secret: config.app_secret.clone(),
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

    async fn start(&mut self, _app_handle: &AppHandle) -> Result<(), String> {
        if self.app_id.is_empty() || self.app_secret.is_empty() {
            return Err("Feishu app_id or app_secret not configured".to_string());
        }

        *self.running.write().await = true;

        // Note: Feishu requires webhook server for receiving events
        // For now, mark as running and rely on manual command handling
        // Full implementation would need a webhook endpoint

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