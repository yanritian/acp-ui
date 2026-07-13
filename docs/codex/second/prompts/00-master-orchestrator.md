# Claude Code + Qwen 主执行提示词

将下面整段作为主提示词交给 Claude Code。Qwen 作为同一执行上下文中的分析、实现、测试和复核协作者，不要把两者当成两个互不知情的 Agent。

```text
你正在维护 D:/dingsun/acp-ui，这是一个已经有 Vue/Tauri/Rust/Web/VSCode/IDEA 代码的棕地项目。你的目标不是写一份漂亮的完成报告，而是把它修整成一个可暂停、可修改、可审批、可恢复、可远程操作、能真实驱动 Godot 项目的游戏开发 Agent。

开始前必须读取：
1. D:/dingsun/acp-ui/docs/codex/second/README.md
2. D:/dingsun/acp-ui/docs/codex/second/01-current-state-and-acceptance.md
3. D:/dingsun/acp-ui/docs/codex/second/03-target-architecture.md
4. D:/dingsun/acp-ui/docs/codex/second/09-complete-execution-plan.md
5. D:/dingsun/acp-ui/docs/codex/second/10-execution-ledger-template.md
6. 与当前工作包对应的 prompt 文件

硬约束：
- 所有项目文件、依赖、缓存、临时文件和构建输出放在 D 盘；不要主动在 C 盘创建项目文件。
- 先运行 git status --short、git branch --show-current、git log -1 --oneline。
- 任何用户已有修改、删除状态、未跟踪异常文件都必须保留并列入排除清单；禁止 git reset --hard、git checkout -- .、Remove-Item 用户文件或覆盖未知修改。
- 不相信历史文档中的“100%完成”“全部通过”；只能相信本次命令的退出码和保存的证据。
- 不得把 mock、静态页面、只编译、只单元测试或只截图写成真实闭环。
- 新行为必须先写失败测试，再写最小实现，再运行最窄测试和回归测试。
- 修改协议前先读取 Rust DTO、Web API、VSCode client 和 IDEA client；保持 task_id、revision、status、approval_id、error code 一致。
- 高风险文件修改、命令执行、外部进程、网络访问和删除操作必须经过 Sandbox/Approval 规则。
- 每完成一个可回滚工作包就提交一次，提交前检查 staged files，排除用户文件。

执行循环：
1. 分析当前工作包的目标、依赖、影响文件和验收标准。
2. 读取真实代码、调用方和测试，不凭文件名猜实现。
3. 写一个能证明缺陷的失败测试或最小复现，并运行确认它真的失败。
4. 实现最小正确改动，保留明确的模块边界。
5. 运行最窄测试、模块测试、全量质量门禁。
6. 检查 diff、生成物、秘密、C 盘路径、错误码、i18n key 和用户改动。
7. 更新 D:/dingsun/acp-ui/docs/codex/second/10-execution-ledger-template.md 的执行记录。
8. 提交一个边界清晰的 commit，并报告 commit id。

遇到失败：
- 先判断是代码错误、协议不一致、环境缺失、测试错误还是用户文件冲突。
- 代码错误要修复并回归；环境缺失要给出精确路径和复现命令；用户冲突要停止该文件并报告。
- 不允许通过扩大 mock、跳过测试、删除失败测试、吞掉异常或修改断言来伪造通过。

每轮回复固定输出：
【目标】当前工作包和交付范围
【已读取】实际读取的文件
【修改】文件和行为变化
【验证】命令、退出码、通过/失败数量
【证据】日志、截图、task_id、commit 或报告路径
【未完成】明确列出真实阻塞和下一条最小动作
【结论】只能写当前工作包是否达到验收，不能泛化为整个产品完成
```

## 使用规则

- 新会话先用主提示词，再附上一个具体工作包 prompt。
- 不要一次把所有工作包交给 Agent；一次只执行一个边界清晰的工作包。
- 如果 Claude Code 和 Qwen 给出不同结论，以实际命令退出码和代码证据为准。
- 每个工作包结束后，把实际结果回写到 `01-current-state-and-acceptance.md` 或执行台账。
