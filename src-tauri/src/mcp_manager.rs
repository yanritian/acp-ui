//! MCP Process Manager Module
//!
//! Manages MCP server processes (Model Context Protocol).
//! NOTE: Module is imported but commands not yet registered in lib.rs - marked for future use.

#![allow(dead_code)] // Reserved for future MCP integration

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

#[cfg(all(desktop, target_os = "windows"))]
use std::os::windows::process::CommandExt;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub transport: McpTransport,
    pub url: Option<String>,
}

/// MCP transport type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    #[default]
    Stdio,
    WebSocket,
    Http,
}

/// Running MCP server instance
struct McpInstance {
    #[allow(dead_code)]
    process: Option<Child>,
    name: String,
    agent_id: String,
}

/// MCP Process Manager - manages MCP server instances per agent
pub struct McpProcessManager {
    instances: Arc<RwLock<HashMap<String, McpInstance>>>,
    configs: Arc<RwLock<HashMap<String, McpServerConfig>>>,
}

impl McpProcessManager {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an MCP server configuration
    pub fn register_config(&self, config: McpServerConfig) {
        self.configs.write().insert(config.name.clone(), config);
    }

    /// Start an MCP server for a specific agent
    pub fn start_mcp(&self, agent_id: &str, mcp_name: &str) -> Result<String, String> {
        // Check if already running for this agent
        let instance_key = format!("{}:{}", agent_id, mcp_name);
        if self.instances.read().contains_key(&instance_key) {
            return Err("MCP already running for this agent".to_string());
        }

        // Get config
        let config = self.configs.read()
            .get(mcp_name)
            .cloned()
            .ok_or_else(|| format!("MCP config '{}' not found", mcp_name))?;

        let _instance_id = uuid::Uuid::new_v4().to_string();

        match config.transport {
            McpTransport::Stdio => {
                // Start stdio MCP process
                #[cfg(desktop)]
                {
                    let mut cmd = Command::new(&config.command);
                    if let Some(args) = &config.args {
                        cmd.args(args);
                    }
                    if let Some(env) = &config.env {
                        cmd.envs(env);
                    }
                    cmd.stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

                    #[cfg(target_os = "windows")]
                    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

                    let child = cmd.spawn()
                        .map_err(|e| format!("Failed to start MCP '{}': {}", mcp_name, e))?;

                    let instance = McpInstance {
                        process: Some(child),
                        name: mcp_name.to_string(),
                        agent_id: agent_id.to_string(),
                    };

                    self.instances.write().insert(instance_key.clone(), instance);
                    println!("Started MCP '{}' for agent '{}'", mcp_name, agent_id);
                }

                #[cfg(not(desktop))]
                {
                    return Err("stdio MCP servers not supported on mobile".to_string());
                }
            }
            McpTransport::WebSocket | McpTransport::Http => {
                // For WebSocket/HTTP MCP, we just register the connection URL
                // The actual connection will be managed by the agent session
                let instance = McpInstance {
                    process: None,
                    name: mcp_name.to_string(),
                    agent_id: agent_id.to_string(),
                };

                self.instances.write().insert(instance_key.clone(), instance);
                println!("Registered MCP '{}' for agent '{}' (remote transport)", mcp_name, agent_id);
            }
        }

        Ok(instance_key)
    }

    /// Stop an MCP server for a specific agent
    pub fn stop_mcp(&self, agent_id: &str, mcp_name: &str) -> Result<(), String> {
        let instance_key = format!("{}:{}", agent_id, mcp_name);

        let mut instances = self.instances.write();
        if let Some(mut instance) = instances.remove(&instance_key) {
            if let Some(mut process) = instance.process.take() {
                process.kill()
                    .map_err(|e| format!("Failed to kill MCP process: {}", e))?;
                let _ = process.wait();
            }
            println!("Stopped MCP '{}' for agent '{}'", mcp_name, agent_id);
        }

        Ok(())
    }

    /// Stop all MCP servers for an agent
    pub fn stop_all_for_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut instances = self.instances.write();
        let keys_to_remove: Vec<String> = instances.keys()
            .filter(|k| k.starts_with(&format!("{}:", agent_id)))
            .cloned()
            .collect();

        for key in keys_to_remove {
            if let Some(mut instance) = instances.remove(&key) {
                if let Some(mut process) = instance.process.take() {
                    process.kill().ok();
                    let _ = process.wait();
                }
            }
        }

        println!("Stopped all MCPs for agent '{}'", agent_id);
        Ok(())
    }

    /// Get list of MCP servers running for an agent
    pub fn get_agent_mcps(&self, agent_id: &str) -> Vec<String> {
        self.instances.read()
            .keys()
            .filter(|k| k.starts_with(&format!("{}:", agent_id)))
            .map(|k| k.split(':').nth(1).unwrap_or("").to_string())
            .collect()
    }

    /// Check if MCP is running for an agent
    pub fn is_running(&self, agent_id: &str, mcp_name: &str) -> bool {
        let instance_key = format!("{}:{}", agent_id, mcp_name);
        self.instances.read().contains_key(&instance_key)
    }

    /// Get MCP server URL for remote transport
    pub fn get_mcp_url(&self, mcp_name: &str) -> Option<String> {
        self.configs.read().get(mcp_name).and_then(|c| c.url.clone())
    }

    /// List all registered MCP configs
    pub fn list_configs(&self) -> Vec<McpServerConfig> {
        self.configs.read().values().cloned().collect()
    }

    /// Validate MCP is accessible by agent (isolation check)
    pub fn validate_access(&self, agent_id: &str, mcp_name: &str) -> Result<(), String> {
        let instance_key = format!("{}:{}", agent_id, mcp_name);

        // Check if MCP is running for this specific agent
        if !self.instances.read().contains_key(&instance_key) {
            return Err(format!("Agent '{}' cannot access MCP '{}' - not running for this agent", agent_id, mcp_name));
        }

        Ok(())
    }
}

impl Default for McpProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default MCP server configurations
pub fn get_default_mcp_configs() -> Vec<McpServerConfig> {
    vec![
        McpServerConfig {
            name: "filesystem".to_string(),
            command: "npx".to_string(),
            args: Some(vec!["@anthropic/mcp-server-filesystem".to_string()]),
            env: None,
            transport: McpTransport::Stdio,
            url: None,
        },
        McpServerConfig {
            name: "git".to_string(),
            command: "npx".to_string(),
            args: Some(vec!["@anthropic/mcp-server-git".to_string()]),
            env: None,
            transport: McpTransport::Stdio,
            url: None,
        },
    ]
}