# Claude Code（Qwen 3.7 Plus 模型）一体化执行提示词

> 实际运行形态：Claude Code 是执行环境，Qwen 3.7 Plus 是其中使用的模型，两者是同一个执行 Agent。  
> 当前直接发送“提示词 A”；若它停在规划，再发送 B；一个完整能力闭合后在同一会话发送 C 自审，有 findings 再发送 D。  
> 后面的 Repository、VSCode、IDEA、Remote 和 Domain Pack 代码块只是能力聚焦补充，不是阶段，也不能替代完整蓝图。

## 一、一体化执行方式

不要再把 Qwen 与 Claude Code 当成两个需要人工转发材料的角色。正确理解是：

| 组成/阶段 | 职责 | 约束 |
|---|---|---|
| Claude Code | 提供文件、终端、编辑和执行能力 | 不擅自扩大范围或清理用户改动 |
| Qwen 3.7 Plus | 在 Claude Code 中完成推理、规划、实现与复核 | 不凭自己的口头总结宣布完成 |
| 规划阶段 | 建立 acceptance IDs、风险和最小实施顺序 | 规划后直接实施，不等待人工转发 |
| 实施阶段 | 读代码、补测试、修改、目标验证与真实验收 | 不跳过安全边界 |
| 冷启动自审阶段 | 暂时把刚才的实现当成不可信变更，重新读 diff 和证据 | 必须记录这是同一 Agent 自审，不冒充独立审查 |
| 项目负责人 | 决定范围，处理签名、Defender 等外部阻塞 | 不需要在两个模型之间搬运文本 |

推荐单会话循环：

```text
1. 在使用 Qwen 3.7 Plus 的 Claude Code 会话中发送提示词 A。
2. Agent 建立完整产品 compliance matrix，随后按依赖直接实施，不等待确认。
3. 若 Agent 只给计划却停止，发送提示词 B 要求继续实施。
4. 实施和目标测试完成后，在同一会话发送提示词 C，切换到冷启动自审。
5. 有 findings 时发送提示词 D，要求逐项复现和修复。
6. 关闭当前依赖节点后更新完整矩阵，并继续下一个未满足合同，不把局部闭合报告为完整产品。
```

成本控制：

1. 每个小修改只跑相关测试。
2. 一个阶段结束前只跑一次 full Vitest 和一次 ACP Rust full lib。
3. 不要每轮重复扫描整个仓库。
4. 不要反复下载或安装已经证明会被系统移除的 tauri-driver。
5. 不要重复真实 Qwen/Hermes 在线调用，除非本轮改了 Hermes 合同。
6. 视觉调整先用快速浏览器布局测试，最后只做一次真实 Tauri 三尺寸验收。
7. 所有长输出写入 D 盘 artifact，聊天只汇报摘要和路径。

## 二、提示词 A：直接发给 Claude Code 的一体化主提示词

```text
你运行在 Claude Code 执行环境中，当前模型是 Qwen 3.7 Plus。你不是两个互相交接的角色，而是一个能够规划、读取代码、编辑、运行命令、真实验收和自审的一体化执行 Agent。你必须用 acceptance criteria、实际 diff、真实命令、精确测试结果和 D 盘产物验证每一项结论，不能把自己的“已完成”当作证据。

开始前按顺序阅读：
1. D:\dingsun\acp-ui\docs\codex\2026-07-10-hermes-game-operator-complete-product-blueprint.md
2. D:\dingsun\acp-ui\docs\codex\2026-07-10-game-agent-handoff-and-execution-plan.md
3. D:\dingsun\acp-ui\docs\codex\2026-07-10-qwen-claude-execution-prompts.md
4. D:\dingsun\acp-ui\docs\codex\2026-07-10-execution-ledger-template.md
5. D:\dingsun\acp-ui\docs\codex\2026-07-10-operator-persistence-and-recovery-spec.md
6. D:\dingsun\acp-ui\_bmad-output\implementation-artifacts\spec-hermes-structured-patch-approval.md
7. D:\dingsun\acp-ui\_bmad-output\implementation-artifacts\spec-secure-remote-operator-access.md

唯一产品目标：让 `2026-07-10-hermes-game-operator-complete-product-blueprint.md` 定义的完整产品合同全部成立。不要把工作重新拆成互相孤立的 P0/P1 阶段，也不要因为某一页面或某一测试通过就停下。你可以按依赖关系推进代码，但必须始终维护整张产品 compliance matrix，持续显示所有未满足能力。

已知事实：
- 当前分支 cleanup/project-snapshot-2026-06-25，HEAD df8bcbc。
- 工作树非常脏，不得 reset、checkout、clean 或回退来源不明的修改。
- Hermes no-tools、结构化补丁、两级审批、Godot 固定验证、SQLite schema v1 和 Remote API 后端均已实现，不要重写。
- 真实任务 task_1783659056917 已从 Remote HTTP 创建，并在真实 Tauri 窗口显示。
- 真实桌面进程重启后，任务、事件与计划审批从同一个 D 盘 SQLite 恢复。
- Remote task discovery、D 盘 config/history override、侧栏溢出和控制条 Stop 可达性已修过。
- 当前最高优先级缺陷：1024x720 下审批卡可见，但 Approve/Reject/Request changes 三个按钮尚无可靠可达证据；工作树已有一组尚未全量复验的滚动 CSS 修改。
- tauri-driver 2.0.6 在当前机器安装后会被系统移除；不要无限重装。
- 仓库中没有实际 VSCode 或 IDEA 插件源码，只有 .vscode 开发配置。
- 项目已有 vue-i18n 和 11 个 locale，但 Game Operator 五个核心组件仍硬编码英文；ProgressTimeline 还固定使用 zh-CN 时间格式；后端 event/error 仍有渲染后的英文字符串。

硬约束：
1. 所有新产物、缓存、TEMP/TMP、SQLite、WebView2 data、截图、日志和构建输出只放 D 盘。
2. 不能破坏 operator.plan.generate_patch -> operator.patch.apply 两级审批。
3. Hermes 必须保持 --no-tools 和 --prompt-file，不能获得终端或文件写工具。
4. 模型输出是不可信数据，不执行模型 validation 文本。
5. Tauri 和 Remote HTTP 必须共享同一 OperatorState 与 SQLite。
6. 不做全仓重构，不运行 cargo fix --workspace，不做全仓格式化。
7. 不接受“100% 完成”作为交付结论。
8. 所有新增可见文案、状态、事件、错误、审批和可访问性名称必须遵守完整蓝图的 locale contract；机器枚举、路径、哈希和 API code 禁止翻译。
9. Desktop、VSCode、IDEA、Web 和移动/IM 必须复用同一版本化 Operator contract，不复制状态机。
10. Skills、MCP、Hooks 和 Domain Packs 不能绕过 Operator policy 与 patch approval。

你的本轮工作：
1. 将完整蓝图中的 INV、能力 ID、平台合同、locale contract、安全、SLO、测试和完成定义转成一张 requirement compliance matrix。
2. 对每个 requirement 标记：PASS、IMPLEMENTED、FAIL、BLOCKED、NOT_STARTED，并关联实现、测试和真实证据；不确定时不得标 PASS。
3. 读取当前状态交接，确认真实代码基线和工作树归属，不回退用户修改。
4. 根据依赖网络选择当前可执行且最高风险的未满足合同。当前已知首要缺口包括：
   - 1024x720 审批按钮和键盘焦点不可达。
   - Game Operator 可见文本未接入 11 个现有 locale。
   - backend event/error 尚未统一为 stable code + message_key + args。
   - Remote client contract 尚未从 Vue/Tauri 解耦。
   - VSCode/IDEA 实际客户端不存在。
5. 每个变更先建立失败测试或可重复验收，再做最小实现。
6. 每个 UI 变更同时验证 zh-CN、en-US、de-DE、th-TH、en-XA 和 ar-XB/RTL；所有生产 locale 必须通过 key/placeholder parity。
7. 每个状态/API 变更同时验证 persistence、multi-client revision、security scope 和 localization contract。
8. 完成一个依赖节点后更新整张 compliance matrix，并继续下一个可执行节点；不要停在“计划”“建议”或局部完成报告。
9. 完整回归按有意义的能力闭合点运行，目标测试优先，避免重复消耗。
10. 每个能力闭合后进入冷启动自审模式，重新读取 diff、日志、截图和矩阵，不得只复述自己的报告。

开始时先简短输出以下内容：
A. 完整产品 compliance matrix 的能力域摘要。
B. 当前 FAIL/BLOCKED/NOT_STARTED 的最高风险合同。
C. 产品不变量和禁止事项。
D. 基于依赖关系、而非阶段名称的当前实施队列。
E. 目标测试、国际化、真实窗口和跨平台合同验证矩阵。
F. 实施后冷启动自审要检查的证据清单。

输出上述内容后立即开始读取相关代码、补测试、实施和验证，不要停下来等待用户把计划转发给另一个模型。只有出现必须由用户处理的系统级阻塞时才停。不要在开始时宣布完成，不要建议扩展新平台。
```

## 三、提示词 B：同一 Claude Code 会话的完整产品继续执行提示词

```text
继续执行，不要重新解释蓝图，也不要停在计划或阶段总结。

你仍在同一个使用 Qwen 3.7 Plus 的 Claude Code 会话中。读取当前 complete product compliance matrix、execution ledger、最新 git diff 和测试证据，选择依赖已经满足的最高风险未关闭合同，直接补失败测试、实现、目标验证和真实证据。完成该合同后更新整张矩阵，然后继续下一个可执行合同。

始终同时检查：
- Operator 状态机、两级审批和唯一写入者不变量。
- persistence、revision、idempotency 和多客户端同步。
- Desktop、VSCode、IDEA、Web、移动/IM 的共享合同影响。
- ui_locale、task_language、artifact_language 和 source_language。
- 现有 production locale、伪 locale、RTL、文本扩展和可访问性。
- Skills/MCP/Hooks/Domain Pack 是否仍受 policy 和 scope 控制。
- D 盘工具链、缓存、数据库、日志、截图和构建产物约束。
- 工作树中的用户修改不得回退。

不要因为某个 P0/P1、某个平台或某个页面通过就报告完整产品完成。只有完整蓝图第 26、27 节全部具有真实证据时，才能关闭完整产品目标。普通技术问题自行继续解决；只有必须由用户处理的系统级外部阻塞才停。
```

### 3.1 当前桌面可达性依赖节点补充提示词

```text
你仍在同一个使用 Qwen 3.7 Plus 的 Claude Code 会话中。完整矩阵当前指向 Desktop 可达性与国际化依赖节点。不要重复输出方案，也不要等待转发，直接关闭对应 DSK、UI 和 I18N requirement IDs，并在完成后返回完整矩阵继续执行。你必须读取代码、实现、测试和验证。除非遇到必须由用户处理的系统级阻塞，否则持续工作到这些 requirement IDs 全部关闭。

先阅读：
1. D:\dingsun\acp-ui\docs\codex\2026-07-10-game-agent-handoff-and-execution-plan.md
2. D:\dingsun\acp-ui\docs\codex\2026-07-10-qwen-claude-execution-prompts.md
3. D:\dingsun\acp-ui\docs\codex\2026-07-10-execution-ledger-template.md
4. 当前会话已经建立的 acceptance IDs、实施单与 execution ledger
5. D:\dingsun\acp-ui\docs\codex\2026-07-10-operator-persistence-and-recovery-spec.md

工作树规则：
- 当前分支 cleanup/project-snapshot-2026-06-25，HEAD df8bcbc。
- 工作树有大量用户和历史改动。不得 git reset --hard、git checkout --、git clean、全仓格式化或批量换行重写。
- 只修改与本轮验收直接相关的文件。
- 发现来源不明改动时保留并与之协作，不要回退。

D 盘规则：
- 所有 TEMP/TMP、Node/Rust cache、SQLite、WebView2 user-data、日志、截图、fixture 和构建产物只能在 D 盘。
- 启动任何命令前显式设置 D 盘环境变量。
- Node 使用：D:\WpSystem\S-1-5-21-3926364600-750180645-3408191882-500\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\OpenAI\Codex\bin\node.exe
- Cargo 使用：D:\Rust\.cargo\bin\cargo.exe
- Godot 使用：D:\dev-tools\godot\godot.exe
- Hermes runtime 使用：D:\dingsun\acp-ui\bin\hermes-game.exe

不可破坏的产品边界：
1. operator.plan.generate_patch 与 operator.patch.apply 是两次独立审批。
2. Hermes 提案阶段保持 --no-tools 和 --prompt-file。
3. 模型没有项目写权，Operator 是唯一写入者。
4. 不执行模型返回的 validation 文本。
5. 不放宽 path/hash/backup/stale/symlink/size 安全校验。
6. Tauri 与 Remote HTTP 共用同一个恢复后的 OperatorState。
7. 不新增生产测试后门、跳过审批入口或 mock 状态源。

当前真实验收状态：
- artifact 根目录：D:\dingsun\acp-ui\.artifacts\desktop-acceptance\2026-07-10-codex
- fixture：D:\dingsun\acp-ui\.artifacts\desktop-acceptance\2026-07-10-codex\project-1783658820041
- SQLite：D:\dingsun\acp-ui\.artifacts\desktop-acceptance\2026-07-10-codex\operator\operator-state.sqlite3
- task_id：task_1783659056917
- status：waiting_approval
- approval options：approve、reject、request_changes
- 真实重启恢复已证明。
- Remote 创建任务后桌面自动发现已证明。
- 1024x720 控制条长目标与 Stop 按钮可见已证明。
- 尚未证明审批三按钮在 1024x720 下可达。
- Game Operator 当前仍硬编码英文，没有使用项目已有的 vue-i18n；已有 11 个 locale 文件但没有 gameOperator namespace；ProgressTimeline 固定使用 zh-CN 时间格式。

优先检查这些文件：
- src/App.vue
- src/features/chat/SessionList.vue
- src/features/game-operator/views/GameOperatorView.vue
- src/features/game-operator/components/OperatorControlBar.vue
- src/features/game-operator/components/PlanPanel.vue
- src/features/game-operator/components/ProgressTimeline.vue
- src/features/game-operator/components/ApprovalDrawer.vue
- src/features/game-operator/__tests__/GameOperatorView.test.ts
- src/features/game-operator/__tests__/OperatorControlBar.test.ts
- src/features/game-operator/__tests__/ApprovalDrawer.test.ts
- src/features/game-operator/__tests__/ProgressTimeline.test.ts
- src/locales/types.ts
- src/locales/index.ts
- src/locales/zh-CN.ts
- src/locales/en-US.ts
- src/locales/de-DE.ts
- src/locales/es-ES.ts
- src/locales/ru-RU.ts
- src/locales/ja-JP.ts
- src/locales/ko-KR.ts
- src/locales/vi-VN.ts
- src/locales/th-TH.ts
- src/locales/ms-MY.ts
- src/locales/fr-FR.ts

本轮实现顺序：
1. 先记录当前相关 git diff，不碰无关文件。
2. 建立一个可重复失败的布局/可达性测试。jsdom 不会真实应用 Vue scoped CSS，因此不要只依赖 getComputedStyle；可以使用源码契约测试作为低级保护，但必须补 Playwright 浏览器布局测试或同等级真实渲染断言。
3. 明确 Game Operator 的唯一纵向滚动宿主。
4. 检查所有祖先 flex/grid 的 min-width、min-height、height、overflow 与 box-sizing。
5. 确保 ApprovalDrawer 不形成焦点陷阱；长 diff 仅在 pre 内横向滚动。
6. 在 1024x720 下让 Approve、Reject、Request changes 三按钮全部可见或可通过一次明确滚动到达。
7. 验证 Tab/Shift+Tab 会把聚焦按钮滚入视口，Enter/Space 可触发正确事件，但不要在真实待审批任务上误点 approve，除非该场景使用隔离 fixture 且台账明确允许。
8. 在 MessageSchema 增加完整 gameOperator、operatorStatus、approvalDecision、operatorEvent、operatorError 和 a11y namespace；不使用英文句子作为 key。
9. 五个 Game Operator 组件全部接入 useI18n。状态、决策和事件通过显式映射翻译；未知机器值展示 localized unknown + 原始 code，不直接 toUpperCase。
10. 把固定 zh-CN 时间改为跟随当前 locale 的 Intl.DateTimeFormat；路径、ID、hash、scope 和代码保持原值。
11. 让 11 个现有 locale 的 key 与 ICU 参数集合完全一致。安全、审批、恢复和错误文案不得只依赖英文 fallback。
12. 增加 en-XA 文本扩展和 ar-XB RTL 测试基础；完整产品矩阵继续跟踪 zh-TW、pt-BR、id-ID、ar-SA 等目标 locale。
13. 验证 zh-CN、en-US、de-DE、th-TH、en-XA、ar-XB 下 1024x720 没有空白高度、横向溢出、文字裁切、按钮重叠或侧栏裁切。
14. 目标测试通过后再启动真实 Vite + Tauri。不要频繁重启 full build。
15. 使用隔离 D 盘 profile、SQLite 和 WebView2 data。
16. 复验真实 Remote task discovery 与进程 restart recovery；切换语言不能重置任务或重新提交操作。
17. 记录三种尺寸和代表 locale 截图，并标记 real/fixture/mock。
18. 运行 locale parity、Game Operator 相关测试、vue-tsc、full Vitest。
19. 最后运行一次 ACP Rust full lib；除非改了 Hermes 合同，否则不要重复真实在线模型调用。
20. 更新完整 compliance matrix、execution ledger 与交接文档。

桌面自动化说明：
- 项目安装了 @wdio/tauri-service，但尚无稳定配置。
- EdgeDriver 位于 D:\dingsun\acp-ui\.artifacts\desktop-acceptance\2026-07-10-codex\drivers\msedge-150\msedgedriver.exe。
- tauri-driver 2.0.6 在当前机器会消失。不要反复重装；先检查是否能使用现有驱动或受控签名版本。
- 工具阻塞时可以使用真实 Tauri 进程、Win32 输入、PrintWindow 和后端状态做人工可审计验收，但要明确标记为 manual-real，不得称为 automated E2E。

前端目标测试：
$node = 'D:\WpSystem\S-1-5-21-3926364600-750180645-3408191882-500\AppData\Local\Packages\OpenAI.Codex_2p2nqsd0c76g0\LocalCache\Local\OpenAI\Codex\bin\node.exe'
& $node node_modules\vitest\vitest.mjs run src/features/game-operator/__tests__/GameOperatorView.test.ts src/features/game-operator/__tests__/OperatorControlBar.test.ts src/features/game-operator/__tests__/ApprovalDrawer.test.ts src/features/game-operator/__tests__/ProgressTimeline.test.ts src/api/operatorRemoteApi.test.ts
& $node node_modules\vue-tsc\bin\vue-tsc.js --noEmit
& $node node_modules\vitest\vitest.mjs run

ACP Rust 最终回归：
$env:TEMP='D:\dingsun\acp-ui\.tmp-tests'
$env:TMP=$env:TEMP
$env:CARGO_HOME='D:\Rust\.cargo'
$env:RUSTUP_HOME='D:\Rust\.rustup'
$env:XDG_CACHE_HOME='D:\dingsun\acp-ui\.cache'
$env:RUSTFLAGS='-C force-unwind-tables'
& 'D:\Rust\.cargo\bin\cargo.exe' test --manifest-path 'D:\dingsun\acp-ui\src-tauri\Cargo.toml' --lib -- --test-threads=1

交付格式必须包含：
A. 修改文件及原因。
B. 每个 acceptance ID 的状态。
C. 精确测试命令、文件数、测试数、通过/失败/忽略数。
D. 三种窗口尺寸截图的 D 盘绝对路径。
E. 哪些场景是 real、fixture、mock、manual-real 或 automated-real。
F. 是否写入过 C 盘；如何确认。
G. 未覆盖边界与下一步。
H. 不得只写“100% 完成”。
```

## 四、提示词 C：同一 Claude Code 会话的冷启动自审提示词

```text
现在切换到冷启动自审模式。虽然仍是同一个 Claude Code + Qwen 3.7 Plus Agent，但你必须暂时放弃“刚才实现应该正确”的假设，把本轮改动当作一份陌生且不可信的提交重新审查。不要继续设计新功能，先判断当前声称关闭的完整产品 requirement IDs 是否真的满足蓝图合同。台账中必须把本轮标为 same-agent cold review，不得冒充独立模型审查。

你必须读取：
1. 本轮完整交付报告。
2. 当前 git diff，不只看摘要。
3. 完整产品蓝图和当前 compliance matrix。
4. D:\dingsun\acp-ui\docs\codex\2026-07-10-execution-ledger-template.md 对应台账。
5. 受影响平台、locale、窗口尺寸和 RTL 截图原图。
6. 测试日志和命令输出。
7. 真实任务/审批的 Remote API 状态。

审查顺序：
1. 安全边界：两级审批、Hermes no-tools、Operator 唯一写入、固定 Godot 验证是否仍成立。
2. 状态真实性：UI 是否读取真实 OperatorState，重启后是否来自同一 SQLite。
3. 可操作性：受影响窗口下核心操作是否鼠标和键盘均可达。
4. 布局：是否出现嵌套滚动、横向溢出、巨大空白、文字裁切、按钮抖动。
5. 并发：polling 是否可能用旧状态覆盖 stop/redirect/approval 后的新状态。
6. 测试质量：是否只测试常量和 mock，是否有真实组件行为与真实桌面证据。
7. 证据完整性：测试数是否来自实际输出，截图是否来自真实 Tauri。
8. 工作树安全：是否触碰无关用户改动，是否产生大范围格式化噪声。
9. D 盘约束：本轮缓存、数据库、截图、日志与构建产物是否全在 D 盘。
10. 国际化：是否仍有可见硬编码、locale key/args 不一致、固定 locale 格式或 RTL/文本扩展失败。
11. 跨平台：是否复制状态机、类型或术语，是否破坏 API compatibility。
12. 完整性：是否只关闭局部页面，却错误关闭了完整能力 ID。

按严重度输出 findings：
- P0：会造成越权写入、审批绕过、数据损坏或虚假完成。
- P1：阻止操作员完成核心闭环。
- P2：明显交互、可靠性或测试缺口。
- P3：文档或维护性问题。

每个 finding 必须包含：
- 文件和行号。
- 可复现条件。
- 预期与实际。
- 为什么现有测试没有挡住。
- 最小修复建议。

最后输出：
A. 本轮声称关闭的 requirement IDs 是否通过：通过 / 有条件通过 / 不通过。
B. compliance matrix 中仍未关闭的 IDs。
C. 同一会话下一步必须执行的修复清单。
D. 是否允许把该能力标为 PASS，以及下一个依赖可执行的合同。

没有完整证据时必须判定为“已实现待验收”，不能推测通过。
```

## 五、提示词 D：同一 Claude Code 会话的 findings 修复提示词

```text
你刚刚在冷启动自审中针对完整产品 requirement IDs 产出了 findings。现在先处理这些 findings 与其直接根因，不在缺陷未关闭时假装推进其他能力。

执行规则：
1. 逐条复现 finding，不能直接假设 Qwen 正确或错误。
2. 为每条已确认问题先补失败测试或可重复验收。
3. 做最小范围修复，不回退用户其他改动。
4. 每修一类问题只跑目标测试。
5. 所有 findings 关闭后再跑一次 full Vitest、vue-tsc 和 ACP Rust full lib。
6. 重新生成受影响尺寸的真实 Tauri 截图，不需要重复无关场景。
7. 更新 execution ledger，保留原 finding、修复 commit/diff 和证据路径。
8. 如果 finding 实际不成立，给出命令、截图或代码路径证明，不要只口头反驳。
9. 不允许用隐藏按钮、固定魔法高度、生产 test hook 或跳过审批来让测试变绿。
10. 交付时按 finding ID 逐条说明 resolved / not reproducible / blocked。
```

## 六、Repository 治理能力补充提示词

```text
当前 compliance matrix 指向 Repository 治理、共享合同或变更归属缺口。聚焦完成这些 requirement IDs；不要借治理之名做全仓重写。

先读：
- D:\dingsun\acp-ui\docs\codex\2026-07-10-game-agent-handoff-and-execution-plan.md 的阶段 C。
- 当前 execution ledger。

目标：
1. 生成完整但可读的 worktree inventory。
2. 标记 active、experimental、legacy、archive-candidate。
3. 固定 Game Agent 主线模块和验证入口。
4. 抽出无 Vue、无 Tauri 依赖的 operator-remote-client 包。
5. 不删除任何历史模块，除非有引用证明、替代测试和项目负责人确认。

必须先盘点再修改。每一批目录治理独立提交，独立回归。禁止全仓格式化、批量 cargo fix、换行符重写和大爆炸式移动。

交付：
- worktree inventory。
- 模块所有权表。
- 依赖图。
- active/legacy 分类依据。
- 统一 D 盘测试脚本。
- 远程客户端包的合同测试。
- 每批变更的测试证据与回退方式。
```

## 七、VSCode 客户端能力补充提示词

```text
依赖合同：真实 Operator 闭环可用，Remote Operator contract 已版本化且不会由 VSCode 自行复制。若依赖不成立，先回到 compliance matrix 修复依赖，不要制造孤立插件。

你现在实现第一个真实 VSCode Game Operator 客户端。仓库中没有现成 VSCode 扩展源码，.vscode 只是开发配置。不要声称是在“补全旧插件”。

建议目录：
- D:\dingsun\acp-ui\clients\vscode-game-operator
- D:\dingsun\acp-ui\packages\operator-remote-client

边界：
1. 扩展只调用 Remote Operator API，不嵌入 Agent runtime。
2. 不直接写游戏项目。
3. Token 存 VSCode SecretStorage，不进 settings、URL、日志、遥测或 git。
4. workspace folder 映射 project_path，必须遵守后端 project root scope。
5. 计划审批和文件审批必须明确区分。
6. diff 使用 VSCode diff editor，候选内容使用只读虚拟文档 provider。
7. 断线、401、403、scope mismatch、后端重启均有明确状态。
8. 关闭 VSCode 不自动停止任务。
9. 使用 vscode.env.language 与用户偏好映射 ui_locale，遵守完整蓝图的 message key、术语、文本扩展和可访问性合同。

最小功能：
- 连接设置与健康检查。
- Activity Bar 的 Tasks、Timeline、Approvals、Memory。
- Start/Pause/Resume/Stop/Redirect。
- Approve/Reject/Request changes。
- 逐文件 diff。
- 状态栏与通知。
- 打开 ACP UI 与备份目录。

验收：
- Extension Development Host 真实运行。
- 连接 D 盘 ACP Remote Operator。
- 使用隔离 Godot fixture 创建任务。
- 完成计划审批与至少一项控制操作。
- diff editor 显示真实后端候选内容。
- Token 泄漏扫描通过。
- 单测、扩展集成测试和一条真实 ACP smoke 通过。

只在以上证据完整后报告 VSCode MVP 完成。
```

## 八、IDEA 客户端能力补充提示词

```text
依赖合同：operator-remote-client、稳定 error/event/message key 和真实 Operator 闭环可用。现在实现 IDEA 工具窗口，不复制后端逻辑。

建议目录：
D:\dingsun\acp-ui\clients\idea-game-operator

技术：Kotlin、IntelliJ Platform Gradle Plugin、ToolWindow、PasswordSafe、DiffManager。

边界：
1. Project.basePath 只作为默认 project path。
2. Token 使用 PasswordSafe。
3. HTTP 在后台线程执行，UI 更新回 EDT。
4. 文件写入只由 ACP Operator 完成。
5. 不通过 PSI、VirtualFile 或 shell 绕过补丁审批。
6. 计划审批与文件审批使用不同文案、动作和风险说明。
7. 遵循 IDE locale 与用户 ui_locale，所有术语、错误和审批通过共享 locale contract 渲染。

最小功能与 VSCode 对齐：任务、timeline、审批、diff、控制、断线恢复、错误状态。

验收：
- 使用 IntelliJ Sandbox 启动真实插件。
- 连接真实 ACP Remote Operator。
- 创建或接管一个 D 盘 Godot 任务。
- 使用 IDE diff viewer 查看候选补丁。
- 完成计划审批或 request_changes。
- 凭据不出现在配置和日志。
- 单测、sandbox UI smoke 和真实 API smoke 通过。
```

## 九、Web 远程与公网安全能力补充提示词

```text
当前 Remote Operator 只允许本机或受控局域网。你的目标是建立可部署的远程控制面，但在 TLS、身份和 RBAC 未完成前绝不能直接暴露公网。

必须实现：
1. TLS 反向代理参考部署。
2. OIDC/OAuth2 登录。
3. viewer/operator/approver/admin RBAC。
4. project scope 与 action scope。
5. 持久化审计、查询、导出与保留策略。
6. rate limit、重放防护、请求幂等。
7. 密钥轮换、吊销与会话过期。
8. WebSocket 首帧认证或一次性短期 ticket，禁止 query token。
9. CSP、Origin、CORS 和 CSRF 策略。
10. Remote 与 Tauri 继续共享同一 OperatorState。
11. 根据账户偏好和 Accept-Language 渲染 UI，支持所有 production locale、RTL、时区和 Intl 格式。

威胁模型至少覆盖：
- Token 泄漏。
- 越权项目访问。
- 跨 Origin 调用。
- 重放审批。
- 旧客户端覆盖新状态。
- 日志泄密。
- WebSocket 劫持。
- 公网暴力请求。

必须有安全测试和部署边界文档。没有 TLS/OIDC/RBAC 证据时，只能报告“局域网 MVP”，不能报告“支持公网远程操作”。
```

## 十、Game Domain Pack 与 Skills/MCP/Hooks 能力补充提示词

```text
依赖合同：真实 Godot 闭环、共享 Operator contract 和受控 Remote 能力可用。现在把分散的 skill、MCP、hook 和 memory 基础设施收敛为第一个 Game Domain Pack；若依赖不成立，先修复 compliance matrix 中对应合同，不建立绕过控制面的孤立扩展。

第一版只做 Godot，不做 Unity/Unreal/Ren'Py。

每个扩展声明：
- id、version、domain。
- input/output schema。
- required scopes。
- side effects。
- approval level。
- timeout 与资源限制。
- audit fields。
- 可取消性与幂等语义。

绝对边界：
1. Skill/MCP/Hook 不得直接写项目。
2. 所有候选改动进入 structured patch。
3. 所有真实写入进入 operator.patch.apply。
4. 模型 validation 文本不执行。
5. MCP 返回内容视为不可信外部数据。
6. Hook 失败必须有明确 fail-open/fail-closed 策略，安全检查默认 fail-closed。
7. Memory 写入必须有来源、时间、scope 和可信度；未验证推断不能升级为长期项目事实。
8. 每个 Domain Pack 提供 locale glossary 和 message key，不得返回只能由英文字符串解析的状态。

先实现：
- godot-project-inspect skill。
- godot-gameplay-change skill。
- patch-policy hook。
- validation-result hook。
- project decision memory。

验收：
- 从 VSCode 或 Tauri 创建真实任务。
- skill 只产生结构化意图或候选补丁。
- MCP 不可越过 scope。
- hook 事件可观察、可取消、可审计。
- memory 面板能显示来源并允许操作员纠正。
- 所有真实写入仍经过两级审批。
```

## 十一、通用交付模板提示词

每次结束时附加给任一模型：

```text
请按以下结构交付，不要写泛泛总结：

1. 本轮范围
- 阶段：
- acceptance IDs：
- 明确未做：

2. 修改
- 文件绝对路径：
- 每个文件为何修改：
- 是否触碰既有用户改动：

3. 验证
- 精确命令：
- 精确通过/失败/忽略数量：
- real / fixture / mock / manual-real / automated-real 分类：
- 日志和截图绝对路径：

4. 安全边界
- 两级审批：
- Hermes no-tools：
- Operator 唯一写入：
- D 盘约束：
- Token/密钥泄漏检查：

5. 结果
- 完成 / 已实现待验收 / 阻塞：
- 未关闭 acceptance IDs：
- 已知风险：
- 下一步唯一优先项：

禁止只回复“100% 完成”。
```
