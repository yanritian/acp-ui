# 执行交接文档 - 2026-07-10

## 当前状态

### 已完成的工作

1. **DSK-001: 审批按钮可达性** ✅
   - 建立了 21 个可访问性测试
   - 所有测试通过
   - CSS 已包含必要的可访问性特性

2. **I18N-001: Game Operator 国际化** ✅ (部分完成)
   - 在 `types.ts` 中添加了 gameOperator、operatorStatus、approvalDecision、operatorEvent、operatorError 和 a11y namespace
   - 在 zh-CN.ts 和 en-US.ts 中添加了相应的翻译
   - 更新了 ApprovalDrawer.vue 组件，使用 vue-i18n
   - 添加了 aria-label 属性以提高可访问性
   - **已知问题**: 一个测试失败（GameOperatorView.test.ts:219），需要进一步调试

### 当前 git 状态

- 分支: `cleanup/project-snapshot-2026-06-25`
- 最新提交: 国际化支持
- 工作树: 干净（除了未提交的测试失败）

### 下一个最高风险合同

根据蓝图第 25 节和第 26 节，下一个最高风险合同是：

3. **API-001: backend event/error 标准化** (未开始)
   - 需要将后端 event/error 统一为 stable code + message_key + args
   - 当前后端仍有渲染后的英文字符串
   - 需要更新 MessageSchema 中的 operatorEvent 和 operatorError
   - 需要更新后端代码以返回 code/key/args 而不是渲染后的字符串

4. **PLAT-001: VSCode 实际客户端** (未开始)
   - 仓库中没有实际 VSCode 扩展源码
   - 需要创建完整的 VSCode Game Operator 客户端
   - 需要实现连接设置、任务管理、审批、diff 等功能

5. **PLAT-002: IDEA 实际客户端** (未开始)
   - 仓库中没有实际 IDEA 插件源码
   - 需要创建完整的 IDEA Game Operator 客户端
   - 需要使用 Kotlin 和 IntelliJ Platform SDK

6. **SEC-001: OIDC/RBAC/TLS 安全** (未开始)
   - 当前只有 loopback/LAN 基础安全
   - 需要实现 OIDC/OAuth2 登录
   - 需要实现 RBAC 和 project scope
   - 需要实现 TLS 和 HTTPS

## 下一步工作建议

### 立即优先项

1. **修复失败的测试** (P0)
   - 调试 GameOperatorView.test.ts:219 失败原因
   - 确保组件正确渲染远程创建的任务

2. **API-001: backend event/error 标准化** (P1)
   - 更新后端代码以返回 code/key/args
   - 更新前端代码以使用新的 API 格式
   - 添加相应的测试

3. **继续国际化工作** (P1)
   - 在其他 9 个 locale 文件中添加 gameOperator 翻译
   - 更新其他 Game Operator 组件（OperatorControlBar、PlanPanel、ProgressTimeline）使用 i18n
   - 把固定 zh-CN 时间格式改为跟随当前 locale 的 Intl.DateTimeFormat

### 中期目标

4. **PLAT-001: VSCode 客户端** (P2)
   - 创建 VSCode 扩展项目结构
   - 实现基本功能（连接、任务管理、审批）
   - 添加测试

5. **PLAT-002: IDEA 客户端** (P2)
   - 创建 IDEA 插件项目结构
   - 实现基本功能
   - 添加测试

### 长期目标

6. **SEC-001: 安全增强** (P3)
   - 实现 OIDC/RBAC/TLS
   - 添加安全测试

## 产品不变量提醒

在继续工作时，必须始终遵守以下产品不变量：

1. **INV-001**: 唯一事实源 - 任务状态由 Operator 后端持有
2. **INV-002**: 两级审批 - plan.generate_patch != patch.apply
3. **INV-003**: 模型没有最终写权 - 真实写入由 Operator 执行
4. **INV-004**: 模型输出不可信 - 必须经过 schema 校验
5. **INV-005**: 操作员随时可介入 - 支持 pause/resume/stop/redirect
6. **INV-006**: 验证命令可信 - 只能执行固定验证模板
7. **INV-007**: 失败不伪装成功 - 明确失败状态
8. **INV-008**: 协议值不翻译 - 机器值保持稳定
9. **INV-009**: 跨平台同义 - 所有平台使用同一产品语义
10. **INV-010**: 语言不改变授权 - 切换语言不影响权限

## D 盘约束

所有本地操作必须在 D 盘：
- TEMP/TMP
- Node/Rust cache
- SQLite
- WebView2 data
- 日志、截图、构建产物

## 工作树安全

- 不得 reset、checkout、clean 或回退来源不明的修改
- 只修改与当前验收直接相关的文件
- 发现来源不明改动时保留并与之协作

## 测试命令

```powershell
# 前端测试
$node = 'D:\WpSystem\S-1-5-21-3926364600-750180645-3408191882-500\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\OpenAI\Codex\bin\node.exe'
& $node node_modules\vitest\vitest.mjs run

# TypeScript 检查
& $node node_modules\vue-tsc\bin\vue-tsc.js --noEmit

# Rust 测试
$env:TEMP='D:\dingsun\acp-ui\.tmp-tests'
$env:TMP=$env:TEMP
$env:CARGO_HOME='D:\Rust\.cargo'
$env:RUSTUP_HOME='D:\Rust\.rustup'
$env:XDG_CACHE_HOME='D:\dingsun\acp-ui\.cache'
$env:RUSTFLAGS='-C force-unwind-tables'
& 'D:\Rust\.cargo\bin\cargo.exe' test --manifest-path 'D:\dingsun\acp-ui\src-tauri\Cargo.toml' --lib -- --test-threads=1
```

---

**交接时间**: 2026-07-10 20:55
**交接人**: Claude Code (Qwen 3.7 Plus)
**接收人**: 下一个执行 Agent