---
name: WP-02~08 Runtime 协议分析
description: 2026-07-13 WP-02~08 协议状态分析
type: project
---

## 1. 运行摘要

```text
日期：2026-07-13
执行者：Claude Code + Qwen
仓库：D:/dingsun/acp-ui
分支：cleanup/project-snapshot-2026-06-25
起始 HEAD：3b454ca
工作包：WP-02、WP-03、WP-04、WP-08
目标：让 Operator Control Plane 成为唯一任务事实源，支持 revision、事件、审批、补丁、checkpoint、暂停、恢复、停止和重启恢复
```

## 2. 当前实现状态

### 2.1 已实现

| 组件 | 文件 | 状态 |
|---|---|---|
| 状态机 | state_machine.rs | ✅ 基础转移 |
| 事件类型 | types.rs | ✅ 28 种事件 |
| 任务类型 | types.rs | ✅ OperatorTask |
| 审批类型 | types.rs | ✅ ApprovalRequest |
| 补丁类型 | structured_patch.rs | ✅ FilePatch |
| 持久化 | persistence.rs | ✅ SQLite |
| 安全边界 | security.rs | ✅ PathGuard |
| 审批队列 | approval_queue.rs | ✅ 内存队列 |

### 2.2 缺失功能

| 功能 | 描述 | 优先级 |
|---|---|---|
| revision 字段 | 乐观并发控制 | HIGH |
| expected_revision 检查 | 写操作版本校验 | HIGH |
| REVISION_CONFLICT 错误 | 409 响应 | HIGH |
| checkpoint 恢复 | 重启后状态恢复 | MEDIUM |
| 审批持久化 | 审批记录不可变 | MEDIUM |
| 事件序列号 | sequence 连续递增 | MEDIUM |
| after_sequence 游标 | 事件流续读 | LOW |

### 2.3 状态转移表

当前 state_machine.rs 实现的转移：

```text
Idle -> Planning (start_task)
Planning -> WaitingApproval (plan_ready)
WaitingApproval -> Running (approve)
Running -> WaitingApproval (await_approval)
WaitingApproval -> Planning (request_changes)
WaitingApproval -> Cancelled (reject)
Running -> Paused (pause)
Paused -> Running (resume)
Planning/WaitingApproval/Running/Paused -> Redirecting (redirect)
Redirecting -> Planning (replan)
Planning/WaitingApproval/Running/Paused/Redirecting -> Cancelling (stop)
Cancelling -> Cancelled (cleanup_done)
Running -> Completed (complete)
Running/Paused/Planning -> Failed (fail)
Failed -> Planning (retry)
```

缺失转移（根据文档）：
- `created -> inspecting`
- `executing -> validating`
- `validating -> completed/failed`

## 3. TDD 测试清单

### 3.1 需要编写的失败测试

1. **revision 冲突测试**
   - 两个客户端同时写入，只有一个成功
   - 旧版本写入返回 409 REVISION_CONFLICT

2. **终态测试**
   - completed 状态不接受 pause/resume/stop
   - failed 状态不接受 control 命令
   - cancelled 状态不接受 control 命令

3. **审批测试**
   - high/critical 补丁没有审批不能 apply
   - reject 后文件 hash 不改变
   - 重复审批返回错误

4. **补丁测试**
   - apply 前检查文件 hash
   - 外部修改返回 PATCH_HASH_MISMATCH
   - apply 失败可回滚

5. **恢复测试**
   - 重启后恢复 waiting_approval 状态
   - 重启后恢复 paused 状态
   - 重启后不重复 apply 已应用补丁

## 4. 实现计划

### Phase 1: revision 字段 (WP-02)
1. 修改 OperatorTask 添加 revision: u64
2. 修改所有写操作检查 expected_revision
3. 返回 409 REVISION_CONFLICT
4. 更新前端和客户端类型

### Phase 2: 状态机完善 (WP-03)
1. 添加 inspecting、validating 状态
2. 实现状态转移表完整覆盖
3. 添加 checkpoint 写入
4. 实现 pause 协作取消

### Phase 3: 审批闭环 (WP-04)
1. 持久化审批记录
2. 审批状态机
3. 补丁 hash 检查
4. 审计事件

### Phase 4: 恢复机制 (WP-08)
1. checkpoint 序列化
2. 重启恢复流程
3. 事件游标恢复
4. 记忆快照

## 5. 风险与阻塞

- **影响范围大**: 修改 OperatorTask 影响所有测试
- **客户端同步**: 需要 VSCode、IDEA、Web 同步更新
- **时间估计**: 完整实现需要 2-3 天工作量

## 6. 当前状态

> WP-02 JSON Schema 已创建。revision 字段实现需要大量修改，建议分阶段执行。当前工作重点是编写 TDD 测试并逐步实现。