# Hermes Game Operator 测试计划

## 单元测试

### 1. 状态机测试

文件: `src-tauri/src/operator/state_machine.rs`

测试用例：

| 测试名称 | 初始状态 | 操作 | 期望状态 | 期望结果 |
|---------|---------|------|---------|---------|
| test_start_task | Idle | start_task | Planning | Ok |
| test_plan_ready | Planning | plan_ready | WaitingApproval | Ok |
| test_approve | WaitingApproval | approve | Running | Ok |
| test_reject | WaitingApproval | reject | Cancelled | Ok |
| test_pause | Running | pause | Paused | Ok |
| test_resume | Paused | resume | Running | Ok |
| test_stop_running | Running | stop | Cancelling | Ok |
| test_stop_paused | Paused | stop | Cancelling | Ok |
| test_stop_idle | Idle | stop | Error | InvalidTransition |
| test_complete | Running | complete | Completed | Ok |
| test_fail | Running | fail("error") | Failed | Ok |
| test_retry | Failed | retry | Planning | Ok |
| test_redirect | Running | redirect | Redirecting | Ok |
| test_replan | Redirecting | replan | Planning | Ok |

验收命令: `cargo test operator::state_machine`

### 2. Godot 分析器测试

文件: `src-tauri/src/domains/games/godot/project_analyzer.rs`

测试用例：

| 测试名称 | 输入 | 期望输出 |
|---------|------|---------|
| test_detect_valid_project | 包含 project.godot 的目录 | true |
| test_detect_invalid_project | 不包含 project.godot 的目录 | false |
| test_analyze_project | 有效 Godot 项目路径 | GodotProjectInfo |
| test_extract_project_name | project.godot 内容 | 项目名称 |
| test_find_player_controllers | 项目路径 | 玩家控制器列表 |

验收命令: `cargo test domains::games::godot`

## 集成测试

### 3. Tauri 命令测试

文件: `src-tauri/src/operator/commands.rs`

测试用例：

| 测试名称 | 命令 | 输入 | 期望输出 |
|---------|------|------|---------|
| test_start_task | operator_start_task | StartTaskRequest | StartTaskResponse |
| test_get_task | operator_get_task | task_id | OperatorTask |
| test_list_tasks | operator_list_tasks | - | Vec<OperatorTask> |
| test_pause_task | operator_pause_task | task_id | Ok |
| test_resume_task | operator_resume_task | task_id | Ok |
| test_stop_task | operator_stop_task | task_id | Ok |
| test_approve | operator_approve | ApproveRequest | Ok |
| test_list_events | operator_list_events | task_id | Vec<OperatorEvent> |
| test_godot_detect | godot_detect_project | path | bool |
| test_godot_analyze | godot_analyze_project | project_path | GodotProjectInfo |

验收命令: `cargo test operator::commands`

### 4. 端到端流程测试

场景：完整任务执行流程

步骤：
1. 调用 `operator_start_task` 创建任务
2. 验证状态为 Planning
3. 调用 `operator_pause_task` 暂停
4. 验证状态为 Paused
5. 调用 `operator_resume_task` 恢复
6. 验证状态为 Running
7. 调用 `operator_approve` 批准操作
8. 验证状态保持 Running
9. 调用 `operator_stop_task` 停止
10. 验证状态为 Cancelled
11. 调用 `operator_list_events` 获取事件
12. 验证事件流完整

验收命令: `cargo test operator::e2e`

## 前端测试

### 5. 组件测试

文件: `src/features/game-operator/components/*.vue`

| 组件 | 测试内容 |
|------|---------|
| OperatorControlBar | 状态显示、按钮点击 |
| ProgressTimeline | 事件渲染、时间戳格式化 |
| PlanPanel | 计划步骤显示、状态标记 |
| ApprovalDrawer | 审批卡片显示、批准/拒绝按钮 |

验收命令: `npm run test`

### 6. API 测试

文件: `src/api/__tests__/operator.test.ts`

| 测试名称 | API | 期望 |
|---------|-----|------|
| test_start_task | OperatorApi.startTask | 返回 task_id |
| test_get_task | OperatorApi.getTask | 返回 OperatorTask |
| test_pause_task | OperatorApi.pauseTask | 成功 |
| test_list_events | OperatorApi.listEvents | 返回事件列表 |

验收命令: `npm run test`

## E2E 测试

### 7. Playwright 测试

文件: `tests/e2e/game-operator.spec.ts`

场景：用户完整操作流程

```typescript
test('complete task workflow', async ({ page }) => {
  // 1. 导航到 Game Operator
  await page.goto('/games');

  // 2. 选择项目
  await page.fill('[data-testid="project-path"]', '/path/to/godot/project');

  // 3. 输入目标
  await page.fill('[data-testid="task-goal"]', '给 Player 添加二段跳');

  // 4. 启动任务
  await page.click('[data-testid="start-task"]');

  // 5. 验证任务状态
  await expect(page.locator('[data-testid="task-status"]')).toContainText('PLANNING');

  // 6. 等待计划生成
  await expect(page.locator('[data-testid="plan-panel"]')).toBeVisible();

  // 7. 批准操作
  await page.click('[data-testid="approve-btn"]');

  // 8. 验证任务完成
  await expect(page.locator('[data-testid="task-status"]')).toContainText('COMPLETED');
});
```

验收命令: `npx playwright test game-operator`

## 测试数据

### Godot 测试项目

创建最小测试项目:

```
test-godot-project/
├── project.godot
├── scenes/
│   └── Main.tscn
├── scripts/
│   └── Player.gd
└── assets/
    └── player.png
```

`project.godot`:
```ini
; Engine configuration file.
config/name="Test Project"
config/features=PackedStringArray("4.2")
run/main_scene="res://scenes/Main.tscn"
```

`scripts/Player.gd`:
```gdscript
extends CharacterBody2D

func _physics_process(delta):
    if Input.is_action_just_pressed("jump"):
        if is_on_floor():
            velocity.y = -400
```

## 覆盖率目标

| 模块 | 目标覆盖率 |
|------|-----------|
| operator/state_machine | 100% |
| operator/commands | 90% |
| domains/games/godot | 80% |
| game-operator 组件 | 70% |
| operatorApi | 85% |

验收命令: `cargo tarpaulin` (Rust) / `npm run test -- --coverage` (前端)
