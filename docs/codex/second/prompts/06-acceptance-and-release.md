# 真实验收与发布提示词

```text
你负责 WP-11。你的任务是判断当前版本是否真的达到“Godot 游戏开发 Agent 可用”，而不是替项目写宣传总结。

先读取：
- docs/codex/second/01-current-state-and-acceptance.md
- docs/codex/second/08-testing-and-acceptance.md
- docs/codex/second/10-execution-ledger-template.md
- docs/codex/second/05-game-domain-godot.md

执行真实验收：
1. 在 D 盘启动真实 Operator server，记录版本、端口和 health 响应。
2. 使用固定 Godot fixture 创建任务，记录 task_id、revision 和事件序号。
3. 验证扫描、计划、暂停、修改目标、恢复、审批拒绝、重新计划、审批通过、patch apply、验证和完成。
4. 在执行过程中验证 pause、resume、stop 的真实效果，不接受只变 UI 文本。
5. 重启服务，确认任务、checkpoint、pending approval、event cursor 和审计可恢复。
6. 用 Web、VSCode、IDEA 读取同一个任务，确认状态、revision 和事件一致。
7. 运行安全测试、协议契约测试、Rust 集成测试、Godot fixture、Playwright、VSCode pretest 和 IDEA wrapper build。
8. 采集命令、退出码、日志、diff、审计 id、截图和失败复现路径。

发布门禁：
- 任意 P0/P1 finding 未关闭，结论为 BLOCKED。
- 真实 Godot fixture 没跑，结论为 NOT VERIFIED，不是 PASS。
- 只有 mock E2E，结论为 UI VERIFIED，不是 REMOTE CLOSED LOOP。
- 有 ignored 测试时说明依赖和风险，不能写“全部通过”。
- 有未提交用户文件时明确 excluded files，不要擅自清理。
- 生成物未清理或协议文档过期时，发布结论最多为 READY FOR INTERNAL TEST。

最终报告固定包含：
【版本】commit、分支、时间
【环境】所有 D 盘工具路径和版本
【真实闭环】task_id、Godot fixture、事件和审计证据
【测试】命令、退出码、数量、ignored 和失败
【发现】P0/P1/P2/P3
【结论】PASS、READY FOR INTERNAL TEST、NOT VERIFIED 或 BLOCKED 四选一
【下一动作】只有一条最小可执行动作，不能写空泛路线图
```
