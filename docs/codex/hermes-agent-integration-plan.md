# Hermes Agent API Integration Plan

> 日期: 2026-07-08
> 状态: 计划阶段
> 优先级: P0 (MVP 闭环必需)

## 1. 当前状态

| 模块 | 状态 | 说明 |
|------|------|------|
| `agent_bridge.rs` | Mock | 返回模拟结果，无真实 Agent 调用 |
| `hermes_runtime.rs` | Mock | API 配置定义，未实现真实调用 |
| `task_executor.rs` | Mock | 使用 mock 桥接，无法真实执行 |
| 前端 API | Mock | 调用 Tauri 命令，返回模拟状态 |

## 2. 集成目标

**短期目标**: 实现 Godot MVP 的真实 Agent 执行闭环

- 用户输入目标 → Hermes 分析项目 → 生成计划 → 用户审批 → 执行修改 → 输出总结

**长期目标**: 支持多领域 Agent 执行

- Godot → Unity → Ren'Py → Enterprise → Video → Comic

## 3. 集成方案

### 方案 A: Hermes CLI 子进程调用

```rust
// 启动 Hermes CLI 作为子进程
let output = Command::new("hermes")
    .arg("--goal")
    .arg(&goal)
    .arg("--project")
    .arg(&project_path)
    .output()?;
```

**优点**:
- 简单直接
- Hermes CLI 已有完整能力
- 进程隔离安全

**缺点**:
- 需要安装 Hermes CLI
- 进程间通信开销
- 流式响应处理复杂

### 方案 B: Hermes HTTP API

```rust
// 通过 HTTP 调用 Hermes 服务
let client = reqwest::Client::new();
let response = client
    .post("http://localhost:8080/api/execute")
    .json(&request)
    .send()?;
```

**优点**:
- 支持 WebSocket 流式响应
- 独立服务可复用
- 跨平台兼容

**缺点**:
- 需要部署 Hermes 服务
- 网络依赖

### 方案 C: Hermes Rust Crate 直接集成

```rust
// 直接使用 Hermes crate
use hermes_core::{HermesAgent, AgentConfig};

let agent = HermesAgent::new(config);
let result = agent.execute(goal, context)?;
```

**优点**:
- 性能最优
- 无外部依赖
- 状态共享方便

**缺点**:
- 编译依赖复杂
- 版本同步问题

## 4. 推荐方案

**Phase 1 MVP**: 方案 A (Hermes CLI 子进程)

- 快速实现，验证闭环
- 利用现有 Hermes CLI 能力
- 逐步过渡到方案 C

**Phase 2**: 方案 C (Hermes Rust Crate)

- 深度集成，提升性能
- 统一版本管理
- 实现流式事件传递

## 5. 实现步骤

### Step 1: CLI 集成 (Day 1-2)

1. 实现 `HermesCliBridge` 结构
2. 射 Hermes CLI 命令
3. 解析 CLI 输出为 Rust 类型
4. 处理错误和超时

```rust
pub struct HermesCliBridge {
    cli_path: PathBuf,
    project_path: PathBuf,
}

impl HermesCliBridge {
    pub async fn execute(&self, goal: &str) -> Result<HermesResult, Error> {
        let mut child = Command::new(&self.cli_path)
            .arg("execute")
            .arg("--goal")
            .arg(goal)
            .arg("--project")
            .arg(&self.project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Stream stdout for events
        let stdout = child.stdout.take()?;
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let event = parse_hermes_event(line?)?;
            self.emit_event(event);
        }

        let status = child.wait()?;
        Ok(HermesResult::from_status(status))
    }
}
```

### Step 2: 事件流解析 (Day 2-3)

1. 定义 Hermes 输出格式协议
2. 实现 JSON 解析器
3. 映射 Hermes 事件到 OperatorEvent
4. 支持流式事件传递

### Step 3: 前端集成 (Day 3-4)

1. 修改前端 API 调用真实命令
2. 处理流式事件更新 UI
3. 实现真实审批流程
4. 展示真实 diff 结果

### Step 4: E2E 验证 (Day 4-5)

1. 使用测试 Godot 项目验证
2. 测试暂停/继续/停止
3. 测试审批流程
4. 测试错误恢复

## 6. 验收标准

| 标准 | 验收命令 |
|------|----------|
| Hermes CLI 可调用 | `hermes --version` 返回版本 |
| 项目分析真实执行 | Hermes 返回真实脚本/场景列表 |
| 计划生成真实 | Hermes 返回基于分析的计划 |
| 文件修改真实 | 实际文件被修改，diff 展示 |
| 任务总结真实 | 包含 token 使用、执行时间统计 |

## 7. 依赖条件

- Hermes CLI 已安装 (`hermes --version`)
- Hermes 支持 Godot 领域 Skill
- Hermes 支持 `--project` 参数
- Hermes 输出 JSON 格式事件

## 8. 风险

| 风险 | 影响 | 对策 |
|------|------|------|
| Hermes CLI 不存在 | 阻塞 MVP | 安装 Hermes 或实现 fallback |
| 输出格式变化 | 解析失败 | 定义稳定协议，版本检查 |
| 进程超时 | 任务卡死 | 设置超时，支持中断 |
| 权限问题 | 无法执行 | 检查文件权限，用户确认 |

## 9. 下一步

1. 确认 Hermes CLI 安装状态
2. 实现 `HermesCliBridge`
3. 替换 mock 为真实调用
4. E2E 验证完整闭环