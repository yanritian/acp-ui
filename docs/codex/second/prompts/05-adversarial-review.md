# 对抗式复核提示词

```text
你不是来表扬当前实现的。你负责对当前工作包做一次 P0/P1/P2/P3 代码审查，重点寻找会让“可控游戏开发 Agent”在真实环境中失效的问题。

先读取：
- docs/codex/second/01-current-state-and-acceptance.md
- docs/codex/second/03-target-architecture.md
- docs/codex/second/06-agent-capabilities-and-security.md
- 当前 git diff 和所有受影响测试

按以下顺序审查：
1. 数据一致性：多客户端写入、revision、事件 sequence、重复请求和重启恢复。
2. 安全性：路径穿越、符号链接、命令注入、token、网络外传、越权、重放审批。
3. 可控性：pause 是否真的暂停、stop 是否回收子进程、apply 是否可回滚、拒绝是否不改文件。
4. 协议：Rust、Web、VSCode、IDEA 的字段、错误码、默认端口、approval task_id 是否一致。
5. 游戏领域：project.godot 识别、Godot 版本、结构化 scene/resource 编辑、headless 验证和日志证据。
6. i18n/UX：硬编码文案、缺失 key、懒加载时序、长文本布局、键盘和 aria 状态。
7. 测试真实性：mock 是否掩盖错误、ignored 是否合理、E2E 是否真的调用后端、命令是否使用 D 盘。
8. 仓库卫生：生成物、秘密、C 盘路径、异常用户文件、无关改动和过期完成文档。

每个发现必须包含：
- 严重级别 P0/P1/P2/P3
- 绝对文件路径和行号
- 触发条件
- 实际后果
- 最小修复方案
- 应新增的回归测试

发现问题后先写失败测试，再修复，再重新运行相关测试。禁止只写审查报告不修复明显问题；无法修复时必须给出阻塞原因。

最后输出：按严重级别排序的 findings、已修复项、验证命令和退出码、剩余风险。不要使用“看起来没问题”“应该可以”“100%完成”。
```
