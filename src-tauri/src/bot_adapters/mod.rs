//! Bot Adapters Module
//!
//! Provides unified interface for multiple bot platforms:
//! - App WebSocket (mobile/desktop control)
//! - Telegram Bot
//! - Feishu Bot
//! - Discord Bot

#![allow(dead_code)]

mod app_ws;
mod telegram;
mod feishu;

pub use app_ws::AppWsAdapter;
pub use telegram::TelegramAdapter;
pub use feishu::FeishuAdapter;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// Bot response to user command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Bot adapter trait - unified interface for all platforms
pub trait BotAdapter {
    /// Platform name
    fn platform(&self) -> &str;

    /// Whether the adapter is running
    fn is_running(&self) -> bool;

    /// Start the adapter
    async fn start(&mut self, app_handle: &AppHandle) -> Result<(), String>;

    /// Stop the adapter
    async fn stop(&mut self) -> Result<(), String>;

    /// Parse text command from user
    fn parse_command(&self, text: &str) -> Option<BotCommand>;

    /// Handle parsed command and return response
    async fn handle_command(&self, command: BotCommand, app_handle: &AppHandle) -> BotResponse;
}

/// Parsed bot command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BotCommand {
    /// Get system status
    Status,

    /// List configured agents
    ListAgents,

    /// Create a task with single agent
    Agent {
        prompt: String,
        agent_name: Option<String>,
    },

    /// Create a multi-agent task
    Team {
        prompt: String,
        agents: Vec<String>,
        routing: String,
    },

    /// Pause an agent
    Pause {
        agent_id: String,
    },

    /// Resume an agent
    Resume {
        agent_id: String,
    },

    /// Cancel an agent/task
    Cancel {
        target_id: String,
    },

    /// Get history
    History {
        limit: Option<u64>,
    },

    /// Unknown/unrecognized command
    Unknown {
        raw: String,
    },
}

/// Parse bot text into BotCommand
pub fn parse_bot_text(text: &str) -> BotCommand {
    let text = text.trim();

    // Remove leading slash if present
    let text = text.strip_prefix('/').unwrap_or(text);

    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.is_empty() {
        return BotCommand::Unknown { raw: text.to_string() };
    }

    match parts[0].to_lowercase().as_str() {
        "status" => BotCommand::Status,
        "agents" | "list" => BotCommand::ListAgents,
        "agent" => {
            // /agent <prompt> or /agent <agent_name> <prompt>
            if parts.len() < 2 {
                return BotCommand::Unknown { raw: text.to_string() };
            }

            // Check if second part is an agent name (contains no spaces in remaining)
            let remaining = parts[1..].join(" ");
            BotCommand::Agent {
                prompt: remaining,
                agent_name: None,
            }
        },
        "team" => {
            // /team <agents> <prompt> or /team broadcast <prompt>
            if parts.len() < 3 {
                return BotCommand::Unknown { raw: text.to_string() };
            }

            let routing = if parts[1] == "broadcast" {
                "broadcast"
            } else {
                "single"
            };

            let agents: Vec<String> = if routing == "broadcast" {
                // /team broadcast <prompt>
                Vec::new()
            } else {
                // /team codex,claude-code <prompt>
                parts[1].split(',').map(|s| s.trim().to_string()).collect()
            };

            let prompt = parts[2..].join(" ");

            BotCommand::Team {
                prompt,
                agents,
                routing: routing.to_string(),
            }
        },
        "pause" => {
            if parts.len() < 2 {
                return BotCommand::Unknown { raw: text.to_string() };
            }
            BotCommand::Pause {
                agent_id: parts[1].to_string(),
            }
        },
        "resume" => {
            if parts.len() < 2 {
                return BotCommand::Unknown { raw: text.to_string() };
            }
            BotCommand::Resume {
                agent_id: parts[1].to_string(),
            }
        },
        "cancel" => {
            if parts.len() < 2 {
                return BotCommand::Unknown { raw: text.to_string() };
            }
            BotCommand::Cancel {
                target_id: parts[1].to_string(),
            }
        },
        "history" => {
            let limit = if parts.len() > 1 {
                parts[1].parse::<u64>().ok()
            } else {
                Some(10)
            };
            BotCommand::History { limit }
        },
        _ => BotCommand::Unknown { raw: text.to_string() },
    }
}

/// Format bot response as platform-specific message
pub fn format_response(response: &BotResponse, platform: &str) -> String {
    match platform {
        "telegram" => format_telegram_response(response),
        "feishu" => format_feishu_response(response),
        "app" => format_app_response(response),
        _ => response.message.clone(),
    }
}

fn format_telegram_response(response: &BotResponse) -> String {
    let icon = if response.success { "✅" } else { "❌" };
    let mut msg = format!("{} {}", icon, response.message);

    if let Some(data) = &response.data {
        msg.push_str("\n\n");
        msg.push_str(&serde_json::to_string_pretty(data).unwrap_or_default());
    }

    msg
}

fn format_feishu_response(response: &BotResponse) -> String {
    let icon = if response.success { "✅" } else { "❌" };
    format!("{} {}", icon, response.message)
}

fn format_app_response(response: &BotResponse) -> String {
    response.message.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_command() {
        let cmd = parse_bot_text("/status");
        assert!(matches!(cmd, BotCommand::Status));

        let cmd = parse_bot_text("status");
        assert!(matches!(cmd, BotCommand::Status));

        let cmd = parse_bot_text("STATUS");
        assert!(matches!(cmd, BotCommand::Status));
    }

    #[test]
    fn test_parse_agents_command() {
        let cmd = parse_bot_text("/agents");
        assert!(matches!(cmd, BotCommand::ListAgents));

        let cmd = parse_bot_text("list");
        assert!(matches!(cmd, BotCommand::ListAgents));
    }

    #[test]
    fn test_parse_agent_command() {
        let cmd = parse_bot_text("/agent write hello world");
        match cmd {
            BotCommand::Agent { prompt, agent_name } => {
                assert_eq!(prompt, "write hello world");
                assert!(agent_name.is_none());
            },
            _ => panic!("Expected Agent command"),
        }

        // Missing prompt
        let cmd = parse_bot_text("/agent");
        assert!(matches!(cmd, BotCommand::Unknown { .. }));
    }

    #[test]
    fn test_parse_team_command() {
        let cmd = parse_bot_text("/team codex,claude-code review this code");
        match cmd {
            BotCommand::Team { prompt, agents, routing } => {
                assert_eq!(prompt, "review this code");
                assert_eq!(agents, vec!["codex", "claude-code"]);
                assert_eq!(routing, "single");
            },
            _ => panic!("Expected Team command"),
        }

        let cmd = parse_bot_text("/team broadcast analyze the system");
        match cmd {
            BotCommand::Team { prompt, agents, routing } => {
                assert_eq!(prompt, "analyze the system");
                assert!(agents.is_empty());
                assert_eq!(routing, "broadcast");
            },
            _ => panic!("Expected Team command with broadcast"),
        }
    }

    #[test]
    fn test_parse_pause_resume_cancel() {
        let cmd = parse_bot_text("/pause agent-123");
        match cmd {
            BotCommand::Pause { agent_id } => assert_eq!(agent_id, "agent-123"),
            _ => panic!("Expected Pause command"),
        }

        let cmd = parse_bot_text("/resume agent-456");
        match cmd {
            BotCommand::Resume { agent_id } => assert_eq!(agent_id, "agent-456"),
            _ => panic!("Expected Resume command"),
        }

        let cmd = parse_bot_text("/cancel task-789");
        match cmd {
            BotCommand::Cancel { target_id } => assert_eq!(target_id, "task-789"),
            _ => panic!("Expected Cancel command"),
        }

        // Missing ID
        let cmd = parse_bot_text("/pause");
        assert!(matches!(cmd, BotCommand::Unknown { .. }));
    }

    #[test]
    fn test_parse_history_command() {
        let cmd = parse_bot_text("/history");
        match cmd {
            BotCommand::History { limit } => assert_eq!(limit, Some(10)),
            _ => panic!("Expected History command"),
        }

        let cmd = parse_bot_text("/history 5");
        match cmd {
            BotCommand::History { limit } => assert_eq!(limit, Some(5)),
            _ => panic!("Expected History command with limit"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        let cmd = parse_bot_text("/foobar");
        match cmd {
            BotCommand::Unknown { raw } => assert_eq!(raw, "foobar"),
            _ => panic!("Expected Unknown command"),
        }

        let cmd = parse_bot_text("");
        assert!(matches!(cmd, BotCommand::Unknown { .. }));
    }

    #[test]
    fn test_format_response_telegram() {
        let response = BotResponse {
            success: true,
            message: "系统运行正常".to_string(),
            data: Some(serde_json::json!({ "agents": 3 })),
        };

        let formatted = format_response(&response, "telegram");
        assert!(formatted.starts_with("✅"));
        assert!(formatted.contains("系统运行正常"));
        assert!(formatted.contains("agents"));
    }

    #[test]
    fn test_format_response_feishu() {
        let response = BotResponse {
            success: false,
            message: "执行失败".to_string(),
            data: None,
        };

        let formatted = format_response(&response, "feishu");
        assert!(formatted.starts_with("❌"));
        assert!(formatted.contains("执行失败"));
    }

    #[test]
    fn test_format_response_app() {
        let response = BotResponse {
            success: true,
            message: "任务完成".to_string(),
            data: None,
        };

        let formatted = format_response(&response, "app");
        assert_eq!(formatted, "任务完成");
    }
}