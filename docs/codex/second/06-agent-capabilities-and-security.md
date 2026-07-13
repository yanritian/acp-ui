# Agent 能力、审批与远程安全

## 1. 能力注册模型

Skill、MCP 和 Hook 都是受治理的能力，不是 Agent 随意执行的脚本。

每个能力声明：

```json
{
  "id": "godot.project.inspect",
  "version": "1.0.0",
  "kind": "skill",
  "input_schema": "schemas/operator/godot-project-inspect.input.json",
  "output_schema": "schemas/operator/godot-project-inspect.output.json",
  "risk": "low",
  "requires_approval": false,
  "supports_cancel": true,
  "timeout_seconds": 30,
  "allowed_domains": ["game.godot"]
}
```

注册表必须校验版本、来源、签名或可信目录、输入 schema、权限和兼容的领域包。未知能力不能被调用。

## 2. 风险等级

- `low`：只读项目文件、读取状态、生成摘要。
- `medium`：运行受限测试、生成补丁、修改单个允许文件。
- `high`：应用补丁、修改场景/项目设置、启动外部进程。
- `critical`：删除、批量重命名、访问边界外路径、网络外传、修改凭据。

默认策略：low 自动执行；medium 生成 proposal 并允许团队策略自动批准；high 必须人工审批；critical 必须人工审批和二次确认。策略可以收紧，不能由 Agent 放宽。

## 3. Hook 生命周期

推荐 Hook：

```text
before_task_created
after_task_created
before_plan_published
before_tool_call
after_tool_call
before_patch_apply
after_patch_apply
before_validation
after_validation
before_task_resume
after_task_completed
on_task_failed
```

Hook 必须有超时、最大输出、失败策略和审计。安全 Hook 失败时默认阻断高风险动作；信息型 Hook 失败可以继续，但必须记录警告。

## 4. Sandbox

Sandbox 至少控制：

- 允许的项目根目录和临时目录。
- 允许的命令、参数模式和环境变量。
- 网络访问开关、域名 allowlist 和上传大小。
- CPU、内存、进程数、运行时间和输出大小。
- 可读、可写、不可访问的路径集合。
- 子进程树回收和取消信号。

任何路径都必须经过 canonicalize 后检查边界，不能只做字符串前缀判断。Windows 路径、UNC 路径、符号链接、大小写和短路径都要有测试。

## 5. 远程认证与授权

远程 API 默认只绑定本机回环地址；显式开启远程时必须配置：

1. TLS 或可信反向代理。
2. 短期 token、撤销机制和 token scope。
3. 角色：viewer、operator、approver、admin。
4. 任务级和项目级资源授权。
5. 每个写请求的 request id、actor id、来源和幂等键。
6. 速率限制、body 大小限制、并发任务上限和空闲超时。

角色权限：

| 动作 | viewer | operator | approver | admin |
|---|---:|---:|---:|---:|
| 查看任务/事件 | yes | yes | yes | yes |
| 创建任务 | no | yes | yes | yes |
| 暂停/恢复/停止 | no | yes | yes | yes |
| 批准高风险动作 | no | no | yes | yes |
| 修改安全策略 | no | no | no | yes |
| 管理 token/能力 | no | no | no | yes |

## 6. 审计要求

审计记录不可由普通客户端删除，至少包含 actor、task、action、target、decision、request id、before hash、after hash、结果、时间和错误码。UI 的“审批成功”只有在服务端返回持久化审计 id 后才能显示成功。

## 7. 威胁模型

必须测试：路径穿越、符号链接逃逸、命令注入、环境变量泄露、token 日志泄露、重放审批、重复 apply、revision 冲突、恶意 Skill、MCP 返回超大输出、远程未授权控制、客户端伪造状态和任务跨租户读取。

安全测试失败时，不能通过放宽校验来让测试变绿；应修复边界并保留回归测试。
