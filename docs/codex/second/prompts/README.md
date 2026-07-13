# 提示词索引

## 推荐调用顺序

1. `00-master-orchestrator.md`：每次新会话必先发送，建立 D 盘、保护用户改动、TDD、证据和提交纪律。
2. `01-recon-and-cleanup.md`：仓库盘点和生成物清理。
3. `02-runtime-and-protocol.md`：任务状态机、协议、审批、补丁、checkpoint 和恢复。
4. `03-game-godot.md`：Godot fixture 和真实游戏开发闭环。
5. `04-clients-and-i18n.md`：Web/Tauri/VSCode/IDEA 一致性和 13 种语言。
6. `05-adversarial-review.md`：每个工作包完成后做对抗式复核并修复发现。
7. `06-acceptance-and-release.md`：真实验收、发布门禁和最终结论。

## 传给 Agent 的最小格式

```text
读取 D:/dingsun/acp-ui/docs/codex/second/prompts/00-master-orchestrator.md，作为本次会话硬约束。
然后读取 D:/dingsun/acp-ui/docs/codex/second/prompts/<具体文件>.md。
本次只执行 <WP 编号>，不要扩大范围。
完成后按主提示词的固定格式返回，并更新执行台账。
```

## 何时停止

出现以下任一情况必须停止当前工作包并报告：

- 发现用户修改会被覆盖。
- 需要 C 盘工具或路径才能继续。
- 真实 Godot/Hermes 环境缺失。
- 协议与现有客户端冲突且没有兼容方案。
- 安全边界、审批或回滚语义不清楚。
- 测试失败但无法判断是代码问题还是环境问题。
