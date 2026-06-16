---
title: acp-swarm 系统架构设计
status: final
created: 2026-06-09
updated: 2026-06-09
project: acp-swarm
version: 1.0.0
---

# acp-swarm 系统架构设计

*基于 PRD v2.0.0-mvp（42 个 FR）的完整技术架构。*

---

## 0. Architecture Overview

### 0.1 设计原则

1. **分层解耦**：协议层 / 引擎层 / 传输层 / 存储层 严格分离
2. **Trait 抽象**：所有外部依赖（存储、传输、Worker 通信）通过 trait 抽象，便于测试和替换
3. **事件驱动**：所有状态变更 emit 事件，通过 `AcpMessageBus` 广播
4. **Workspace 拆分**：9 个独立 crate，每个职责单一，依赖关系单向

### 0.2 架构分层图

```
┌──────────────────────────────────────────────────────────────────┐
│                    CLI / Dashboard / Embed                        │
│              (acp-cli / Vue Dashboard / iframe)                  │
├──────────────────────────────────────────────────────────────────┤
│                    Orchestration Layer                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ swarm-engine │  │workflow-engine│  │   hook-runtime       │   │
│  │ (Goal/Reconcile│  │ (DAG/Checkpoint│  │ (16 HookTypes +     │   │
│  │  /Topology)  │  │  /Approval)   │  │  4 Built-in Hooks)  │   │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘   │
│         │                 │                      │               │
├─────────┼─────────────────┼──────────────────────┼───────────────┤
│         │         Core Layer (acp-core)          │               │
│  ┌──────┴───────┐  ┌──────┴───────┐  ┌──────────┴───────────┐   │
│  │   goal.rs    │  │  skill.rs    │  │   db/events.rs       │   │
│  │ (Goal/       │  │ (WorkerSkill/│  │ (EventStore trait +  │   │
│  │  Completion/ │  │  SwarmSkill/ │  │  SQLite impl)        │   │
│  │  Evaluator)  │  │  Router)     │  │                      │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
├──────────────────────────────────────────────────────────────────┤
│                    Transport Layer                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │acp-transport │  │  event-bus   │  │   tool-sandbox       │   │
│  │ (stdio/WS/   │  │ (AcpMessage/ │  │ (SandboxConfig/      │   │
│  │  HTTP/Tauri) │  │  Bus/Adapters│  │  PathFilter/Audit)   │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
├──────────────────────────────────────────────────────────────────┤
│                    External Workers                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │  Codex CLI   │  │Claude Code   │  │   Custom Agent       │   │
│  │  (stdio)     │  │CLI (stdio)   │  │   (HTTP/WS)          │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### 0.3 Crate 依赖关系

```
acp-core (零内部依赖)
    ↑
    │
    ├──→ event-bus (acp-core)
    │
    ├──→ goal-parser (acp-core)
    │
    ├──→ tool-sandbox (acp-core)
    │
    ├──→ acp-transport (acp-core + event-bus)
    │
    ├──→ hook-runtime (acp-core + event-bus)
    │
    ├──→ swarm-engine (acp-core + event-bus + goal-parser + acp-transport)
    │
    ├──→ workflow-engine (acp-core + event-bus)
    │
    └──→ acp-cli (所有 crate)
```

---

## 1. Crate 详细设计

### 1.1 acp-core

**职责**：核心类型定义 + 序列化 + 工具函数。零内部依赖。

**文件结构**：
```
crates/acp-core/src/
├── lib.rs              # 导出所有模块
├── goal.rs             # Goal, CompletionCondition, Evaluator, GoalStatus
├── skill.rs            # WorkerSkill, WorkerCapabilityManifest, SwarmSkillRouter
├── worker.rs           # WorkerStatus, WorkerLocation, HealthStatus
├── event.rs            # AcpEvent, EventType (20+ 种)
├── message.rs          # AcpMessage (统一消息格式)
├── checkpoint.rs       # Checkpoint, FileChange, RollbackStrategy
├── hook.rs             # HookType (16 种), HookScope, HookContext
├── sandbox.rs          # SandboxConfig, SandboxResult
├── error.rs            # SwarmError, GoalError, WorkerError
└── db/
    ├── mod.rs
    └── events.rs       # EventStore trait + SqliteEventStore
```

**核心类型定义**：

```rust
// goal.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub parent_task_id: String,
    pub parent_goal_id: Option<String>,
    pub completion_condition: CompletionCondition,
    pub evaluator: Evaluator,
    pub executor: String,  // Worker ID
    pub depends_on: Vec<String>,
    pub token_budget: u64,
    pub tokens_used: u64,
    pub max_iterations: u32,
    pub current_iteration: u32,
    pub per_iteration_timeout_ms: u64,
    pub status: GoalStatus,
    pub iteration_log: Vec<IterationRecord>,
    pub created_at: DateTime<Utc>,
    pub converged_at: Option<DateTime<Utc>>,
    pub output_files: Vec<String>,
    pub metadata: serde_json::Value,
}

impl Goal {
    pub fn is_terminal(&self) -> bool {
        matches!(self.status,
            GoalStatus::Converged
            | GoalStatus::BudgetExhausted
            | GoalStatus::MaxIterReached
            | GoalStatus::Failed(_)
            | GoalStatus::Cancelled
        )
    }

    pub fn remaining_budget(&self) -> u64 {
        self.token_budget.saturating_sub(self.tokens_used)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CompletionCondition {
    CommandSuccess {
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
        env: Option<std::collections::HashMap<String, String>>,
    },
    OutputContains {
        command: String,
        args: Vec<String>,
        pattern: String,
        case_sensitive: bool,
    },
    OutputMatches {
        command: String,
        args: Vec<String>,
        regex: String,
    },
    All { conditions: Vec<CompletionCondition> },
    Any { conditions: Vec<CompletionCondition> },
    FileCheck {
        path: String,
        content_contains: Option<String>,
        content_matches: Option<String>,
        max_size_bytes: Option<u64>,
    },
    HttpHealthCheck {
        url: String,
        method: String,
        expected_status: Option<u16>,
        body_contains: Option<String>,
    },
    QueenJudgment { criteria: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Evaluator {
    Auto,
    Queen,
    Adversarial { evaluator_worker: String },
    Hybrid {
        auto_condition: CompletionCondition,
        human_evaluator: Box<Evaluator>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Pending,
    Active,
    Evaluating,
    Converged,
    Iterating,
    BudgetExhausted,
    MaxIterReached,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    pub iteration: u32,
    pub execution_started_at: DateTime<Utc>,
    pub execution_finished_at: DateTime<Utc>,
    pub worker_output: String,
    pub tokens_used: u64,
    pub cumulative_tokens: u64,
    pub evaluation: EvaluationResult,
    pub evaluation_duration_ms: u64,
    pub tool_calls: Vec<String>,
    pub files_modified: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub converged: bool,
    pub feedback: String,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub duration_ms: u64,
    pub sub_results: Option<Vec<EvaluationResult>>,
}
```

```rust
// skill.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSkill {
    pub name: String,
    pub proficiency: f32,       // 0.0 - 1.0
    pub execution_count: u64,
    pub success_rate: f32,      // EWMA
    pub avg_duration_ms: u64,   // EWMA
    pub required_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilityManifest {
    pub worker_id: String,
    pub worker_type: String,
    pub skills: Vec<WorkerSkill>,
    pub max_concurrency: u32,
    pub max_context_tokens: u64,
    pub output_formats: Vec<String>,
}

pub struct SwarmSkillRouter {
    routes: std::collections::HashMap<String, Vec<(String, WorkerSkill)>>,
    worker_load: std::collections::HashMap<String, u32>,
}

impl SwarmSkillRouter {
    pub fn new() -> Self { /* ... */ }

    pub fn register_worker(&mut self, manifest: &WorkerCapabilityManifest) {
        for skill in &manifest.skills {
            self.routes
                .entry(skill.name.clone())
                .or_default()
                .push((manifest.worker_id.clone(), skill.clone()));
        }
        self.worker_load.insert(manifest.worker_id.clone(), 0);
    }

    pub fn route(&self, skill_name: &str) -> Option<SkillRouteDecision> {
        let candidates = self.routes.get(skill_name)?;
        let available: Vec<_> = candidates
            .iter()
            .filter(|(wid, _)| {
                self.worker_load.get(wid).copied().unwrap_or(u32::MAX) < 10
            })
            .collect();

        if available.is_empty() { return None; }

        // Score: proficiency * 0.4 + success_rate * 0.4 + (1/load) * 0.2
        let best = available
            .iter()
            .max_by(|(_, a), (_, b)| {
                let score_a = a.proficiency * 0.4 + a.success_rate * 0.4
                    + 1.0 / (self.worker_load.get(&a.name).copied().unwrap_or(1) as f32) * 0.2;
                let score_b = b.proficiency * 0.4 + b.success_rate * 0.4
                    + 1.0 / (self.worker_load.get(&b.name).copied().unwrap_or(1) as f32) * 0.2;
                score_a.partial_cmp(&score_b).unwrap()
            })?;

        Some(SkillRouteDecision {
            target_worker: best.0.clone(),
            skill_name: skill_name.to_string(),
            confidence: best.1.proficiency * best.1.success_rate,
            reason: "best_combined_score".to_string(),
        })
    }

    // EWMA 更新 (alpha = 0.1)
    pub fn record_execution(&mut self, worker_id: &str, skill_name: &str, success: bool, duration_ms: u64) {
        let alpha = 0.1;
        if let Some(entries) = self.routes.get_mut(skill_name) {
            for (wid, skill) in entries.iter_mut() {
                if wid == worker_id {
                    skill.execution_count += 1;
                    skill.success_rate = if success {
                        skill.success_rate * (1.0 - alpha) + alpha
                    } else {
                        skill.success_rate * (1.0 - alpha)
                    };
                    skill.avg_duration_ms = ((skill.avg_duration_ms as f64 * (1.0 - alpha))
                        + (duration_ms as f64 * alpha)) as u64;
                }
            }
        }
    }
}
```

```rust
// db/events.rs

use async_trait::async_trait;
use rusqlite::Connection;

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn store_event(&self, event: &crate::event::AcpEvent) -> Result<(), String>;
    async fn get_events(&self, filter: EventFilter) -> Result<Vec<crate::event::AcpEvent>, String>;
    async fn get_events_since(&self, since: chrono::DateTime<chrono::Utc>) -> Result<Vec<crate::event::AcpEvent>, String>;
}

pub struct SqliteEventStore {
    conn: Connection,
}

impl SqliteEventStore {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                task_id TEXT,
                data TEXT NOT NULL
            )",
            [],
        ).map_err(|e| e.to_string())?;
        Ok(Self { conn })
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn store_event(&self, event: &crate::event::AcpEvent) -> Result<(), String> {
        let data_json = serde_json::to_string(&event.data).map_err(|e| e.to_string())?;
        self.conn.execute(
            "INSERT INTO events (id, type, timestamp, task_id, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![event.id, event.event_type, event.timestamp.timestamp_millis(), event.task_id, data_json],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    // ... 其他方法实现
}
```

---

### 1.2 event-bus

**职责**：消息路由 + 事件广播 + 传输层适配器。

**文件结构**：
```
crates/event-bus/src/
├── lib.rs           # AcpMessageBus 主结构
├── adapters.rs      # TauriAdapter, StdioAdapter, WebSocketAdapter, HttpAdapter
└── schema.rs        # JSON Schema 验证
```

**核心实现**：
```rust
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct AcpMessageBus {
    broadcast: broadcast::Sender<crate::AcpEvent>,
    subscribers: Arc<RwLock<HashMap<String, broadcast::Sender<crate::AcpEvent>>>>,
}

impl AcpMessageBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            broadcast: tx,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn emit(&self, event: crate::AcpEvent) -> Result<(), String> {
        // 广播给所有订阅者
        let _ = self.broadcast.send(event.clone());

        // 定向路由（如果有 target_id）
        if let Some(target) = &event.target_id {
            let subs = self.subscribers.read().await;
            if let Some(tx) = subs.get(target) {
                let _ = tx.send(event);
            }
        }
        Ok(())
    }

    pub async fn subscribe(&self, target_id: &str) -> broadcast::Receiver<crate::AcpEvent> {
        let (tx, rx) = broadcast::channel(256);
        self.subscribers.write().await.insert(target_id.to_string(), tx);
        rx
    }

    pub fn subscribe_all(&self) -> broadcast::Receiver<crate::AcpEvent> {
        self.broadcast.subscribe()
    }
}
```

---

### 1.3 swarm-engine

**职责**：Goal-Driven 编排引擎核心。包含 Reconcile Loop、拓扑执行、Queen 选举。

**文件结构**：
```
crates/swarm-engine/src/
├── lib.rs
├── goal.rs           # Goal 管理（CRUD + 状态机）
├── reconcile.rs      # Reconcile Loop 实现
├── evaluator.rs      # ConditionEvaluator (执行 CompletionCondition)
├── topology/
│   ├── mod.rs
│   ├── star.rs       # Star 拓扑（Queen 分解 + Workers 并行）
│   └── chain.rs      # Chain 拓扑（顺序执行）
├── queen.rs          # Queen 选举 + Lease 管理 + 防脑裂
├── graph.rs          # GoalGraph（DAG 依赖管理）
└── worker_manager.rs # Worker 注册/心跳/故障转移
```

**Reconcile Loop 实现**：
```rust
// reconcile.rs

pub struct ReconcileLoop {
    bus: Arc<AcpMessageBus>,
    event_store: Arc<dyn EventStore>,
    evaluator: ConditionEvaluator,
}

impl ReconcileLoop {
    pub async fn run(&self, goal: &mut Goal, worker_comm: &dyn WorkerCommunication) -> GoalOutcome {
        loop {
            // Step 1: Token Budget Check
            if goal.tokens_used >= goal.token_budget {
                goal.status = GoalStatus::BudgetExhausted;
                self.emit_status_change(goal).await;
                return GoalOutcome::BudgetExhausted {
                    iterations: goal.current_iteration,
                    tokens_used: goal.tokens_used,
                    last_feedback: goal.iteration_log.last()
                        .map(|r| r.evaluation.feedback.clone())
                        .unwrap_or_default(),
                };
            }

            // Step 2: Max Iteration Check
            if goal.current_iteration >= goal.max_iterations {
                goal.status = GoalStatus::MaxIterReached;
                self.emit_status_change(goal).await;
                return GoalOutcome::MaxIterReached {
                    iterations: goal.current_iteration,
                    tokens_used: goal.tokens_used,
                    last_feedback: goal.iteration_log.last()
                        .map(|r| r.evaluation.feedback.clone())
                        .unwrap_or_default(),
                };
            }

            // Step 3: Build worker message
            let worker_message = self.build_worker_message(goal);

            // Step 4: Dispatch to Worker
            goal.status = GoalStatus::Active;
            self.emit_status_change(goal).await;

            let execution_result = match worker_comm.execute(goal.executor.as_str(), &worker_message, goal.per_iteration_timeout_ms).await {
                Ok(r) => r,
                Err(e) => {
                    goal.status = GoalStatus::Failed(e.clone());
                    self.emit_status_change(goal).await;
                    return GoalOutcome::Failed(e);
                }
            };

            goal.tokens_used += execution_result.tokens_used;

            // Step 5: Evaluate completion condition
            goal.status = GoalStatus::Evaluating;
            self.emit_status_change(goal).await;

            let evaluation = self.evaluator.evaluate(&goal.completion_condition, &goal.executor).await;

            // Step 6: Record iteration
            let record = IterationRecord {
                iteration: goal.current_iteration,
                execution_started_at: execution_result.started_at,
                execution_finished_at: execution_result.finished_at,
                worker_output: execution_result.output.clone(),
                tokens_used: execution_result.tokens_used,
                cumulative_tokens: goal.tokens_used,
                evaluation: evaluation.clone(),
                evaluation_duration_ms: evaluation.duration_ms,
                tool_calls: execution_result.tool_calls,
                files_modified: execution_result.files_modified,
            };
            goal.iteration_log.push(record);

            // Step 7: Check convergence
            if evaluation.converged {
                goal.status = GoalStatus::Converged;
                goal.converged_at = Some(chrono::Utc::now());
                self.emit_status_change(goal).await;
                return GoalOutcome::Converged {
                    iterations: goal.current_iteration + 1,
                    tokens_used: goal.tokens_used,
                    final_feedback: evaluation.feedback,
                };
            }

            // Step 8: Oscillation detection
            if self.detect_oscillation(goal) {
                // Queen intervenes: change Worker or modify Goal
                // TODO: Queen intervention logic
                goal.status = GoalStatus::Failed("Oscillation detected, Queen intervention needed".to_string());
                self.emit_status_change(goal).await;
                return GoalOutcome::Failed("Oscillation detected".to_string());
            }

            // Prepare next iteration
            goal.current_iteration += 1;
            goal.status = GoalStatus::Iterating;
            self.emit_status_change(goal).await;

            // Append feedback to description
            goal.description = format!(
                "{}\n\n[Iteration {} feedback]: {}",
                goal.description, goal.current_iteration, evaluation.feedback
            );
        }
    }

    fn detect_oscillation(&self, goal: &Goal) -> bool {
        if goal.iteration_log.len() < 3 { return false; }
        let last_three = &goal.iteration_log[goal.iteration_log.len() - 3..];
        let feedbacks: Vec<&str> = last_three.iter().map(|r| r.evaluation.feedback.as_str()).collect();
        feedbacks[0] == feedbacks[1] && feedbacks[1] == feedbacks[2]
    }
}
```

**Queen 选举**：
```rust
// queen.rs

pub struct QueenElectionManager {
    workers: Arc<RwLock<HashMap<String, WorkerCapabilityManifest>>>,
    current_queen: Arc<RwLock<Option<QueenLease>>>,
    bus: Arc<AcpMessageBus>,
}

impl QueenElectionManager {
    pub async fn elect_queen(&self) -> Result<String, String> {
        let workers = self.workers.read().await;

        // Score all workers
        let mut scored: Vec<(String, f32)> = workers
            .iter()
            .map(|(wid, manifest)| {
                let proficiency = manifest.skills.iter().map(|s| s.proficiency).sum::<f32>()
                    / manifest.skills.len().max(1) as f32;
                let success_rate = manifest.skills.iter().map(|s| s.success_rate).sum::<f32>()
                    / manifest.skills.len().max(1) as f32;
                let load = 0.0; // TODO: get actual load

                let mut score = proficiency * 0.4 + success_rate * 0.4 + (1.0 / (load + 1.0)) * 0.2;

                // Claude Code bonus
                if manifest.worker_type == "claude_code" {
                    score += 0.1;
                }

                (wid.clone(), score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if let Some((queen_id, _)) = scored.first() {
            let lease = QueenLease {
                lease_id: uuid::Uuid::new_v4().to_string(),
                queen_id: queen_id.clone(),
                ttl_seconds: 120,
                renew_interval_seconds: 30,
                granted_at: chrono::Utc::now(),
            };

            *self.current_queen.write().await = Some(lease.clone());

            self.bus.emit(AcpEvent {
                event_type: "queen.elected".to_string(),
                timestamp: chrono::Utc::now(),
                task_id: None,
                data: serde_json::to_value(&lease).unwrap(),
            }).await?;

            Ok(queen_id.clone())
        } else {
            Err("No workers available for Queen election".to_string())
        }
    }
}
```

---

### 1.4 goal-parser

**职责**：`.goal` YAML 文件解析 + 验证。

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GoalFile {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub workers: Vec<WorkerDecl>,
    pub goals: Vec<GoalDecl>,
    pub top: TopGoal,
}

pub fn parse_goal_file(path: &str) -> Result<GoalGraph, GoalParseError> {
    let content = std::fs::read_to_string(path)?;
    let goal_file: GoalFile = serde_yaml::from_str(&content)?;

    // Validate: executor must be in workers list
    let worker_ids: Vec<&str> = goal_file.workers.iter().map(|w| w.id.as_str()).collect();
    for goal in &goal_file.goals {
        if !worker_ids.contains(&goal.executor.as_str()) {
            return Err(GoalParseError {
                line: 0,
                column: 0,
                message: format!("Executor '{}' not found in workers list", goal.executor),
            });
        }
    }

    // Build GoalGraph
    let mut graph = GoalGraph::new();
    // ... convert GoalDecl to Goal and add to graph

    // Validate: no circular dependencies
    if graph.has_cycles() {
        return Err(GoalParseError {
            line: 0,
            column: 0,
            message: "Circular dependencies detected in goals".to_string(),
        });
    }

    Ok(graph)
}
```

---

## 2. EventStore 抽象层设计

### 2.1 Trait 定义

```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn store_event(&self, event: &AcpEvent) -> Result<(), String>;
    async fn get_events(&self, filter: EventFilter) -> Result<Vec<AcpEvent>, String>;
    async fn get_events_since(&self, since: DateTime<Utc>) -> Result<Vec<AcpEvent>, String>;
    async fn delete_events_older_than(&self, before: DateTime<Utc>) -> Result<u64, String>;
}

pub struct EventFilter {
    pub event_type: Option<String>,
    pub task_id: Option<String>,
    pub limit: Option<u64>,
}
```

### 2.2 SQLite 实现（v1）

- 单文件 `acp-swarm.db`
- 表：`events`, `workers`, `goals`, `checkpoints`
- 保留 30 天事件

### 2.3 PostgreSQL 实现（v2+）

- 通过相同的 `EventStore` trait 实现
- 支持多租户、分布式部署
- 无需修改业务逻辑

---

## 3. Worker Protocol 双端实现

### 3.1 Rust 端（编排引擎）

```rust
// acp-transport/src/worker_protocol.rs

pub struct WorkerProtocolServer {
    bus: Arc<AcpMessageBus>,
    worker_manager: Arc<WorkerManager>,
}

impl WorkerProtocolServer {
    pub async fn handle_register(&self, req: RegisterRequest) -> Result<RegisterResponse, String> {
        self.worker_manager.register_worker(req.capabilities).await?;
        self.bus.emit(AcpEvent::worker_registered(&req.worker_id, &req.worker_type)).await?;
        Ok(RegisterResponse {
            status: "registered".to_string(),
            worker_id: req.worker_id,
            heartbeat_interval_ms: 10000,
        })
    }

    pub async fn handle_heartbeat(&self, req: HeartbeatRequest) -> Result<HeartbeatResponse, String> {
        self.worker_manager.update_heartbeat(&req.worker_id, req.status, req.current_load).await?;
        Ok(HeartbeatResponse { status: "ok".to_string() })
    }

    // ... goal/receive, goal/progress, goal/result
}
```

### 3.2 TypeScript 端（SDK）

```typescript
// packages/worker-sdk/src/worker.ts

import { EventEmitter } from 'events';

export class Worker extends EventEmitter {
  private config: WorkerConfig;
  private heartbeatInterval: NodeJS.Timeout | null = null;

  constructor(config: WorkerConfig) {
    super();
    this.config = config;
  }

  onGoal(handler: (goal: Goal) => Promise<GoalResult>): void {
    this.on('goal', handler);
  }

  async start(): Promise<void> {
    // 1. Register with orchestrator
    const registerResp = await fetch(`${this.config.orchestratorUrl}/acp/worker/register`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        worker_id: this.config.workerId,
        worker_type: this.config.workerType,
        version: '1.0',
        capabilities: this.config.capabilities,
      }),
    });

    // 2. Start heartbeat
    this.heartbeatInterval = setInterval(async () => {
      await fetch(`${this.config.orchestratorUrl}/acp/worker/heartbeat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          worker_id: this.config.workerId,
          status: 'idle',
          current_load: 0,
          active_goals: [],
          memory_usage_mb: process.memoryUsage().heapUsed / 1024 / 1024,
          uptime_seconds: process.uptime(),
        }),
      });
    }, 10000);

    // 3. Listen for goals (via WebSocket or polling)
    // ...
  }
}
```

---

## 4. 迁移路径（acp-ui → acp-swarm）

### 4.1 Phase 0: 仓库初始化（1 天）

- Fork acp-ui → acp-swarm
- 创建 Cargo workspace 骨架
- 删除死代码（bot_gateway.rs, hermes_traits.rs 等）

### 4.2 Phase 1: Crate 提取（3 天）

| 源文件 | 目标 Crate |
|--------|-----------|
| `agent_orchestration.rs` | `swarm-engine/src/orchestration.rs` |
| `swarm_orchestrator.rs` | `swarm-engine/src/orchestrator.rs` |
| `smart_router.rs` | `swarm-engine/src/router.rs` |
| `workflow_engine.rs` | `workflow-engine/src/engine.rs` |
| `team_dag.rs` | `workflow-engine/src/dag.rs` |
| `approval_engine.rs` | `workflow-engine/src/approval.rs` |
| `hooks_executor.rs` | `hook-runtime/src/executor.rs` |
| `plugin_registry.rs` | `acp-core/src/plugin.rs` |
| `self_healing.rs` | `swarm-engine/src/healing.rs` |
| `circuit_breaker.rs` | `acp-core/src/circuit_breaker.rs` |
| `permission_checker.rs` | `tool-sandbox/src/permission.rs` |
| `mcp_client.rs` | `acp-transport/src/mcp.rs` |
| `config.rs` | `acp-core/src/config.rs` |
| `agent_config_parser.rs` | `acp-core/src/config_parser.rs` |
| `skill_commands.rs` | `acp-core/src/skill.rs` |

### 4.3 Phase 2: 前端清理（2 天）

- 删除 25 个 orphan 文件
- 更新 router.ts
- 简化 self-improvement/ (8 → 2 files)

### 4.4 Phase 3: Swarm 原型（5 天）

- 实现 Goal 数据结构
- 实现 CompletionCondition evaluator
- 实现 Reconcile Loop
- 实现 Star/Chain 拓扑
- 前端 SwarmDashboard 接入真实数据

---

## 5. 关键设计决策

### 5.1 为什么用 Cargo Workspace 而不是单体 crate？

- 独立编译：改 `acp-core` 不需要重编 `swarm-engine`
- 可复用：其他项目可以 `cargo add acp-core` 只引用协议层
- 测试隔离：每个 crate 独立测试

### 5.2 为什么 EventStore 用 trait 抽象？

- v1 SQLite 简单，适合单机
- v2 PostgreSQL 支持多租户、分布式
- 业务逻辑不依赖具体存储实现

### 5.3 为什么 Queen TTL 从 30s 改为 120s？

- Goal 分解可能需要 30-60 秒
- 30s TTL 容易在分解过程中过期
- 120s + Busy Queen 延期模式更安全

### 5.4 为什么 Reconcile Loop 需要震荡检测？

- Agent 可能陷入死循环（修一个 bug 引入另一个）
- 相同反馈连续 3 次 → 说明 Agent 没有进展
- Queen 介入：换 Worker 或修改 Goal 描述

---

## 6. 测试策略

### 6.1 单元测试（每个 crate）

- `acp-core`: Goal 序列化、CompletionCondition 评估
- `swarm-engine`: Reconcile Loop、Queen 选举、拓扑执行
- `goal-parser`: YAML 解析、循环依赖检测
- `event-bus`: 消息路由、广播

### 6.2 集成测试

- Worker 注册 → Goal 提交 → Reconcile Loop → 收敛
- Checkpoint 创建 → 文件修改 → Rollback → 验证恢复

### 6.3 稳定性测试

- 100 个 Goal 连续执行（内存泄漏检测）
- Queen Lease 过期 → 重新选举 → 旧 Queen 拒绝

### 6.4 安全测试

- Tool 沙箱：`.env` 访问拦截
- Worker 隔离：Worker A 不能访问 Worker B 的数据

---

*Architecture version: 1.0.0*
*Last updated: 2026-06-09*
