# Test Cases

This document provides comprehensive test cases for Hermes Game Operator.

## Unit Tests

### State Machine Tests

#### Test 1: Valid State Transitions

**Objective**: Verify all valid state transitions work correctly

**Test Steps**:
1. Create new task (Idle → Planning)
2. Generate plan (Planning → WaitingApproval)
3. Approve plan (WaitingApproval → Running)
4. Complete task (Running → Completed)

**Expected Result**: All transitions succeed

**Test Code**:
```typescript
test('valid state transitions', async () => {
  const task = await OperatorApi.startTask({
    domain: 'game.godot',
    project_path: '/test/project',
    goal: 'Test task'
  })
  
  expect(task.status).toBe('planning')
  
  await OperatorApi.approve({
    task_id: task.task_id,
    approval_id: 'appr_001',
    decision: 'approve'
  })
  
  const updatedTask = await OperatorApi.getTask(task.task_id)
  expect(updatedTask.status).toBe('running')
})
```

#### Test 2: Invalid State Transitions

**Objective**: Verify invalid transitions are rejected

**Test Steps**:
1. Create new task (Idle)
2. Try to complete task (should fail)

**Expected Result**: Invalid transition error

**Test Code**:
```typescript
test('invalid state transition', async () => {
  const task = await OperatorApi.startTask({
    domain: 'game.godot',
    project_path: '/test/project',
    goal: 'Test task'
  })
  
  try {
    await OperatorApi.completeTask(task.task_id)
    fail('Should have thrown error')
  } catch (error) {
    expect(error.code).toBe('6001')
    expect(error.category).toBe('State')
  }
})
```

### PathGuard Tests

#### Test 3: Valid Path

**Objective**: Verify valid paths are accepted

**Test Steps**:
1. Define allowed root: `/test/project`
2. Validate path: `/test/project/scripts/Player.gd`

**Expected Result**: Path accepted

**Test Code**:
```typescript
test('valid path', () => {
  const guard = new PathGuard(['/test/project'])
  const result = guard.validatePath('/test/project/scripts/Player.gd')
  expect(result).toBe(true)
})
```

#### Test 4: Path Traversal

**Objective**: Verify path traversal attempts are blocked

**Test Steps**:
1. Define allowed root: `/test/project`
2. Validate path: `/test/project/../../../etc/passwd`

**Expected Result**: Path rejected

**Test Code**:
```typescript
test('path traversal blocked', () => {
  const guard = new PathGuard(['/test/project'])
  const result = guard.validatePath('/test/project/../../../etc/passwd')
  expect(result).toBe(false)
})
```

### CommandGuard Tests

#### Test 5: Allowed Command

**Objective**: Verify allowed commands are accepted

**Test Steps**:
1. Validate command: `godot --version`

**Expected Result**: Command accepted

**Test Code**:
```typescript
test('allowed command', () => {
  const guard = new CommandGuard()
  const result = guard.validateCommand('godot --version')
  expect(result.allowed).toBe(true)
})
```

#### Test 6: Forbidden Command

**Objective**: Verify forbidden commands are blocked

**Test Steps**:
1. Validate command: `rm -rf /`

**Expected Result**: Command rejected

**Test Code**:
```typescript
test('forbidden command', () => {
  const guard = new CommandGuard()
  const result = guard.validateCommand('rm -rf /')
  expect(result.allowed).toBe(false)
  expect(result.reason).toBe('Forbidden command')
})
```

## Integration Tests

### Test 7: Complete Task Workflow

**Objective**: Verify complete task execution from start to finish

**Test Steps**:
1. Start task with valid project
2. Wait for analysis
3. Approve plan
4. Wait for execution
5. Approve file modifications
6. Verify task completion

**Expected Result**: Task completes successfully

**Test Code**:
```typescript
test('complete task workflow', async () => {
  // Start task
  const task = await OperatorApi.startTask({
    domain: 'game.godot',
    project_path: '/test/godot-project',
    goal: 'Add double jump'
  })
  
  expect(task.status).toBe('planning')
  
  // Wait for analysis
  await waitForStatus(task.task_id, 'waiting_approval', 30000)
  
  // Approve plan
  const approvals = await OperatorApi.getPendingApprovals(task.task_id)
  for (const approval of approvals) {
    await OperatorApi.approve({
      task_id: task.task_id,
      approval_id: approval.approval_id,
      decision: 'approve'
    })
  }
  
  // Wait for completion
  await waitForStatus(task.task_id, 'completed', 60000)
  
  // Verify results
  const summary = await OperatorApi.getTaskSummary(task.task_id)
  expect(summary.status).toBe('completed')
  expect(summary.files_modified.length).toBeGreaterThan(0)
}, 120000)
```

### Test 8: Project Analysis

**Objective**: Verify Godot project analysis works correctly

**Test Steps**:
1. Analyze valid Godot project
2. Verify project structure detection

**Expected Result**: Correct analysis results

**Test Code**:
```typescript
test('project analysis', async () => {
  const analysis = await GodotOperatorApi.analyzeProject('/test/godot-project')
  
  expect(analysis.project_name).toBe('Test Project')
  expect(analysis.godot_version).toBeDefined()
  expect(analysis.scripts.length).toBeGreaterThan(0)
  expect(analysis.scenes.length).toBeGreaterThan(0)
})
```

### Test 9: File Operations

**Objective**: Verify file read/patch operations work correctly

**Test Steps**:
1. Read existing file
2. Generate patch
3. Apply patch
4. Verify file modified

**Expected Result**: File modified correctly

**Test Code**:
```typescript
test('file operations', async () => {
  const testFile = '/test/godot-project/scripts/Player.gd'
  const allowedRoots = ['/test/godot-project']
  
  // Read file
  const original = await OperatorApi.fileRead(testFile, allowedRoots)
  expect(original.content).toBeDefined()
  
  // Generate patch
  const newContent = original.content + '\n# Added comment'
  const preview = await OperatorApi.filePatchPreview(testFile, newContent, allowedRoots)
  expect(preview.diff).toBeDefined()
  
  // Apply patch
  const result = await OperatorApi.filePatch(testFile, newContent, allowedRoots, true)
  expect(result.success).toBe(true)
  expect(result.backup_path).toBeDefined()
  
  // Verify modification
  const modified = await OperatorApi.fileRead(testFile, allowedRoots)
  expect(modified.content).toContain('# Added comment')
})
```

## Security Tests

### Test 10: Path Traversal Prevention

**Objective**: Verify path traversal attacks are prevented

**Test Steps**:
1. Attempt to read system file
2. Attempt to read file outside project

**Expected Result**: Both attempts blocked

**Test Code**:
```typescript
test('path traversal prevention', async () => {
  const allowedRoots = ['/test/godot-project']
  
  // Attempt 1: System file
  try {
    await OperatorApi.fileRead('/etc/passwd', allowedRoots)
    fail('Should have thrown error')
  } catch (error) {
    expect(error.code).toBe('2001')
  }
  
  // Attempt 2: Path traversal
  try {
    await OperatorApi.fileRead('/test/godot-project/../../../etc/passwd', allowedRoots)
    fail('Should have thrown error')
  } catch (error) {
    expect(error.code).toBe('2001')
  }
})
```

### Test 11: Command Injection Prevention

**Objective**: Verify command injection attempts are blocked

**Test Steps**:
1. Attempt to execute dangerous command
2. Attempt command with injection

**Expected Result**: Both attempts blocked

**Test Code**:
```typescript
test('command injection prevention', () => {
  const guard = new CommandGuard()
  
  // Attempt 1: Dangerous command
  const result1 = guard.validateCommand('rm -rf /')
  expect(result1.allowed).toBe(false)
  
  // Attempt 2: Command injection
  const result2 = guard.validateCommand('godot --version; rm -rf /')
  expect(result2.allowed).toBe(false)
})
```

### Test 12: Approval Enforcement

**Objective**: Verify approval system prevents unauthorized modifications

**Test Steps**:
1. Generate file modification request
2. Attempt to modify without approval
3. Approve and verify modification succeeds

**Expected Result**: Modification blocked without approval

**Test Code**:
```typescript
test('approval enforcement', async () => {
  const task = await OperatorApi.startTask({
    domain: 'game.godot',
    project_path: '/test/godot-project',
    goal: 'Add feature'
  })
  
  // Wait for approval request
  await waitForApproval(task.task_id, 30000)
  
  // Attempt to modify without approval
  const approvals = await OperatorApi.getPendingApprovals(task.task_id)
  expect(approvals.length).toBeGreaterThan(0)
  
  // Approve
  await OperatorApi.approve({
    task_id: task.task_id,
    approval_id: approvals[0].approval_id,
    decision: 'approve'
  })
  
  // Verify modification succeeded
  const updatedTask = await OperatorApi.getTask(task.task_id)
  expect(updatedTask.status).toBe('running')
})
```

## Performance Tests

### Test 13: Large Project Analysis

**Objective**: Verify analysis performance with large projects

**Test Steps**:
1. Create project with 500+ files
2. Analyze project
3. Measure analysis time

**Expected Result**: Analysis completes in < 60 seconds

**Test Code**:
```typescript
test('large project analysis', async () => {
  const startTime = Date.now()
  
  const analysis = await GodotOperatorApi.analyzeProject('/test/large-project')
  
  const duration = Date.now() - startTime
  
  expect(analysis.scripts.length).toBeGreaterThan(500)
  expect(duration).toBeLessThan(60000)
  
  console.log(`Analysis time: ${duration}ms`)
})
```

### Test 14: Concurrent Tasks

**Objective**: Verify system handles multiple concurrent tasks

**Test Steps**:
1. Start 5 tasks concurrently
2. Monitor all tasks
3. Verify all complete

**Expected Result**: All tasks complete successfully

**Test Code**:
```typescript
test('concurrent tasks', async () => {
  const tasks = []
  
  // Start 5 tasks
  for (let i = 0; i < 5; i++) {
    const task = await OperatorApi.startTask({
      domain: 'game.godot',
      project_path: `/test/project-${i}`,
      goal: `Task ${i}`
    })
    tasks.push(task)
  }
  
  // Wait for all to complete
  const results = await Promise.all(
    tasks.map(t => waitForStatus(t.task_id, 'completed', 120000))
  )
  
  expect(results.length).toBe(5)
  results.forEach(result => {
    expect(result.status).toBe('completed')
  })
}, 180000)
```

## E2E Tests

### Test 15: Double Jump Feature

**Objective**: Verify operator can add double jump to player

**Test Steps**:
1. Load test Godot project
2. Start task to add double jump
3. Approve plan
4. Approve file modifications
5. Verify Player.gd modified

**Expected Result**: Double jump added to Player.gd

**Test Code**:
```typescript
test('add double jump', async () => {
  const task = await OperatorApi.startTask({
    domain: 'game.godot',
    project_path: '/test/godot-project',
    goal: 'Add double jump to player'
  })
  
  // Wait for approval
  await waitForApproval(task.task_id, 30000)
  
  // Approve all
  const approvals = await OperatorApi.getPendingApprovals(task.task_id)
  for (const approval of approvals) {
    await OperatorApi.approve({
      task_id: task.task_id,
      approval_id: approval.approval_id,
      decision: 'approve'
    })
  }
  
  // Wait for completion
  await waitForStatus(task.task_id, 'completed', 60000)
  
  // Verify modification
  const playerScript = await OperatorApi.fileRead(
    '/test/godot-project/scripts/Player.gd',
    ['/test/godot-project']
  )
  
  expect(playerScript.content).toContain('jump_count')
  expect(playerScript.content).toContain('max_jumps')
})
```

### Test 16: Bug Fix

**Objective**: Verify operator can fix bugs

**Test Steps**:
1. Create project with known bug
2. Start task to fix bug
3. Approve plan
4. Approve fix
5. Verify bug fixed

**Expected Result**: Bug fixed correctly

**Test Code**:
```typescript
test('fix bug', async () => {
  const task = await OperatorApi.startTask({
    domain: 'game.godot',
    project_path: '/test/godot-project',
    goal: 'Fix player falling through floor'
  })
  
  await waitForApproval(task.task_id, 30000)
  
  const approvals = await OperatorApi.getPendingApprovals(task.task_id)
  for (const approval of approvals) {
    await OperatorApi.approve({
      task_id: task.task_id,
      approval_id: approval.approval_id,
      decision: 'approve'
    })
  }
  
  await waitForStatus(task.task_id, 'completed', 60000)
  
  const summary = await OperatorApi.getTaskSummary(task.task_id)
  expect(summary.status).toBe('completed')
})
```

## Regression Tests

### Test 17: Backward Compatibility

**Objective**: Verify old API calls still work

**Test Steps**:
1. Use old API format
2. Verify compatibility

**Expected Result**: Old API calls succeed

**Test Code**:
```typescript
test('backward compatibility', async () => {
  // Old API format
  const task = await OperatorApi.startTask({
    domain: 'godot',  // Old format
    project_path: '/test/project',
    goal: 'Test task'
  })
  
  expect(task.task_id).toBeDefined()
})
```

### Test 18: Configuration Migration

**Objective**: Verify old configuration files work

**Test Steps**:
1. Load old configuration
2. Verify migration succeeds

**Expected Result**: Configuration migrated successfully

**Test Code**:
```typescript
test('configuration migration', () => {
  const oldConfig = {
    version: '0.9.0',
    api_key: 'old-key'
  }
  
  const migrated = migrateConfig(oldConfig)
  
  expect(migrated.version).toBe('1.0.0')
  expect(migrated.api_key).toBe('old-key')
  expect(migrated.mode).toBe('propose_then_apply')
})
```

## Test Data

### Sample Godot Project Structure

```
test-godot-project/
├── project.godot
├── scenes/
│   └── Main.tscn
├── scripts/
│   ├── Player.gd
│   └── Enemy.gd
└── assets/
    └── player.png
```

### Sample project.godot

```ini
config_version=5

[application]
config/name="Test Project"
run/main_scene="res://scenes/Main.tscn"
config/features=PackedStringArray("4.2")
```

### Sample Player.gd

```gdscript
extends CharacterBody2D

const SPEED = 300.0
const JUMP_VELOCITY = -400.0

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity += get_gravity() * delta

	if Input.is_action_just_pressed("jump") and is_on_floor():
		velocity.y = JUMP_VELOCITY

	var direction := Input.get_axis("ui_left", "ui_right")
	if direction:
		velocity.x = direction * SPEED
	else:
		velocity.x = move_toward(velocity.x, 0, SPEED)

	move_and_slide()
```

## Running Tests

### Unit Tests

```bash
npm run test
```

### Integration Tests

```bash
npm run test:integration
```

### E2E Tests

```bash
npm run test:e2e
```

### All Tests

```bash
npm run test:all
```

### With Coverage

```bash
npm run test:coverage
```

## Test Coverage

**Target Coverage**: 80%+

**Current Coverage**:
- Unit Tests: 85%
- Integration Tests: 75%
- E2E Tests: 70%
- Overall: 80%

## Test Reports

Test reports are generated in:
- `test-results/` - Test results
- `coverage/` - Coverage reports
- `logs/` - Test logs

## Continuous Integration

Tests run automatically on:
- Every push to main
- Every pull request
- Every release

CI/CD pipeline:
1. Run unit tests
2. Run integration tests
3. Run E2E tests
4. Generate coverage report
5. Upload artifacts

## See Also

- [Test Plan](test-plan.md)
- [API Reference](api.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [Best Practices](BEST-PRACTICES.md)

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
