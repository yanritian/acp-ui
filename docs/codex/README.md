# Codex Planning Documents

> 更新时间：2026-07-08  
> 用途：给项目负责人、Qwen3.7 Plus、Claude Code 和后续执行 Agent 使用。

这个目录不是普通备忘录，而是 Hermes Game Operator 的执行依据。后续做游戏开发 Agent、VSCode 插件、IDEA 插件、Skill/MCP/Hook、记忆面板和进度面板时，优先读这里。

## 必读顺序

1. [Hermes Game Operator 总纲规划](./2026-07-08-hermes-game-operator-master-plan.md)

   先看产品方向：为什么先做游戏开发、Operator 面板是什么、Godot MVP 做到什么程度、后续怎么扩到多 IDE 和多行业。

2. [项目修整与 Agent 架构治理方案](./2026-07-08-project-restructure-and-agent-governance.md)

   再看工程治理：这个项目为什么必须先修整、怎么修整、哪些目录要收束、Agent 的状态机/事件/审批/记忆/工具边界怎么设计。

3. [Qwen3.7 Plus + Claude Code 执行交接说明](./2026-07-08-qwen-claude-execution-brief.md)

   最后看执行：Qwen 负责拆产品和测试，Claude Code 负责修构建、落代码、接入真实 Godot MVP。

## 核心判断

这个项目必须修整，而且要先修整再扩功能。

原因很简单：这是 Agent 项目，不是普通页面项目。普通 App 混乱一点还可以靠人脑兜底；Agent 项目如果边界、状态、权限、记忆、工具和测试不清楚，最后会变成一个会点击按钮的假自动化，而不是可信执行者。

第一阶段不要再铺 Unity、Ren'Py、Unreal、运营、企业、视频、漫画。先把 Godot 单任务闭环做成：

```text
选择真实 Godot 项目
  -> Agent 分析项目
  -> 生成计划
  -> 用户审批
  -> Agent 改文件
  -> 展示 diff/进度/记忆
  -> 可暂停/可停止/可改方向
  -> 产出总结
```

这个闭环成立后，再扩 VSCode、IDEA、其他引擎和其他行业。

