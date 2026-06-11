# hello-swarm-goal.md
# Goal DSL 语法说明

## 概述

`.goal` 文件定义了 Goal-Driven 执行的工作流程。每个 Goal 有明确的完成条件，系统会自动迭代直到条件满足。

## 文件格式

支持两种格式：
- YAML (`.goal.yaml`)
- JSON (`.goal.json`)

## 基本结构

```yaml
name: workflow-name
description: "Workflow description"

goals:
  - id: goal-001
    description: "Goal description"
    completion_condition: { ... }
    evaluator: auto
    executor: worker-id
    depends_on: []
    token_budget: 50000
    max_iterations: 5

topology: chain | star

queen:
  lease_ttl_seconds: 30
  worker_id: worker-id

workers:
  - id: worker-001
    type: codex | claude_code
    skills: [ ... ]
```

## Goal 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| id | string | Goal唯一标识 |
| description | string | 目标描述 |
| completion_condition | object | 完成条件 |
| evaluator | string | 评估器类型 (auto/queen/adversarial) |
| executor | string | 执行Worker ID |
| depends_on | array | 依赖的Goal ID列表 |
| token_budget | number | Token预算上限 |
| max_iterations | number | 最大迭代次数 |

## CompletionCondition 类型

### command_success
命令退出码为0表示成功。

```yaml
completion_condition:
  type: command_success
  command: "npm test"
  args: []
  cwd: null
```

### file_check
检查文件是否存在或包含特定内容。

```yaml
completion_condition:
  type: file_check
  path: "docs/api.md"
  content_contains: "API Reference"
```

### output_contains
命令输出包含特定文本。

```yaml
completion_condition:
  type: output_contains
  command: "npm run lint"
  pattern: "0 errors"
  case_sensitive: false
```

### all (AND)
所有子条件都必须满足。

```yaml
completion_condition:
  type: all
  conditions:
    - { type: command_success, command: "npm test" }
    - { type: output_matches, pattern: "Coverage > 80%" }
```

### any (OR)
任一子条件满足即可。

```yaml
completion_condition:
  type: any
  conditions:
    - { type: command_success, command: "eslint" }
    - { type: command_success, command: "prettier --check" }
```

## 拓扑类型

| 类型 | 说明 |
|------|------|
| chain | 按依赖顺序执行，Goal必须等待依赖收敛后启动 |
| star | 并行执行，所有无依赖的Goal同时启动 |

## 示例

见 `hello-swarm.goal.yaml`