// Game Designer Agent - AI-powered game design and planning
//
// Phase 3: Game Development Agents
// Provides intelligent game design, story generation, and system architecture

use crate::agent_adapter::{
    AgentAdapter,
    types::{AdapterType, Capability, AgentConfig, AgentTask, AgentResult, AgentError,
            TaskOutput, ResultStatus, ActualCost, TokenUsage, HealthMetrics},
    health_tracker::HealthTracker,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Game Designer Agent
pub struct GameDesignerAgent {
    id: String,
    name: String,
    config: Option<AgentConfig>,
    health_tracker: HealthTracker,
    token_history: Vec<TokenUsage>,
    model: String,  // Claude model to use
}

/// Game concept - high-level game idea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConcept {
    pub title: String,
    pub genre: String,
    pub core_idea: String,
    pub target_audience: String,
    pub estimated_scope: ScopeLevel,
}

/// Scope level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeLevel {
    Small,    // 1-2 weeks
    Medium,   // 1-2 months
    Large,    // 3+ months
}

/// Mechanic design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanicDesign {
    pub name: String,
    pub description: String,
    pub implementation_priority: Priority,
    pub code_hints: String,
}

/// Priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// System design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDesign {
    pub name: String,
    pub components: Vec<String>,
    pub interactions: Vec<String>,
    pub code_structure: String,
}

/// Asset requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRequirement {
    pub name: String,
    pub asset_type: AssetType,
    pub description: String,
    pub style: String,
    pub quantity: u32,
    pub specifications: String,
}

/// Asset type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Sprite2D,
    Model3D,
    Audio,
    Animation,
    Font,
    Texture,
}

/// Technical architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalArchitecture {
    pub engine: GameEngine,
    pub programming_language: String,
    pub project_type: String,
    pub scenes: Vec<String>,
    pub networking: NetworkingType,
}

/// Game engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEngine {
    RenPy,
    Godot,
    Unity,
    Unreal,
}

/// Networking type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkingType {
    SinglePlayer,
    LocalMultiplayer,
    OnlineMultiplayer,
}

/// Development phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentPhase {
    pub phase_name: String,
    pub tasks: Vec<String>,
    pub estimated_days: u32,
}

/// Complete Game Design Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDesignDoc {
    pub concept: GameConcept,
    pub core_mechanics: Vec<MechanicDesign>,
    pub systems: Vec<SystemDesign>,
    pub asset_requirements: Vec<AssetRequirement>,
    pub technical_architecture: TechnicalArchitecture,
    pub development_phases: Vec<DevelopmentPhase>,
}

/// Design request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignRequest {
    pub user_input: String,
    pub constraints: Option<DesignConstraints>,
}

/// Design constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignConstraints {
    pub preferred_engine: Option<GameEngine>,
    pub target_platform: Option<String>,
    pub max_development_time: Option<u32>,  // days
    pub team_size: Option<u32>,
}

impl GameDesignerAgent {
    pub fn new() -> Self {
        Self {
            id: "game-designer".to_string(),
            name: "Game Designer Agent".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            model: "claude-opus-4-7".to_string(),
        }
    }

    pub fn with_model(model: String) -> Self {
        Self {
            id: "game-designer".to_string(),
            name: "Game Designer Agent".to_string(),
            config: None,
            health_tracker: HealthTracker::new(),
            token_history: Vec::new(),
            model,
        }
    }

    fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "game-concept".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.01,
                latency_ms: 5000,
            },
            Capability {
                name: "mechanic-design".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.01,
                latency_ms: 8000,
            },
            Capability {
                name: "system-design".to_string(),
                proficiency: 0.80,
                cost_per_unit: 0.01,
                latency_ms: 10000,
            },
            Capability {
                name: "asset-planning".to_string(),
                proficiency: 0.85,
                cost_per_unit: 0.01,
                latency_ms: 6000,
            },
            Capability {
                name: "tech-architecture".to_string(),
                proficiency: 0.90,
                cost_per_unit: 0.01,
                latency_ms: 7000,
            },
        ]
    }

    /// Generate complete game design document from user input
    pub async fn generate_gdd(&self, request: &DesignRequest) -> Result<GameDesignDoc, AgentError> {
        // Step 1: Generate game concept
        let concept = self.generate_concept(&request.user_input).await?;

        // Step 2: Generate core mechanics
        let mechanics = self.generate_mechanics(&concept).await?;

        // Step 3: Generate systems
        let systems = self.generate_systems(&concept, &mechanics).await?;

        // Step 4: Generate asset requirements
        let assets = self.generate_assets(&concept, &mechanics, &systems).await?;

        // Step 5: Generate technical architecture
        let tech = self.generate_technical_architecture(&concept, request.constraints.as_ref()).await?;

        // Step 6: Generate development plan
        let phases = self.generate_development_plan(&mechanics, &systems, &assets).await?;

        Ok(GameDesignDoc {
            concept,
            core_mechanics: mechanics,
            systems,
            asset_requirements: assets,
            technical_architecture: tech,
            development_phases: phases,
        })
    }

    /// Generate game concept
    async fn generate_concept(&self, user_input: &str) -> Result<GameConcept, AgentError> {
        // In production, this would call Claude API
        // For now, return a template-based concept

        let concept = GameConcept {
            title: self.extract_title(user_input).unwrap_or_else(|| "Untitled Game".to_string()),
            genre: self.extract_genre(user_input).unwrap_or_else(|| "Visual Novel".to_string()),
            core_idea: user_input.to_string(),
            target_audience: "General audience".to_string(),
            estimated_scope: ScopeLevel::Medium,
        };

        Ok(concept)
    }

    /// Generate core mechanics
    async fn generate_mechanics(&self, concept: &GameConcept) -> Result<Vec<MechanicDesign>, AgentError> {
        let mut mechanics = Vec::new();

        // Generate mechanics based on genre
        match concept.genre.to_lowercase().as_str() {
            "visual novel" | "galgame" => {
                mechanics.push(MechanicDesign {
                    name: "Dialogue System".to_string(),
                    description: "Character dialogue with branching choices".to_string(),
                    implementation_priority: Priority::Critical,
                    code_hints: "Use menu: for choices, jump: for branching".to_string(),
                });
                mechanics.push(MechanicDesign {
                    name: "Character Affection".to_string(),
                    description: "Track relationship levels with characters".to_string(),
                    implementation_priority: Priority::High,
                    code_hints: "Use variables to track affection points".to_string(),
                });
                mechanics.push(MechanicDesign {
                    name: "Multiple Endings".to_string(),
                    description: "Different story endings based on choices".to_string(),
                    implementation_priority: Priority::High,
                    code_hints: "Use conditional jumps based on variables".to_string(),
                });
            }
            "fps" | "first person shooter" => {
                mechanics.push(MechanicDesign {
                    name: "Shooting".to_string(),
                    description: "First-person weapon shooting mechanics".to_string(),
                    implementation_priority: Priority::Critical,
                    code_hints: "Use raycasting for hit detection".to_string(),
                });
                mechanics.push(MechanicDesign {
                    name: "Movement".to_string(),
                    description: "WASD movement with mouse look".to_string(),
                    implementation_priority: Priority::Critical,
                    code_hints: "Use CharacterController or RigidBody".to_string(),
                });
                mechanics.push(MechanicDesign {
                    name: "Health System".to_string(),
                    description: "Player health and damage".to_string(),
                    implementation_priority: Priority::High,
                    code_hints: "Track HP, apply damage on hit".to_string(),
                });
            }
            "moba" => {
                mechanics.push(MechanicDesign {
                    name: "Hero Selection".to_string(),
                    description: "Choose from multiple heroes with unique abilities".to_string(),
                    implementation_priority: Priority::Critical,
                    code_hints: "Hero base class with virtual methods".to_string(),
                });
                mechanics.push(MechanicDesign {
                    name: "Skill System".to_string(),
                    description: "Each hero has 3-4 unique skills".to_string(),
                    implementation_priority: Priority::Critical,
                    code_hints: "Skill base class with cooldown management".to_string(),
                });
                mechanics.push(MechanicDesign {
                    name: "Minion Spawning".to_string(),
                    description: "Automatic minion waves".to_string(),
                    implementation_priority: Priority::High,
                    code_hints: "Timer-based spawning system".to_string(),
                });
            }
            _ => {
                mechanics.push(MechanicDesign {
                    name: "Core Gameplay".to_string(),
                    description: "Basic game mechanics".to_string(),
                    implementation_priority: Priority::Critical,
                    code_hints: "Implement core loop first".to_string(),
                });
            }
        }

        Ok(mechanics)
    }

    /// Generate systems
    async fn generate_systems(&self, concept: &GameConcept, mechanics: &[MechanicDesign]) -> Result<Vec<SystemDesign>, AgentError> {
        let mut systems = Vec::new();

        // Add common systems
        systems.push(SystemDesign {
            name: "UI System".to_string(),
            components: vec![
                "MainMenu".to_string(),
                "HUD".to_string(),
                "PauseMenu".to_string(),
                "SettingsMenu".to_string(),
            ],
            interactions: vec![
                "Menu navigation".to_string(),
                "HUD updates".to_string(),
                "Settings persistence".to_string(),
            ],
            code_structure: "Use UI Canvas with separate panels".to_string(),
        });

        systems.push(SystemDesign {
            name: "Save/Load System".to_string(),
            components: vec![
                "SaveManager".to_string(),
                "SaveSlot".to_string(),
                "AutoSave".to_string(),
            ],
            interactions: vec![
                "Serialize game state".to_string(),
                "Multiple save slots".to_string(),
                "Auto-save on milestones".to_string(),
            ],
            code_structure: "JSON serialization with version migration".to_string(),
        });

        // Add genre-specific systems
        match concept.genre.to_lowercase().as_str() {
            "visual novel" | "galgame" => {
                systems.push(SystemDesign {
                    name: "Dialogue System".to_string(),
                    components: vec![
                        "DialogueManager".to_string(),
                        "CharacterDisplay".to_string(),
                        "ChoiceHandler".to_string(),
                    ],
                    interactions: vec![
                        "Display character sprites".to_string(),
                        "Show dialogue text".to_string(),
                        "Handle player choices".to_string(),
                    ],
                    code_structure: "State machine for dialogue flow".to_string(),
                });

                systems.push(SystemDesign {
                    name: "Affection System".to_string(),
                    components: vec![
                        "AffectionTracker".to_string(),
                        "CharacterData".to_string(),
                        "EventTrigger".to_string(),
                    ],
                    interactions: vec![
                        "Track relationship levels".to_string(),
                        "Trigger events at thresholds".to_string(),
                        "Affect story branches".to_string(),
                    ],
                    code_structure: "Dictionary of character ID to affection value".to_string(),
                });
            }
            "fps" | "first person shooter" => {
                systems.push(SystemDesign {
                    name: "Weapon System".to_string(),
                    components: vec![
                        "WeaponManager".to_string(),
                        "WeaponBase".to_string(),
                        "AmmoSystem".to_string(),
                    ],
                    interactions: vec![
                        "Weapon switching".to_string(),
                        "Ammo management".to_string(),
                        "Weapon stats".to_string(),
                    ],
                    code_structure: "Weapon base class with polymorphic subclasses".to_string(),
                });

                systems.push(SystemDesign {
                    name: "AI System".to_string(),
                    components: vec![
                        "EnemyAI".to_string(),
                        "Pathfinding".to_string(),
                        "BehaviorTree".to_string(),
                    ],
                    interactions: vec![
                        "Enemy behavior".to_string(),
                        "Pathfinding to player".to_string(),
                        "Attack patterns".to_string(),
                    ],
                    code_structure: "Behavior tree or state machine for AI".to_string(),
                });
            }
            "moba" => {
                systems.push(SystemDesign {
                    name: "Hero System".to_string(),
                    components: vec![
                        "HeroBase".to_string(),
                        "HeroData".to_string(),
                        "LevelUpSystem".to_string(),
                    ],
                    interactions: vec![
                        "Hero selection".to_string(),
                        "Stats progression".to_string(),
                        "Skill unlocks".to_string(),
                    ],
                    code_structure: "Hero base class with data-driven stats".to_string(),
                });

                systems.push(SystemDesign {
                    name: "Network System".to_string(),
                    components: vec![
                        "NetworkManager".to_string(),
                        "GameStateSync".to_string(),
                        "PlayerInput".to_string(),
                    ],
                    interactions: vec![
                        "Player synchronization".to_string(),
                        "Game state replication".to_string(),
                        "Input prediction".to_string(),
                    ],
                    code_structure: "Authoritative server with client prediction".to_string(),
                });
            }
            _ => {}
        }

        Ok(systems)
    }

    /// Generate asset requirements
    async fn generate_assets(
        &self,
        concept: &GameConcept,
        _mechanics: &[MechanicDesign],
        _systems: &[SystemDesign],
    ) -> Result<Vec<AssetRequirement>, AgentError> {
        let mut assets = Vec::new();

        // Add common assets
        assets.push(AssetRequirement {
            name: "UI Icons".to_string(),
            asset_type: AssetType::Sprite2D,
            description: "Menu and HUD icons".to_string(),
            style: "Clean, modern".to_string(),
            quantity: 20,
            specifications: "64x64 PNG with transparency".to_string(),
        });

        assets.push(AssetRequirement {
            name: "Background Music".to_string(),
            asset_type: AssetType::Audio,
            description: "Theme music for different scenes".to_string(),
            style: match concept.genre.to_lowercase().as_str() {
                "visual novel" | "galgame" => "Emotional, atmospheric".to_string(),
                "fps" => "Intense, action-oriented".to_string(),
                "moba" => "Epic, orchestral".to_string(),
                _ => "Generic game music".to_string(),
            },
            quantity: 5,
            specifications: "MP3 or OGG, 2-3 minutes each, loopable".to_string(),
        });

        assets.push(AssetRequirement {
            name: "Sound Effects".to_string(),
            asset_type: AssetType::Audio,
            description: "UI clicks, actions, events".to_string(),
            style: "Crisp, responsive".to_string(),
            quantity: 30,
            specifications: "WAV or OGG, short (<1s), high quality".to_string(),
        });

        // Add genre-specific assets
        match concept.genre.to_lowercase().as_str() {
            "visual novel" | "galgame" => {
                assets.push(AssetRequirement {
                    name: "Character Sprites".to_string(),
                    asset_type: AssetType::Sprite2D,
                    description: "Character portraits with expressions".to_string(),
                    style: "Anime style".to_string(),
                    quantity: 10,  // 3-5 characters, 2-3 expressions each
                    specifications: "PNG with transparency, 800x1200 or similar".to_string(),
                });

                assets.push(AssetRequirement {
                    name: "Backgrounds".to_string(),
                    asset_type: AssetType::Sprite2D,
                    description: "Scene backgrounds".to_string(),
                    style: "Detailed, atmospheric".to_string(),
                    quantity: 8,
                    specifications: "PNG, 1920x1080, high quality".to_string(),
                });
            }
            "fps" | "first person shooter" => {
                assets.push(AssetRequirement {
                    name: "Weapon Models".to_string(),
                    asset_type: AssetType::Model3D,
                    description: "First-person weapon models".to_string(),
                    style: "Realistic, detailed".to_string(),
                    quantity: 5,
                    specifications: "FBX or GLB, game-ready, <50k polygons".to_string(),
                });

                assets.push(AssetRequirement {
                    name: "Enemy Models".to_string(),
                    asset_type: AssetType::Model3D,
                    description: "Enemy character models".to_string(),
                    style: "Varied designs".to_string(),
                    quantity: 3,
                    specifications: "FBX or GLB, with animations, <30k polygons".to_string(),
                });

                assets.push(AssetRequirement {
                    name: "Environment Models".to_string(),
                    asset_type: AssetType::Model3D,
                    description: "Level props and structures".to_string(),
                    style: "Consistent art style".to_string(),
                    quantity: 20,
                    specifications: "FBX or GLB, modular, optimized".to_string(),
                });
            }
            "moba" => {
                assets.push(AssetRequirement {
                    name: "Hero Models".to_string(),
                    asset_type: AssetType::Model3D,
                    description: "Playable hero characters".to_string(),
                    style: "Distinct silhouettes".to_string(),
                    quantity: 10,
                    specifications: "FBX or GLB, with animations, <40k polygons".to_string(),
                });

                assets.push(AssetRequirement {
                    name: "Skill Effects".to_string(),
                    asset_type: AssetType::Animation,
                    description: "Visual effects for skills".to_string(),
                    style: "Flashy, readable".to_string(),
                    quantity: 40,  // 4 skills per hero
                    specifications: "Particle systems or animations".to_string(),
                });

                assets.push(AssetRequirement {
                    name: "Map Assets".to_string(),
                    asset_type: AssetType::Model3D,
                    description: "Map structures and props".to_string(),
                    style: "Consistent with heroes".to_string(),
                    quantity: 30,
                    specifications: "FBX or GLB, modular, optimized".to_string(),
                });
            }
            _ => {}
        }

        Ok(assets)
    }

    /// Generate technical architecture
    async fn generate_technical_architecture(
        &self,
        concept: &GameConcept,
        constraints: Option<&DesignConstraints>,
    ) -> Result<TechnicalArchitecture, AgentError> {
        // Determine engine based on genre and constraints
        let engine = if let Some(constraints) = constraints {
            constraints.preferred_engine.unwrap_or_else(|| self.recommend_engine(concept))
        } else {
            self.recommend_engine(concept)
        };

        let (language, project_type) = match engine {
            GameEngine::RenPy => ("Ren'Py Script".to_string(), "Visual Novel Script".to_string()),
            GameEngine::Godot => ("GDScript".to_string(), "Scene-based".to_string()),
            GameEngine::Unity => ("C#".to_string(), "Component-based".to_string()),
            GameEngine::Unreal => ("C++/Blueprint".to_string(), "Actor-based".to_string()),
        };

        // Determine networking based on genre
        let networking = match concept.genre.to_lowercase().as_str() {
            "moba" => NetworkingType::OnlineMultiplayer,
            "fps" => {
                if concept.core_idea.contains("multiplayer") {
                    NetworkingType::OnlineMultiplayer
                } else {
                    NetworkingType::SinglePlayer
                }
            }
            _ => NetworkingType::SinglePlayer,
        };

        // Generate scene list
        let scenes = self.generate_scene_list(concept);

        Ok(TechnicalArchitecture {
            engine,
            programming_language: language,
            project_type,
            scenes,
            networking,
        })
    }

    /// Generate development plan
    async fn generate_development_plan(
        &self,
        mechanics: &[MechanicDesign],
        systems: &[SystemDesign],
        assets: &[AssetRequirement],
    ) -> Result<Vec<DevelopmentPhase>, AgentError> {
        let mut phases = Vec::new();

        // Phase 1: Prototype
        phases.push(DevelopmentPhase {
            phase_name: "Prototype".to_string(),
            tasks: vec![
                "Set up project structure".to_string(),
                "Implement core mechanics".to_string(),
                "Create placeholder assets".to_string(),
                "Test basic gameplay".to_string(),
            ],
            estimated_days: 7,
        });

        // Phase 2: Core Systems
        phases.push(DevelopmentPhase {
            phase_name: "Core Systems".to_string(),
            tasks: vec![
                "Implement all core systems".to_string(),
                "Integrate mechanics".to_string(),
                "Add UI".to_string(),
                "Implement save/load".to_string(),
            ],
            estimated_days: 14,
        });

        // Phase 3: Content
        phases.push(DevelopmentPhase {
            phase_name: "Content Creation".to_string(),
            tasks: vec![
                "Create final assets".to_string(),
                "Add levels/scenes".to_string(),
                "Implement all mechanics".to_string(),
                "Add audio".to_string(),
            ],
            estimated_days: 21,
        });

        // Phase 4: Polish
        phases.push(DevelopmentPhase {
            phase_name: "Polish & Testing".to_string(),
            tasks: vec![
                "Bug fixing".to_string(),
                "Performance optimization".to_string(),
                "Balance adjustments".to_string(),
                "Final testing".to_string(),
            ],
            estimated_days: 7,
        });

        Ok(phases)
    }

    // Helper methods

    fn extract_title(&self, input: &str) -> Option<String> {
        // Simple extraction - in production, use AI
        if input.contains("做") || input.contains("制作") {
            // Extract after "做" or "制作"
            Some(input.to_string())
        } else {
            None
        }
    }

    fn extract_genre(&self, input: &str) -> Option<String> {
        let input_lower = input.to_lowercase();

        if input_lower.contains("galgame") || input_lower.contains("视觉小说") || input_lower.contains("visual novel") {
            Some("Visual Novel".to_string())
        } else if input_lower.contains("fps") || input_lower.contains("射击") || input_lower.contains("first person") {
            Some("FPS".to_string())
        } else if input_lower.contains("moba") {
            Some("MOBA".to_string())
        } else if input_lower.contains("rpg") {
            Some("RPG".to_string())
        } else {
            None
        }
    }

    pub fn recommend_engine(&self, concept: &GameConcept) -> GameEngine {
        match concept.genre.to_lowercase().as_str() {
            "visual novel" | "galgame" => GameEngine::RenPy,
            "fps" | "first person shooter" => {
                if concept.estimated_scope == ScopeLevel::Large {
                    GameEngine::Unreal
                } else {
                    GameEngine::Unity
                }
            }
            "moba" => GameEngine::Unity,
            "rpg" => {
                if concept.estimated_scope == ScopeLevel::Large {
                    GameEngine::Unreal
                } else {
                    GameEngine::Godot
                }
            }
            _ => GameEngine::Godot,
        }
    }

    fn generate_scene_list(&self, concept: &GameConcept) -> Vec<String> {
        let mut scenes = vec![
            "MainMenu".to_string(),
            "Settings".to_string(),
        ];

        match concept.genre.to_lowercase().as_str() {
            "visual novel" | "galgame" => {
                scenes.extend(vec![
                    "Prologue".to_string(),
                    "Chapter1".to_string(),
                    "Chapter2".to_string(),
                    "Chapter3".to_string(),
                    "Ending".to_string(),
                ]);
            }
            "fps" | "first person shooter" => {
                scenes.extend(vec![
                    "Tutorial".to_string(),
                    "Level1".to_string(),
                    "Level2".to_string(),
                    "BossFight".to_string(),
                ]);
            }
            "moba" => {
                scenes.extend(vec![
                    "HeroSelect".to_string(),
                    "GameMap".to_string(),
                    "ResultScreen".to_string(),
                ]);
            }
            _ => {
                scenes.extend(vec![
                    "Level1".to_string(),
                    "Level2".to_string(),
                ]);
            }
        }

        scenes
    }
}

impl Default for GameDesignerAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for GameDesignerAgent {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn adapter_type(&self) -> AdapterType {
        AdapterType::Api  // Uses Claude API
    }

    fn capabilities(&self) -> Vec<Capability> {
        Self::get_capabilities()
    }

    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError> {
        self.config = Some(config);
        Ok(())
    }

    async fn validate_config(&self) -> Result<bool, AgentError> {
        // Check if Claude API key is available
        Ok(true)
    }

    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError> {
        let start = Instant::now();

        // Parse task description as design request
        let request = DesignRequest {
            user_input: task.description.clone(),
            constraints: None,
        };

        // Generate GDD
        let gdd = self.generate_gdd(&request).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Serialize GDD to JSON
        let gdd_json = serde_json::to_string_pretty(&gdd)
            .map_err(|e| AgentError::ExecutionError {
                message: format!("Failed to serialize GDD: {}", e),
                retryable: false,
            })?;

        Ok(AgentResult {
            task_id: task.id.clone(),
            agent_id: self.id.clone(),
            status: ResultStatus::Success,
            output: TaskOutput::Text(gdd_json),
            input_tokens: 0,  // TODO: Track actual tokens
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
                ("genre".to_string(), gdd.concept.genre),
                ("engine".to_string(), format!("{:?}", gdd.technical_architecture.engine)),
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
                input_tokens: 2000,
                output_tokens: 4000,
                total_tokens: 6000,
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
    fn test_game_designer_agent_new() {
        let agent = GameDesignerAgent::new();
        assert_eq!(agent.id(), "game-designer");
        assert_eq!(agent.adapter_type(), AdapterType::Api);
    }

    #[test]
    fn test_recommend_engine() {
        let agent = GameDesignerAgent::new();

        let vn_concept = GameConcept {
            title: "Test".to_string(),
            genre: "Visual Novel".to_string(),
            core_idea: "Test".to_string(),
            target_audience: "Test".to_string(),
            estimated_scope: ScopeLevel::Medium,
        };
        assert_eq!(agent.recommend_engine(&vn_concept), GameEngine::RenPy);

        let fps_concept = GameConcept {
            title: "Test".to_string(),
            genre: "FPS".to_string(),
            core_idea: "Test".to_string(),
            target_audience: "Test".to_string(),
            estimated_scope: ScopeLevel::Large,
        };
        assert_eq!(agent.recommend_engine(&fps_concept), GameEngine::Unreal);
    }

    #[test]
    fn test_extract_genre() {
        let agent = GameDesignerAgent::new();

        assert_eq!(agent.extract_genre("我想做一个galgame"), Some("Visual Novel".to_string()));
        assert_eq!(agent.extract_genre("I want to make an FPS game"), Some("FPS".to_string()));
        assert_eq!(agent.extract_genre("Let's create a MOBA"), Some("MOBA".to_string()));
    }
}
