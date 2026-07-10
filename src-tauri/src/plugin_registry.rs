//! Plugin Registry - Unified Plugin System for ACP-UI
//!
//! This is the core of the pluggable architecture. ALL capabilities
//! (Skills, MCP, Hooks, CLI, Bot Adapters) are modeled as plugins
//! registered in this central registry.

#![allow(dead_code)] // PluginMessage reserved for future plugin communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use crate::AppState;

// ---------------------------------------------------------------------------
// Plugin Kinds & Descriptors
// ---------------------------------------------------------------------------

/// Plugin kinds - every capability is one of these
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    /// Reusable agent capabilities
    Skill,
    /// Model Context Protocol servers
    Mcp,
    /// Agent lifecycle interceptors
    Hook,
    /// Command-line extensions
    Cli,
    /// Platform bridges (Telegram, Feishu, etc.)
    Adapter,
}

impl std::fmt::Display for PluginKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginKind::Skill => write!(f, "skill"),
            PluginKind::Mcp => write!(f, "mcp"),
            PluginKind::Hook => write!(f, "hook"),
            PluginKind::Cli => write!(f, "cli"),
            PluginKind::Adapter => write!(f, "adapter"),
        }
    }
}

impl std::str::FromStr for PluginKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "skill" => Ok(PluginKind::Skill),
            "mcp" => Ok(PluginKind::Mcp),
            "hook" => Ok(PluginKind::Hook),
            "cli" => Ok(PluginKind::Cli),
            "adapter" => Ok(PluginKind::Adapter),
            other => Err(format!(
                "Unknown plugin kind: '{}'. Valid: skill, mcp, hook, cli, adapter",
                other
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Capability Descriptor
// ---------------------------------------------------------------------------

/// Capability descriptor - what a plugin can do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub requires_permission: bool,
}

// ---------------------------------------------------------------------------
// Plugin Metadata
// ---------------------------------------------------------------------------

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub id: String,
    pub kind: PluginKind,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub capabilities: Vec<Capability>,
    pub enabled: bool,
    pub registered_at: String,
    pub last_health_check: Option<String>,
    pub health_status: HealthStatus,
    pub config: serde_json::Value,
}

/// Health status of a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin Messages & Responses
// ---------------------------------------------------------------------------

/// Message sent to/from plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    pub plugin_id: String,
    pub capability: String,
    pub payload: serde_json::Value,
    pub context: HashMap<String, String>,
}

/// Response from plugin invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub plugin_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub token_usage: Option<TokenUsage>,
}

/// Token usage tracking for LLM-backed plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
}

// ---------------------------------------------------------------------------
// Execution Records & Stats
// ---------------------------------------------------------------------------

/// Plugin execution record for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionRecord {
    pub id: String,
    pub plugin_id: String,
    pub capability: String,
    pub input: serde_json::Value,
    pub response: PluginResponse,
    pub timestamp: String,
}

/// Aggregated statistics for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    pub total_invocations: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub average_duration_ms: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub success_rate: f64,
}

impl Default for PluginStats {
    fn default() -> Self {
        Self {
            total_invocations: 0,
            success_count: 0,
            failure_count: 0,
            average_duration_ms: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            success_rate: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// The Central Plugin Registry
// ---------------------------------------------------------------------------

/// The central plugin registry - stores all plugins and their execution history
pub struct PluginRegistry {
    plugins: HashMap<String, PluginMeta>,
    execution_history: Vec<PluginExecutionRecord>,
    max_history: usize,
}

impl PluginRegistry {
    /// Create a new empty plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            execution_history: Vec::new(),
            max_history: 10_000,
        }
    }

    /// Register a new plugin. Returns error if a plugin with the same ID already exists.
    pub fn register(&mut self, mut meta: PluginMeta) -> Result<(), String> {
        if meta.id.is_empty() {
            return Err("Plugin ID cannot be empty".to_string());
        }
        if self.plugins.contains_key(&meta.id) {
            return Err(format!(
                "Plugin with ID '{}' is already registered",
                meta.id
            ));
        }

        // Set registration timestamp
        meta.registered_at = chrono::Utc::now().to_rfc3339();
        meta.health_status = HealthStatus::Unknown;

        let id = meta.id.clone();
        self.plugins.insert(id.clone(), meta);
        println!("[PluginRegistry] Registered plugin: {}", id);
        Ok(())
    }

    /// Unregister a plugin by ID. Returns the removed metadata on success.
    pub fn unregister(&mut self, id: &str) -> Result<PluginMeta, String> {
        self.plugins
            .remove(id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))
            .inspect(|meta| {
                println!(
                    "[PluginRegistry] Unregistered plugin: {} ({})",
                    id, meta.name
                );
            })
    }

    /// List all plugins, optionally filtered by kind
    pub fn list(&self, kind: Option<PluginKind>) -> Vec<&PluginMeta> {
        let mut plugins: Vec<&PluginMeta> = match kind {
            Some(k) => self.plugins.values().filter(|p| p.kind == k).collect(),
            None => self.plugins.values().collect(),
        };
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        plugins
    }

    /// Get a specific plugin by ID
    pub fn get(&self, id: &str) -> Option<&PluginMeta> {
        self.plugins.get(id)
    }

    /// Enable or disable a plugin
    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), String> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))?;
        plugin.enabled = enabled;
        println!(
            "[PluginRegistry] Plugin '{}' {} ",
            id,
            if enabled { "enabled" } else { "disabled" }
        );
        Ok(())
    }

    /// Update plugin configuration
    pub fn update_config(&mut self, id: &str, config: serde_json::Value) -> Result<(), String> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))?;
        plugin.config = config;
        println!("[PluginRegistry] Updated config for plugin '{}'", id);
        Ok(())
    }

    /// Update health status of a plugin
    pub fn update_health(&mut self, id: &str, status: HealthStatus) -> Result<(), String> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))?;
        plugin.health_status = status;
        plugin.last_health_check = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Record a plugin invocation in the execution history
    pub fn record_execution(&mut self, record: PluginExecutionRecord) {
        self.execution_history.push(record);

        // Trim oldest records if over max_history
        if self.execution_history.len() > self.max_history {
            let drain_count = self.execution_history.len() - self.max_history;
            self.execution_history.drain(..drain_count);
        }
    }

    /// Get execution history for a plugin, most recent first
    pub fn get_history(&self, plugin_id: &str, limit: usize) -> Vec<&PluginExecutionRecord> {
        self.execution_history
            .iter()
            .rev()
            .filter(|r| r.plugin_id == plugin_id)
            .take(limit)
            .collect()
    }

    /// Compute aggregated statistics for a plugin
    pub fn get_stats(&self, plugin_id: &str) -> PluginStats {
        let records: Vec<&PluginExecutionRecord> = self
            .execution_history
            .iter()
            .filter(|r| r.plugin_id == plugin_id)
            .collect();

        let total = records.len() as u64;
        if total == 0 {
            return PluginStats::default();
        }

        let success_count = records.iter().filter(|r| r.response.success).count() as u64;
        let failure_count = total - success_count;

        let total_duration: u64 = records.iter().map(|r| r.response.duration_ms).sum();
        let avg_duration = total_duration as f64 / total as f64;

        let total_input_tokens: u64 = records
            .iter()
            .filter_map(|r| r.response.token_usage.as_ref())
            .map(|u| u.input_tokens)
            .sum();

        let total_output_tokens: u64 = records
            .iter()
            .filter_map(|r| r.response.token_usage.as_ref())
            .map(|u| u.output_tokens)
            .sum();

        let success_rate = success_count as f64 / total as f64;

        PluginStats {
            total_invocations: total,
            success_count,
            failure_count,
            average_duration_ms: avg_duration,
            total_input_tokens,
            total_output_tokens,
            success_rate,
        }
    }

    /// Search plugins by name or capability name (case-insensitive substring match)
    pub fn search(&self, query: &str) -> Vec<&PluginMeta> {
        let query_lower = query.to_lowercase();
        self.plugins
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.id.to_lowercase().contains(&query_lower)
                    || p.capabilities
                        .iter()
                        .any(|c| c.name.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Discover plugins that provide a specific capability
    pub fn discover_by_capability(&self, capability: &str) -> Vec<&PluginMeta> {
        let cap_lower = capability.to_lowercase();
        self.plugins
            .values()
            .filter(|p| {
                p.enabled
                    && p.capabilities
                        .iter()
                        .any(|c| c.name.to_lowercase() == cap_lower)
            })
            .collect()
    }

    /// Seed the registry with default plugins from the existing skill and MCP systems.
    /// This bridges the old system to the new unified plugin architecture.
    pub fn seed_defaults(&mut self) {
        let now = chrono::Utc::now().to_rfc3339();

        // ---- Seed 16 CORE_SKILLS as Skill plugins ----
        let core_skills: Vec<(&str, &str, &str)> = vec![
            (
                "web-research",
                "Core",
                "Web search and information extraction",
            ),
            ("code-execution", "Core", "Execute and debug code"),
            (
                "file-operations",
                "Core",
                "Read, write, search, patch files",
            ),
            (
                "browser-automation",
                "Core",
                "Browser control and automation",
            ),
            (
                "vision-analysis",
                "Core",
                "Image understanding and analysis",
            ),
            ("media-generation", "Core", "Generate images, video, audio"),
            (
                "skill-management",
                "Management",
                "CRUD and version management",
            ),
            ("memory-ops", "Management", "Memory store and retrieve"),
            ("task-management", "Management", "Todo and task planning"),
            ("communication", "Management", "Messages and clarification"),
            ("delegation", "Management", "Delegate to sub-agents"),
            ("security-audit", "Management", "Security scanning"),
            (
                "code-review",
                "Development",
                "Code review and quality check",
            ),
            ("testing", "Development", "Test generation and execution"),
            ("planning", "Development", "Project planning and breakdown"),
            ("documentation", "Development", "Documentation generation"),
        ];

        for (name, category, description) in core_skills {
            let plugin_id = format!("skill:{}", name);
            if self.plugins.contains_key(&plugin_id) {
                continue;
            }
            let meta = PluginMeta {
                id: plugin_id.clone(),
                kind: PluginKind::Skill,
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: format!("[{}] {}", category, description),
                author: Some("acp-ui".to_string()),
                capabilities: vec![Capability {
                    name: name.to_string(),
                    description: description.to_string(),
                    input_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "parameters": { "type": "object" },
                            "context": { "type": "object" }
                        }
                    }),
                    output_schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "success": { "type": "boolean" },
                            "output": { "type": "string" }
                        }
                    }),
                    requires_permission: false,
                }],
                enabled: true,
                registered_at: now.clone(),
                last_health_check: None,
                health_status: HealthStatus::Healthy,
                config: serde_json::json!({
                    "category": category,
                    "generation": 0
                }),
            };
            self.plugins.insert(plugin_id, meta);
        }

        // ---- Seed MCP plugins (filesystem, git) ----
        let mcp_plugins: Vec<(&str, &str, &str, Vec<&str>)> = vec![
            (
                "mcp:filesystem",
                "filesystem",
                "File system access via MCP",
                vec!["-y", "@modelcontextprotocol/server-filesystem", "."],
            ),
            (
                "mcp:git",
                "git",
                "Git version control via MCP",
                vec!["-y", "@modelcontextprotocol/server-git"],
            ),
        ];

        for (plugin_id, name, description, args) in mcp_plugins {
            if self.plugins.contains_key(plugin_id) {
                continue;
            }
            let meta = PluginMeta {
                id: plugin_id.to_string(),
                kind: PluginKind::Mcp,
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: description.to_string(),
                author: Some("modelcontextprotocol".to_string()),
                capabilities: vec![Capability {
                    name: format!("mcp_{}", name),
                    description: description.to_string(),
                    input_schema: serde_json::json!({ "type": "object" }),
                    output_schema: serde_json::json!({ "type": "object" }),
                    requires_permission: true,
                }],
                enabled: true,
                registered_at: now.clone(),
                last_health_check: None,
                health_status: HealthStatus::Unknown,
                config: serde_json::json!({
                    "command": "npx",
                    "args": args,
                    "transport": "stdio"
                }),
            };
            self.plugins.insert(plugin_id.to_string(), meta);
        }

        println!(
            "[PluginRegistry] Seeded defaults: 16 skills + 2 MCP plugins (total: {})",
            self.plugins.len()
        );
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// List all plugins, optionally filtered by kind string ("skill", "mcp", "hook", "cli", "adapter")
#[tauri::command]
pub fn plugin_list(
    state: State<'_, AppState>,
    kind: Option<String>,
) -> Result<Vec<PluginMeta>, String> {
    let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    let kind_filter = match kind {
        Some(k) => Some(k.parse::<PluginKind>()?),
        None => None,
    };
    Ok(registry.list(kind_filter).into_iter().cloned().collect())
}

/// Register a new plugin
#[tauri::command]
pub fn plugin_register(state: State<'_, AppState>, meta: PluginMeta) -> Result<(), String> {
    let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    registry.register(meta)
}

/// Unregister a plugin by ID, returning the removed metadata
#[tauri::command]
pub fn plugin_unregister(state: State<'_, AppState>, id: String) -> Result<PluginMeta, String> {
    let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    registry.unregister(&id)
}

/// Get a specific plugin by ID
#[tauri::command]
pub fn plugin_get(state: State<'_, AppState>, id: String) -> Result<PluginMeta, String> {
    let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    registry
        .get(&id)
        .cloned()
        .ok_or_else(|| format!("Plugin '{}' not found", id))
}

/// Enable or disable a plugin
#[tauri::command]
pub fn plugin_set_enabled(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    registry.set_enabled(&id, enabled)
}

/// Update a plugin's configuration
#[tauri::command]
pub fn plugin_update_config(
    state: State<'_, AppState>,
    id: String,
    config: serde_json::Value,
) -> Result<(), String> {
    let mut registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    registry.update_config(&id, config)
}

/// Search plugins by name or capability
#[tauri::command]
pub fn plugin_search(state: State<'_, AppState>, query: String) -> Result<Vec<PluginMeta>, String> {
    let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    Ok(registry.search(&query).into_iter().cloned().collect())
}

/// Get aggregated execution statistics for a plugin
#[tauri::command]
pub fn plugin_get_stats(state: State<'_, AppState>, id: String) -> Result<PluginStats, String> {
    let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    if registry.get(&id).is_none() {
        return Err(format!("Plugin '{}' not found", id));
    }
    Ok(registry.get_stats(&id))
}

/// Get execution history for a plugin
#[tauri::command]
pub fn plugin_get_history(
    state: State<'_, AppState>,
    id: String,
    limit: Option<usize>,
) -> Result<Vec<PluginExecutionRecord>, String> {
    let registry = state.plugin_registry.lock().map_err(|e| e.to_string())?;
    if registry.get(&id).is_none() {
        return Err(format!("Plugin '{}' not found", id));
    }
    let limit = limit.unwrap_or(50);
    Ok(registry
        .get_history(&id, limit)
        .into_iter()
        .cloned()
        .collect())
}
