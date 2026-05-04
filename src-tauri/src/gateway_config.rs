use crate::database::DatabaseManager;

use crate::{AppConfig, GatewayConfig, TunnelConfig};

const CONFIG_KEY: &str = "gateway_config";

/// Load gateway config from database, falling back to defaults
pub fn load_gateway_config(db: &DatabaseManager) -> Result<GatewayConfig, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT config_json FROM gateway_config WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map(rusqlite::params![CONFIG_KEY], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;

    if let Some(raw) = rows.next().transpose().map_err(|e| e.to_string())? {
        serde_json::from_str(&raw).map_err(|e| format!("Failed to parse gateway config: {}", e))
    } else {
        Ok(default_config())
    }
}

/// Save gateway config to database
pub fn save_gateway_config(db: &DatabaseManager, config: &GatewayConfig) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    let json =
        serde_json::to_string(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO gateway_config (id, config_json, updated_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![CONFIG_KEY, json, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Return a config with sensitive fields masked (for returning to the UI)
pub fn mask_config(config: &GatewayConfig) -> GatewayConfig {
    let mut masked = config.clone();

    // Mask feishu secrets
    if let Some(ref mut f) = masked.feishu {
        if !f.app_secret.is_empty() {
            f.app_secret = "****".to_string();
        }
        if let Some(ref mut ek) = f.encrypt_key {
            if !ek.is_empty() {
                *ek = "****".to_string();
            }
        }
    }

    // Mask telegram bot token
    if let Some(ref mut t) = masked.telegram {
        if !t.bot_token.is_empty() {
            t.bot_token = "****".to_string();
        }
    }

    // Mask discord bot token
    if let Some(ref mut d) = masked.discord {
        if !d.bot_token.is_empty() {
            d.bot_token = "****".to_string();
        }
    }

    masked
}

/// Check if a token matches a stored one (constant-time comparison)
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Default config used when no saved config exists
pub fn default_config() -> GatewayConfig {
    GatewayConfig {
        feishu: None,
        telegram: None,
        discord: None,
        app: Some(AppConfig {
            websocket_port: 1420,
            auth_mode: "qrcode".to_string(),
            enabled: true,
        }),
        tunnel: Some(TunnelConfig {
            enabled: false,
            provider: "ngrok".to_string(),
            ngrok_token: None,
            ngrok_region: Some("ap".to_string()),
            custom_url: None,
            status: "stopped".to_string(),
            public_url: None,
        }),
    }
}
