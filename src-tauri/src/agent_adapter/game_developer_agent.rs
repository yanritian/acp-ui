// Game Developer Agent - AI-powered game code generation
//
// Phase 3: Game Development Agents
// Provides intelligent code generation for different game engines

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics},
    health_tracker::HealthTracker,
    game_designer_agent::{GameDesignDoc, GameEngine},
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Game Developer Agent
pub struct GameDeveloperAgent {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    model: String,
}

/// Code generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenRequest {
    pub gdd: GameDesignDoc,
    pub output_format: CodeOutputFormat,
    pub target_files: Option<Vec<String>>,
}

/// Code output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeOutputFormat {
    SingleFile,
    MultipleFiles,
    ProjectStructure,
}

/// Generated code result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub files: Vec<CodeFile>,
    pub total_lines: u32,
    pub language: String,
    pub engine: GameEngine,
}

/// Single code file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFile {
    pub path: String,
    pub content: String,
    pub language: String,
}

impl GameDeveloperAgent {
    pub fn new() -> Self {
        Self {
            id: "game-developer".to_string(),
            name: "Game Developer Agent".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            model: "claude-opus-4-7".to_string(),
        }
    }

    pub fn with_model(model: String) -> Self {
        Self {
            id: "game-developer".to_string(),
            name: "Game Developer Agent".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            model,
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "code-generation".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.01,
                latency_ms: 10000,
            },
            Capability {
                name: "renpy-script".to_string(),
                proficiency: 0.95,
                cost_per_unit: 0.01,
                latency_ms: 5000,
            },
            Capability {
                name: "gdscript".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.01,
                latency_ms: 8000,
            },
            Capability {
                name: "csharp-unity".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.01,
                latency_ms: 8000,
            },
            Capability {
                name: "cpp-unreal".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.01,
                latency_ms: 12000,
            },
        ]
    }

    /// Generate code from GDD
    pub async fn generate_code(&self, request: &CodeGenRequest) -> Result<GeneratedCode, AgentError> {
        match request.gdd.technical_architecture.engine {
            GameEngine::RenPy => self.generate_renpy_code(&request.gdd).await,
            GameEngine::Godot => self.generate_godot_code(&request.gdd).await,
            GameEngine::Unity => self.generate_unity_code(&request.gdd).await,
            GameEngine::Unreal => self.generate_unreal_code(&request.gdd).await,
        }
    }

    /// Generate Ren'Py code
    async fn generate_renpy_code(&self, gdd: &GameDesignDoc) -> Result<GeneratedCode, AgentError> {
        let mut files = Vec::new();

        // Generate script.rpy
        let script_content = self.generate_renpy_script(gdd).await?;
        files.push(CodeFile {
            path: "game/script.rpy".to_string(),
            content: script_content,
            language: "renpy".to_string(),
        });

        // Generate characters.rpy
        let characters_content = self.generate_renpy_characters(gdd).await?;
        files.push(CodeFile {
            path: "game/characters.rpy".to_string(),
            content: characters_content,
            language: "renpy".to_string(),
        });

        // Generate screens.rpy
        let screens_content = self.generate_renpy_screens(gdd).await?;
        files.push(CodeFile {
            path: "game/screens.rpy".to_string(),
            content: screens_content,
            language: "renpy".to_string(),
        });

        let total_lines: u32 = files.iter().map(|f| f.content.lines().count() as u32).sum();

        Ok(GeneratedCode {
            files,
            total_lines,
            language: "Ren'Py Script".to_string(),
            engine: GameEngine::RenPy,
        })
    }

    /// Generate Ren'Py script
    async fn generate_renpy_script(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut script = String::new();

        script.push_str(&format!("## {} - Auto-generated by AI Game Developer\n\n", gdd.concept.title));

        // Import characters
        script.push_str("# Import characters\n");
        script.push_str("label start:\n");
        script.push_str("    jump chapter1\n\n");

        // Generate chapters based on scenes
        for (index, scene) in gdd.technical_architecture.scenes.iter().enumerate() {
            if scene != "MainMenu" && scene != "Settings" {
                script.push_str(&format!("label {}:\n", scene.to_lowercase()));
                script.push_str(&format!("    # Scene: {}\n", scene));

                // Add some placeholder dialogue
                if let Some(mechanic) = gdd.core_mechanics.first() {
                    script.push_str(&format!("    \"Welcome to {} - {}\"\n\n", gdd.concept.title, mechanic.name));
                } else {
                    script.push_str(&format!("    \"Welcome to {}\"\n\n", gdd.concept.title));
                }

                // Jump to next scene or ending
                if index < gdd.technical_architecture.scenes.len() - 1 {
                    let next_scene = &gdd.technical_architecture.scenes[index + 1];
                    if next_scene != "MainMenu" && next_scene != "Settings" {
                        script.push_str(&format!("    jump {}\n\n", next_scene.to_lowercase()));
                    } else {
                        script.push_str("    return\n\n");
                    }
                } else {
                    script.push_str("    return\n\n");
                }
            }
        }

        Ok(script)
    }

    /// Generate Ren'Py characters
    async fn generate_renpy_characters(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str("## Character Definitions - Auto-generated\n\n");

        // Generate character definitions based on systems
        for system in &gdd.systems {
            if system.name.contains("Dialogue") || system.name.contains("Character") {
                for component in &system.components {
                    if component.contains("Character") {
                        let char_id = component.to_lowercase().replace("character", "char");
                        let char_name = component.clone();
                        content.push_str(&format!(
                            "define {} = Character(\"{}\", color=\"#c8ffc8\")\n",
                            char_id, char_name
                        ));
                    }
                }
            }
        }

        // Add default characters if none defined
        if !content.contains("define ") {
            content.push_str("define e = Character(\"Eileen\", color=\"#c8ffc8\")\n");
            content.push_str("define m = Character(\"Me\", color=\"#c8c8ff\")\n");
        }

        Ok(content)
    }

    /// Generate Ren'Py screens
    async fn generate_renpy_screens(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str("## Custom Screens - Auto-generated\n\n");

        // Generate main menu
        content.push_str("screen main_menu():\n");
        content.push_str("    tag menu\n");
        content.push_str("    \n");
        content.push_str("    add gui.main_menu_background\n");
        content.push_str("    \n");
        content.push_str("    vbox:\n");
        content.push_str("        xalign 0.5\n");
        content.push_str("        yalign 0.5\n");
        content.push_str("        spacing 20\n");
        content.push_str("        \n");
        content.push_str("        textbutton \"Start Game\" action Start()\n");
        content.push_str("        textbutton \"Load Game\" action ShowMenu(\"load\")\n");
        content.push_str("        textbutton \"Settings\" action ShowMenu(\"preferences\")\n");
        content.push_str("        textbutton \"Quit\" action Quit()\n\n");

        // Generate HUD if needed
        if gdd.systems.iter().any(|s| s.name.contains("UI") || s.name.contains("HUD")) {
            content.push_str("screen game_hud():\n");
            content.push_str("    zorder 100\n");
            content.push_str("    \n");
            content.push_str("    # Add HUD elements here\n");
            content.push_str("    text \"HUD\" xalign 0.5 yalign 0.1\n");
        }

        Ok(content)
    }

    /// Generate Godot code
    async fn generate_godot_code(&self, gdd: &GameDesignDoc) -> Result<GeneratedCode, AgentError> {
        let mut files = Vec::new();

        // Generate main scene script
        let main_script = self.generate_godot_main_script(gdd).await?;
        files.push(CodeFile {
            path: "res://Main.gd".to_string(),
            content: main_script,
            language: "gdscript".to_string(),
        });

        // Generate player script
        let player_script = self.generate_godot_player_script(gdd).await?;
        files.push(CodeFile {
            path: "res://Player.gd".to_string(),
            content: player_script,
            language: "gdscript".to_string(),
        });

        // Generate project.godot
        let project_config = self.generate_godot_project_config(gdd).await?;
        files.push(CodeFile {
            path: "project.godot".to_string(),
            content: project_config,
            language: "ini".to_string(),
        });

        let total_lines: u32 = files.iter().map(|f| f.content.lines().count() as u32).sum();

        Ok(GeneratedCode {
            files,
            total_lines,
            language: "GDScript".to_string(),
            engine: GameEngine::Godot,
        })
    }

    /// Generate Godot main script
    async fn generate_godot_main_script(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str(&format!("## {} - Main Scene Script\n", gdd.concept.title));
        content.push_str("extends Node\n\n");

        content.push_str("# Game state\n");
        content.push_str("var game_started = false\n");
        content.push_str("var score = 0\n\n");

        content.push_str("func _ready():\n");
        content.push_str("    print(\"Game initialized\")\n");
        content.push_str("    start_game()\n\n");

        content.push_str("func start_game():\n");
        content.push_str("    game_started = true\n");
        content.push_str("    print(\"Game started!\")\n\n");

        content.push_str("func _process(delta):\n");
        content.push_str("    if game_started:\n");
        content.push_str("        # Update game logic here\n");
        content.push_str("        pass\n");

        Ok(content)
    }

    /// Generate Godot player script
    async fn generate_godot_player_script(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str("extends CharacterBody2D\n\n");

        content.push_str("# Player properties\n");
        content.push_str("const SPEED = 300.0\n");
        content.push_str("const JUMP_VELOCITY = -400.0\n\n");

        content.push_str("# Get gravity from project settings\n");
        content.push_str("var gravity = ProjectSettings.get_setting(\"physics/2d/default_gravity\")\n\n");

        content.push_str("func _physics_process(delta):\n");
        content.push_str("    # Add gravity\n");
        content.push_str("    if not is_on_floor():\n");
        content.push_str("        velocity.y += gravity * delta\n\n");

        content.push_str("    # Handle jump\n");
        content.push_str("    if Input.is_action_just_pressed(\"ui_accept\") and is_on_floor():\n");
        content.push_str("        velocity.y = JUMP_VELOCITY\n\n");

        content.push_str("    # Get input direction\n");
        content.push_str("    var direction = Input.get_axis(\"ui_left\", \"ui_right\")\n");
        content.push_str("    if direction:\n");
        content.push_str("        velocity.x = direction * SPEED\n");
        content.push_str("    else:\n");
        content.push_str("        velocity.x = move_toward(velocity.x, 0, SPEED)\n\n");

        content.push_str("    move_and_slide()\n");

        Ok(content)
    }

    /// Generate Godot project config
    async fn generate_godot_project_config(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str("; Engine configuration file.\n");
        content.push_str("; Auto-generated by AI Game Developer\n\n");

        content.push_str("[application]\n\n");
        content.push_str(&format!("config/name=\"{}\"\n", gdd.concept.title));
        content.push_str("run/main_scene=\"res://Main.tscn\"\n");
        content.push_str("config/features=PackedStringArray(\"4.2\")\n\n");

        content.push_str("[display]\n\n");
        content.push_str("window/size/viewport_width=1920\n");
        content.push_str("window/size/viewport_height=1080\n\n");

        content.push_str("[input]\n\n");
        content.push_str("move_left={\n");
        content.push_str("\"deadzone\": 0.5,\n");
        content.push_str("\"events\": [Object(InputEventKey,\"resource_local_to_scene\":false,\"resource_name\":\"\",\"device\":0,\"window_id\":0,\"alt_pressed\":false,\"shift_pressed\":false,\"ctrl_pressed\":false,\"meta_pressed\":false,\"pressed\":false,\"keycode\":0,\"physical_keycode\":65,\"key_label\":0,\"unicode\":97,\"location\":0,\"echo\":false,\"script\":null)]\n");
        content.push_str("}\n");
        content.push_str("move_right={\n");
        content.push_str("\"deadzone\": 0.5,\n");
        content.push_str("\"events\": [Object(InputEventKey,\"resource_local_to_scene\":false,\"resource_name\":\"\",\"device\":0,\"window_id\":0,\"alt_pressed\":false,\"shift_pressed\":false,\"ctrl_pressed\":false,\"meta_pressed\":false,\"pressed\":false,\"keycode\":0,\"physical_keycode\":68,\"key_label\":0,\"unicode\":100,\"location\":0,\"echo\":false,\"script\":null)]\n");
        content.push_str("}\n");

        Ok(content)
    }

    /// Generate Unity code
    async fn generate_unity_code(&self, gdd: &GameDesignDoc) -> Result<GeneratedCode, AgentError> {
        let mut files = Vec::new();

        // Generate GameManager.cs
        let game_manager = self.generate_unity_game_manager(gdd).await?;
        files.push(CodeFile {
            path: "Assets/Scripts/GameManager.cs".to_string(),
            content: game_manager,
            language: "csharp".to_string(),
        });

        // Generate PlayerController.cs
        let player_controller = self.generate_unity_player_controller(gdd).await?;
        files.push(CodeFile {
            path: "Assets/Scripts/PlayerController.cs".to_string(),
            content: player_controller,
            language: "csharp".to_string(),
        });

        let total_lines: u32 = files.iter().map(|f| f.content.lines().count() as u32).sum();

        Ok(GeneratedCode {
            files,
            total_lines,
            language: "C#".to_string(),
            engine: GameEngine::Unity,
        })
    }

    /// Generate Unity GameManager
    async fn generate_unity_game_manager(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str(&format!("// {} - Game Manager\n", gdd.concept.title));
        content.push_str("using UnityEngine;\n\n");
        content.push_str("public class GameManager : MonoBehaviour\n");
        content.push_str("{\n");
        content.push_str("    public static GameManager Instance { get; private set; }\n\n");
        content.push_str("    [SerializeField] private bool gameStarted = false;\n");
        content.push_str("    [SerializeField] private int score = 0;\n\n");
        content.push_str("    private void Awake()\n");
        content.push_str("    {\n");
        content.push_str("        if (Instance == null)\n");
        content.push_str("        {\n");
        content.push_str("            Instance = this;\n");
        content.push_str("            DontDestroyOnLoad(gameObject);\n");
        content.push_str("        }\n");
        content.push_str("        else\n");
        content.push_str("        {\n");
        content.push_str("            Destroy(gameObject);\n");
        content.push_str("        }\n");
        content.push_str("    }\n\n");
        content.push_str("    private void Start()\n");
        content.push_str("    {\n");
        content.push_str("        Debug.Log(\"Game initialized\");\n");
        content.push_str("        StartGame();\n");
        content.push_str("    }\n\n");
        content.push_str("    public void StartGame()\n");
        content.push_str("    {\n");
        content.push_str("        gameStarted = true;\n");
        content.push_str("        Debug.Log(\"Game started!\");\n");
        content.push_str("    }\n\n");
        content.push_str("    private void Update()\n");
        content.push_str("    {\n");
        content.push_str("        if (gameStarted)\n");
        content.push_str("        {\n");
        content.push_str("            // Update game logic here\n");
        content.push_str("        }\n");
        content.push_str("    }\n");
        content.push_str("}\n");

        Ok(content)
    }

    /// Generate Unity PlayerController
    async fn generate_unity_player_controller(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str("using UnityEngine;\n\n");
        content.push_str("[RequireComponent(typeof(CharacterController))]\n");
        content.push_str("public class PlayerController : MonoBehaviour\n");
        content.push_str("{\n");
        content.push_str("    [SerializeField] private float moveSpeed = 5f;\n");
        content.push_str("    [SerializeField] private float jumpHeight = 2f;\n");
        content.push_str("    [SerializeField] private float gravity = -9.81f;\n\n");
        content.push_str("    private CharacterController controller;\n");
        content.push_str("    private Vector3 velocity;\n");
        content.push_str("    private bool isGrounded;\n\n");
        content.push_str("    private void Start()\n");
        content.push_str("    {\n");
        content.push_str("        controller = GetComponent<CharacterController>();\n");
        content.push_str("    }\n\n");
        content.push_str("    private void Update()\n");
        content.push_str("    {\n");
        content.push_str("        isGrounded = controller.isGrounded;\n\n");
        content.push_str("        // Get input\n");
        content.push_str("        float horizontal = Input.GetAxis(\"Horizontal\");\n");
        content.push_str("        float vertical = Input.GetAxis(\"Vertical\");\n\n");
        content.push_str("        // Calculate movement\n");
        content.push_str("        Vector3 move = transform.right * horizontal + transform.forward * vertical;\n");
        content.push_str("        controller.Move(move * moveSpeed * Time.deltaTime);\n\n");
        content.push_str("        // Handle jump\n");
        content.push_str("        if (Input.GetButtonDown(\"Jump\") && isGrounded)\n");
        content.push_str("        {\n");
        content.push_str("            velocity.y = Mathf.Sqrt(jumpHeight * -2f * gravity);\n");
        content.push_str("        }\n\n");
        content.push_str("        // Apply gravity\n");
        content.push_str("        velocity.y += gravity * Time.deltaTime;\n");
        content.push_str("        controller.Move(velocity * Time.deltaTime);\n");
        content.push_str("    }\n");
        content.push_str("}\n");

        Ok(content)
    }

    /// Generate Unreal code
    async fn generate_unreal_code(&self, gdd: &GameDesignDoc) -> Result<GeneratedCode, AgentError> {
        let mut files = Vec::new();

        // Generate GameMode.h
        let gamemode_header = self.generate_unreal_gamemode_header(gdd).await?;
        files.push(CodeFile {
            path: "Source/MyGame/GameMode.h".to_string(),
            content: gamemode_header,
            language: "cpp".to_string(),
        });

        // Generate GameMode.cpp
        let gamemode_cpp = self.generate_unreal_gamemode_cpp(gdd).await?;
        files.push(CodeFile {
            path: "Source/MyGame/GameMode.cpp".to_string(),
            content: gamemode_cpp,
            language: "cpp".to_string(),
        });

        let total_lines: u32 = files.iter().map(|f| f.content.lines().count() as u32).sum();

        Ok(GeneratedCode {
            files,
            total_lines,
            language: "C++".to_string(),
            engine: GameEngine::Unreal,
        })
    }

    /// Generate Unreal GameMode header
    async fn generate_unreal_gamemode_header(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str(&format!("// {} - Game Mode Header\n", gdd.concept.title));
        content.push_str("#pragma once\n\n");
        content.push_str("#include \"CoreMinimal.h\"\n");
        content.push_str("#include \"GameFramework/GameModeBase.h\"\n");
        content.push_str("#include \"GameMode.generated.h\"\n\n");
        content.push_str("UCLASS()\n");
        content.push_str("class AGameMode : public AGameModeBase\n");
        content.push_str("{\n");
        content.push_str("    GENERATED_BODY()\n\n");
        content.push_str("public:\n");
        content.push_str("    AGameMode();\n\n");
        content.push_str("    virtual void BeginPlay() override;\n\n");
        content.push_str("    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = \"Game\")\n");
        content.push_str("    bool bGameStarted;\n\n");
        content.push_str("    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = \"Game\")\n");
        content.push_str("    int32 Score;\n\n");
        content.push_str("    UFUNCTION(BlueprintCallable, Category = \"Game\")\n");
        content.push_str("    void StartGame();\n");
        content.push_str("};\n");

        Ok(content)
    }

    /// Generate Unreal GameMode cpp
    async fn generate_unreal_gamemode_cpp(&self, gdd: &GameDesignDoc) -> Result<String, AgentError> {
        let mut content = String::new();

        content.push_str(&format!("// {} - Game Mode Implementation\n", gdd.concept.title));
        content.push_str("#include \"GameMode.h\"\n\n");
        content.push_str("AGameMode::AGameMode()\n");
        content.push_str("{\n");
        content.push_str("    bGameStarted = false;\n");
        content.push_str("    Score = 0;\n");
        content.push_str("}\n\n");
        content.push_str("void AGameMode::BeginPlay()\n");
        content.push_str("{\n");
        content.push_str("    Super::BeginPlay();\n");
        content.push_str("    UE_LOG(LogTemp, Log, TEXT(\"Game initialized\"));\n");
        content.push_str("    StartGame();\n");
        content.push_str("}\n\n");
        content.push_str("void AGameMode::StartGame()\n");
        content.push_str("{\n");
        content.push_str("    bGameStarted = true;\n");
        content.push_str("    UE_LOG(LogTemp, Log, TEXT(\"Game started!\"));\n");
        content.push_str("}\n");

        Ok(content)
    }
}

impl Default for GameDeveloperAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for GameDeveloperAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Api
    }

    fn capabilities(&self) -> Vec<Capability> {
        Self::get_capabilities()
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        self.config = Some(config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // Parse task description as code generation request
        // For now, create a simple GDD from task description
        let gdd = GameDesignDoc {
            concept: crate::agent_adapter::game_designer_agent::GameConcept {
                title: task.description.clone(),
                genre: "Visual Novel".to_string(),
                core_idea: task.description.clone(),
                target_audience: "General".to_string(),
                estimated_scope: crate::agent_adapter::game_designer_agent::ScopeLevel::Medium,
            },
            core_mechanics: vec![],
            systems: vec![],
            asset_requirements: vec![],
            technical_architecture: crate::agent_adapter::game_designer_agent::TechnicalArchitecture {
                engine: GameEngine::RenPy,
                programming_language: "Ren'Py Script".to_string(),
                project_type: "Visual Novel".to_string(),
                scenes: vec!["MainMenu".to_string(), "Game".to_string()],
                networking: crate::agent_adapter::game_designer_agent::NetworkingType::SinglePlayer,
            },
            development_phases: vec![],
        };

        let request = CodeGenRequest {
            gdd,
            output_format: CodeOutputFormat::MultipleFiles,
            target_files: None,
        };

        let generated_code = self.generate_code(&request).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Serialize generated code
        let code_json = serde_json::to_string_pretty(&generated_code)
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to serialize code: {}", e),
                retryable: false,
            })?;

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(code_json),
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
                ("engine".to_string(), format!("{:?}", generated_code.engine)),
                ("total_lines".to_string(), generated_code.total_lines.to_string()),
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
            min_cost: 0.05,
            max_cost: 0.15,
            currency: "USD".to_string(),
            breakdown: HashMap::from([
                ("claude_api".to_string(), 0.10),
            ]),
            token_estimate: crate::agent_adapter::types::TokenEstimate {
                input_tokens: 1000,
                output_tokens: 3000,
                total_tokens: 4000,
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
    fn test_game_developer_agent_new() {
        let agent = GameDeveloperAgent::new();
        assert_eq!(agent.id(), "game-developer");
        assert_eq!(agent.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_capabilities() {
        let agent = GameDeveloperAgent::new();
        let caps = agent.capabilities();
        assert!(caps.iter().any(|c| c.name == "code-generation"));
        assert!(caps.iter().any(|c| c.name == "renpy-script"));
    }
}
