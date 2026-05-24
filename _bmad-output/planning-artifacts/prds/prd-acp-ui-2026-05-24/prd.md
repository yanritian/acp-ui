---
title: ACP-UI 产品需求文档
status: final
created: 2026-05-24
updated: 2026-05-24
project: acp-ui
---

# ACP-UI 产品需求文档

> **文档状态**: Draft
> **产品经理**: John (BMad PM Agent)
> **创建日期**: 2026-05-24

---

## 一、产品定位

### 1.1 核心价值

**ACP-UI = 中国开发工具链的 AI 编排平台**

```
核心价值 = 
  Docker-like Agent编排系统 +
  Claude Code/Codex 基底 +
  多维记忆系统 +
  智能路由与自进化 +
  中国工具链集成（HBuilderX/微信DevTools）+
  可视化多Agent协作
```

### 1.2 差异化优势

| 优势 | 说明 | 对标竞品 |
|------|------|----------|
| **中国工具链** | HBuilderX/微信DevTools/阿里Coding Plan | 国际竞品不支持 |
| **Docker-like Agent** | Base/Template/Instance/Team 模式 | 国内竞品无此架构 |
| **多维记忆** | 输入→Agent→会话全流程记录 | OpenDevin部分支持 |
| **智能路由** | 自动选择Claude Code/Codex/Team | 独创 |
| **自进化** | 从执行历史学习模式生成Skills | 独创 |
| **可视化GUI** | Tauri Dashboard + 实时监控 | ChatDev仅CLI |
| **开源免费** | Apache 2.0 + 自备API密钥 | 商业产品收费 |

### 1.3 竞品定位

| 维度 | 国际竞品 | 国内竞品 | ACP-UI |
|------|----------|----------|--------|
| 多Agent | MetaGPT/OpenDevin ✅ | 商业产品 ❌ | ✅ |
| 本地私有化 | OpenDevin ✅ | 通义灵码 ✅ | ✅ |
| 中国工具链 | ❌ | ❌ | **✅ 独家** |
| 可视化GUI | OpenDevin Web ✅ | ChatDev CLI ❌ | ✅ |
| Docker-like架构 | ❌ | ❌ | **✅ 独创** |
| 智能路由 | ❌ | ❌ | **✅ 独创** |

---

## 二、目标用户

### 2.1 用户画像

**主要用户：个人开发者**

| 特征 | 描述 |
|------|------|
| **角色** | 全栈开发者，独立开发者 |
| **痛点** | 单 Agent 无法处理复杂任务，需要多个 Agent 协作 |
| **场景** | 多客户端开发（小程序 + App + Web），需要统一控制 |
| **期望** | 本地部署，隐私保护，完全控制 Agent 行为 |
| **规模** | 个人自用，无需团队协作功能 |

### 2.2 使用场景

| 场景 | 描述 |
|------|------|
| **多 Agent 并行开发** | 同时用 Claude Code 和 Codex 处理不同模块 |
| **开发工具控制** | 通过 ACP-UI 控制 HBuilderX 编译、微信 DevTools 调试 |
| **远程监控** | 手机端实时查看 Agent 执行状态、日志输出 |
| **多平台 Bot** | 飞书/Telegram/Discord 接收 Agent 执行结果 |

### 2.3 用户旅程

**UJ-001: 多 Agent 并行开发流程**

```
1. 创建任务 → 用户在 ACP-UI Dashboard 输入开发任务描述
2. 选择 Agent → 选择 Claude Code 处理前端，Codex 处理后端
3. 启动执行 → 点击"并行执行"，两个 Agent 同时启动
4. 监控进度 → Dashboard 实时显示两个 Agent 的输出流（stdout/stderr 分离）
5. 处理错误 → Agent A 报错时，用户可暂停 Agent B，修复 A 后继续
6. 查看结果 → 任务完成，Dashboard 显示耗时、修改文件列表、生成摘要
```

**UJ-002: 开发工具控制流程**

```
1. 连接工具 → 在配置面板添加 HBuilderX 路径
2. 选择项目 → 从下拉菜单选择小程序项目
3. 执行编译 → Agent 发送编译指令，HBuilderXAdapter 执行
4. 监控输出 → Console 面板显示编译日志、错误行号
5. 调试修复 → Agent 根据错误自动修复，重新编译验证
```

### 2.4 成功指标

| 指标 | 目标值 | 验证方法 |
|------|--------|----------|
| **SM-001: Agent 真实输出率** | 100% 任务有真实 stdout/stderr | 手动检查每次任务输出不为空且非 mock |
| **SM-002: 并行 Agent 数** | ≥2 Agent 同时运行无阻塞 | 启动 2 Agent，观察 Dashboard 两个输出流并行更新 |
| **SM-003: 首次成功率** | Phase 1 任务 80% 无假成功 | 统计任务执行记录中 "假成功" 比例 < 20% |
| **SM-004: 工具适配器可用** | ≥1 适配器能编译并启动项目 | HBuilderX 或微信 DevTools 能编译小程序并打开预览 |
| **SM-005: 宠物状态同步** | 宠物状态与 Agent 状态延迟 < 500ms | Agent 启动时宠物变为"活跃"，Agent 空闲时变为"休息" |

**反指标**:
- Agent 崩溃率 > 5% → 触发自动重启，记录崩溃日志
- WebSocket 断连率 > 10次/小时 → 检查网络或降级为本地模式

---

## 三、技术架构

### 3.1 三层沙箱架构

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 0: 核心沙箱（Phase 0-2）                               │
│ ├── Agent 进程管理                                           │
│ ├── MCP 基础连接                                             │
│ ├── 权限检查                                                 │
│ ├── cwd 限制                                                 │
│ └── 远程 WebSocket + Token 认证                              │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: 增强沙箱（Phase 3-4）                               │
│ ├── 记忆系统（作用域 + 任务注入）                             │
│ ├── Skills 隔离（每个 Agent 独立）                            │
│ ├── Hooks 执行（PreToolUse/PostToolUse）                      │
│ ├── 图片输出支持                                             │
│ └── 仪表盘基础                                               │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: 完整沙箱（Phase 5+）                                │
│ ├── Hermes 宠物系统                                          │
│ ├── GUI 完整仪表盘                                           │
│ ├── 多平台卡片（飞书/Telegram/Discord）                       │
│ ├── 操作按钮（不只是文字）                                    │
│ └── 自定义协议扩展                                           │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Agent 沙箱支持

| Agent 类型 | 沙箱能力 | 状态 |
|------------|---------|------|
| **Claude Code** | MCP + Skills + Hooks + 记忆 | 已有基础 |
| **Codex** | MCP + Skills + Hooks | 需确认集成 |
| **其他 Agent** | 可扩展接口 | 未来支持 |

### 3.3 可插拔调度层

```
调度层架构：

┌─────────────────────────────────────────────────────────┐
│              ExecutiveAgentManager                       │
│  ┌─────────────────────────────────────────────────────┐│
│  │ SchedulerType:                                       ││
│  │   - HermesRust (默认)                                ││
│  │   - CodingPlan (阿里 Coding Plan)                    ││
│  │   - Custom (未来可扩展)                              ││
│  └─────────────────────────────────────────────────────┘│
│                          │                               │
│                          ▼                               │
│  ┌─────────────────────────────────────────────────────┐│
│  │ Hermes AgentLoop (Rust Native)                      ││
│  │  ├── hermes-agent                                    ││
│  │  ├── hermes-config                                   ││
│  │  ├── hermes-core                                     ││
│  │  ├── hermes-tools                                    ││
│  │  ├── hermes-intelligence                             ││
│  │  ├── hermes-environments                             ││
│  │  └── hermes-skills                                   ││
│  └─────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────┘
```

### 3.4 多端架构

| 端 | 技术栈 | 优先级 | 状态 |
|----|--------|--------|------|
| **Web** | Vue 3.5 + TypeScript | P0 | ⚠️ 功能缺失 |
| **Windows-App** | Tauri 2 + Rust | P0 | ⚠️ 功能缺失 |
| **Flutter 移动端** | Flutter 3.22+ | P1 暂缓 | ❌ 构建问题多 |

---

## 四、功能需求

### 4.1 核心功能（P0）

#### FR-001: 多 Agent 会话

**描述**: 支持同时运行多个 Agent，每个 Agent 有独立的会话和输出。

**验收标准**:
- 能选择 Claude Code 或 Codex 作为 Agent
- 能同时启动 2-3 个 Agent 并行执行
- 每个 Agent 的输出独立显示

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 真实输出 | stdout 或 stderr 非空且非 "mock" 字串 | 执行简单任务 `echo "test"`，检查输出包含 "test" |
| 真实错误 | stderr 包含实际错误信息 + Agent exit_code ≠ 0 | 执行无效命令 `invalid_cmd`，检查 stderr 包含 "not found" |
| 并行无阻塞 | 2 Agent 输出流同时更新，无串行等待 | 启动 2 Agent 同时执行，观察 Dashboard 日志交错更新 |

**依赖**: 无前置依赖

#### FR-002: 远程控制

**描述**: 通过 WebSocket 远程控制 Agent，支持 Token 认证。

**验收标准**:
- WebSocket 连接带 Token 认证
- 远程命令有回执（成功/失败）
- 能获取 Agent 状态快照
- 能远程暂停/继续/取消 Agent

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| Token 认证 | 无 Token 连接被拒绝，有效 Token 连接成功 | 发送 WS 连接不带 Token → 期望 401；带有效 Token → 期望 200 |
| 命令回执 | 每个远程命令返回 `{success: bool, message: string}` | 发送 `pause` 命令，检查回执包含 `success: true` |
| 状态快照 | 快照包含 `{status, running_tasks, last_error}` | 发送 `get_snapshot`，验证返回 JSON 包含必填字段 |

**依赖**: FR-001（需要 Agent 先能运行）

#### FR-003: 多客户端适配器

**描述**: 支持控制多种开发工具。

**验收标准**:
- BrowserAdapter: 控制浏览器，截图，Console 监听
- HBuilderXAdapter: 编译/运行小程序
- WeChatDevToolsAdapter: 调试微信小程序，API 监控
- AndroidStudioAdapter: Gradle 构建，ADB 安装

**操作化验证 (Phase 1 只需其一)**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| HBuilderX 编译 | 能编译小程序项目并生成 dist 目录 | 配置 HBuilderX 路径，执行 `compile`，检查 dist/ 存在 |
| 微信 DevTools 调试 | 能打开小程序预览并监听 Console | 配置 DevTools 路径，执行 `debug`，检查 Console 输出捕获 |

**依赖**: FR-001（Agent 需能发送指令）

### 4.2 增强功能（P1）

#### FR-004: 记忆系统

**描述**: 多级记忆系统，支持作用域和任务注入。

**验收标准**:
- 支持 global/agent/session/task 四种作用域
- 任务发送前能注入相关记忆
- 记忆可搜索和筛选

#### FR-005: Skills 隔离

**描述**: 每个 Agent 有独立的 Skills 配置。

**验收标准**:
- Agent 只能调用配置的 Skills
- 未配置的 Skills 调用报错
- Skills 动态加载

#### FR-006: Hooks 执行

**描述**: 支持 PreToolUse 和 PostToolUse Hooks。

**验收标准**:
- PreToolUse 能阻断工具执行
- PostToolUse 能记录输出
- 每个 Agent 独立 Hooks

### 4.3 完整功能（P1）

#### FR-007: Hermes 宠物系统

**描述**: Agent 拟人化宠物头像，显示活动和情绪。

**验收标准**:
- 宠物头像根据 Agent 状态变化
- 支持点击交互
- 与 Agent 执行状态实时同步

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 状态同步 | Agent 状态变化后宠物 UI 更新延迟 < 500ms | 启动 Agent，观察宠物从 "休息" → "活跃"；测量时间差 |
| 真实数据 |宠物状态字段值来自 Agent 实时状态而非静态 mock | 检查宠物状态字段 `{activity, mood}` 来自 Agent 状态对象 |
| 点击响应 | 点击宠物显示 Agent 详情面板 | 点击宠物头像，验证详情面板弹出并显示 Agent 信息 |

**依赖**: FR-001（需要 Agent 状态数据）

#### FR-008: 多平台 Bot

**描述**: 支持飞书/Telegram/Discord Bot。

**验收标准**:
- Bot 能接收任务指令
- 返回图片 + 卡片 + 操作按钮
- 不只是文字回复

---

### 4.4 智能系统（P1）

#### FR-011: 自我修复系统

**描述**: Agent 能自动检测异常状态并执行修复动作。

**验收标准**:
- Agent 进程崩溃后自动重启
- LLM Rate Limited 时自动切换 Fallback 模型
- Context 溢出时自动压缩历史
- 修复动作记录到记忆系统供学习

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 崩溃恢复 | Agent 进程退出后 10s 内重启 | 手动 kill Agent 进程，观察 Dashboard 显示 '重启中' 后恢复 |
| Fallback | Rate Limited 后切换备用模型 | 模拟 429 响应，检查日志显示 'Switched to fallback model' |
| Context 压缩 | 消息超过阈值自动压缩 | 发送 100+ 条消息，检查 context 长度下降 |

**依赖**: Hermes FallbackChain 扩展

#### FR-012: 智能调度系统

**描述**: 根据任务复杂度自动选择 Agent 数量和模型。

**验收标准**:
- 简单任务（单文件修改）→ 单Agent + Haiku
- 中等任务（多文件重构）→ 单Agent + Sonnet
- 复杂任务（跨模块开发）→ 多Agent协作
- 自动调整 temperature/max_tokens

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 简单任务路由 | '修改一行代码' 任务用 Haiku | 发送简单任务，检查 model 字段为 haiku |
| 复杂任务路由 | '重构整个模块' 任务启动多Agent | 发送复杂任务，检查 SubAgentOrchestrator 调用记录 |

**依赖**: TaskComplexityAnalyzer 实现

#### FR-013: 自进化系统（P2）

**描述**: Agent 从执行历史学习模式，自动优化配置和生成 Skills。

**验收标准**:
- 分析执行历史识别成功/失败模式
- 自动调整 Agent 参数提升成功率
- 成功模式自动生成 Skill 供复用
- 进化日志可追溯

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 模式学习 | 执行 10+ 次同类型任务后识别模式 | 执行 10 次小程序编译任务，检查 Pattern 表有记录 |
| Skill 生成 | 成功模式自动创建 Skill | 检查 Skills 目录新增 auto-generated skill |
| 参数优化 | 学习后 Agent 成功率提升 | 对比学习前后成功率统计 |

**依赖**: FR-018（需要记忆系统存储学习结果）

---

### 4.5 Agent 编排系统（P1）

#### FR-014: Agent Registry 系统

**描述**: Docker-like Agent 配置仓库，管理 Agent Base 和 Template。

**验收标准**:
- 支持定义 Agent Base（claude-code-base, codex-base）
- 支持从 Base 派生 Agent Template
- Template 可配置 skills/hooks/rules/mcp/memory
- 配置以 YAML 格式存储

**Docker 映射**:
| Docker 概念 | ACP-UI 概念 |
|-------------|-------------|
| Image | Agent Base |
| 派生 Image | Agent Template |
| Container | Agent Instance |
| Volume | Agent Memory/Session |
| Registry | YAML 配置目录 |

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| Base 加载 | YAML 文件加载成功 | 创建 claude-code-base.yaml，执行 `acp-agent base list` 显示 |
| Template 派生 | 从 Base 创建 Template | `acp-agent template create frontend-dev --from claude-code-base` 成功 |

**依赖**: 无前置依赖

#### FR-015: Agent Instance 管理

**描述**: Docker-like Agent 实例管理，启动/停止/监控 Agent 进程。

**验收标准**:
- 从 Template 启动 Agent Instance（Container）
- 每个 Instance 有独立 pid/session_id/volume
- 支持 ps/stop/logs/exec 命令
- Instance 状态实时更新

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 实例启动 | Template → Instance 进程启动 | `acp-agent run frontend-dev` 后 `ps` 显示 running |
| 独立 Volume | 每个 Instance 有独立数据目录 | 检查 `config/volumes/frontend-dev/` 目录存在 |

**依赖**: FR-014

#### FR-016: Team 编排系统

**描述**: 多 Agent Team 编排，定义 Team Template 和同步点。

**验收标准**:
- Team Template 定义多个 Agent 组合
- 支持 parallel/sequential/hybrid 执行策略
- Sync Points 定义协作同步点
- Team Instance 运行时协调

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| Team 创建 | Team Template 定义成功 | 创建 team-fullstack.yaml，包含 2 agents |
| Team 启动 | Team Instance 启动多个 Agent | `acp-agent team run team-fullstack` 启动 2 container |
| 同步点触发 | Agent 到达 Sync Point 自动暂停 | 触发 code-review sync point，检查 Agent 状态变为 waiting |

**依赖**: FR-015

#### FR-017: Claude Code/Codex 配置同步（P2）

**描述**: 读取/导入/导出 Claude Code 和 Codex 的本地配置。

**验收标准**:
- 读取 Claude Code ~/.claude/ 目录配置
- 导入 skills/hooks/rules 到 ACP-UI Registry
- 导出 ACP-UI Template 配置到 Claude Code
- 显示配置差异对比

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 配置读取 | 显示 Claude Code 当前配置 | `acp-agent sync claude-code --show-config` 输出 settings/skills |
| Skills 导入 | Claude skills 导入 Registry | 执行导入后 `acp-agent base inspect` 显示新增 skills |

**依赖**: FR-014

---

### 4.6 多维记忆系统（P1）

#### FR-018: 多维记忆系统

**描述**: 多层级记录系统，覆盖输入→Agent→会话全流程。

**层级结构**:
```
输入层 (InputLog)
├── source: app / bot / api
├── input_type: text / image / document
└── route_to: claude-code / codex / team

Agent层 (AgentFlow)
├── flow_steps: receive → analyze → tool_calls → respond
├── internal_state: context_length, thinking_tokens
└── state_snapshots (JSON)

会话层 (Session)
├── messages (SQLite)
├── embeddings (Chroma 向量)
└── semantic_index: 语义检索
```

**数据存储**:
| 层级 | 数据类型 | 存储方案 |
|------|----------|----------|
| 输入日志 | 结构化 | SQLite |
| Agent 流程 | 时序 + JSON | SQLite |
| 会话消息 | 结构化 + 向量 | SQLite + Chroma |
| 路由决策 | 结构化 | SQLite |
| 长期记忆 | 向量 | Chroma |

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 输入记录 | 每个 input 有 log 记录 | 发送输入后查询 `SELECT * FROM input_logs` |
| Agent 流程 | 记录每个工具调用步骤 | 执行任务后查询 `flow_steps_json` 包含 tool_calls |
| 语义检索 | 能搜索相关历史消息 | 发送查询，向量检索返回相关消息 |

**依赖**: SQLite + Chroma 集成

#### FR-019: 智能清理系统（P2）

**描述**: 基于 LLM 状态的自动清理策略。

**验收标准**:
- context_length > 80% → 智能压缩旧消息
- session_age > 7 days → 归档到长期记忆
- error_rate > threshold → 清理失败流程
- 清理动作记录到 clear_history

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 自动压缩 | context 超阈值自动触发 | 发送大量消息触发压缩，检查 context_length 下降 |
| LLM 摘要 | 压缩生成有效摘要 | 检查压缩后的摘要内容保留关键信息 |

**依赖**: FR-018

#### FR-020: 智能路由系统

**描述**: 根据输入特征自动决定使用 Claude Code / Codex / Team。

**路由决策因子**:
- task_type: frontend / backend / fullstack
- input_type: text / image / document
- complexity: simple / medium / complex
- agent_load: 当前运行 Agent 数
- historical_success_rate: 历史成功率

**路由规则**:
| 条件 | 路由目标 | 理由 |
|------|----------|------|
| frontend + simple | Claude Code (Haiku) | 快速响应 |
| backend + complex | Codex (Sonnet) | 深度推理 |
| image 输入 | Claude Code (Sonnet) | 视觉优势 |
| document 输入 | Team | 多 Agent 分工 |
| fullstack + complex | Team | 并行处理 |

**操作化验证**:
| 验证项 | 条件 | 测试方法 |
|--------|------|----------|
| 路由记录 | 每次路由有决策记录 | 发送输入后查询 `route_decisions` 表有记录 |
| 图片路由 | 图片输入路由到 Claude Code | 发送图片，检查 decision 为 claude-code |

**依赖**: FR-018

---

## 五、非功能需求

### NFR-001: 性能

| 要求 | 标准 |
|------|------|
| Agent 启动延迟 | < 3 秒 |
| WebSocket 推送延迟 | < 100ms |
| 日志虚拟滚动 | 10000+ 条不卡顿 |

### NFR-002: 安全

| 要求 | 标准 |
|------|------|
| SQL 参数绑定 | 无拼接 |
| Token 不明文打印 | 只存 hash |
| 无硬编码密钥 | grep 验证 |

### NFR-003: 可靠性

| 要求 | 标准 |
|------|------|
| Agent 崩溃自动恢复 | 重启机制 |
| WebSocket 断线重连 | 自动重连 |
| 状态持久化 | SQLite |

---

## 六、当前问题

### 6.1 P0 阻塞问题

| 问题 | 影响 | 修复优先级 |
|------|------|-----------|
| Web 构建失败 | vue-tsc 找不到 | 🔴 立即 |
| Tauri/Rust 构建失败 | Hermes crates 未链接 | 🔴 立即 |
| 功能全是"壳" | 无真实 Agent 输出 | 🔴 立即 |

### 6.2 功能缺失

| 功能 | 当前状态 | 目标状态 |
|------|---------|---------|
| 多 Agent 并行 | 假响应 | 真实调用 + 真实输出 |
| 远程控制 | 无认证 + 无回执 | Token + 回执 |
| 仪表盘 | Mock 数据 | 真实数据 |
| 多平台 Bot | 只打印日志 | 真实执行 + 卡片回复 |

---

## 七、实施优先级

### Phase 完成门控定义

**门控规则**: Phase N 完成需满足:
1. 所有 checkbox 任务项完成
2. 相关 FR 操作化验证全部通过
3. 无遗留 CRITICAL/HIGH 阻塞问题
4. 代码审查通过（`code-reviewer` agent）

**允许跨 Phase**: 若某任务阻塞但非关键，可标记 `[DEFERRED]` 进入下一 Phase，但需在决策日志记录原因和 revisit 条件。

### Phase 0: 构建修复（立即）

**完成门控**: `npm run build` + `cargo check` 全部 PASS，无编译错误。

- [ ] 修复 Web 构建（vue-tsc）
- [ ] 修复 Tauri/Rust 构建（Hermes crates 链接）
- [ ] 验证 npm run build PASS
- [ ] 验证 cargo check PASS
- [ ] SQLite + Chroma 集成基础

### Phase 1: Agent 系统 + 记忆系统

**完成门控**: FR-001 + FR-003 + FR-007 + FR-014 + FR-015 + FR-018 + FR-020 操作化验证通过。

- [ ] Agent 进程能真实启动
- [ ] Agent 能接收真实 prompt
- [ ] Agent 能输出真实结果或真实错误
- [ ] 不再显示假成功（SM-001: 100% 真实输出）
- [ ] 至少一个开发工具适配器可用（HBuilderX 或微信 DevTools）
- [ ] Hermes 宠物系统基础功能
- [ ] FR-014 Agent Registry (Base + Template YAML)
- [ ] FR-015 Agent Instance (Container 启动/停止)
- [ ] FR-016 Team 编排 (基础)
- [ ] FR-018 多维记忆系统 (input_logs + agent_flows + sessions)
- [ ] FR-020 智能路由 (基础任务类型分析)
- [ ] FR-011 自我修复 (基础崩溃恢复)
- [ ] FR-012 智能调度 (基础复杂度分析)

### Phase 2: 增强功能

**完成门控**: FR-002 + FR-004 + FR-016(完整) + FR-019 操作化验证通过。

- [ ] WebSocket + Token 认证
- [ ] 远程命令回执
- [ ] Agent 状态快照
- [ ] FR-016 Team 编排 (完整 sync points)
- [ ] FR-017 配置同步 (Claude Code/Codex)
- [ ] FR-019 智能清理 (LLM摘要压缩)
- [ ] 记忆系统（作用域完整）
- [ ] Skills 隔离
- [ ] Hooks 执行
- [ ] FR-013 自进化 (基础模式学习)

### Phase 3: 完整体验

**完成门控**: FR-003(完整) + FR-008 + FR-013(完整) 操作化验证通过。

- [ ] 多客户端适配器 (完整：HBuilderX + 微信DevTools + AndroidStudio)
- [ ] 多平台 Bot 卡片
- [ ] FR-013 自进化 (完整Skill生成)
- [ ] Flutter 暂缓，降级为 H5
- [ ] 企业私有化部署包
- [ ] 开源发布（Apache 2.0）

---

## 八、验收标准

| 功能 | 验收标准 |
|------|---------|
| **多 Agent** | 真实调用多个 Agent，展示各自的输出/错误/耗时 |
| **远程控制** | WebSocket 带 Token，远程命令有回执 |
| **调度层** | Hermes Rust + Coding Plan 可切换 |
| **多客户端** | 能控制 HBuilderX/微信 DevTools/浏览器 |
| **构建** | npm run build + cargo check PASS |
| **安全** | SQL 参数绑定 + Token hash |

---

## 九、附录

### A. 决策记录

见 `.decision-log.md`

### B. 相关文档

- `docs/system-architecture.md`
- `docs/project-completion-plan.md`
- `docs/superpowers/plans/2026-05-04-agent-teams-platform-completion.md`

### C. Hermes Crates

- `src-tauri/hermes-crates/hermes-agent`
- `src-tauri/hermes-crates/hermes-config`
- `src-tauri/hermes-crates/hermes-core`
- `src-tauri/hermes-crates/hermes-tools`
- `src-tauri/hermes-crates/hermes-intelligence`
- `src-tauri/hermes-crates/hermes-environments`
- `src-tauri/hermes-crates/hermes-skills`