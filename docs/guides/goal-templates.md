# Goal Templates

Common goal templates for typical workflows.

## 1. Code Fix Goal

```yaml
- id: fix-typescript
  description: "Fix TypeScript compilation errors"
  completion_condition:
    type: command_success
    command: "npx vue-tsc --noEmit"
  evaluator: auto
  token_budget: 80000
  max_iterations: 5
```

## 2. Test Execution Goal

```yaml
- id: run-tests
  description: "Execute unit tests"
  completion_condition:
    type: command_success
    command: "npm test"
  evaluator: auto
  token_budget: 50000
  max_iterations: 3
```

## 3. File Creation Goal

```yaml
- id: create-config
  description: "Create configuration file"
  completion_condition:
    type: file_check
    path: "config.json"
    must_exist: true
    content_contains: "API_KEY"
  evaluator: auto
  token_budget: 30000
  max_iterations: 2
```

## 4. Code Review Goal

```yaml
- id: code-review
  description: "Review code quality"
  completion_condition:
    type: queen_judgment
    criteria: "Code follows best practices, no security issues"
  evaluator:
    type: queen
    queen_worker_id: queen-001
  token_budget: 60000
  max_iterations: 3
```

## 5. API Health Goal

```yaml
- id: health-check
  description: "Verify API is healthy"
  completion_condition:
    type: http_health_check
    url: "http://localhost:3000/api/health"
    expected_status: 200
  evaluator: auto
  token_budget: 10000
  max_iterations: 3
```

## 6. Compound Condition Goal

```yaml
- id: full-validation
  description: "Complete validation"
  completion_condition:
    type: all
    conditions:
      - type: command_success
        command: "npm test"
      - type: file_check
        path: "coverage/report.html"
        must_exist: true
  evaluator: auto
  token_budget: 100000
  max_iterations: 5
```

## 7. Documentation Goal

```yaml
- id: update-readme
  description: "Update README with latest features"
  completion_condition:
    type: file_check
    path: "README.md"
    content_contains: "New Feature"
  evaluator: auto
  token_budget: 20000
  max_iterations: 2
```

## 8. Hybrid Evaluation Goal

```yaml
- id: quality-check
  description: "Ensure code quality with Queen review"
  completion_condition:
    type: all
    conditions:
      - type: command_success
        command: "npm run lint"
      - type: queen_judgment
        criteria: "Architecture is sound"
  evaluator:
    type: hybrid
    auto_conditions: [...]
    queen_criteria: "Overall quality is excellent"
  token_budget: 80000
  max_iterations: 4
```

## Template Selection Guide

| Task Type | Recommended Template |
|-----------|---------------------|
| Build/Compile | Code Fix |
| Testing | Test Execution |
| Documentation | File Creation / Documentation |
| Deployment | API Health |
| Code Quality | Code Review / Hybrid |
| Multi-step | Compound Condition |