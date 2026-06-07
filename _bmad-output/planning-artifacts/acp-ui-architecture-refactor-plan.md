# ACP-UI 架构改造计划

## 核心定位
> 跨平台异步 AI Agent 控制器 —— 让开发者从"坐在电脑前"变成"随时随地控制"

## 架构策略
> 不做状态机，做状态机的定义工具。复杂度被引擎吸收，用户面对极简界面。

## 商业策略
> **先完全开源** — 大家一起用，一起完善
> 开源即增长，社区驱动迭代

## 核心原则
1. **本地优先** — 所有数据、执行、存储都在本机，本机即数字人
2. **安全隐私** — 全在本机，传输层加解密
3. **可靠性** — 回滚为主，失败可恢复
4. **全生态接入** — Claude Code / Codex / Cursor 等全部接入
5. **暂不考虑云端** — 本地为主
6. **驾驭而非替代** — ACP-UI 不做专业 Agent，而是驾驭已有 Agent

---

## 改造项完成状态

### P0 — 核心差异化

| # | 改造项 | 状态 | 提交 | 文件 |
|---|--------|------|------|------|
| 1 | 异步审批协议 | ✅ 完成 | e5989fa | `src-tauri/src/approval_engine.rs` |
| 2 | 移动端极简重设计 | ✅ 完成 | e5989fa | `acp_ui_flutter/lib/screens/*` |

### P1 — 扩展性基础

| # | 改造项 | 状态 | 提交 | 文件 |
|---|--------|------|------|------|
| 3 | 工作流定义 DSL | ✅ 完成 | e5989fa | `src/lib/workflow/workflow-dsl.ts` |
| 4 | 推送通知系统 | ✅ 完成 | e5989fa | `src/lib/notifications/push-service.ts` |
| 6 | Sync Engine 持久化 | ✅ 完成 | e5989fa | `src-tauri/src/sync_engine.rs` |
| 7 | Agent 驾驭层 | ✅ 完成 | e5989fa | `src-tauri/src/agent_orchestration.rs` |
| 8 | Bot 网关层 | ✅ 完成 | e5989fa | `src-tauri/src/bot_gateway.rs` |

### P2 — 生态开放

| # | 改造项 | 状态 | 提交 | 文件 |
|---|--------|------|------|------|
| 5 | ACP 协议文档 | ✅ 完成 | e5989fa | `src/lib/acp-protocol/index.ts` |

---

## 已有不需要改的部分
- ✅ Hermes Flow + Agent Teams
- ✅ 自我修复/进化引擎
- ✅ QA 自动化
- ✅ Smart Router
- ✅ Circuit Breaker

---

## 验证结果
- **Rust 编译**: 0 错误，59 警告（均为已有代码）
- **TypeScript 编译**: 0 错误
- **Flutter**: 0 错误
- **测试**: 236 全部通过
- **新增代码**: +5,517 行

*完成日期: 2026-06-07*
