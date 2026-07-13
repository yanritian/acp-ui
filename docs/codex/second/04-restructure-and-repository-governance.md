# 仓库修整与 Agent 治理

## 1. 修整目标

这不是一次“把所有文件移动到漂亮目录”的重构，而是建立可持续的边界：协议只有一个来源，任务状态只有一个事实源，客户端只做适配，领域工具不污染通用运行时，生成物不进入源码提交。

## 2. 修整顺序

执行时必须按以下顺序，不能先大规模重命名：

1. 记录当前 `git status`、当前分支、HEAD、测试基线和异常文件。
2. 统计源码、测试、构建产物、缓存、截图、驱动和文档的归属。
3. 建立 `schemas/operator`，先复制并验证现有 DTO，再迁移消费者。
4. 将客户端重复的 API URL、状态字符串和错误处理迁移到协议适配层。
5. 将 Rust 中的任务状态、审批、补丁、事件和恢复逻辑拆成明确模块。
6. 迁移测试到 `tests/contract`、`tests/e2e`、模块旁单元测试三类。
7. 删除已确认的生成物，补充 `.gitignore`，单独提交仓库清理。
8. 最后才做命名、文件移动和历史文档归档。

## 3. 生成物治理

下列内容默认不能提交：

```text
node_modules/
dist/
dist-web/
target/
build/
.gradle/
.playwright-browsers/
.tmp-tests/playwright_*/
.artifacts/**/appdata/
.artifacts/**/drivers/
.artifacts/**/target/
```

`.artifacts` 中如果有真正需要长期保留的验收证据，只保留压缩后的报告、截图索引和版本信息，不保留浏览器缓存、驱动压缩包、Cargo target 和用户目录镜像。

清理命令必须先生成清单：

```powershell
git ls-files .artifacts > D:\dingsun\acp-ui\.tmp-tests\artifacts-tracked-before.txt
Get-ChildItem .artifacts -Recurse -File | Measure-Object Length -Sum
```

执行者确认清单后，使用单独提交：

```powershell
git add .gitignore .artifacts
git commit -m "chore: remove generated acceptance artifacts"
```

不能使用 `git reset --hard`、`git checkout -- .` 或任何会覆盖用户修改的清理方式。

## 4. Agent 修改规则

### 修改前

- 读取目标文件、调用方、相关测试和协议定义。
- 说明行为变化、兼容影响和回滚方法。
- 为新行为准备失败测试或最小复现。

### 修改中

- 不跨越控制面、执行面、能力面和领域面的边界直接调用内部实现。
- 不在组件中拼接 URL、猜测状态或写持久化业务逻辑。
- 不在工具中直接执行未经过 Sandbox/Approval 的高风险动作。
- 一个提交只包含一个可回滚的逻辑单元。

### 修改后

- 运行最窄测试，再运行受影响模块测试，再运行全量质量门禁。
- 检查 diff、未跟踪文件、秘密、绝对路径和生成物。
- 在 `10-execution-ledger-template.md` 记录命令和退出码。

## 5. 错误码治理

错误码使用稳定机器值，文案使用 locale key：

```text
TASK_NOT_FOUND
REVISION_CONFLICT
INVALID_TASK_STATE
APPROVAL_REQUIRED
APPROVAL_NOT_FOUND
PATCH_HASH_MISMATCH
PROJECT_BOUNDARY_VIOLATION
CAPABILITY_NOT_ALLOWED
REMOTE_UNAUTHORIZED
REMOTE_FORBIDDEN
REMOTE_RATE_LIMITED
EXECUTION_CANCELLED
RECOVERY_REQUIRED
```

客户端不能根据英文错误信息做分支判断。新增错误码必须有 Rust 测试、前端归一化测试和至少一个客户端处理路径。

## 6. 提交策略

推荐提交顺序：

```text
chore: establish operator contract baseline
refactor: isolate operator control plane
feat: add resumable task execution
feat: add patch approval and rollback
feat: add godot domain capabilities
feat: connect clients to versioned operator api
feat: add remote auth and audit enforcement
test: add real game operator acceptance flow
chore: clean generated artifacts and update evidence
docs: refresh second execution package
```

每个提交都必须能单独解释“为什么改、如何验证、还缺什么”。
