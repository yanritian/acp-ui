//! invoke_skill - Meta-tool for delegating capabilities to Skill ecosystem
//!
//! This is the core meta-tool inspired by OpenClacky's architecture:
//! - Only 12-16 core tools (immutable)
//! - All capabilities delegated through invoke_skill
//! - Skills can self-evolve based on execution feedback

use async_trait::async_trait;
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

use hermes_core::{
    tool_schema, JsonSchema, Skill, SkillProvider, ToolError, ToolHandler, ToolSchema,
    EvolutionTriggers,
};

use super::skill_evolution;

// ---------------------------------------------------------------------------
// SkillInvocation - Local type for invocation request
// ---------------------------------------------------------------------------

/// Invocation request for invoke_skill meta-tool
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillInvocation {
    /// Skill name to invoke
    pub skill_name: String,
    /// Parameters for skill execution
    #[serde(default)]
    pub parameters: serde_json::Map<String, serde_json::Value>,
    /// Execution context
    #[serde(default)]
    pub context: SkillContext,
}

/// Context for skill execution
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillContext {
    /// Session ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Agent name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    /// Task ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Working directory
    #[serde(default)]
    pub cwd: String,
}

/// Result from skill execution
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SkillExecutionResult {
    /// Success status
    pub success: bool,
    /// Output content
    #[serde(default)]
    pub output: String,
    /// Error message if failed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Execution duration in milliseconds
    #[serde(default)]
    pub duration_ms: u64,
    /// Number of tool calls made
    #[serde(default)]
    pub tool_calls_count: u32,
    /// Whether self-evolution was triggered
    #[serde(default)]
    pub evolved: bool,
    /// Evolution summary if evolved
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolution_summary: Option<String>,
}

// ---------------------------------------------------------------------------
// InvokeSkillHandler - The Core Meta-Tool
// ---------------------------------------------------------------------------

/// Meta-tool that invokes a skill and orchestrates its execution.
///
/// This is the single entry point for all skill-based capabilities.
/// Skills define what to do, invoke_skill handles how to execute it.
pub struct InvokeSkillHandler {
    provider: Arc<dyn SkillProvider>,
    evolution_triggers: EvolutionTriggers,
}

impl InvokeSkillHandler {
    pub fn new(provider: Arc<dyn SkillProvider>) -> Self {
        Self {
            provider,
            evolution_triggers: EvolutionTriggers::default(),
        }
    }

    pub fn with_evolution_triggers(mut self, triggers: EvolutionTriggers) -> Self {
        self.evolution_triggers = triggers;
        self
    }

    /// Parse skill content and prepare execution context
    fn prepare_execution(&self, skill: &Skill, invocation: &SkillInvocation) -> String {
        // Build execution prompt from skill content
        let mut prompt = skill.content.clone();

        // Inject parameters
        if !invocation.parameters.is_empty() {
            prompt.push_str("\n\n## Parameters\n");
            for (key, value) in &invocation.parameters {
                prompt.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        // Inject context
        if let Some(session_id) = &invocation.context.session_id {
            prompt.push_str(&format!("\nSession: {}\n", session_id));
        }
        if !invocation.context.cwd.is_empty() {
            prompt.push_str(&format!("\nWorking Directory: {}\n", invocation.context.cwd));
        }

        prompt
    }

    /// Check if skill should evolve and trigger evolution
    async fn check_evolution(&self, skill: &Skill) -> Option<(Skill, String)> {
        if let Some(reason) = skill.should_evolve(&self.evolution_triggers) {
            // Trigger evolution through skill_evolution module
            let evolved = skill_evolution::evolve_skill(skill, &reason);
            let summary = format!(
                "Skill '{}' evolved due to {:?}",
                skill.name,
                reason
            );
            Some((evolved, summary))
        } else {
            None
        }
    }
}

#[async_trait]
impl ToolHandler for InvokeSkillHandler {
    async fn execute(&self, params: Value) -> Result<String, ToolError> {
        let start_time = Instant::now();

        // Parse invocation
        let invocation: SkillInvocation = serde_json::from_value(params.clone())
            .map_err(|e| ToolError::InvalidParams(format!("Invalid invocation: {}", e)))?;

        // Load skill
        let skill = self
            .provider
            .get_skill(&invocation.skill_name)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?
            .ok_or_else(|| ToolError::NotFound(format!(
                "Skill '{}' not found. Use skills_list to see available skills.",
                invocation.skill_name
            )))?;

        // Prepare execution context
        let execution_prompt = self.prepare_execution(&skill, &invocation);
        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Check for evolution opportunity first
        if let Some((evolved_skill, summary)) = self.check_evolution(&skill).await {
            // Update skill with evolved version
            self.provider
                .update_skill(&evolved_skill.name, &evolved_skill.content)
                .await
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

            // Return result with evolution info
            let evolved_result = SkillExecutionResult {
                success: true,
                output: execution_prompt,
                error: None,
                duration_ms,
                tool_calls_count: 0,
                evolved: true,
                evolution_summary: Some(summary),
            };

            return Ok(serde_json::to_string(&evolved_result)
                .unwrap_or_else(|_| "Skill executed and evolved".to_string()));
        }

        // Execute skill (return the prepared prompt for LLM to process)
        // In production, this would integrate with the agent loop
        let result = SkillExecutionResult {
            success: true,
            output: execution_prompt,
            error: None,
            duration_ms,
            tool_calls_count: 0,
            evolved: false,
            evolution_summary: None,
        };

        Ok(serde_json::to_string(&result)
            .unwrap_or_else(|_| "Skill executed successfully".to_string()))
    }

    fn schema(&self) -> ToolSchema {
        let mut props = IndexMap::new();

        props.insert(
            "skill_name".into(),
            json!({
                "type": "string",
                "description": "Name of the skill to invoke. Use fuzzy matching or exact name."
            }),
        );

        props.insert(
            "parameters".into(),
            json!({
                "type": "object",
                "description": "Parameters for skill execution. Keys depend on skill definition.",
                "additionalProperties": true
            }),
        );

        props.insert(
            "context".into(),
            json!({
                "type": "object",
                "description": "Execution context (session_id, cwd, etc.)",
                "properties": {
                    "session_id": {"type": "string"},
                    "cwd": {"type": "string", "description": "Working directory"}
                }
            }),
        );

        tool_schema(
            "invoke_skill",
            "Invoke a skill by name. Skills are capabilities defined in the Skill ecosystem. \
             Use skills_list to discover available skills. Fuzzy matching supported.",
            JsonSchema::object(props, vec!["skill_name".into()]),
        )
    }
}

// ---------------------------------------------------------------------------
// CreateSkillHandler - Natural Language Skill Creation
// ---------------------------------------------------------------------------

/// Tool for creating skills from natural language descriptions.
///
/// Inspired by OpenClacky's approach: "agent drafts SKILL.md, breaks down steps"
pub struct CreateSkillHandler {
    provider: Arc<dyn SkillProvider>,
}

impl CreateSkillHandler {
    pub fn new(provider: Arc<dyn SkillProvider>) -> Self {
        Self { provider }
    }

    /// Generate skill content from natural language description
    fn generate_skill_content(name: &str, description: &str, natural_spec: &str) -> String {
        format!(
            "# {}\n\n\
             ## Description\n{}\n\n\
             ## Natural Language Specification\n{}\n\n\
             ## Steps\n\
             1. Understand the task context\n\
             2. Execute the appropriate actions\n\
             3. Verify and report results\n\n\
             ## Parameters\n\
             (Define parameters based on usage patterns)\n",
            name,
            description,
            natural_spec
        )
    }
}

#[async_trait]
impl ToolHandler for CreateSkillHandler {
    async fn execute(&self, params: Value) -> Result<String, ToolError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParams("Missing 'name' parameter".into()))?;

        let description = params
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Auto-generated skill");

        let natural_spec = params
            .get("natural_spec")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParams("Missing 'natural_spec' parameter".into()))?;

        // Generate skill content from natural language
        let content = Self::generate_skill_content(name, description, natural_spec);

        // Create the skill
        self.provider
            .create_skill(name, &content, Some("user-created"))
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        Ok(json!({
            "created": true,
            "name": name,
            "message": "Skill created from natural language. Use invoke_skill to execute it."
        }).to_string())
    }

    fn schema(&self) -> ToolSchema {
        let mut props = IndexMap::new();

        props.insert(
            "name".into(),
            json!({
                "type": "string",
                "description": "Skill name (kebab-case, e.g., 'my-custom-task')"
            }),
        );

        props.insert(
            "description".into(),
            json!({
                "type": "string",
                "description": "Brief description of what this skill does"
            }),
        );

        props.insert(
            "natural_spec".into(),
            json!({
                "type": "string",
                "description": "Natural language description of the skill's behavior. \
                                Example: 'When user asks to deploy, first build, then test, then push to production'"
            }),
        );

        tool_schema(
            "create_skill",
            "Create a new skill from natural language description. \
             The skill will self-evolve over time based on execution feedback.",
            JsonSchema::object(props, vec!["name".into(), "natural_spec".into()]),
        )
    }
}

// ---------------------------------------------------------------------------
// Core Skill Registry - Built-in Skills
// ---------------------------------------------------------------------------

/// List of 12-16 core immutable skills
pub const CORE_SKILLS: &[&str] = &[
    // Core operational skills
    "web-research",       // Web search and extraction
    "code-execution",     // Execute and debug code
    "file-operations",    // Read, write, search, patch files
    "browser-automation", // Browser control
    "vision-analysis",    // Image understanding
    "media-generation",   // Generate images/video/audio
    // Management skills
    "skill-management",   // CRUD and version management
    "memory-ops",         // Memory store and retrieve
    "task-management",    // Todo and planning
    "communication",      // Messages and clarification
    "delegation",         // Delegate to sub-agents
    "security-audit",     // Security scanning
    // Development skills
    "code-review",        // Review and quality check
    "testing",            // Test generation and execution
    "planning",           // Project planning and breakdown
    "documentation",      // Doc generation
];

#[cfg(test)]
mod tests {
    use super::*;
    use hermes_core::{AgentError, SkillMeta};

    struct MockSkillProvider;

    #[async_trait]
    impl SkillProvider for MockSkillProvider {
        async fn create_skill(
            &self,
            name: &str,
            content: &str,
            _category: Option<&str>,
        ) -> Result<Skill, AgentError> {
            Ok(Skill::minimal(name, content))
        }
        async fn get_skill(&self, name: &str) -> Result<Option<Skill>, AgentError> {
            Ok(Some(Skill::minimal(name, "test content")))
        }
        async fn list_skills(&self) -> Result<Vec<SkillMeta>, AgentError> {
            Ok(vec![])
        }
        async fn update_skill(&self, name: &str, content: &str) -> Result<Skill, AgentError> {
            Ok(Skill::minimal(name, content))
        }
        async fn delete_skill(&self, _name: &str) -> Result<(), AgentError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_invoke_skill_schema() {
        let handler = InvokeSkillHandler::new(Arc::new(MockSkillProvider));
        let schema = handler.schema();
        assert_eq!(schema.name, "invoke_skill");
    }

    #[tokio::test]
    async fn test_invoke_skill_missing_name() {
        let handler = InvokeSkillHandler::new(Arc::new(MockSkillProvider));
        let result = handler.execute(json!({"parameters": {}})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_skill() {
        let handler = CreateSkillHandler::new(Arc::new(MockSkillProvider));
        let result = handler
            .execute(json!({
                "name": "my-task",
                "description": "Custom task",
                "natural_spec": "Do X then Y"
            }))
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_core_skills_count() {
        // Should be 12-16 core skills
        assert!(CORE_SKILLS.len() >= 12);
        assert!(CORE_SKILLS.len() <= 16);
    }
}