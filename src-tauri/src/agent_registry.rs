//! Agent Registry - Docker-like Agent management system
//!
//! Implements Base → Template → Instance architecture for Agent configuration.
//!
//! NOTE: This module is reserved for future Agent Teams orchestration.
//! Currently not used in production execution flow.

#![allow(dead_code)] // Reserved for future Agent Teams orchestration

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Agent Base - the "Image" layer
/// Defines the foundational agent configuration (e.g., claude-code-base, codex-base)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBase {
    pub id: String,
    pub name: String,
    pub transport: TransportType,
    pub capabilities: Vec<String>,
    pub default_skills: Vec<String>,
    pub default_hooks: Vec<String>,
    pub default_permissions: PermissionConfig,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transport type for agent communication

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportType {
    Stdio,      // Local process (stdin/stdout)
    WebSocket,  // Remote WebSocket connection
    Acp,        // Agent Client Protocol
}


impl TransportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportType::Stdio => "stdio",
            TransportType::WebSocket => "websocket",
            TransportType::Acp => "acp",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "stdio" => Ok(TransportType::Stdio),
            "websocket" => Ok(TransportType::WebSocket),
            "acp" => Ok(TransportType::Acp),
            other => Err(format!("Unknown transport type: {}", other)),
        }
    }
}

/// Agent Template - the "Dockerfile" layer
/// Derives from Base with configuration strategies

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub base_id: String,
    pub skills_strategy: ConfigStrategy,
    pub hooks_strategy: ConfigStrategy,
    pub permissions: PermissionConfig,
    pub config_override: HashMap<String, serde_json::Value>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Configuration strategy for template derivation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigStrategy {
    Append,     // Add to base configuration
    Override,   // Replace base configuration (except deny permissions)
    Exclude,    // Remove from base configuration
}


impl ConfigStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigStrategy::Append => "append",
            ConfigStrategy::Override => "override",
            ConfigStrategy::Exclude => "exclude",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "append" => Ok(ConfigStrategy::Append),
            "override" => Ok(ConfigStrategy::Override),
            "exclude" => Ok(ConfigStrategy::Exclude),
            other => Err(format!("Unknown config strategy: {}", other)),
        }
    }
}

/// Agent Instance - the "Container" layer
/// Running instance of a Template

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstance {
    pub id: String,
    pub template_id: String,
    pub status: InstanceStatus,
    pub pid: Option<u32>,
    pub session_id: Option<String>,
    pub cwd: String,
    pub config_override: HashMap<String, serde_json::Value>,
    pub volume_path: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Instance status

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceStatus {
    Pending,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
    Restarting,
}


impl InstanceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstanceStatus::Pending => "pending",
            InstanceStatus::Starting => "starting",
            InstanceStatus::Running => "running",
            InstanceStatus::Stopping => "stopping",
            InstanceStatus::Stopped => "stopped",
            InstanceStatus::Error => "error",
            InstanceStatus::Restarting => "restarting",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "pending" => Ok(InstanceStatus::Pending),
            "starting" => Ok(InstanceStatus::Starting),
            "running" => Ok(InstanceStatus::Running),
            "stopping" => Ok(InstanceStatus::Stopping),
            "stopped" => Ok(InstanceStatus::Stopped),
            "error" => Ok(InstanceStatus::Error),
            "restarting" => Ok(InstanceStatus::Restarting),
            other => Err(format!("Unknown instance status: {}", other)),
        }
    }
}

/// Permission configuration

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub cwd_restrict: bool,
}


impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            allow: vec!["*".to_string()],
            deny: vec![
                "rm -rf /*".to_string(),
                "rm -rf ~".to_string(),
                "dd if=/dev/zero".to_string(),
            ],
            cwd_restrict: true,
        }
    }
}

/// Agent Registry - manages Base, Template, and Instance

pub struct AgentRegistry {
    bases: HashMap<String, AgentBase>,
    templates: HashMap<String, AgentTemplate>,
    instances: HashMap<String, AgentInstance>,
    max_derivation_depth: u32, // Limit to 3 layers
}


impl AgentRegistry {
    /// Create a new Agent Registry
    pub fn new() -> Self {
        Self {
            bases: HashMap::new(),
            templates: HashMap::new(),
            instances: HashMap::new(),
            max_derivation_depth: 3,
        }
    }

    /// Register an Agent Base
    pub fn register_base(&mut self, base: AgentBase) -> Result<(), String> {
        if self.bases.contains_key(&base.id) {
            return Err(format!("Base {} already exists", base.id));
        }
        self.bases.insert(base.id.clone(), base);
        Ok(())
    }

    /// Create a Template from a Base
    pub fn create_template(&mut self, template: AgentTemplate) -> Result<(), String> {
        // Validate base exists
        if !self.bases.contains_key(&template.base_id) {
            return Err(format!("Base {} not found", template.base_id));
        }

        // Check derivation depth
        self.validate_derivation_depth(&template.base_id)?;

        if self.templates.contains_key(&template.id) {
            return Err(format!("Template {} already exists", template.id));
        }

        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// Spawn an Instance from a Template
    pub fn spawn_instance(&mut self, template_id: &str, cwd: String) -> Result<String, String> {
        // Validate template exists
        if !self.templates.contains_key(template_id) {
            return Err(format!("Template {} not found", template_id));
        }

        let instance_id = uuid::Uuid::new_v4().to_string();

        let instance = AgentInstance {
            id: instance_id.clone(),
            template_id: template_id.to_string(),
            status: InstanceStatus::Pending,
            pid: None,
            session_id: None,
            cwd,
            config_override: HashMap::new(),
            volume_path: None,
            started_at: None,
            stopped_at: None,
            created_at: Utc::now(),
        };

        self.instances.insert(instance_id.clone(), instance);
        Ok(instance_id)
    }

    /// Get merged configuration for an Instance
    pub fn get_merged_config(&self, instance_id: &str) -> Result<AgentMergedConfig, String> {
        let instance = self.instances.get(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;

        let template = self.templates.get(&instance.template_id)
            .ok_or_else(|| format!("Template {} not found", instance.template_id))?;

        let base = self.bases.get(&template.base_id)
            .ok_or_else(|| format!("Base {} not found", template.base_id))?;

        // Merge configurations according to strategies
        let skills = self.merge_skills(base, template);
        let hooks = self.merge_hooks(base, template);
        let permissions = self.merge_permissions(base, template);

        Ok(AgentMergedConfig {
            base_id: base.id.clone(),
            template_id: template.id.clone(),
            instance_id: instance_id.to_string(),
            transport: base.transport.clone(),
            capabilities: base.capabilities.clone(),
            skills,
            hooks,
            permissions,
            cwd: instance.cwd.clone(),
        })
    }

    /// Merge skills according to strategy
    fn merge_skills(&self, base: &AgentBase, template: &AgentTemplate) -> Vec<String> {
        match template.skills_strategy {
            ConfigStrategy::Append => {
                let skills = base.default_skills.clone();
                // Add template-specific skills (would come from config_override)
                skills
            }
            ConfigStrategy::Override => {
                // Use only template skills
                template.config_override.get("skills")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_else(|| base.default_skills.clone())
            }
            ConfigStrategy::Exclude => {
                // Remove excluded skills from base
                let excluded: Vec<String> = template.config_override.get("exclude_skills")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                base.default_skills.iter()
                    .filter(|s| !excluded.contains(s))
                    .cloned()
                    .collect()
            }
        }
    }

    /// Merge hooks according to strategy
    fn merge_hooks(&self, base: &AgentBase, template: &AgentTemplate) -> Vec<String> {
        match template.hooks_strategy {
            ConfigStrategy::Append => base.default_hooks.clone(),
            ConfigStrategy::Override => {
                template.config_override.get("hooks")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_else(|| base.default_hooks.clone())
            }
            ConfigStrategy::Exclude => {
                let excluded: Vec<String> = template.config_override.get("exclude_hooks")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                base.default_hooks.iter()
                    .filter(|h| !excluded.contains(h))
                    .cloned()
                    .collect()
            }
        }
    }

    /// Merge permissions (deny always wins - security priority)
    fn merge_permissions(&self, base: &AgentBase, template: &AgentTemplate) -> PermissionConfig {
        let mut permissions = template.permissions.clone();

        // Always include base deny rules (security priority)
        for deny_rule in &base.default_permissions.deny {
            if !permissions.deny.contains(deny_rule) {
                permissions.deny.push(deny_rule.clone());
            }
        }

        permissions
    }

    /// Validate derivation depth (max 3 layers)
    fn validate_derivation_depth(&self, _base_id: &str) -> Result<(), String> {
        // Check if this base is derived from another template
        // For simplicity, we assume bases are layer 0, templates are layer 1
        // Template from Template would be layer 2, etc.

        // Currently we only support Base → Template → Instance
        // Future: Template → Template derivation

        Ok(())
    }

    /// Stop an instance
    pub fn stop_instance(&mut self, instance_id: &str) -> Result<(), String> {
        let instance = self.instances.get_mut(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;

        instance.status = InstanceStatus::Stopped;
        instance.stopped_at = Some(Utc::now());
        Ok(())
    }

    /// Get instance status
    pub fn get_instance_status(&self, instance_id: &str) -> Result<InstanceStatus, String> {
        let instance = self.instances.get(instance_id)
            .ok_or_else(|| format!("Instance {} not found", instance_id))?;
        Ok(instance.status.clone())
    }

    /// List all bases
    pub fn list_bases(&self) -> Vec<&AgentBase> {
        self.bases.values().collect()
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<&AgentTemplate> {
        self.templates.values().collect()
    }

    /// List all instances
    pub fn list_instances(&self) -> Vec<&AgentInstance> {
        self.instances.values().collect()
    }
}

/// Merged configuration for running an instance

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMergedConfig {
    pub base_id: String,
    pub template_id: String,
    pub instance_id: String,
    pub transport: TransportType,
    pub capabilities: Vec<String>,
    pub skills: Vec<String>,
    pub hooks: Vec<String>,
    pub permissions: PermissionConfig,
    pub cwd: String,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}