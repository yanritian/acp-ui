# Hermes Game Operator 第二轮执行包

> 文档版本：2026-07-13
> 适用项目：`D:/dingsun/acp-ui`
> 目标读者：项目负责人、Claude Code、Qwen、后端/前端/客户端/QA 执行者

## 这套文档解决什么问题

本目录不是宣传材料，也不是“已经完成”的总结。它是一份可以让 Claude Code + Qwen 在同一个仓库里持续执行、验证、复盘和交接的工程执行包。

产品目标是做一个以游戏开发为首个领域的 Agent 工作台：操作员可以启动任务、随时暂停、修改目标或计划、审批高风险动作、恢复执行，并持续看到进度、事件、记忆和最终走向；Agent 是真正执行者，可以通过 Skill、MCP、Hook 和领域工具完成工作；Web、Tauri、VSCode、IDEA 与远程控制都必须遵守同一套协议。

## 阅读顺序

1. `01-current-state-and-acceptance.md`：先看真实现状和不可伪造的验收边界。
2. `02-product-contract.md`：确认第一版产品到底要交付什么。
3. `03-target-architecture.md`：确认模块边界、状态机、事件、记忆和远程协议。
4. `04-restructure-and-repository-governance.md`：按规则修整混乱仓库。
5. `05-game-domain-godot.md`：完成游戏开发首个领域包。
6. `06-agent-capabilities-and-security.md`：实现 Skill、MCP、Hook、审批和远程安全。
7. `07-clients-i18n-and-ux.md`：统一 Web、Tauri、VSCode、IDEA 和多语言体验。
8. `08-testing-and-acceptance.md`：按证据验收，不按口头汇报验收。
9. `09-complete-execution-plan.md`：唯一的执行清单；它是一个完整计划，不按“做完一阶段就宣布项目完成”推进。
10. `prompts/`：直接交给 Claude Code + Qwen 的提示词。
11. `10-execution-ledger-template.md`：每次执行必须留下的记录模板。

## 规则优先级

发生冲突时按以下顺序处理：

1. 当前代码、测试结果和后端实际协议。
2. `01-current-state-and-acceptance.md` 的事实记录。
3. `03-target-architecture.md` 的边界和契约。
4. `09-complete-execution-plan.md` 的任务清单。
5. 其他历史文档、完成报告和聊天记录。

历史文档中出现的“100% 完成”“全部平台已完成”等文字，除非有本目录规定的命令、日志、截图或录屏证据，否则只能视为待核验声明。

## 给执行 Agent 的硬约束

- 所有项目操作、缓存、依赖和构建产物优先放在 D 盘；不要主动在 C 盘创建项目文件。
- 开始前必须读取本目录 README、当前状态和执行计划。
- 修改前先读取目标文件和相邻测试，不能只看文件名猜实现。
- 新行为遵循 TDD：先写失败测试，再写最小实现，再运行回归测试。
- 每个工作单元都必须说明修改文件、验证命令、退出码和剩余风险。
- 不得删除、覆盖或回滚用户已有修改；不确定的异常文件必须隔离并报告。
- 不得把 mock、静态演示、只编译未运行、只 UI 测试或只单元测试写成“真实闭环完成”。
- 每次提交只包含一个可解释的边界；提交信息必须能反映行为变化。

## 下一轮的完成定义

只有同时满足以下条件，才可以对外写“游戏开发 Agent 首个可用版本完成”：

- Web/Tauri、VSCode、IDEA 使用同一份版本化协议。
- 真实后端进程能够创建任务、暂停、修改、恢复、审批、回滚和结束任务。
- Godot 测试项目能够完成一次真实的扫描、计划、补丁、验证和运行结果回传。
- Skill、MCP、Hook 至少各有一个真实执行案例和失败案例。
- 远程访问有认证、授权、审计、超时、取消和重放保护。
- 至少一个完整的 Playwright + Rust/HTTP + Godot 实机验收记录存在。
- 13 种 Web 语言没有新增硬编码英文；缺翻译时有明确 fallback 和检测。
- 所有构建、测试、版本和提交证据写入执行台账。
