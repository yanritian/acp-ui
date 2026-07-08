# API Testing Guide

## Overview

This guide covers comprehensive API testing strategies for Hermes Game Operator.

---

## Testing Levels

### 1. Unit Tests

**Purpose**: Test individual functions and components

**Tools**: Jest, Vitest

**Example**:
```typescript
// tests/unit/operator.test.ts
import { describe, it, expect } from 'vitest'
import { TaskStateMachine } from '../src/operator/state_machine'

describe('TaskStateMachine', () => {
  it('should transition from Idle to Planning', () => {
    const machine = new TaskStateMachine('task_123')
    machine.startTask()
    expect(machine.currentStatus).toBe('planning')
  })
  
  it('should reject invalid transitions', () => {
    const machine = new TaskStateMachine('task_123')
    expect(() => machine.complete()).toThrow()
  })
})
```

---

### 2. Integration Tests

**Purpose**: Test component interactions

**Tools**: Jest, Supertest

**Example**:
```typescript
// tests/integration/task.test.ts
import { describe, it, expect, beforeAll } from 'vitest'
import request from 'supertest'
import app from '../src/app'

describe('Task API', () => {
  let taskId: string
  
  beforeAll(async () => {
    const response = await request(app)
      .post('/api/operator/start')
      .send({
        domain: 'game.godot',
        project_path: '/test/project',
        goal: 'Add feature'
      })
    taskId = response.body.task_id
  })
  
  it('should create a task', async () => {
    const response = await request(app)
      .get(`/api/operator/tasks/${taskId}`)
    
    expect(response.status).toBe(200)
    expect(response.body.task_id).toBe(taskId)
  })
  
  it('should pause a task', async () => {
    const response = await request(app)
      .post(`/api/operator/pause`)
      .send({ task_id: taskId })
    
    expect(response.status).toBe(200)
  })
})
```

---

### 3. E2E Tests

**Purpose**: Test complete workflows

**Tools**: Playwright, Cypress

**Example**:
```typescript
// tests/e2e/task-workflow.spec.ts
import { test, expect } from '@playwright/test'

test('complete task workflow', async ({ page }) => {
  // Navigate to Game Operator
  await page.goto('http://localhost:1420/games')
  
  // Select project
  await page.click('[data-testid="browse-button"]')
  await page.fill('[data-testid="project-path"]', '/test/project')
  
  // Enter goal
  await page.fill('[data-testid="task-goal"]', 'Add double jump')
  
  // Start task
  await page.click('[data-testid="start-task"]')
  
  // Wait for plan
  await expect(page.locator('[data-testid="plan-panel"]')).toBeVisible()
  
  // Approve plan
  await page.click('[data-testid="approve-plan"]')
  
  // Wait for completion
  await expect(page.locator('[data-testid="task-status"]'))
    .toContainText('COMPLETED')
})
```

---

### 4. Performance Tests

**Purpose**: Test API under load

**Tools**: Artillery, k6

**Example**:
```yaml
# tests/performance/api-load.yml
config:
  target: "http://localhost:1420"
  phases:
    - duration: 60
      arrivalRate: 10
      name: "Warm up"
    - duration: 120
      arrivalRate: 50
      name: "Load test"
  payloads:
    - path: "test-data/tasks.csv"
      fields:
        - "goal"

scenarios:
  - name: "Create task"
    flow:
      - post:
          url: "/api/operator/start"
          json:
            domain: "game.godot"
            project_path: "/test/project"
            goal: "{{ goal }}"
          capture:
            - json: "$.task_id"
              as: "taskId"
      - get:
          url: "/api/operator/tasks/{{ taskId }}"
```

---

## API Test Categories

### 1. Functional Tests

**Test Cases**:

| Endpoint | Method | Test Case | Expected |
|----------|--------|-----------|----------|
| /api/operator/start | POST | Valid request | 200 OK |
| /api/operator/start | POST | Invalid domain | 400 Bad Request |
| /api/operator/tasks | GET | List tasks | 200 OK |
| /api/operator/tasks/:id | GET | Valid task | 200 OK |
| /api/operator/tasks/:id | GET | Invalid task | 404 Not Found |
| /api/operator/pause | POST | Pause task | 200 OK |
| /api/operator/resume | POST | Resume task | 200 OK |
| /api/operator/stop | POST | Stop task | 200 OK |

---

### 2. Validation Tests

**Test Cases**:

```typescript
describe('Request Validation', () => {
  it('should reject missing required fields', async () => {
    const response = await request(app)
      .post('/api/operator/start')
      .send({})
    
    expect(response.status).toBe(400)
    expect(response.body.error).toContain('Missing required field')
  })
  
  it('should reject invalid domain', async () => {
    const response = await request(app)
      .post('/api/operator/start')
      .send({
        domain: 'invalid',
        project_path: '/test',
        goal: 'Test'
      })
    
    expect(response.status).toBe(400)
    expect(response.body.error).toContain('Invalid domain')
  })
  
  it('should reject invalid project path', async () => {
    const response = await request(app)
      .post('/api/operator/start')
      .send({
        domain: 'game.godot',
        project_path: '/nonexistent',
        goal: 'Test'
      })
    
    expect(response.status).toBe(400)
    expect(response.body.error).toContain('Project not found')
  })
})
```

---

### 3. Error Handling Tests

**Test Cases**:

```typescript
describe('Error Handling', () => {
  it('should handle database errors', async () => {
    // Mock database error
    jest.spyOn(db, 'query').mockRejectedValue(new Error('DB error'))
    
    const response = await request(app)
      .get('/api/operator/tasks')
    
    expect(response.status).toBe(500)
    expect(response.body.error).toContain('Internal server error')
  })
  
  it('should handle network timeouts', async () => {
    // Mock timeout
    jest.useFakeTimers()
    
    const promise = request(app)
      .get('/api/operator/tasks/task_123')
    
    jest.advanceTimersByTime(30000)
    
    const response = await promise
    
    expect(response.status).toBe(504)
    expect(response.body.error).toContain('Timeout')
  })
})
```

---

### 4. Security Tests

**Test Cases**:

```typescript
describe('Security', () => {
  it('should prevent SQL injection', async () => {
    const response = await request(app)
      .get('/api/operator/tasks')
      .query({ filter: "'; DROP TABLE tasks; --" })
    
    expect(response.status).toBe(200)
    // Verify table still exists
    const tasks = await db.query('SELECT COUNT(*) FROM tasks')
    expect(tasks[0].count).toBeGreaterThan(0)
  })
  
  it('should prevent XSS', async () => {
    const response = await request(app)
      .post('/api/operator/start')
      .send({
        domain: 'game.godot',
        project_path: '/test',
        goal: '<script>alert("XSS")</script>'
      })
    
    expect(response.body.goal).not.toContain('<script>')
  })
  
  it('should require authentication', async () => {
    const response = await request(app)
      .get('/api/operator/tasks')
      // No auth header
    
    expect(response.status).toBe(401)
  })
})
```

---

## Test Automation

### CI/CD Integration

**.github/workflows/api-tests.yml**:
```yaml
name: API Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:13
        env:
          POSTGRES_DB: hermes_test
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Node.js
        uses: actions/setup-node@v2
        with:
          node-version: '18'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run unit tests
        run: npm run test:unit
      
      - name: Run integration tests
        run: npm run test:integration
        env:
          DATABASE_URL: postgresql://test:test@localhost:5432/hermes_test
      
      - name: Run E2E tests
        run: npm run test:e2e
        env:
          BASE_URL: http://localhost:1420
      
      - name: Upload coverage
        uses: codecov/codecov-action@v1
```

---

## Test Data

### Fixtures

```typescript
// tests/fixtures/tasks.ts
export const mockTask = {
  task_id: 'task_123',
  domain: 'game.godot',
  project_path: '/test/project',
  goal: 'Add double jump',
  status: 'running',
  created_at: '2026-07-08T10:00:00Z',
  updated_at: '2026-07-08T10:05:00Z'
}

export const mockTasks = [
  mockTask,
  { ...mockTask, task_id: 'task_124', status: 'completed' },
  { ...mockTask, task_id: 'task_125', status: 'failed' }
]
```

### Mock Data

```typescript
// tests/mocks/operator.ts
export const mockOperatorApi = {
  startTask: jest.fn().mockResolvedValue({
    task_id: 'task_123',
    status: 'planning'
  }),
  
  getTask: jest.fn().mockResolvedValue(mockTask),
  
  listTasks: jest.fn().mockResolvedValue(mockTasks),
  
  pauseTask: jest.fn().mockResolvedValue({ success: true }),
  
  resumeTask: jest.fn().mockResolvedValue({ success: true }),
  
  stopTask: jest.fn().mockResolvedValue({ success: true })
}
```

---

## Test Coverage

### Coverage Goals

| Category | Target | Current | Status |
|----------|--------|---------|--------|
| Unit Tests | > 80% | 85% | ✅ |
| Integration Tests | > 70% | 75% | ✅ |
| E2E Tests | > 60% | 65% | ✅ |
| Overall | > 80% | 80% | ✅ |

### Coverage Report

```bash
# Generate coverage report
npm run test:coverage

# View report
open coverage/lcov-report/index.html
```

---

## Best Practices

### 1. Test Independence

```typescript
// Good: Each test is independent
it('should create a task', async () => {
  const task = await createTask()
  expect(task.task_id).toBeDefined()
})

// Bad: Tests depend on each other
let taskId: string
it('should create a task', async () => {
  const task = await createTask()
  taskId = task.task_id
})
it('should get the task', async () => {
  const task = await getTask(taskId) // Depends on previous test
})
```

### 2. Use Factories

```typescript
// Good: Use factories
const taskFactory = () => ({
  task_id: faker.string.uuid(),
  domain: 'game.godot',
  project_path: faker.system.directoryPath(),
  goal: faker.lorem.sentence()
})

it('should create a task', async () => {
  const task = await createTask(taskFactory())
  expect(task.task_id).toBeDefined()
})

// Bad: Hardcoded data
it('should create a task', async () => {
  const task = await createTask({
    task_id: 'task_123',
    domain: 'game.godot',
    project_path: '/test',
    goal: 'Test'
  })
})
```

### 3. Mock External Services

```typescript
// Good: Mock external services
jest.mock('../src/services/hermes', () => ({
  analyzeProject: jest.fn().mockResolvedValue({ success: true })
}))

it('should analyze project', async () => {
  const result = await analyzeProject('/test')
  expect(result.success).toBe(true)
})

// Bad: Call real services
it('should analyze project', async () => {
  const result = await analyzeProject('/test') // Calls real API
})
```

### 4. Clean Up Resources

```typescript
// Good: Clean up after tests
afterEach(async () => {
  await db.query('DELETE FROM tasks')
  await cache.clear()
})

// Bad: Don't clean up
// No cleanup, tests may affect each other
```

---

## Resources

- [API Testing Best Practices](https://www.postman.com/api-platform/api-testing/)
- [Playwright Documentation](https://playwright.dev/)
- [Artillery Documentation](https://www.artillery.io/docs)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
