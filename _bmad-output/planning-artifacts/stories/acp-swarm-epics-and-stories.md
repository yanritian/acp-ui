# acp-swarm Epic 和 Story 列表

*基于 PRD v2.0.0（42 个 FR）和架构设计文档*

---

## Epic 1: 仓库初始化与 Crate 拆分

**目标**：创建 acp-swarm 仓库，建立 Cargo workspace，拆分现有代码到 9 个 crate。

**验收标准**：
- `cargo check --workspace` 零错误零警告
- 每个 crate 独立编译测试
- src-tauri 变为薄壳（只做 Tauri 命令注册）

### Story 1.1: Fork 仓库并创建 Cargo workspace 骨架
- **FR 覆盖**：架构 Phase 0
- **估时**：4h
- **验收**：
  - [ ] 创建 acp-swarm 仓库（从 acp-ui fork）
  - [ ] 创建 Cargo.toml workspace 定义（9 个 crate）
  - [ ] 每个 crate 创建 src/lib.rs 空文件
  - [ ] `cargo check --workspace` 通过

### Story 1.2: 删除死代码和 orphan 文件
- **FR 覆盖**：架构 Phase 1
- **估时**：2h
- **验收**：
  - [ ] 删除 bot_gateway.rs（1245 行孤儿代码）
  - [ ] 删除 hermes_traits.rs（633 行死代码）
  - [ ] 删除 25 个前端 orphan 文件
  - [ ] `cargo check` 和 `npx vue-tsc --noEmit` 通过

### Story 1.3: 拆分 acp-core crate（零内部依赖）
- **FR 覆盖**：FR-38（部分）
- **估时**：6h
- **验收**：
  - [ ] 迁移 config.rs, agent_config_parser.rs, plugin_registry.rs, skill_commands.rs, circuit_breaker.rs
  - [ ] 创建 goal.rs（Goal/CompletionCondition/Evaluator/GoalStatus 类型定义）
  - [ ] 创建 skill.rs（WorkerSkill/WorkerCapabilityManifest/SwarmSkillRouter）
  - [ ] 创建 event.rs（AcpEvent/EventType）
  - [ ] 创建 error.rs（SwarmError/GoalError/WorkerError）
  - [ ] `cargo test -p acp-core` 通过

### Story 1.4: 拆分 event-bus crate
- **FR 覆盖**：FR-35（部分）
- **估时**：4h
- **验收**：
  - [ ] 创建 lib.rs（AcpMessageBus 主结构）
  - [ ] 实现 broadcast + 定向路由
  - [ ] 创建 adapters.rs 骨架（TauriAdapter, StdioAdapter, WebSocketAdapter, HttpAdapter）
  - [ ] `cargo test -p event-bus` 通过

### Story 1.5: 拆分 swarm-engine crate
- **FR 覆盖**：FR-1 到 FR-5（部分），FR-16 到 FR-20
- **估时**：6h
- **验收**：
  - [ ] 迁移 agent_orchestration.rs → orchestration.rs
  - [ ] 迁移 swarm_orchestrator.rs → orchestrator.rs
  - [ ] 迁移 smart_router.rs → router.rs
  - [ ] 迁移 self_healing.rs → healing.rs
  - [ ] 创建空文件：reconcile.rs, queen.rs, topology/
  - [ ] `cargo test -p swarm-engine` 通过

### Story 1.6: 拆分 workflow-engine crate
- **FR 覆盖**：架构 Phase 1
- **估时**：4h
- **验收**：
  - [ ] 迁移 workflow_engine.rs → engine.rs
  - [ ] 迁移 team_dag.rs → dag.rs
  - [ ] 迁移 approval_engine.rs → approval.rs
  - [ ] 创建空文件：checkpoint.rs
  - [ ] `cargo test -p workflow-engine` 通过

### Story 1.7: 拆分其他 crate
- **FR 覆盖**：架构 Phase 1
- **估时**：4h
- **验收**：
  - [ ] hook-runtime: 迁移 hooks_executor.rs → executor.rs
  - [ ] tool-sandbox: 迁移 permission_checker.rs → permission.rs
  - [ ] acp-transport: 迁移 agent_bus.rs → bus.rs, mcp_client.rs → mcp.rs
  - [ ] goal-parser: 创建空骨架
  - [ ] acp-cli: 创建空骨架
  - [ ] 所有 crate `cargo check` 通过

### Story 1.8: src-tauri 变为薄壳
- **FR 覆盖**：架构 Phase 1
- **估时**：4h
- **验收**：
  - [ ] src-tauri/src/lib.rs 只保留 Tauri 命令注册
  - [ ] 所有业务逻辑移到对应 crate
  - [ ] `cargo check -p acp-swarm-tauri` 通过
  - [ ] Tauri 命令仍然可以调用（集成测试）

**Epic 1 总计：34h（约 4.5 天）**

---

## Epic 2: 核心引擎实现

**目标**：实现 Goal-Driven 编排引擎核心（Reconcile Loop、拓扑执行、Queen 选举）。

**验收标准**：
- 单个 Goal 的 Reconcile Loop 能跑通（迭代到收敛）
- Star 和 Chain 拓扑可以执行
- Queen 选举和防脑裂工作正常

### Story 2.1: 实现 Goal 数据结构完整方法
- **FR 覆盖**：FR-1
- **估时**：4h
- **验收**：
  - [ ] Goal struct 所有字段完整（20+ 字段）
  - [ ] 实现 is_terminal() 方法
  - [ ] 实现 remaining_budget() 方法
  - [ ] 实现 append_feedback() 方法
  - [ ] serde 序列化/反序列化测试通过

### Story 2.2: 实现 CompletionCondition 枚举和评估器
- **FR 覆盖**：FR-2
- **估时**：8h
- **验收**：
  - [ ] 8 种 CompletionCondition 类型完整定义
  - [ ] ConditionEvaluator::evaluate() 实现所有 8 种类型
  - [ ] CommandSuccess: 执行命令，检查退出码
  - [ ] OutputContains: 执行命令，检查输出包含指定文本
  - [ ] OutputMatches: 执行命令，检查输出匹配正则
  - [ ] All/Any: 递归评估子条件
  - [ ] FileCheck: 检查文件存在和内容
  - [ ] HttpHealthCheck: HTTP 请求检查
  - [ ] QueenJudgment: 返回占位符（v2+ 实现）
  - [ ] 每种类型至少 2 个测试用例

### Story 2.3: 实现 GoalStatus 状态机
- **FR 覆盖**：FR-3
- **估时**：2h
- **验收**：
  - [ ] 9 种状态完整定义
  - [ ] 状态转移验证（非法转移返回错误）
  - [ ] 每次状态转移 emit 事件（goal.status_changed）
  - [ ] 状态机测试覆盖所有合法转移

### Story 2.4: 实现 Reconcile Loop 核心算法
- **FR 覆盖**：FR-4
- **估时**：12h
- **验收**：
  - [ ] 8 步算法完整实现（Budget Check → Max Iter → Build Message → Dispatch → Evaluate → Record → Check Converge → Oscillation）
  - [ ] Token Budget Check: tokens_used >= token_budget → BudgetExhausted
  - [ ] Max Iteration Check: current_iteration >= max_iterations → MaxIterReached
  - [ ] Dispatch to Worker: 调用 Worker 通信接口
  - [ ] Evaluate: 调用 ConditionEvaluator
  - [ ] Record: 创建 IterationRecord 并追加到 iteration_log
  - [ ] Check Convergence: evaluation.converged → Converged
  - [ ] Oscillation Detection: 相同反馈连续 3 次 → Queen 介入（v2+ 实现 Queen 逻辑，v1 返回 Failed）
  - [ ] 至少 5 个测试用例：
    - 第 1 次迭代就收敛
    - 第 3 次迭代收敛
    - Token Budget 耗尽
    - Max Iteration 达到
    - 震荡检测触发

### Story 2.5: 实现 Evaluator 分离（Auto + Queen）
- **FR 覆盖**：FR-5
- **估时**：6h
- **验收**：
  - [ ] Auto Evaluator: 直接调用 ConditionEvaluator::evaluate()
  - [ ] Queen Evaluator: 先跑 Auto，通过后让 Queen Worker 审查
  - [ ] Queen 审查通过 HTTP 调用 Queen Worker
  - [ ] Queen 评估的 Token 计入 Goal 的 token_budget
  - [ ] Adversarial/Hybrid 返回 NotImplemented（v2+）
  - [ ] 测试：Auto 评估收敛、Queen 评估收敛

### Story 2.6: 实现 Star 拓扑
- **FR 覆盖**：FR-16
- **估时**：8h
- **验收**：
  - [ ] Queen Worker 接收顶层 Goal
  - [ ] Queen 分解为多个子 Goal（调用 LLM 或规则）
  - [ ] 子 Goal 分配给其他 Worker 并行执行
  - [ ] 所有子 Goal Converged 后，Queen 验证顶层条件
  - [ ] Queen 不再强制 claude_code 类型
  - [ ] 如果没有 claude_code Worker，第一个注册的 Worker 成为 Queen（降级模式）
  - [ ] 测试：2 个 Worker 的 Star 拓扑

### Story 2.7: 实现 Chain 拓扑
- **FR 覆盖**：FR-17
- **估时**：4h
- **验收**：
  - [ ] Goal 按依赖链顺序执行
  - [ ] 每个 Goal 必须等前一个 Converged 才能开始
  - [ ] 前一个 Goal Failed → 后续 Goal 不执行
  - [ ] 测试：3 个 Goal 的 Chain 拓扑

### Story 2.8: 实现 Queen Lease 租约
- **FR 覆盖**：FR-18
- **估时**：4h
- **验收**：
  - [ ] QueenLease struct: lease_id, queen_id, ttl_seconds (默认 120), renew_interval_seconds (默认 30), granted_at
  - [ ] Queen 必须每 renew_interval_seconds 续期一次
  - [ ] 连续 3 次未续期 → 租约失效
  - [ ] Busy Queen 模式：Queen 正在执行 Goal 分解时，租约自动延期
  - [ ] 测试：租约过期、续期成功、Busy Queen 延期

### Story 2.9: 实现 Queen 选举算法
- **FR 覆盖**：FR-19
- **估时**：6h
- **验收**：
  - [ ] 收集所有已注册 Worker（不再限制类型）
  - [ ] 计算得分：proficiency * 0.4 + success_rate * 0.4 + (1/load) * 0.2
  - [ ] claude_code 类型额外加分（+0.1）
  - [ ] 得分最高的成为 Queen
  - [ ] 分配租约
  - [ ] 选举结果 emit 事件（queen.elected）
  - [ ] 测试：3 个 Worker 的选举（验证得分计算）

### Story 2.10: 实现防脑裂验证器
- **FR 覆盖**：FR-20
- **估时**：4h
- **验收**：
  - [ ] AntiSplitBrainValidator struct
  - [ ] 验证每个命令是否来自有效 Queen
  - [ ] 无效 Queen 的命令拒绝执行
  - [ ] 记录所有拒绝的命令（用于审计）
  - [ ] 测试：有效 Queen 命令通过、无效 Queen 命令拒绝

**Epic 2 总计：58h（约 7.5 天）**

---

## Epic 3: Worker Protocol + SDK

**目标**：实现 Worker 协议 5 个端点，提供 Rust 和 TypeScript SDK。

**验收标准**：
- Worker 可以注册、心跳、接收 Goal、汇报进度、返回结果
- Rust SDK 可以创建一个 Worker 并执行 Goal
- TypeScript SDK 可以创建一个 Worker 并执行 Goal

### Story 3.1: 实现 Worker 注册接口
- **FR 覆盖**：FR-6
- **估时**：4h
- **验收**：
  - [ ] POST /v1/worker/register 端点
  - [ ] Request: { worker_id, worker_type, version, capabilities: WorkerCapabilityManifest }
  - [ ] Response: { status: "registered", worker_id, heartbeat_interval_ms }
  - [ ] 注册信息通过 EventStore 持久化
  - [ ] emit worker.registered 事件
  - [ ] 测试：注册成功、重复注册失败

### Story 3.2: 实现 Worker 心跳接口
- **FR 覆盖**：FR-7
- **估时**：3h
- **验收**：
  - [ ] POST /v1/worker/heartbeat 端点
  - [ ] Request: { worker_id, status, current_load, active_goals, memory_usage_mb, uptime_seconds }
  - [ ] Response: { status: "ok" }
  - [ ] 连续 3 次未收到心跳 → Worker 失联 → emit worker.disconnected
  - [ ] 测试：心跳正常、心跳超时触发失联

### Story 3.3: 实现 Worker 接收 Goal 接口
- **FR 覆盖**：FR-8
- **估时**：4h
- **验收**：
  - [ ] POST /v1/worker/goal/receive 端点
  - [ ] Request: { goal_id, description, iteration, feedback_from_last_iteration, workspace_path, git_branch, token_budget, timeout_ms }
  - [ ] Response: { status: "accepted", execution_id, estimated_duration_ms }
  - [ ] Worker 只能在 git_branch 上工作
  - [ ] feedback_from_last_iteration: Option<EvaluationResult>
  - [ ] 测试：接收 Goal 成功、git_branch 验证

### Story 3.4: 实现 Worker 进度汇报接口（可选）
- **FR 覆盖**：FR-9
- **估时**：2h
- **验收**：
  - [ ] POST /v1/worker/goal/progress 端点
  - [ ] Request: { execution_id, goal_id, progress_pct, current_action, tokens_used, files_modified }
  - [ ] Response: { status: "ok" }
  - [ ] emit goal.progress 事件
  - [ ] 测试：进度汇报成功

### Story 3.5: 实现 Worker 返回结果接口
- **FR 覆盖**：FR-10
- **估时**：4h
- **验收**：
  - [ ] POST /v1/worker/goal/result 端点
  - [ ] Request: { execution_id, goal_id, success, output, tokens_used, files_modified, tool_calls, git_commit_hash }
  - [ ] Response: { status: "received", next_action: "evaluating" }
  - [ ] 编排器收到结果后自动进入评估阶段
  - [ ] git_commit_hash 用于 Checkpoint 回溯
  - [ ] 测试：返回结果成功、触发评估

### Story 3.6: 实现 Rust Worker SDK
- **FR 覆盖**：FR-14
- **估时**：8h
- **验收**：
  - [ ] acp-worker-sdk crate
  - [ ] WorkerConfig struct: worker_id, worker_type, orchestrator_url, skills, max_concurrency
  - [ ] Worker::new(config) 创建 Worker 实例
  - [ ] worker.on_goal(|goal| async { ... }) 注册 Goal 处理函数
  - [ ] worker.start().await 启动 Worker（自动注册、心跳、接收 Goal）
  - [ ] SDK 处理 HTTP 通信（注册、心跳、结果返回）
  - [ ] SDK 处理重试逻辑（网络失败时自动重试）
  - [ ] SDK 提供日志（info 级别记录关键事件）
  - [ ] 示例：创建一个 echo Worker，接收 Goal 并返回固定输出

### Story 3.7: 实现 TypeScript Worker SDK
- **FR 覆盖**：FR-15
- **估时**：8h
- **验收**：
  - [ ] @acp-swarm/worker-sdk npm 包
  - [ ] WorkerConfig interface
  - [ ] new Worker(config) 创建 Worker 实例
  - [ ] worker.onGoal(async (goal) => { ... }) 注册 Goal 处理函数
  - [ ] await worker.start() 启动 Worker
  - [ ] SDK 支持 Node.js 18+ 和 Deno
  - [ ] SDK 提供 TypeScript 类型定义
  - [ ] SDK 处理 HTTP 通信和重试逻辑
  - [ ] 示例：创建一个 echo Worker

**Epic 3 总计：33h（约 4 天）**

---

## Epic 4: .goal 解析器

**目标**：实现 .goal YAML 文件解析器，提供示例库。

**验收标准**：
- .goal 文件可以正确解析为 GoalGraph
- 解析器验证 executor 和 depends_on
- 4 个示例文件可以跑通

### Story 4.1: 定义 .goal YAML Schema
- **FR 覆盖**：FR-11
- **估时**：4h
- **验收**：
  - [ ] GoalFile struct: name, description, version, workers, goals, top
  - [ ] WorkerDecl: id, type, fallback
  - [ ] GoalDecl: id, executor, description, condition, evaluator, depends_on, budget, max_iter, timeout_ms, checkpoint
  - [ ] TopGoal: condition, evaluator
  - [ ] 8 种 CompletionCondition 的 YAML 表示
  - [ ] 4 种 Evaluator 的 YAML 表示
  - [ ] serde 序列化/反序列化测试

### Story 4.2: 实现 .goal 文件解析器
- **FR 覆盖**：FR-12
- **估时**：8h
- **验收**：
  - [ ] goal-parser crate
  - [ ] parse_goal_file(path) -> Result<GoalGraph, GoalParseError>
  - [ ] 验证 executor 必须是 workers 中声明的 Worker ID
  - [ ] 验证 depends_on 不能形成循环依赖（拓扑排序）
  - [ ] 为每个 Goal 分配唯一 git_branch（格式：worker/{worker_id}/goal/{goal_id}）
  - [ ] GoalParseError: line, column, message
  - [ ] 测试：解析成功、executor 验证失败、循环依赖检测

### Story 4.3: 创建 .goal 示例库
- **FR 覆盖**：FR-13
- **估时**：4h
- **验收**：
  - [ ] examples/goals/hello-swarm.goal（5 分钟快速开始，echo mock Worker）
  - [ ] examples/goals/fix-typescript-errors.goal
  - [ ] examples/goals/generate-api-docs.goal
  - [ ] examples/goals/refactor-module.goal
  - [ ] 每个示例包含 README 说明预期行为和 Token 消耗
  - [ ] hello-swarm.goal 可以用 echo mock Worker 跑通

**Epic 4 总计：16h（约 2 天）**

---

## Epic 5: Dashboard + 事件流

**目标**：实现 Web Dashboard 和事件流系统。

**验收标准**：
- Dashboard 显示 Worker 卡片、Goal 进度、Event Log
- 事件流通过 WebSocket 实时推送
- Token 预算追踪和验证工作正常

### Story 5.1: 实现事件类型定义（20+ 种）
- **FR 覆盖**：FR-25
- **估时**：6h
- **验收**：
  - [ ] AcpEvent struct: event_type, timestamp, task_id, data
  - [ ] Worker 生命周期：worker.registered, worker.heartbeat, worker.disconnected, worker.reconnected
  - [ ] Goal 生命周期：goal.submitted, goal.active, goal.progress, goal.evaluating, evaluation.complete, goal.iterating, goal.converged, goal.failed, goal.blocked
  - [ ] Checkpoint: checkpoint.created, rollback.started, rollback.complete
  - [ ] Swarm: swarm.started, swarm.converged, swarm.failed
  - [ ] Queen: queen.elected, queen.failover
  - [ ] 所有事件通过 AcpMessageBus 广播
  - [ ] 所有事件通过 EventStore 持久化

### Story 5.2: 实现指标收集
- **FR 覆盖**：FR-26
- **估时**：4h
- **验收**：
  - [ ] 收集：Worker 数量、Goal 成功率、平均迭代轮次、平均 Token 消耗、平均执行时长
  - [ ] 指标每 10 秒聚合一次
  - [ ] /metrics 端点暴露（Prometheus 格式）
  - [ ] 测试：指标收集和暴露

### Story 5.3: 实现 Token 预算追踪
- **FR 覆盖**：FR-27
- **估时**：2h
- **验收**：
  - [ ] 追踪每个 Goal 的 token_budget, tokens_used, remaining_budget()
  - [ ] 每轮迭代累加 tokens_used
  - [ ] tokens_used >= token_budget → BudgetExhausted
  - [ ] Dashboard 显示 Token 消耗进度条

### Story 5.4: 实现 Token 预算验证
- **FR 覆盖**：FR-28
- **估时**：4h
- **验收**：
  - [ ] CLI Worker（Codex/Claude Code）：编排引擎从 CLI 输出中解析 token 使用量
  - [ ] HTTP Worker：Worker SDK 自动报告，编排引擎交叉验证 API 响应
  - [ ] Token 消耗不一致时记录警告日志
  - [ ] 测试：CLI Worker token 解析、HTTP Worker 交叉验证

### Story 5.5: 实现 Worker 卡片展示
- **FR 覆盖**：FR-21
- **估时**：6h
- **验收**：
  - [ ] Worker 卡片显示：ID、类型、健康状态、Queen 徽章、负载、资源使用、操作按钮
  - [ ] WebSocket 实时更新（worker.heartbeat 事件驱动）
  - [ ] 点击 Worker 卡片显示详细信息（skills、iteration_log）
  - [ ] 测试：Worker 卡片渲染、实时更新

### Story 5.6: 实现 Goal 进度展示
- **FR 覆盖**：FR-22
- **估时**：6h
- **验收**：
  - [ ] Goal 进度显示：ID、描述、状态、迭代轮次、Token 进度条、执行者、评估器
  - [ ] Token 进度条颜色：< 50% 绿色, 50-80% 黄色, > 80% 红色
  - [ ] 实时 diff 展示：Goal Converged 时显示文件变更（green additions, red deletions）
  - [ ] WebSocket 推送后 500ms 内渲染
  - [ ] 测试：Goal 进度渲染、diff 展示

### Story 5.7: 实现 Event Log 展示
- **FR 覆盖**：FR-23
- **估时**：4h
- **验收**：
  - [ ] Event Log 显示：时间戳、事件类型、事件详情（JSON）
  - [ ] 保留最近 1000 条
  - [ ] 按事件类型过滤
  - [ ] 导出 JSONL
  - [ ] 测试：Event Log 渲染、过滤、导出

### Story 5.8: 实现执行任务界面
- **FR 覆盖**：FR-24
- **估时**：6h
- **验收**：
  - [ ] Goal 输入（描述或上传 .goal 文件）
  - [ ] 拓扑选择（Star/Chain）
  - [ ] Execute 按钮
  - [ ] 点击 Execute 后显示 Goal 进度
  - [ ] 执行完成后显示结果摘要
  - [ ] 支持取消正在执行的 Goal
  - [ ] 测试：执行任务、取消任务

**Epic 5 总计：32h（约 4 天）**

---

## Epic 6: Skill 路由 + Checkpoint + Hook

**目标**：实现 Skill 路由系统、Checkpoint/Rollback、Hook 扩展。

**验收标准**：
- Worker 可以注册 Skill，编排引擎可以根据 Skill 路由任务
- Checkpoint 可以创建和恢复
- 16 种 Hook 类型可以触发和执行

### Story 6.1: 实现 Worker 能力注册
- **FR 覆盖**：FR-29
- **估时**：4h
- **验收**：
  - [ ] Worker 注册时必须提交 WorkerCapabilityManifest
  - [ ] WorkerCapabilityManifest: worker_id, worker_type, skills: Vec<WorkerSkill>, max_concurrency, max_context_tokens, output_formats
  - [ ] WorkerSkill: name, proficiency, success_rate, avg_duration_ms, required_permissions
  - [ ] 注册信息存储到 SwarmSkillRouter
  - [ ] 测试：Worker 能力注册成功

### Story 6.2: 实现 Skill 路由算法
- **FR 覆盖**：FR-30
- **估时**：6h
- **验收**：
  - [ ] SwarmSkillRouter.route(skill_name) -> Option<SkillRouteDecision>
  - [ ] 评分公式：proficiency * 0.4 + success_rate * 0.4 + (1/load) * 0.2
  - [ ] EWMA 更新（alpha=0.1）：每次执行后更新 success_rate 和 avg_duration_ms
  - [ ] SkillRouteDecision: target_worker, skill_name, confidence, reason
  - [ ] 测试：code-review → Claude Code, bug-fixing → Codex, code-generation → best score

### Story 6.3: 实现 Checkpoint 创建
- **FR 覆盖**：FR-31
- **估时**：6h
- **验收**：
  - [ ] Checkpoint struct: id, workflow_id, stage_id, created_at, inputs, outputs, file_changes, executor_worker, success, duration_ms
  - [ ] FileChange: path, change_type (Created/Modified/Deleted), before_hash, before_content, after_hash
  - [ ] Stage 成功后自动创建（如果 checkpoint: on_converge）
  - [ ] 文件变更通过 git diff 捕获
  - [ ] Checkpoint 持久化到 EventStore
  - [ ] 测试：Checkpoint 创建、文件变更捕获

### Story 6.4: 实现 Rollback 执行
- **FR 覆盖**：FR-32
- **估时**：6h
- **验收**：
  - [ ] RollbackStrategy enum: FileSystemOnly, FileSystemAndRerun, DataOnly
  - [ ] rollback(checkpoint_id, strategy) -> Result<RollbackResult, String>
  - [ ] FileSystemOnly: 恢复文件到 Checkpoint 状态
  - [ ] FileSystemAndRerun: 恢复文件 + 标记后续 Stage 需重新执行
  - [ ] DataOnly: 恢复数据（inputs/outputs）
  - [ ] RollbackResult: success, rolled_back_checkpoint, files_restored, files_failed, stages_to_rerun, errors
  - [ ] 测试：3 种策略的 Rollback

### Story 6.5: 实现 HookType 扩展（16 种）
- **FR 覆盖**：FR-33
- **估时**：6h
- **验收**：
  - [ ] 原有 4 种：PreToolUse, PostToolUse, PostToolUseFailure, Stop
  - [ ] Worker 级别：PreTask, PostTask, OnTaskTimeout, OnWorkerJoin, OnWorkerLeave, OnWorkerError
  - [ ] Workflow 级别：PreStage, PostStage, OnStageFailure, OnCheckpoint
  - [ ] Swarm 级别：OnSwarmStart, OnSwarmComplete, OnSwarmFailure, OnQueenElected, OnQueenFailover
  - [ ] HookScope enum: Worker(String), Stage(String), Swarm, Global
  - [ ] HookRegistration struct: id, name, hook_type, scope, config, priority, enabled
  - [ ] HookContext struct: event_type, worker_id, task_id, stage_id, workflow_id, data, timestamp
  - [ ] 测试：16 种 Hook 类型注册和触发

### Story 6.6: 实现 4 个内置 Hook
- **FR 覆盖**：FR-34
- **估时**：6h
- **验收**：
  - [ ] worker-auto-failover: Worker 连续 3 次失败 → Evict + 任务重分配
  - [ ] stage-quality-gate: Stage 输出为空 → 阻止进入下一 Stage
  - [ ] auto-checkpoint: Stage 成功 → 自动创建检查点
  - [ ] token-budget-guard: 任务超出预算 → 拦截
  - [ ] 每个 Hook 包含详细的配置选项
  - [ ] 测试：4 个内置 Hook 的触发和执行

**Epic 6 总计：34h（约 4.5 天）**

---

## Epic 7: ACP 消息总线 + Tool 沙箱

**目标**：实现 AcpMessageBus 和 ToolSandbox。

**验收标准**：
- 所有消息通过 AcpMessageBus 路由
- TS 和 Rust 端的消息格式完全对齐
- Worker 的工具执行必须通过沙箱

### Story 7.1: 实现 AcpMessageBus 完整功能
- **FR 覆盖**：FR-35
- **估时**：8h
- **验收**：
  - [ ] AcpMessage struct: type, version, id, timestamp, source_id, target_id, payload, correlation_id
  - [ ] 支持定向路由（target_id）和全局广播
  - [ ] 传输层适配器：TauriAdapter, StdioAdapter, WebSocketAdapter, HttpAdapter
  - [ ] 所有消息通过 AcpMessageBus 路由
  - [ ] TS 和 Rust 端的消息格式完全对齐（JSON Schema 验证）
  - [ ] 测试：消息路由、广播、适配器切换

### Story 7.2: 实现 ToolSandbox
- **FR 覆盖**：FR-36
- **估时**：6h
- **验收**：
  - [ ] SandboxConfig: allowed_paths, denied_patterns (默认 .env, .git, node_modules, .ssh), network_allowed, timeout_ms, max_output_bytes
  - [ ] SandboxResult: success, output, error, blocked_paths, duration_ms, timed_out
  - [ ] Worker 的工具执行必须通过沙箱
  - [ ] 拒绝的路径访问记录到审计日志
  - [ ] 测试：路径白名单、拒绝模式、超时控制

**Epic 7 总计：14h（约 2 天）**

---

## Epic 8: 测试矩阵

**目标**：建立 4 层测试体系。

**验收标准**：
- 单元测试覆盖率 70% lines
- 集成测试覆盖关键流程
- 稳定性测试验证长时间运行
- 安全测试验证权限和隔离

### Story 8.1: 建立测试分类和配置
- **FR 覆盖**：FR-37
- **估时**：4h
- **验收**：
  - [ ] 单元测试：crates/*/tests/ + src/tests/unit/ (30s 超时)
  - [ ] 集成测试：src/tests/integration/ (60s 超时)
  - [ ] 稳定性测试：src/tests/stability/ (120s 超时)
  - [ ] 安全测试：src/tests/security/ (30s 超时)
  - [ ] vitest.config.ts 配置
  - [ ] package.json scripts: test, test:unit, test:integration, test:stability, test:security, test:coverage
  - [ ] 覆盖率目标：70% lines, 60% functions, 50% branches

### Story 8.2: 编写关键测试用例
- **FR 覆盖**：FR-37
- **估时**：12h
- **验收**：
  - [ ] Skill Router: code-review → Claude Code, bug-fixing → Codex, code-generation → best score
  - [ ] Checkpoint: create → rollback → verify files restored
  - [ ] Reconcile Loop: iteration 1 fails → iteration 2 converges
  - [ ] Oscillation Detection: same feedback 3 times → Queen intervenes
  - [ ] Queen Failover: lease expired → new Queen elected → old Queen rejected
  - [ ] 每个测试用例包含详细的断言和注释
  - [ ] 测试通过率 100%

### Story 8.3: 稳定性测试和安全测试
- **FR 覆盖**：FR-37
- **估时**：8h
- **验收**：
  - [ ] 稳定性测试：100 个 Goal 连续执行（内存泄漏检测）
  - [ ] 稳定性测试：Worker 失联 → 故障转移 → 恢复
  - [ ] 安全测试：Tool 沙箱 .env 访问拦截
  - [ ] 安全测试：Worker 隔离（Worker A 不能访问 Worker B 的数据）
  - [ ] 测试通过

**Epic 8 总计：24h（约 3 天）**

---

## Epic 9: 开源策略

**目标**：建立开源项目的文档体系和社区钩子。

**验收标准**：
- README 包含 What + Why + One command
- 文档体系完整（getting-started, protocols, guides, verticals, api-reference, rfcs）
- 贡献者体验良好（CONTRIBUTING.md, Worker Adapter Template, good first issue）
- 60 秒 demo 视频

### Story 9.1: 编写 README
- **FR 覆盖**：FR-39
- **估时**：4h
- **验收**：
  - [ ] 前 3 行：What + Why + One command to try it
  - [ ] 30 秒 demo GIF 嵌入
  - [ ] scripts/check-prereqs.sh 预检脚本
  - [ ] Badges: Build status, License, Discord
  - [ ] 安装指南（cargo install acp-swarm）
  - [ ] 快速开始（5 分钟跑通 hello-swarm.goal）

### Story 9.2: 建立文档体系
- **FR 覆盖**：FR-40
- **估时**：8h
- **验收**：
  - [ ] docs/getting-started.md（5 分钟跑通 hello-swarm.goal）
  - [ ] docs/protocols/（4 个协议规范：worker-protocol, goal-protocol, event-protocol, embed-protocol）
  - [ ] docs/guides/（使用指南：build-a-worker, write-goals, goal-templates, build-a-dashboard, git-workspace-isolation）
  - [ ] docs/verticals/（垂直领域示例：coding-agent-swarm, creative-pipeline, ml-training-loop, game-dev-workflow）
  - [ ] docs/api-reference/（API 参考：cli, rust-sdk, typescript-sdk）
  - [ ] docs/rfcs/（设计文档：001-goal-driven-orchestration, 002-distributed-patterns, 003-learning-loop）

### Story 9.3: 改善贡献者体验
- **FR 覆盖**：FR-41
- **估时**：6h
- **验收**：
  - [ ] CONTRIBUTING.md（贡献流程）
  - [ ] Worker Adapter Template（cargo generate 模板）
  - [ ] 5 个 "good first issue" 标签
  - [ ] #showcase Discord 频道
  - [ ] CODE_OF_CONDUCT.md
  - [ ] PULL_REQUEST_TEMPLATE.md

### Story 9.4: 制作 demo 视频
- **FR 覆盖**：FR-42
- **估时**：4h
- **验收**：
  - [ ] 60 秒维护者 demo 视频
  - [ ] 视频结构：Problem → One command → Success
  - [ ] YouTube 上传
  - [ ] README 嵌入

**Epic 9 总计：22h（约 3 天）**

---

## 总结

| Epic | Story 数 | 总工时 | 天数 |
|------|---------|--------|------|
| Epic 1: 仓库初始化与 Crate 拆分 | 8 | 34h | 4.5 天 |
| Epic 2: 核心引擎实现 | 10 | 58h | 7.5 天 |
| Epic 3: Worker Protocol + SDK | 7 | 33h | 4 天 |
| Epic 4: .goal 解析器 | 3 | 16h | 2 天 |
| Epic 5: Dashboard + 事件流 | 8 | 32h | 4 天 |
| Epic 6: Skill 路由 + Checkpoint + Hook | 6 | 34h | 4.5 天 |
| Epic 7: ACP 消息总线 + Tool 沙箱 | 2 | 14h | 2 天 |
| Epic 8: 测试矩阵 | 3 | 24h | 3 天 |
| Epic 9: 开源策略 | 4 | 22h | 3 天 |
| **总计** | **51** | **267h** | **34.5 天** |

**FR 覆盖**：42/42（100%）

**执行顺序建议**：
1. Epic 1（仓库初始化）— 必须先做
2. Epic 2（核心引擎）— 最核心，依赖 Epic 1
3. Epic 4（.goal 解析器）— 依赖 Epic 2
4. Epic 3（Worker Protocol + SDK）— 依赖 Epic 2
5. Epic 7（ACP 消息总线 + Tool 沙箱）— 依赖 Epic 1
6. Epic 5（Dashboard + 事件流）— 依赖 Epic 2, 7
7. Epic 6（Skill 路由 + Checkpoint + Hook）— 依赖 Epic 2, 3
8. Epic 8（测试矩阵）— 依赖所有 Epic
9. Epic 9（开源策略）— 最后做

**关键路径**：Epic 1 → Epic 2 → Epic 3/4 → Epic 5/6 → Epic 8

---

*Created: 2026-06-09*
*Based on: PRD v2.0.0, Architecture v1.0.0*
