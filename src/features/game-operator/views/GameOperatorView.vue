<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { OperatorApi, GodotOperatorApi } from '@/api/operatorApi'
import { open } from '@tauri-apps/plugin-dialog'
import type { OperatorTask, OperatorEvent, ApprovalRequest } from '@/types/operator'
import OperatorControlBar from '../components/OperatorControlBar.vue'
import ProgressTimeline from '../components/ProgressTimeline.vue'
import PlanPanel from '../components/PlanPanel.vue'
import ApprovalDrawer from '../components/ApprovalDrawer.vue'

// State
const currentTask = ref<OperatorTask | null>(null)
const events = ref<OperatorEvent[]>([])
const pendingApprovals = ref<ApprovalRequest[]>([])
const selectedProjectPath = ref('')
const taskGoal = ref('')
const isLoading = ref(false)
const error = ref<string | null>(null)

// Polling interval for events
let eventPollInterval: number | null = null

// ============================================================================
// Task Management
// ============================================================================

async function handleSelectProject() {
  // Use Tauri dialog to select folder
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Godot Project Directory'
    })

    if (!selected) {
      return // User cancelled
    }

    selectedProjectPath.value = selected as string

    // Detect if it's a Godot project
    const isGodot = await GodotOperatorApi.detectProject(selectedProjectPath.value)
    if (!isGodot) {
      error.value = 'Selected directory is not a Godot project'
      return
    }
    error.value = null
  } catch (e: any) {
    error.value = `Failed to select project: ${e.message}`
  }
}

async function handleStartTask() {
  if (!selectedProjectPath.value || !taskGoal.value) {
    error.value = 'Please select a project and enter a goal'
    return
  }

  isLoading.value = true
  error.value = null

  try {
    const response = await OperatorApi.startTask({
      domain: 'game.godot',
      project_path: selectedProjectPath.value,
      goal: taskGoal.value,
      mode: 'propose_then_apply',
      approval_policy: 'safe_default',
    })

    currentTask.value = {
      task_id: response.task_id,
      domain: 'game.godot',
      project_path: selectedProjectPath.value,
      goal: taskGoal.value,
      status: response.status,
      mode: 'propose_then_apply',
      approval_policy: 'safe_default',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }

    // Start polling for events
    startEventPolling()
  } catch (e: any) {
    error.value = e.message
  } finally {
    isLoading.value = false
  }
}

async function handlePause() {
  if (!currentTask.value) return
  try {
    await OperatorApi.pauseTask(currentTask.value.task_id)
    await refreshTask()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleResume() {
  if (!currentTask.value) return
  try {
    await OperatorApi.resumeTask(currentTask.value.task_id)
    await refreshTask()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleStop() {
  if (!currentTask.value) return
  try {
    await OperatorApi.stopTask(currentTask.value.task_id)
    await refreshTask()
    stopEventPolling()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleApprove(approvalId: string, decision: 'approve' | 'reject') {
  if (!currentTask.value) return
  try {
    await OperatorApi.approve({
      task_id: currentTask.value.task_id,
      approval_id: approvalId,
      decision,
    })
    await refreshApprovals()
  } catch (e: any) {
    error.value = e.message
  }
}

// ============================================================================
// Event Polling
// ============================================================================

function startEventPolling() {
  if (eventPollInterval) return
  eventPollInterval = window.setInterval(async () => {
    await refreshEvents()
    await refreshApprovals()
  }, 2000) // Poll every 2 seconds
}

function stopEventPolling() {
  if (eventPollInterval) {
    clearInterval(eventPollInterval)
    eventPollInterval = null
  }
}

async function refreshTask() {
  if (!currentTask.value) return
  try {
    currentTask.value = await OperatorApi.getTask(currentTask.value.task_id)
    // Stop polling when task reaches terminal state
    const terminalStates = ['completed', 'failed', 'cancelled']
    if (currentTask.value.status && terminalStates.includes(currentTask.value.status)) {
      stopEventPolling()
    }
  } catch (e: any) {
    // Silent fail - task may not exist yet
  }
}

async function refreshEvents() {
  if (!currentTask.value) return
  try {
    events.value = await OperatorApi.listEvents(currentTask.value.task_id, 100)
  } catch (e: any) {
    // Silent fail - events may not be available yet
  }
}

async function refreshApprovals() {
  if (!currentTask.value) return
  try {
    pendingApprovals.value = await OperatorApi.getPendingApprovals(currentTask.value.task_id)
  } catch (e: any) {
    // Silent fail - approvals may not be available yet
  }
}

// ============================================================================
// Lifecycle
// ============================================================================

onMounted(async () => {
  // Load saved tasks from backend
  try {
    const tasks = await OperatorApi.listTasks()
    if (tasks.length > 0) {
      // Find the most recent non-terminal task
      const activeTask = tasks.find(t =>
        !['completed', 'failed', 'cancelled'].includes(t.status)
      )
      if (activeTask) {
        currentTask.value = activeTask
        startEventPolling()
      }
    }
  } catch (e) {
    // Silent fail - tasks may not be available yet
  }
})

onUnmounted(() => {
  stopEventPolling()
})
</script>

<template>
  <div class="game-operator-view">
    <!-- Header -->
    <div class="operator-header">
      <h1>🎮 Hermes Game Operator</h1>
      <p class="subtitle">Godot MVP - Single Task Closed Loop</p>
    </div>

    <!-- Project Selection -->
    <div v-if="!currentTask" class="project-selection">
      <div class="form-group">
        <label>Godot Project Path</label>
        <div class="path-input">
          <input
            v-model="selectedProjectPath"
            type="text"
            placeholder="/path/to/your/godot/project"
            class="path-field"
          />
          <button @click="handleSelectProject" class="btn-secondary">
            Browse...
          </button>
        </div>
      </div>

      <div class="form-group">
        <label>Task Goal</label>
        <textarea
          v-model="taskGoal"
          placeholder="e.g., Add double jump to the player character"
          rows="3"
          class="goal-field"
        ></textarea>
      </div>

      <button
        @click="handleStartTask"
        :disabled="isLoading || !selectedProjectPath || !taskGoal"
        class="btn-primary"
      >
        {{ isLoading ? 'Starting...' : 'Start Task' }}
      </button>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <!-- Task Execution View -->
    <div v-else class="task-execution">
      <!-- Control Bar -->
      <OperatorControlBar
        :task="currentTask"
        @pause="handlePause"
        @resume="handleResume"
        @stop="handleStop"
      />

      <!-- Main Content -->
      <div class="operator-content">
        <!-- Left: Plan Panel -->
        <div class="left-panel">
          <PlanPanel :task="currentTask" :events="events" />
        </div>

        <!-- Center: Progress Timeline -->
        <div class="center-panel">
          <ProgressTimeline :events="events" />
        </div>

        <!-- Right: Approval Drawer -->
        <div class="right-panel">
          <ApprovalDrawer
            :approvals="pendingApprovals"
            @approve="handleApprove"
          />
        </div>
      </div>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.game-operator-view {
  padding: 2rem;
  max-width: 1400px;
  margin: 0 auto;
}

.operator-header {
  margin-bottom: 2rem;
}

.operator-header h1 {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.subtitle {
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.project-selection {
  max-width: 600px;
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
}

.path-input {
  display: flex;
  gap: 0.5rem;
}

.path-field,
.goal-field {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: monospace;
}

.goal-field {
  resize: vertical;
}

.btn-primary,
.btn-secondary {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.btn-primary {
  background: var(--bg-primary);
  color: white;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.error-message {
  margin-top: 1rem;
  padding: 0.75rem;
  background: var(--bg-danger);
  color: white;
  border-radius: 4px;
}

.task-execution {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.operator-content {
  display: grid;
  grid-template-columns: 300px 1fr 350px;
  gap: 1.5rem;
  min-height: 600px;
}

.left-panel,
.center-panel,
.right-panel {
  background: var(--bg-sidebar);
  border-radius: 8px;
  padding: 1rem;
}

@media (max-width: 1200px) {
  .operator-content {
    grid-template-columns: 1fr;
  }
}
</style>
