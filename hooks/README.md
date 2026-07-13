# Hook 能力目录

Hook 是受治理的生命周期扩展。注册时必须使用 D 盘工作区内的真实文件；执行器会限制超时、截断输出，并解析显式的 `__HOOK_SIGNAL:BLOCK__`、`__HOOK_ABORT:` 信号。

示例脚本：

- `pre-tool-use.cmd`：工具执行前的放行 Hook。
- `post-tool-use.cmd`：工具成功后的记录 Hook。
- `post-tool-use-failure.cmd`：工具失败后的告警 Hook。

脚本不应自行修改项目文件，实际修改必须经过 Operator 的结构化补丁审批。
