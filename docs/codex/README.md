# Codex Planning Documents

> 更新时间：2026-07-10  
> 用途：给项目负责人、使用 Qwen 3.7 Plus 模型的 Claude Code 和后续执行 Agent 使用。

这个目录不是普通备忘录，而是 Hermes Game Operator 的执行依据。后续做游戏开发 Agent、VSCode 插件、IDEA 插件、Skill/MCP/Hook、记忆面板和进度面板时，优先读这里。

## 当前交接

从 2026-07-10 起，后续执行先读以下四份：

1. [Hermes Game Operator 完整产品与技术蓝图](./2026-07-10-hermes-game-operator-complete-product-blueprint.md)

   不按阶段切割的完整目标产品规范。一次性定义产品不变量、完整用户旅程、状态机、平台合同、国际化、Agent 扩展、安全、可靠性、发布和最终完成定义。

2. [游戏开发 Agent 当前交接与执行状态](./2026-07-10-game-agent-handoff-and-execution-plan.md)

   当前真实状态、桌面验收证据、未解决问题和 D 盘执行环境。它用于描述“现在在哪里”，不再用于定义最终产品边界。

3. [Claude Code（Qwen 3.7 Plus 模型）一体化执行提示词](./2026-07-10-qwen-claude-execution-prompts.md)

   可直接发给同一个 Claude Code 会话的规划、继续实施、冷启动自审、修复及后续阶段提示词，不需要人工在两个角色之间转发。

4. [Game Agent 执行与验收台账模板](./2026-07-10-execution-ledger-template.md)

   每轮必须填写的 acceptance IDs、测试结果、真实证据、安全不变量和阻塞记录。

完整产品蓝图定义“最终必须是什么”，当前交接定义“现在哪里”，执行提示词定义“同一个 Agent 如何工作”。三者职责不同，不得用当前缺口清单缩小完整产品边界。早期文档仍用于理解设计历史。

## 历史必读顺序

1. [Hermes Game Operator 总纲规划](./2026-07-08-hermes-game-operator-master-plan.md)

   先看产品方向：为什么先做游戏开发、Operator 面板是什么、Godot MVP 做到什么程度、后续怎么扩到多 IDE 和多行业。

2. [项目修整与 Agent 架构治理方案](./2026-07-08-project-restructure-and-agent-governance.md)

   再看工程治理：这个项目为什么必须先修整、怎么修整、哪些目录要收束、Agent 的状态机/事件/审批/记忆/工具边界怎么设计。

3. [Qwen3.7 Plus + Claude Code 执行交接说明](./2026-07-08-qwen-claude-execution-brief.md)

   这是早期按两个角色编写的历史说明。当前实际运行方式是 Claude Code 使用 Qwen 3.7 Plus 模型，最新一体化提示词优先。

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
