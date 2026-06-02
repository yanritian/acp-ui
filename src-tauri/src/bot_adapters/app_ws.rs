//! App WebSocket Adapter
//!
//! Handles bot commands via the Gateway WebSocket connection.
//! Mobile apps and external tools connect through WebSocket and send commands.

use crate::bot_adapters::{BotAdapter, BotCommand, BotResponse, parse_bot_text};
use crate::AppState;
use tauri::{AppHandle, Emitter, Manager};

/// App WebSocket Bot Adapter
pub struct AppWsAdapter {
    running: bool,
}

impl AppWsAdapter {
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl BotAdapter for AppWsAdapter {
    fn platform(&self) -> &str {
        "app"
    }

    fn is_running(&self) -> bool {
        self.running
    }

    async fn start(&mut self, _app_handle: &AppHandle) -> Result<(), String> {
        // WebSocket server is started separately by Gateway
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.running = false;
        Ok(())
    }

    fn parse_command(&self, text: &str) -> Option<BotCommand> {
        let cmd = parse_bot_text(text);
        // Return None for Unknown commands
        match cmd {
            BotCommand::Unknown { .. } => None,
            _ => Some(cmd),
        }
    }

    async fn handle_command(&self, command: BotCommand, app_handle: &AppHandle) -> BotResponse {
        let state = app_handle.state::<AppState>();

        match command {
            BotCommand::Status => {
                // Get real status from database
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

                let agent_count = {
                    let config_manager = state.config_manager.read();
                    config_manager.as_ref()
                        .map(|cm| cm.get_config().agents.len())
                        .unwrap_or(0)
                };

                let ws_running = {
                    let ws_guard = state.ws_server.lock().ok();
                    ws_guard.and_then(|g| g.as_ref().map(|w| w.is_running())).unwrap_or(false)
                };

                BotResponse {
                    success: true,
                    message: "系统状态".to_string(),
                    data: Some(serde_json::json!({
                        "agents_configured": agent_count,
                        "running_tasks": running_tasks,
                        "total_tasks": total_tasks,
                        "ws_server_running": ws_running,
                    })),
                }
            },

            BotCommand::ListAgents => {
                let config_manager = state.config_manager.read();
                let agents = config_manager.as_ref()
                    .map(|cm| {
                        cm.get_config().agents.keys().cloned().collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                BotResponse {
                    success: true,
                    message: format!("已配置 {} 个 Agent", agents.len()),
                    data: Some(serde_json::json!({ "agents": agents })),
                }
            },

            BotCommand::Agent { prompt, agent_name } => {
                // Emit event to frontend to create task
                let _ = app_handle.emit("bot-command", serde_json::json!({
                    "type": "agent",
                    "prompt": prompt,
                    "agent_name": agent_name,
                }));

                BotResponse {
                    success: true,
                    message: "任务已创建，正在执行...".to_string(),
                    data: Some(serde_json::json!({ "prompt": prompt })),
                }
            },

            BotCommand::Team { prompt, agents, routing } => {
                let _ = app_handle.emit("bot-command", serde_json::json!({
                    "type": "team",
                    "prompt": prompt,
                    "agents": agents,
                    "routing": routing,
                }));

                BotResponse {
                    success: true,
                    message: "多 Agent 任务已创建...".to_string(),
                    data: Some(serde_json::json!({ "prompt": prompt, "agents": agents })),
                }
            },

            BotCommand::Pause { agent_id } => {
                let _ = app_handle.emit("bot-command", serde_json::json!({
                    "type": "pause",
                    "agent_id": agent_id,
                }));

                BotResponse {
                    success: true,
                    message: format!("Agent {} 已暂停", agent_id),
                    data: None,
                }
            },

            BotCommand::Resume { agent_id } => {
                let _ = app_handle.emit("bot-command", serde_json::json!({
                    "type": "resume",
                    "agent_id": agent_id,
                }));

                BotResponse {
                    success: true,
                    message: format!("Agent {} 已恢复", agent_id),
                    data: None,
                }
            },

            BotCommand::Cancel { target_id } => {
                let _ = app_handle.emit("bot-command", serde_json::json!({
                    "type": "cancel",
                    "target_id": target_id,
                }));

                BotResponse {
                    success: true,
                    message: format!("任务 {} 已取消", target_id),
                    data: None,
                }
            },

            BotCommand::History { limit } => {
                // Get statistics from database (history not available yet)
                let db_guard = state.database.lock().ok();
                let stats = if let Some(guard) = db_guard {
                    if let Some(database) = guard.as_ref() {
                        database.get_statistics().ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                let _limit = limit.unwrap_or(10);

                BotResponse {
                    success: true,
                    message: "获取任务统计".to_string(),
                    data: Some(serde_json::json!({
                        "statistics": stats,
                        "total_tasks": stats.as_ref().map(|s| s.total_tasks).unwrap_or(0),
                    })),
                }
            },

            BotCommand::Unknown { raw } => {
                BotResponse {
                    success: false,
                    message: format!("未知命令: {}", raw),
                    data: None,
                }
            },
        }
    }
}

impl Default for AppWsAdapter {
    fn default() -> Self {
        Self::new()
    }
}