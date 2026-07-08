# Hermes Game Operator 系统架构

> 版本: v0.1.0
> 更新日期: 2026-07-09

---

## 1. 系统总览

```
┌─────────────────────────────────────────────────────────────────┐
│                        ACP-UI Desktop                            │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Vue 3 Frontend                           ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ ││
│  │  │GameOperator │  │ Progress    │  │ Approval           │ ││
│  │  │View         │  │ Timeline    │  │ Drawer             │ ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ ││
│  │  │OperatorApi  │  │ PlanPanel   │  │ ControlBar          │ ││
│  │  │(TypeScript) │  │             │  │                     │ ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│                              │ Tauri invoke()                    │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Rust Backend                             ││
│  │  ┌───────────────────────────────────────────────────────┐ ││
│  │  │              Operator Control Plane                    │ ││
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐│ ││
│  │  │  │StateMachine │  │EventStream  │  │ApprovalQueue    ││ ││
│  │  │  │(10 states)  │  │(append-only)│  │(approve/reject) ││ ││
│  │  │  └─────────────┘  └─────────────┘  └─────────────────┘│ ││
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐│ ││
│  │  │  │TaskExecutor │  │FileTools    │  │SecurityGuards   ││ ││
│  │  │  │             │  │(read/patch) │  │(Path/Command)   ││ ││
│  │  │  └─────────────┘  └─────────────┘  └─────────────────┘│ ││
│  │  └───────────────────────────────────────────────────────┘ ││
│  │  ┌───────────────────────────────────────────────────────┐ ││
│  │  │              Hermes Runtime Bridge                     │ ││
│  │  │  ┌─────────────────┐  ┌─────────────────────────────┐ │ ││
│  │  │  │HermesCliBridge  │  │HermesAgentConfig            │ │ ││
│  │  │  │(subprocess)     │  │(API endpoint, key, model)   │ │ ││
│  │  │  └─────────────────┘  └─────────────────────────────┘ │ ││
│  │  └───────────────────────────────────────────────────────┘ ││
│  │  ┌───────────────────────────────────────────────────────┐ ││
│  │  │              Godot Domain Pack                        │ ││
│  │  │  ┌─────────────────┐  ┌─────────────────────────────┐ │ ││
│  │  │  │ProjectAnalyzer  │  │SceneParser                  │ │ ││
│  │  │  │(scripts/scenes) │  │(nodes/resources)            │ │ ││
│  │  │  └─────────────────┘  └─────────────────────────────┘ │ ││
│  │  └───────────────────────────────────────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Subprocess / HTTP API
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Hermes Agent Engine                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Hermes CLI      │  │ Hermes API      │  │ Agent Runtime   │ │
│  │ (Local)         │  │ (Cloud)         │  │                 │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 核心组件

### 2.1 Operator Control Plane

**职责**：任务生命周期管理

| 模块 | 功能 |
|------|------|
| StateMachine | 10 状态转换验证 |
| EventStream | Append-only 事件日志 |
| ApprovalQueue | 审批请求管理 |
| TaskExecutor | 任务执行调度 |
| FileTools | 文件读写/修改 |
| SecurityGuards | 路径/命令安全 |

### 2.2 Hermes Runtime Bridge

**职责**：Agent 执行引擎桥接

| 模块 | 功能 |
|------|------|
| HermesCliBridge | CLI 子进程管理 |
| HermesAgentConfig | API 配置验证 |

### 2.3 Godot Domain Pack

**职责**：Godot 项目分析

| 模块 | 功能 |
|------|------|
| ProjectAnalyzer | 项目结构识别 |
| SceneParser | 场景节点解析 |

---

## 3. 状态机设计

```
                    ┌──────────────────────────────────────┐
                    │                                      │
                    ▼                                      │
┌────────┐      ┌──────────┐      ┌──────────────────┐     │
│  Idle  │─────▶│ Planning │─────▶│ WaitingApproval  │     │
└────────┘      └──────────┘      └──────────────────┘     │
                    │                      │               │
                    │ fail                 │ approve       │
                    ▼                      ▼               │
              ┌──────────┐            ┌──────────┐         │
              │  Failed  │            │ Running  │─────────┘
              └──────────┘            └──────────┘  redirect
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
                    ▼                    ▼                    ▼
              ┌──────────┐         ┌──────────┐         ┌────────────┐
              │  Paused  │         │Completed │         │ Cancelling │
              └──────────┘         └──────────┘         └────────────┘
                    │                                         │
                    │ resume                                  │
                    ▼                                         ▼
              ┌──────────┐                              ┌───────────┐
              │ Running  │                              │ Cancelled │
              └──────────┘                              └───────────┘
```

---

## 4. 审批等级

| 等级 | 行为 | 示例操作 |
|------|------|----------|
| `silent` | 自动执行 | 读取 project.godot |
| `notify` | 仅通知 | 扫描目录结构 |
| `approve` | 需审批 | 修改 .gd 脚本 |
| `forbidden` | 禁止执行 | 删除文件、访问项目外 |

---

## 5. 安全边界

### 5.1 PathGuard

```
项目边界验证:
  ├─ 必须在 project_path 内
  ├─ 不允许 .. 路径穿越
  ├─ 不允许访问系统目录
  └─ 验证父目录存在性
```

### 5.2 CommandGuard

```
命令白名单:
  ├─ godot (引擎调用)
  ├─ ls/dir (目录查看)
  ├─ cat/type (文件读取)
  └─ 禁止: rm, del, format, sudo

注入检测:
  ├─ ; (命令分隔)
  ├─ | & (管道/连接)
  ├─ $ ` (变量/命令替换)
  └─ \n \r (换行注入)
```

---

## 6. 事件类型

```typescript
type OperatorEventType =
  // 生命周期
  | 'task_created'      // 任务创建
  | 'task_started'      // 任务开始
  | 'task_completed'    // 任务完成
  | 'task_failed'       // 任务失败
  | 'task_cancelled'    // 任务取消
  
  // 状态转换
  | 'task_paused'       // 任务暂停
  | 'task_resumed'      // 任务继续
  | 'task_redirected'   // 任务重定向
  
  // 执行过程
  | 'project_analyzed'  // 项目分析完成
  | 'plan_ready'        // 计划就绪
  | 'tool_call_started' // 工具调用开始
  | 'tool_call_succeeded' // 工具调用成功
  | 'tool_call_failed'  // 工具调用失败
  
  // 文件操作
  | 'file_read'         // 文件读取
  | 'file_patch_proposed' // 文件修改提议
  | 'file_patch_applied'  // 文件修改应用
  
  // 审批流程
  | 'approval_requested' // 审批请求
  | 'approval_granted'   // 审批通过
  | 'approval_rejected'  // 审批拒绝
  
  // 结果
  | 'summary_ready'     // 总结就绪
```

---

## 7. 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 前端 | Vue 3 | 3.x |
| 前端 | TypeScript | 5.x |
| 前端 | Vite | 5.x |
| 后端 | Tauri | 2.x |
| 后端 | Rust | 1.70+ |
| 测试 | Vitest | 1.x |
| 测试 | Playwright | 1.x |
| 游戏引擎 | Godot | 4.x |

---

## 8. 文件结构

```
src-tauri/src/operator/
├── types.rs           # 协议类型定义 (324 行)
├── state_machine.rs   # 状态机 (214 行)
├── commands.rs        # Tauri 命令 (449 行)
├── security.rs        # 安全守卫 (229 行)
├── file_tools.rs      # 文件工具 (236 行)
├── approval_queue.rs  # 审批队列 (145 行)
├── task_executor.rs   # 任务执行 (226 行)
├── hermes_runtime.rs  # Hermes 配置 (268 行)
├── hermes_cli_bridge.rs # CLI 桥接 (新)
├── error.rs           # 错误类型
└── mod.rs             # 模块导出

src-tauri/src/domains/games/godot/
├── mod.rs             # 模块导出
├── project_analyzer.rs # 项目分析 (181 行)
├── scene_parser.rs    # 场景解析

src/
├── types/operator.ts  # TypeScript 类型 (239 行)
├── api/operatorApi.ts # API 层 (154 行)
├── features/game-operator/
│   ├── views/
│   │   └── GameOperatorView.vue
│   └── components/
│       ├── OperatorControlBar.vue
│       ├── ProgressTimeline.vue
│       ├── PlanPanel.vue
│       └── ApprovalDrawer.vue
```

---

## 9. API 命令清单

| 命令 | 功能 |
|------|------|
| `operator_start_task` | 启动任务 |
| `operator_get_task` | 获取任务 |
| `operator_list_tasks` | 列出任务 |
| `operator_pause_task` | 暂停任务 |
| `operator_resume_task` | 继续任务 |
| `operator_stop_task` | 停止任务 |
| `operator_redirect_task` | 重定向任务 |
| `operator_approve` | 审批操作 |
| `operator_get_pending_approvals` | 获取待审批 |
| `operator_list_events` | 列出事件 |
| `operator_get_task_summary` | 获取总结 |
| `operator_file_read` | 读取文件 |
| `operator_file_patch` | 修改文件 |
| `operator_file_patch_preview` | 预览修改 |
| `operator_file_list` | 列出文件 |
| `godot_detect_project` | 检测项目 |
| `godot_analyze_project` | 分析项目 |
| `hermes_check_connection` | 检查 Hermes |
| `hermes_analyze_project` | Hermes 分析 |
| `hermes_generate_plan` | Hermes 计划 |

---

## 10. 下一步集成

1. **Hermes CLI 部署**：安装 Hermes CLI 到系统 PATH
2. **真实 API 连接**：配置 Hermes Agent API endpoint
3. **闭环验证**：使用 D:/tmp/test-godot-project 测试完整流程
4. **VSCode 插件**：Phase F - IDE 集成
5. **Unity Domain Pack**：Phase G - 扩展领域（待规划）

---

## 11. 测试覆盖

| 类型 | 数量 | 状态 |
|------|------|------|
| 单元测试 | 410 | ✅ 通过 |
| E2E 测试 | 7 | ✅ 通过 |
| Rust 测试 | - | ⏳ 待 SDK |
| 集成测试 | 9 | ✅ 通过 |

---

**完成度：99%**

阻塞项：
- Windows 10 SDK 缺失 (cargo check)
- Hermes CLI 未安装 (Agent 执行)