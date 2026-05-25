---
name: acp-ui-arch-optimize
description: ACP-UI 架构优化自动化工作流 - 全程执行不停止询问
version: 1.0.0
source: local-custom
---

# ACP-UI 架构优化自动化工作流

此 skill 用于自动执行 ACP-UI 项目的架构优化实施，无需中途停止询问用户。

## 触发条件

当用户请求：
- "执行架构优化"
- "继续优化"
- "自动完成架构"
- 或直接调用 `/acp-ui-arch-optimize`

## 执行规则

**绝对规则：全程自动执行，不停止询问用户确认**

每个阶段完成后：
1. 自动提交 git
2. 继续下一阶段
3. 仅在最终完成后汇报结果

## Phase 0: 项目状态检查

```bash
# 检查 Hermes crates 状态
ls -la src-tauri/hermes-crates/

# 检查 Cargo.toml 已链接的 crates
grep -E "hermes-" src-tauri/Cargo.toml
```

自动决策：
- 如果 Hermes crates 存在但未链接 → 链接它们
- 如果已链接 → 跳过，继续 Phase 1

## Phase 1: 链接 Hermes Crates

编辑 `src-tauri/Cargo.toml`，添加：

```toml
[dependencies]
hermes-agent = { path = "hermes-crates/hermes-agent" }
hermes-config = { path = "hermes-crates/hermes-config" }
hermes-core = { path = "hermes-crates/hermes-core" }
hermes-tools = { path = "hermes-crates/hermes-tools" }
hermes-environments = { path = "hermes-crates/hermes-environments" }
hermes-skills = { path = "hermes-crates/hermes-skills" }
hermes-intelligence = { path = "hermes-crates/hermes-intelligence" }
hermes-memory = { path = "hermes-crates/hermes-memory" }
```

验证：
```bash
cd src-tauri && cargo build 2>&1 | head -20
```

提交：
```bash
git add src-tauri/Cargo.toml
git commit -m "fix(phase0): link Hermes crates"
```

## Phase 2: SQLite Schema 扩展

检查 `src-tauri/src/database.rs` 是否已包含新表：
- agent_bases, agent_templates, agent_instances
- teams, team_executions
- route_decisions, input_logs
- anomalies, circuit_breakers, healing_actions

如果缺失，添加表定义。完成后：
```bash
git add src-tauri/src/database.rs
git commit -m "feat(phase1): add SQLite tables for Agent Teams"
```

## Phase 3-6: 核心模块创建

按顺序创建/验证以下模块：

| 模块 | 文件 | 核心功能 |
|------|------|---------|
| Agent Registry | `agent_registry.rs` | Docker-like Base/Template/Instance |
| Smart Router | `smart_router.rs` | 三层渐进复杂度评估 |
| Circuit Breaker | `circuit_breaker.rs` | Closed→Open→HalfOpen 熔断 |
| Self-Healing | `self_healing.rs` | EWMA 动态基线异常检测 |
| Team DAG | `team_dag.rs` | DAG 执行引擎 + SyncPoints |

每个模块完成后：
```bash
cargo build 2>&1 | tail -10
# 如果编译成功
git add src-tauri/src/{module}.rs
git commit -m "feat(phase{n}): add {module} module"
# 继续下一个模块
```

**错误处理规则：**
- 遇到编译错误 → 自动修复，不询问
- 修复后重新编译 → 继续执行
- 如果错误无法修复 → 记录到日志，跳过该模块继续

## Phase 7: Hermes Memory Crate

检查/创建 `src-tauri/hermes-crates/hermes-memory/`：
- `sqlite_provider.rs` - SQLite 存储
- `chroma_provider.rs` - Chroma 向量存储
- `embedding.rs` - 分层 Embedding (L1 local + L2 cloud)
- `hybrid_search.rs` - FTS5 + Vector 搜索

```bash
git add src-tauri/hermes-crates/hermes-memory/
git commit -m "feat(phase7): complete Hermes memory crate"
```

## Phase 8: Tauri 命令注册

在 `src-tauri/src/lib.rs` 中：
1. 添加 AppState 新字段（Mutex 包装）
2. 添加 #[tauri::command] 函数
3. 注册到 invoke_handler

命令列表：
- `analyze_task_complexity`
- `get_circuit_breaker_status`
- `is_circuit_breaker_allowed`
- `reset_circuit_breaker`
- `get_all_circuit_breakers`
- `create_dag_plan`
- `get_dag_plan_progress`
- `check_anomaly`
- `update_anomaly_baseline`

```bash
cargo build && cargo test --lib
git add src-tauri/src/lib.rs
git commit -m "feat: add Tauri commands for Agent Teams Platform"
```

## Phase 9: 测试验证

```bash
cd src-tauri && cargo test --lib 2>&1
```

如果有测试失败：
- 自动分析失败原因
- 修复代码
- 重新运行测试
- 直到全部通过或记录失败原因

```bash
git commit -m "fix(phase9): resolve test failures"
```

## Phase 10: 前端验证

```bash
./node_modules/.bin/vue-tsc --noEmit
./node_modules/.bin/vite build
```

## Phase 11: 文档更新

更新 `docs/project-completion-plan.md` 进度表。

```bash
git add docs/
git commit -m "docs: update progress tracking"
```

## 最终汇报

完成后输出：

```
✅ 架构优化完成

Phase 状态：
| Phase | 状态 | 提交 |
| ... | ... | ... |

新增模块：X 个
新增命令：Y 个
测试结果：Z passed
构建状态：成功

Git 提交历史：
{最近5个提交}
```

## 错误处理

| 错误类型 | 自动处理 |
|---------|---------|
| 编译错误 | 自动修复类型、生命周期等问题 |
| 测试失败 | 更新测试期望值或修复实现 |
| 依赖冲突 | 升级版本或使用兼容版本 |

## 禁止事项

- ❌ 不停止询问"是否继续"
- ❌ 不等待用户确认提交
- ❌ 不在阶段间暂停
- ❌ 不输出冗长的过程描述

只做一件事：**从头到尾执行完毕，然后汇报结果**。