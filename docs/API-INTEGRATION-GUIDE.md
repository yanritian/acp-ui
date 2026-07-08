# API Integration Guide

## Overview

This guide explains how to integrate with Hermes Game Operator's API, both from the frontend and external applications.

---

## Frontend Integration

### Using the OperatorApi Client

The easiest way to interact with the API from Vue components is using the `OperatorApi` client.

#### Installation

```typescript
import { OperatorApi } from '@/api/operatorApi'
```

#### Basic Usage

```typescript
// Start a task
const task = await OperatorApi.startTask({
  domain: 'game.godot',
  project_path: '/path/to/project',
  goal: 'Add double jump to player',
  mode: 'propose_then_apply',
  approval_policy: 'safe_default'
})

console.log(`Task started: ${task.task_id}`)
```

---

### Task Management

#### Start a Task

```typescript
async function startNewTask() {
  try {
    const response = await OperatorApi.startTask({
      domain: 'game.godot',
      project_path: selectedProjectPath.value,
      goal: taskGoal.value,
      mode: 'propose_then_apply',
      approval_policy: 'safe_default'
    })
    
    currentTask.value = {
      task_id: response.task_id,
      status: response.status,
      // ... other fields
    }
    
    // Start polling for events
    startEventPolling()
  } catch (error) {
    console.error('Failed to start task:', error)
  }
}
```

#### Get Task Details

```typescript
async function refreshTask() {
  if (!currentTask.value) return
  
  try {
    const task = await OperatorApi.getTask(currentTask.value.task_id)
    currentTask.value = task
  } catch (error) {
    console.error('Failed to refresh task:', error)
  }
}
```

#### List All Tasks

```typescript
async function loadTasks() {
  try {
    const tasks = await OperatorApi.listTasks()
    taskList.value = tasks
  } catch (error) {
    console.error('Failed to load tasks:', error)
  }
}
```

---

### Task Control

#### Pause a Task

```typescript
async function pauseTask() {
  if (!currentTask.value) return
  
  try {
    await OperatorApi.pauseTask(currentTask.value.task_id)
    await refreshTask()
  } catch (error) {
    console.error('Failed to pause task:', error)
  }
}
```

#### Resume a Task

```typescript
async function resumeTask() {
  if (!currentTask.value) return
  
  try {
    await OperatorApi.resumeTask(currentTask.value.task_id)
    await refreshTask()
  } catch (error) {
    console.error('Failed to resume task:', error)
  }
}
```

#### Stop a Task

```typescript
async function stopTask() {
  if (!currentTask.value) return
  
  try {
    await OperatorApi.stopTask(currentTask.value.task_id)
    await refreshTask()
    stopEventPolling()
  } catch (error) {
    console.error('Failed to stop task:', error)
  }
}
```

#### Redirect a Task

```typescript
async function redirectTask(newGoal: string) {
  if (!currentTask.value) return
  
  try {
    await OperatorApi.redirectTask({
      task_id: currentTask.value.task_id,
      new_goal: newGoal,
      preserve_completed_work: true
    })
    await refreshTask()
  } catch (error) {
    console.error('Failed to redirect task:', error)
  }
}
```

---

### Approval System

#### Get Pending Approvals

```typescript
async function checkApprovals() {
  if (!currentTask.value) return
  
  try {
    const approvals = await OperatorApi.getPendingApprovals(
      currentTask.value.task_id
    )
    pendingApprovals.value = approvals
  } catch (error) {
    console.error('Failed to check approvals:', error)
  }
}
```

#### Approve an Action

```typescript
async function approveAction(approvalId: string) {
  if (!currentTask.value) return
  
  try {
    await OperatorApi.approve({
      task_id: currentTask.value.task_id,
      approval_id: approvalId,
      decision: 'approve',
      comment: 'Looks good'
    })
    await checkApprovals()
  } catch (error) {
    console.error('Failed to approve action:', error)
  }
}
```

#### Reject an Action

```typescript
async function rejectAction(approvalId: string) {
  if (!currentTask.value) return
  
  try {
    await OperatorApi.approve({
      task_id: currentTask.value.task_id,
      approval_id: approvalId,
      decision: 'reject',
      comment: 'Needs changes'
    })
    await checkApprovals()
  } catch (error) {
    console.error('Failed to reject action:', error)
  }
}
```

---

### Event Monitoring

#### List Events

```typescript
async function refreshEvents() {
  if (!currentTask.value) return
  
  try {
    const events = await OperatorApi.listEvents(
      currentTask.value.task_id,
      100
    )
    eventList.value = events
  } catch (error) {
    console.error('Failed to refresh events:', error)
  }
}
```

#### Poll Events

```typescript
let pollInterval: number | null = null

function startEventPolling() {
  if (pollInterval) return
  
  pollInterval = window.setInterval(async () => {
    await refreshEvents()
    await checkApprovals()
    await refreshTask()
  }, 2000) // Poll every 2 seconds
}

function stopEventPolling() {
  if (pollInterval) {
    clearInterval(pollInterval)
    pollInterval = null
  }
}
```

---

### File Operations

#### Read a File

```typescript
async function readFile(path: string) {
  if (!currentTask.value) return
  
  try {
    const result = await OperatorApi.fileRead(
      currentTask.value.task_id,
      path
    )
    console.log(`File content: ${result.content}`)
    console.log(`Lines: ${result.line_count}`)
  } catch (error) {
    console.error('Failed to read file:', error)
  }
}
```

#### Preview a Patch

```typescript
async function previewPatch(path: string, newContent: string) {
  if (!currentTask.value) return
  
  try {
    const preview = await OperatorApi.filePatchPreview(
      currentTask.value.task_id,
      path,
      newContent
    )
    console.log(`Diff:\n${preview.diff}`)
  } catch (error) {
    console.error('Failed to preview patch:', error)
  }
}
```

#### Apply a Patch

```typescript
async function applyPatch(path: string, newContent: string) {
  if (!currentTask.value) return
  
  try {
    const result = await OperatorApi.filePatch(
      currentTask.value.task_id,
      path,
      newContent,
      true // create backup
    )
    console.log(`Modified ${result.lines_changed} lines`)
    if (result.backup_path) {
      console.log(`Backup: ${result.backup_path}`)
    }
  } catch (error) {
    console.error('Failed to apply patch:', error)
  }
}
```

#### List Files

```typescript
async function listFiles(path: string) {
  if (!currentTask.value) return
  
  try {
    const result = await OperatorApi.fileList(
      currentTask.value.task_id,
      path
    )
    console.log('Directories:', result.directories)
    console.log('Files:', result.files)
  } catch (error) {
    console.error('Failed to list files:', error)
  }
}
```

---

### Task Summary

#### Get Task Summary

```typescript
async function getSummary() {
  if (!currentTask.value) return
  
  try {
    const summary = await OperatorApi.getTaskSummary(
      currentTask.value.task_id
    )
    console.log(`Status: ${summary.status}`)
    console.log(`Files modified: ${summary.files_modified.length}`)
    console.log(`Duration: ${summary.duration_seconds}s`)
    console.log(`Summary: ${summary.summary}`)
  } catch (error) {
    console.error('Failed to get summary:', error)
  }
}
```

---

## Godot-Specific API

### Detect Godot Project

```typescript
async function detectProject(path: string) {
  try {
    const isGodot = await GodotOperatorApi.detectProject(path)
    if (isGodot) {
      console.log('Valid Godot project')
    } else {
      console.log('Not a Godot project')
    }
  } catch (error) {
    console.error('Failed to detect project:', error)
  }
}
```

### Analyze Godot Project

```typescript
async function analyzeProject(path: string) {
  try {
    const analysis = await GodotOperatorApi.analyzeProject(path)
    console.log(`Project: ${analysis.project_name}`)
    console.log(`Godot Version: ${analysis.godot_version}`)
    console.log(`Scripts: ${analysis.scripts.length}`)
    console.log(`Scenes: ${analysis.scenes.length}`)
    console.log(`Player Controllers: ${analysis.player_controllers.length}`)
  } catch (error) {
    console.error('Failed to analyze project:', error)
  }
}
```

---

## Error Handling

### Try-Catch Pattern

```typescript
async function safeApiCall() {
  try {
    const result = await OperatorApi.someMethod()
    // Handle success
  } catch (error) {
    if (error.code === '1001') {
      // Handle validation error
    } else if (error.code === '2001') {
      // Handle security error
    } else if (error.code === '3001') {
      // Handle network error
    } else {
      // Handle other errors
      console.error('Unexpected error:', error)
    }
  }
}
```

### Error Categories

| Code | Category | Description |
|------|----------|-------------|
| 1xxx | Validation | Invalid input |
| 2xxx | Security | Security violation |
| 3xxx | Network | Network error |
| 4xxx | FileSystem | File system error |
| 5xxx | Agent | Agent error |
| 6xxx | State | Invalid state |
| 7xxx | Approval | Approval error |
| 8xxx | Config | Configuration error |

---

## Type Definitions

### Task Status

```typescript
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
```

### Event Types

```typescript
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
```

---

## Best Practices

### 1. Always Handle Errors

```typescript
// Good
try {
  await OperatorApi.someMethod()
} catch (error) {
  showError(error.message)
}

// Bad
await OperatorApi.someMethod() // No error handling
```

### 2. Poll Responsibly

```typescript
// Good: Poll with interval
const pollInterval = setInterval(async () => {
  await refreshEvents()
}, 2000)

// Bad: Poll in tight loop
while (true) {
  await refreshEvents()
}
```

### 3. Clean Up Resources

```typescript
// Good: Clean up on unmount
onUnmounted(() => {
  stopEventPolling()
})

// Bad: Never clean up
// No cleanup
```

### 4. Validate Inputs

```typescript
// Good: Validate before calling API
if (!taskId || !approvalId) {
  showError('Missing required parameters')
  return
}

await OperatorApi.approve({ task_id: taskId, approval_id: approvalId })

// Bad: Don't validate
await OperatorApi.approve({ task_id: '', approval_id: '' })
```

### 5. Show Loading States

```typescript
// Good: Show loading state
isLoading.value = true
try {
  await OperatorApi.someMethod()
} finally {
  isLoading.value = false
}

// Bad: No loading state
await OperatorApi.someMethod()
```

---

## Examples

### Complete Task Workflow

```typescript
async function executeCompleteWorkflow() {
  try {
    // Step 1: Start task
    const task = await OperatorApi.startTask({
      domain: 'game.godot',
      project_path: '/path/to/project',
      goal: 'Add double jump',
      mode: 'propose_then_apply',
      approval_policy: 'safe_default'
    })
    
    console.log(`Task started: ${task.task_id}`)
    
    // Step 2: Monitor progress
    const pollInterval = setInterval(async () => {
      const currentTask = await OperatorApi.getTask(task.task_id)
      console.log(`Status: ${currentTask.status}`)
      
      // Check for approvals
      const approvals = await OperatorApi.getPendingApprovals(task.task_id)
      for (const approval of approvals) {
        // Auto-approve for this example
        await OperatorApi.approve({
          task_id: task.task_id,
          approval_id: approval.approval_id,
          decision: 'approve'
        })
      }
      
      // Check if complete
      if (currentTask.status === 'completed' || currentTask.status === 'failed') {
        clearInterval(pollInterval)
        
        // Get summary
        const summary = await OperatorApi.getTaskSummary(task.task_id)
        console.log(`Summary: ${summary.summary}`)
      }
    }, 2000)
    
  } catch (error) {
    console.error('Workflow failed:', error)
  }
}
```

### Batch Operations

```typescript
async function executeBatchTasks(tasks: Array<{ goal: string, project: string }>) {
  const results = []
  
  for (const taskConfig of tasks) {
    try {
      const task = await OperatorApi.startTask({
        domain: 'game.godot',
        project_path: taskConfig.project,
        goal: taskConfig.goal,
        mode: 'propose_then_apply'
      })
      
      // Wait for completion
      let status = 'running'
      while (status === 'running' || status === 'paused') {
        await new Promise(resolve => setTimeout(resolve, 1000))
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
```

---

## Resources

- [API Reference](api.md)
- [API Examples](API-EXAMPLES.md)
- [User Manual](USER-MANUAL.md)
- [Best Practices](BEST-PRACTICES.md)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
