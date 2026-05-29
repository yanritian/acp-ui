# 项目任务分解规划：Skill 元工具架构

## 已明确的决策

- **架构模式**：采用 `invoke_skill` 元工具作为能力委托入口，所有 Skill 通过统一的元工具接口调用
- **Skill 数量**：12-16 个核心工具（永不增加），通过 Skill 扩展能力而非增加工具数量
- **Skill 格式**：基于 SKILL.md 文件（YAML frontmatter + Markdown body），与现有 `skill_orchestrator.rs` 兼容
- **项目技术栈**：Tauri 2.x + Vue 3 + Rust（hermes-crates 系列）
- **现有基础**：已有 `hermes-tools/skills.rs`（SkillsListHandler, SkillViewHandler, SkillManageHandler）、`skill_orchestrator.rs`、`skills-loader.ts`

## 整体规划概述

### 项目目标

将 OpenClacky 的 Skill 元工具模式集成到 ACP-UI，实现：
1. 收敛工具数量到 12-16 个核心工具，通过 `invoke_skill` 元工具统一委托
2. Skill 支持自然语言创建（通过 LLM 自动生成 SKILL.md）
3. Skill 自进化机制（执行反馈 → 自动改进 SKILL.md 内容）
4. 与现有 hermes-skills crate 和 skills-loader.ts 无缝集成

### 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x |
| 前端 | Vue 3 + TypeScript + Vite |
| 后端 | Rust（hermes-crates：hermes-core, hermes-tools, hermes-agent） |
| Skill 存储 | 文件系统（~/.hermes/skills/）+ 可选 SQLite |
| LLM 交互 | OpenAI-compatible API |
| 状态管理 | Vue Composition API（现有 skills-loader.ts） |

### 主要阶段

1. **Phase 1：核心基础设施** — 定义 `invoke_skill` 元工具 + Skill 执行引擎（Rust 层）
2. **Phase 2：Skill 管理与自进化** — 增强 Skill CRUD + 自进化反馈循环
3. **Phase 3：前端集成** — Vue UI 组件 + skills-loader.ts 重构
4. **Phase 4：12-16 个核心 Skill 定义** — 设计核心 Skill 清单 + SKILL.md 文件
5. **Phase 5：测试与优化** — 集成测试 + 性能优化 + 文档

### 详细任务分解

#### 阶段 1：核心基础设施（Rust 层）

- **任务 1.1**：设计并实现 `invoke_skill` 元工具 Handler
  - 目标：创建统一的 Skill 调用入口，替代分散的 Skill 调用方式
  - 输入：现有 `SkillProvider` trait、`SkillOrchestrator`、`ToolHandler` trait
  - 输出：`InvokeSkillHandler` 结构体，实现 `ToolHandler` trait
  - 涉及文件：
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/tools/invoke_skill.rs`
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/tools/mod.rs`（导出新模块）
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/lib.rs`（重新导出）
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/register_builtins.rs`（注册 invoke_skill 工具）
  - 预估工作量：4-6 小时

- **任务 1.2**：扩展 `SkillProvider` trait 增加执行能力
  - 目标：让 SkillProvider 不仅能 CRUD，还能执行 Skill 定义的操作
  - 输入：现有 `SkillProvider` trait（`hermes-core/src/traits.rs`）
  - 输出：新增 `execute_skill` 方法到 trait
  - 涉及文件：
    - **修改**：`src-tauri/hermes-crates/hermes-core/src/traits.rs`（添加 execute_skill 方法）
    - **修改**：所有实现 `SkillProvider` 的地方（含 mock、测试）
  - 预估工作量：2-3 小时

- **任务 1.3**：实现 Skill 参数解析与验证
  - 目标：解析 Skill 的 frontmatter 中定义的参数 schema，验证传入参数
  - 输入：SKILL.md frontmatter 格式、参数验证规则
  - 输出：参数解析器 + 验证器
  - 涉及文件：
    - **修改**：`src-tauri/hermes-crates/hermes-agent/src/skill_orchestrator.rs`（增强 frontmatter 解析，支持 parameters 字段）
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/tools/skill_params.rs`（参数解析与验证模块）
  - 预估工作量：3-4 小时

- **任务 1.4**：实现 Skill 执行上下文与结果封装
  - 目标：为 Skill 执行提供上下文（session_id、agent_id、history 等）和标准化结果
  - 输入：现有 `AgentResult`、`ToolResult` 类型
  - 输出：`SkillExecutionContext` + `SkillExecutionResult` 类型
  - 涉及文件：
    - **修改**：`src-tauri/hermes-crates/hermes-core/src/types.rs`（新增 SkillExecutionContext 和 SkillExecutionResult）
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/tools/invoke_skill.rs`（使用上下文）
  - 预估工作量：2-3 小时

#### 阶段 2：Skill 管理与自进化

- **任务 2.1**：增强 `SkillManageHandler` 支持自然语言创建
  - 目标：通过 LLM 将自然语言描述转换为 SKILL.md 格式
  - 输入：现有 `SkillManageHandler`（`skills.rs`）、LLM 调用接口
  - 输出：新增 `nl_create` action，接收自然语言描述，调用 LLM 生成 SKILL.md
  - 涉及文件：
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/tools/skills.rs`（新增 nl_create action）
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/register_builtins.rs`（如需要）
  - 预估工作量：3-4 小时

- **任务 2.2**：实现 Skill 自进化机制
  - 目标：Skill 执行后收集反馈，自动改进 SKILL.md 内容
  - 输入：执行结果、用户反馈、现有 `self_improve` action
  - 输出：增强的自进化逻辑（结构化反馈 + LLM 优化建议 + 版本历史）
  - 涉及文件：
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/tools/skills.rs`（增强 self_improve action）
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/tools/skill_evolution.rs`（自进化逻辑模块）
    - **修改**：`src-tauri/hermes-crates/hermes-core/src/types.rs`（新增 SkillVersion、SkillFeedback 类型）
  - 预估工作量：4-5 小时

- **任务 2.3**：实现 Skill 版本历史与回滚
  - 目标：记录 Skill 每次改进的版本，支持回滚到历史版本
  - 输入：Skill 文件、版本存储策略
  - 输出：版本历史管理 + 回滚功能
  - 涉及文件：
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/tools/skill_versioning.rs`
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/tools/skills.rs`（新增 version_list、version_rollback actions）
  - 预估工作量：3-4 小时

- **任务 2.4**：扩展 SkillProvider 实现（文件系统增强）
  - 目标：增强文件系统 SkillProvider 支持版本历史和进化反馈存储
  - 输入：现有文件系统 SkillProvider 实现
  - 输出：带版本管理的文件系统 SkillProvider
  - 涉及文件：
    - **搜索定位**：需要找到 SkillProvider 的文件系统实现（可能在 hermes-agent 或独立的 provider crate 中）
    - **修改**：对应的文件系统实现文件
  - 预估工作量：3-4 小时

#### 阶段 3：前端集成

- **任务 3.1**：重构 `skills-loader.ts`
  - 目标：与新的 Rust `invoke_skill` 工具对接，统一前端 Skill 管理
  - 输入：现有 `src/lib/skills-loader.ts`、Tauri invoke API
  - 输出：重构后的 SkillsLoader，通过 Tauri 命令调用 Rust 层
  - 涉及文件：
    - **修改**：`src/lib/skills-loader.ts`（重构为通过 Tauri invoke 调用 Rust 层）
    - **新建**：`src/lib/skill-types.ts`（TypeScript 类型定义，与 Rust 类型对应）
  - 预估工作量：3-4 小时

- **任务 3.2**：创建 Skill 管理 UI 组件
  - 目标：提供 Skill 浏览、创建、编辑、执行的 UI 界面
  - 输入：现有 Vue 组件结构、UI 设计
  - 输出：SkillManager、SkillEditor、SkillExecutor 组件
  - 涉及文件：
    - **新建**：`src/components/skills/SkillManager.vue`
    - **新建**：`src/components/skills/SkillEditor.vue`
    - **新建**：`src/components/skills/SkillExecutor.vue`
    - **新建**：`src/components/skills/SkillVersionHistory.vue`
    - **修改**：`src/router/index.ts` 或对应路由文件（添加路由）
  - 预估工作量：6-8 小时（如果使用现有 UI 组件库可缩短）

- **任务 3.3**：实现 Tauri Commands 桥接
  - 目标：在 Tauri Rust 层暴露 Skill 相关命令供前端调用
  - 输入：现有 Tauri commands 结构
  - 输出：skill_invoke、skill_manage 等 Tauri commands
  - 涉及文件：
    - **搜索定位**：需要找到现有 Tauri commands 文件（可能在 src-tauri/src/ 下）
    - **新建/修改**：对应的 commands 文件
  - 预估工作量：3-4 小时

#### 阶段 4：12-16 个核心 Skill 定义

- **任务 4.1**：设计核心 Skill 清单
  - 目标：确定 12-16 个核心 Skill 及其职责边界
  - 输入：现有工具列表（register_builtins.rs 中注册的 50+ 工具）
  - 输出：核心 Skill 清单文档 + 收敛方案
  - 涉及文件：
    - **新建**：`docs/core-skills-list.md`（核心 Skill 清单）
  - 预估工作量：2-3 小时（设计决策）

- **任务 4.2**：创建核心 Skill 的 SKILL.md 文件
  - 目标：为每个核心 Skill 创建完整的 SKILL.md 定义
  - 输入：核心 Skill 清单、现有工具实现
  - 输出：12-16 个 SKILL.md 文件
  - 涉及文件：
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/skills/` 目录下 12-16 个 SKILL.md 文件
    - 或 **新建**：`~/.hermes/skills/` 目录下（根据存储策略决定）
  - 预估工作量：6-10 小时（每个 Skill 约 30-45 分钟）

- **任务 4.3**：收敛现有工具到核心 Skill
  - 目标：将现有的 50+ 工具分组到 12-16 个 Skill 下
  - 输入：现有工具注册清单、核心 Skill 清单
  - 输出：工具分组映射 + 兼容性层
  - 涉及文件：
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/register_builtins.rs`（工具分组调整）
    - **修改**：`src-tauri/hermes-crates/hermes-tools/src/tools/skill_commands.rs`（如有）
  - 预估工作量：4-6 小时

#### 阶段 5：测试与优化

- **任务 5.1**：Rust 层单元测试
  - 目标：为 invoke_skill、skill_evolution、skill_versioning 编写单元测试
  - 输入：各模块实现
  - 输出：覆盖率 80%+ 的单元测试
  - 涉及文件：
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/tools/invoke_skill.rs` 中的 tests 模块
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/tools/skill_evolution.rs` 中的 tests 模块
    - **新建**：`src-tauri/hermes-crates/hermes-tools/src/tools/skill_versioning.rs` 中的 tests 模块
  - 预估工作量：4-6 小时

- **任务 5.2**：集成测试
  - 目标：端到端测试 Skill 创建 → 执行 → 反馈 → 进化完整流程
  - 输入：完整实现
  - 输出：集成测试用例
  - 涉及文件：
    - **新建**：`src-tauri/hermes-crates/hermes-tools/tests/skill_integration.rs`
    - **新建**：`src-tauri/hermes-crates/hermes-agent/tests/skill_e2e.rs`
  - 预估工作量：4-6 小时

- **任务 5.3**：前端 E2E 测试
  - 目标：测试 Skill 管理 UI 的完整用户流程
  - 输入：前端实现、Playwright（项目已有）
  - 输出：Playwright E2E 测试用例
  - 涉及文件：
    - **新建**：`tests/functional/skill-management.spec.ts`
  - 预估工作量：3-4 小时

- **任务 5.4**：性能优化与文档
  - 目标：优化 Skill 加载性能 + 编写完整文档
  - 输入：完整实现、测试结果
  - 输出：性能优化补丁 + 技术文档
  - 涉及文件：
    - **新建**：`docs/skill-meta-tool-architecture.md`（完整架构文档）
    - **新建**：`docs/skill-creation-guide.md`（Skill 创建指南）
    - **新建**：`docs/skill-evolution-guide.md`（自进化机制说明）
  - 预估工作量：3-4 小时

### 依赖关系与关键路径

```
Phase 1（核心基础设施）
  ├── 1.1 invoke_skill Handler
  ├── 1.2 SkillProvider 扩展
  ├── 1.3 参数解析验证
  └── 1.4 执行上下文
        ↓
Phase 2（Skill 管理与自进化）
  ├── 2.1 自然语言创建
  ├── 2.2 自进化机制
  ├── 2.3 版本历史
  └── 2.4 文件系统增强
        ↓
Phase 3（前端集成）── 与 Phase 4 并行
  ├── 3.1 skills-loader 重构
  ├── 3.2 UI 组件
  └── 3.3 Tauri Commands

Phase 4（核心 Skill 定义）── 与 Phase 3 并行
  ├── 4.1 核心 Skill 清单
  ├── 4.2 SKILL.md 文件
  └── 4.3 工具收敛
        ↓
Phase 5（测试与优化）
  ├── 5.1 Rust 单元测试
  ├── 5.2 集成测试
  ├── 5.3 前端 E2E
  └── 5.4 性能优化与文档
```

**关键路径**：Phase 1 → Phase 2 → Phase 4.1 → Phase 4.2 → Phase 5

## 需要进一步明确的问题

### 问题 1：Skill 存储位置与格式

**推荐方案**：

- 方案 A：**文件系统存储（~/.hermes/skills/）**
  - 优点：与现有 `skill_orchestrator.rs` 完全兼容、简单直观、易于版本控制、人类可读
  - 缺点：大规模 Skill 时性能可能下降、无结构化查询能力
  
- 方案 B：**SQLite 存储 + 文件系统缓存**
  - 优点：支持结构化查询、版本历史管理更高效、支持元数据过滤
  - 缺点：需要额外的数据库层、与现有文件系统 SkillProvider 不兼容、增加复杂度

- 方案 C：**混合模式（文件系统为主 + SQLite 索引）**
  - 优点：兼顾可读性和查询效率、向后兼容
  - 缺点：需要维护双写一致性、实现复杂度最高

**等待用户选择**：

```
请选择您偏好的方案，或提供其他建议：
[ ] 方案 A：纯文件系统存储（推荐，与现有架构最兼容）
[ ] 方案 B：SQLite 存储
[ ] 方案 C：混合模式
[ ] 其他方案：_______
```

### 问题 2：invoke_skill 的调用方式

**推荐方案**：

- 方案 A：**LLM 驱动调用**
  - LLM 决定调用哪个 Skill，通过 `invoke_skill` 工具传入 skill_name 和 params
  - Skill 内容（SKILL.md body）作为 system prompt 注入给 LLM
  - 优点：灵活、自然语言交互、与 OpenClacky 模式一致
  - 缺点：每次调用需要 LLM 交互、延迟较高

- 方案 B：**直接执行调用**
  - `invoke_skill` 直接解析 Skill 定义并执行（类似函数调用）
  - 适用于有明确执行逻辑的 Skill（如代码生成、文件操作）
  - 优点：快速、可预测、无需 LLM 开销
  - 缺点：需要 Skill 定义中包含可执行逻辑（代码或指令）

- 方案 C：**混合模式（LLM 路由 + 直接执行）**
  - `invoke_skill` 根据 Skill 类型决定路由：
    - `type: llm-guided` → LLM 驱动
    - `type: direct` → 直接执行
  - 优点：兼顾灵活性和性能
  - 缺点：需要定义 Skill 类型系统

**等待用户选择**：

```
请选择您偏好的方案，或提供其他建议：
[ ] 方案 A：LLM 驱动调用（推荐，与 OpenClacky 模式一致）
[ ] 方案 B：直接执行调用
[ ] 方案 C：混合模式
[ ] 其他方案：_______
```

### 问题 3：12-16 个核心 Skill 的具体定义

以下是初步建议的核心 Skill 清单，需要您确认或调整：

| # | Skill 名称 | 职责 | 收敛的现有工具 |
|---|-----------|------|---------------|
| 1 | `web-research` | 网页搜索与信息提取 | web_search, web_extract |
| 2 | `code-execution` | 代码执行与调试 | execute_code, terminal |
| 3 | `file-operations` | 文件读写、搜索、补丁 | read_file, write_file, patch, search_files |
| 4 | `browser-automation` | 浏览器控制与自动化 | 10 个 browser_* 工具 |
| 5 | `vision-analysis` | 图像理解与分析 | vision_analyze |
| 6 | `media-generation` | 图像/视频/音频生成 | image_gen, gpt_image, video_gen, audio_gen |
| 7 | `skill-management` | Skill CRUD 与版本管理 | skills_list, skill_view, skill_manage |
| 8 | `memory-ops` | 记忆存储与检索 | memory, session_search |
| 9 | `task-management` | 任务与计划管理 | todo, cronjob |
| 10 | `communication` | 消息发送与澄清 | send_message, clarify |
| 11 | `delegation` | 任务委派与子 Agent | delegate_task |
| 12 | `security-audit` | 安全扫描与 URL 检查 | osv_check, url_safety |
| 13 | `code-review` | 代码审查与质量检查 | （新增） |
| 14 | `testing` | 测试生成与执行 | （新增） |
| 15 | `planning` | 项目规划与任务分解 | （新增） |
| 16 | `documentation` | 文档生成与维护 | （新增） |

**等待用户选择**：

```
请确认或调整上述核心 Skill 清单：
[ ] 确认上述清单
[ ] 需要调整（请说明）：_______
[ ] 建议增减 Skill 数量
```

### 问题 4：自进化机制的反馈来源

**推荐方案**：

- 方案 A：**用户显式反馈**
  - 用户手动提供反馈（评分、文字评价）
  - 优点：反馈质量高、意图明确
  - 缺点：依赖用户主动性、反馈频率低

- 方案 B：**隐式反馈（执行结果分析）**
  - 根据 Skill 执行结果（成功率、错误类型、执行时间）自动推断
  - 优点：自动收集、反馈频率高
  - 缺点：推断可能不准确

- 方案 C：**混合模式**
  - 结合用户显式反馈和隐式执行数据
  - 优点：全面、准确
  - 缺点：实现复杂度最高

**等待用户选择**：

```
请选择您偏好的方案，或提供其他建议：
[ ] 方案 A：用户显式反馈
[ ] 方案 B：隐式反馈
[ ] 方案 C：混合模式（推荐）
[ ] 其他方案：_______
```

## 用户反馈区域

请在此区域补充您对整体规划的意见和建议：

```
用户补充内容：

---

---

---

```
