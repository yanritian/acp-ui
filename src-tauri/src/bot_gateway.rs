// ============================================================================
// Bot Gateway Layer
// ============================================================================
// Unified rich-response infrastructure for multi-platform bot integration.
// Supports: Feishu, DingTalk, WeChat, WeCom, Telegram, Discord.
//
// Key types:
//   RichResponse   - Platform-agnostic response envelope
//   BotAdapter     - Per-platform send trait (sync API for simplicity)
//   GatewayRouter  - Registration, routing, broadcast, normalization
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 1. Unified Rich Response Types
// ============================================================================

/// Style for card buttons
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ButtonStyle {
    Primary,
    Default,
    Danger,
}

/// An interactive button on a response card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardButton {
    pub text: String,
    pub action: String,
    pub style: ButtonStyle,
}

/// An interactive card that can be displayed on supporting platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCard {
    pub title: String,
    pub subtitle: Option<String>,
    pub progress: Option<u8>, // 0-100
    pub buttons: Vec<CardButton>,
}

/// Type of media attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentType {
    Image,
    Video,
    Document,
    Audio,
}

/// A file/media attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub attachment_type: AttachmentType,
    pub url: String,
    pub name: String,
}

/// Platform-agnostic rich response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichResponse {
    pub text: String,
    pub card: Option<ResponseCard>,
    pub attachments: Vec<Attachment>,
}

impl RichResponse {
    /// Create a simple text-only response
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            card: None,
            attachments: Vec::new(),
        }
    }

    /// Create a response with a card
    pub fn card(card: ResponseCard) -> Self {
        Self {
            text: card.title.clone(),
            card: Some(card),
            attachments: Vec::new(),
        }
    }

    /// Add an attachment
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Check if response is non-empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
            && self.card.is_none()
            && self.attachments.is_empty()
    }
}

// ============================================================================
// 2. BotAdapter Trait
// ============================================================================

/// Per-platform adapter for sending rich responses.
///
/// Uses a synchronous API for simplicity. Implementations that need async
/// internally should use tokio::task::block_in_place or a sync HTTP client.
pub trait BotAdapter: Send + Sync {
    /// Platform identifier (e.g., "feishu", "telegram", "discord")
    fn platform_name(&self) -> &str;

    /// Send a rich response to a specific chat
    fn send_rich_response(&self, chat_id: &str, response: &RichResponse) -> Result<(), String>;

    /// Send plain text to a specific chat
    fn send_text(&self, chat_id: &str, text: &str) -> Result<(), String>;

    /// Whether this platform supports interactive cards
    fn supports_cards(&self) -> bool;

    /// Whether this platform supports interactive buttons
    fn supports_buttons(&self) -> bool;

    /// Whether this platform supports file/media attachments
    fn supports_attachments(&self) -> bool;
}

// ============================================================================
// 3. Platform Adapters
// ============================================================================

// ---- Feishu Adapter ----

/// Feishu (Lark) Bot Gateway Adapter
///
/// Supports: cards (interactive), buttons (via card), attachments (image keys).
/// Uses the Feishu Open Platform API for message delivery.
pub struct FeishuGatewayAdapter {
    app_id: String,
    app_secret: String,
    tenant_token: std::sync::Mutex<Option<CachedToken>>,
    client: ureq::Agent,
}

/// Cached token with expiry (Feishu tokens expire after 2 hours)
struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl FeishuGatewayAdapter {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            tenant_token: std::sync::Mutex::new(None),
            client: ureq::Agent::new(),
        }
    }

    /// Obtain (and cache) the tenant access token.
    /// Feishu tokens expire after 2 hours; we refresh when < 5 minutes remain.
    fn ensure_token(&self) -> Result<String, String> {
        // Check cache with expiry
        {
            let guard = self.tenant_token.lock().map_err(|e| e.to_string())?;
            if let Some(ref cached) = *guard {
                let now = chrono::Utc::now();
                if now + chrono::Duration::minutes(5) < cached.expires_at {
                    return Ok(cached.token.clone());
                }
                // Token expired or near-expiry — will refresh below
            }
        }

        // Fetch from Feishu API
        let resp = self.client
            .post("https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal")
            .send_json(&ureq::json!({
                "app_id": self.app_id,
                "app_secret": self.app_secret,
            }))
            .map_err(|e| e.to_string())?;

        let body: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
        let code = body.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
        if code != 0 {
            let msg = body.get("msg").and_then(|v| v.as_str()).unwrap_or("unknown");
            return Err(format!("Feishu auth error (code {}): {}", code, msg));
        }

        let token = body
            .get("tenant_access_token")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or("No tenant_access_token in response")?;

        // Cache with 2-hour expiry (Feishu default TTL)
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(2);
        if let Ok(mut guard) = self.tenant_token.lock() {
            *guard = Some(CachedToken { token: token.clone(), expires_at });
        }

        Ok(token)
    }

    /// Send text message via Feishu API
    fn feishu_send_text(&self, chat_id: &str, text: &str, token: &str) -> Result<(), String> {
        let content = serde_json::json!({ "text": text }).to_string();
        let body = serde_json::json!({
            "receive_id": chat_id,
            "msg_type": "text",
            "content": content,
        });

        let resp = self.client
            .post("https://open.feishu.cn/open-apis/im/v1/messages")
            .set("Authorization", &format!("Bearer {}", token))
            .query("receive_id_type", "chat_id")
            .send_json(&body)
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        if status != 200 {
            return Err(format!("Feishu API error: HTTP {}", status));
        }

        Ok(())
    }

    /// Send interactive card message via Feishu API
    fn feishu_send_card(&self, chat_id: &str, card: &ResponseCard, token: &str) -> Result<(), String> {
        let mut elements: Vec<serde_json::Value> = Vec::new();

        // Title element
        elements.push(serde_json::json!({
            "tag": "markdown",
            "content": format!("**{}**", card.title),
        }));

        // Subtitle
        if let Some(ref sub) = card.subtitle {
            elements.push(serde_json::json!({
                "tag": "markdown",
                "content": sub.clone(),
            }));
        }

        // Progress bar
        if let Some(progress) = card.progress {
            elements.push(serde_json::json!({
                "tag": "markdown",
                "content": format!("Progress: {}%", progress),
            }));
        }

        // Buttons (Feishu calls them "actions")
        if !card.buttons.is_empty() {
            let action_elements: Vec<serde_json::Value> = card.buttons
                .iter()
                .map(|b| {
                    let btn_type = match b.style {
                        ButtonStyle::Primary => "primary",
                        ButtonStyle::Danger => "danger",
                        ButtonStyle::Default => "default",
                    };
                    serde_json::json!({
                        "tag": "button",
                        "text": { "tag": "plain_text", "content": b.text },
                        "type": btn_type,
                        "value": { "action": b.action },
                    })
                })
                .collect();

            elements.push(serde_json::json!({
                "tag": "action",
                "actions": action_elements,
            }));
        }

        let card_json = serde_json::json!({
            "config": { "wide_screen_mode": true },
            "header": {
                "title": { "tag": "plain_text", "content": &card.title },
            },
            "elements": elements,
        });

        let body = serde_json::json!({
            "receive_id": chat_id,
            "msg_type": "interactive",
            "content": card_json.to_string(),
        });

        let resp = self.client
            .post("https://open.feishu.cn/open-apis/im/v1/messages")
            .set("Authorization", &format!("Bearer {}", token))
            .query("receive_id_type", "chat_id")
            .send_json(&body)
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        if status != 200 {
            return Err(format!("Feishu card API error: HTTP {}", status));
        }

        Ok(())
    }
}

impl BotAdapter for FeishuGatewayAdapter {
    fn platform_name(&self) -> &str {
        "feishu"
    }

    fn send_rich_response(&self, chat_id: &str, response: &RichResponse) -> Result<(), String> {
        let token = self.ensure_token()?;

        // Send card first if supported and present
        if let Some(ref card) = response.card {
            self.feishu_send_card(chat_id, card, &token)?;
        }

        // Send text (always, as fallback or additional context)
        if !response.text.is_empty() {
            self.feishu_send_text(chat_id, &response.text, &token)?;
        }

        // Send attachments as image messages
        for attachment in &response.attachments {
            if attachment.attachment_type == AttachmentType::Image {
                // Feishu image message requires uploading image first then using image_key
                // For now, send as text with URL
                self.feishu_send_text(
                    chat_id,
                    &format!("[{}]({})", attachment.name, attachment.url),
                    &token,
                )?;
            }
        }

        Ok(())
    }

    fn send_text(&self, chat_id: &str, text: &str) -> Result<(), String> {
        let token = self.ensure_token()?;
        self.feishu_send_text(chat_id, text, &token)
    }

    fn supports_cards(&self) -> bool { true }
    fn supports_buttons(&self) -> bool { true }
    fn supports_attachments(&self) -> bool { true }
}

// ---- DingTalk Adapter (Stub) ----

/// DingTalk Bot Gateway Adapter (stub implementation)
///
/// Supports: cards (markdown/actionCard), buttons (via actionCard),
/// attachments (media upload).
/// Structure is ready; full API integration pending.
pub struct DingtalkGatewayAdapter {
    webhook_url: String,
    access_token: Option<String>,
    client: ureq::Agent,
}

impl DingtalkGatewayAdapter {
    pub fn new(webhook_url: String, access_token: Option<String>) -> Self {
        Self {
            webhook_url,
            access_token,
            client: ureq::Agent::new(),
        }
    }
}

impl BotAdapter for DingtalkGatewayAdapter {
    fn platform_name(&self) -> &str {
        "dingtalk"
    }

    fn send_rich_response(&self, _chat_id: &str, response: &RichResponse) -> Result<(), String> {
        // TODO: Implement DingTalk markdown / actionCard / OA message sending
        // DingTalk supports:
        //   - text messages via webhook
        //   - markdown cards via webhook
        //   - actionCard with buttons via webhook
        //   - OA message cards via server API
        let text = if !response.text.is_empty() {
            response.text.clone()
        } else if let Some(ref card) = response.card {
            format!("**{}**\n{}", card.title, card.subtitle.as_deref().unwrap_or(""))
        } else {
            String::new()
        };

        if text.is_empty() {
            return Err("DingTalk: no content to send".into());
        }

        // Stub: log the would-be payload
        tracing::info!("[dingtalk] would send: {}", text);
        Ok(())
    }

    fn send_text(&self, _chat_id: &str, text: &str) -> Result<(), String> {
        // TODO: POST to DingTalk webhook with text payload
        tracing::info!("[dingtalk] would send text: {}", text);
        Ok(())
    }

    fn supports_cards(&self) -> bool { true }
    fn supports_buttons(&self) -> bool { true }
    fn supports_attachments(&self) -> bool { true }
}

// ---- WeChat Adapter (Stub) ----

/// WeChat (Personal) Bot Gateway Adapter (stub implementation)
///
/// Supports: text, image, video, file (via WeChat protocol).
/// Note: Personal WeChat has limited official API support.
/// Production should consider using WeCom (Enterprise WeChat) instead.
pub struct WechatGatewayAdapter {
    client: ureq::Agent,
}

impl WechatGatewayAdapter {
    pub fn new() -> Self {
        Self {
            client: ureq::Agent::new(),
        }
    }
}

impl Default for WechatGatewayAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BotAdapter for WechatGatewayAdapter {
    fn platform_name(&self) -> &str {
        "wechat"
    }

    fn send_rich_response(&self, _chat_id: &str, response: &RichResponse) -> Result<(), String> {
        // TODO: Implement WeChat message sending
        // WeChat personal has no official public API.
        // Options:
        //   - wxbot / web WeChat protocol (unofficial, risk of ban)
        //   - Recommend migrating to WeCom for official API support
        let mut text = response.text.clone();
        if let Some(ref card) = response.card {
            if text.is_empty() {
                text = card.title.clone();
            }
        }
        tracing::info!("[wechat] would send: {}", text);
        Ok(())
    }

    fn send_text(&self, _chat_id: &str, text: &str) -> Result<(), String> {
        tracing::info!("[wechat] would send text: {}", text);
        Ok(())
    }

    fn supports_cards(&self) -> bool { false }
    fn supports_buttons(&self) -> bool { false }
    fn supports_attachments(&self) -> bool { true }
}

// ---- WeCom (Enterprise WeChat) Adapter (Stub) ----

/// WeCom (Enterprise WeChat) Bot Gateway Adapter (stub implementation)
///
/// Supports: text, image, file, voice, video, markdown cards.
/// Uses WeCom Bot API or Group Robot Webhook.
pub struct WecomGatewayAdapter {
    corp_id: String,
    agent_secret: String,
    agent_id: String,
    client: ureq::Agent,
}

impl WecomGatewayAdapter {
    pub fn new(corp_id: String, agent_secret: String, agent_id: String) -> Self {
        Self {
            corp_id,
            agent_secret,
            agent_id,
            client: ureq::Agent::new(),
        }
    }
}

impl BotAdapter for WecomGatewayAdapter {
    fn platform_name(&self) -> &str {
        "wecom"
    }

    fn send_rich_response(&self, _chat_id: &str, response: &RichResponse) -> Result<(), String> {
        // TODO: Implement WeCom message sending
        // WeCom supports:
        //   - Text messages via server API
        //   - Image / File / Video via media upload + send
        //   - TextCard (simple card with title + description + URL)
        //   - Markdown messages (in group robots)
        let mut text = response.text.clone();
        if let Some(ref card) = response.card {
            if text.is_empty() {
                text = format!("**{}**\n{}", card.title, card.subtitle.as_deref().unwrap_or(""));
            }
        }
        tracing::info!("[wecom] would send: {}", text);
        Ok(())
    }

    fn send_text(&self, _chat_id: &str, text: &str) -> Result<(), String> {
        tracing::info!("[wecom] would send text: {}", text);
        Ok(())
    }

    fn supports_cards(&self) -> bool { true }
    fn supports_buttons(&self) -> bool { false }
    fn supports_attachments(&self) -> bool { true }
}

// ---- Telegram Adapter (Stub with structure) ----

/// Telegram Bot Gateway Adapter (stub with full structure)
///
/// Supports: text, photo, document, video, audio, inline keyboards.
/// Uses Telegram Bot API (sendText, sendPhoto, sendDocument, etc.).
pub struct TelegramGatewayAdapter {
    bot_token: String,
    client: ureq::Agent,
}

impl TelegramGatewayAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            client: ureq::Agent::new(),
        }
    }

    fn api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.bot_token, method)
    }
}

impl BotAdapter for TelegramGatewayAdapter {
    fn platform_name(&self) -> &str {
        "telegram"
    }

    fn send_rich_response(&self, chat_id: &str, response: &RichResponse) -> Result<(), String> {
        // TODO: Full Telegram implementation
        // Telegram supports:
        //   - sendMessage with Markdown/HTML parse mode
        //   - sendPhoto, sendDocument, sendVideo, sendAudio
        //   - InlineKeyboardMarkup with callback buttons
        //   - ReplyKeyboardMarkup
        //
        // Implementation sketch:
        //   1. If card present, build text with card info + inline keyboard
        //   2. Send text via sendMessage
        //   3. Send attachments via respective media endpoints

        let mut text = response.text.clone();

        // Convert card to text for Telegram (until full impl)
        if let Some(ref card) = response.card {
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(&format!("**{}**", card.title));
            if let Some(ref sub) = card.subtitle {
                text.push_str(&format!("\n_{}_", sub));
            }
            if let Some(progress) = card.progress {
                text.push_str(&format!("\nProgress: {}%", progress));
            }
            if !card.buttons.is_empty() {
                text.push_str("\n\nOptions:");
                for (i, btn) in card.buttons.iter().enumerate() {
                    text.push_str(&format!("\n{}. {} ({})", i + 1, btn.text, btn.action));
                }
            }
        }

        // Send attachments as URLs for now
        for attachment in &response.attachments {
            text.push_str(&format!("\n[{}]({})", attachment.name, attachment.url));
        }

        // Stub: log the would-be API call
        tracing::info!("[telegram] would send to {}: {}", chat_id, text);
        Ok(())
    }

    fn send_text(&self, chat_id: &str, text: &str) -> Result<(), String> {
        // TODO: POST to self.api_url("sendMessage")
        tracing::info!("[telegram] would send text to {}: {}", chat_id, text);
        Ok(())
    }

    fn supports_cards(&self) -> bool { false } // Telegram uses inline keyboards, not cards
    fn supports_buttons(&self) -> bool { true } // Inline keyboard buttons
    fn supports_attachments(&self) -> bool { true }
}

// ---- Discord Adapter (Stub with structure) ----

/// Discord Bot Gateway Adapter (stub with full structure)
///
/// Supports: embeds (cards), buttons (components), file attachments.
/// Uses Discord REST API or bot gateway.
pub struct DiscordGatewayAdapter {
    bot_token: String,
    client: ureq::Agent,
}

impl DiscordGatewayAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            client: ureq::Agent::new(),
        }
    }

    fn api_url(&self, channel_id: &str, method: &str) -> String {
        format!("https://discord.com/api/v10/channels/{}/{}", channel_id, method)
    }
}

impl BotAdapter for DiscordGatewayAdapter {
    fn platform_name(&self) -> &str {
        "discord"
    }

    fn send_rich_response(&self, chat_id: &str, response: &RichResponse) -> Result<(), String> {
        // TODO: Full Discord implementation
        // Discord supports:
        //   - Embeds (rich cards with title, description, fields, color, image, thumbnail)
        //   - Message Components (ActionRow with Button, SelectMenu)
        //   - File attachments
        //
        // Implementation sketch:
        //   1. Build embed from ResponseCard
        //   2. Build components from CardButtons
        //   3. POST to channel/messages endpoint

        let mut text = response.text.clone();

        // Convert card to text for Discord (until full embed impl)
        if let Some(ref card) = response.card {
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(&format!("**{}**", card.title));
            if let Some(ref sub) = card.subtitle {
                text.push_str(&format!("\n*{}*", sub));
            }
            if let Some(progress) = card.progress {
                text.push_str(&format!("\nProgress: {}%", progress));
            }
            if !card.buttons.is_empty() {
                text.push_str("\n\nOptions:");
                for (i, btn) in card.buttons.iter().enumerate() {
                    let emoji = match btn.style {
                        ButtonStyle::Primary => "\u{1F535}",
                        ButtonStyle::Danger => "\u{1F534}",
                        ButtonStyle::Default => "\u{26AA}",
                    };
                    text.push_str(&format!("\n{} {}. {} → `{}`", emoji, i + 1, btn.text, btn.action));
                }
            }
        }

        // Send attachments as links
        for attachment in &response.attachments {
            text.push_str(&format!("\n[{}]({})", attachment.name, attachment.url));
        }

        // Stub: log the would-be API call
        tracing::info!("[discord] would send to {}: {}", chat_id, text);
        Ok(())
    }

    fn send_text(&self, chat_id: &str, text: &str) -> Result<(), String> {
        // TODO: POST to self.api_url(chat_id, "messages")
        tracing::info!("[discord] would send text to {}: {}", chat_id, text);
        Ok(())
    }

    fn supports_cards(&self) -> bool { true } // Discord embeds
    fn supports_buttons(&self) -> bool { true } // Discord message components
    fn supports_attachments(&self) -> bool { true }
}

// ============================================================================
// 4. GatewayRouter
// ============================================================================

/// Central router for dispatching rich responses to the correct platform adapter.
///
/// Responsibilities:
///   - Register platform adapters
///   - Route responses to the correct adapter
///   - Broadcast to all registered platforms
///   - Normalize responses based on platform capabilities
pub struct GatewayRouter {
    adapters: HashMap<String, Box<dyn BotAdapter>>,
}

impl GatewayRouter {
    /// Create a new empty GatewayRouter
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a platform adapter
    pub fn register(&mut self, platform: &str, adapter: Box<dyn BotAdapter>) {
        self.adapters.insert(platform.to_lowercase(), adapter);
    }

    /// Get a reference to a registered adapter
    pub fn get_adapter(&self, platform: &str) -> Option<&dyn BotAdapter> {
        self.adapters.get(&platform.to_lowercase()).map(|b| b.as_ref())
    }

    /// Check if a platform is registered
    pub fn is_registered(&self, platform: &str) -> bool {
        self.adapters.contains_key(&platform.to_lowercase())
    }

    /// List all registered platform names
    pub fn list_platforms(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }

    /// Send a rich response to a specific platform and chat
    pub fn send(
        &self,
        platform: &str,
        chat_id: &str,
        response: &RichResponse,
    ) -> Result<(), String> {
        let platform = platform.to_lowercase();
        let adapter = self
            .adapters
            .get(&platform)
            .ok_or_else(|| format!("No adapter registered for platform: {}", platform))?;

        let normalized = self.normalize_for_platform(response, &platform);
        adapter.send_rich_response(chat_id, &normalized)
    }

    /// Broadcast a rich response to all registered platforms
    /// Returns a list of (platform_name, result) pairs
    pub fn broadcast(
        &self,
        response: &RichResponse,
    ) -> Vec<(String, Result<(), String>)> {
        // Use a single chat_id placeholder; callers should use per-platform chat IDs
        // For broadcast, we send to each platform's default/first known chat
        self.adapters
            .iter()
            .map(|(platform, adapter)| {
                let normalized = self.normalize_for_platform(response, platform);
                let result = adapter.send_rich_response("broadcast", &normalized);
                (platform.clone(), result)
            })
            .collect()
    }

    /// Send a rich response with per-platform chat IDs
    pub fn broadcast_with_targets(
        &self,
        response: &RichResponse,
        chat_ids: &HashMap<String, String>, // platform -> chat_id
    ) -> Vec<(String, Result<(), String>)> {
        self.adapters
            .iter()
            .filter_map(|(platform, adapter)| {
                chat_ids.get(platform).map(|chat_id| {
                    let normalized = self.normalize_for_platform(response, platform);
                    let result = adapter.send_rich_response(chat_id, &normalized);
                    (platform.clone(), result)
                })
            })
            .collect()
    }

    /// Normalize a rich response for a specific platform's capabilities.
    ///
    /// Transformations:
    /// - If platform doesn't support cards, convert card content to text
    /// - If platform doesn't support buttons, convert to numbered text options
    /// - If platform doesn't support attachments, convert URLs to text links
    pub fn normalize_for_platform(
        &self,
        response: &RichResponse,
        platform: &str,
    ) -> RichResponse {
        let adapter = match self.adapters.get(&platform.to_lowercase()) {
            Some(a) => a.as_ref(),
            None => return response.clone(), // No adapter, return as-is
        };

        let supports_cards = adapter.supports_cards();
        let supports_buttons = adapter.supports_buttons();
        let supports_attachments = adapter.supports_attachments();

        let mut text = response.text.clone();
        let mut card = response.card.clone();
        let mut attachments = response.attachments.clone();

        // If platform doesn't support cards, convert card to text
        if !supports_cards {
            if let Some(ref c) = card {
                if !text.is_empty() {
                    text.push_str("\n\n");
                }
                text.push_str(&format!("**{}**", c.title));

                if let Some(ref subtitle) = c.subtitle {
                    text.push_str(&format!("\n{}", subtitle));
                }

                if let Some(progress) = c.progress {
                    text.push_str(&format!("\nProgress: {}%", progress));
                }

                // Buttons will be handled below
            }
            card = None;
        }

        // If platform doesn't support buttons, convert to numbered text
        if !supports_buttons {
            if let Some(ref c) = card {
                if !c.buttons.is_empty() {
                    text.push_str("\n\nOptions:");
                    for (i, btn) in c.buttons.iter().enumerate() {
                        text.push_str(&format!("\n{}. {} ({})", i + 1, btn.text, btn.action));
                    }
                }
            }

            // Clear buttons from card since they won't render
            if let Some(ref mut c) = card {
                c.buttons.clear();
            }
        }

        // If platform doesn't support attachments, convert to text links
        if !supports_attachments && !attachments.is_empty() {
            for attachment in &attachments {
                text.push_str(&format!("\n[{}]({})", attachment.name, attachment.url));
            }
            attachments.clear();
        }

        RichResponse {
            text,
            card,
            attachments,
        }
    }
}

impl Default for GatewayRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 5. Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Test stub adapter for unit testing ----
    struct TestAdapter {
        name: String,
        cards: bool,
        buttons: bool,
        attachments: bool,
    }

    impl TestAdapter {
        fn full(name: &str) -> Self {
            Self {
                name: name.into(),
                cards: true,
                buttons: true,
                attachments: true,
            }
        }

        fn text_only(name: &str) -> Self {
            Self {
                name: name.into(),
                cards: false,
                buttons: false,
                attachments: false,
            }
        }
    }

    impl BotAdapter for TestAdapter {
        fn platform_name(&self) -> &str {
            &self.name
        }

        fn send_rich_response(&self, _chat_id: &str, _response: &RichResponse) -> Result<(), String> {
            Ok(())
        }

        fn send_text(&self, _chat_id: &str, _text: &str) -> Result<(), String> {
            Ok(())
        }

        fn supports_cards(&self) -> bool { self.cards }
        fn supports_buttons(&self) -> bool { self.buttons }
        fn supports_attachments(&self) -> bool { self.attachments }
    }

    #[test]
    fn test_rich_response_text() {
        let r = RichResponse::text("hello");
        assert_eq!(r.text, "hello");
        assert!(r.card.is_none());
        assert!(r.attachments.is_empty());
    }

    #[test]
    fn test_rich_response_card() {
        let card = ResponseCard {
            title: "Test".into(),
            subtitle: Some("Subtitle".into()),
            progress: Some(50),
            buttons: vec![CardButton {
                text: "OK".into(),
                action: "ok".into(),
                style: ButtonStyle::Primary,
            }],
        };
        let r = RichResponse::card(card.clone());
        assert!(r.card.is_some());
        assert_eq!(r.card.as_ref().unwrap().title, "Test");
    }

    #[test]
    fn test_rich_response_with_attachment() {
        let r = RichResponse::text("check this")
            .with_attachment(Attachment {
                attachment_type: AttachmentType::Image,
                url: "https://example.com/img.png".into(),
                name: "screenshot".into(),
            });
        assert_eq!(r.attachments.len(), 1);
        assert_eq!(r.attachments[0].name, "screenshot");
    }

    #[test]
    fn test_gateway_register_and_send() {
        let mut router = GatewayRouter::new();
        router.register("test", Box::new(TestAdapter::full("test")));

        assert!(router.is_registered("test"));
        assert!(router.is_registered("TEST")); // case-insensitive
        assert!(!router.is_registered("unknown"));

        let response = RichResponse::text("hello");
        let result = router.send("test", "chat-1", &response);
        assert!(result.is_ok());

        let result = router.send("unknown", "chat-1", &response);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_platforms() {
        let mut router = GatewayRouter::new();
        router.register("feishu", Box::new(TestAdapter::full("feishu")));
        router.register("telegram", Box::new(TestAdapter::full("telegram")));

        let platforms = router.list_platforms();
        assert_eq!(platforms.len(), 2);
        assert!(platforms.contains(&"feishu"));
        assert!(platforms.contains(&"telegram"));
    }

    #[test]
    fn test_normalize_no_cards() {
        let mut router = GatewayRouter::new();
        router.register("text-only", Box::new(TestAdapter::text_only("text-only")));

        let card = ResponseCard {
            title: "Card Title".into(),
            subtitle: Some("Card Subtitle".into()),
            progress: Some(75),
            buttons: vec![CardButton {
                text: "Approve".into(),
                action: "approve".into(),
                style: ButtonStyle::Primary,
            }],
        };
        let response = RichResponse::card(card);

        let normalized = router.normalize_for_platform(&response, "text-only");

        // Card should be converted to text
        assert!(normalized.card.is_none());
        assert!(normalized.text.contains("Card Title"));
        assert!(normalized.text.contains("Card Subtitle"));
        assert!(normalized.text.contains("75%"));
        // Buttons should be numbered
        assert!(normalized.text.contains("1. Approve"));
    }

    #[test]
    fn test_normalize_no_buttons() {
        let mut router = GatewayRouter::new();
        let adapter = TestAdapter {
            name: "no-buttons".into(),
            cards: true,
            buttons: false,
            attachments: true,
        };
        router.register("no-buttons", Box::new(adapter));

        let card = ResponseCard {
            title: "Title".into(),
            subtitle: None,
            progress: None,
            buttons: vec![
                CardButton { text: "A".into(), action: "a".into(), style: ButtonStyle::Primary },
                CardButton { text: "B".into(), action: "b".into(), style: ButtonStyle::Default },
            ],
        };
        let response = RichResponse::card(card);

        let normalized = router.normalize_for_platform(&response, "no-buttons");

        // Card is kept (supports cards) but buttons moved to text
        assert!(normalized.card.is_some());
        assert!(normalized.card.as_ref().unwrap().buttons.is_empty());
        assert!(normalized.text.contains("1. A"));
        assert!(normalized.text.contains("2. B"));
    }

    #[test]
    fn test_normalize_no_attachments() {
        let mut router = GatewayRouter::new();
        let adapter = TestAdapter {
            name: "no-attach".into(),
            cards: true,
            buttons: true,
            attachments: false,
        };
        router.register("no-attach", Box::new(adapter));

        let response = RichResponse::text("look at this")
            .with_attachment(Attachment {
                attachment_type: AttachmentType::Image,
                url: "https://example.com/a.png".into(),
                name: "img".into(),
            });

        let normalized = router.normalize_for_platform(&response, "no-attach");

        assert!(normalized.attachments.is_empty());
        assert!(normalized.text.contains("[img](https://example.com/a.png)"));
    }

    #[test]
    fn test_normalize_full_platform_keeps_everything() {
        let mut router = GatewayRouter::new();
        router.register("full", Box::new(TestAdapter::full("full")));

        let card = ResponseCard {
            title: "Full Card".into(),
            subtitle: Some("sub".into()),
            progress: Some(100),
            buttons: vec![CardButton {
                text: "Go".into(),
                action: "go".into(),
                style: ButtonStyle::Primary,
            }],
        };
        let response = RichResponse::card(card)
            .with_attachment(Attachment {
                attachment_type: AttachmentType::Document,
                url: "https://example.com/doc.pdf".into(),
                name: "report".into(),
            });

        let normalized = router.normalize_for_platform(&response, "full");

        assert!(normalized.card.is_some());
        assert_eq!(normalized.card.as_ref().unwrap().buttons.len(), 1);
        assert_eq!(normalized.attachments.len(), 1);
        assert_eq!(normalized.text, "Full Card"); // Original text from card title
    }

    #[test]
    fn test_broadcast() {
        let mut router = GatewayRouter::new();
        router.register("a", Box::new(TestAdapter::full("a")));
        router.register("b", Box::new(TestAdapter::full("b")));

        let results = router.broadcast(&RichResponse::text("hello all"));
        assert_eq!(results.len(), 2);

        for (platform, result) in results {
            assert!(result.is_ok(), "Broadcast to {} failed", platform);
        }
    }

    #[test]
    fn test_broadcast_with_targets() {
        let mut router = GatewayRouter::new();
        router.register("feishu", Box::new(TestAdapter::full("feishu")));
        router.register("telegram", Box::new(TestAdapter::full("telegram")));

        let mut chat_ids = HashMap::new();
        chat_ids.insert("feishu".into(), "feishu-chat-1".into());
        chat_ids.insert("telegram".into(), "telegram-chat-1".into());
        // "discord" not registered, should be skipped

        let results = router.broadcast_with_targets(
            &RichResponse::text("broadcast"),
            &chat_ids,
        );

        assert_eq!(results.len(), 2);
        for (_, result) in &results {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_feishu_adapter_capabilities() {
        let adapter = FeishuGatewayAdapter::new("id".into(), "secret".into());
        assert_eq!(adapter.platform_name(), "feishu");
        assert!(adapter.supports_cards());
        assert!(adapter.supports_buttons());
        assert!(adapter.supports_attachments());
    }

    #[test]
    fn test_telegram_adapter_capabilities() {
        let adapter = TelegramGatewayAdapter::new("token".into());
        assert_eq!(adapter.platform_name(), "telegram");
        assert!(!adapter.supports_cards()); // Uses inline keyboards
        assert!(adapter.supports_buttons());
        assert!(adapter.supports_attachments());
    }

    #[test]
    fn test_discord_adapter_capabilities() {
        let adapter = DiscordGatewayAdapter::new("token".into());
        assert_eq!(adapter.platform_name(), "discord");
        assert!(adapter.supports_cards()); // Embeds
        assert!(adapter.supports_buttons()); // Components
        assert!(adapter.supports_attachments());
    }

    #[test]
    fn test_wechat_adapter_capabilities() {
        let adapter = WechatGatewayAdapter::new();
        assert_eq!(adapter.platform_name(), "wechat");
        assert!(!adapter.supports_cards());
        assert!(!adapter.supports_buttons());
        assert!(adapter.supports_attachments());
    }

    #[test]
    fn test_wecom_adapter_capabilities() {
        let adapter = WecomGatewayAdapter::new("corp".into(), "secret".into(), "1".into());
        assert_eq!(adapter.platform_name(), "wecom");
        assert!(adapter.supports_cards());
        assert!(!adapter.supports_buttons());
        assert!(adapter.supports_attachments());
    }

    #[test]
    fn test_dingtalk_adapter_capabilities() {
        let adapter = DingtalkGatewayAdapter::new("url".into(), None);
        assert_eq!(adapter.platform_name(), "dingtalk");
        assert!(adapter.supports_cards());
        assert!(adapter.supports_buttons());
        assert!(adapter.supports_attachments());
    }

    #[test]
    fn test_rich_response_is_empty() {
        assert!(RichResponse::text("").is_empty());
        assert!(!RichResponse::text("hello").is_empty());

        let card = ResponseCard {
            title: "T".into(),
            subtitle: None,
            progress: None,
            buttons: vec![],
        };
        assert!(!RichResponse::card(card).is_empty());
    }

    #[test]
    fn test_rich_response_with_attachment_builder() {
        let r = RichResponse::text("base")
            .with_attachment(Attachment {
                attachment_type: AttachmentType::Video,
                url: "https://v.com/1.mp4".into(),
                name: "demo".into(),
            })
            .with_attachment(Attachment {
                attachment_type: AttachmentType::Audio,
                url: "https://a.com/1.mp3".into(),
                name: "audio".into(),
            });

        assert_eq!(r.text, "base");
        assert_eq!(r.attachments.len(), 2);
    }
}
