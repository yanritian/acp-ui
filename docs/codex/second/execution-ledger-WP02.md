---
name: WP-02 Operator 协议唯一来源
description: 2026-07-13 WP-02 协议状态分析
type: project
---

## 1. 运行摘要

```text
日期：2026-07-13
执行者：Claude Code + Qwen
仓库：D:/dingsun/acp-ui
分支：cleanup/project-snapshot-2026-06-25
起始 HEAD：79f7d8c
工作包：WP-02
目标：建立 Operator 协议唯一来源，定义 JSON Schema，确保类型一致性
是否修改代码：no (分析阶段)
是否修改文档：yes (协议分析报告)
是否产生提交：pending
```

## 2. 当前协议定义位置

| 类型 | Rust (src-tauri/src/operator/types.rs) | TypeScript (src/types/operator.ts) | VSCode | IDEA |
|---|---|---|---|---|
| OperatorTaskStatus | ✅ enum | ✅ type | ⚠️ string | ⚠️ String |
| OperatorTask | ✅ struct | ✅ interface | ⚠️ 重复定义 | ⚠️ 重复定义 |
| OperatorEventType | ✅ enum (28 种) | ✅ type (28 种) | ❌ 缺失 | ❌ 缺失 |
| OperatorEvent | ✅ struct | ✅ interface | ⚠️ 重复定义 | ⚠️ 重复定义 |
| ApprovalLevel | ✅ enum | ✅ type | ❌ 缺失 | ❌ 缺失 |
| ApprovalDecision | ✅ enum | ✅ type | ✅ 使用 | ✅ 使用 |
| ApprovalRequest | ✅ struct | ✅ interface | ⚠️ 重复定义 | ⚠️ 重复定义 |
| ErrorCategory | ✅ enum | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |
| ErrorCodes | ✅ codes 模块 | ❌ 缺失 | ❌ 缺失 | ❌ 缺失 |

## 3. 一致性问题

### 3.1 缺少 revision 字段

Rust `OperatorTask` 和 TypeScript `OperatorTask` 都缺少 `revision` 字段，无法实现乐观并发控制。

文档要求：
> 所有写操作都带 `expected_revision`。版本不匹配时返回 `409 REVISION_CONFLICT`。

### 3.2 客户端类型重复定义

- **VSCode**: `clients/vscode-game-operator/src/client.ts` 定义了 `OperatorTask`, `OperatorEvent`, `ApprovalRequest`
- **IDEA**: `clients/idea-game-operator/src/main/kotlin/.../api/GameOperatorApiClient.kt` 定义了相同类型

问题：
- 类型字段可能与主定义不同步
- 缺少事件类型枚举，使用自由字符串

### 3.3 缺少统一 JSON Schema

`schemas/` 目录不存在，没有版本化的协议定义。

### 3.4 默认端口一致性

| 客户端 | 默认端口 | 配置位置 |
|---|---|---|
| VSCode | `http://127.0.0.1:1422` | `package.json` 配置项 |
| IDEA | 无默认 | 构造函数参数 |
| Web/Tauri | N/A | 使用 Tauri invoke |

VSCode 和 IDEA 使用相同端口 1422，符合文档要求。

### 3.5 API 路径一致性

| 端点 | VSCode | IDEA | 文档要求 |
|---|---|---|---|
| GET /api/health | ✅ | ✅ | ✅ |
| GET /api/operator/tasks | ✅ | ✅ | ✅ |
| POST /api/operator/tasks | ✅ | ✅ | ✅ |
| POST /api/operator/tasks/{id}/pause | ✅ | ✅ | ✅ |
| POST /api/operator/tasks/{id}/resume | ✅ | ✅ | ✅ |
| POST /api/operator/tasks/{id}/stop | ✅ | ✅ | ✅ |
| GET /api/operator/tasks/{id}/events | ✅ | ✅ | ✅ |
| GET /api/operator/tasks/{id}/approvals | ✅ | ✅ | ✅ |
| POST /api/operator/approvals/decision | ✅ | ✅ | ✅ |

API 路径一致，符合文档要求。

## 4. 待完成任务

WP-02 需要完成以下工作：

### 4.1 创建 JSON Schema

```text
schemas/operator/
├── v1/
│   ├── task.schema.json
│   ├── event.schema.json
│   ├── approval.schema.json
│   ├── patch-proposal.schema.json
│   ├── checkpoint.schema.json
│   └── error-response.schema.json
```

### 4.2 添加 revision 字段

修改以下类型：
- Rust: `OperatorTask`, `StartTaskRequest`, `ApproveRequest`
- TypeScript: 对应接口
- 客户端: 对应 data class

### 4.3 建立枚举定义

创建共享枚举：
- `OperatorTaskStatus`
- `OperatorEventType`
- `ApprovalLevel`
- `ApprovalDecision`
- `ErrorCategory`
- `ErrorCodes`

### 4.4 契约测试

创建 `tests/contract/` 目录，验证类型字段一致性。

## 5. 风险与阻塞项

- **风险**: 修改 OperatorTask 可能影响现有测试
- **阻塞**: 需要先完成 revision 字段设计再开始 WP-03 状态机
- **依赖**: 无外部依赖

## 6. 下一步

1. 创建 `schemas/operator/` 目录结构
2. 定义核心类型 JSON Schema
3. 添加 `revision` 字段到 Rust DTO
4. 同步 TypeScript 和客户端类型
5. 创建契约测试

## 7. 发现问题

### 7.1 测试失败

Rust 测试有 11 个失败，都是 operator::commands::tests 模块：
- `approving_ready_task_uses_hermes_game_cli_when_available` - 状态断言失败
- `background_hermes_game_task_can_be_cancelled`
- `background_hermes_game_task_can_be_redirected`
- `background_hermes_game_task_can_pause_and_resume`
- `failed_godot_validation_keeps_applied_files_and_backup_visible`
- `hermes_connection_status_accepts_explicit_cli_path`
- `missing_pending_patch_does_not_consume_the_approval`
- `passed_godot_validation_completes_the_applied_patch_task`
- `patch_approval_refuses_to_overwrite_an_operator_edit`
- `pending_patch_survives_restart_and_keeps_second_approval`
- `stopping_while_patch_waits_invalidates_the_pending_write`

这些测试失败可能是 Hermes CLI 环境问题或测试本身的问题，需要单独调查。

### 7.2 修改影响范围

添加 revision 字段需要修改以下文件：
- `src-tauri/src/operator/commands.rs` (约 20 处初始化)
- `src-tauri/src/operator/agent_bridge.rs`
- `src-tauri/src/operator/hermes_cli_bridge.rs`
- `src-tauri/src/operator/state_machine.rs`
- `src-tauri/src/operator/task_executor.rs`
- `src-tauri/src/approval_engine.rs`
- `src-tauri/src/http_server.rs`

建议将 WP-02 拆分为多个子任务逐步完成。

## 8. 结论

> WP-02 进入分析阶段。已创建 JSON Schema 定义协议结构。发现 11 个 Rust 测试失败需要调查。revision 字段修改影响范围大，建议拆分子任务逐步完成。