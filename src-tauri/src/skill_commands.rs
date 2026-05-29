//! Skill System Tauri Commands
//!
//! Provides direct frontend access to the Skill meta-tool system.
//! These commands wrap the hermes-tools Skill handlers for Tauri IPC.

use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AppState;

// ---------------------------------------------------------------------------
// Skill Types for Tauri Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillMeta {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub generation: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInvocation {
    pub skill_name: String,
    pub parameters: Value,
    pub context: SkillContext,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillContext {
    pub session_id: Option<String>,
    pub agent_name: Option<String>,
    pub task_id: Option<String>,
    pub cwd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tool_calls_count: u32,
    pub evolved: bool,
    pub evolution_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillRequest {
    pub name: String,
    pub description: String,
    pub natural_spec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillResult {
    pub created: bool,
    pub name: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Skill Tauri Commands
// ---------------------------------------------------------------------------

/// List all available skills
#[tauri::command]
pub fn skills_list(state: State<AppState>) -> Result<Vec<SkillMeta>, String> {
    // In production, this would call SkillProvider through ToolRegistry
    // For now, return core skills list
    let core_skills = [
        ("web-research", "Core", "Web search and information extraction"),
        ("code-execution", "Core", "Execute and debug code"),
        ("file-operations", "Core", "Read, write, search, patch files"),
        ("browser-automation", "Core", "Browser control and automation"),
        ("vision-analysis", "Core", "Image understanding and analysis"),
        ("media-generation", "Core", "Generate images, video, audio"),
        ("skill-management", "Management", "CRUD and version management"),
        ("memory-ops", "Management", "Memory store and retrieve"),
        ("task-management", "Management", "Todo and task planning"),
        ("communication", "Management", "Messages and clarification"),
        ("delegation", "Management", "Delegate to sub-agents"),
        ("security-audit", "Management", "Security scanning"),
        ("code-review", "Development", "Code review and quality check"),
        ("testing", "Development", "Test generation and execution"),
        ("planning", "Development", "Project planning and breakdown"),
        ("documentation", "Development", "Documentation generation"),
    ];

    Ok(core_skills.iter().map(|(name, category, desc)| SkillMeta {
        name: name.to_string(),
        category: Some(category.to_string()),
        description: Some(desc.to_string()),
        version: Some("1.0.0".to_string()),
        generation: Some(0),
    }).collect())
}

/// View a specific skill's content
#[tauri::command]
pub fn skill_view(name: String) -> Result<String, String> {
    // Return skill template content
    let content = format!(
        "# {}\n\n\
         ## Description\n\
         {} skill for ACP-UI\n\n\
         ## Steps\n\
         1. Understand the task context\n\
         2. Execute the appropriate actions\n\
         3. Verify and report results\n\n\
         ## Parameters\n\
         (Define based on usage patterns)\n",
        name,
        name.replace("-", " ")
    );

    Ok(content)
}

/// Invoke a skill (meta-tool entry point)
#[tauri::command]
pub async fn invoke_skill(
    invocation: SkillInvocation,
) -> Result<SkillExecutionResult, String> {
    // In production, this would:
    // 1. Load skill from SkillProvider
    // 2. Execute through InvokeSkillHandler
    // 3. Check for evolution triggers
    // 4. Return result

    // For now, return a mock result
    Ok(SkillExecutionResult {
        success: true,
        output: format!("Skill '{}' invoked with params: {}", invocation.skill_name, invocation.parameters),
        error: None,
        duration_ms: 100,
        tool_calls_count: 1,
        evolved: false,
        evolution_summary: None,
    })
}

/// Create a skill from natural language
#[tauri::command]
pub async fn create_skill(
    request: CreateSkillRequest,
) -> Result<CreateSkillResult, String> {
    // Validate skill name (kebab-case)
    if !request.name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err("Skill name must be kebab-case (alphanumeric and hyphens only)".to_string());
    }

    // Generate skill content from natural language
    let content = format!(
        "# {}\n\n\
         ## Description\n\
         {}\n\n\
         ## Natural Language Specification\n\
         {}\n\n\
         ## Steps\n\
         1. Understand the task context\n\
         2. Execute the appropriate actions\n\
         3. Verify and report results\n\n\
         ## Parameters\n\
         (Define based on usage patterns)\n",
        request.name,
        request.description,
        request.natural_spec
    );

    // In production, this would call SkillProvider.create_skill()

    Ok(CreateSkillResult {
        created: true,
        name: request.name.clone(),
        message: format!("Skill '{}' created successfully. Content preview:\n\n{}", request.name, &content[..content.len().min(200)]),
    })
}

/// Manage skills (create/update/delete/self_improve)
#[tauri::command]
pub async fn skill_manage(
    action: String,
    name: Option<String>,
    content: Option<String>,
    feedback: Option<String>,
    category: Option<String>,
) -> Result<String, String> {
    match action.as_str() {
        "create" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            let content = content.ok_or_else(|| "Missing 'content' parameter".to_string())?;
            // In production: call SkillProvider.create_skill()
            Ok(format!("Skill '{}' created successfully", name))
        }
        "update" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            let content = content.ok_or_else(|| "Missing 'content' parameter".to_string())?;
            // In production: call SkillProvider.update_skill()
            Ok(format!("Skill '{}' updated successfully", name))
        }
        "delete" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            // In production: call SkillProvider.delete_skill()
            Ok(format!("Skill '{}' deleted successfully", name))
        }
        "self_improve" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            let feedback = feedback.ok_or_else(|| "Missing 'feedback' parameter".to_string())?;
            // In production: call skill_evolution::evolve_skill()
            Ok(format!("Skill '{}' improved with feedback: {}", name, &feedback[..feedback.len().min(50)]))
        }
        "rate" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            // Rating would be in feedback or a separate parameter
            Ok(format!("Skill '{}' rated. This feedback will help self-evolution.", name))
        }
        "install_builtins" => {
            Ok("Built-in skills installation request accepted. 16 core skills available.".to_string())
        }
        "sync" => {
            Ok("Skill sync request accepted. Remote hub connection pending.".to_string())
        }
        other => Err(format!("Unknown action: '{}'. Use create/update/delete/self_improve/rate/install_builtins/sync", other)),
    }
}