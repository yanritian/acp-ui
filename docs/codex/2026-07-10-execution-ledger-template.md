# Game Agent 执行与验收台账模板

> 使用要求：每轮复制本文件为 `docs/codex/execution-ledger-<阶段>-<日期>.md`。  
> 台账由执行者持续更新。当前 Qwen 3.7 Plus 运行在 Claude Code 内，复核采用同一 Agent 冷启动自审，并必须如实标记；若以后有独立审查者，再追加独立复核。  
> 没有证据路径的勾选无效。

## 一、状态定义

| 状态 | 含义 |
|---|---|
| `NOT_STARTED` | 尚未实施 |
| `IN_PROGRESS` | 正在实施，尚不能验收 |
| `IMPLEMENTED` | 代码存在，真实或全量证据不完整 |
| `PASS` | acceptance criteria 与证据全部满足 |
| `FAIL` | 已复现且不满足预期 |
| `BLOCKED` | 外部条件阻止继续，已有完整阻塞证据 |
| `N/A` | 经审查确认不适用，并写明原因 |

禁止使用模糊状态：`差不多`、`基本完成`、`应该可以`、`100%`。

## 二、本轮元数据

```text
当前工作焦点：
执行者：
审查者：
开始时间：
结束时间：
仓库：D:\dingsun\acp-ui
分支：
开始 HEAD：
结束 HEAD：
本轮 artifact 根目录：
本轮 SQLite：
本轮 Godot fixture：
本轮 WebView2 data：
```

## 三、范围

### 3.1 本轮目标

```text
用一句可验收的话描述目标。
```

### 3.2 明确不做

- [ ] 不扩展其他行业。
- [ ] 不在依赖合同未满足时制造孤立平台实现。
- [ ] 不做全仓格式化或重构。
- [ ] 不清理来源不明的用户改动。
- [ ] 不重复实现已完成的后端闭环。

### 3.3 允许修改的文件

| 路径 | 原因 | 所有权/来源已确认 |
|---|---|---|
|  |  |  |

### 3.4 禁止修改的文件或边界

| 边界 | 原因 |
|---|---|
| 两级审批语义 | 防止授权合并 |
| Hermes `--no-tools` | 防止模型直接写入 |
| structured patch policy | 防止路径与内容越界 |
| 固定 Godot validation | 防止执行模型命令 |
| Remote/Tauri 共享状态 | 防止状态分叉 |

## 三-A、完整产品 Compliance Matrix

本表必须持续存在，不能只保留当前桌面缺陷。执行者应把完整蓝图的 Requirement IDs 细化到独立行；下面是能力域总表，不代表未列出的子项可以省略。

| 能力域 | 蓝图范围 | 当前基线判断 | 本轮状态 | 实现映射 | 测试/真实证据 | 未满足合同 |
|---|---|---|---|---|---|---|
| 产品不变量 | INV-001..010 | 部分已实现 |  |  |  |  |
| Project Workspace | PRJ-001..006 | Godot path/detection 部分实现 |  |  |  |  |
| Task and Goal | TSK-001..006 | 基础任务与控制已实现 |  |  |  |  |
| Plan and Approval | PLN/APR | 两级审批已实现，多人规则未实现 |  |  |  |  |
| Patch and Artifact | PAT/ART | 文本 patch 已实现，binary artifact 未完成 |  |  |  |  |
| Validation | VAL-001..006 | Godot 已实现，其他引擎未完成 |  |  |  |  |
| Progress/Memory/Trajectory | OBS + 蓝图 15 | timeline 基础存在，完整合同未完成 |  |  |  |  |
| Domain Data Model | 蓝图 6 | 核心实体部分存在 |  |  |  |  |
| State/Concurrency | 蓝图 7 | 基础状态机存在，lease/revision 待审计 |  |  |  |  |
| Agent Runtime | 蓝图 8 | Hermes no-tools 已实现 |  |  |  |  |
| Game Domains | 蓝图 9 | Godot 部分完整，其余仅旧资产/声明 |  |  |  |  |
| Tauri Desktop | 蓝图 10.2 | real task/recovery 已证明，可达性未通过 |  |  |  |  |
| VSCode | 蓝图 10.3 | 无真实扩展源码 |  |  |  |  |
| IDEA | 蓝图 10.4 | 无真实插件源码 |  |  |  |  |
| Web | 蓝图 10.5 | Remote client 基座存在 |  |  |  |  |
| Mobile/IM | 蓝图 10.6 | 分散 adapter，统一合同未完成 |  |  |  |  |
| UI/Information Architecture | 蓝图 11 | Game Operator 基础存在 |  |  |  |  |
| Internationalization | 蓝图 12 | 11 locale 存在，Game Operator 全英文 |  |  |  |  |
| Remote API | 蓝图 13 | 本地/LAN 路由存在，完整版本合同未完成 |  |  |  |  |
| Persistence/Sync | 蓝图 14 | SQLite v1 已实现，多端 revision 待完善 |  |  |  |  |
| Skills/MCP/Hooks/Domain Pack | 蓝图 16 | 基础设施分散，治理合同未完成 |  |  |  |  |
| Security | 蓝图 17 | loopback/LAN 基础存在，企业合同未完成 |  |  |  |  |
| Reliability/Performance | 蓝图 18 | 部分 timeout/cancel 存在，SLO 未闭环 |  |  |  |  |
| Observability/Audit/Cost | 蓝图 19 | event/audit 基础存在，完整 telemetry 未完成 |  |  |  |  |
| Configuration/Policy | 蓝图 20 | 分散配置存在，优先级合同未完整 |  |  |  |  |
| Testing/A11y/I18n QA | 蓝图 21 | 单测丰富，desktop/i18n/a11y/chaos 有缺口 |  |  |  |  |
| Build/Release/Deployment | 蓝图 22 | Debug 可运行，签名发布体系未完成 |  |  |  |  |
| Repository Governance | 蓝图 23 | 工作树混乱，ownership 未固定 |  |  |  |  |
| Complete Acceptance | 蓝图 26/27 | 未满足 |  |  |  |  |

每次关闭一个 requirement ID 后，都必须更新本表以及对应细化行。当前工作焦点可以改变，但完整矩阵不能删除或缩减。

## 四、D 盘预检

| 检查 | 值 | 状态 | 证据 |
|---|---|---|---|
| `TEMP` |  |  |  |
| `TMP` |  |  |  |
| `CARGO_HOME` |  |  |  |
| `RUSTUP_HOME` |  |  |  |
| `XDG_CACHE_HOME` |  |  |  |
| `HOME` |  |  |  |
| `APPDATA` |  |  |  |
| `LOCALAPPDATA` |  |  |  |
| `WEBVIEW2_USER_DATA_FOLDER` |  |  |  |
| `ACP_UI_CONFIG_PATH` |  |  |  |
| `ACP_UI_HISTORY_DB` |  |  |  |
| `ACP_OPERATOR_STATE_DB` |  |  |  |
| Node 路径 |  |  |  |
| Cargo 路径 |  |  |  |
| Godot 路径 |  |  |  |
| Hermes 路径 |  |  |  |

预检结论：

```text
是否发现本轮产物写入 C 盘：
检查方法：
处置：
```

## 五、工作树归属

| 文件/目录 | 开始时状态 | 本轮是否修改 | 来源推断 | 处理 |
|---|---|---|---|---|
|  |  |  |  |  |

确认：

- [ ] 未执行 `git reset --hard`。
- [ ] 未执行 `git checkout --` 回退文件。
- [ ] 未执行破坏性 `git clean`。
- [ ] 未做全仓格式化、换行或编码重写。
- [ ] 未暂存无关改动。

## 六、当前桌面缺陷与真实场景台账

以下为当前已知状态。新执行者必须复验后才能把 `IMPLEMENTED` 改成 `PASS`。

| ID | Acceptance criteria | 当前状态 | 当前证据 | 下一动作 |
|---|---|---|---|---|
| DSK-01 | 真实 Tauri 进程进入 Game Operator | PASS | `initial-1200x800.png`、真实 PID 记录 | 新轮次 smoke 复验 |
| DSK-02 | Remote 创建任务后桌面自动发现 | PASS | `remote-task-discovered.png` | 保留回归测试 |
| DSK-03 | 计划审批卡显示三种真实 options | IMPLEMENTED | API 返回三 options；审批卡可见 | 证明三个按钮可达 |
| DSK-04 | 逐文件结构化 diff 审批 | IMPLEMENTED | 后端 HTTP/Rust 已验收 | 真实桌面长 diff 待验收 |
| DSK-05 | Pause/Resume/Stop/Redirect 控制 | IMPLEMENTED | 后端测试；Stop UI 可见 | 真实窗口逐项控制 |
| DSK-06 | waiting_approval 下重启恢复 | PASS | `operator-restarted-restored-1024x720.png` | 自动化覆盖待补 |
| DSK-07 | stale 不覆盖操作员修改 | PASS-BACKEND | Rust 回归 | 桌面错误展示待验收 |
| DSK-08 | validation passed/failed/skipped 正确显示 | PASS-BACKEND | Rust + real Godot smoke | 桌面三态截图待补 |
| DSK-09 | 三种窗口尺寸布局可用 | FAIL | 1024x720 审批按钮尚不可见 | 修复唯一滚动宿主 |
| DSK-10 | 键盘可到达全部审批按钮 | FAIL | Tab 进入 Redirect，隐藏按钮未稳定滚入 | 修复焦点滚动 |
| DSK-11 | 长 diff 可读且不造成整页横向溢出 | NOT_STARTED | 无真实桌面证据 | 建立 deterministic fixture |
| DSK-12 | 自动化 desktop E2E 可重复 | BLOCKED | tauri-driver 被系统移除 | 签名驱动或受控环境 |

说明：`PASS-BACKEND` 不是最终合法状态。复制台账时应改为 `IMPLEMENTED`，直到完整蓝图要求的 UI、locale、a11y 与跨平台证据补齐。

## 七、自定义 Acceptance IDs

| ID | Given | When | Then | 证据类型 | 状态 |
|---|---|---|---|---|---|
|  |  |  |  | real/fixture/mock |  |

每个 Then 必须可观测，不能写“体验良好”“功能正常”。

## 八、测试先行记录

| 测试 ID | 失败前行为 | 新测试路径 | Red 证据 | Green 证据 |
|---|---|---|---|---|
|  |  |  |  |  |

若无法自动测试，填写可重复的真实验收步骤与失败截图，不能留空。

## 九、实现记录

| 顺序 | 文件 | 修改 | 为什么是最小范围 | 风险 |
|---|---|---|---|---|
| 1 |  |  |  |  |

## 十、测试结果

### 10.1 目标测试

| 命令 | 文件数 | passed | failed | ignored/skipped | 日志路径 |
|---|---:|---:|---:|---:|---|
|  |  |  |  |  |  |

### 10.2 类型与构建

| 命令 | 退出码 | 结果 | 日志路径 |
|---|---:|---|---|
| vue-tsc |  |  |  |
| Vite build |  |  |  |
| Cargo check/build |  |  |  |

### 10.3 全量回归

| 套件 | 最后旧基线 | 本轮实际结果 | 是否达到基线 | 日志路径 |
|---|---|---|---|---|
| Frontend Vitest | 79 files / 1209 tests |  |  |  |
| ACP Rust lib | 312 passed / 2 ignored，后增 1 条测试 |  |  |  |
| Hermes Game | 7 passed |  |  |  |
| Real Hermes opt-in | 1 passed |  |  |  |
| Real Godot opt-in | 1 passed |  |  |  |

不运行某项时写原因，不允许把旧基线抄成当前结果。

## 十一、真实运行证据

### 11.1 进程

| 进程 | 可执行文件 | PID | 端口 | 启动命令日志 | 退出码 |
|---|---|---:|---|---|---:|
| Vite |  |  | 1420 |  |  |
| ACP UI |  |  | 1421/1422 |  |  |
| Hermes |  |  | N/A |  |  |
| Godot |  |  | N/A |  |  |

### 11.2 数据

| 数据 | 绝对路径 | 隔离 | 是否提交 |
|---|---|---|---|
| SQLite |  | 是/否 | 否 |
| WebView2 |  | 是/否 | 否 |
| Godot fixture |  | 是/否 | 否 |
| Backup |  | 是/否 | 否 |

### 11.3 任务

```text
task_id:
project_path:
goal:
start status:
end status:
approval IDs:
recovery origin:
```

## 十二、截图矩阵

| 场景 | 1440x900 | 1280x800 | 1024x720 | 键盘 | 结论 |
|---|---|---|---|---|---|
| 初始任务 |  |  |  |  |  |
| 计划审批 |  |  |  |  |  |
| 补丁审批 |  |  |  |  |  |
| running 控制条 |  |  |  |  |  |
| paused |  |  |  |  |  |
| validation error |  |  |  |  |  |
| restart recovery |  |  |  |  |  |

截图检查：

- [ ] 无整页横向溢出。
- [ ] 无内容或按钮裁切。
- [ ] 无文字重叠。
- [ ] 无巨大意外空白。
- [ ] 审批按钮可达。
- [ ] 状态不只靠颜色表达。
- [ ] 长 diff 可独立滚动。
- [ ] 错误不会显示为 completed。

## 十三、安全不变量

| 不变量 | 检查方式 | 结果 | 证据 |
|---|---|---|---|
| 计划审批与补丁审批分离 | action/approval ID/event |  |  |
| Hermes 无工具 | 命令与 tool count |  |  |
| Operator 唯一写入 | 调用链与文件监测 |  |  |
| 模型 validation 不执行 | runner 参数来源 |  |  |
| 路径与 stale 校验保留 | Rust 测试 |  |  |
| Remote/Tauri 共享状态 | 同 task ID/sequence/SQLite |  |  |
| Token 不泄漏 | URL/日志/审计/配置扫描 |  |  |
| D 盘约束 | 环境与产物扫描 |  |  |

## 十四、阻塞记录

阻塞只有满足以下内容才有效：

```text
Blocker ID:
首次发生时间:
复现命令:
精确错误:
发生频率:
已尝试方案:
每个方案结果:
为什么不能继续:
需要用户/系统/外部团队做什么:
解除后第一条命令:
相关日志路径:
```

当前已知工具阻塞示例：

```text
Blocker ID: TOOL-TAURI-DRIVER-001
现象: tauri-driver 2.0.6 安装成功后，未签名二进制在首次执行前后从 D 盘消失。
影响: 无法建立稳定 WebDriver desktop E2E。
已尝试: 多个 D 盘 install root；检查 Defender 事件；EdgeDriver 单独下载。
结论: 不再无上限重复安装。需要签名二进制、用户允许规则或受控构建机。
替代证据: manual-real Tauri + Win32 input + PrintWindow + Remote/SQLite 状态。
```

## 十五、冷启动自审与独立审查 Findings

当前一体化运行方式下，`审查类型` 填 `same-agent cold review`，不得填“独立审查”。只有另一个独立会话或不同审查者真正重新核验时，才能填 `independent review`。

| Finding ID | 审查类型 | 严重度 | 文件/行 | 问题 | 复现 | 修复状态 | 证据 |
|---|---|---|---|---|---|---|---|
|  | same-agent cold review / independent review | P0/P1/P2/P3 |  |  |  |  |  |

## 十六、最终判定

```text
本轮结论：完成 / 已实现待验收 / 阻塞
通过的 acceptance IDs：
未通过的 acceptance IDs：
明确未覆盖：
是否允许关闭当前 requirement IDs：
下一步唯一优先项：
```

签字：

```text
执行者：
日期：
审查者：
日期：
项目负责人决定：
```
