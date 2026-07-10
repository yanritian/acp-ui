---
title: 'Secure Remote Operator Access'
type: 'feature'
created: '2026-07-10'
status: 'done'
baseline_commit: 'df8bcbce9ecd9ee41644d06c66135bd82c54083f'
context:
  - '{project-root}/docs/codex/2026-07-09-runtime-closure-check.md'
---

<frozen-after-approval reason="human-owned intent - do not modify unless human renegotiates">

## Intent

**Problem:** Remote Operator 已能真实控制任务，但默认监听 `0.0.0.0:1422`、无鉴权且 CORS 全开放，不能安全提供给 Web、IDE、移动端或局域网客户端。

**Approach:** 增加默认本机绑定、Bearer Token、Origin 限制、Client/Domain/项目根目录 allowlist 和脱敏审计；本机开发保持开箱即用，显式开放非回环地址时强制认证与项目边界。

## Boundaries & Constraints

**Always:** 默认绑定 `127.0.0.1`；非回环绑定必须配置 Token 和项目根目录；Token 恒定时间比较且不进入日志、响应或 Debug；远程写操作和鉴权失败进入有界审计；保留共享 `OperatorState` 与现有暂停/恢复/停止/重定向语义；本轮工具和测试输出只在 D 盘。

**Ask First:** TLS、账户/RBAC、数据库审计、删除或覆盖文件的二次审批。

**Never:** URL 携带 Token、任意 Origin CORS、复制或 mock Operator 状态、记录请求正文/Goal/审批评论/Secret、大范围整理无关代码。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Behavior | Error Handling |
|----------|---------------|-------------------|----------------|
| 本机默认 | 无 bind/token | 仅监听回环地址，合法本机客户端可用 | 配置错误时拒绝启动 |
| 远程启动 | 非回环 bind | Token + project roots 缺一不可 | 错误不含 Secret |
| 未授权 | Token 缺失/错误 | `401`，状态不变 | 写脱敏审计 |
| 越界任务 | client/domain/path 不在 allowlist | `403`，不创建任务 | 只记录拒绝类别 |
| 合法写操作 | start/control/approve | 保持原业务结果并生成审计 | 业务失败也记录状态 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/operator/security.rs` -- 远程访问策略与 allowlist 校验。
- `src-tauri/Cargo.toml` -- 固定长度 Token 摘要与恒定时间比较依赖。
- `src-tauri/src/http_server.rs` -- Axum 鉴权、CORS、审计、绑定和 HTTP 测试。
- `src-tauri/src/lib.rs` -- 读取 `ACP_OPERATOR_HTTP_*` 并启动。
- `src/api/operatorRemoteApi.ts`, `src/types/operator.ts` -- Token/客户端头、审计 SDK 类型。
- `src/api/operatorRemoteApi.test.ts` -- SDK Header 注入和 Token 不进 URL 的回归测试。
- `docs/codex/2026-07-09-runtime-closure-check.md` -- 配置与验证记录。

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/operator/security.rs` -- 先写测试，再实现 Token/Origin/Client/Domain/Root 策略。
- [x] `src-tauri/src/http_server.rs` -- 添加认证授权、审计查询、受限 CORS、bind config 与回环测试。
- [x] `src-tauri/src/lib.rs` -- 默认回环，环境变量显式开放远程。
- [x] `src/api/operatorRemoteApi.ts`, `src/types/operator.ts` -- SDK 自动注入认证和客户端标识。
- [x] `docs/codex/2026-07-09-runtime-closure-check.md` -- 记录配置、威胁边界和测试结果。

**Acceptance Criteria:**
- Given 默认环境，when 服务启动，then 仅绑定回环地址且本机任务闭环不回归。
- Given 非回环 bind 缺少 Token 或 roots，when 解析配置，then 拒绝启动。
- Given无效 Token/Origin/allowlist 输入，when 请求到达，then 返回 401/403 且状态不变。
- Given 合法客户端，when 执行远程写操作，then 原业务结果一致且可查询脱敏审计。

## Spec Change Log

## Design Notes

认证授权留在 HTTP 边界，Operator core 继续与传输无关。Token 只保留 SHA-256 固定长度摘要并恒定时间比较；远程创建时改存 canonical 项目路径，所有既有任务读取和控制也重新校验 scope。审计先用独立有界内存队列，避免把未授权请求伪造成任务事件；长期留存后续再接数据库。

## Verification

**Commands:**
- `D:\Rust\.cargo\bin\cargo.exe test operator::security::tests --lib`
- `D:\Rust\.cargo\bin\cargo.exe test http_server::tests --lib`
- `D:\Rust\.cargo\bin\cargo.exe test operator::commands::tests --lib`
- `D:\Rust\.cargo\bin\cargo.exe test --lib`
- D 盘 Node 可用后运行 `npm run typecheck`；否则明确记录未验证。

## Suggested Review Order

**HTTP 安全边界**

- 统一认证、Origin、Client 和脱敏审计入口。
  [`http_server.rs:357`](../../src-tauri/src/http_server.rs#L357)

- 远程监听配置默认回环并拒绝弱配置。
  [`http_server.rs:187`](../../src-tauri/src/http_server.rs#L187)

**策略与任务范围**

- Token 固定摘要、恒定时间比较和 allowlist 核心。
  [`security.rs:336`](../../src-tauri/src/operator/security.rs#L336)

- canonical 项目路径消除原始别名继续传播。
  [`security.rs:447`](../../src-tauri/src/operator/security.rs#L447)

- 所有既有任务读取和控制统一重验 scope。
  [`http_server.rs:1074`](../../src-tauri/src/http_server.rs#L1074)

- 新任务入状态前完成 Domain、Root 和 canonical 校验。
  [`http_server.rs:821`](../../src-tauri/src/http_server.rs#L821)

**启动与客户端**

- Tauri 启动时集中解析并拒绝不安全环境配置。
  [`lib.rs:413`](../../src-tauri/src/lib.rs#L413)

- SDK 自动注入 Bearer 和客户端标识并查询审计。
  [`operatorRemoteApi.ts:74`](../../src/api/operatorRemoteApi.ts#L74)

- 前端共享审计 schema 与 Rust JSON 对齐。
  [`operator.ts:256`](../../src/types/operator.ts#L256)

**验证与文档**

- HTTP 回归覆盖鉴权、CORS、scope、canonical 和脱敏。
  [`http_server.rs:1689`](../../src-tauri/src/http_server.rs#L1689)

- SDK 回归证明 Token 只进 Header、不进 URL。
  [`operatorRemoteApi.test.ts:5`](../../src/api/operatorRemoteApi.test.ts#L5)

- 安全配置、威胁边界和最终测试结果集中记录。
  [`2026-07-09-runtime-closure-check.md:267`](../../docs/codex/2026-07-09-runtime-closure-check.md#L267)

- 固定摘要与恒定时间依赖显式纳入主 crate。
  [`Cargo.toml:82`](../../src-tauri/Cargo.toml#L82)
