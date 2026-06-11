# Getting Started - ACP-Swarm

## 5分钟快速上手

### 1. 安装依赖

```bash
# 克隆仓库
git clone https://github.com/your-org/acp-swarm.git
cd acp-swarm

# 安装前端依赖
npm install

# 安装 Rust 依赖
cd src-tauri
cargo build --workspace
```

### 2. 启动开发服务器

```bash
# 回到项目根目录
cd ..

# 启动 Tauri 开发模式
npm run tauri dev
```

### 3. 创建第一个 Goal

在 Dashboard 中点击 **"New Goal"** 按钮，输入：

```
Fix all TypeScript compilation errors
```

设置完成条件为：

```json
{
  "type": "command_success",
  "command": "npx vue-tsc --noEmit",
  "expectedExitCode": 0
}
```

### 4. 观察执行过程

- Goal 进入 **Active** 状态
- Worker 开始执行
- 每轮执行后自动评估
- 成功后进入 **Converged** 状态

---

## 核心概念

### Goal（目标）

Goal 是蜂群编排的核心单元，包含：

| 字段 | 说明 |
|------|------|
| id | Goal唯一标识 |
| description | 目标描述 |
| completionCondition | 完成条件 |
| evaluator | 评估器类型 |
| maxIterations | 最大迭代次数 |

### CompletionCondition（完成条件）

支持8种类型：

| 类型 | 说明 | 示例 |
|------|------|------|
| command_success | 命令退出码为0 | `npm test` |
| output_contains | 输出包含文本 | "0 errors" |
| output_matches | 输出匹配正则 | 覆盖率 > 80% |
| file_check | 文件存在/内容匹配 | `docs/api.md` |
| http_health_check | HTTP端点返回2xx | `/api/health` |
| all | 所有条件满足 | 测试通过 AND 覆盖率达标 |
| any | 任一条件满足 | 至少一个linter通过 |
| queen_judgment | Queen主观判断 | 架构是否合理 |

### Worker（执行器）

Worker 是实际执行 Goal 的 Agent：

| 类型 | 说明 |
|------|------|
| Codex | 自主代码生成 |
| Claude Code | 高级推理分析 |

---

## 下一步

- [Worker 接入指南](./guides/build-a-worker.md)
- [协议规范](./protocols/)
- [RFC-001: Goal驱动架构](../rfcs/RFC-001-Goal驱动架构.md)