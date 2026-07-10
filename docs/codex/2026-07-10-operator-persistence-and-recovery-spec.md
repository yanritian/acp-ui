# Operator 持久化与重启恢复规格

> 状态：已实现并验收  
> 日期：2026-07-10  
> 范围：Godot Game Agent 控制面，不扩展其他领域

## 一、目标

当前 Tauri 与 Remote HTTP 共享同一个 `Arc<Mutex<OperatorState>>`。本轮已经增加 D 盘 SQLite 持久化边界，使桌面进程重启后仍能恢复：

- 任务与终态。
- append-only 事件流。
- 计划审批和补丁审批。
- 文件变更记录。
- 待审批结构化补丁及其哈希快照。
- 补丁已应用但仍待 Godot 验证的元数据。
- 被重启打断前的执行阶段。

恢复不会自动启动 Hermes、Godot 或任何模型调用。只有操作员显式 resume 才能继续执行。

## 二、硬约束

1. 数据库、WAL、临时文件和测试夹具只允许位于 D 盘。
2. 默认数据库为 `D:\dingsun\acp-ui\.operator\operator-state.sqlite3`，可通过 `ACP_OPERATOR_STATE_DB` 指向其他 D 盘绝对路径。
3. 非 D 盘路径、UNC 路径、无法规范化到 D 盘的路径必须拒绝。
4. 不持久化 bearer token、API key、模型密钥或远程请求正文。
5. 模型补丁仍是不可信数据；恢复 pending patch 时必须重新走严格 parser、路径保护和 stale hash 校验。
6. 事件表只允许插入，不允许更新或删除。
7. schema 比当前程序更新时拒绝启动，不能静默降级或清空。
8. 持久化初始化失败时生产应用不得静默回退为内存模式。

## 三、Schema v1

SQLite 使用 `PRAGMA user_version = 1`，启用 WAL、foreign keys、busy timeout。业务数据采用稳定 JSON payload 保存，主键和顺序字段保持可查询性。

| 表 | 主键 | 用途 |
|---|---|---|
| `operator_meta` | `key` | 最后保存时间与实现元数据 |
| `operator_tasks` | `task_id` | 完整 `OperatorTask` JSON |
| `operator_events` | `event_id` | append-only `OperatorEvent`，含 task sequence |
| `operator_approvals` | `approval_id` | 完整审批与决策 JSON |
| `operator_file_changes` | `(task_id, position)` | 文件变更顺序快照 |
| `operator_pending_patches` | `task_id` | patch ID、项目根、可重新解析的 artifact JSON |
| `operator_pending_validations` | `task_id` | project、patch、backup、files_applied |
| `operator_recovery_origins` | `task_id` | 被重启打断前的状态 |

`operator_events` 通过 trigger 拒绝 UPDATE 和 DELETE。保存时使用 `INSERT OR IGNORE`；若相同 event ID 的 task、sequence、payload 或 created_at 任一不一致，保存失败。加载时要求每个任务的 sequence 从 0 连续递增，并复核 JSON 内的 event ID、task ID、timestamp，避免事件被悄悄重排或形成断档。

## 四、保存事务

每个控制面检查点保存完整可恢复快照，使用 `BEGIN IMMEDIATE`：

1. 先在内存中完成 JSON 序列化和 D 盘路径校验。
2. upsert tasks、approvals、file changes。
3. 仅追加尚未存在的 events。
4. 在同一事务中替换 pending patches、pending validations、recovery origins。
5. 更新 `last_saved_at`。
6. commit 后才向调用方报告持久化成功。

首个纵向闭环的检查点：任务创建并进入计划审批、审批决策后准备启动后台执行、后台执行结束、pause、resume、stop、redirect、patch apply 与 validation 结束。

## 五、补丁快照

`PreparedPatchSet` 包含规范化路径和预览时哈希，但不是直接 serde 类型。持久层将它转换为受限 `StructuredPatchSet` JSON：

- `version = 1`。
- 保存 summary、validation。
- 保存每个 path、operation、new content。
- replace 保存预览时 `expected_sha256`。
- create 不保存 expected hash。
- patch ID 单独保存，恢复成功后保留原 ID，使审批 `diff_id` 不漂移。

加载时调用现有 `prepare_structured_patch`。若目标已变化、create 目标已出现、项目不存在或路径边界变化，则不恢复可执行 patch：任务转为 failed，未决 patch 审批由 recovery 解决，事件说明 stale 原因。绝不直接反序列化成可写路径后执行。

## 六、恢复映射

| 持久状态 | 启动后状态 | 行为 |
|---|---|---|
| `waiting_approval` | 原样 | 审批可继续读取和决策 |
| `paused` | 原样 | 不启动子进程 |
| `planning` | `paused` | 记录 recovery origin；显式 resume 后重新分析并生成计划审批 |
| `redirecting` | `paused` | 按 planning 恢复，不跳过审批 |
| `running` | `paused` | 记录 recovery origin；显式 resume 后重启当前 agent 或 pending validation |
| `cancelling` | `cancelled` | 视为进程已退出，完成取消收口 |
| `idle` | `paused` | 作为未完成 planning 处理 |
| `completed/cancelled/failed` | 原样 | 不追加重复恢复事件 |

主动转为 paused 的任务追加 `TaskPaused` 事件，source 为 `operator_recovery`，payload 包含 `recovered_from`。`operator_recovery_origins` 必须随快照保存，保证恢复后再次重启仍知道原阶段。

## 七、Resume 语义

- origin 为 `planning`、`redirecting` 或 `idle`：显式 resume 只重新执行确定性的项目分析与计划生成，最终回到 `waiting_approval`；不能直接进入 Hermes 写提案阶段。
- origin 为 `running` 且存在 pending validation：重新执行固定 Godot 验证模板。
- origin 为 `running` 且无 pending validation：按现有 Hermes Game 受控提案路径重启执行。
- 原本就是 `paused`：沿现有 pause/resume 语义处理。
- resume 检查点先落库为 running/planning，再启动后台进程；若再次崩溃，下次启动仍会重新暂停。

## 八、故障策略

- 数据库损坏、schema 过新、JSON 不可解析：返回明确错误，生产不静默使用空状态。
- 保存失败：API 返回错误，不宣称操作已持久化。
- 相同 event ID 内容冲突：视为审计一致性错误。
- stale pending patch：只使对应任务失败，不阻止其他健康任务恢复。
- 缺少 Godot：沿现有 `ValidationSkipped`，不能写成 passed。
- SQLite 中不得保存 Remote Operator bearer token；远程审计持久化不在 schema v1 范围内。

## 九、验收测试

1. 非 D 数据库路径拒绝且不创建文件。
2. 首次建库得到 schema v1，重复打开幂等。
3. schema 版本高于程序时拒绝。
4. task/event/approval/file change roundtrip。
5. event trigger 拒绝 update/delete，重复同内容保存幂等。
6. waiting approval 重启后仍可审批。
7. running 重启后变 paused，且不 spawn 子进程。
8. recovery origin 经第二次重启仍存在。
9. planning 显式 resume 后回到新的 plan approval。
10. pending patch 正常恢复；文件被用户修改后恢复为 stale failure。
11. pending validation 恢复后，显式 resume 可重新运行验证。
12. 一次失败保存不覆盖上一个已提交快照。
13. Remote HTTP 从同一恢复后的 `OperatorState` 读取任务、事件和审批。
14. ACP Rust 全量、真实 Hermes artifact、真实 Godot、Hermes Game 和前端全量保持通过。
15. 已持久化事件在内存中被重排时，后续保存拒绝并保留原事务。
16. 数据库事件 sequence 出现断档时，启动恢复明确失败。

## 十、实现与验收证据

核心实现：

- `src-tauri/src/operator/persistence.rs`：D 盘路径守卫、schema v1、事务快照、append-only 事件和严格加载校验。
- `src-tauri/src/operator/commands.rs`：持久化检查点、启动恢复、stale patch 收口和显式 resume。
- `src-tauri/src/operator/state_machine.rs`：从持久状态和事件历史恢复状态机。
- `src-tauri/src/lib.rs`：生产启动使用默认持久库，初始化失败拒绝静默回退到内存。
- `src-tauri/src/http_server.rs`：Remote HTTP 与 Tauri 读取同一份恢复状态，并有真实 Axum 路由测试。

2026-07-10 验收结果：

```text
ACP Rust default suite: 312 passed, 0 failed, 2 ignored
Real Hermes artifact apply: 1 passed, 0 failed
Real Godot 4.7 validation: 1 passed, 0 failed
Hermes Game Rust: 7 passed, 0 failed
Frontend Vitest: 79 files, 1209 tests passed
Vue typecheck: passed, 0 errors
cargo fmt --check: passed
```

真实纵向链路使用同一份隔离项目：Qwen/Hermes 的结构化产物先通过严格 parser、备份并应用，再由 `D:\dev-tools\godot\godot.exe` 4.7.stable 对应用后的项目执行固定 headless editor 校验。默认 Rust 套件中的两个 ignored 用例均已通过显式 D 盘变量单独执行。

测试覆盖的重启窗口包括：等待计划审批、运行中断、二次重启保留 recovery origin、规划中断后显式 resume、pending validation、pending patch 二次审批、停机期间用户修改导致 stale、事务失败回滚、未来 schema、非 D 路径、事件重排/断档和 Remote HTTP 恢复读取。

## 十一、非目标

- 不在本轮实现云同步、多设备冲突合并或公网数据库。
- 不持久化模型密钥、HTTP token 或完整远程请求体。
- 不恢复已退出进程的 PID。
- 不自动执行任何未完成任务。
- 不顺手清理全仓 warning、旧 adapter 或其他领域代码。
