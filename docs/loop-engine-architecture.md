# Loop 工程架构设计

## 状态：规划中

## 1. Loop 工程核心概念

```
┌─────────────────────────────────────────────────────────────┐
│                    LOOP ENGINE                               │
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │ Reconcile   │───▶│ Self-Healing│───▶│ Evolution   │     │
│  │ Loop        │    │ Loop        │    │ Loop        │     │
│  │ (执行)      │    │ (修复)      │    │ (优化)      │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│        │                  │                  │             │
│        ▼                  ▼                  ▼             │
│  ┌─────────────────────────────────────────────────┐      │
│  │              Learning Loop (知识沉淀)             │      │
│  └─────────────────────────────────────────────────┘      │
│                         │                                   │
│                         ▼                                   │
│  ┌─────────────────────────────────────────────────┐      │
│  │              Loop Orchestrator                   │      │
│  │  - 统一调度                                      │      │
│  │  - 状态持久化                                    │      │
│  │  - 可视化面板                                    │      │
│  └─────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## 2. 四层 Loop 设计

### Layer 1: ReconcileLoop (已实现)

**功能**: Goal 迭代执行
- Budget 控制 (token + iteration)
- Worker 分派
- CompletionCondition 评估
- 失败重试 + feedback 记录

**状态**: ✅ 已实现 (reconcile.rs)
- 同步版本: reconcile_once()
- 异步版本: reconcile_once_async()

**需要改进**:
- 添加 Goal 失败后的自动 Self-Healing 触发
- 记录 tokens_used (目前为 0)

### Layer 2: Self-Healing Loop (框架已实现)

**功能**: 异常检测 + 自动修复
- EWMA 动态基线
- Anomaly 检测 (心跳超时、内存/CPU 异常、错误率)
- HealingAction 执行 (重启、切换模型、压缩上下文)
- Pattern 持久化 (反馈学习)

**状态**: ✅ 框架已实现 (self_healing.rs)
- AnomalyDetector
- HealingExecutor
- Tauri commands: healing_execute, healing_list_actions

**需要激活**:
- 与 ReconcileLoop 集成 (Goal 失败时触发检测)
- Health Monitor 定期检查
- 实时 metrics 采集

### Layer 3: Evolution Loop (框架已实现)

**功能**: Capability 自动演进
- Pattern extraction (从成功执行中提取模式)
- Skill 自动生成
- Fitness Score 追踪
- EWMA proficiency 更新

**状态**: 🟡 框架存在 (capability-evolver.ts)
- calculateCompositeScore()
- evolutionLog

**需要完善**:
- Pattern → Skill 自动生成流程
- 与 Goal 成功事件集成
- 长期 Fitness 数据库存储

### Layer 4: Learning Loop (待设计)

**功能**: 知识沉淀 + 跨会话共享
- Evolution 事件持久化
- Pattern 库共享
- 最佳实践推荐
- 类似任务匹配

**状态**: ❌ 待设计

## 3. Loop Orchestrator (新增)

统一调度所有 Loop：

```rust
pub struct LoopOrchestrator {
    reconcile: ReconcileLoop,
    healer: HealingExecutor,
    evolver: CapabilityEvolver,
    learner: LearningEngine,
    state: LoopState,
}

pub struct LoopState {
    active_goals: HashMap<String, Goal>,
    anomalies: Vec<AnomalyRecord>,
    evolutions: Vec<EvolutionEvent>,
    patterns: Vec<Pattern>,
}
```

## 4. 实施计划

### Phase 1: ReconcileLoop 增强 (Week 1)
1. 添加 Goal 失败 → Self-Healing 触发
2. Token tracking 实现
3. async 版本完善

### Phase 2: Self-Healing 激活 (Week 2)
1. Health Monitor 定时任务
2. Metrics 采集器
3. 与 ReconcileLoop 集成

### Phase 3: Evolution Loop 完善 (Week 3)
1. Pattern → Skill 生成器
2. Goal 成功事件监听
3. Fitness 数据库存储

### Phase 4: Learning Loop 设计 (Week 4)
1. Pattern 库存储
2. 跨会话检索
3. 最佳实践推荐

### Phase 5: Loop Orchestrator (Week 5)
1. 统一调度器
2. 状态持久化
3. Vue 可视化面板

## 5. 关键接口

### Goal → Self-Healing 触发
```rust
// 在 ReconcileLoop::reconcile_once_async() 中
if goal.status == GoalStatus::Failed { .. } {
    // 触发 Self-Healing
    let anomaly = self.detect_anomaly_from_goal(goal);
    self.healer.execute_healing(&anomaly.id)?;
}
```

### Goal 成功 → Evolution 触发
```typescript
// 在 goal-api.ts 中
if (goal.status === 'Converged') {
    // 提取 pattern
    const patterns = await evolver.extractPatterns(goal.iterationLog);
    // 生成/优化 Skill
    await evolver.evolveSkills(patterns);
}
```

## 6. 文件规划

| 文件 | 内容 |
|------|------|
| src-tauri/src/loop_engine.rs | LoopOrchestrator + 统一调度 |
| src-tauri/src/health_monitor.rs | Metrics 采集 + 定时检查 |
| src/lib/loop-api.ts | 前端 Loop API |
| src/stores/loop.ts | Vue Store |
| src/features/loop/LoopDashboard.vue | 可视化面板 |