//! Hermes Traits - Self-Evolving, Self-Healing, MCP-Integrated, Skill-Capable
//!
//! Phase 4 enhancement: Deep integration with MCP + Skill system
//! These traits extend Hermes Core with self-improvement capabilities.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Self-Evolving Trait
// ---------------------------------------------------------------------------

/// Execution result to analyze for evolution suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    pub task_id: String,
    pub agent_name: String,
    pub skill_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub tokens_used: u64,
    pub error: Option<String>,
    pub output_quality_score: Option<f64>,
    pub timestamp: i64,
}

/// Evolution suggestion generated from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionSuggestion {
    pub id: String,
    pub target_type: EvolutionTarget,
    pub target_id: String,
    pub priority: EvolutionPriority,
    pub description: String,
    pub evidence: serde_json::Value,
    pub auto_applicable: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvolutionTarget {
    Skill,
    AgentConfig,
    Workflow,
    SystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvolutionPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Evolution record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionRecord {
    pub id: String,
    pub suggestion_id: String,
    pub applied_at: i64,
    pub before_state: serde_json::Value,
    pub after_state: serde_json::Value,
    pub success: bool,
    pub rollback_available: bool,
}

/// Self-Evolving trait - analyze and auto-improve
pub trait SelfEvolving {
    /// Analyze execution result and generate evolution suggestion
    fn analyze_execution_result(&self, result: &ExecutionResult) -> Option<EvolutionSuggestion>;

    /// Apply an evolution suggestion
    fn apply_evolution(&mut self, suggestion: &EvolutionSuggestion) -> Result<EvolutionRecord, String>;

    /// Get evolution history
    fn get_evolution_history(&self) -> Vec<EvolutionRecord>;

    /// Rollback an evolution if needed
    fn rollback_evolution(&mut self, record_id: &str) -> Result<(), String>;

    /// Get pending evolution suggestions
    fn get_pending_suggestions(&self) -> Vec<EvolutionSuggestion>;
}

// ---------------------------------------------------------------------------
// Self-Healing Trait
// ---------------------------------------------------------------------------

/// Failure pattern detected from error analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailurePattern {
    pub pattern_id: String,
    pub error_type: String,
    pub error_message_pattern: String,
    pub frequency: u32,
    pub first_seen: i64,
    pub last_seen: i64,
    pub affected_agents: Vec<String>,
    pub affected_skills: Vec<String>,
}

/// Healing action to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealingAction {
    pub action_id: String,
    pub action_type: HealingActionType,
    pub description: String,
    pub parameters: serde_json::Value,
    pub estimated_success_rate: f64,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealingActionType {
    Retry,
    Fallback,
    Reset,
    Degrade,
    AutoFix,
    RestartAgent,
    ClearCache,
    AdjustConfig,
}

/// Healing result after applying action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealingResult {
    pub action_id: String,
    pub success: bool,
    pub resolved_at: Option<i64>,
    pub resolution_description: Option<String>,
    pub follow_up_actions: Vec<HealingAction>,
    pub duration_ms: u64,
}

/// Healing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealingStats {
    pub total_attempts: u32,
    pub successful_heals: u32,
    pub success_rate: f64,
    pub average_duration_ms: u64,
    pub most_common_patterns: Vec<FailurePattern>,
    pub most_effective_actions: Vec<String>,
}

/// Self-Healing trait - detect and recover from failures
pub trait SelfHealing {
    /// Detect failure pattern from error
    fn detect_failure_pattern(&self, error: &str, context: serde_json::Value) -> Option<FailurePattern>;

    /// Attempt healing with suggested action
    fn attempt_healing(&mut self, pattern: &FailurePattern) -> Result<HealingResult, String>;

    /// Get healing statistics
    fn get_healing_stats(&self) -> HealingStats;

    /// Register a new healing strategy
    fn register_healing_strategy(&mut self, pattern_type: &str, action: HealingAction);

    /// Get available healing strategies for a pattern
    fn get_available_strategies(&self, pattern: &FailurePattern) -> Vec<HealingAction>;
}

// ---------------------------------------------------------------------------
// MCP-Integrated Trait
// ---------------------------------------------------------------------------

/// MCP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub server_name: String,
    pub server_type: String,
    pub connected: bool,
    pub tools_count: u32,
    pub resources_count: u32,
    pub last_ping: Option<i64>,
}

/// MCP tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    pub tool_name: String,
    pub server_name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: Option<serde_json::Value>,
}

/// MCP call result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCallResult {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub server_name: String,
    pub tool_name: String,
}

/// MCP-Integrated trait - call MCP tools
pub trait McpIntegrated {
    /// Call an MCP tool
    fn call_mcp_tool(&self, server: &str, tool: &str, params: serde_json::Value) -> Result<McpCallResult, String>;

    /// List available MCP tools on a server
    fn list_mcp_tools(&self, server: &str) -> Result<Vec<McpToolInfo>, String>;

    /// Validate MCP access for an agent
    fn validate_mcp_access(&self, agent: &str, tool: &str) -> bool;

    /// Get connected MCP servers
    fn get_connected_servers(&self) -> Vec<McpServerInfo>;

    /// Register MCP server connection
    fn register_mcp_server(&mut self, server_info: McpServerInfo);

    /// Disconnect MCP server
    fn disconnect_mcp_server(&mut self, server_name: &str) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Skill-Capable Trait
// ---------------------------------------------------------------------------

/// Skill invocation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillResult {
    pub skill_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tool_calls_count: u32,
    pub evolved: bool,
    pub evolution_summary: Option<String>,
}

/// Skill feedback for evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillFeedback {
    pub skill_name: String,
    pub rating: u8, // 1-5
    pub execution_id: String,
    pub success: bool,
    pub suggestions: Vec<String>,
    pub timestamp: i64,
}

/// Skill version information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillVersion {
    pub skill_name: String,
    pub semver: String,
    pub generation: u32,
    pub content_hash: String,
    pub last_evolved_at: Option<i64>,
    pub last_evolution_reason: Option<String>,
}

/// Skill-Capable trait - invoke and evolve skills
pub trait SkillCapable {
    /// Invoke a skill
    fn invoke_skill(&self, skill: &str, params: serde_json::Value) -> Result<SkillResult, String>;

    /// Evolve a skill based on feedback
    fn evolve_skill(&mut self, skill: &str, feedback: &SkillFeedback) -> Result<SkillVersion, String>;

    /// Get skill version history
    fn get_skill_history(&self, skill: &str) -> Result<Vec<SkillVersion>, String>;

    /// Rollback skill to previous version
    fn rollback_skill(&mut self, skill: &str, version: &str) -> Result<SkillVersion, String>;

    /// List available skills
    fn list_skills(&self) -> Vec<String>;

    /// Get skill info
    fn get_skill_info(&self, skill: &str) -> Option<SkillVersion>;
}

// ---------------------------------------------------------------------------
// Enhanced Hermes - Combining all traits
// ---------------------------------------------------------------------------

/// Enhanced Hermes struct that implements all four traits
pub struct EnhancedHermes {
    /// Evolution records storage
    evolution_records: Vec<EvolutionRecord>,
    /// Pending evolution suggestions
    pending_suggestions: Vec<EvolutionSuggestion>,
    /// Failure patterns library
    failure_patterns: Vec<FailurePattern>,
    /// Healing strategies registry
    healing_strategies: HashMap<String, Vec<HealingAction>>,
    /// Healing stats
    healing_stats: HealingStats,
    /// MCP servers registry
    mcp_servers: HashMap<String, McpServerInfo>,
    /// Skills registry
    skills_registry: HashMap<String, SkillVersion>,
    /// Skill feedback history
    skill_feedbacks: HashMap<String, Vec<SkillFeedback>>,
}

use std::collections::HashMap;

impl EnhancedHermes {
    pub fn new() -> Self {
        Self {
            evolution_records: Vec::new(),
            pending_suggestions: Vec::new(),
            failure_patterns: Vec::new(),
            healing_strategies: HashMap::new(),
            healing_stats: HealingStats {
                total_attempts: 0,
                successful_heals: 0,
                success_rate: 0.0,
                average_duration_ms: 0,
                most_common_patterns: Vec::new(),
                most_effective_actions: Vec::new(),
            },
            mcp_servers: HashMap::new(),
            skills_registry: HashMap::new(),
            skill_feedbacks: HashMap::new(),
        }
    }
}

impl SelfEvolving for EnhancedHermes {
    fn analyze_execution_result(&self, result: &ExecutionResult) -> Option<EvolutionSuggestion> {
        // Analyze execution result for improvement opportunities
        if !result.success {
            // Failed execution - suggest improvement
            return Some(EvolutionSuggestion {
                id: uuid::Uuid::new_v4().to_string(),
                target_type: if result.skill_name.is_empty() {
                    EvolutionTarget::AgentConfig
                } else {
                    EvolutionTarget::Skill
                },
                target_id: if result.skill_name.is_empty() {
                    result.agent_name.clone()
                } else {
                    result.skill_name.clone()
                },
                priority: EvolutionPriority::High,
                description: format!("Fix failure pattern: {}", result.error.unwrap_or_default()),
                evidence: serde_json::to_value(result).unwrap_or(serde_json::Value::Null),
                auto_applicable: false, // Needs manual review for failures
                created_at: chrono::Utc::now().timestamp_millis(),
            });
        }

        if result.output_quality_score.unwrap_or(5.0) < 3.0 {
            // Low quality output - suggest improvement
            return Some(EvolutionSuggestion {
                id: uuid::Uuid::new_v4().to_string(),
                target_type: EvolutionTarget::Skill,
                target_id: result.skill_name.clone(),
                priority: EvolutionPriority::Medium,
                description: "Improve output quality based on low score".to_string(),
                evidence: serde_json::json!({
                    "quality_score": result.output_quality_score,
                    "task_id": result.task_id,
                }),
                auto_applicable: true,
                created_at: chrono::Utc::now().timestamp_millis(),
            });
        }

        None
    }

    fn apply_evolution(&mut self, suggestion: &EvolutionSuggestion) -> Result<EvolutionRecord, String> {
        if !suggestion.auto_applicable {
            return Err("Evolution requires manual approval".to_string());
        }

        let record = EvolutionRecord {
            id: uuid::Uuid::new_v4().to_string(),
            suggestion_id: suggestion.id.clone(),
            applied_at: chrono::Utc::now().timestamp_millis(),
            before_state: serde_json::Value::Null, // Would capture actual state
            after_state: serde_json::Value::Null,  // Would capture new state
            success: true,
            rollback_available: true,
        };

        self.evolution_records.push(record.clone());
        Ok(record)
    }

    fn get_evolution_history(&self) -> Vec<EvolutionRecord> {
        self.evolution_records.clone()
    }

    fn rollback_evolution(&mut self, record_id: &str) -> Result<(), String> {
        let record = self.evolution_records.iter()
            .find(|r| r.id == record_id)
            .ok_or_else(|| format!("Evolution record '{}' not found", record_id))?;

        if !record.rollback_available {
            return Err("Rollback not available for this evolution".to_string());
        }

        // Would restore before_state
        println!("[EnhancedHermes] Rollback evolution '{}' applied", record_id);
        Ok(())
    }

    fn get_pending_suggestions(&self) -> Vec<EvolutionSuggestion> {
        self.pending_suggestions.clone()
    }
}

impl SelfHealing for EnhancedHermes {
    fn detect_failure_pattern(&self, error: &str, context: serde_json::Value) -> Option<FailurePattern> {
        // Match error against known patterns
        for pattern in &self.failure_patterns {
            if error.contains(&pattern.error_message_pattern) {
                return Some(pattern.clone());
            }
        }

        // Create new pattern for unknown error
        Some(FailurePattern {
            pattern_id: uuid::Uuid::new_v4().to_string(),
            error_type: "unknown".to_string(),
            error_message_pattern: error.to_string(),
            frequency: 1,
            first_seen: chrono::Utc::now().timestamp_millis(),
            last_seen: chrono::Utc::now().timestamp_millis(),
            affected_agents: vec![context.get("agent").and_then(|v| v.as_str()).unwrap_or("").to_string()],
            affected_skills: vec![context.get("skill").and_then(|v| v.as_str()).unwrap_or("").to_string()],
        })
    }

    fn attempt_healing(&mut self, pattern: &FailurePattern) -> Result<HealingResult, String> {
        let strategies = self.get_available_strategies(pattern);
        if strategies.is_empty() {
            return Err("No healing strategies available for this pattern".to_string());
        }

        // Try first strategy
        let action = strategies.first().unwrap();
        let start_time = chrono::Utc::now().timestamp_millis();

        // Simulate healing attempt
        let success = action.estimated_success_rate > 0.5;

        // Update stats
        self.healing_stats.total_attempts += 1;
        if success {
            self.healing_stats.successful_heals += 1;
        }
        self.healing_stats.success_rate = self.healing_stats.successful_heals as f64 / self.healing_stats.total_attempts as f64;

        Ok(HealingResult {
            action_id: action.action_id.clone(),
            success,
            resolved_at: if success { Some(chrono::Utc::now().timestamp_millis()) } else { None },
            resolution_description: if success { Some("Healing action applied successfully".to_string()) } else { None },
            follow_up_actions: if !success { strategies.into_iter().skip(1).collect() } else { vec![] },
            duration_ms: (chrono::Utc::now().timestamp_millis() - start_time) as u64,
        })
    }

    fn get_healing_stats(&self) -> HealingStats {
        self.healing_stats.clone()
    }

    fn register_healing_strategy(&mut self, pattern_type: &str, action: HealingAction) {
        self.healing_strategies.entry(pattern_type.to_string())
            .or_default()
            .push(action);
    }

    fn get_available_strategies(&self, pattern: &FailurePattern) -> Vec<HealingAction> {
        self.healing_strategies.get(&pattern.error_type)
            .cloned()
            .unwrap_or_default()
    }
}

impl McpIntegrated for EnhancedHermes {
    fn call_mcp_tool(&self, server: &str, tool: &str, params: serde_json::Value) -> Result<McpCallResult, String> {
        // Check server is connected
        if !self.mcp_servers.get(server).map(|s| s.connected).unwrap_or(false) {
            return Err(format!("MCP server '{}' not connected", server));
        }

        // Simulate MCP call (actual implementation would use MCP client)
        Ok(McpCallResult {
            success: true,
            result: Some(params),
            error: None,
            duration_ms: 100,
            server_name: server.to_string(),
            tool_name: tool.to_string(),
        })
    }

    fn list_mcp_tools(&self, server: &str) -> Result<Vec<McpToolInfo>, String> {
        if !self.mcp_servers.contains_key(server) {
            return Err(format!("MCP server '{}' not registered", server));
        }

        // Would query actual server for tools
        Ok(vec![])
    }

    fn validate_mcp_access(&self, agent: &str, tool: &str) -> bool {
        // Check permission rules
        true // Default allow
    }

    fn get_connected_servers(&self) -> Vec<McpServerInfo> {
        self.mcp_servers.values().cloned().collect()
    }

    fn register_mcp_server(&mut self, server_info: McpServerInfo) {
        self.mcp_servers.insert(server_info.server_name.clone(), server_info);
    }

    fn disconnect_mcp_server(&mut self, server_name: &str) -> Result<(), String> {
        if let Some(server) = self.mcp_servers.get_mut(server_name) {
            server.connected = false;
            Ok(())
        } else {
            Err(format!("MCP server '{}' not found", server_name))
        }
    }
}

impl SkillCapable for EnhancedHermes {
    fn invoke_skill(&self, skill: &str, params: serde_json::Value) -> Result<SkillResult, String> {
        if !self.skills_registry.contains_key(skill) {
            return Err(format!("Skill '{}' not registered", skill));
        }

        // Simulate skill invocation
        Ok(SkillResult {
            skill_name: skill.to_string(),
            success: true,
            output: format!("Skill '{}' executed with params: {}", skill, params),
            error: None,
            duration_ms: 500,
            tool_calls_count: 3,
            evolved: false,
            evolution_summary: None,
        })
    }

    fn evolve_skill(&mut self, skill: &str, feedback: &SkillFeedback) -> Result<SkillVersion, String> {
        let current = self.skills_registry.get(skill)
            .ok_or_else(|| format!("Skill '{}' not found", skill))?;

        let new_version = SkillVersion {
            skill_name: skill.to_string(),
            semver: increment_version(&current.semver),
            generation: current.generation + 1,
            content_hash: uuid::Uuid::new_v4().to_string(),
            last_evolved_at: Some(feedback.timestamp),
            last_evolution_reason: Some(feedback.suggestions.join("; ")),
        };

        self.skills_registry.insert(skill.to_string(), new_version.clone());
        self.skill_feedbacks.entry(skill.to_string()).or_default().push(feedback.clone());

        Ok(new_version)
    }

    fn get_skill_history(&self, skill: &str) -> Result<Vec<SkillVersion>, String> {
        // Would retrieve from storage
        Ok(self.skills_registry.get(skill).map(|v| vec![v.clone()]).unwrap_or_default())
    }

    fn rollback_skill(&mut self, skill: &str, version: &str) -> Result<SkillVersion, String> {
        // Would restore previous version
        let current = self.skills_registry.get_mut(skill)
            .ok_or_else(|| format!("Skill '{}' not found", skill))?;
        current.semver = version.to_string();
        Ok(current.clone())
    }

    fn list_skills(&self) -> Vec<String> {
        self.skills_registry.keys().cloned().collect()
    }

    fn get_skill_info(&self, skill: &str) -> Option<SkillVersion> {
        self.skills_registry.get(skill).cloned()
    }
}

impl Default for EnhancedHermes {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

fn increment_version(semver: &str) -> String {
    let parts: Vec<&str> = semver.split('.').collect();
    if parts.len() == 3 {
        let patch = parts[2].parse::<u32>().unwrap_or(0) + 1;
        format!("{}.{}.{}", parts[0], parts[1], patch)
    } else {
        "1.0.1".to_string()
    }
}