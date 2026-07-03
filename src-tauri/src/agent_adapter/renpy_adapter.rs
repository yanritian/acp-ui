// Ren'Py Game Adapter - Visual Novel / Galgame development support
//
// Phase 3: Game Development Agents
// Supports Ren'Py project creation, script generation, and game building

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskInput, TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics},
    health_tracker::HealthTracker,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Ren'Py Game Adapter
pub struct RenPyAdapter {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    renpy_path: PathBuf,
    project_path: Option<PathBuf>,
}

/// Ren'Py platform target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenPyPlatform {
    Windows,
    MacOS,
    Linux,
    Android,
    iOS,
    Web,
}

impl RenPyPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            RenPyPlatform::Windows => "win",
            RenPyPlatform::MacOS => "mac",
            RenPyPlatform::Linux => "linux",
            RenPyPlatform::Android => "android",
            RenPyPlatform::iOS => "ios",
            RenPyPlatform::Web => "web",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "win" | "windows" => Ok(RenPyPlatform::Windows),
            "mac" | "macos" => Ok(RenPyPlatform::MacOS),
            "linux" => Ok(RenPyPlatform::Linux),
            "android" => Ok(RenPyPlatform::Android),
            "ios" => Ok(RenPyPlatform::iOS),
            "web" | "webgl" => Ok(RenPyPlatform::Web),
            other => Err(format!("Unknown Ren'Py platform: {}", other)),
        }
    }
}

/// Story specification for script generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySpec {
    pub title: String,
    pub characters: Vec<CharacterSpec>,
    pub scenes: Vec<SceneSpec>,
    pub backgrounds: Vec<BackgroundSpec>,
}

/// Character specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSpec {
    pub id: String,
    pub name: String,
    pub color: String,
    pub expressions: Vec<ExpressionSpec>,
}

/// Expression specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionSpec {
    pub name: String,
    pub image_filename: String,
}

/// Scene specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSpec {
    pub label: String,
    pub background: Option<String>,
    pub dialogues: Vec<DialogueSpec>,
    pub menu: Option<MenuSpec>,
    pub next_label: Option<String>,
}

/// Dialogue specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueSpec {
    pub speaker: Option<String>,
    pub text: String,
    pub character_shows: Vec<CharacterShowSpec>,
}

/// Character show specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterShowSpec {
    pub character_id: String,
    pub expression: String,
    pub position: String,
}

/// Menu specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuSpec {
    pub prompt: Option<String>,
    pub choices: Vec<ChoiceSpec>,
}

/// Choice specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceSpec {
    pub text: String,
    pub target_label: String,
}

/// Background specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSpec {
    pub id: String,
    pub image_filename: String,
}

impl RenPyAdapter {
    pub fn new() -> Self {
        Self {
            id: "renpy-game".to_string(),
            name: "Ren'Py Game Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            renpy_path: PathBuf::from("D:/RenPy/renpy-8.5.3-sdk/renpy.exe"),
            project_path: None,
        }
    }

    pub fn with_renpy_path(renpy_path: PathBuf) -> Self {
        Self {
            id: "renpy-game".to_string(),
            name: "Ren'Py Game Adapter".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            renpy_path,
            project_path: None,
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "renpy-build".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.0,
                latency_ms: 60000,
            },
            Capability {
                name: "renpy-dev".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.0,
                latency_ms: 30000,
            },
            Capability {
                name: "renpy-script".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.002,
                latency_ms: 5000,
            },
            Capability {
                name: "renpy-scene".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.002,
                latency_ms: 8000,
            },
        ]
    }

    /// Find Ren'Py executable
    fn find_renpy_executable(&self) -> Result<PathBuf, AgentError> {
        // Check configured path first
        if self.renpy_path.exists() {
            return Ok(self.renpy_path.clone());
        }

        // Try common paths
        let common_paths = if cfg!(target_os = "windows") {
            vec![
                "D:/RenPy/renpy-8.5.3-sdk/renpy.exe",
                "C:/Program Files/RenPy/renpy.exe",
                "C:/RenPy/renpy.exe",
            ]
        } else if cfg!(target_os = "macos") {
            vec![
                "/Applications/RenPy/renpy.app/Contents/MacOS/renpy",
                "/usr/local/bin/renpy",
            ]
        } else {
            vec![
                "/usr/bin/renpy",
                "/usr/local/bin/renpy",
                "~/.local/bin/renpy",
            ]
        };

        for path in common_paths {
            let p = PathBuf::from(path);
            if p.exists() {
                return Ok(p);
            }
        }

        Err(AgentError::ConfigurationError {
            message: "Ren'Py not found. Please install Ren'Py SDK and configure the path.".to_string(),
        })
    }

    /// Detect Ren'Py project
    pub fn detect_renpy_project(cwd: &PathBuf) -> bool {
        // Check for project.json or game/script.rpy
        cwd.join("project.json").exists() || cwd.join("game/script.rpy").exists()
    }

    /// Get Ren'Py version
    pub async fn get_version(&self) -> Result<String, AgentError> {
        let renpy_exe = self.find_renpy_executable()?;

        let output = tokio::process::Command::new(&renpy_exe)
            .arg("--version")
            .output()
            .await
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to get Ren'Py version: {}", e),
                retryable: false,
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(AgentError::ExecutionError {
                message: format!("Failed to get version: {}", String::from_utf8_lossy(&output.stderr)),
                retryable: false,
            })
        }
    }

    /// Create a new Ren'Py project
    pub async fn create_project(&self, name: &str, base_path: &PathBuf) -> Result<PathBuf, AgentError> {
        let project_path = base_path.join(name);

        // Check if project already exists
        if project_path.exists() {
            return Err(AgentError::ConfigurationError {
                message: format!("Project already exists: {}", project_path.to_string_lossy()),
            });
        }

        // Create project by copying template
        let renpy_exe = self.find_renpy_executable()?;
        let renpy_sdk = renpy_exe.parent()
            .ok_or_else(|| AgentError::ConfigurationError {
                message: "Failed to get Ren'Py SDK path".to_string(),
            })?;

        let template_path = renpy_sdk.join("the_question");
        if !template_path.exists() {
            return Err(AgentError::ConfigurationError {
                message: "Ren'Py template project not found".to_string(),
            });
        }

        // Copy template
        self.copy_dir_recursive(&template_path, &project_path)
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to create project: {}", e),
                retryable: true,
            })?;

        // Update project configuration
        self.update_project_config(&project_path, name).await?;

        Ok(project_path)
    }

    /// Copy directory recursively
    fn copy_dir_recursive(&self, src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// Update project configuration
    async fn update_project_config(&self, project_path: &PathBuf, name: &str) -> Result<(), AgentError> {
        // Update project.json
        let project_json_path = project_path.join("project.json");
        if project_json_path.exists() {
            let content = std::fs::read_to_string(&project_json_path)
                .map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to read project.json: {}", e),
                    retryable: false,
                })?;

            let mut config: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to parse project.json: {}", e),
                    retryable: false,
                })?;

            config["display_name"] = serde_json::Value::String(name.to_string());

            let new_content = serde_json::to_string_pretty(&config)
                .map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to serialize project.json: {}", e),
                    retryable: false,
                })?;

            std::fs::write(&project_json_path, new_content)
                .map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to write project.json: {}", e),
                    retryable: false,
                })?;
        }

        // Update options.rpy
        let options_rpy_path = project_path.join("game/options.rpy");
        if options_rpy_path.exists() {
            let content = std::fs::read_to_string(&options_rpy_path)
                .map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to read options.rpy: {}", e),
                    retryable: false,
                })?;

            // Replace game name
            let new_content = content.replace(
                "define config.name = _(",
                &format!("define config.name = _(\"{}\")\n## OLD: define config.name = _(", name)
            );

            // Update save directory
            let save_dir = name.to_lowercase().replace(" ", "-");
            let new_content = new_content.replace(
                "define config.save_directory =",
                &format!("define config.save_directory = \"{}\"", save_dir)
            );

            std::fs::write(&options_rpy_path, new_content)
                .map_err(|e| AgentError::ExecutionError {
                    message: format!("Failed to write options.rpy: {}", e),
                    retryable: false,
                })?;
        }

        Ok(())
    }

    /// Generate Ren'Py script from story specification
    pub async fn generate_script(&self, spec: &StorySpec, project_path: &PathBuf) -> Result<String, AgentError> {
        // Validate specification
        self.validate_story_spec(spec)?;

        // Generate script content
        let mut script = String::new();

        // File header
        script.push_str(&format!("## {} - Auto-generated by AI\n\n", spec.title));

        // Character definitions
        script.push_str("## Character Definitions\n");
        for char in &spec.characters {
            script.push_str(&format!(
                "define {} = Character(\"{}\", color=\"{}\", image=\"{}\")\n",
                char.id, char.name, char.color, char.id
            ));
        }

        // Character expressions
        script.push_str("\n## Character Expressions\n");
        for char in &spec.characters {
            for expr in &char.expressions {
                script.push_str(&format!(
                    "image {} {} = \"characters/{}\"\n",
                    char.id, expr.name, expr.image_filename
                ));
            }
        }

        // Backgrounds
        script.push_str("\n## Backgrounds\n");
        for bg in &spec.backgrounds {
            script.push_str(&format!(
                "image {} = \"backgrounds/{}\"\n",
                bg.id, bg.image_filename
            ));
        }

        // Scenes
        script.push_str("\n## Scenes\n");
        for scene in &spec.scenes {
            script.push_str(&format!("\nlabel {}:\n", scene.label));

            // Background
            if let Some(bg) = &scene.background {
                script.push_str(&format!("    scene {} with fade\n", bg));
            }

            // Dialogues
            for dialogue in &scene.dialogues {
                // Character shows
                for char_show in &dialogue.character_shows {
                    script.push_str(&format!(
                        "    show {} {} at {}\n",
                        char_show.character_id, char_show.expression, char_show.position
                    ));
                }

                // Dialogue text
                if let Some(speaker) = &dialogue.speaker {
                    script.push_str(&format!("    {} \"{}\"\n", speaker, dialogue.text));
                } else {
                    script.push_str(&format!("    \"{}\"\n", dialogue.text));
                }
            }

            // Menu
            if let Some(menu) = &scene.menu {
                if let Some(prompt) = &menu.prompt {
                    script.push_str(&format!("    \"{}\"\n", prompt));
                }

                script.push_str("\n    menu:\n");
                for choice in &menu.choices {
                    script.push_str(&format!("        \"{}\":\n", choice.text));
                    script.push_str(&format!("            jump {}\n", choice.target_label));
                }
            }

            // Jump to next label
            if let Some(next) = &scene.next_label {
                script.push_str(&format!("    jump {}\n", next));
            }
        }

        // Write to file
        let script_path = project_path.join("game/script.rpy");
        std::fs::write(&script_path, &script)
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to write script.rpy: {}", e),
                retryable: false,
            })?;

        Ok(script)
    }

    /// Validate story specification
    fn validate_story_spec(&self, spec: &StorySpec) -> Result<(), AgentError> {
        // Check characters
        if spec.characters.is_empty() {
            return Err(AgentError::ConfigurationError {
                message: "Story must have at least one character".to_string(),
            });
        }

        // Check scenes
        if spec.scenes.is_empty() {
            return Err(AgentError::ConfigurationError {
                message: "Story must have at least one scene".to_string(),
            });
        }

        // Check start label exists
        if !spec.scenes.iter().any(|s| s.label == "start") {
            return Err(AgentError::ConfigurationError {
                message: "Story must have a 'start' label".to_string(),
            });
        }

        // Validate jump targets
        let labels: std::collections::HashSet<_> = spec.scenes.iter().map(|s| s.label.as_str()).collect();

        for scene in &spec.scenes {
            if let Some(next) = &scene.next_label {
                if !labels.contains(next.as_str()) {
                    return Err(AgentError::ConfigurationError {
                        message: format!("Scene '{}' jumps to non-existent label: {}", scene.label, next),
                    });
                }
            }

            if let Some(menu) = &scene.menu {
                for choice in &menu.choices {
                    if !labels.contains(choice.target_label.as_str()) {
                        return Err(AgentError::ConfigurationError {
                            message: format!("Choice in '{}' jumps to non-existent label: {}", scene.label, choice.target_label),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Run the game
    pub async fn run_game(&self, project_path: &PathBuf) -> Result<u32, AgentError> {
        let renpy_exe = self.find_renpy_executable()?;

        let child = tokio::process::Command::new(&renpy_exe)
            .arg(project_path.to_string_lossy().as_ref())
            .spawn()
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to launch game: {}", e),
                retryable: true,
            })?;

        Ok(child.id().unwrap_or(0))
    }

    /// Compile the game (check for errors)
    pub async fn compile_game(&self, project_path: &PathBuf) -> Result<String, AgentError> {
        let renpy_exe = self.find_renpy_executable()?;

        let output = tokio::process::Command::new(&renpy_exe)
            .arg(project_path.to_string_lossy().as_ref())
            .arg("compile")
            .output()
            .await
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to compile game: {}", e),
                retryable: true,
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AgentError::ExecutionError {
                message: format!("Compilation failed: {}", String::from_utf8_lossy(&output.stderr)),
                retryable: true,
            })
        }
    }

    /// Lint the game (check for issues)
    pub async fn lint_game(&self, project_path: &PathBuf) -> Result<String, AgentError> {
        let renpy_exe = self.find_renpy_executable()?;

        let output = tokio::process::Command::new(&renpy_exe)
            .arg(project_path.to_string_lossy().as_ref())
            .arg("lint")
            .output()
            .await
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to lint game: {}", e),
                retryable: true,
            })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(AgentError::ExecutionError {
                message: format!("Lint failed: {}", String::from_utf8_lossy(&output.stderr)),
                retryable: true,
            })
        }
    }
}

impl Default for RenPyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for RenPyAdapter {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Cli
    }

    fn capabilities(&self) -> Vec<Capability> {
        Self::get_capabilities()
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        // Extract Ren'Py path from config if provided
        if let Some(cwd) = &config.cwd {
            self.project_path = Some(PathBuf::from(cwd));
        }

        self.config = Some(config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        self.find_renpy_executable()?;
        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let config = self.config.as_ref().ok_or(AgentError::ConfigurationError {
            message: "Ren'Py Adapter not configured".to_string(),
        })?;

        let cwd = PathBuf::from(config.cwd.clone().unwrap_or_else(|| {
            std::env::current_dir().unwrap().to_string_lossy().to_string()
        }));

        let start = Instant::now();

        // Parse task description
        let desc_lower = task.description.to_lowercase();
        let is_create = desc_lower.contains("create") || desc_lower.contains("new");
        let is_generate = desc_lower.contains("generate") || desc_lower.contains("script");
        let is_run = desc_lower.contains("run") || desc_lower.contains("launch");
        let is_compile = desc_lower.contains("compile") || desc_lower.contains("build");
        let is_lint = desc_lower.contains("lint") || desc_lower.contains("check");

        let output = if is_create {
            // Create new project
            let name = match &task.input {
                TaskInput::Text(t) => t.clone(),
                _ => "NewGame".to_string(),
            };

            let project_path = self.create_project(&name, &cwd).await?;
            format!("Created Ren'Py project at: {}", project_path.to_string_lossy())
        } else if is_run {
            // Run game
            let pid = self.run_game(&cwd).await?;
            format!("Game launched with PID: {}", pid)
        } else if is_compile {
            // Compile game
            let result = self.compile_game(&cwd).await?;
            format!("Compilation successful:\n{}", result)
        } else if is_lint {
            // Lint game
            let result = self.lint_game(&cwd).await?;
            format!("Lint results:\n{}", result)
        } else {
            // Default: compile
            let result = self.compile_game(&cwd).await?;
            format!("Compiled:\n{}", result)
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(output),
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            cost: ActualCost {
                amount: 0.0,
                currency: "USD".to_string(),
                token_usage: TokenUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                },
            },
            duration_ms,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metadata: HashMap::from([
                ("cwd".to_string(), cwd.to_string_lossy().to_string()),
            ]),
        })
    }

    async fn cancel(&self, _task_id: &str) -> Result<(), AgentError> {
        Ok(())
    }

    fn status(&self) -> crate::agent_adapter::types::AgentStatus {
        crate::agent_adapter::types::AgentStatus::Idle
    }

    async fn health(&self) -> HealthMetrics {
        self.health_tracker.get_metrics()
    }

    fn cost_estimate(&self, _task: &AgentTask) -> crate::agent_adapter::types::CostEstimate {
        crate::agent_adapter::types::CostEstimate {
            min_cost: 0.0,
            max_cost: 0.0,
            currency: "USD".to_string(),
            breakdown: HashMap::new(),
            token_estimate: crate::agent_adapter::types::TokenEstimate {
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            },
        }
    }

    fn actual_cost(&self, _task_id: &str) -> Option<ActualCost> {
        None
    }

    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage> {
        self.token_history.iter().rev().take(last_n as usize).cloned().collect()
    }

    async fn update_health(&mut self, result: &AgentResult) {
        match result.status {
            ResultStatus::Success => {
                self.health_tracker.record_success(result.duration_ms);
            }
            _ => {
                self.health_tracker.record_error();
            }
        }
    }

    fn is_circuit_breaker_allowed(&self) -> bool {
        true
    }

    async fn reset_circuit_breaker(&mut self) {
        self.health_tracker.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renpy_adapter_new() {
        let adapter = RenPyAdapter::new();
        assert_eq!(adapter.id(), "renpy-game");
        assert_eq!(adapter.adapter_type(), AdapterType::Cli);
    }

    #[test]
    fn test_renpy_platform_from_str() {
        assert_eq!(RenPyPlatform::from_str("windows").unwrap(), RenPyPlatform::Windows);
        assert_eq!(RenPyPlatform::from_str("mac").unwrap(), RenPyPlatform::MacOS);
        assert_eq!(RenPyPlatform::from_str("linux").unwrap(), RenPyPlatform::Linux);
        assert!(RenPyPlatform::from_str("unknown").is_err());
    }

    #[test]
    fn test_renpy_platform_as_str() {
        assert_eq!(RenPyPlatform::Windows.as_str(), "win");
        assert_eq!(RenPyPlatform::MacOS.as_str(), "mac");
        assert_eq!(RenPyPlatform::Linux.as_str(), "linux");
    }

    #[test]
    fn test_capabilities() {
        let adapter = RenPyAdapter::new();
        let caps = adapter.capabilities();
        assert!(caps.iter().any(|c| c.name == "renpy-build"));
        assert!(caps.iter().any(|c| c.name == "renpy-script"));
    }

    #[test]
    fn test_detect_renpy_project() {
        // Test with non-project directory
        let cwd = PathBuf::from(".");
        assert!(!RenPyAdapter::detect_renpy_project(&cwd));
    }

    #[test]
    fn test_validate_story_spec_empty_characters() {
        let adapter = RenPyAdapter::new();
        let spec = StorySpec {
            title: "Test".to_string(),
            characters: vec![],
            scenes: vec![SceneSpec {
                label: "start".to_string(),
                background: None,
                dialogues: vec![],
                menu: None,
                next_label: None,
            }],
            backgrounds: vec![],
        };

        assert!(adapter.validate_story_spec(&spec).is_err());
    }

    #[test]
    fn test_validate_story_spec_no_start() {
        let adapter = RenPyAdapter::new();
        let spec = StorySpec {
            title: "Test".to_string(),
            characters: vec![CharacterSpec {
                id: "e".to_string(),
                name: "Eileen".to_string(),
                color: "#ffffff".to_string(),
                expressions: vec![],
            }],
            scenes: vec![SceneSpec {
                label: "chapter1".to_string(),
                background: None,
                dialogues: vec![],
                menu: None,
                next_label: None,
            }],
            backgrounds: vec![],
        };

        assert!(adapter.validate_story_spec(&spec).is_err());
    }
}
