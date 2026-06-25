# C-1 代码双写统一计划

## 问题背景

`src-tauri/src/` 和 `crates/swarm-engine/src/` 存在两套完全不同的 Goal 实现：

| 维度 | src-tauri/src/goal.rs | swarm-engine/goal.rs |
|------|----------------------|---------------------|
| 构造器参数 | 3 参数 | 4 参数 |
| Worker 字段 | assigned_worker: Option<WorkerId> | executor: String |
| Token 字段 | token_budget: Option<u64>, token_used | token_budget: u64, tokens_used |
| 依赖字段 | dependencies | depends_on |
| 父 Goal | parent_id | parent_goal_id |
| 时间戳 | u64 (UNIX ms) | chrono::DateTime<Utc> |

## 解决方案

### 方案 A: Re-export（推荐）

让 swarm-engine crate 直接 re-export src-tauri/src/ 的类型：

```rust
// crates/swarm-engine/src/lib.rs
pub use acp_ui_lib::goal::{Goal, GoalStatus, CompletionCondition};
```

**优点**: 最小改动，保持 src-tauri/src/ 为权威版本
**缺点**: swarm-engine 需要依赖 acp-ui_lib（反向依赖）

### 方案 B: Core Crate 提取

将核心类型提取到 acp-core crate：

1. 在 acp-core 中定义统一的 Goal 类型
2. src-tauri/src/ 和 swarm-engine 都使用 acp-core 的 Goal
3. 执行逻辑保留在各自模块

**优点**: 清晰的依赖关系，可独立使用
**缺点**: 需要修改大量代码

## 实施步骤（方案 B）

### Phase 1: acp-core 扩展

1. 在 acp-core/goal_protocol.rs 中添加：
   - GoalStatus enum
   - CompletionCondition enum  
   - IterationRecord struct
   - Goal struct（包含 GoalSpec）

2. 添加 serde 和 chrono 支持

### Phase 2: src-tauri/src/ 迁移

1. 修改 goal.rs 使用 acp_core::Goal
2. 添加本地扩展字段（如 assigned_worker）
3. 保持 Tauri 命令兼容

### Phase 3: swarm-engine 迁移

1. 修改 goal.rs 使用 acp_core::Goal
2. 删除重复的 Goal 定义
3. 更新所有引用

### Phase 4: 类型对齐

1. 统一字段命名（dependencies vs depends_on）
2. 统一时间戳类型
3. 统一 token 字段命名

## 预估工作量

- Phase 1: 1-2 小时
- Phase 2: 2-3 小时
- Phase 3: 2-3 小时
- Phase 4: 1-2 小时

总计: 6-10 小时

## 状态

计划待执行。建议在单独的 feature branch 中实施。