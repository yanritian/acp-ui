# API Examples

This document provides practical examples of using the Hermes Game Operator API.

## Task Management

### Start a Simple Task

```typescript
import { OperatorApi } from '@/api/operatorApi'

// Start a task to add double jump
const result = await OperatorApi.startTask({
  domain: 'game.godot',
  project_path: '/path/to/godot/project',
  goal: 'Add double jump ability to the player character',
  mode: 'propose_then_apply',
  approval_policy: 'safe_default'
})

console.log(`Task started: ${result.task_id}`)
console.log(`Status: ${result.status}`)
```

### Monitor Task Progress

```typescript
// Get task details
const task = await OperatorApi.getTask('task_123')
console.log(`Task: ${task.goal}`)
console.log(`Status: ${task.status}`)

// List recent events
const events = await OperatorApi.listEvents('task_123', 10)
events.forEach(event => {
  console.log(`[${event.timestamp}] ${event.title}`)
})
```

### Pause and Resume Task

```typescript
// Pause the task
await OperatorApi.pauseTask('task_123')
console.log('Task paused')

// Resume the task
await OperatorApi.resumeTask('task_123')
console.log('Task resumed')
```

### Stop a Task

```typescript
// Stop the task
await OperatorApi.stopTask('task_123')
console.log('Task stopped')

// Get final summary
const summary = await OperatorApi.getTaskSummary('task_123')
console.log(`Summary: ${summary.summary}`)
console.log(`Files modified: ${summary.files_modified.length}`)
```

## File Operations

### Read a File

```typescript
// Read a GDScript file
const result = await OperatorApi.fileRead(
  '/path/to/project/scripts/Player.gd',
  ['/path/to/project']
)

console.log(`File: ${result.path}`)
console.log(`Size: ${result.size_bytes} bytes`)
console.log(`Lines: ${result.line_count}`)
console.log(`Content:\n${result.content}`)
```

### Preview a Patch

```typescript
// Preview changes without applying
const preview = await OperatorApi.filePatchPreview(
  '/path/to/project/scripts/Player.gd',
  newContent,
  ['/path/to/project']
)

console.log('Diff:')
console.log(preview.diff)
console.log(`Lines changed: ${preview.diff.split('\n').length}`)
```

### Apply a Patch

```typescript
// Apply changes with backup
const result = await OperatorApi.filePatch(
  '/path/to/project/scripts/Player.gd',
  newContent,
  ['/path/to/project'],
  true  // create backup
)

console.log(`Success: ${result.success}`)
console.log(`Lines changed: ${result.lines_changed}`)
if (result.backup_path) {
  console.log(`Backup: ${result.backup_path}`)
}
```

### List Files

```typescript
// List project structure
const listing = await OperatorApi.fileList(
  '/path/to/project/scripts',
  ['/path/to/project']
)

console.log('Directories:')
listing.directories.forEach(dir => console.log(`  📁 ${dir}`))

console.log('Files:')
listing.files.forEach(file => console.log(`  📄 ${file}`))
```

## Godot Project Analysis

### Detect Godot Project

```typescript
// Check if directory is a Godot project
const isGodot = await GodotOperatorApi.detectProject('/path/to/project')

if (isGodot) {
  console.log('✅ Valid Godot project')
} else {
  console.log('❌ Not a Godot project')
}
```

### Analyze Project

```typescript
// Analyze project structure
const analysis = await GodotOperatorApi.analyzeProject('/path/to/project')

console.log(`Project: ${analysis.project_name}`)
console.log(`Godot Version: ${analysis.godot_version}`)
console.log(`Main Scene: ${analysis.main_scene}`)
console.log(`Scripts: ${analysis.scripts.length}`)
console.log(`Scenes: ${analysis.scenes.length}`)
console.log(`Assets: ${analysis.assets.length}`)

// Show all scripts
console.log('\nScripts:')
analysis.scripts.forEach(script => {
  console.log(`  📄 ${script}`)
})

// Show all scenes
console.log('\nScenes:')
analysis.scenes.forEach(scene => {
  console.log(`  🎬 ${scene}`)
})
```

## Approval Workflow

### Get Pending Approvals

```typescript
// Get all pending approvals
const approvals = await OperatorApi.getPendingApprovals('task_123')

console.log(`Pending approvals: ${approvals.length}`)

approvals.forEach(approval => {
  console.log(`\nApproval: ${approval.approval_id}`)
  console.log(`  Action: ${approval.action}`)
  console.log(`  Level: ${approval.level}`)
  console.log(`  Title: ${approval.title}`)
  console.log(`  Reason: ${approval.reason}`)
  
  if (approval.preview?.files) {
    console.log(`  Files: ${approval.preview.files.join(', ')}`)
  }
})
```

### Approve an Action

```typescript
// Approve a file modification
await OperatorApi.approve({
  task_id: 'task_123',
  approval_id: 'approval_001',
  decision: 'approve',
  comment: 'Changes look good'
})

console.log('✅ Approved')
```

### Reject an Action

```typescript
// Reject a dangerous operation
await OperatorApi.approve({
  task_id: 'task_123',
  approval_id: 'approval_001',
  decision: 'reject',
  comment: 'Too risky, need different approach'
})

console.log('❌ Rejected')
```

## Complete Workflow Example

```typescript
import { OperatorApi, GodotOperatorApi } from '@/api/operatorApi'

async function executeGameDevTask() {
  try {
    // Step 1: Verify project
    const isGodot = await GodotOperatorApi.detectProject('/path/to/project')
    if (!isGodot) {
      throw new Error('Not a Godot project')
    }
    console.log('✅ Valid Godot project')

    // Step 2: Analyze project
    const analysis = await GodotOperatorApi.analyzeProject('/path/to/project')
    console.log(`Project: ${analysis.project_name}`)
    console.log(`Scripts: ${analysis.scripts.length}`)

    // Step 3: Start task
    const task = await OperatorApi.startTask({
      domain: 'game.godot',
      project_path: '/path/to/project',
      goal: 'Add double jump to player',
      mode: 'propose_then_apply',
      approval_policy: 'safe_default'
    })
    console.log(`✅ Task started: ${task.task_id}`)

    // Step 4: Monitor progress
    let taskStatus = task.status
    while (taskStatus !== 'completed' && taskStatus !== 'failed') {
      const currentTask = await OperatorApi.getTask(task.task_id)
      taskStatus = currentTask.status
      
      const events = await OperatorApi.listEvents(task.task_id, 5)
      console.log(`\nRecent events:`)
      events.forEach(event => {
        console.log(`  [${event.timestamp}] ${event.title}`)
      })
      
      // Check for pending approvals
      const approvals = await OperatorApi.getPendingApprovals(task.task_id)
      if (approvals.length > 0) {
        console.log(`\n⚠️ Pending approvals: ${approvals.length}`)
        // Auto-approve for this example
        for (const approval of approvals) {
          await OperatorApi.approve({
            task_id: task.task_id,
            approval_id: approval.approval_id,
            decision: 'approve'
          })
          console.log(`✅ Approved: ${approval.title}`)
        }
      }
      
      // Wait before next check
      await new Promise(resolve => setTimeout(resolve, 2000))
    }

    // Step 5: Get results
    const summary = await OperatorApi.getTaskSummary(task.task_id)
    console.log(`\n📊 Task Summary:`)
    console.log(`  Status: ${summary.status}`)
    console.log(`  Files Modified: ${summary.files_modified.length}`)
    console.log(`  Duration: ${summary.duration_seconds}s`)
    
    if (summary.files_modified.length > 0) {
      console.log(`\nModified files:`)
      summary.files_modified.forEach(file => {
        console.log(`  📄 ${file}`)
      })
    }
    
    if (taskStatus === 'completed') {
      console.log('\n🎉 Task completed successfully!')
    } else {
      console.log('\n❌ Task failed')
    }

  } catch (error) {
    console.error('Error:', error.message)
  }
}

// Execute the workflow
executeGameDevTask()
```

## Error Handling

```typescript
try {
  const result = await OperatorApi.startTask({
    domain: 'game.godot',
    project_path: '/path/to/project',
    goal: 'Add feature'
  })
} catch (error) {
  // Handle different error types
  switch (error.code) {
    case '1001':
      console.error('Invalid path')
      break
    case '2001':
      console.error('Path outside boundary')
      break
    case '3001':
      console.error('API unavailable')
      break
    case '5001':
      console.error('Agent not initialized')
      break
    default:
      console.error(`Error ${error.code}: ${error.message}`)
  }
  
  // Log error details
  console.error('Category:', error.category)
  console.error('Severity:', error.severity)
  console.error('Recoverable:', error.recoverable)
}
```

## Real-time Event Monitoring

```typescript
// Subscribe to task events
function monitorTask(taskId: string) {
  let lastEventIndex = 0
  
  const interval = setInterval(async () => {
    const events = await OperatorApi.listEvents(taskId, 100)
    const newEvents = events.slice(lastEventIndex)
    
    newEvents.forEach(event => {
      console.log(`[${event.timestamp}] ${event.type}: ${event.title}`)
      
      // Handle specific events
      switch (event.type) {
        case 'file_modified':
          console.log(`  📄 Modified: ${event.payload?.path}`)
          break
        case 'approval_requested':
          console.log(`  ⚠️ Approval needed: ${event.title}`)
          break
        case 'task_completed':
          console.log(`  🎉 Task completed!`)
          clearInterval(interval)
          break
        case 'task_failed':
          console.log(`  ❌ Task failed: ${event.message}`)
          clearInterval(interval)
          break
      }
    })
    
    lastEventIndex = events.length
  }, 1000)
  
  return () => clearInterval(interval)
}

// Start monitoring
const stopMonitoring = monitorTask('task_123')

// Stop monitoring when done
// stopMonitoring()
```

## Batch Operations

```typescript
// Execute multiple tasks in sequence
async function executeBatchTasks(tasks: Array<{ goal: string, project: string }>) {
  const results = []
  
  for (const taskConfig of tasks) {
    console.log(`\nStarting task: ${taskConfig.goal}`)
    
    try {
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: taskConfig.project,
        goal: taskConfig.goal,
        mode: 'propose_then_apply'
      })
      
      // Wait for completion
      let status = task.status
      while (status !== 'completed' && status !== 'failed') {
        await new Promise(resolve => setTimeout(resolve, 2000))
        const current = await OperatorApi.getTask(task.task_id)
        status = current.status
      }
      
      const summary = await OperatorApi.getTaskSummary(task.task_id)
      results.push({
        goal: taskConfig.goal,
        status: summary.status,
        filesModified: summary.files_modified.length
      })
      
    } catch (error) {
      results.push({
        goal: taskConfig.goal,
        status: 'error',
        error: error.message
      })
    }
  }
  
  return results
}

// Execute batch
const batchResults = await executeBatchTasks([
  { goal: 'Add double jump', project: '/path/to/project' },
  { goal: 'Add sprint ability', project: '/path/to/project' },
  { goal: 'Fix enemy AI', project: '/path/to/project' }
])

console.log('\nBatch Results:')
batchResults.forEach(result => {
  console.log(`  ${result.goal}: ${result.status}`)
})
```

## Type Definitions

```typescript
// Task Status
type OperatorTaskStatus = 
  | 'idle'
  | 'planning'
  | 'waiting_approval'
  | 'running'
  | 'paused'
  | 'redirecting'
  | 'cancelling'
  | 'cancelled'
  | 'failed'
  | 'completed'

// Event Types
type OperatorEventType =
  | 'task_created'
  | 'task_started'
  | 'project_analyzed'
  | 'plan_ready'
  | 'approval_requested'
  | 'approval_granted'
  | 'approval_rejected'
  | 'file_read'
  | 'file_modified'
  | 'task_completed'
  | 'task_failed'

// Approval Decision
type ApprovalDecision = 'approve' | 'reject' | 'request_changes'

// Approval Level
type ApprovalLevel = 'silent' | 'notify' | 'approve' | 'forbidden'
```

## See Also

- [API Reference](api.md)
- [User Manual](USER-MANUAL.md)
- [Best Practices](BEST-PRACTICES.md)
- [Troubleshooting](TROUBLESHOOTING.md)

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
