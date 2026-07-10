use crate::bot_adapters::{self, BotAdapter};
use crate::tunnel::NgrokManager;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

// ===== Gateway Config Structs =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConfig {
    pub feishu: Option<FeishuConfig>,
    pub telegram: Option<TelegramConfig>,
    pub discord: Option<DiscordConfig>,
    pub app: Option<AppConfig>,
    pub tunnel: Option<TunnelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeishuConfig {
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub app_secret: String,
    pub encrypt_key: Option<String>,
    pub verification_token: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramConfig {
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    #[serde(default)]
    pub bot_token: String,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub websocket_port: u16,
    pub auth_mode: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelConfig {
    pub enabled: bool,
    pub provider: String,
    pub ngrok_token: Option<String>,
    pub ngrok_region: Option<String>,
    pub custom_url: Option<String>,
    pub status: String,
    pub public_url: Option<String>,
}

// ===== Gateway Commands =====

#[tauri::command]
pub fn get_gateway_config(state: State<AppState>) -> Result<GatewayConfig, String> {
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let config = crate::gateway_config::load_gateway_config(db)?;
    Ok(crate::gateway_config::mask_config(&config))
}

#[tauri::command]
pub fn save_gateway_config(config: GatewayConfig, state: State<AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|e| e.to_string())?;
    let db = db
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    crate::gateway_config::save_gateway_config(db, &config)?;
    Ok(())
}

#[tauri::command]
pub async fn start_gateway(
    config: GatewayConfig,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!("Starting gateway with config");

    // Persist config to database - release lock before async operations
    {
        let db = state.database.lock().map_err(|e| e.to_string())?;
        let db = db
            .as_ref()
            .ok_or_else(|| "Database not initialized".to_string())?;
        crate::gateway_config::save_gateway_config(db, &config)?;
    }

    // Generate auth token for app gateway
    if let Some(app_config) = &config.app {
        if app_config.enabled {
            let token = Uuid::new_v4().to_string();
            // Store token in ws_server so the WebSocket server can use it
            let ws = state.ws_server.lock().unwrap();
            if let Some(server) = ws.as_ref() {
                server.set_auth_token(Some(token.clone()));
                println!("Gateway auth token generated");
            }
        }
    }

    // Start Feishu Bot if enabled
    if let Some(feishu) = &config.feishu {
        if feishu.enabled {
            let mut adapter = bot_adapters::FeishuAdapter::new(feishu);
            adapter.start(&app_handle).await?;
            *state.feishu_adapter.lock().unwrap() = Some(adapter);
            println!("Feishu Bot started, App ID: {}", feishu.app_id);
        }
    }

    // Start Telegram Bot if enabled
    if let Some(telegram) = &config.telegram {
        if telegram.enabled {
            let mut adapter = bot_adapters::TelegramAdapter::new(telegram);
            adapter.start(&app_handle).await?;
            *state.telegram_adapter.lock().unwrap() = Some(adapter);
            println!("Telegram Bot started");
        }
    }

    // Discord Bot — TODO(Phase 4): implement adapter
    if let Some(discord) = &config.discord {
        if discord.enabled {
            println!("Discord Bot enabled, Guild: {:?}", discord.guild_id);
        }
    }

    let _ = app_handle.emit("gateway-started", config);

    Ok(())
}

#[tauri::command]
pub async fn stop_gateway(app_handle: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    println!("Stopping gateway");

    // 1. Stop WebSocket server
    {
        let ws = state.ws_server.lock().unwrap();
        if let Some(server) = ws.as_ref() {
            server.stop();
            println!("WebSocket server stopped");
        }
    }

    // 2. Stop Feishu Bot - release lock before await
    let feishu_adapter = {
        let mut feishu = state.feishu_adapter.lock().unwrap();
        feishu.take()
    };
    if let Some(mut adapter) = feishu_adapter {
        adapter.stop().await?;
        println!("Feishu Bot stopped");
    }

    // 3. Stop Telegram Bot - release lock before await
    let telegram_adapter = {
        let mut telegram = state.telegram_adapter.lock().unwrap();
        telegram.take()
    };
    if let Some(mut adapter) = telegram_adapter {
        adapter.stop().await?;
        println!("Telegram Bot stopped");
    }

    // 4. Discord Bot — TODO(Phase 4): implement adapter

    // Emit gateway stopped event
    let _ = app_handle.emit("gateway-stopped", ());

    Ok(())
}

#[tauri::command]
pub async fn start_tunnel(
    provider: String,
    token: Option<String>,
    port: u16,
    region: Option<String>,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    println!(
        "Starting tunnel with provider: {}, port: {}",
        provider, port
    );

    match provider.as_str() {
        "ngrok" => {
            // ngrok needs token, if not available return error
            if token.is_none() || token.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                return Err("ngrok token required. Get it from https://ngrok.com".to_string());
            }

            let ngrok_token = token.unwrap();
            let ngrok_region = region.unwrap_or_else(|| "ap".to_string());

            // Create or get NgrokManager
            let manager = {
                let mut tm = state.tunnel_manager.lock().unwrap();
                if tm.is_none() {
                    *tm = Some(NgrokManager::new());
                }
                tm.clone().unwrap()
            };

            // Start ngrok process
            manager.start(&ngrok_token, port, &ngrok_region, app_handle)
        }
        "frp" => Err(
            "frp requires custom server configuration. Please set custom URL in config."
                .to_string(),
        ),
        "cloudflare" => {
            // Cloudflare tunnel using cloudflared CLI
            Err("Cloudflare tunnel requires cloudflared CLI. Run: cloudflared tunnel --url http://localhost:PORT".to_string())
        }
        _ => Err(format!("Unknown tunnel provider: {}", provider)),
    }
}

#[tauri::command]
pub fn stop_tunnel(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    println!("Stopping tunnel");

    let manager_guard = state.tunnel_manager.lock().unwrap();

    if let Some(manager) = manager_guard.as_ref() {
        manager.stop(app_handle)?;
    }

    Ok(())
}
