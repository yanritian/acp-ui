# Writing Goals

This guide explains how to write effective `.goal.yaml` files for the ACP-Swarm system.

## Goal Structure

A `.goal.yaml` file defines a goal-driven workflow with the following structure:

```yaml
name: my-workflow
description: "Brief description of the workflow"

goals:
  - id: goal-001
    description: "What this goal should achieve"
    completion_condition:
      type: command_success
      command: "npm test"
      args: []
    evaluator: auto
    executor: worker-001
    depends_on: []
    token_budget: 50000
    max_iterations: 5

topology: chain  # or star for parallel execution
workers:
  - id: worker-001
    type: claude_code
    skills: [...]
```

## Completion Conditions

Choose the right condition type for your goal:

| Type | Use Case | Example |
|------|----------|---------|
| `command_success` | Shell commands | `npm test`, `cargo build` |
| `output_contains` | Output validation | Check for "success" in output |
| `output_matches` | Pattern matching | Regex validation |
| `file_check` | File existence/content | Verify file created |
| `http_health_check` | API health | Endpoint returns 200 |
| `queen_judgment` | Human review | Code quality check |
| `all` | Multiple conditions | All must pass |
| `any` | Alternative conditions | Any one passes |

## Best Practices

### 1. Clear Descriptions
```yaml
description: "Fix TypeScript compilation errors in src/components/"
# NOT: description: "Fix errors"
```

### 2. Reasonable Budgets
```yaml
token_budget: 80000  # 80K tokens for complex tasks
max_iterations: 5    # Allow retries
```

### 3. Dependency Chains
```yaml
# Sequential execution
depends_on: [previous-goal-id]

# Parallel execution (no dependencies)
depends_on: []
```

### 4. Evaluator Selection
- `auto`: Use for automated checks (commands, files)
- `queen`: Use for subjective evaluation (code quality)
- `adversarial`: Use for critical verification
- `hybrid`: Combine auto + queen judgment

## Example: Full Workflow

```yaml
name: release-workflow
description: "Release new version"

goals:
  # Step 1: Build
  - id: build
    description: "Build production bundle"
    completion_condition:
      type: command_success
      command: "npm run build"
    evaluator: auto
    executor: build-worker
    depends_on: []

  # Step 2: Test
  - id: test
    description: "Run all tests"
    completion_condition:
      type: command_success
      command: "npm test"
    evaluator: auto
    executor: test-worker
    depends_on: [build]

  # Step 3: Deploy
  - id: deploy
    description: "Deploy to production"
    completion_condition:
      type: http_health_check
      url: "https://api.example.com/health"
      expected_status: 200
    evaluator: auto
    executor: deploy-worker
    depends_on: [test]

topology: chain
```