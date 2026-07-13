# 2026-07-13 直接接管后的验收记录

## 1. 结论先行

本轮已经由 Codex 直接在 `D:/dingsun/acp-ui` 完成修复和验证，没有继续依赖 Claude Code 或 Qwen。

核心游戏 Agent 后端闭环已经有真实证据：

`开始任务 -> 生成 Hermes Game 提案 -> 人工审批 -> 结构化补丁审批 -> 写入 Godot 项目 -> Godot 4.7 headless 验证 -> completed -> checkpoint/memory snapshot`

Rust 全量测试通过。当前不能声称所有客户端都完成最终验收，原因是用户明确禁止使用 C 盘，而本机 D 盘没有可用的 Node.js 运行时，也没有 JDK 17+。因此前端和 IDEA 的最终命令级验证必须等 D 盘运行时补齐后再做。

## 2. 本轮实际修复

### 2.1 Operator 并发控制和事件分页

- 任务和事件增加 `revision`、`sequence`、`task_revision`。
- pause/resume/stop/redirect/approval 支持 `expected_revision`。
- 过期写入返回 `REVISION_CONFLICT`，避免远程客户端覆盖更新后的任务。
- HTTP 事件接口支持 `after_sequence`。
- 修复游标分页 bug：带 `after_sequence` 时只返回游标之后的下一页，不再混入旧事件；无游标时仍返回最新一页。
- 新增分页边界测试，覆盖首屏、连续翻页、超大游标和 `limit=0`。

### 2.2 Godot 游戏闭环

- Godot `.tscn` 节点、属性、连接解析支持带引号逗号的属性值。
- Hermes Game proposal-only 模式只生成结构化提案，不直接写游戏项目。
- 补丁审批前生成 diff，应用前重新校验文件状态，应用后执行 Godot 验证。
- 任务完成时写入 checkpoint 和 memory snapshot 事件。
- 真实 Hermes Game + Godot 测试覆盖提案、审批、写入、headless 验证和 completed 状态。
- 兼容 Hermes CLI 可能输出的 `expected_sha256: ""`：空字符串按未提供处理，非空值仍必须是 64 位 SHA-256 并参与冲突检查。

### 2.3 Skill

- `invoke_skill` 不再为未知 Skill 返回 mock 成功。
- 当前只公开有真实后端处理器的内置 Skill：
  - `godot-analyze`
  - `godot-codegen`
- 新增真实 `skills/godot-analyze/SKILL.md` 和 `skills/godot-codegen/SKILL.md`。
- `godot-analyze` 只读解析项目。
- `godot-codegen` 只生成 proposal artifact，写入必须经过 Operator 补丁审批。
- Skill 管理不允许覆盖或删除内置 Skill。
- 前端 SkillInvoker 改为使用 `invokeOrProxy`，并按 Tauri 实际返回对象处理，不再错误地对对象执行 `JSON.parse`。
- Plugin Registry 不再伪造 16 个 healthy core skills，也不再默认注册需要 `npx` 的虚假 MCP；只注册两个真实 Godot Skill，MCP 必须在真实 D 盘可执行文件配置并连接后出现。

### 2.4 Hook 和 MCP 安全

- Hook 脚本必须是 D 盘工作区内的 regular file。
- 修复 Windows `canonicalize()` 返回 `\\?\\D:` 扩展路径导致合法 Hook 被误判越界的问题。
- Hook 超时限制为 120 秒，stdout/stderr 限制为 64 KiB。
- MCP 命令必须是 D 盘绝对可执行文件，响应限制为 2 MiB。
- MCP stdout 读取增加 30 秒超时；超时会杀掉子进程并返回错误，不会永久阻塞 Agent。
- 新增真实 Python stdio MCP fixture，完成 initialize、tools/list、tools/call、disconnect。

## 3. 已执行命令和证据

以下命令均从 D 盘执行。Rust 使用固定 D 盘工具链：

```powershell
$env:RUSTUP_HOME='D:\Rust\.rustup'
$env:CARGO_HOME='D:\Rust\.cargo'
D:\Rust\.cargo\bin\rustup.exe run stable cargo ...
```

### 3.1 Rust 编译

```text
命令：rustup run stable cargo test --manifest-path D:\dingsun\acp-ui\src-tauri\Cargo.toml --no-run
退出码：0
结论：主 crate、测试 crate 和 examples 成功编译。
```

### 3.2 Rust 全量测试

```text
命令：rustup run stable cargo test --manifest-path D:\dingsun\acp-ui\src-tauri\Cargo.toml -q -- --test-threads=1
退出码：0
主库：320 passed, 0 failed, 4 ignored
其余集成目标：全部 passed
```

说明：默认并行执行曾出现一次 `bounded_process_times_out_and_is_reaped`
的机器调度抖动，单测单独执行为 0.19 秒，串行全量执行稳定通过。因此本记录以
串行全量结果作为最终 Rust 验收证据，不把一次并行抖动写成业务失败或伪造成功。

### 3.3 真实 MCP stdio

```text
命令：cargo test --lib mcp_client::tests::real_stdio_fixture_roundtrip_discovers_and_calls_a_tool -- --ignored --nocapture
环境：ACP_REAL_PYTHON_BIN=D:\Python313\python.exe
退出码：0
证据：MCP Client: Connected to 'echo' with 1 tools；随后正常 Disconnected。
```

### 3.4 真实 Hermes Game + Godot

```text
命令：cargo test --lib operator::commands::tests::real_hermes_godot_closed_loop_applies_and_validates_a_patch -- --ignored --nocapture
环境：
  ACP_REAL_HERMES_GAME_CLI=D:\dingsun\acp-ui\bin\hermes-game.exe
  ACP_REAL_GODOT_BIN=D:\dev-tools\godot\4.7-stable\Godot_v4.7-stable_win64_console.exe
退出码：0
证据：1 passed；事件包含 ValidationPassed；任务状态为 Completed；Godot 项目文件确实发生变更。
```

### 3.5 关键回归测试

```text
Hook fixture：1 passed
HTTP after_sequence 分页：1 passed
Godot 场景解析：2 passed
结构化补丁空 expected_sha256：1 passed
```

## 4. 未完成的验证和原因

### 4.1 IDEA 插件

尝试过：

```text
D:\dingsun\acp-ui\clients\idea-game-operator\gradlew.bat build --no-daemon
结果：超过 600 秒没有完成，已停止后台 Gradle 子进程。
```

为遵守 D 盘约束，又尝试使用 D 盘唯一发现的 Java：

```text
D:\Java\jdk8\bin\java.exe
结果：离线 Gradle 构建仍超时，未把它当作成功。
```

当前 D 盘没有 JDK 17+。IDEA Gradle 构建不能用 C 盘的 `C:\Program Files\Java` 代替。补齐 D 盘 JDK 17+ 后，应重新执行：

```powershell
$env:JAVA_HOME='D:\Java\jdk-17'
$env:Path="$env:JAVA_HOME\bin;$env:Path"
Set-Location D:\dingsun\acp-ui\clients\idea-game-operator
.\gradlew.bat clean build --no-daemon --offline
```

### 4.2 Web/Tauri 前端

已检查 D 盘常见工具目录，没有找到 `node.exe`、`bun.exe`、`deno.exe` 或可用替代运行时。故以下命令本轮没有伪造执行结果：

```text
npm run typecheck
npm run test
npm run build
npm run test:e2e
```

补齐 D 盘 Node.js 后，必须依次执行并把真实退出码追加到本目录台账：

```powershell
Set-Location D:\dingsun\acp-ui
D:\nodejs\node.exe node_modules\vue-tsc\bin\vue-tsc.js --noEmit
D:\nodejs\node.exe node_modules\vitest\vitest.mjs run
D:\nodejs\node.exe node_modules\vite\bin\vite.js build
D:\nodejs\node.exe node_modules\@playwright\test\cli.js test
```

## 5. 当前工作树处理规则

本轮不擅自处理以下已有异常：

- `.claude/scheduled_tasks.lock` 的删除状态。
- 根目录异常命名文件 `Ddingsunacp-uidocsFAQ.md`。

本轮测试产生的 Godot `.godot`、`.uid` 和临时 proposal 文件属于验证产物，不应进入功能提交；清理动作只针对已确认由测试生成的路径。

## 6. 最终判断

可以确认：游戏 Agent 的 Rust Operator、真实 Hermes Game、Godot headless、结构化补丁审批、事件游标、Hook、MCP stdio 和持久化主链已经形成可运行闭环。

不能确认：前端当前工作树已经通过最新 Node 测试，或 IDEA 插件已经通过本轮构建。它们是运行时缺失导致的可复现环境阻塞，不是被静态检查掩盖的“完成”。

下次验收只需要补齐 D 盘 Node.js 与 JDK 17+，完成第 4 节命令，并把结果追加到本文件；不要把旧报告中“100% 完成”之类没有当前命令证据的文字当作验收依据。
