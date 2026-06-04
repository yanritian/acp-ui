# ACP-UI 项目重构报告

> 分支: `refactor/project-cleanup`
> 基于: `my-agent-teams-platform` (commit 3a848f4)
> 日期: 2026-06-04

## 概述

本次重构针对 ACP-UI 项目的三大核心问题进行修复：项目根目录混乱、TypeScript 类型安全缺失、以及两个巨型文件（`lib.rs` 2157 行 / `App.vue` 1724 行）的拆分。所有修改已通过 TypeScript strict mode 编译和 Rust cargo check 验证。

---

## 一、项目根目录清理

### 问题
项目根目录存在 84 张截图（.png）、3 个 Python 脚本、4 个 bat 文件、1 个 cloudflared.exe，以及 10+ 个临时测试目录（test_demo, test_erp 等）。这些文件大部分是开发调试过程中的残留物。

### 修复措施

| 操作 | 详情 |
|------|------|
| 已追踪的截图 | 通过 `git mv` 移至 `docs/screenshots/`（5 个文件） |
| 未追踪的截图 | 移至 `docs/screenshots/`（约 80 个 .png 文件） |
| Python/Bat 脚本 | 移至 `docs/` 目录 |
| cloudflared.exe | 移至 `docs/` 目录 |
| 临时测试目录 | 移至 `_cleanup/` 目录（test_demo, test_erp, test_final 等 7 个） |
| 其余锁定目录 | test-workspace, flutter, hermes 因进程占用未能移动，已被 .gitignore 排除 |

### .gitignore 更新

新增或修改的排除规则：

```gitignore
# 截图：仅保留 assets/ 和 docs/screenshots/
!docs/screenshots/*.png
!assets/

# 根目录级脚本
/*.py
/*.bat
/*.cmd

# 临时目录
_cleanup/
workspace/
flutter/
hermes/
screen_base64.txt

# Flutter 项目内杂项
acp_ui_flutter/windows_bak/
acp_ui_flutter/*.py
acp_ui_flutter/*.txt
acp_ui_flutter/*.json
acp_ui_flutter/*.png
```

### 验证方式
- 根目录已无 .png / .py / .bat / .exe 文件
- `git ls-files` 确认无杂项文件被追踪

---

## 二、TypeScript Strict Mode 启用

### 问题
`tsconfig.json` 中 `strict: false`、`noImplicitAny: false`，55000+ 行 TypeScript 代码几乎没有类型安全保护。

### 修复措施

**tsconfig.json 变更：**
```json
// 修改前
"strict": false,
"noImplicitAny": false

// 修改后
"strict": true,
"noImplicitAny": true
```

**修复的编译错误（共 10 个）：**

| 文件 | 错误类型 | 修复方式 |
|------|---------|---------|
| `src/components/ExecutiveSessionView.vue:352` | strictNullChecks | 添加 `if (!invoke) return;` null 守卫 |
| `src/lib/collaboration/mock-data-generator.ts:454` | 空数组推断 | `const conditions = []` → `const conditions: any[] = []` |
| `src/lib/collaboration/mock-data-generator.ts:474` | 空数组推断 | `const constraints = []` → `const constraints: any[] = []` |
| `src/lib/collaboration/mock-data-generator.ts:498` | 空数组推断 | `const examples = []` → `const examples: any[] = []` |
| `src/components/agent-pet/GrowthSystem.vue:195` | 索引签名 | 添加 `as Record<string, any>` 类型断言 |
| `src/components/agent-pet/GrowthSystem.vue:199` | 索引签名 | 同上 |
| `src/lib/sync/websocket-sync-service.ts:310` | 隐式 any | 添加 `(agent: any)` 参数类型 |
| `src/lib/sync/websocket-sync-service.ts:318` | 隐式 any | 添加 `(pet: any)` 参数类型 |
| `src/lib/sync/websocket-sync-service.ts:325` | 隐式 any | 添加 `(event: any)` 参数类型 |
| `src/components/AppSidebar.vue:167-169` | undefined 不匹配 | 添加默认值 `\|\| ''`, `\|\| []`, `\|\| 0` |

### 验证方式
```bash
npx vue-tsc --noEmit  # 退出码 0，零错误
```

### 后续建议
- `noUnusedLocals` 和 `noUnusedParameters` 当前仍为 false，开启后预计有约 65 个未使用导入/变量警告，建议逐步清理
- 项目中有约 60 处 `any` 使用，可逐步替换为精确类型

---

## 三、Rust lib.rs 拆分

### 问题
`src-tauri/src/lib.rs` 单文件 2157 行，包含 80+ 个 Tauri command 函数、6 个数据结构体、应用入口函数，所有业务逻辑堆叠在一起。

### 重构方案

创建 `src-tauri/src/commands/` 模块目录，按业务领域拆分为 13 个子模块：

```
src-tauri/src/commands/
  mod.rs                  # 模块声明 + 重导出
  config.rs               # 配置管理命令 (4 个)
  agent_lifecycle.rs      # Agent 生命周期命令 (13 个)
  task_history.rs         # 任务历史命令 (7 个)
  memory.rs               # MemoryRecord + 记忆命令 (5 个)
  self_healing.rs         # ErrorRecord/SolutionRecord + 自愈命令 (5 个)
  self_evolution.rs       # EvolutionRecord/PatternRecord + 进化命令 (5 个)
  gateway.rs              # 网关配置结构体 + 网关命令 (6 个)
  websocket_cmds.rs       # WebSocket/QR 码命令 (6 个)
  log_stream_cmds.rs      # 日志流命令 (8 个)
  permission.rs           # 权限检查命令 (4 个)
  agent_config.rs         # Agent 配置解析命令 (6 个)
  executive.rs            # Executive Agent + Session 命令 (11 个)
  teams.rs                # Agent Teams 平台命令 (9 个)
```

### 重构效果

| 指标 | 修改前 | 修改后 |
|------|--------|--------|
| lib.rs 行数 | 2157 | 291 |
| 代码减少 | - | 86.5% |
| commands/ 总行数 | - | 1937 |
| 单文件最大行数 | 2157 (lib.rs) | ~350 (self_healing.rs / self_evolution.rs) |

### 额外修改

由于 `GatewayConfig`、`FeishuConfig`、`TelegramConfig` 等结构体从 crate 根迁移到了 `commands::gateway`，更新了以下文件的导入路径：

- `src-tauri/src/gateway_config.rs` — 导入路径改为 `crate::commands::gateway::*`
- `src-tauri/src/bot_adapters/feishu.rs` — 导入路径改为 `crate::commands::gateway::FeishuConfig`
- `src-tauri/src/bot_adapters/telegram.rs` — 导入路径改为 `crate::commands::gateway::TelegramConfig`

### 验证方式
```bash
cd src-tauri && cargo check  # 0 错误，30 个既有 dead code 警告（finance 模块）
```

---

## 四、App.vue 重构 + Vue Router 引入

### 问题
`App.vue` 单文件 1724 行，21 个视图状态通过 `currentView` ref + `v-if/v-else-if` 链条切换，无路由支持，无代码分割。

### 重构方案

#### 4.1 引入 Vue Router

新增 `src/router.ts`，配置 22 个路由，全部使用懒加载 `() => import(...)` 实现按需代码分割：

```ts
// 路由示例
{ path: '/chat', component: () => import('./components/ChatView.vue') },
{ path: '/multi-agent', component: () => import('./components/MultiAgentChat.vue') },
// ... 共 22 个路由
```

使用 `createWebHashHistory()` 模式，兼容 Tauri WebView。

`main.ts` 添加 `app.use(router)` 注册路由插件。

`package.json` 新增依赖 `vue-router: ^4.5.1`。

#### 4.2 提取 Composables

创建 `src/composables/` 目录，提取 4 个组合式函数：

| Composable | 文件 | 职责 |
|-----------|------|------|
| `useResponsiveLayout` | `useResponsiveLayout.ts` | 窄屏检测、侧边栏切换、媒体查询监听 |
| `useReconnect` | `useReconnect.ts` | 前台重连、页面可见性变化、手动重连 |
| `usePreferences` | `usePreferences.ts` | KVStore 偏好持久化、selectedCwd、文件夹选择器 |
| `useBotCommand` | `useBotCommand.ts` | Tauri bot-command 事件监听与处理 |

#### 4.3 提取子组件

| 组件 | 文件 | 来源 |
|------|------|------|
| `AppSidebar` | `components/AppSidebar.vue` | App.vue 侧边栏模板 (~155 行) |
| `ConnectionBanner` | `components/ConnectionBanner.vue` | 重连/错误 banner (~18 行) |
| `WelcomeScreen` | `components/WelcomeScreen.vue` | 欢迎屏幕 (~70 行) |
| `StatusView` | `components/StatusView.vue` | Agent 状态面板（原内联视图） |
| `MonitorView` | `components/MonitorView.vue` | 实时监控面板（原内联视图） |

#### 4.4 辅助模块

提取 `src/lib/mock-task-dag.ts`，将 App.vue 中硬编码的演示 DAG 数据移至独立模块。

### 重构效果

| 指标 | 修改前 | 修改后 |
|------|--------|--------|
| App.vue 行数 | 1724 | 455 |
| 代码减少 | - | 73.6% |
| 视图切换方式 | 21 个 v-else-if | `<router-view />` |
| 代码分割 | 无 | 22 个懒加载路由 |
| URL 导航 | 不支持 | Hash 模式路由 |
| 浏览器前进/后退 | 不支持 | 开箱即用 |

### 验证方式
```bash
npx vue-tsc --noEmit  # 退出码 0，零错误
npm install           # vue-router 依赖安装成功
```

---

## 五、变更文件汇总

### 新增文件 (24 个)

| 路径 | 说明 |
|------|------|
| `src/router.ts` | Vue Router 配置 |
| `src/composables/useResponsiveLayout.ts` | 响应式布局 composable |
| `src/composables/useReconnect.ts` | 重连逻辑 composable |
| `src/composables/usePreferences.ts` | 偏好设置 composable |
| `src/composables/useBotCommand.ts` | Bot 命令 composable |
| `src/components/AppSidebar.vue` | 侧边栏组件 |
| `src/components/ConnectionBanner.vue` | 连接状态横幅 |
| `src/components/WelcomeScreen.vue` | 欢迎屏幕 |
| `src/components/StatusView.vue` | Agent 状态面板 |
| `src/components/MonitorView.vue` | 实时监控面板 |
| `src/lib/mock-task-dag.ts` | 模拟 Task DAG 数据 |
| `src-tauri/src/commands/mod.rs` | 命令模块声明 |
| `src-tauri/src/commands/config.rs` | 配置命令 |
| `src-tauri/src/commands/agent_lifecycle.rs` | Agent 生命周期命令 |
| `src-tauri/src/commands/task_history.rs` | 任务历史命令 |
| `src-tauri/src/commands/memory.rs` | 记忆系统命令 |
| `src-tauri/src/commands/self_healing.rs` | 自愈系统命令 |
| `src-tauri/src/commands/self_evolution.rs` | 自进化系统命令 |
| `src-tauri/src/commands/gateway.rs` | 网关命令 |
| `src-tauri/src/commands/websocket_cmds.rs` | WebSocket 命令 |
| `src-tauri/src/commands/log_stream_cmds.rs` | 日志流命令 |
| `src-tauri/src/commands/permission.rs` | 权限命令 |
| `src-tauri/src/commands/agent_config.rs` | Agent 配置命令 |
| `src-tauri/src/commands/executive.rs` | Executive Agent 命令 |
| `src-tauri/src/commands/teams.rs` | Agent Teams 命令 |

### 修改文件 (9 个)

| 路径 | 修改说明 |
|------|---------|
| `.gitignore` | 新增排除规则，完善截图/脚本/临时目录规则 |
| `package.json` | 新增 vue-router 依赖 |
| `tsconfig.json` | 启用 strict + noImplicitAny |
| `src/main.ts` | 注册 Vue Router 插件 |
| `src/App.vue` | 完全重构：使用 Router + Composables + 子组件 |
| `src-tauri/src/lib.rs` | 从 2157 行精简至 291 行，命令函数移至 commands/ |
| `src-tauri/src/gateway_config.rs` | 更新导入路径 |
| `src-tauri/src/bot_adapters/feishu.rs` | 更新导入路径 |
| `src-tauri/src/bot_adapters/telegram.rs` | 更新导入路径 |

### TypeScript 修复文件 (4 个)

| 路径 | 修复说明 |
|------|---------|
| `src/components/ExecutiveSessionView.vue` | 添加 invoke null 守卫 |
| `src/lib/collaboration/mock-data-generator.ts` | 空数组类型注解 |
| `src/components/agent-pet/GrowthSystem.vue` | 索引签名类型断言 |
| `src/lib/sync/websocket-sync-service.ts` | 回调参数类型注解 |

---

## 六、编译验证结果

| 检查项 | 结果 |
|--------|------|
| `npx vue-tsc --noEmit` (TypeScript strict mode) | 0 错误 |
| `cargo check` (Rust 编译) | 0 错误，30 个既有 dead code 警告 |
| `npm install` (依赖安装) | 成功 |

---

## 七、审查建议

对于 Claude Code 审查，建议重点关注以下方面：

1. **commands/ 模块拆分质量**：检查每个模块的 `use` 导入是否完整，是否有遗漏的依赖
2. **Vue Router 路由配置**：确认所有 22 个路由路径与 feature-registry.ts 中的 ID 一致
3. **Composables 提取**：检查是否有遗漏的清理逻辑（onBeforeUnmount）
4. **AppSidebar.vue**：确认所有 emit 事件在 App.vue 中正确处理
5. **App.vue 重构完整性**：确认原有功能（mock DAG 初始化、bot command 处理、重连逻辑）均正确迁移

---

## 八、后续路线图建议

以下是对项目未来方向的规划备忘，与本次重构无关但值得记录：

1. **可插拔能力系统**：参考 OpenClacky 省 token 模式，实现 Skill/MCP/Hook/CLI 的可插拔架构
2. **Agent 蜂群**：以 Codex / Claude Code 作为顶层 Agent 编排器，而非通过 subagent 启动
3. **Ultra Workflow**：实现类似 Claude Code 最新功能的超长工作流支持
4. **自我进化**：集成 Hermes/OpenClaw 的自我进化、自我修复能力（Hook 机制：agent 自我运行、检测、修复、思考）
5. **远程 Agent 管理**：类似 Qoder Mobile / Trae Mobile / Remote Claude Code 的远程查看和控制能力
6. **noUnusedLocals 清理**：开启后清理约 65 个未使用的导入/变量
7. **减少 any 使用**：逐步替换约 60 处 `any` 类型为精确类型
