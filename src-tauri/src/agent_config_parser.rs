use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::permission_checker::PermissionConfig;

/// Agent configuration parsed from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfigParsed {
    pub name: String,
    pub cwd: Option<String>,
    pub env: HashMap<String, String>,
    pub permissions: PermissionConfig,
    pub mcp_servers: Vec<String>,
    pub skills: Vec<String>,
    pub hooks: Vec<String>,
    pub capabilities: Vec<String>,
    pub transport: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Validation error for agent config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Result of config parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub config: Option<AgentConfigParsed>,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Agent configuration parser and validator
pub struct AgentConfigParser {
    required_fields: Vec<String>,
    valid_transports: Vec<String>,
}

impl AgentConfigParser {
    pub fn new() -> Self {
        Self {
            required_fields: vec!["name".to_string(), "transport".to_string()],
            valid_transports: vec!["stdio".to_string(), "websocket".to_string(), "http".to_string()],
        }
    }

    /// Parse YAML content into AgentConfig (using serde_yaml for proper nested structure support)
    pub fn parse_yaml(&self, yaml_content: &str) -> ParseResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Parse YAML using serde_yaml (proper nested structure support)
        let config: Option<AgentConfigParsed> = match serde_yaml::from_str(yaml_content) {
            Ok(c) => Some(c),
            Err(e) => {
                // Try fallback simple parser for legacy configs
                match self.parse_simple_yaml_fallback(yaml_content) {
                    Ok(c) => {
                        warnings.push(format!("Used fallback parser (serde_yaml failed: {})", e));
                        Some(c)
                    },
                    Err(fallback_err) => {
                        errors.push(ValidationError {
                            field: "yaml".to_string(),
                            message: format!("Failed to parse YAML: {} (fallback also failed: {})", e, fallback_err),
                        });
                        None
                    }
                }
            }
        };

        let config = match config {
            Some(c) => c,
            None => return ParseResult { config: None, errors, warnings },
        };

        // Validate required fields
        for field in &self.required_fields {
            if field == "name" && config.name.is_empty() {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    message: "Agent name is required".to_string(),
                });
            }
            if field == "transport" && !self.valid_transports.contains(&config.transport) {
                errors.push(ValidationError {
                    field: "transport".to_string(),
                    message: format!("Invalid transport '{}'. Valid options: {}", config.transport, self.valid_transports.join(", ")),
                });
            }
        }

        // Validate transport-specific requirements
        if config.transport == "stdio" {
            if config.command.is_none() || config.command.as_ref().map(|c| c.is_empty()).unwrap_or(true) {
                errors.push(ValidationError {
                    field: "command".to_string(),
                    message: "stdio transport requires 'command' field".to_string(),
                });
            }
        } else if (config.transport == "websocket" || config.transport == "http")
            && (config.url.is_none() || config.url.as_ref().map(|u| u.is_empty()).unwrap_or(true)) {
                errors.push(ValidationError {
                    field: "url".to_string(),
                    message: format!("{} transport requires 'url' field", config.transport),
                });
        }

        // Validate working directory if specified
        if let Some(cwd) = &config.cwd {
            if !Path::new(cwd).exists() {
                warnings.push(format!("Working directory '{}' does not exist", cwd));
            }
        }

        // Validate MCP servers exist
        for mcp in &config.mcp_servers {
            // TODO(Phase 4): Validate MCP server config at parse time
            warnings.push(format!("MCP server '{}' will be validated at runtime", mcp));
        }

        // Validate skills exist
        for skill in &config.skills {
            // TODO(Phase 4): Validate skill registration at parse time
            warnings.push(format!("Skill '{}' will be validated at runtime", skill));
        }

        if errors.is_empty() {
            ParseResult { config: Some(config), errors, warnings }
        } else {
            ParseResult { config: None, errors, warnings }
        }
    }

    /// Fallback simple YAML parser (for legacy flat configs)
    fn parse_simple_yaml_fallback(&self, content: &str) -> Result<AgentConfigParsed, String> {
        // This is a fallback parser for simple flat YAML configs
        // serde_yaml handles nested structures properly
        let mut config = AgentConfigParsed {
            name: String::new(),
            cwd: None,
            env: HashMap::new(),
            permissions: PermissionConfig::default(),
            mcp_servers: Vec::new(),
            skills: Vec::new(),
            hooks: Vec::new(),
            capabilities: Vec::new(),
            transport: "stdio".to_string(),
            command: None,
            args: None,
            url: None,
            headers: None,
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = self.parse_line(line) {
                match key.as_str() {
                    "name" => config.name = value.to_string(),
                    "cwd" => config.cwd = Some(value.to_string()),
                    "transport" => config.transport = value.to_string(),
                    "command" => config.command = Some(value.to_string()),
                    "url" => config.url = Some(value.to_string()),
                    "mcp_servers" => config.mcp_servers = self.parse_list(&value),
                    "skills" => config.skills = self.parse_list(&value),
                    "hooks" => config.hooks = self.parse_list(&value),
                    "capabilities" => config.capabilities = self.parse_list(&value),
                    _ => {}
                }
            }
        }

        Ok(config)
    }

    fn parse_line(&self, line: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            // Remove quotes if present
            let value = value.trim_matches('"').trim_matches('\'').to_string();
            Some((key, value))
        } else {
            None
        }
    }

    fn parse_list(&self, value: &str) -> Vec<String> {
        // Parse comma-separated list or YAML array format
        if value.starts_with('[') && value.ends_with(']') {
            // Array format: [item1, item2]
            let inner = &value[1..value.len()-1];
            inner.split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Comma-separated
            value.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }

    /// Save parsed config to SQLite
    pub fn save_to_db(&self, config: &AgentConfigParsed, conn: &Connection) -> Result<String, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Serialize config to JSON
        let config_json = serde_json::to_string(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // Check if agent_configs table exists, create if not
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                config_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        ).map_err(|e| e.to_string())?;

        // Insert or update
        conn.execute(
            "INSERT OR REPLACE INTO agent_configs (id, name, config_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, config.name, config_json, now, now],
        ).map_err(|e| e.to_string())?;

        Ok(id)
    }

    /// Load config from SQLite by name
    pub fn load_from_db(&self, name: &str, conn: &Connection) -> Result<Option<AgentConfigParsed>, String> {
        let mut stmt = conn.prepare(
            "SELECT config_json FROM agent_configs WHERE name = ?1"
        ).map_err(|e| e.to_string())?;

        let result = stmt.query_row(params![name], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        });

        match result {
            Ok(json) => {
                let config: AgentConfigParsed = serde_json::from_str(&json)
                    .map_err(|e| format!("Failed to deserialize config: {}", e))?;
                Ok(Some(config))
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// List all saved configs
    pub fn list_configs(&self, conn: &Connection) -> Result<Vec<(String, String)>, String> {
        let mut stmt = conn.prepare(
            "SELECT id, name FROM agent_configs ORDER BY name"
        ).map_err(|e| e.to_string())?;

        let configs = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(configs)
    }

    /// Delete config by name
    pub fn delete_config(&self, name: &str, conn: &Connection) -> Result<(), String> {
        conn.execute(
            "DELETE FROM agent_configs WHERE name = ?1",
            params![name],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }
}

impl Default for AgentConfigParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Example YAML config template
pub fn get_example_config() -> String {
    r#"# Agent configuration example
name: my-agent
transport: stdio
command: npx
cwd: /home/user/project
mcp_servers: [filesystem, git]
skills: [code-review, testing]
hooks: [pre-commit]
capabilities: [file-read, file-write, bash]

# Permission settings (optional)
# permissions:
#   deny_by_default: false
#   allow:
#     - pattern: "Read:.*"
#       description: "Can read any file"
#   deny:
#     - pattern: "Bash:rm -rf.*"
#       description: "Cannot run destructive commands"
"#.to_string()
}