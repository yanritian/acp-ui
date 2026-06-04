# ACP-UI 当前问题清单

> **发现日期**: 2026-06-03
> **当前版本**: v0.1.14
> **分支**: my-agent-teams-platform
> **最后更新**: 2026-06-04 10:03

---

## 一、构建问题 (阻断性)

### 1. TypeScript 编译失败 ✅ 已修复
**文件**: `src/lib/agent-runtime/output-buffer.ts`
**修复**: 添加 `isSessionNotification` 类型守卫函数
**提交**: 9bfc3eb

---

## 二、核心功能缺失 (阻断性)

### 2. 执行记录不保存 ✅ 已修复
**文件**: `src/stores/multi-session.ts`
**修复**: `sendPrompt()` 完成后调用 `getHistoryStore().saveTask()`
**提交**: 9bfc3eb

### 3. 对话消息不持久化 ✅ 已修复
**文件**: `src/stores/multi-session.ts`
**修复**: 添加 `messagesStore` 持久化，`applyOutputToSession` 后自动保存
**提交**: 待提交

### 4. Tauri 后端命令缺失 ✅ 已修复
**文件**: `src-tauri/src/lib.rs`, `src/lib/storage/history-store-tauri.ts`
**修复**: 添加 `save_task_history` Tauri 命令和前端调用
**提交**: 待提交

---

## 三、假实现问题 (经分析已正常工作)

根据代码审查，以下功能实际是正常工作的：

### 5. MultiAgentChat.vue ✅ 正常工作
调用 `teamRuntime.runTeamTask()` → `AgentTeamsService.runTeamTask()` → `AcpSessionRunner.prompt()`

### 6. WorkflowView.vue ✅ 正常工作
`runWorkflow()` 使用 `teamRuntime.runTeamTask()` 执行真实 Agent

### 7. TeamOrchestrationView.vue ✅ 正常工作
显示 `teamRuntime` 中的真实任务数据

### 8. orchestrator.ts ⚠️ 未被主流程使用
这是一个独立的任务调度系统，主流程使用 `AgentTeamsService`

### 9. AgentTeamsService ✅ 正常工作
调用 `AcpSessionRunner` 执行真实 Agent

### 10. GatewaySettings.vue ✅ 正常工作
通过 `invoke('save_gateway_config')` 持久化

### 11. BotSettings.vue ✅ 正常工作
通过 localStorage 或 Tauri invoke 持久化

---

## 四、优先级排序

| 优先级 | 问题 | 预估时间 |
|--------|------|----------|
| P0 | TypeScript 编译失败 | 10 分钟 |
| P0 | 执行记录不保存 | 20 分钟 |
| P1 | 对话消息不持久化 | 1 小时 |
| P1 | Tauri 命令实现 | 2 小时 |
| P2 | 假实现替换（5-11） | 需评估每个 |

---

## 五、修复策略

### Phase 1: 让 App 可运行 (P0)
1. 修复 TypeScript 类型错误
2. 实现执行记录保存

### Phase 2: 让数据可留存 (P1)
3. 对话消息持久化
4. Tauri 后端命令实现

### Phase 3: 让功能真实 (P2)
5. 逐个替换假实现，对接真实 Agent 执行

---

## 六、下一步行动

立即修复 P0 问题：
1. `output-buffer.ts` 类型断言
2. `multi-session.ts` 添加 `saveTask` 调用