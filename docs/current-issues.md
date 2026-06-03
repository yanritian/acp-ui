# ACP-UI 当前问题清单

> **发现日期**: 2026-06-03
> **当前版本**: v0.1.14
> **分支**: my-agent-teams-platform
> **最后更新**: 2026-06-03 23:31

---

## 一、构建问题 (阻断性)

### 1. TypeScript 编译失败 ✅ 已修复
**文件**: `src/lib/agent-runtime/output-buffer.ts`
**修复**: 添加 `isSessionNotification` 类型守卫函数
**提交**: 待提交

---

## 二、核心功能缺失 (阻断性)

### 2. 执行记录不保存 ✅ 已修复
**文件**: `src/stores/multi-session.ts`
**修复**: `sendPrompt()` 完成后调用 `getHistoryStore().saveTask()`
**提交**: 待提交

### 3. 对话消息不持久化
**文件**: `src/stores/multi-session.ts`
**问题**: `session.messages` 只存内存，刷新页面就消失
**影响**: 用户无法查看历史对话
**修复难度**: 中（需要设计存储方案）

### 4. Tauri 后端命令缺失
**文件**: `src/lib/storage/history-store-tauri.ts`
**问题**: `saveTask` 标记 TODO，未实现 Tauri 命令
**影响**: 桌面端无法保存历史
**修复难度**: 中（需要 Rust 后端实现）

---

## 三、假实现问题 (功能性问题)

根据 `docs/project-completion-plan.md` 文档标注：

### 5. MultiAgentChat.vue - 假响应
**位置**: `src/components/MultiAgentChat.vue`
**问题**: 返回模拟数据，不是真实 Agent 执行结果
**影响**: 多 Agent 协作演示是假的

### 6. WorkflowView.vue - 固定示例数据
**位置**: `src/components/WorkflowView.vue`
**问题**: 显示硬编码的示例 DAG，不反映实际执行状态
**影响**: 工作流可视化无意义

### 7. TeamOrchestrationView.vue - 空面板
**位置**: `src/components/TeamOrchestrationView.vue`
**问题**: 界面是空的，没有实际功能
**影响**: Agent 团队编排不可用

### 8. orchestrator.ts - setTimeout 模拟执行
**位置**: `src/lib/orchestrator.ts`
**问题**: 用 setTimeout 假装执行，不调用真实 Agent
**影响**: 所有编排逻辑是假的

### 9. multi-agent/bridge.ts - output 空字符串
**位置**: `src/lib/multi-agent/bridge.ts`
**问题**: output 永远返回空字符串
**影响**: 多 Agent 输出无内容

### 10. workflow/skill-engine.ts - 模拟成功
**位置**: `src/lib/workflow/skill-engine.ts`
**问题**: 永远返回成功，不实际执行 skill
**影响**: Skill 执行是假的

### 11. core/agent-teams-service.ts - 返回空对象
**位置**: `src/lib/core/agent-teams-service.ts`
**问题**: 返回空对象 `{}`
**影响**: Agent Teams 服务不工作

### 12. GatewaySettings.vue - 未持久化
**位置**: `src/components/GatewaySettings.vue`
**问题**: 设置不保存，刷新就丢失
**影响**: Gateway 配置无法保留

### 13. BotSettings.vue - 只监听事件
**位置**: `src/components/BotSettings.vue`
**问题**: 只监听 bot-command 事件，无实际 Bot 连接
**影响**: Bot 集成不工作

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