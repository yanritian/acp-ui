# Godot 游戏领域闭环提示词

```text
你负责 WP-07。目标是让 Agent 能在固定 D 盘 Godot fixture 中完成一次可审阅、可审批、可回滚、可验证的真实游戏开发任务。

先读取：
- docs/codex/second/05-game-domain-godot.md
- docs/codex/second/03-target-architecture.md
- docs/codex/test-godot-project.md
- src-tauri/src/operator/
- 当前 Godot/Hermes 相关代码、Skill、MCP、Hook 和测试

环境规则：
1. 发现 D 盘 godot.exe、Hermes CLI、Java、Node 和 Cargo 路径；不要使用 C 盘工具替代。
2. 如果 Godot/Hermes 缺失，输出精确缺失路径和命令，不得把测试自动标绿。
3. fixture 必须位于 D:/dingsun/acp-ui 或明确的 D 盘测试目录，并且可重建。

实现顺序：
1. 写失败测试证明 project.godot 识别、项目边界和忽略目录规则。
2. 写失败测试证明目标文件扫描、计划文件列表、风险和验证命令都结构化返回。
3. 实现 Godot project inspection，记录项目版本、主场景、输入映射和规则文件来源。
4. 为最小目标准备基线，例如给玩家角色增加二段跳；先保存基线验证结果。
5. 生成结构化 patch proposal，包含 old hash、new hash、diff、风险和验证命令。
6. 无审批时确认 patch 不会被应用；拒绝时确认文件 hash 不变。
7. 批准后应用补丁，生成 backup、审计和 checkpoint。
8. 执行 godot headless 验证，保存退出码、stdout/stderr、日志路径和结果摘要。
9. 人为制造一个验证失败，确认任务进入 failed 并保留恢复信息。
10. 修复后重新执行，确认最终事件序列和记忆摘要可以追溯到真实工具输出。

安全要求：
- .godot、target、build 和临时缓存不能成为项目事实记忆。
- 场景/资源优先结构化解析，不使用脆弱正则覆盖未知内容。
- 外部修改导致 hash 不匹配时要求人工确认。
- 启动 Godot、删除文件、批量改名和项目设置变更必须经过 high/critical 审批。

输出：fixture 路径、真实命令、task_id、完整事件序列、补丁 diff、审批记录、验证退出码、失败复现、恢复证据和 commit id。
```
