//! Feishu Rich Message Service
//!
//! Supports sending non-text messages (images, files, cards) via Feishu API.
//! Based on remote_claude implementation analysis.
//!
//! SECURITY: app_id and app_secret should be provided via environment variables:
//! - FEISHU_APP_ID
//! - FEISHU_APP_SECRET
//!
//! NOTE: Module is imported but commands not yet registered in lib.rs - marked for future use.

#![allow(dead_code)]

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Feishu API endpoints
const FEISHU_BASE_URL: &str = "https://open.feishu.cn/open-apis";
const AUTH_URL: &str = "/auth/v3/tenant_access_token/internal";
const MESSAGE_URL: &str = "/im/v1/messages";
const IMAGE_URL: &str = "/im/v1/images";
const FILE_URL: &str = "/drive/v1/files/upload_all";

/// Token expiry buffer (refresh 5 minutes before actual expiry)
const TOKEN_EXPIRY_BUFFER: Duration = Duration::from_secs(300);

/// Feishu Rich Message Service (with auto token refresh)
pub struct FeishuRichMessageService {
    app_id: String,
    app_secret: String,
    tenant_token: Option<String>,
    token_expiry: Option<Instant>,
    client: Client,
}

impl FeishuRichMessageService {
    /// Create service from environment variables (recommended)
    pub fn from_env() -> Result<Self, String> {
        let app_id =
            std::env::var("FEISHU_APP_ID").map_err(|_| "FEISHU_APP_ID not set in environment")?;
        let app_secret = std::env::var("FEISHU_APP_SECRET")
            .map_err(|_| "FEISHU_APP_SECRET not set in environment")?;
        Ok(Self::new(app_id, app_secret))
    }

    /// Create service with explicit credentials (use from_env() for security)
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            tenant_token: None,
            token_expiry: None,
            client: Client::new(),
        }
    }

    /// Check if token needs refresh
    fn needs_refresh(&self) -> bool {
        match &self.token_expiry {
            Some(expiry) => Instant::now() >= *expiry,
            None => true,
        }
    }

    /// Get tenant access token (auto-refresh if expired)
    pub async fn get_tenant_token(&mut self) -> Result<String, String> {
        // Check if we need to refresh the token
        if self.needs_refresh() {
            self.refresh_token().await?;
        }
        self.tenant_token
            .clone()
            .ok_or_else(|| "Token not available".to_string())
    }

    /// Refresh tenant access token
    async fn refresh_token(&mut self) -> Result<(), String> {
        let url = format!("{}{}", FEISHU_BASE_URL, AUTH_URL);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "app_id": self.app_id,
                "app_secret": self.app_secret
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to request token: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if json["code"].as_i64() != Some(0) {
            return Err(format!(
                "Feishu API error: {}",
                json["msg"].as_str().unwrap_or("unknown")
            ));
        }

        let token = json["tenant_access_token"]
            .as_str()
            .ok_or_else(|| "Token not found in response".to_string())?
            .to_string();

        // Token expires in 2 hours (7200 seconds), set expiry with buffer
        let expire_seconds = json["expire"].as_i64().unwrap_or(7200) as u64;
        let expiry = Instant::now() + Duration::from_secs(expire_seconds) - TOKEN_EXPIRY_BUFFER;

        self.tenant_token = Some(token);
        self.token_expiry = Some(expiry);

        println!(
            "Feishu token refreshed, expires in {} seconds",
            expire_seconds
        );
        Ok(())
    }

    /// Send text message (with auto token refresh)
    pub async fn send_text(
        &mut self,
        receive_id: &str,
        receive_id_type: &str,
        text: &str,
    ) -> Result<String, String> {
        let token = self.get_tenant_token().await?;

        let url = format!(
            "{}{}?receive_id_type={}",
            FEISHU_BASE_URL, MESSAGE_URL, receive_id_type
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "receive_id": receive_id,
                "msg_type": "text",
                "content": serde_json::json!({"text": text}).to_string()
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to send text: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        self.parse_message_response(&body)
    }

    /// Send image message (with auto token refresh)
    pub async fn send_image(
        &mut self,
        receive_id: &str,
        receive_id_type: &str,
        image_data: &[u8],
        image_type: &str,
    ) -> Result<String, String> {
        // Upload image first
        let image_key = self.upload_image(image_data, image_type).await?;

        // Send image message
        let token = self.get_tenant_token().await?;

        let url = format!(
            "{}{}?receive_id_type={}",
            FEISHU_BASE_URL, MESSAGE_URL, receive_id_type
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "receive_id": receive_id,
                "msg_type": "image",
                "content": serde_json::json!({"image_key": image_key}).to_string()
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to send image message: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        self.parse_message_response(&body)
    }

    /// Upload image to Feishu (with auto token refresh)
    async fn upload_image(
        &mut self,
        image_data: &[u8],
        image_type: &str,
    ) -> Result<String, String> {
        let token = self.get_tenant_token().await?;

        let url = format!("{}{}", FEISHU_BASE_URL, IMAGE_URL);

        // Create multipart form
        let part = reqwest::multipart::Part::bytes(image_data.to_vec())
            .file_name("image")
            .mime_str(image_type)
            .map_err(|e| format!("Failed to set mime type: {}", e))?;

        let form = reqwest::multipart::Form::new()
            .part("image", part)
            .text("image_type", "message");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to upload image: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if json["code"].as_i64() != Some(0) {
            return Err(format!(
                "Image upload error: {}",
                json["msg"].as_str().unwrap_or("unknown")
            ));
        }

        Ok(json["data"]["image_key"]
            .as_str()
            .ok_or_else(|| "image_key not found".to_string())?
            .to_string())
    }

    /// Send interactive card message (with auto token refresh)
    pub async fn send_card(
        &mut self,
        receive_id: &str,
        receive_id_type: &str,
        card: &FeishuCard,
    ) -> Result<String, String> {
        let token = self.get_tenant_token().await?;

        let url = format!(
            "{}{}?receive_id_type={}",
            FEISHU_BASE_URL, MESSAGE_URL, receive_id_type
        );

        let card_json = serde_json::to_string(&serde_json::json!({
            "type": "template",
            "data": {
                "template_id": card.template_id,
                "template_variable": card.variables
            }
        }))
        .map_err(|e| format!("Failed to serialize card: {}", e))?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "receive_id": receive_id,
                "msg_type": "interactive",
                "content": serde_json::json!({
                    "type": "card",
                    "data": serde_json::json!({
                        "card": card_json
                    })
                }).to_string()
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to send card: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        self.parse_message_response(&body)
    }

    /// Send file message (for small files < 20MB, with auto token refresh)
    pub async fn send_file(
        &mut self,
        receive_id: &str,
        receive_id_type: &str,
        file_data: &[u8],
        file_name: &str,
        file_type: &str,
    ) -> Result<String, String> {
        // Upload file first
        let file_token = self.upload_file(file_data, file_name, file_type).await?;

        // Send file message
        let token = self.get_tenant_token().await?;

        let url = format!(
            "{}{}?receive_id_type={}",
            FEISHU_BASE_URL, MESSAGE_URL, receive_id_type
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "receive_id": receive_id,
                "msg_type": "file",
                "content": serde_json::json!({"file_key": file_token}).to_string()
            }))
            .send()
            .await
            .map_err(|e| format!("Failed to send file message: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        self.parse_message_response(&body)
    }

    /// Upload file to Feishu Drive (with auto token refresh)
    async fn upload_file(
        &mut self,
        file_data: &[u8],
        file_name: &str,
        file_type: &str,
    ) -> Result<String, String> {
        let token = self.get_tenant_token().await?;

        let url = format!("{}{}", FEISHU_BASE_URL, FILE_URL);

        // Convert to owned strings to satisfy 'static lifetime requirement
        let file_name_owned = file_name.to_string();

        let part = reqwest::multipart::Part::bytes(file_data.to_vec())
            .file_name(file_name_owned.clone())
            .mime_str(file_type)
            .map_err(|e| format!("Failed to set mime type: {}", e))?;

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("file_name", file_name_owned)
            .text("parent_type", "ccm_resource")
            .text("parent_node", "ccm_global_root_folder");

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to upload file: {}", e))?;

        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let json: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if json["code"].as_i64() != Some(0) {
            return Err(format!(
                "File upload error: {}",
                json["msg"].as_str().unwrap_or("unknown")
            ));
        }

        Ok(json["data"]["file_token"]
            .as_str()
            .ok_or_else(|| "file_token not found".to_string())?
            .to_string())
    }

    /// Parse message response to get message_id
    fn parse_message_response(&self, body: &str) -> Result<String, String> {
        let json: serde_json::Value =
            serde_json::from_str(body).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        if json["code"].as_i64() != Some(0) {
            return Err(format!(
                "Message error: {}",
                json["msg"].as_str().unwrap_or("unknown")
            ));
        }

        Ok(json["data"]["message_id"]
            .as_str()
            .ok_or_else(|| "message_id not found".to_string())?
            .to_string())
    }
}

/// Interactive card structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuCard {
    template_id: String,
    variables: serde_json::Value,
}

impl FeishuCard {
    pub fn new(template_id: &str, variables: serde_json::Value) -> Self {
        Self {
            template_id: template_id.to_string(),
            variables,
        }
    }

    /// Create a status card
    pub fn status_card(title: &str, content: &str, status: &str) -> Self {
        Self::new(
            "AAqkWzJ7R",
            serde_json::json!({
                "title": title,
                "content": content,
                "status": status
            }),
        )
    }

    /// Create a stream output card (for code execution)
    pub fn stream_card(title: &str, code_output: &str) -> Self {
        Self::new(
            "AAqkWzJ7S",
            serde_json::json!({
                "title": title,
                "code_output": code_output
            }),
        )
    }

    /// Create a menu card with buttons
    pub fn menu_card(title: &str, options: &[(&str, &str)]) -> Self {
        let buttons = options
            .iter()
            .map(|(text, value)| {
                serde_json::json!({
                    "text": text,
                    "value": value
                })
            })
            .collect::<Vec<_>>();

        Self::new(
            "AAqkWzJ7T",
            serde_json::json!({
                "title": title,
                "buttons": buttons
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_creation() {
        let card = FeishuCard::status_card("Test Title", "Test Content", "running");
        assert_eq!(card.template_id, "AAqkWzJ7R");
    }

    #[test]
    fn test_menu_card() {
        let card = FeishuCard::menu_card("Choose Action", &[("Run", "run"), ("Stop", "stop")]);
        assert!(card.variables["buttons"].is_array());
    }
}
