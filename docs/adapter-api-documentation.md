# Agent Platform Adapter API Documentation

## Overview

Agent Platform provides 19 unified Adapters for different development scenarios:

| Category | Adapters |
|----------|----------|
| **CLI Agents** | Claude Code, Codex |
| **Desktop Development** | Tauri Desktop, Electron Desktop |
| **Mobile Development** | Flutter SDK |
| **Mini Program** | WeChat Mini Program |
| **Game Development** | Unity, Godot |
| **Marketing Content** | Kimi, Jimeng (Image), Kling (Video) |
| **Office Automation** | WPS Office |
| **Containerized Deployment** | Docker, Kubernetes |
| **Enterprise Platforms** | Douyin, Kuaishou, DingTalk, Feishu |

---

## Adapter Interface

All Adapters implement the unified `AgentAdapter` trait:

```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn adapter_type(&self) -> AdapterType;
    fn capabilities(&self) -> Vec<Capability>;
    
    async fn configure(&mut self, config: AgentConfig) -> Result<(), AgentError>;
    async fn validate_config(&self) -> Result<bool, AgentError>;
    async fn execute(&self, task: AgentTask) -> Result<AgentResult, AgentError>;
    async fn cancel(&self, task_id: &str) -> Result<(), AgentError>;
    
    fn status(&self) -> AgentStatus;
    async fn health(&self) -> HealthMetrics;
    fn cost_estimate(&self, task: &AgentTask) -> CostEstimate;
    fn actual_cost(&self, task_id: &str) -> Option<ActualCost>;
    fn token_usage_summary(&self, last_n: u32) -> Vec<TokenUsage>;
    
    async fn update_health(&mut self, result: &AgentResult);
    fn is_circuit_breaker_allowed(&self) -> bool;
    async fn reset_circuit_breaker(&mut self);
}
```

---

## Phase 8: Development Tools

### 1. WeChat Mini Program Adapter

**ID:** `wechat-miniprogram`

**Capabilities:**
- `miniprogram_create` - Create new mini program project
- `miniprogram_compile` - Compile project
- `miniprogram_preview` - Generate preview QR code
- `miniprogram_upload` - Upload to WeChat platform
- `miniprogram_audit` - Code audit
- `miniprogram_build_npm` - Build NPM dependencies

**Usage Example:**
```rust
let adapter = WeChatMiniProgramAdapter::new();
let config = AgentConfig {
    metadata: HashMap::from([
        ("app_id".to_string(), "wx1234567890".to_string()),
        ("project_path".to_string(), "./my-miniprogram".to_string()),
    ]),
    ..Default::default()
};
adapter.configure(config).await?;

let task = AgentTask {
    description: "编译小程序".to_string(),
    input: TaskInput::Text("./my-miniprogram".to_string()),
    ..Default::default()
};
let result = adapter.execute(task).await?;
```

### 2. Flutter SDK Adapter

**ID:** `flutter-sdk`

**Capabilities:**
- `flutter_create` - Create new Flutter project
- `flutter_run` - Run app with hot reload
- `flutter_build_android` - Build Android APK/AppBundle
- `flutter_build_ios` - Build iOS IPA
- `flutter_build_web` - Build Web application
- `flutter_test` - Run tests with coverage
- `flutter_analyze` - Code analysis
- `flutter_pub` - Dependency management

**Supported Platforms:**
- Android, iOS, Web, Windows, macOS, Linux

### 3. Docker Adapter

**ID:** `docker`

**Capabilities:**
- `dockerfile_generate` - Generate Dockerfile (Rust/Node/Flutter/Python)
- `docker_build` - Build Docker image
- `docker_run` - Run container
- `docker_push` - Push to registry
- `docker_compose` - Docker Compose orchestration
- `container_management` - Container lifecycle management

### 4. Kubernetes Adapter

**ID:** `kubernetes`

**Capabilities:**
- `k8s_yaml_generate` - Generate Deployment/Service/Ingress YAML
- `k8s_apply` - Apply configuration
- `k8s_deploy` - Deploy application
- `k8s_scale` - Scale replicas
- `k8s_logs` - View Pod logs
- `k8s_ingress` - Ingress configuration

---

## Phase 9: Enterprise Platforms

### 5. Douyin Open Platform Adapter

**ID:** `douyin-open`

**Capabilities:**
- `douyin_oauth` - OAuth authentication
- `douyin_publish` - Publish short video
- `douyin_video_list` - Get video list
- `douyin_analytics` - Data analytics (play/like/comment)
- `douyin_live` - Live stream management

**Configuration:**
```rust
metadata: HashMap::from([
    ("client_id".to_string(), "your_app_key".to_string()),
    ("client_secret".to_string(), "your_app_secret".to_string()),
])
```

### 6. Kuaishou Open Platform Adapter

**ID:** `kuaishou-open`

**Capabilities:**
- `kuaishou_oauth` - OAuth authentication
- `kuaishou_publish` - Publish short video
- `kuaishou_video_list` - Get video list
- `kuaishou_analytics` - Data analytics

### 7. DingTalk Open Platform Adapter

**ID:** `dingtalk-open`

**Capabilities:**
- `dingtalk_oauth` - OAuth authentication
- `dingtalk_message` - Send work notification
- `dingtalk_approval` - Create approval workflow
- `dingtalk_group` - Group management
- `dingtalk_schedule` - Schedule management

**Features:**
- Work notification messages
- Approval workflow creation
- Group management (create/add/remove)
- Schedule/calendar integration

### 8. Feishu/Lark Open Platform Adapter

**ID:** `feishu-open`

**Capabilities:**
- `feishu_oauth` - OAuth authentication
- `feishu_message` - Send messages
- `feishu_document` - Create/manage documents
- `feishu_bitable` - Bitable (multi-dimensional table) operations
- `feishu_folder` - Folder management
- `feishu_schedule` - Schedule management

**Supported Document Types:**
- Docx (document)
- Sheet (spreadsheet)
- Bitable (multi-dimensional table)
- MindNote (mind mapping)

---

## One-Shot Interface

Unified entry point for all agent operations:

```rust
let interface = OneShotInterface::new();
let request = OneShotRequest {
    input: "帮我写一段抖音广告文案".to_string(),
    preferred_agent: None,
    max_cost: Some(0.5),
    ..Default::default()
};

let response = interface.process(request);
// Response contains:
// - detected_role: UserRole::Marketer
// - detected_scene: "copywriting"
// - selected_agent: "kimi-marketing"
// - selection_reason: "Selected kimi-marketing for scene: copywriting"
// - transparency: DAG visualization, cost estimate, proficiency scores
```

## User Role Detection

Automatic detection of 8 user roles based on input keywords:

| Role | Keywords |
|------|----------|
| Developer | 代码, 编程, bug, 函数, API, Rust, TypeScript |
| Marketer | 文案, 营销, 推广, 抖音, 快手, 小红书 |
| Designer | 设计, UI, UX, 界面, 图标, Figma |
| Finance | 财务, 报表, Excel, 预算, 成本 |
| Gamer | 游戏, Unity, Godot, 角色, 动画 |
| Writer | 写作, 文章, 小说, 翻译 |
| Analyst | 分析, 数据, 统计, 图表, KPI |
| General | Default for unrecognized input |

## Health Monitoring

All Adapters include EWMA-based health monitoring with circuit breaker:

- Health score: 0-100 (EWMA weighted average)
- Success rate tracking
- Latency monitoring
- Circuit breaker (Closed/Open/HalfOpen states)
- Automatic recovery

## Cost Estimation

Zero-cost for CLI-based Adapters (local tools).

API-based Adapters charge based on:
- Token usage
- API call count
- Data volume

---

## Summary

| Adapter Count | Categories |
|---------------|------------|
| 19 Adapters | 9 Categories |
| CLI: 6 | API: 13 |
| Free: 8 | Paid: 11 |

**Developer Coverage: 100%**
- Games: Unity + Godot
- Web: Claude Code + Codex
- Desktop: Tauri + Electron
- Mobile: Flutter SDK
- Mini Program: WeChat
- Deployment: Docker + Kubernetes
- Enterprise: Douyin + Kuaishou + DingTalk + Feishu