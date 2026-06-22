# ACP-UI 版本发布文档

## v0.1.14 — Loop Engine Edition

**发布日期**: 2026-06-22  
**分支**: `refactor/project-cleanup`

---

## 🚀 主要更新

### Loop Engine — 统一循环引擎

本次发布引入了 **Loop Engine**，一个统一的循环系统编排器，整合了：

| 循环类型 | 功能 | 状态 |
|---------|------|------|
| **ReconcileLoop** | Goal 迭代执行 + 收敛检查 | ✅ 已实现 |
| **Self-Healing Loop** | 异常检测 + 自动修复 | ✅ 已激活 |
| **Evolution Loop** | Pattern 提取 + Skill 演进 | ✅ 已完善 |
| **Learning Loop** | 知识沉淀 + 跨会话共享 | 🔄 设计完成 |

**核心特性**:
- Goal 失败自动触发 Self-Healing
- Goal 成功自动触发 Evolution
- Pattern 自动学习并持久化
- 健康监控 + EWMA 异常检测

---

## 📦 新增文件

### 后端 (Rust)

| 文件 | 说明 |
|------|------|
| `src-tauri/src/loop_engine.rs` | Loop Engine 核心实现 |
| `docs/loop-engine-architecture.md` | 架构设计文档 |

### 前端 (Vue/TypeScript)

| 文件 | 说明 |
|------|------|
| `src/lib/loop-api.ts` | 前端 API (4 个 Tauri 命令封装) |
| `src/stores/loop.ts` | Vue reactive store |
| `src/features/loop/LoopDashboard.vue` | 可视化面板 |

---

## 🔧 Tauri 命令

| 命令 | 功能 |
|------|------|
| `loop_execute_goal` | 执行 Goal 并触发相关 Loop |
| `loop_get_state` | 获取 Loop 状态 |
| `loop_get_stats` | 获取统计数据 |
| `loop_update_metrics` | 更新健康指标 |

---

## 🐛 Bug 修复

### P0 严重问题 (4项)

| 问题 | 文件 | 修复 |
|------|------|------|
| `remove_goal()` 逻辑 BUG | goal_graph.rs | 先保存 deps 再清理 |
| 命令注入风险 | goal_evaluator.rs | 添加 denylist + 长度限制 |
| WebSocket Promise 泄漏 | client.ts | onclose reject pending |
| 假数据返回 | swarm_orchestrator.rs | 返回显式错误信息 |

### P1-P3 修复 (13项)

- remote-control/server.ts: 8 个假数据改为真实 API 调用
- queen_lease.rs: 双锁死锁风险修复
- channel-registry.ts: 渠道硬编码修复
- capability-evolver.ts: 评分范围限制
- kv-cache.ts: 64位 BigInt 哈希
- topology.rs: Pipeline 实现
- 等等...

---

## 🧹 代码清理

- 8 个预留功能模块添加 `#![allow(dead_code)]` 注释
- indexmap 升级到 v2 (解决 schemars 兼容性)
- orchestrator.ts, task-parser.ts 标记 `@deprecated`

---

## 🧪 测试状态

| 类型 | 数量 | 结果 |
|------|------|------|
| TypeScript 单元测试 | 294 | ✅ 全部通过 |
| Rust 单元测试 | 40+ | ✅ 全部通过 |
| TypeScript 编译 | - | ✅ 0 错误 |
| Rust 编译 | - | ✅ 0 警告 |

---

## ⚠️ 已知问题 (系统环境)

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| Flutter Maven 403 | 网络/防火墙阻断 | 使用 VPN |
| Rust 构建 OS Error 5 | Windows 权限拒绝 | 临时禁用防病毒软件 |

---

## 📊 代码健康

```
TypeScript:   Typecheck ✅ 0 errors
Flutter:      analyze ✅ 182 warnings/info (无错误)
Rust:         cargo check ✅ 0 warnings
Tests:        294/294 ✅ 全部通过
```

---

## 🔗 提交历史

```
7ee4537 chore: Project cleanup & dependency fixes
9054783 feat: Loop Engine - unified orchestrator for all loop systems
f959aef feat: Loop Engine frontend - API, store, and dashboard
```

---

## 📝 贡献者

- **Claude Opus 4.7** — Loop Engine 设计与实现
- **yan_fan_tian** — 项目维护与测试验证

---

## 🗺️ 下一步计划

### Phase 6: UI 集成

1. 将 LoopDashboard 雷达图集成到主界面
2. 添加 Goal 提交 UI (GoalSubmitPanel)
3. 实现实时状态推送 (WebSocket)

### Phase 7: 生产验证

1. 运行真实 Goal 测试
2. 验证 Self-Healing 自动修复
3. 验证 Evolution Pattern 学习

---

## 📚 相关文档

- [Loop Engine Architecture](./docs/loop-engine-architecture.md)
- [Project Completion Plan](./docs/project-completion-plan.md)
- [System Architecture](./docs/system-architecture.md)

---

**签署**: ACP-UI Team  
**日期**: 2026-06-22