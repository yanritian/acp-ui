# 测试、证据与验收

## 1. 测试层级

### L0：静态检查

- TypeScript/Vue 类型检查。
- Rust `cargo fmt --check`、`cargo clippy`。
- Kotlin/Gradle 编译。
- VSCode `compile`、`lint`。
- `git diff --check`、秘密扫描、生成物扫描。

### L1：模块单元测试

覆盖状态机、错误码、路径边界、revision、审批策略、patch hash、记忆、checkpoint、Hook 超时和 locale key。

### L2：协议契约测试

同一组 fixture 分别由 Web API adapter、VSCode client、IDEA client 读取和写入，断言 URL、方法、body、错误码和响应字段一致。

### L3：服务集成测试

启动真实 Rust remote service，使用 HTTP 客户端完成任务生命周期，验证事件顺序、审批、重启恢复和审计。

### L4：领域实机测试

绑定固定 Godot fixture，真实扫描、补丁、headless 验证和运行结果回传；Godot 或 Hermes 缺失时测试必须明确失败，不能自动 skip 成绿色。

### L5：客户端 E2E

Playwright 验证 Web；VSCode Extension Host 验证命令和树视图；IDEA sandbox 验证工具窗口和 action；Tauri driver 验证桌面窗口、远程配置和关闭回收。

## 2. D 盘验证约定

先发现可执行文件：

```powershell
Get-ChildItem D:\ -Recurse -Filter node.exe -File -ErrorAction SilentlyContinue | Select-Object -First 20 -ExpandProperty FullName
Get-ChildItem D:\ -Recurse -Filter gradle.bat -File -ErrorAction SilentlyContinue | Select-Object -First 20 -ExpandProperty FullName
```

实际执行必须使用发现到的 D 盘路径，测试临时目录使用 `D:\dingsun\acp-ui\.tmp-tests`。不要因为 C 盘有 Node 或 Java 就切换过去。

核心命令：

```powershell
node node_modules\vue-tsc\bin\vue-tsc.js --noEmit
node node_modules\vitest\vitest.mjs run --reporter=basic
node node_modules\vite\bin\vite.js build
cargo test --manifest-path src-tauri\Cargo.toml --lib -- --test-threads=1
clients\idea-game-operator\gradlew.bat --no-daemon build
clients\vscode-game-operator\npm.cmd ci --ignore-scripts
clients\vscode-game-operator\npm.cmd run pretest
node node_modules\@playwright\test\cli.js test --config=playwright.config.ts tests/e2e/game-operator.spec.ts --project=chromium
```

命令中的 `node`、`npm.cmd` 需要替换成 D 盘实际可执行文件；每条命令记录退出码。

## 3. 完整游戏闭环验收

验收脚本必须按以下顺序执行：

1. 启动真实 Operator server，记录端口、版本和健康检查响应。
2. 创建一个 Godot fixture 任务，记录 `task_id`、初始 revision 和事件序号。
3. 等待计划事件，确认计划包含目标文件、风险和验证命令。
4. 在生成补丁前点击暂停，确认执行停在安全边界并保存 checkpoint。
5. 修改目标或计划约束，确认 revision 增加且旧计划不能直接 apply。
6. 恢复任务，确认新计划继续执行。
7. 在高风险补丁处确认审批项包含 diff、task id、approval id 和 hash。
8. 拒绝一次，确认没有文件变化；再重新生成并批准一次。
9. 运行真实 Godot 验证，确认退出码、日志引用和结果事件。
10. 在验证期间暂停、恢复和停止各执行一次，确认状态和审计正确。
11. 重启 Operator server，确认任务、事件、checkpoint 和待审批项可恢复。
12. 从 Web、VSCode、IDEA 读取同一任务，确认状态、revision、事件序号一致。

## 4. 验收证据格式

每个验收项必须同时有：命令或操作、开始时间、结束时间、退出码、版本、输入 fixture、关键输出、日志路径和结论。截图只能证明 UI，不足以证明后端文件发生了正确变化。

## 5. 质量门禁

以下任一项失败都不能写“完成”：

- 类型检查失败。
- 受影响模块测试失败。
- Rust 编译失败。
- 客户端 compile/lint 失败。
- 协议契约字段不一致。
- 高风险动作绕过审批。
- 真实 fixture 没有验证结果。
- 远程请求没有认证或审计。
- 新增 locale key 没有 13 个语言实现。
- Git diff 含生成物、秘密、C 盘绝对路径或用户私有文件。
