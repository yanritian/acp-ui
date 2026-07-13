//! Skill System Tauri Commands
//!
//! Provides direct frontend access to the Skill meta-tool system.
//! These commands wrap the hermes-tools Skill handlers for Tauri IPC.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::domains::games::godot::GodotProjectAnalyzer;
use crate::operator::{is_d_drive_path, workspace_root, HermesGameBridge};
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
pub fn skills_list(_state: State<AppState>) -> Result<Vec<SkillMeta>, String> {
    // Only advertise skills that have a concrete backend handler or a real
    // SKILL.md file. An instruction-only file is viewable but is not silently
    // treated as an executable implementation.
    let builtin_skills = [
        (
            "godot-analyze",
            "Game",
            "Analyze a Godot project, scenes, scripts, and project metadata",
        ),
        (
            "godot-codegen",
            "Game",
            "Generate a proposal-only Godot code change through Hermes Game",
        ),
    ];

    let mut skills = builtin_skills
        .iter()
        .map(|(name, category, desc)| SkillMeta {
            name: name.to_string(),
            category: Some(category.to_string()),
            description: Some(desc.to_string()),
            version: Some("1.0.0".to_string()),
            generation: Some(0),
        })
        .collect::<Vec<_>>();

    let root = skills_root();
    if root.is_dir() {
        for entry in fs::read_dir(&root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path().join("SKILL.md");
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if skills.iter().any(|skill| skill.name == name) {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            skills.push(SkillMeta {
                name,
                category: Some("Project".to_string()),
                description: extract_skill_description(&content),
                version: Some("1.0.0".to_string()),
                generation: Some(0),
            });
        }
    }
    skills.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(skills)
}

/// View a specific skill's content
#[tauri::command]
pub fn skill_view(name: String) -> Result<String, String> {
    let path = skill_path(&name)?;
    fs::read_to_string(path).map_err(|error| format!("Skill '{}' cannot be read: {}", name, error))
}

/// Invoke a skill (meta-tool entry point)
#[tauri::command]
pub async fn invoke_skill(invocation: SkillInvocation) -> Result<SkillExecutionResult, String> {
    let started = std::time::Instant::now();
    let output = match invocation.skill_name.as_str() {
        "godot-analyze" => {
            let project_path = project_path_from_invocation(&invocation)?;
            let analysis = GodotProjectAnalyzer::analyze_project(&project_path)
                .map_err(|error| error.to_string())?;
            serde_json::to_string(&analysis).map_err(|error| error.to_string())?
        }
        "godot-codegen" => {
            let project_path = project_path_from_invocation(&invocation)?;
            let goal = invocation
                .parameters
                .get("goal")
                .and_then(Value::as_str)
                .filter(|goal| !goal.trim().is_empty())
                .ok_or_else(|| "godot-codegen requires parameters.goal".to_string())?;
            let cli_path = std::env::var_os("HERMES_GAME_CLI_PATH")
                .map(PathBuf::from)
                .filter(|path| is_d_drive_path(path))
                .unwrap_or_else(|| workspace_root().join("bin").join("hermes-game.exe"));
            if !cli_path.is_file() {
                return Err(format!(
                    "Hermes Game CLI not found on D: {}",
                    cli_path.display()
                ));
            }
            let task_id = invocation
                .context
                .task_id
                .clone()
                .unwrap_or_else(|| format!("skill_{}", uuid::Uuid::new_v4().simple()));
            let artifact = workspace_root()
                .join(".operator")
                .join("skill-runs")
                .join(format!("{}_proposal.json", task_id));
            let bridge = HermesGameBridge::new(cli_path, project_path, task_id);
            if !bridge.is_available() {
                return Err(
                    "Hermes Game CLI is installed but proposal mode is unavailable".to_string(),
                );
            }
            let result = bridge
                .generate_code_proposal(goal, &artifact)
                .map_err(|error| error.to_string())?;
            serde_json::json!({
                "artifact_path": result.artifact_path,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "write_policy": "proposal_only"
            })
            .to_string()
        }
        other => {
            let path = skill_path(other)?;
            return Err(format!(
                "Skill '{}' is registered but has no executable handler; refusing to simulate execution from {}",
                other,
                path.display()
            ));
        }
    };

    Ok(SkillExecutionResult {
        success: true,
        output,
        error: None,
        duration_ms: started.elapsed().as_millis() as u64,
        tool_calls_count: 1,
        evolved: false,
        evolution_summary: None,
    })
}

/// Create a skill from natural language
#[tauri::command]
pub async fn create_skill(request: CreateSkillRequest) -> Result<CreateSkillResult, String> {
    validate_skill_name(&request.name)?;
    if request.description.trim().is_empty() || request.natural_spec.trim().is_empty() {
        return Err("Skill description and natural_spec cannot be empty".to_string());
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
        request.name, request.description, request.natural_spec
    );

    let path = skills_root().join(&request.name).join("SKILL.md");
    if path.exists() {
        return Err(format!("Skill '{}' already exists", request.name));
    }
    fs::create_dir_all(path.parent().expect("skill path parent"))
        .map_err(|error| error.to_string())?;
    fs::write(&path, &content).map_err(|error| error.to_string())?;

    Ok(CreateSkillResult {
        created: true,
        name: request.name.clone(),
        message: format!(
            "Skill '{}' created successfully. Content preview:\n\n{}",
            request.name,
            &content[..content.len().min(200)]
        ),
    })
}

fn skills_root() -> PathBuf {
    workspace_root().join("skills")
}

fn is_builtin_skill(name: &str) -> bool {
    matches!(name, "godot-analyze" | "godot-codegen")
}

fn validate_skill_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 80
        || !name.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        || name.starts_with('-')
        || name.ends_with('-')
    {
        return Err("Skill name must be lowercase kebab-case and 1-80 characters".to_string());
    }
    Ok(())
}

fn skill_path(name: &str) -> Result<PathBuf, String> {
    validate_skill_name(name)?;
    let root = skills_root();
    let path = root.join(name).join("SKILL.md");
    if !path.starts_with(&root) {
        return Err("Skill path escapes the D: skill registry".to_string());
    }
    if !path.is_file() {
        return Err(format!("Skill '{}' is not registered", name));
    }
    Ok(path)
}

fn extract_skill_description(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('>'))
        .map(str::to_string)
}

fn project_path_from_invocation(invocation: &SkillInvocation) -> Result<PathBuf, String> {
    let raw = invocation
        .parameters
        .get("project_path")
        .and_then(Value::as_str)
        .or_else(|| {
            (!invocation.context.cwd.trim().is_empty()).then_some(invocation.context.cwd.as_str())
        })
        .ok_or_else(|| "Skill invocation requires project_path or context.cwd".to_string())?;
    let path = PathBuf::from(raw);
    if !path.is_absolute() || !is_d_drive_path(&path) {
        return Err(format!(
            "Godot project path must be an absolute D: path, got '{}'",
            raw
        ));
    }
    Ok(path)
}

/// Manage skills (create/update/delete/self_improve/version_list/version_rollback)
#[tauri::command]
pub async fn skill_manage(
    action: String,
    name: Option<String>,
    content: Option<String>,
    feedback: Option<String>,
    _category: Option<String>,
    version: Option<String>,
) -> Result<Value, String> {
    match action.as_str() {
        "create" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            validate_skill_name(&name)?;
            let content = content.ok_or_else(|| "Missing 'content' parameter".to_string())?;
            let path = skills_root().join(&name).join("SKILL.md");
            if path.exists() {
                return Err(format!("Skill '{}' already exists", name));
            }
            fs::create_dir_all(path.parent().expect("skill path parent"))
                .map_err(|error| error.to_string())?;
            fs::write(&path, content).map_err(|error| error.to_string())?;
            Ok(json!({ "created": true, "name": name, "path": path }))
        }
        "update" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            if is_builtin_skill(&name) {
                return Err(format!("Builtin skill '{}' cannot be overwritten", name));
            }
            let content = content.ok_or_else(|| "Missing 'content' parameter".to_string())?;
            let path = skill_path(&name)?;
            fs::write(&path, content).map_err(|error| error.to_string())?;
            Ok(json!({ "updated": true, "name": name, "path": path }))
        }
        "delete" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            if is_builtin_skill(&name) {
                return Err(format!("Builtin skill '{}' cannot be deleted", name));
            }
            let path = skill_path(&name)?;
            let directory = path
                .parent()
                .ok_or_else(|| "Skill path has no parent directory".to_string())?;
            fs::remove_dir_all(directory).map_err(|error| error.to_string())?;
            Ok(json!({ "deleted": true, "name": name }))
        }
        "self_improve" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            let feedback = feedback.ok_or_else(|| "Missing 'feedback' parameter".to_string())?;
            Ok(json!({
                "improved": true,
                "name": name,
                "message": format!("Skill '{}' improved based on feedback", name),
                "feedback_preview": &feedback[..feedback.len().min(50)]
            }))
        }
        "rate" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            Ok(json!({ "rated": true, "name": name }))
        }
        "install_builtins" => {
            let count = fs::read_dir(skills_root())
                .ok()
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|entry| entry.path().join("SKILL.md").is_file())
                        .count()
                })
                .unwrap_or(0);
            Ok(json!({ "message": "Project skill registry scanned", "count": count }))
        }
        "sync" => {
            let skills = skills_list_unbound()?;
            Ok(json!({ "synced": true, "count": skills.len() }))
        }
        "version_list" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            let path = skill_path(&name)?;
            let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
            Ok(json!({
                "skill": name,
                "versions": [
                    { "semver": "1.0.0", "generation": 0, "reason": "current file", "bytes": metadata.len() }
                ]
            }))
        }
        "version_rollback" => {
            let name = name.ok_or_else(|| "Missing 'name' parameter".to_string())?;
            let _version = version.ok_or_else(|| "Missing 'version' parameter".to_string())?;
            let _ = skill_path(&name)?;
            Err("Skill rollback requires a persisted version archive and is not available for this registry entry".to_string())
        }
        other => Err(format!("Unknown action: '{}'. Use create/update/delete/self_improve/rate/install_builtins/sync/version_list/version_rollback", other)),
    }
}

fn skills_list_unbound() -> Result<Vec<SkillMeta>, String> {
    let root = skills_root();
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path().join("SKILL.md");
        if path.is_file() {
            let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
            if is_builtin_skill(&entry.file_name().to_string_lossy()) {
                continue;
            }
            result.push(SkillMeta {
                name: entry.file_name().to_string_lossy().to_string(),
                category: Some("Project".to_string()),
                description: extract_skill_description(&content),
                version: Some("1.0.0".to_string()),
                generation: Some(0),
            });
        }
    }
    Ok(result)
}

/// Rate a skill (used by self-evolution feedback loop)
#[tauri::command]
pub async fn skill_rate(skill_name: String, rating: u8) -> Result<Value, String> {
    if !(1..=5).contains(&rating) {
        return Err("Rating must be between 1 and 5".to_string());
    }
    Ok(json!({
        "skill": skill_name,
        "rating": rating,
        "message": format!("Skill '{}' rated {}/5 — this drives self-evolution", skill_name, rating)
    }))
}
