# Agent Platform 产品需求文档 (PRD) v3.0

> **版本**: v3.0（聚焦代码 + 游戏开发）
> **创建日期**: 2026-06-23
> **状态**: 规划中
> **分支**: refactor/project-cleanup

---

## 一、产品定位

### 1.1 核心愿景

**ACP-UI = Local-First Self-Optimizing Agent Platform for Code & Game Development**

一个本地优先、自优化的 Agent 平台，专注于代码开发和游戏开发场景，支持 Web、小程序、桌面端、游戏（桌面/移动/小程序）的全栈开发。

### 1.2 目标用户

| 用户类型 | 场景 | 核心需求 |
|----------|------|----------|
| **Web 开发者** | React/Vue/Next.js 开发 | 代码生成、组件开发、API 集成、性能优化 |
| **小程序开发者** | 微信/支付宝/抖音小程序 | 跨平台适配、性能优化、快速开发 |
| **桌面端开发者** | Tauri/Electron/Swift/Qt | 原生功能、跨平台构建、打包发布 |
| **游戏开发者** | Unity/Cocos/Godot | 游戏原型、美术资源、跨平台发布、性能优化 |

### 1.3 核心价值主张

**提高效率 + 本地优先 + 隐私保护 + 自优化**

```
用户只需要：
- 输入一句话（如"帮我开发一个 2D 平台跳跃游戏"）
- 系统自动识别场景（游戏开发）
- 系统自动选择 Agent 组合（Claude Code + 即梦 + ComfyUI）
- 系统自动编排执行（代码 + 美术 + 测试）
- 系统自动优化（根据历史数据优化 Agent 选择）

系统自动完成：
- 场景识别（代码 / 游戏 / Web / 小程序 / 桌面）
- Agent 选择（Self-Optimizing，基于历史数据）
- 隐私保护（代码本地处理，Privacy-Preserving）
- 工作流生成（Adaptive Workflow，根据平台调整）
- 多 Agent 协作（代码 + 美术 + 测试，Multi-Agent Consensus）
- 结果汇总（代码 + 资源 + 测试报告）
```

### 1.4 核心技术壁垒

| 技术 | 说明 | 差异化优势 |
|------|------|------------|
| **Self-Optimizing Agent Selection** | 根据项目类型和历史数据自动优化 Agent 选择 | 与 Claude Code / Cursor 差异化 |
| **Privacy-Preserving Orchestration** | 代码本地处理，敏感数据不上传云端 | 企业级隐私保护 |
| **Adaptive Workflow Generation** | 根据平台特性自动调整工作流 | 跨平台开发优化 |
| **Multi-Agent Consensus** | 多个 Agent 投票确认代码质量 | 提高代码可靠性 |
| **Game Resource Generator** | 自动生成游戏美术、音效资源 | 游戏开发专属 |
| **Cross-Platform Game Adapter** | 自动适配桌面、移动、小程序 | 游戏跨平台 |
| **Game Performance Optimizer** | 自动优化游戏性能 | 游戏性能优化 |

---

## 二、竞品分析

### 2.1 主要竞品对比

| 竞品 | 定位 | 核心功能 | 定价 | 优劣势 |
|------|------|----------|------|--------|
| **Claude Code** | 编程助手 | 代码生成、调试 | $20/月 | ✅ 编程能力强 ❌ 只支持 Claude，代码上传云端 |
| **Cursor** | AI IDE | 代码编辑 + AI | $20/月 | ✅ 集成 IDE ❌ 只支持代码，无游戏开发 |
| **Cline** | VS Code 插件 | 代码生成 | 免费 + API | ✅ 开源 ❌ 配置复杂，无游戏开发 |
| **Dify** | AI 应用平台 | 工作流编排 | 开源 + 付费 | ✅ 可视化编排 ❌ 学习曲线，无代码优化 |
| **Coze** | Bot 平台 | Bot 创建 | 免费 | ✅ 易用 ❌ 功能有限，无代码开发 |

### 2.2 我们的差异化优势

| 维度 | Claude Code / Cursor | ACP-UI |
|------|---------------------|--------|
| **隐私保护** | 代码上传云端 | 代码本地处理（Privacy-Preserving） |
| **Agent 组合** | 单一 Agent | 多 Agent 组合（代码 + 美术 + 测试） |
| **跨平台** | 单一平台 | Web + 小程序 + 桌面端 + 游戏 |
| **自优化** | 无 | Self-Optimizing Agent Selection |
| **游戏开发** | 无 | 游戏原型 + 美术资源 + 跨平台 + 性能优化 |
| **多 Agent 共识** | 无 | Multi-Agent Consensus 确认代码质量 |

---

## 三、场景聚焦

### 3.1 代码场景

| 子场景 | 技术栈 | Agent 组合 | 执行模式 |
|--------|--------|-----------|----------|
| **Web 开发** | React/Vue/Next.js/Nuxt.js | Claude Code + Codex + Test Agent | 串行 + 并行 |
| **小程序开发** | 微信/支付宝/抖音/uni-app | Claude Code + Codex + Test Agent | 串行 + 并行 |
| **桌面端开发** | Tauri/Electron/Swift/Qt | Claude Code + Codex + Test Agent | 串行 + 并行 |

### 3.2 游戏场景

| 子场景 | 技术栈 | Agent 组合 | 执行模式 |
|--------|--------|-----------|----------|
| **桌面端游戏** | Unity/UE/Cocos/Godot | Claude Code + Codex + 即梦 + ComfyUI + Test Agent | 串行 + 并行 |
| **移动游戏** | Unity/Cocos | Claude Code + Codex + 即梦 + ComfyUI + Test Agent | 串行 + 并行 |
| **小程序游戏** | Cocos Creator | Claude Code + Codex + 即梦 + ComfyUI + Test Agent | 串行 + 并行 |

### 3.3 其他场景（后期扩展）

| 场景 | 状态 | 说明 |
|------|------|------|
| **营销场景** | 🔄 后期 | Kimi + 可灵 + 即梦，营销方案 + 视频 + 海报 |
| **财务场景** | 🔄 后期 | WPS AI + 通义千问，数据分析 + 报表 |
| **设计场景** | 🔄 后期 | ComfyUI + DALL-E，UI 设计 + 原型 |
| **运营场景** | 🔄 后期 | 多平台对接，内容分发 + 数据分析 |

---

## 四、功能模块详细设计

### 4.1 One-Shot Interface（一句话入口）

#### 4.1.1 功能要求

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| **自然语言输入** | 用户输入一句话，系统自动识别场景 | P0 |
| **场景识别** | 自动识别代码 / 游戏 / Web / 小程序 / 桌面 | P0 |
| **Agent 选择** | Self-Optimizing，基于历史数据优化 | P0 |
| **隐私保护** | Privacy-Preserving，代码本地处理 | P0 |
| **透明决策展示** | 展示 Agent 选择理由 + 执行预估 + DAG | P0 |
| **工作流生成** | Adaptive Workflow，根据平台调整 | P0 |
| **执行监控** | 实时展示各 Agent 执行进度 + 成本 + Token | P0 |
| **结果汇总** | 将各 Agent 输出整合为最终结果 | P0 |

#### 4.1.2 场景识别机制

```rust
// src-tauri/src/scene_identifier.rs

pub struct SceneIdentifier {
    keyword_rules: HashMap<String, SceneType>,
    llm_classifier: LlmClassifier,
    history_memory: MemorySystem,
}

#[derive(Debug, Clone)]
pub enum SceneType {
    WebDevelopment,
    MiniProgramDevelopment,
    DesktopDevelopment,
    GameDevelopment,
    GameArtGeneration,
    GameCrossPlatform,
    GamePerformanceOptimization,
    // 后期扩展
    Marketing,
    Finance,
    Design,
}

impl SceneIdentifier {
    pub async fn identify(&self, user_input: &str) -> SceneIdentificationResult {
        // Step 1: 关键词快速匹配
        if let Some(scene) = self.keyword_match(user_input) {
            return SceneIdentificationResult::Certain(scene);
        }
        
        // Step 2: LLM 语义分类
        let llm_result = self.llm_classifier.classify(user_input).await;
        if llm_result.confidence > 0.8 {
            return SceneIdentificationResult::Certain(llm_result.scene);
        }
        
        // Step 3: 反问用户
        SceneIdentificationResult::Uncertain {
            possible_scenes: llm_result.top_scenes,
            recommendation: self.get_historical_preference(),
        }
    }
}
```

### 4.2 Self-Optimizing Agent Selection

#### 4.2.1 核心逻辑

```rust
// src-tauri/src/self_optimizing_router.rs

pub struct SelfOptimizingRouter {
    router: SmartRouter,
    performance_history: Vec<ExecutionRecord>,
    project_context: ProjectContext,
    optimization_loop: EvolutionLoop,
}

pub struct ProjectContext {
    scene_type: SceneType,           // Web / MiniProgram / Desktop / Game
    tech_stack: TechStack,           // React / Vue / Unity / Cocos
    language: Language,              // TypeScript / Rust / C#
    historical_performance: HashMap<String, f32>,  // Agent → success_rate
}

impl SelfOptimizingRouter {
    pub async fn select_optimal_agents(&self, task: &Task) -> AgentSelection {
        // 基于项目类型和历史数据优化选择
        let optimized_strategy = match self.project_context.scene_type {
            SceneType::WebDevelopment => self.optimize_for_web(task).await,
            SceneType::MiniProgramDevelopment => self.optimize_for_miniprogram(task).await,
            SceneType::DesktopDevelopment => self.optimize_for_desktop(task).await,
            SceneType::GameDevelopment => self.optimize_for_game(task).await,
            SceneType::GameArtGeneration => self.optimize_for_game_art(task).await,
            SceneType::GameCrossPlatform => self.optimize_for_cross_platform(task).await,
            SceneType::GamePerformanceOptimization => self.optimize_for_performance(task).await,
            _ => self.default_optimization(task).await,
        };
        
        optimized_strategy.select(task)
    }
    
    async fn optimize_for_game(&self, task: &Task) -> OptimizedStrategy {
        // 游戏开发：Claude Code 在 Unity/Cocos 上表现更好
        // 需要美术资源：即梦 + ComfyUI
        let weights = self.calculate_weights_for_game(task);
        
        OptimizedStrategy {
            primary_agent: "claude-code",      // 游戏代码
            secondary_agents: vec![
                "codex",                       // 代码优化
                "jimeng",                      // 美术资源
                "comfyui",                     // 场景资源
                "test-agent",                  // 测试
            ],
            weights,
        }
    }
}
```

### 4.3 Privacy-Preserving Orchestration

#### 4.3.1 核心逻辑

```rust
// src-tauri/src/privacy_orchestrator.rs

pub struct PrivacyPreservingOrchestrator {
    local_agent: LocalAgentRunner,      // 本地 CLI Agent
    cloud_agent: Option<CloudAgentRunner>,  // 可选云端 Agent
    privacy_policy: PrivacyPolicy,
}

pub struct PrivacyPolicy {
    // 代码文件：必须本地处理
    code_files: PrivacyLevel::LocalOnly,
    
    // 游戏资源：可以云端生成
    game_resources: PrivacyLevel::CloudAllowed,
    
    // 配置文件：可以云端处理
    config_files: PrivacyLevel::CloudAllowed,
    
    // 测试数据：根据敏感度决定
    test_data: PrivacyLevel::Conditional,
}

impl PrivacyPreservingOrchestrator {
    pub async fn execute_task(&self, task: &Task) -> Result<TaskResult> {
        // 分析任务中的文件类型
        let file_analysis = self.analyze_files(&task.files);
        
        // 根据隐私策略选择 Agent
        for file in &task.files {
            match self.privacy_policy.get_level(&file.type) {
                PrivacyLevel::LocalOnly => {
                    // 必须本地处理（代码文件）
                    self.local_agent.execute(&file).await?;
                }
                PrivacyLevel::CloudAllowed => {
                    // 可以云端处理（游戏资源）
                    if let Some(cloud) = &self.cloud_agent {
                        cloud.execute(&file).await?;
                    } else {
                        self.local_agent.execute(&file).await?;
                    }
                }
                PrivacyLevel::Conditional => {
                    // 根据敏感度决定
                    if self.is_sensitive(&file) {
                        self.local_agent.execute(&file).await?;
                    } else {
                        self.cloud_agent.as_ref().unwrap().execute(&file).await?;
                    }
                }
            }
        }
        
        Ok(result)
    }
}
```

### 4.4 Game Resource Generator

#### 4.4.1 核心逻辑

```rust
// src-tauri/src/game_resource_generator.rs

pub struct GameResourceGenerator {
    image_generators: Vec<Arc<dyn AgentAdapter>>,  // 即梦, ComfyUI, DALL-E
    audio_generators: Vec<Arc<dyn AgentAdapter>>,  // 可灵（视频转音频）
    text_generators: Vec<Arc<dyn AgentAdapter>>,   // Kimi
    style_manager: GameStyleManager,
}

pub struct GameResourceRequest {
    resource_type: ResourceType,  // Character, Scene, UI, Audio
    style: GameStyle,             // Pixel, Cartoon, Realistic
    description: String,
    platform: Platform,           // Desktop, Mobile, MiniProgram
}

impl GameResourceGenerator {
    pub async fn generate_resources(&self, request: &GameResourceRequest) -> GameResources {
        match request.resource_type {
            ResourceType::Character => {
                // 角色生成：即梦 + ComfyUI
                let character_images = self.generate_character_images(request).await?;
                GameResources { images: character_images, ..Default::default() }
            }
            ResourceType::Scene => {
                // 场景生成：ComfyUI + DALL-E
                let scene_images = self.generate_scene_images(request).await?;
                GameResources { images: scene_images, ..Default::default() }
            }
            ResourceType::UI => {
                // UI 生成：即梦 + DALL-E
                let ui_elements = self.generate_ui_elements(request).await?;
                GameResources { images: ui_elements, ..Default::default() }
            }
            ResourceType::Audio => {
                // 音频生成：可灵（视频转音频）
                let audio_files = self.generate_audio(request).await?;
                GameResources { audio: audio_files, ..Default::default() }
            }
        }
    }
}
```

### 4.5 Cross-Platform Game Adapter

#### 4.5.1 核心逻辑

```rust
// src-tauri/src/cross_platform_game_adapter.rs

pub struct CrossPlatformGameAdapter {
    code_agents: Vec<Arc<dyn AgentAdapter>>,  // Claude Code, Codex
    platform_adapters: HashMap<Platform, PlatformAdapter>,
    consensus_engine: MultiAgentConsensus,
}

impl CrossPlatformGameAdapter {
    pub async fn adapt_game_to_platforms(
        &self,
        game: &Game,
        platforms: &[Platform],  // [Desktop, Mobile, MiniProgram]
    ) -> Result<MultiPlatformGame> {
        // 分析游戏核心逻辑
        let core_logic = self.analyze_core_logic(game).await?;
        
        // 为每个平台生成适配代码
        let mut platform_games = HashMap::new();
        
        for platform in platforms {
            let adapter = self.platform_adapters.get(platform).unwrap();
            
            // 根据平台特性适配
            let adapted_code = self.adapt_to_platform(
                &core_logic,
                adapter,
            ).await?;
            
            platform_games.insert(platform.clone(), adapted_code);
        }
        
        // 多 Agent 共识：确认跨平台兼容性
        let consensus = self.consensus_engine.reach_consensus(
            &platform_games,
            ConsensusStrategy::CompatibilityCheck,
        ).await?;
        
        if !consensus.is_compatible() {
            // 自动修复兼容性问题
            let fixed_games = self.fix_compatibility_issues(
                &platform_games,
                &consensus.issues(),
            ).await?;
            
            Ok(MultiPlatformGame { games: fixed_games, ..Default::default() })
        } else {
            Ok(MultiPlatformGame { games: platform_games, ..Default::default() })
        }
    }
}
```

### 4.6 Game Performance Optimizer

#### 4.6.1 核心逻辑

```rust
// src-tauri/src/game_performance_optimizer.rs

pub struct GamePerformanceOptimizer {
    code_agents: Vec<Arc<dyn AgentAdapter>>,
    test_agents: Vec<Arc<dyn AgentAdapter>>,
    profiler: GameProfiler,
}

impl GamePerformanceOptimizer {
    pub async fn optimize_game(&self, game: &Game) -> Result<OptimizedGame> {
        // 分析当前性能
        let metrics = self.profiler.profile(game).await?;
        
        // 识别性能瓶颈
        let bottlenecks = self.identify_bottlenecks(&metrics);
        
        // 多 Agent 协作优化
        let mut optimized = game.clone();
        
        for bottleneck in bottlenecks {
            // Claude Code 分析瓶颈
            let analysis = self.code_agents[0].execute(AgentTask {
                description: format!("分析并优化性能瓶颈：{:?}", bottleneck),
                ..Default::default()
            }).await?;
            
            // Codex 实现优化
            let optimization = self.code_agents[1].execute(AgentTask {
                description: format!("实现优化方案：{:?}", analysis.output),
                ..Default::default()
            }).await?;
            
            // Test Agent 验证优化效果
            let test_result = self.test_agents[0].execute(AgentTask {
                description: format!("测试优化效果：{:?}", optimization.output),
                ..Default::default()
            }).await?;
            
            // 如果优化有效，应用到游戏
            if test_result.success {
                optimized.apply_optimization(optimization.output.to_string());
            }
        }
        
        // 最终性能验证
        let final_metrics = self.profiler.profile(&optimized).await?;
        
        Ok(OptimizedGame {
            game: optimized,
            before_metrics: metrics,
            after_metrics: final_metrics,
        })
    }
}
```

---

## 五、实施计划（6 Phase × 4 Week = 24 Week）

### Phase 1（Week 1-4）：Web + 小程序开发 MVP

**目标**：实现 Web + 小程序开发的 Agent 编排

| 任务 | 交付物 | 验收标准 |
|------|--------|----------|
| 实现 ClaudeCodeAdapter | `claude_adapter.rs` | CLI 执行成功 + Token 追踪 |
| 实现 CodexAdapter | `codex_adapter.rs` | CLI 执行成功 |
| 实现 Privacy-Preserving Orchestrator | `privacy_orchestrator.rs` | 代码本地处理 |
| 实现 Self-Optimizing Router | `self_optimizing_router.rs` | 基于历史数据优化 |
| 实现基本 DAG 执行 | `dag_engine.rs` | 串行 + 并行执行 |
| 用户测试（Web + 小程序） | 测试报告 | 用户反馈收集 |

**交付**：Web + 小程序开发 MVP

### Phase 2（Week 5-8）：桌面端开发

**目标**：扩展到桌面端开发

| 任务 | 交付物 | 验收标准 |
|------|--------|----------|
| 实现 Tauri/Electron 项目模板 | `templates/desktop/` | 模板可用 |
| 实现跨平台适配 Agent 组合 | `cross_platform_adapter.rs` | 桌面端适配成功 |
| 实现原生功能集成 Agent | `native_integration_agent.rs` | 原生功能可用 |
| 用户测试（桌面端） | 测试报告 | 用户反馈收集 |

**交付**：Web + 小程序 + 桌面端 MVP

### Phase 3（Week 9-12）：游戏开发基础

**目标**：实现游戏开发基础功能

| 任务 | 交付物 | 验收标准 |
|------|--------|----------|
| 实现 Game Resource Generator | `game_resource_generator.rs` | 美术资源生成成功 |
| 实现游戏原型开发 Agent 组合 | `game_prototype_agents.rs` | 游戏原型可运行 |
| 实现游戏美术资源生成 | `game_art_generator.rs` | 角色 + 场景 + UI 生成 |
| 用户测试（游戏原型） | 测试报告 | 用户反馈收集 |

**交付**：游戏原型开发 MVP

### Phase 4（Week 13-16）：跨平台游戏

**目标**：实现跨平台游戏开发

| 任务 | 交付物 | 验收标准 |
|------|--------|----------|
| 实现 Cross-Platform Game Adapter | `cross_platform_game_adapter.rs` | 跨平台适配成功 |
| 实现 Multi-Agent Consensus | `multi_agent_consensus.rs` | 多 Agent 共识达成 |
| 实现跨平台兼容性检查 | `compatibility_checker.rs` | 兼容性检查通过 |
| 用户测试（跨平台游戏） | 测试报告 | 用户反馈收集 |

**交付**：跨平台游戏开发 MVP

### Phase 5（Week 17-20）：游戏性能优化

**目标**：实现游戏性能优化

| 任务 | 交付物 | 验收标准 |
|------|--------|----------|
| 实现 Game Performance Optimizer | `game_performance_optimizer.rs` | 性能优化成功 |
| 实现 Game Profiler | `game_profiler.rs` | 性能分析准确 |
| 实现性能瓶颈识别 | `bottleneck_identifier.rs` | 瓶颈识别准确 |
| 用户测试（性能优化） | 测试报告 | 用户反馈收集 |

**交付**：游戏性能优化 MVP

### Phase 6（Week 21-24）：优化与发布

**目标**：优化体验，发布 v1.0

| 任务 | 交付物 | 验收标准 |
|------|--------|----------|
| 优化 Self-Optimizing Router | 优化报告 | 选择准确率 ≥ 85% |
| 优化 Privacy-Preserving Orchestrator | 优化报告 | 隐私保护有效 |
| 优化 Adaptive Workflow | 优化报告 | 工作流自适应成功 |
| 完善文档 | `docs/USER-GUIDE.md` | 文档完整 |
| 发布 v1.0 | GitHub Release | CHANGELOG + 截图 |

**交付**：v1.0 发布

---

## 六、商业模式（待设计）

### 6.1 定价策略（待讨论）

| 模式 | 说明 | 优点 | 缺点 |
|------|------|------|------|
| **订阅制** | 月费 $29/¥99 | 稳定收入 | 用户门槛高 |
| **Freemium** | 免费版 + 付费版 | 用户增长快 | 免费版功能有限 |
| **按次付费** | 每次执行收费 | 低门槛 | 收入不稳定 |

### 6.2 免费版 vs 付费版（待设计）

| 功能 | 免费版 | 付费版 |
|------|--------|--------|
| 执行次数 | 100 次/月 | 无限 |
| Agent 选择 | 基础 Agent | 高级 Agent |
| 隐私保护 | 基础 | 高级 |
| 跨平台 | 单平台 | 多平台 |
| 游戏开发 | ❌ | ✅ |

---

## 七、验收标准

### 7.1 功能验收

| 功能 | 标准 | 场景验证 |
|------|------|----------|
| **场景识别** | 识别准确率 ≥ 85% | 输入"开发一个 2D 游戏" → 识别为游戏开发 |
| **Self-Optimizing** | 选择准确率 ≥ 85% | 根据历史数据优化 Agent 选择 |
| **Privacy-Preserving** | 代码本地处理率 100% | 代码文件不上传云端 |
| **Adaptive Workflow** | 跨平台适配成功率 ≥ 90% | 自动调整工作流 |
| **Multi-Agent Consensus** | 共识达成率 ≥ 80% | 多 Agent 投票确认 |
| **Game Resource Generator** | 资源生成成功率 ≥ 90% | 游戏美术资源生成 |
| **Cross-Platform Adapter** | 跨平台兼容性 ≥ 85% | 桌面 + 移动 + 小程序 |
| **Performance Optimizer** | 性能提升 ≥ 20% | 游戏性能优化 |

### 7.2 技术验收

| 检查项 | 标准 | 验证方式 |
|--------|------|----------|
| **Agent Adapter 接口** | 所有 Agent 实现统一 trait | `cargo test --workspace` |
| **本地 Agent 测试** | Claude Code + Codex 执行成功 | 用户验收 |
| **Token 追踪** | 实时 Token 用量显示 | 执行中显示 input/output/total tokens |
| **健康监控** | EWMA 5 分钟更新 + 熔断触发 | 健康分数 ≤ 80 触发熔断 |
| **测试覆盖** | 80%+ 单元测试 | `cargo tarpaulin` |

---

## 八、风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| **场景识别不准确** | Agent 选择错误 | LLM 分类 + 反问确认 + 用户手动修正 |
| **Self-Optimizing 不准确** | 选择次优 Agent | 基于历史数据 + 持续优化 |
| **隐私保护不充分** | 数据泄露 | 严格本地处理 + 加密存储 |
| **跨平台兼容性问题** | 适配失败 | Multi-Agent Consensus + 自动修复 |
| **游戏资源质量差** | 用户体验差 | 多 Agent 生成 + 质量筛选 |
| **性能优化效果差** | 性能提升小 | 多轮优化 + 性能分析 |

---

## 九、里程碑

| Milestone | 日期 | 交付物 | 用户验证 |
|-----------|------|--------|----------|
| **M1**: Web + 小程序 MVP | 2026-07-20 | Web + 小程序开发 | 用户输入一句话 → 自动生成代码 |
| **M2**: 桌面端开发 | 2026-08-17 | 桌面端开发 | 跨平台桌面应用生成 |
| **M3**: 游戏原型 | 2026-09-14 | 游戏原型开发 | 输入游戏描述 → 生成可运行原型 |
| **M4**: 跨平台游戏 | 2026-10-12 | 跨平台游戏 | 桌面 + 移动 + 小程序 |
| **M5**: 游戏性能优化 | 2026-11-09 | 性能优化 | 性能提升 ≥ 20% |
| **M6**: v1.0 发布 | 2026-12-07 | 完整产品 | 代码 + 游戏全场景覆盖 |

---

**签署**: ACP-UI Team + yan_fan_tian
**日期**: 2026-06-23