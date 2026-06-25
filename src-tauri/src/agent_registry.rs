//! Agent Registry - Docker-like Agent management system
//!
//! Implements Base → Template → Instance architecture for Agent configuration.
//!
//! Features:
//! - Agent Base/Template/Instance hierarchy (Phase 1 Week 1)
//! - Self-Optimizing capabilities tracking (Phase 1 Week 2)
//! - Scene/platform specific performance metrics

#![allow(dead_code)] // Reserved for future Agent Teams orchestration

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Re-export for self-optimizing router integration
pub use crate::self_optimizing_router::{ProficiencyScore, SelfOptimizingConfig};
pub use crate::agent_adapter::types::{SceneType, Platform};

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
    /// Agent proficiency scores (for Self-Optimizing Router)
    proficiency_scores: HashMap<String, HashMap<String, ProficiencyScore>>, // agent_id -> (scene:platform) -> score
    /// Self-optimizing config
    optimizing_config: SelfOptimizingConfig,
}


impl AgentRegistry {
    /// Create a new Agent Registry
    pub fn new() -> Self {
        Self {
            bases: HashMap::new(),
            templates: HashMap::new(),
            instances: HashMap::new(),
            max_derivation_depth: 3,
            proficiency_scores: HashMap::new(),
            optimizing_config: SelfOptimizingConfig::default(),
        }
    }

    /// Create with custom self-optimizing config
    pub fn with_optimizing_config(config: SelfOptimizingConfig) -> Self {
        Self {
            bases: HashMap::new(),
            templates: HashMap::new(),
            instances: HashMap::new(),
            max_derivation_depth: 3,
            proficiency_scores: HashMap::new(),
            optimizing_config: config,
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

    // === Self-Optimizing Router Integration (Phase 1 Week 2) ===

    /// Record execution for proficiency tracking
    pub fn record_execution(
        &mut self,
        agent_id: &str,
        scene: SceneType,
        platform: Platform,
        success: bool,
        latency_ms: u64,
        cost: f32,
    ) {
        let key = format!("{}:{}", scene_to_short(&scene), platform_to_short(&platform));

        // Get or create agent's proficiency map
        let agent_scores = self.proficiency_scores.entry(agent_id.to_string()).or_insert_with(HashMap::new);

        // Get or create proficiency score for this scene+platform
        let score = agent_scores.entry(key.clone()).or_insert_with(|| {
            ProficiencyScore::new(agent_id.to_string(), scene, platform)
        });

        // Update proficiency
        score.update(success, latency_ms, cost, self.optimizing_config.ewma_alpha);
    }

    /// Get best agent for scene+platform based on proficiency
    pub fn get_best_agent_for(&self, scene: SceneType, platform: Platform) -> Option<String> {
        let key = format!("{}:{}", scene_to_short(&scene), platform_to_short(&platform));

        // Find agent with highest proficiency for this scene+platform
        let mut best: Option<(String, f32)> = None;

        for (agent_id, scores) in &self.proficiency_scores {
            if let Some(score) = scores.get(&key) {
                let composite = score.composite_score(&self.optimizing_config);
                if best.is_none() || composite > best.as_ref().unwrap().1 {
                    best = Some((agent_id.clone(), composite));
                }
            }
        }

        best.map(|(id, _)| id)
    }

    /// Get agent proficiency scores
    pub fn get_agent_proficiencies(&self, agent_id: &str) -> Option<Vec<ProficiencyScore>> {
        self.proficiency_scores.get(agent_id).map(|scores| scores.values().cloned().collect())
    }

    /// Get all proficiency scores
    pub fn get_all_proficiencies(&self) -> Vec<ProficiencyScore> {
        self.proficiency_scores.values()
            .flat_map(|scores| scores.values().cloned())
            .collect()
    }

    /// Reset proficiency scores
    pub fn reset_proficiencies(&mut self) {
        self.proficiency_scores.clear();
    }

    /// Get agents capable of handling a scene
    pub fn get_agents_for_scene(&self, scene: SceneType) -> Vec<String> {
        let scene_str = scene_to_short(&scene);

        self.proficiency_scores.keys()
            .filter(|agent_id| {
                self.proficiency_scores.get(*agent_id)
                    .map(|scores| scores.keys().any(|k| k.starts_with(&scene_str)))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}

// Helper functions for scene/platform conversion

fn scene_to_short(scene: &SceneType) -> &'static str {
    match scene {
        SceneType::WebDevelopment => "web",
        SceneType::MiniProgramDevelopment => "mini",
        SceneType::DesktopDevelopment => "desktop",
        SceneType::GameDevelopment => "game",
        SceneType::GameArtGeneration => "game_art",
        SceneType::GameCrossPlatform => "game_cross",
        SceneType::GamePerformanceOptimization => "game_perf",
        SceneType::Marketing => "marketing",
        SceneType::Finance => "finance",
        SceneType::Design => "design",
    }
}

fn platform_to_short(platform: &Platform) -> &'static str {
    match platform {
        Platform::Web => "web",
        Platform::MiniProgram => "mini",
        Platform::Desktop => "desktop",
        Platform::Mobile => "mobile",
        Platform::Game => "game",
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