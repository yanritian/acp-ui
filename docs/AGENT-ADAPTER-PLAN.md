# Agent Adapter 实现计划 v3.0

> **版本**: v3.0（聚焦代码 + 游戏开发）
> **创建日期**: 2026-06-23
> **状态**: 规划中
> **依赖文档**: docs/AGENT-PLATFORM-PRD.md
> **用户明确优先级**: 先实现代码场景（Web/小程序/桌面端），再实现游戏场景

---

## 一、技术目标

### 1.1 核心目标

```
目标 1: 统一接口 — 所有 Agent 通过同一 trait 调用
目标 2: 场景聚焦 — 针对代码 + 游戏开发优化
目标 3: 本地优先 — 代码本地处理，隐私保护（Privacy-Preserving）
目标 4: 自优化 — 根据历史数据优化 Agent 选择（Self-Optimizing）
目标 5: 自适应 — 根据平台特性调整工作流（Adaptive Workflow）
目标 6: 多 Agent 协作 — 代码 + 美术 + 测试，Multi-Agent Consensus
```

### 1.2 当前状态

| 组件 | 状态 | 说明 |
|------|------|------|
| **AcpSessionRunner** | ✅ 已实现 | CLI Agent 执行基础 |
| **Agent Registry** | ✅ 已实现 | Docker-like 注册模式 |
| **Loop Engine** | ✅ 已实现 | Self-Healing + Evolution |
| **Memory System** | ✅ 已实现 | hermes-memory crate |
| **Smart Router** | ✅ 已实现 | 智能路由 |
| **Circuit Breaker** | ✅ 已实现 | 熔断器 |
| **Team DAG** | ✅ 已实现 | DAG 执行引擎 |
| **Self-Optimizing Router** | ❌ 待实现 | 基于历史数据优化 |
| **Privacy-Preserving Orchestrator** | ❌ 待实现 | 代码本地处理 |
| **Adaptive Workflow Generator** | ❌ 待实现 | 根据平台调整工作流 |
| **Game Resource Generator** | ❌ 待实现 | 游戏美术资源生成 |
| **Cross-Platform Game Adapter** | ❌ 待实现 | 跨平台游戏适配 |
| **Game Performance Optimizer** | ❌ 待实现 | 游戏性能优化 |

---

## 二、AgentAdapter Trait 定义

### 2.1 Rust Trait（核心接口）

```rust
// src-tauri/src/agent_adapter/mod.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterType {
    Cli,      // 本地 CLI Agent (Claude Code, Codex)
    Api,      // HTTP API Agent (Kimi, 可灵, WPS)
    Hybrid,   // 混合型 (WebSocket + HTTP)
}

/// Agent 能力标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub proficiency: f32,      // 0.0 - 1.0，熟练度
    pub cost_per_unit: f32,    // USD or CNY，单位成本
    pub latency_ms: u64,       // 平均响应时间
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub api_key: Option<String>,        // API 密钥（加密存储）
    pub endpoint: Option<String>,       // API endpoint
    pub model: Option<String>,          // 模型名称
    pub timeout_ms: u64,                // 超时时间
    pub max_retries: u32,               // 最大重试次数
    pub cwd: Option<String>,            // 工作目录（CLI Agent）
    pub extra: HashMap<String, String>, // 额外配置
}

/// Agent 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub description: String,
    pub input: TaskInput,
    pub constraints: TaskConstraints,
    pub scene_type: Option<SceneType>,  // 场景类型（代码/游戏）
    pub platform: Option<Platform>,     // 平台（Web/小程序/桌面/游戏）
}

/// 场景类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneType {
    WebDevelopment,
    MiniProgramDevelopment,
    DesktopDevelopment,
    GameDevelopment,
    GameArtGeneration,
    GameCrossPlatform,
    GamePerformanceOptimization,
}

/// 平台
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Web,
    MiniProgram,
    Desktop,
    Mobile,
    Game,
}

/// 任务输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskInput {
    Text(String),                       // 文本输入
    File { path: String },              // 文件输入
    Url { url: String },                // URL 输入
    Data { bytes: Vec<u8> },            // 二进制数据
    Multi { inputs: Vec<TaskInput> },   // 多输入
}

/// 任务约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraints {
    pub timeout_ms: u64,                // 超时时间
    pub max_tokens: Option<u64>,        // 最大 Token
    pub budget_limit: Option<f32>,      // 成本上限
    pub quality_level: QualityLevel,    // 质量等级
}

/// Agent 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: String,
    pub agent_id: String,
    pub status: ResultStatus,
    pub output: TaskOutput,
    pub input_tokens: u64,              // 输入 Token 数
    pub output_tokens: u64,             // 输出 Token 数
    pub total_tokens: u64,              // 总 Token 数
    pub cost: ActualCost,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Agent Adapter 统一接口
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn adapter_type(&self) -> AdapterType;
    fn capabilities(&self) -> &[Capability];
    
    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError>;
    async fn validate_config(&self) -> Result<bool, AgentError>;
    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError>;
    async fn cancel(&self, task_id: &str) -> Result<(), AgentError>;
    
    fn status(&self) -> AgentStatus;
    async fn health(&self) -> HealthMetrics;
    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate;
    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage>;
    
    async fn update_health(&mut self, result: &AgentResult);
    fn is_circuit_breaker_allowed(&self) -> bool;
    async fn reset_circuit_breaker(&mut self);
}
```

---

## 三、Self-Optimizing Router

### 3.1 核心实现

```rust
// src-tauri/src/self_optimizing_router.rs

use crate::agent_adapter::{AgentAdapter, AgentTask, SceneType, Platform};
use crate::agent_registry::AgentRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SelfOptimizingRouter {
    registry: Arc<AgentRegistry>,
    performance_history: Arc<RwLock<Vec<ExecutionRecord>>>,
    project_contexts: Arc<RwLock<HashMap<String, ProjectContext>>>,
    optimization_loop: Arc<LoopEngine>,
}

pub struct ExecutionRecord {
    pub task_id: String,
    pub scene_type: SceneType,
    pub platform: Platform,
    pub agent_id: String,
    pub success: bool,
    pub duration_ms: u64,
    pub cost: f32,
    pub timestamp: u64,
}

pub struct ProjectContext {
    pub project_id: String,
    pub scene_type: SceneType,
    pub tech_stack: TechStack,
    pub historical_performance: HashMap<String, AgentPerformance>,
}

pub struct AgentPerformance {
    pub agent_id: String,
    pub success_rate: f32,
    pub avg_duration_ms: u64,
    pub avg_cost: f32,
    pub last_used: u64,
}

impl SelfOptimizingRouter {
    pub async fn select_optimal_agents(
        &self,
        task: &AgentTask,
        project_id: &str,
    ) -> AgentSelection {
        // 获取项目上下文
        let context = self.get_project_context(project_id).await;
        
        // 基于场景类型优化选择
        let strategy = match task.scene_type {
            Some(SceneType::WebDevelopment) => self.optimize_for_web(&context, task).await,
            Some(SceneType::MiniProgramDevelopment) => self.optimize_for_miniprogram(&context, task).await,
            Some(SceneType::DesktopDevelopment) => self.optimize_for_desktop(&context, task).await,
            Some(SceneType::GameDevelopment) => self.optimize_for_game(&context, task).await,
            Some(SceneType::GameArtGeneration) => self.optimize_for_game_art(&context, task).await,
            Some(SceneType::GameCrossPlatform) => self.optimize_for_cross_platform(&context, task).await,
            Some(SceneType::GamePerformanceOptimization) => self.optimize_for_performance(&context, task).await,
            None => self.default_optimization(&context, task).await,
        };
        
        strategy.select(task)
    }
    
    async fn optimize_for_web(&self, context: &ProjectContext, task: &AgentTask) -> OptimizedStrategy {
        // Web 开发：Claude Code 在 React/Vue 上表现更好
        let weights = self.calculate_weights_for_web(context, task);
        
        OptimizedStrategy {
            primary_agent: "claude-code",
            secondary_agents: vec!["codex", "test-agent"],
            weights,
            execution_mode: ExecutionMode::SerialThenParallel,
        }
    }
    
    async fn optimize_for_game(&self, context: &ProjectContext, task: &AgentTask) -> OptimizedStrategy {
        // 游戏开发：Claude Code (代码) + 即梦 (美术) + ComfyUI (场景) + Test Agent
        let weights = self.calculate_weights_for_game(context, task);
        
        OptimizedStrategy {
            primary_agent: "claude-code",
            secondary_agents: vec!["codex", "jimeng", "comfyui", "test-agent"],
            weights,
            execution_mode: ExecutionMode::Hybrid,
        }
    }
    
    async fn record_execution(&self, record: ExecutionRecord) {
        // 记录执行历史
        self.performance_history.write().await.push(record);
        
        // 更新项目上下文中的 Agent 性能
        self.update_agent_performance(&record).await;
        
        // 触发优化循环（如果积累足够数据）
        if self.performance_history.read().await.len() % 10 == 0 {
            self.optimization_loop.evolve().await;
        }
    }
}
```

---

## 四、Privacy-Preserving Orchestrator

### 4.1 核心实现

```rust
// src-tauri/src/privacy_orchestrator.rs

use crate::agent_adapter::{AgentAdapter, AgentTask, TaskInput};
use std::sync::Arc;

pub struct PrivacyPreservingOrchestrator {
    local_agents: Vec<Arc<dyn AgentAdapter>>,      // 本地 CLI Agent
    cloud_agents: Vec<Arc<dyn AgentAdapter>>,      // 云端 API Agent
    privacy_policy: PrivacyPolicy,
}

pub struct PrivacyPolicy {
    // 代码文件：必须本地处理
    pub code_files: PrivacyLevel,
    
    // 游戏资源：可以云端生成
    pub game_resources: PrivacyLevel,
    
    // 配置文件：可以云端处理
    pub config_files: PrivacyLevel,
    
    // 测试数据：根据敏感度决定
    pub test_data: PrivacyLevel,
}

pub enum PrivacyLevel {
    LocalOnly,        // 必须本地处理
    CloudAllowed,     // 可以云端处理
    Conditional,      // 根据敏感度决定
}

impl PrivacyPreservingOrchestrator {
    pub async fn execute_task(&self, task: &AgentTask) -> Result<TaskResult, OrchestratorError> {
        // 分析任务中的文件类型
        let file_analysis = self.analyze_files(&task.input);
        
        // 根据隐私策略选择 Agent
        let mut results = Vec::new();
        
        for file_info in file_analysis {
            let agent = match self.privacy_policy.get_level(&file_info.file_type) {
                PrivacyLevel::LocalOnly => {
                    // 必须本地处理（代码文件）
                    self.select_local_agent(&file_info).await?
                }
                PrivacyLevel::CloudAllowed => {
                    // 可以云端处理（游戏资源）
                    self.select_cloud_agent(&file_info).await
                        .or_else(|| self.select_local_agent(&file_info))?
                }
                PrivacyLevel::Conditional => {
                    // 根据敏感度决定
                    if self.is_sensitive(&file_info) {
                        self.select_local_agent(&file_info).await?
                    } else {
                        self.select_cloud_agent(&file_info).await
                            .or_else(|| self.select_local_agent(&file_info))?
                    }
                }
            };
            
            let result = agent.execute(task.clone()).await?;
            results.push(result);
        }
        
        Ok(TaskResult { results })
    }
    
    fn is_sensitive(&self, file_info: &FileInfo) -> bool {
        // 检查文件是否包含敏感信息
        file_info.contains_api_keys 
            || file_info.contains_passwords 
            || file_info.contains_personal_data
    }
}
```

---

## 五、Game Resource Generator

### 5.1 核心实现

```rust
// src-tauri/src/game_resource_generator.rs

use crate::agent_adapter::{AgentAdapter, AgentTask};
use std::sync::Arc;

pub struct GameResourceGenerator {
    image_generators: Vec<Arc<dyn AgentAdapter>>,  // 即梦, ComfyUI, DALL-E
    audio_generators: Vec<Arc<dyn AgentAdapter>>,  // 可灵（视频转音频）
    text_generators: Vec<Arc<dyn AgentAdapter>>,   // Kimi
    style_manager: GameStyleManager,
}

pub struct GameResourceRequest {
    pub resource_type: ResourceType,
    pub style: GameStyle,
    pub description: String,
    pub platform: Platform,
    pub count: u32,
}

pub enum ResourceType {
    Character,    // 角色
    Scene,        // 场景
    UI,           // UI 元素
    Audio,        // 音频
    Animation,    // 动画
}

pub enum GameStyle {
    Pixel,        // 像素风
    Cartoon,      // 卡通风
    Realistic,    // 写实风
    Anime,        // 动漫风
}

impl GameResourceGenerator {
    pub async fn generate_resources(
        &self,
        request: &GameResourceRequest,
    ) -> Result<GameResources, GeneratorError> {
        match request.resource_type {
            ResourceType::Character => {
                self.generate_character_resources(request).await
            }
            ResourceType::Scene => {
                self.generate_scene_resources(request).await
            }
            ResourceType::UI => {
                self.generate_ui_resources(request).await
            }
            ResourceType::Audio => {
                self.generate_audio_resources(request).await
            }
            ResourceType::Animation => {
                self.generate_animation_resources(request).await
            }
        }
    }
    
    async fn generate_character_resources(
        &self,
        request: &GameResourceRequest,
    ) -> Result<GameResources, GeneratorError> {
        // Step 1: 生成角色描述
        let description = self.text_generators[0]
            .execute(AgentTask {
                description: format!(
                    "描述一个{}风格的{}游戏角色：{}",
                    request.style.to_string(),
                    request.platform.to_string(),
                    request.description
                ),
                ..Default::default()
            })
            .await?;
        
        // Step 2: 并行生成多张角色图
        let mut image_futures = Vec::new();
        
        for generator in &self.image_generators {
            let desc = description.output.to_string();
            let future = generator.execute(AgentTask {
                description: desc,
                ..Default::default()
            });
            image_futures.push(future);
        }
        
        let image_results = futures::future::join_all(image_futures).await;
        let images = image_results.into_iter()
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(GameResources {
            images,
            audio: Vec::new(),
            metadata: HashMap::new(),
        })
    }
}
```

---

## 六、实施步骤（Phase 1: Week 1-4）

### Week 1: AgentAdapter Trait + 本地 CLI Adapter

- [ ] 创建 `src-tauri/src/agent_adapter/mod.rs` trait 定义
- [ ] 创建 `src-tauri/src/agent_adapter/types.rs` 类型定义
- [ ] 实现 `ClaudeCodeAdapter` (CLI stdio + Token 追踪)
- [ ] 实现 `CodexAdapter` (CLI stdio)
- [ ] 实现 `HealthTracker` (EWMA + 熔断器)
- [ ] 单元测试 `claude_adapter_test.rs`

### Week 2: AgentRegistry + Self-Optimizing Router

- [ ] 创建 `src-tauri/src/agent_registry/mod.rs` Registry 实现
- [ ] 实现 `CapabilityIndex` 能力索引
- [ ] 实现 `SelfOptimizingRouter` 自优化选择
- [ ] 实现 `ProjectContext` 项目上下文
- [ ] 单元测试 `self_optimizing_router_test.rs`

### Week 3: Privacy-Preserving Orchestrator + DAG Engine

- [ ] 创建 `src-tauri/src/privacy_orchestrator.rs` 隐私编排
- [ ] 实现 `PrivacyPolicy` 隐私策略
- [ ] 实现 `DAGEngine` 串行 + 并行执行
- [ ] 实现 `ResultAggregator` 结果汇总
- [ ] 单元测试 `privacy_orchestrator_test.rs`

### Week 4: Tauri Commands + 前端 API + 测试

- [ ] 实现 8 个 Tauri 命令
- [ ] 前端 API 封装 `src/lib/agent-adapter/api.ts`
- [ ] Vue Store `src/stores/agent-registry.ts`
- [ ] 集成测试
- [ ] **用户验收**: Web + 小程序开发 MVP

---

## 七、验收标准（Phase 1）

| 标准 | 说明 | 验证方式 |
|------|------|----------|
| **本地 Agent 测试成功** | Claude Code + Codex 执行成功 | 用户验收 |
| **Agent Adapter 接口** | 所有 Agent 实现统一 trait | `cargo test --workspace` |
| **Self-Optimizing** | 基于历史数据优化 Agent 选择 | 选择准确率 ≥ 80% |
| **Privacy-Preserving** | 代码本地处理 | 代码文件不上传云端 |
| **Token 追踪** | 实时 Token 用量显示 | 执行中显示 input/output/total tokens |
| **健康监控** | EWMA 5 分钟更新 + 熔断触发 | 健康分数 ≤ 80 触发熔断 |
| **测试覆盖** | 80%+ 单元测试 | `cargo tarpaulin` |

---

**签署**: ACP-UI Team + yan_fan_tian
**日期**: 2026-06-23