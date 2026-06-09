# 下一个迭代规划：蜂群编排核心实现

> **目标**: 跑通一个真实的异构 Agent 蜂群 Demo
> **时间**: 2026-06-09 - 2026-06-12 (4天)
> **战略对齐**: 完成战略文档 Phase 1-2 的核心部分

---

## 一、战略对齐检查

**判断标准**: 这个改动是在加固编排层的地基，还是在外墙刷漆？

| 任务 | 战略价值 | 优先级 |
|------|----------|--------|
| Rust Worker 适配器 | 地基 - 真实通信管道 | P0 |
| 任务分片 + 副本容错 | 地基 - 容错机制 | P0 |
| Queen 租约选举 | 地基 - 高可用 | P0 |
| 前端蜂群面板改造 | 展示层 | P1 |
| Token 优化 | 成本优化 (Phase 4+) | P2 |
| 自进化 | 增强 (Phase 5+) | P2 |

---

## 二、核心任务清单

### Day 1: Rust Worker 适配器 (2026-06-09)

**目标**: 实现 stdio 进程通信，注册 Codex/Claude Code 为真实 Worker

#### Task 1.1: SwarmAgentAdapter Trait

文件: `src-tauri/src/swarm_adapters/mod.rs`

```rust
pub trait SwarmAgentAdapter: Send + Sync {
    fn send_task(&self, task: &TaskDescription) -> Result<TaskHandle, SwarmError>;
    fn get_status(&self) -> Result<WorkerStatus, SwarmError>;
    fn cancel_task(&self, task_id: &str) -> Result<(), SwarmError>;
    fn capabilities(&self) -> WorkerCapabilities;
    fn health_check(&self) -> bool;
}
```

#### Task 1.2: CodexAdapter 实现

文件: `src-tauri/src/swarm_adapters/codex.rs`

- 通过 stdio 启动 `codex --full-auto` 进程
- 实时读取 stdout/stderr
- 进程状态管理 (PID, 内存, CPU)

#### Task 1.3: ClaudeCodeAdapter 实现

文件: `src-tauri/src/swarm_adapters/claude_code.rs`

- 通过 stdio 启动 `claude --print` 进程
- 处理交互式输出

#### Task 1.4: Tauri 命令扩展

修改: `src-tauri/src/lib.rs`

新增命令:
- `swarm_register_worker`
- `swarm_list_workers`
- `swarm_send_task`
- `swarm_get_worker_status`
- `swarm_health_check`

**验收**: `cargo check` 通过。通过 Tauri 命令能启动 Codex 进程并获取输出。

---

### Day 2: 任务分片 + 副本容错 (2026-06-10)

**目标**: ES 分片思想的落地

#### Task 2.1: TaskShard 结构体

文件: `src-tauri/src/swarm_types.rs`

```rust
pub struct TaskShard {
    pub id: String,
    pub parent_task: String,
    pub payload: TaskPayload,
    pub primary_worker: WorkerId,
    pub replica_worker: Option<WorkerId>,
    pub status: ShardStatus,
    pub timeout_ms: u64,
}
```

#### Task 2.2: 任务分解器

文件: `src-tauri/src/task_partitioner.rs`

- 让 Queen 分析任务并拆分为子任务
- 解析 JSON 格式的分解结果

#### Task 2.3: 副本容错逻辑

文件: `src-tauri/src/swarm_orchestrator.rs`

- Primary Worker 超时 → 切换 Replica
- Worker 崩溃 → 自动找替代
- 失败重试策略 (最多 3 次)

**验收**: 能拆分一个"写登录页面"任务为 3 个子任务，Primary 失败后自动切换 Replica。

---

### Day 3: Queen 租约选举 (2026-06-11)

**目标**: ZK 租约思想的落地

#### Task 3.1: QueenLease 结构体

文件: `src-tauri/src/queen_lease.rs`

```rust
pub struct QueenLease {
    pub queen_id: WorkerId,
    pub granted_at: Instant,
    pub ttl_seconds: u64,        // 默认 30 秒
    pub renew_interval: u64,     // 每 10 秒续约
}
```

#### Task 3.2: 自动选举逻辑

- Queen 租约过期 → 按能力排序选举新 Queen
- 防脑裂：Worker 只接受有效租约的 Queen 指令

#### Task 3.3: Adaptive Queen 升级

- 任务复杂度超出 Queen 能力 → 自动切换更强的 Queen

**验收**: Queen 进程崩溃后，30 秒内自动选举新 Queen。

---

### Day 4: 前端蜂群面板 + Demo (2026-06-12)

**目标**: 可视化蜂群执行过程

#### Task 4.1: SwarmDashboard 改造

修改: `src/views/SwarmDashboard.vue` 或新建

核心 UI:
1. Worker 注册区 (下拉选择 Codex/Claude Code)
2. 拓扑选择器 (Star/Chain 可视化)
3. 任务输入区
4. 实时状态面板 (每个 Worker 卡片)
5. 结果展示区

#### Task 4.2: 连接前端 store

- 使用已有的 `swarm-orchestrator.ts`
- 补充真实 Tauri 命令调用

#### Task 4.3: Demo 场景

任务: "帮我写一个 React 登录页面"

流程:
1. Queen (Claude Code) 拆分任务
2. Worker A (Codex) → LoginPage.tsx
3. Worker B (Claude Code) → validation.ts
4. Worker C (Codex) → LoginPage.test.tsx
5. Queen 合并结果

**验收**: 在 UI 上能实时看到 3 个 Worker 并行工作，最终返回生成的文件。

---

## 三、验收标准

只有一个：

> **在 SwarmDashboard 中，注册 Codex 和 Claude Code 为 Worker，输入真实开发任务，蜂群并行执行并返回正确结果。全程 UI 能看到每个 Worker 状态。**

---

## 四、不做的事 (Phase 2+)

- Token 优化 (KV Cache 模型路由)
- 自进化 Skill 系统
- IM 远程控制
- 云端 Worker (只做本地)
- Mesh/Tree/Ring 拓扑 (只做 Star + Chain)

---

## 五、技术风险

| 风险 | 对策 |
|------|------|
| Codex CLI 不可用 | 用 `echo` 模拟假 Worker，但通信管道真实 |
| stdio 进程管理复杂 | 使用 Rust `async-process` crate |
| 前端状态同步 | Tauri event + Pinia store |

---

## 六、Git 提交计划

```
Day 1: feat: swarm worker adapters - Codex/Claude Code stdio communication
Day 2: feat: task partitioning + replica fault tolerance - ES sharding pattern
Day 3: feat: queen lease election - ZK lease pattern for HA
Day 4: feat: swarm dashboard + demo - visual hive execution
```