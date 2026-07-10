---
title: 'Hermes Structured Patch Approval'
type: 'feature'
created: '2026-07-10'
status: 'done'
baseline_commit: 'df8bcbce9ecd9ee41644d06c66135bd82c54083f'
context:
  - '{project-root}/docs/codex/2026-07-09-runtime-closure-check.md'
  - '{project-root}/_bmad-output/implementation-artifacts/spec-secure-remote-operator-access.md'
---

<frozen-after-approval reason="human-owned intent - do not modify unless human renegotiates">

## Intent

**Problem:** 当前 Hermes Game 在首轮计划审批后只生成 `proposal.md`，任务随即标记完成。Agent 没有结构化、可验证的改动合同，操作员看不到逐文件 diff，也没有独立的应用审批；同时 Hermes Game 注册了终端和写文件工具，仅靠提示词无法保证“只提案、不落盘”。

**Approach:** 把流程拆成两个不可合并的审批关口：首轮审批仅允许无工具 Hermes Game 生成版本化 JSON 补丁集；ACP 严格解析并生成 diff，任务回到 `waiting_approval`；第二次审批才允许受 `PathGuard` 约束的事务式应用。应用前重新校验原文件哈希，集中备份，任一写入失败则回滚。

## Boundaries & Constraints

**Always:** Hermes 提案阶段关闭全部工具；只接受 UTF-8 JSON；路径必须是项目内、使用 `/` 的相对路径；首版只允许 `create` 与 `replace`；预览和应用都校验边界；应用前复核哈希；替换文件必须备份；任务在补丁审批前不得完成；所有本轮构建、缓存、测试和产物只写 D 盘。

**Ask First:** 删除文件、移动/重命名、自动执行 Agent 提供的命令、覆盖哈希已变化的文件、跨项目改动、二进制资源改写。

**Never:** 从 Markdown 代码块猜测补丁；解析或执行任意 shell；使用绝对路径或 `..`；写入 `.git`、`.godot`、`node_modules`、`target`、`build`、`dist`；在提案阶段把终端/写文件/技能工具暴露给模型；审批后跳过 stale-check。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Behavior | Error Handling |
|----------|---------------|-------------------|----------------|
| 合法替换 | 已存在 UTF-8 文件 + 完整新内容 | 生成 diff 和原内容哈希，等待第二次审批 | 不改项目文件 |
| 合法创建 | 不存在文件，父目录已存在 | 生成全新增量预览，等待审批 | 不提前创建目标文件 |
| 非法结构 | Markdown、未知版本、未知字段、空 changes | 拒绝补丁集，任务失败且保留原始 artifact | 错误指出 schema 原因，不尝试修复模型输出 |
| 路径越界 | 绝对路径、反斜杠、`..`、受禁目录、符号链接 | 拒绝整个补丁集 | 不产生应用审批 |
| 内容过期 | 预览后操作员修改目标文件 | 第二次审批时哈希不一致 | 拒绝应用，不覆盖用户改动 |
| 中途失败 | 多文件写入中任一失败 | 回滚此前已替换/创建的文件 | 任务失败，报告回滚结果并保留备份 |
| 拒绝补丁 | patch approval = reject | 任务取消，项目不变 | 清理内存中的 pending patch |
| 请求修改 | patch approval = request_changes | 丢弃旧 pending patch，回到规划/新提案路径 | 旧审批不能再次使用 |
| 重定向/停止 | 等待补丁审批时操作员介入 | 旧审批作废、pending patch 丢弃 | 不允许后续落盘 |

</frozen-after-approval>

## Structured Patch Contract v1

```json
{
  "version": 1,
  "summary": "Add double jump without changing existing input bindings.",
  "changes": [
    {
      "path": "scripts/player.gd",
      "operation": "replace",
      "content": "extends CharacterBody2D\n...",
      "expected_sha256": "optional lowercase hex digest"
    },
    {
      "path": "scripts/jump_state.gd",
      "operation": "create",
      "content": "class_name JumpState\n..."
    }
  ],
  "validation": [
    "Open the project with Godot and verify the main scene loads"
  ]
}
```

`expected_sha256` 是可选的模型断言；无论模型是否提供，ACP 都会在预览时记录真实哈希，并在应用前再次比较。`validation` 在本故事中只作为人工可见建议，不自动当作命令执行。

## Code Map

- `src-tauri/src/operator/structured_patch.rs` -- schema、路径/大小校验、diff、stale-check、备份、应用和回滚。
- `src-tauri/src/operator/hermes_game_bridge.rs` -- 受控 Godot 上下文、严格 JSON 提示、`--no-tools` 和 artifact 读取。
- `src-tauri/src/operator/commands.rs` -- pending patch 状态、第二次审批分派、任务事件与完成语义。
- `src-tauri/src/operator/state_machine.rs` -- `running -> waiting_approval` 与等待审批时停止/请求修改的合法转换。
- `src-tauri/src/operator/types.rs`, `src/types/operator.ts` -- 逐文件 diff 审批预览合同。
- `src/features/game-operator/components/ApprovalDrawer.vue` -- 在审批动作旁直接显示可展开 diff。
- `D:/dev-tools/hermes-game/src/main.rs`, `D:/dev-tools/hermes-game/src/engine.rs` -- CLI `--no-tools` 模式。
- `docs/codex/2026-07-09-runtime-closure-check.md` -- 运行闭环、边界与验证结果。

## Tasks & Acceptance

**Execution:**
- [x] 先写结构化 schema、越界、stale-check、备份/回滚失败用例。
- [x] Hermes Game 增加无工具模式，ACP 提案调用固定携带 `--no-tools`。
- [x] 生成受大小限制、跳过符号链接和敏感目录的 Godot 项目文本上下文。
- [x] 严格解析 v1 补丁集并生成逐文件预览；不合法时整个集合失败。
- [x] 将有效补丁保存为 pending patch，生成独立 `operator.patch.apply` 审批。
- [x] 审批后事务式应用、记录备份和 `FilePatchApplied`；拒绝 stale patch。
- [x] 重定向、停止、拒绝和请求修改均使旧 pending patch 不可再应用。
- [x] 前端审批面板显示 operation、path 和 diff，不暴露隐藏写入口。
- [x] 使用 D 盘工具链完成 Rust、TypeScript、Vitest 和真实 Hermes Game 冒烟验证。

**Acceptance Criteria:**
- Given 一个已批准的 Godot 计划，when Hermes Game 结束，then 项目文件保持不变，任务进入 `waiting_approval`，且存在一个带逐文件 diff 的 patch approval。
- Given 合法 patch approval，when 操作员批准，then 所有文件在项目边界内应用，替换文件有独立备份，任务完成并记录变更事件。
- Given 预览后文件内容发生变化，when 操作员批准，then 应用失败且用户的新内容不被覆盖。
- Given 任一无效或越界 change，when artifact 被解析，then 整个补丁集被拒绝且无项目文件变化。
- Given Hermes Game 提案运行，when 查看调用参数和工具注册，then `--no-tools` 生效，模型无法调用终端、文件写入或技能工具。
- Given 任务等待任一审批，when 操作员停止或重定向，then 原审批和 pending patch 均不可再触发写入。

## Design Notes

补丁集采用“完整目标文件内容”而不是让 ACP 猜测 unified diff 应如何应用。模型输出只是候选数据，真正的路径解析、原内容快照、diff、备份和写入都由确定性 Rust 代码完成。第二次审批通过 `approval.action` 分派，不能复用首轮“批准计划后启动 Agent”的分支。

备份放在 D 盘 ACP workspace 的 `.operator/hermes-runs/<task>/backups/<approval>/`，按项目相对路径保存；不会使用容易覆盖的相邻 `.bak`。应用先做全量 preflight，再备份，再写入；若写入阶段失败，则按逆序恢复已替换文件并删除本次已创建文件。

## Verification

**Commands:**
- `D:\Rust\.cargo\bin\cargo.exe test operator::structured_patch::tests --lib`
- `D:\Rust\.cargo\bin\cargo.exe test operator::commands::tests --lib`
- `D:\Rust\.cargo\bin\cargo.exe test --lib`
- D 盘 Node 直接运行 `vue-tsc --noEmit` 和相关 Vitest 文件。
- `D:\Rust\.cargo\bin\cargo.exe test` 与 `build --release`（workdir `D:\dev-tools\hermes-game`）。
- 使用 D 盘临时 Godot fixture 和 `D:\dingsun\acp-ui\bin\hermes-game.exe` 执行 proposal-only 在线冒烟。

**Results (2026-07-10):**

- ACP Rust：`282 passed; 0 failed; 1 ignored`。ignored 为显式真实 artifact 用例，已单独执行并 `1 passed`。
- Hermes Game Rust：`7 passed; 0 failed`。
- Vue TypeScript：`vue-tsc --noEmit` 通过。
- Game Operator + Remote SDK：5 个 Vitest 文件、100 个测试通过。
- 真实 Hermes Game：0.4.0，Qwen 在线生成严格 v1 JSON，工具数为 0，提案阶段原项目未改。
- 真实产物：通过同一 Rust parser，并成功应用到 D 盘隔离项目副本。
- 回归补充：pending patch 意外缺失时，patch approval 不会被提前消费，任务保持 `waiting_approval`。

## Suggested Review Order

1. `src-tauri/src/operator/structured_patch.rs`
2. `src-tauri/src/operator/hermes_game_bridge.rs`
3. `src-tauri/src/operator/commands.rs`
4. `src-tauri/src/operator/state_machine.rs`
5. `src-tauri/src/operator/types.rs`
6. `src-tauri/src/http_server.rs`
7. `src/features/game-operator/components/ApprovalDrawer.vue`
8. `D:/dev-tools/hermes-game/src/main.rs`
9. `D:/dev-tools/hermes-game/src/engine.rs`
10. `docs/codex/2026-07-10-game-agent-closure-and-roadmap.md`
