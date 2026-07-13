# 仓库盘点与修整提示词

```text
你负责 WP-00 和 WP-01：盘点 D:/dingsun/acp-ui，并安全清理已确认的生成物。

先读取：
- docs/codex/second/README.md
- docs/codex/second/01-current-state-and-acceptance.md
- docs/codex/second/04-restructure-and-repository-governance.md
- docs/codex/second/10-execution-ledger-template.md

执行要求：
1. 输出 git status --short、分支、HEAD、git diff --stat、git ls-files .artifacts 的数量和大小。
2. 列出以下类别：源码、测试、文档、截图、浏览器缓存、驱动、Rust target、Gradle build、node_modules、临时日志。
3. 对每个待删除目录给出绝对路径、文件数量、总大小和是否被 Git 跟踪。
4. 默认保留用户文件、异常 FAQ、.claude/scheduled_tasks.lock 的现状，不要恢复、删除或改名。
5. 只清理已确认的生成物，使用 D 盘 PowerShell，并在删除前检查目标绝对路径确实位于 D:/dingsun/acp-ui 下。
6. 更新 .gitignore，不能用宽泛规则隐藏源码或用户文件。
7. 运行 git diff --check、前端测试、Rust lib 测试和受影响客户端构建。
8. 清理提交必须独立于功能提交，提交信息使用 chore: remove generated acceptance artifacts。

禁止：
- git reset --hard
- git checkout -- .
- 删除整个 .tmp-tests 而不逐项核对
- 把验收截图、日志和缓存混为一类
- 用“仓库干净”代替真实清单

输出：盘点表、删除清单、保留清单、修改文件、每条命令退出码、提交 id 和未解决风险。
```
