//! Telegram Bot Adapter
//!
//! Handles bot commands via Telegram long polling.
//! Uses reqwest to poll Telegram API for updates.

use crate::bot_adapters::{BotAdapter, BotCommand, BotResponse, format_response};
use crate::TelegramConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;

/// Telegram API response
#[derive(Debug, Deserialize)]
struct TelegramResponse<T> {
    ok: bool,
    result: T,
}

/// Telegram Update
#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
}

/// Telegram Message
#[derive(Debug, Deserialize)]
struct TelegramMessage {
    message_id: i64,
    chat: TelegramChat,
    text: Option<String>,
    from: Option<TelegramUser>,
}

/// Telegram Chat
#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
    #[serde(rename = "type")]
    chat_type: String,
}

/// Telegram User
#[derive(Debug, Deserialize)]
struct TelegramUser {
    id: i64,
    username: Option<String>,
}

/// Telegram Send Message response
#[derive(Debug, Serialize)]
struct SendMessageRequest {
    chat_id: i64,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<String>,
}

/// Telegram Bot Adapter
pub struct TelegramAdapter {
    bot_token: String,
    allowed_chat_ids: Vec<i64>,
    client: Client,
    running: Arc<RwLock<bool>>,
    last_update_id: Arc<RwLock<i64>>,
}

impl TelegramAdapter {
    pub fn new(config: &TelegramConfig) -> Self {
        Self {
            bot_token: config.bot_token.clone(),
            allowed_chat_ids: Vec::new(), // Will be populated from first message
            client: Client::new(),
            running: Arc::new(RwLock::new(false)),
            last_update_id: Arc::new(RwLock::new(0)),
        }
    }

    /// Get API URL for given method
    fn api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.bot_token, method)
    }

    /// Send message to Telegram chat
    async fn send_message(&self, chat_id: i64, text: &str) -> Result<(), String> {
        let url = self.api_url("sendMessage");
        let request = SendMessageRequest {
            chat_id,
            text: text.to_string(),
            parse_mode: None,
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Telegram API error: {}", response.status()));
        }

        Ok(())
    }

    /// Poll for updates
    async fn poll_updates(&self, timeout: i64) -> Result<Vec<TelegramUpdate>, String> {
        let url = self.api_url("getUpdates");
        let last_id = *self.last_update_id.read().await;

        let response = self.client
            .get(&url)
            .query(&[
                ("offset", last_id + 1),
                ("timeout", timeout),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Telegram API error: {}", response.status()));
        }

        let result: TelegramResponse<Vec<TelegramUpdate>> = response
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.result)
    }
}

impl BotAdapter for TelegramAdapter {
    fn platform(&self) -> &str {
        "telegram"
    }

    fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }

    async fn start(&mut self, app_handle: &AppHandle) -> Result<(), String> {
        if self.bot_token.is_empty() {
            return Err("Telegram bot token not configured".to_string());
        }

        *self.running.write().await = true;

        // Spawn polling loop
        let running = Arc::clone(&self.running);
        let last_update_id = Arc::clone(&self.last_update_id);
        let client = self.client.clone();
        let bot_token = self.bot_token.clone();
        let allowed_chat_ids = Arc::new(RwLock::new(Vec::<i64>::new()));
        let app_handle_clone = app_handle.clone();

        tokio::spawn(async move {
            let adapter = TelegramAdapter {
                bot_token,
                allowed_chat_ids: Vec::new(),
                client,
                running: Arc::clone(&running),
                last_update_id: Arc::clone(&last_update_id),
            };

            while *running.read().await {
                if let Ok(updates) = adapter.poll_updates(30).await {
                    for update in updates {
                        // Update last_id
                        *last_update_id.write().await = update.update_id;

                        // Process message
                        if let Some(message) = update.message {
                            if let Some(text) = message.text {
                                // Check if chat is allowed (first message auto-adds)
                                let chat_allowed = {
                                    let ids = allowed_chat_ids.read().await;
                                    ids.contains(&message.chat.id) || ids.is_empty()
                                };

                                if chat_allowed {
                                    // Auto-add first chat
                                    if allowed_chat_ids.read().await.is_empty() {
                                        allowed_chat_ids.write().await.push(message.chat.id);
                                    }

                                    // Parse and handle command
                                    let cmd = crate::bot_adapters::parse_bot_text(&text);
                                    let response = adapter.handle_command(cmd, &app_handle_clone).await;
                                    let formatted = format_response(&response, "telegram");

                                    // Send response
                                    let _ = adapter.send_message(message.chat.id, &formatted).await;
                                } else {
                                    let _ = adapter.send_message(
                                        message.chat.id,
                                        "此 Chat ID 未授权。请在桌面端配置中添加。"
                                    ).await;
                                }
                            }
                        }
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
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

impl Default for TelegramAdapter {
    fn default() -> Self {
        Self::new(&TelegramConfig {
            bot_token: String::new(),
            enabled: false,
        })
    }
}