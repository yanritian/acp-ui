# Hermes Game Operator 总纲规划

> 日期：2026-07-08  
> 作者：Codex  
> 状态：执行前规划  
> 适用对象：项目负责人、Qwen3.7 Plus、Claude Code、后续执行 Agent  

## 1. 一句话定位

Hermes Game Operator 是一个面向游戏开发的 Agent 操作员控制台：人类操作员负责目标、方向、审批和接管，Agent 负责分析、计划、编码、运行工具、调用 Skill/MCP/Hook，并在 IDE/桌面面板中持续展示进度、记忆、风险和最终走向。

它不是一个简单的“AI 生成游戏代码”页面，而是一套垂直于游戏开发场景的 Agent 执行工作台。

## 2. 为什么先做游戏开发

项目长期目标很大：运营人员、企业人员、传统开发、视频创作、漫画创作都可以成为垂直领域。但第一阶段必须收缩到一个足够具体、足够复杂、足够能验证平台能力的场景。

游戏开发适合作为第一条主线，因为它天然包含：

- 多文件项目结构：场景、脚本、资源、配置、构建产物。
- 多角色协作：策划、程序、美术、测试、构建发布。
- 强工具依赖：Godot、Unity、Ren'Py、终端、日志、运行器、导出器。
- 需要人类介入：玩法方向、风险审批、视觉取舍、调参。
- 容易做演示：改一个角色能力、生成一个资源、运行一次项目，用户能立即感知价值。

先把游戏开发跑通，后面运营、企业、视频、漫画只是更换 Domain Pack，而不是重建平台。

## 3. 产品愿景

最终形态是一个类似 IDE Copilot、Claude Code、任务编排器和运维控制台融合后的 Agent 操作系统。

用户可以在 VSCode、IDEA、桌面端或移动端中：

- 选择一个项目。
- 输入一个目标。
- 看到 Agent 如何理解目标。
- 审查 Agent 的执行计划。
- 实时观察进度、工具调用、文件变更、记忆使用。
- 随时暂停、插话、修改方向、拒绝或批准危险操作。
- 在任务结束后看到总结、diff、验证结果和后续建议。

Agent 不是黑盒聊天机器人，而是可观察、可中断、可接管、可审计的执行者。

## 4. 核心原则

### 4.1 人是操作员，不是旁观者

用户不是等 Agent 自动跑完，而是像操作一个高级自动化开发员：

- 可以随时停止。
- 可以修改当前目标。
- 可以要求 Agent 重写计划。
- 可以审批或拒绝工具调用。
- 可以查看它为什么这么做。

### 4.2 Agent 必须真实执行，不允许模板冒充 AI

历史上项目已经出现过“假 AI 游戏设计/开发”的问题，即通过硬编码模板、`match`、`push_str` 伪装成 AI 功能。后续明确禁止：

- 禁止固定模板冒充模型生成。
- 禁止 token 永远为 0 的假执行结果。
- 禁止 UI 宣称 AI 已完成但后端只是 mock。
- 禁止 demo 页面混入核心导航而不标识实验状态。

所有 AI 能力必须通过真实 Agent、Skill、MCP、Hook 或模型调用完成。

### 4.3 先闭环，再扩张

第一阶段只做 Godot MVP。Unity、Ren'Py、Unreal、视频、漫画、企业办公全部后置。

MVP 的目标不是“功能多”，而是打通一条可信闭环：

选择 Godot 项目 -> 输入需求 -> Agent 分析项目 -> 生成计划 -> 用户可审查 -> Agent 修改文件 -> Hook 验证 -> 用户可暂停/继续/改方向 -> 输出总结。

### 4.4 IDE 插件只是入口，核心能力放在本地服务

VSCode 插件、未来 IDEA 插件、桌面 UI、移动端都不应该各自实现 Agent 逻辑。

推荐架构：

```text
VSCode Plugin
IDEA Plugin
Desktop UI
Mobile Remote
    |
    v
Hermes Game Operator Core
    |
    +-- Agent Runtime
    +-- Skill/MCP/Hook Runtime
    +-- Memory Store
    +-- Project Analyzer
    +-- Progress/Event Stream
    +-- Approval Controller
```

这样未来扩展 IDE 时只需要做客户端，不需要复制核心逻辑。

## 5. 目标用户

### 5.1 第一目标用户：独立游戏开发者

他们常见需求：

- 想快速实现一个玩法原型。
- 不熟悉某个引擎 API。
- 需要读懂旧项目结构。
- 需要调试报错。
- 想让 AI 修改代码但害怕它乱改。
- 希望能看到 AI 到底做了什么。

### 5.2 第二目标用户：小型游戏团队

他们更关注：

- 任务拆解。
- 多角色协作。
- 改动审计。
- 项目记忆。
- 规范化 Hook。
- 团队共享 Skill。

### 5.3 后续用户

游戏主线跑通后，可复制到：

- 运营人员：文案、投放、数据分析、素材生成。
- 企业人员：报表、流程、知识库、自动化办公。
- 传统开发：代码迁移、重构、测试生成、构建修复。
- 视频创作：脚本、分镜、素材、剪辑任务编排。
- 漫画创作：角色设定、分镜、对白、图像生成。

## 6. 产品形态

### 6.1 桌面主控台

当前 `acp-ui` 的 Tauri/Vue 应用适合作为桌面主控台。

职责：

- 项目选择。
- Agent 会话。
- Operator 面板。
- 进度流。
- 记忆管理。
- 设置、MCP、Hook、Skill 管理。
- 高级调试与配置。

### 6.2 VSCode 插件

已有 VSCode 插件雏形，应作为第一 IDE 入口继续扩展。

职责：

- IDE 内侧边栏。
- 当前项目上下文传递。
- 文件 diff 展示。
- 进度树。
- 暂停/继续/停止按钮。
- 一键把选中文件/错误日志发给 Agent。

### 6.3 IDEA 插件

IDEA 插件是后续目标，不要在 MVP 阶段重写完整能力。

第一阶段只设计协议：

- IDEA 插件作为 Operator Core 客户端。
- 通过 WebSocket/本地端口连接核心服务。
- 发送项目路径、当前文件、选区、错误日志。
- 接收任务事件、文件变更、审批请求。

### 6.4 移动端遥控器

移动端不做完整编辑器，而做桌面做不了或不方便做的事情：

- 远程审批。
- 查看任务状态。
- 接收完成/失败通知。
- 碎片时间插话。
- 暂停危险任务。

移动端是遥控器，不是 IDE 镜像。

## 7. 信息架构

第一版核心导航建议收缩为：

```text
Game Operator
  - Project
  - Session
  - Progress
  - Memory
  - Changes
  - Skills
  - Hooks
  - Settings

Experimental
  - Swarm
  - Agent Teams
  - Token Optimizer
  - Demo Dashboards
```

所有 mock/demo 页面必须标记为 Experimental，不能放在主路径中误导用户。

## 8. Operator 面板设计

Operator 面板是这个产品的核心差异化。

### 8.1 顶部任务状态

展示：

- 当前任务名。
- 当前阶段。
- 运行时长。
- 当前 Agent。
- 当前模型。
- 成本/Token 粗略统计。
- 是否等待用户审批。

状态枚举：

```text
Idle
Analyzing
Planning
WaitingForApproval
Executing
Testing
Paused
Blocked
Completed
Failed
Cancelled
```

### 8.2 操作按钮

必须有：

- Pause：暂停当前执行。
- Resume：继续执行。
- Stop：终止任务。
- Redirect：修改方向。
- Approve：批准当前危险操作。
- Reject：拒绝当前危险操作。
- Open Diff：查看改动。
- Summarize：生成当前进展摘要。

### 8.3 进度流

以时间线展示：

```text
09:31 分析 project.godot
09:32 读取 player.gd
09:33 生成执行计划
09:34 等待用户批准写入 player.gd
09:35 写入 player.gd
09:35 运行 godot --check
09:36 修复语法错误
09:37 完成
```

每条进度事件包含：

- eventId
- taskId
- phase
- title
- detail
- toolName
- filePath
- severity
- timestamp
- durationMs

### 8.4 计划面板

展示 Agent 当前计划：

```text
目标：给 Player 加二段跳

步骤：
1. 识别玩家控制脚本
2. 分析当前跳跃逻辑
3. 增加 jump_count / max_jumps
4. 修改落地重置逻辑
5. 运行检查
6. 输出说明
```

用户可以：

- 要求重写计划。
- 删除某一步。
- 插入新步骤。
- 标记“只分析，不写文件”。

### 8.5 记忆面板

分为三类：

```text
Project Memory
  - 项目使用 Godot 4.2
  - Player 脚本在 scripts/player.gd
  - 团队偏好中文注释

Session Memory
  - 本次目标是二段跳
  - 用户要求不要修改输入映射

User Preference
  - 先展示计划再写文件
  - 高危命令必须手机批准
```

用户可以固定、删除、编辑记忆。

### 8.6 最终走向面板

这是“Agent 会把项目带向哪里”的解释层。

展示：

- Agent 对目标的理解。
- 当前执行路线。
- 预计会影响的文件。
- 当前风险。
- 任务完成后的预期状态。
- 是否偏离原始目标。

示例：

```text
目标理解：在不改变输入配置的前提下，为现有 Player 控制器增加二段跳能力。
预计改动：scripts/player.gd
风险：如果当前项目使用自定义状态机，需要避免直接改 velocity 逻辑。
验收：角色在空中可再次跳跃一次，落地后次数重置。
```

## 9. Agent 执行体系

### 9.1 Agent 是执行者

Agent 负责：

- 读项目。
- 分析需求。
- 制定计划。
- 调用工具。
- 修改文件。
- 运行检查。
- 根据错误修复。
- 输出总结。

但 Agent 不拥有最终控制权。危险操作必须由 Operator 批准。

### 9.2 Skill

Skill 是领域能力包。游戏开发第一阶段需要：

```text
godot-analyze
godot-codegen
godot-debug
godot-improve
godot-run-check
```

每个 Skill 至少包含：

- 触发场景。
- 适用引擎版本。
- 输入要求。
- 输出格式。
- 可用工具。
- 禁止事项。
- 验收标准。

### 9.3 MCP

MCP 是工具连接层。第一阶段需要：

- 文件读写。
- 项目搜索。
- 终端命令。
- Git diff。
- Godot CLI 或项目检查命令。
- 可选：资源生成工具。

MCP 工具必须纳入审批规则。

### 9.4 Hook

Hook 是安全与质量控制层。第一阶段至少需要：

- before_write_file：写文件前记录 diff。
- after_write_file：写文件后生成变更摘要。
- before_command：拦截危险命令。
- after_command：收集命令结果。
- before_finish：检查是否有未说明的改动。

后续可以扩展：

- Godot 项目检查。
- GDScript 格式化。
- Git 状态检查。
- 自动生成测试建议。
- 资源体积检查。

### 9.5 Memory

记忆不是聊天记录垃圾桶，只保存未来仍有价值的信息：

- 项目结构。
- 引擎版本。
- 代码风格。
- 用户偏好。
- 常见错误。
- 已验证方案。

不保存：

- 临时日志。
- 一次性任务过程。
- 大段工具输出。
- 过期 TODO。

## 10. Godot MVP 范围

### 10.1 必须支持

- 选择 Godot 项目目录。
- 检测 `project.godot`。
- 展示项目基本信息。
- 读取脚本目录。
- 根据需求生成计划。
- 写入一个或多个 `.gd` 文件。
- 显示 diff。
- 暂停/继续/停止。
- 拦截写文件和命令执行。
- 输出任务总结。

### 10.2 暂不支持

- Unity。
- Ren'Py。
- Unreal。
- 多 Agent 自动协作。
- 资产市场。
- 完整代码库语义索引。
- 自动发布。
- 云端团队协作。

### 10.3 MVP 演示任务

推荐使用一个小型 Godot 项目，跑通以下任务：

```text
给 Player 增加二段跳能力，不要修改输入映射。先展示计划，等我批准后再改文件。
```

验收结果：

- Agent 找到玩家脚本。
- 生成合理计划。
- 等待批准。
- 修改正确文件。
- 显示 diff。
- 运行检查或给出手动验证步骤。
- 输出总结。

## 11. 技术架构建议

### 11.1 当前项目中的主线资产

当前仓库已有可复用资产：

- Tauri/Vue 桌面主界面。
- GameManager 项目检测入口。
- ExecutiveSession 执行会话入口。
- Hermes crates。
- VSCode 插件产物。
- game_detector/game_launcher/game_export 等工具模块。
- Agent progress 相关前端组件。

不要从零重写，优先收束已有资产。

### 11.2 推荐分层

```text
UI Layer
  - GameManager
  - OperatorPanel
  - ProgressTimeline
  - MemoryPanel
  - ChangePanel

Command Layer
  - hermes_game_start_task
  - hermes_game_pause_task
  - hermes_game_resume_task
  - hermes_game_stop_task
  - hermes_game_approve_action
  - hermes_game_reject_action

Runtime Layer
  - AgentLoop
  - ToolRegistry
  - SkillRuntime
  - HookRuntime
  - ApprovalController

Domain Layer
  - GodotProjectAnalyzer
  - GodotSkillPack
  - GodotRunCheck

Persistence Layer
  - task_events
  - task_memory
  - approvals
  - file_changes
```

### 11.3 事件协议

UI 不应该直接猜 Agent 状态，后端需要持续发事件。

建议事件类型：

```text
task.started
task.phase_changed
task.plan_created
task.memory_used
tool.started
tool.completed
file.read
file.write_requested
file.write_completed
approval.requested
approval.resolved
task.paused
task.resumed
task.cancelled
task.completed
task.failed
```

事件字段：

```json
{
  "id": "evt_001",
  "taskId": "task_001",
  "type": "task.phase_changed",
  "title": "进入编码阶段",
  "detail": "准备修改 scripts/player.gd",
  "timestamp": "2026-07-08T09:30:00+08:00",
  "payload": {}
}
```

## 12. 当前仓库先做的 P0 整理

在开发 Hermes Game Operator 前，必须先让项目回到可验证状态。

### 12.1 构建基线

必须完成：

- 修复前端 `npm run build`。
- Rust 工具链可用后执行 `cargo check`。
- 明确 JS 包管理器，只保留一个 lock。
- `Cargo.lock` 纳入版本控制。

### 12.2 删除旧假 AI 残留

必须完成：

- 旧 GameDesigner/GameDeveloper 代码删除。
- 旧路由删除。
- 旧 Tauri 命令删除。
- 旧测试删除或迁移。
- 旧报告归档。

### 12.3 隐藏实验页面

所有 mock/demo 页面进入实验区，不再干扰主线。

### 12.4 收紧安全边界

当前桌面 capability 允许全路径读写。短期可以保留用于本地 ACP，但远程控制/手机审批上线前必须：

- 明确 workspace 边界。
- 写文件默认仅限项目目录。
- 高危命令必须审批。
- 删除、移动、覆盖文件必须展示 diff 或影响范围。

## 13. 阶段路线图

### Phase 0：基线修复

目标：项目可构建、可测试、可继续开发。

交付：

- 前端 build 通过。
- 假 AI 清理提交。
- 旧测试清理。
- 文档归档。
- lock 文件策略固定。

### Phase 1：Godot Operator 单任务闭环

目标：跑通一个真实 Godot 任务。

交付：

- GameManager -> ExecutiveSession 带项目路径和 `mode=godot`。
- OperatorPanel 初版。
- 任务阶段状态。
- 暂停/继续/停止。
- Godot 项目分析。
- 文件 diff 展示。
- 总结输出。

### Phase 2：Skill/MCP/Hook 正式化

目标：从“能跑”变成“可扩展”。

交付：

- Godot Skill Pack。
- ToolRegistry 注册真实工具。
- Hook Runtime 初版。
- ApprovalController 初版。
- MemoryPanel 初版。

### Phase 3：VSCode 插件集成

目标：让开发者在 IDE 内使用。

交付：

- VSCode Operator Sidebar。
- 连接本地 Operator Core。
- 显示进度事件。
- 显示 diff。
- 暂停/继续/停止。
- 发送当前文件/选区/错误日志。

### Phase 4：Unity/Ren'Py 扩展

目标：验证 Domain Pack 抽象。

交付：

- Unity Skill Pack。
- Ren'Py Skill Pack。
- 引擎选择器。
- 引擎专属项目分析。

### Phase 5：IDEA 插件原型

目标：证明多 IDE 客户端架构成立。

交付：

- IDEA 插件连接本地 Operator Core。
- 项目信息上报。
- 进度面板。
- 基础控制按钮。

## 14. 验收标准

### 14.1 MVP 功能验收

- 用户能选择 Godot 项目。
- 系统能识别项目。
- 用户能输入任务。
- Agent 能生成计划。
- 用户能批准后执行。
- Agent 能修改文件。
- UI 能展示进度。
- 用户能暂停、继续、停止。
- UI 能展示 diff。
- 任务结束有总结。

### 14.2 工程验收

- `npm run build` 通过。
- 单元测试分层清晰。
- Tauri/Rust 能在本机 `cargo check`。
- 没有旧假 AI 路由和命令残留。
- mock/demo 页面不混入核心导航。
- 新增功能有最小测试。

### 14.3 产品验收

找 3-5 个真实游戏开发者试用，观察：

- 是否理解 Operator 面板。
- 是否愿意让 Agent 改文件。
- 是否需要暂停/接管。
- 进度和记忆是否有帮助。
- Godot MVP 是否比普通聊天式 AI 更可信。

## 15. 未来多领域扩展模型

游戏开发做成后，平台抽象为：

```text
Operator Core
  + Domain Pack: Game
  + Domain Pack: Enterprise
  + Domain Pack: Marketing
  + Domain Pack: Video
  + Domain Pack: Comic
```

每个 Domain Pack 包含：

- Skills。
- MCP 工具配置。
- Hooks。
- Memory schema。
- UI 面板扩展。
- 示例任务。
- 验收测试。

这样未来不是重写产品，而是新增领域包。

## 16. 给执行 Agent 的硬性约束

后续 Qwen3.7 Plus 和 Claude Code 执行时必须遵守：

1. 不允许新增假 AI。
2. 不允许没有构建基线就继续扩功能。
3. 不允许一次性做多个垂直领域。
4. 不允许把 IDE 插件写成独立逻辑孤岛。
5. 不允许跳过审批和安全边界。
6. 不允许把 mock 页面伪装成真实功能。
7. 所有执行必须留下可验证产物。

## 17. 推荐下一步

马上执行：

1. 修复当前构建和测试基线。
2. 提交“删除旧假 AI 游戏功能”的清理 commit。
3. 新建 `Hermes Game Operator` 开发分支。
4. 先做 Godot 单任务闭环。
5. 再接 Skill/MCP/Hook。
6. 最后扩 VSCode 插件面板。

最重要的一句话：

先做出一个可信的 Godot Agent 操作员闭环，再谈多引擎、多 IDE、多行业。

