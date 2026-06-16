# acp-swarm 总纲：异构 Agent 蜂群编排引擎

> **版本**: 2.0.0（合并版）
> **最后更新**: 2026-06-11
> **来源**: 合并自 `新仓库建设计划.md` + `总纲.md` + `战略聚焦.md`
> **定位**: Agent 的 Kubernetes，AI 的浏览器
> **一句话**: acp-swarm 不做 Agent，它让 Agent 像团队一样协作。

---

## 一、项目定位

### 1.1 acp-swarm 是什么

acp-swarm 是一个**异构 Agent 蜂群编排引擎**。它不直接执行任何任务——不写代码、不写小说、不训练模型、不生成图片。它只做一件事：**让多个 Agent 按照目标（Goal）协作，自动迭代直到目标达成。**

类比：

| 类比对象 | 它做什么 | 它不做什么 |
|---------|---------|----------|
| 浏览器 | 渲染 HTML/CSS/JS | 不生产网页内容 |
| Kubernetes | 编排容器 | 不构建应用 |
| **acp-swarm** | **编排 Agent** | **不执行具体任务** |

### 1.2 核心原则

1. **领域无关**: 编排引擎不包含任何领域逻辑。它不知道什么是"代码"、什么是"小说"。它只知道 Goal、Worker、CompletionCondition。
2. **协议优先**: 一切通过协议交互。新增一个垂直领域不需要改引擎代码，只需要实现协议。
3. **Goal 驱动**: 不给 Agent 分配"任务"（固定步骤），给 Agent 设定"目标"（完成条件）。Agent 自动迭代直到目标达成。
4. **执行器/评估器分离**: 干活的 Worker 和判断"做完了没"的评估器是不同角色。
5. **可观测性**: 系统的所有状态通过统一事件流暴露。

---

## 二、协议栈规范

### 2.1 Worker Protocol v1 —— Agent 怎么接入蜂群

**任何 Agent，只要实现以下 5 个接口，就能成为 acp-swarm 的 Worker。**

#### 2.1.1 注册

```json
POST /acp/worker/register

{
  "worker_id": "novel-writer-01",
  "worker_type": "novel-llm",
  "version": "1.0",
  "capabilities": {
    "skills": [
      { "name": "chapter-writing", "proficiency": 0.9, "avg_duration_ms": 30000 }
    ],
    "max_concurrency": 2,
    "max_context_tokens": 200000
  }
}

Response: { "status": "registered", "heartbeat_interval_ms": 10000 }
```

#### 2.1.2 心跳

```json
POST /acp/worker/heartbeat

{
  "worker_id": "novel-writer-01",
  "status": "idle",
  "current_load": 0,
  "active_goals": []
}

Response: { "status": "ok" }
```

Worker 必须每 `heartbeat_interval_ms` 毫秒发一次心跳。编排器连续 3 次未收到心跳，判定 Worker 失联。

#### 2.1.3 接收 Goal

```json
POST /acp/worker/goal/receive

{
  "goal_id": "goal-chapter-01",
  "description": "根据大纲生成第一章",
  "iteration": 0,
  "feedback_from_last_iteration": null,
  "workspace_path": "/workspace/novel-project/",
  "token_budget": 50000,
  "timeout_ms": 300000
}

Response: { "status": "accepted", "execution_id": "exec-001" }
```

#### 2.1.4 返回结果

```json
POST /acp/worker/goal/result

{
  "execution_id": "exec-001",
  "goal_id": "goal-chapter-01",
  "success": true,
  "output": "第一章生成完毕，共 3200 字",
  "tokens_used": 32000,
  "files_modified": ["chapter-01/scene-01.md"]
}

Response: { "status": "received", "next_action": "evaluating" }
```

---

### 2.2 Goal Protocol v1 —— 目标怎么定义

详见 [RFC-001-Goal驱动架构.md](./RFC-001-Goal驱动架构.md)。

核心结构：

```yaml
goals:
  - id: goal-001
    description: "Fix all TypeScript compilation errors"
    completion_condition:
      type: command_success
      command: "npx vue-tsc --noEmit"
    evaluator: auto
    executor: claude-worker
    depends_on: []
    token_budget: 80000
    max_iterations: 5
```

---

### 2.3 Event Protocol v1 —— 状态怎么推送

所有 UI 从同一个事件流消费数据：

| 传输方式 | 适用场景 |
|---------|---------|
| WebSocket | Web Dashboard |
| Server-Sent Events | 轻量级客户端 |
| Tauri Events | 桌面应用内部 |

事件类型示例：

```json
{ "type": "goal.submitted", "goal_id": "goal-001" }
{ "type": "goal.active", "iteration": 0 }
{ "type": "goal.evaluating", "evaluator": "auto" }
{ "type": "goal.converged", "iterations": 3 }
{ "type": "goal.failed", "reason": "budget_exhausted" }
{ "type": "worker.disconnected", "worker_id": "codex-01" }
{ "type": "queen.elected", "queen_id": "claude-01" }
```

---

## 三、仓库架构

### 3.1 Cargo Workspace

```
acp-swarm/
├── Cargo.toml              # workspace 根
├── crates/
│   ├── acp-core/           # ACP 协议定义 + 消息类型（零依赖）
│   ├── acp-transport/      # 传输层（stdio/WebSocket/HTTP）
│   ├── swarm-engine/       # 蜂群编排引擎（Goal分解/路由/Reconcile）
│   ├── workflow-engine/    # 工作流引擎（DAG/检查点/回溯）
│   ├── hook-runtime/       # Hook 执行器
│   ├── tool-sandbox/       # Tool 执行沙箱
│   └── acp-cli/            # CLI 入口
├── src/                    # 前端 UI（Vue 3 + Vite）
├── src-tauri/              # Tauri 薄壳（只做命令注册）
├── sdk/
│   ├── rust/               # Rust Worker SDK
│   ├── python/             # Python Worker SDK
│   └── typescript/         # TypeScript Worker SDK
├── rfcs/                   # RFC 文档
├── examples/               # .goal 文件示例库
└── tests/                  # 测试
```

### 3.2 crate 依赖关系

```
acp-core          ← 零内部依赖
  ↑
acp-transport     ← acp-core
event-bus         ← acp-core
tool-sandbox      ← acp-core
  ↑
swarm-engine      ← acp-core + acp-transport + event-bus
workflow-engine   ← acp-core + event-bus
hook-runtime      ← acp-core + event-bus
  ↑
acp-cli           ← 所有 crate
src-tauri         ← 所有 crate
```

---

## 四、分布式系统设计模式映射

### 4.1 Elasticsearch 分片 → 任务分区与副本容错

- **任务分区**: Queen 将大任务拆分为 Task Shard
- **Primary/Replica Worker**: 关键任务有副本，Primary失败时Replica接管
- **Shard 再平衡**: 新Worker加入时重新分配任务

### 4.2 Redis Cluster 哈希槽 → 能力路由

- **能力槽位**: 根据 Worker 声明的能力定义槽位
- **一致性哈希路由**: 任务通过哈希分配到不同Worker实例
- **槽位迁移**: 新Worker加入时动态迁移

### 4.3 ZooKeeper 租约选举 → 防脑裂

- **Queen 租约**: 30秒租约 + 10秒续约
- **自动选举**: 租约过期后自动选举新Queen
- **防脑裂**: Worker只接受租约有效的Queen指令

---

## 五、Git Workspace 隔离

每个 Worker 在独立分支工作，Goal Converged 后合并：

```
main (合并后的代码)
  ├── worker/claude-01/goal-001
  ├── worker/codex-01/goal-002
  └── worker/gemini-01/goal-003
```

---

## 六、学习闭环

### 6.1 执行历史记录

每次 Goal 执行完成后记录到 SQLite，用于自动校准 Worker 能力评分。

### 6.2 自动校准

```rust
// 根据历史执行数据自动校准 Worker 能力评分
pub fn recalibrate_worker_skills(&self) {
    // EWMA: success_rate * (1 / avg_iterations)
}
```

---

## 七、文档体系

```
docs/
├── getting-started.md        # 5分钟跑通 demo
├── protocols/                # 四个协议规范
├── guides/                   # 使用指南
├── verticals/                # 垂直领域示例
├── api-reference/            # API 参考
└── rfcs/                     # 设计文档
```

---

## 参考

- [RFC-001-Goal驱动架构.md](./RFC-001-Goal驱动架构.md) — Goal 数据结构与 Reconcile Loop
- [执行计划.md](./执行计划.md) — 统一的 Phase 划分与任务追踪