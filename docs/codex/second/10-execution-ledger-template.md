# 执行台账模板

每次 Claude Code + Qwen 执行都复制本模板或在本文件下新增一条记录。没有台账的“完成”不进入验收。

## 1. 运行摘要

```text
日期：YYYY-MM-DD
执行者：Claude Code + Qwen
仓库：D:/dingsun/acp-ui
分支：
起始 HEAD：
结束 HEAD：
工作包：WP-XX
目标：
是否修改代码：yes/no
是否修改文档：yes/no
是否产生提交：yes/no
```

## 2. 环境证据

| 工具 | 路径 | 版本 | 结果 |
|---|---|---|---|
| Node | `D:/.../node.exe` |  |  |
| npm | `D:/.../npm.cmd` |  |  |
| Rust/Cargo | `D:/Rust/...` |  |  |
| Java | `D:/Android/Android Studio/jbr` |  |  |
| Gradle | `D:/.../gradle.bat` 或 Wrapper |  |  |
| Godot | `D:/.../godot.exe` |  |  |
| Hermes | `D:/.../hermes` |  |  |

## 3. 修改清单

| 文件 | 行为变化 | 风险 | 回滚方式 |
|---|---|---:|---|
|  |  | low/medium/high |  |

## 4. 命令证据

```text
命令：
工作目录：
开始时间：
结束时间：
退出码：
关键输出：
日志路径：
```

重复填写每一条命令，不能只写“测试通过”。

## 5. 任务闭环证据

```text
task_id：
初始 revision：
最后 revision：
事件范围：
暂停事件：
修改事件：
审批 id：
补丁 hash：
验证命令：
验证退出码：
checkpoint：
重启恢复结果：
```

## 6. 风险与未完成项

- 已知失败：
- 被忽略测试及原因：
- 未验证的平台：
- 依赖外部环境：
- 不应被自动修复的用户文件：
- 下一条最小可执行动作：

## 7. 提交记录

```text
commit：
message：
included files：
excluded user files：
verification after commit：
```

提交前最后执行：

```powershell
git status --short
git diff --check
git diff --stat
git diff --name-only --cached
```
