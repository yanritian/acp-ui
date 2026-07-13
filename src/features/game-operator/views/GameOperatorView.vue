<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { OperatorApi, GodotOperatorApi } from '@/api/operatorApi'
import { OperatorRemoteApi } from '@/api/operatorRemoteApi'
import { open } from '@tauri-apps/plugin-dialog'
import type {
  OperatorTask,
  OperatorEvent,
  ApprovalRequest,
  ApprovalDecision,
  RemotePlatformCapability,
} from '@/types/operator'
import OperatorControlBar from '../components/OperatorControlBar.vue'
import ProgressTimeline from '../components/ProgressTimeline.vue'
import PlanPanel from '../components/PlanPanel.vue'
import ApprovalDrawer from '../components/ApprovalDrawer.vue'

const { t } = useI18n()

const currentTask = ref<OperatorTask | null>(null)
const events = ref<OperatorEvent[]>([])
const pendingApprovals = ref<ApprovalRequest[]>([])
const selectedProjectPath = ref('')
const taskGoal = ref('')
const redirectGoal = ref('')
const isLoading = ref(false)
const isRedirecting = ref(false)
const error = ref<string | null>(null)
const remoteStatus = ref<'checking' | 'online' | 'offline'>('checking')
const remoteError = ref<string | null>(null)
const remotePlatforms = ref<RemotePlatformCapability[]>([])

const terminalStates = ['completed', 'failed', 'cancelled']
const isTerminalTask = computed(() => {
  return currentTask.value ? terminalStates.includes(currentTask.value.status) : false
})
const remoteStatusLabel = computed(() => {
  if (remoteStatus.value === 'online') return 'Remote online'
  if (remoteStatus.value === 'offline') return 'Remote offline'
  return 'Remote checking'
})
const latestEventTitle = computed(() => {
  return events.value[events.value.length - 1]?.title ?? 'No events yet'
})

let eventPollInterval: number | null = null

async function handleSelectProject() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Godot Project Directory',
    })

    if (!selected) {
      return
    }

    selectedProjectPath.value = selected as string

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

    await refreshSnapshot()
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
    await refreshSnapshot()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleResume() {
  if (!currentTask.value) return
  try {
    await OperatorApi.resumeTask(currentTask.value.task_id)
    await refreshSnapshot()
    startEventPolling()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleStop() {
  if (!currentTask.value) return
  try {
    await OperatorApi.stopTask(currentTask.value.task_id)
    await refreshSnapshot()
    startEventPolling()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleApprove(approvalId: string, decision: ApprovalDecision) {
  if (!currentTask.value) return
  try {
    await OperatorApi.approve({
      task_id: currentTask.value.task_id,
      approval_id: approvalId,
      decision,
    })
    await refreshSnapshot()
    startEventPolling()
  } catch (e: any) {
    error.value = e.message
  }
}

async function handleRedirect() {
  if (!currentTask.value || !redirectGoal.value.trim()) return
  isRedirecting.value = true
  error.value = null

  try {
    await OperatorApi.redirectTask({
      task_id: currentTask.value.task_id,
      new_goal: redirectGoal.value.trim(),
      preserve_completed_work: true,
    })
    redirectGoal.value = ''
    pendingApprovals.value = []
    await refreshSnapshot()
    startEventPolling()
  } catch (e: any) {
    error.value = e.message
  } finally {
    isRedirecting.value = false
  }
}

function startEventPolling() {
  if (eventPollInterval) return
  eventPollInterval = window.setInterval(async () => {
    await refreshOperatorState()
  }, 2000)
}

function stopEventPolling() {
  if (eventPollInterval) {
    clearInterval(eventPollInterval)
    eventPollInterval = null
  }
}

async function refreshSnapshot() {
  await Promise.all([
    refreshTask(),
    refreshEvents(),
    refreshApprovals(),
  ])
}

async function discoverActiveTask(): Promise<boolean> {
  try {
    const tasks = await OperatorApi.listTasks()
    const activeTask = tasks.find(task => !terminalStates.includes(task.status))
    if (!activeTask || activeTask.task_id === currentTask.value?.task_id) {
      return false
    }

    currentTask.value = activeTask
    await refreshSnapshot()
    return true
  } catch {
    // The backend may still be starting. The next polling cycle retries.
    return false
  }
}

async function refreshOperatorState() {
  if (!currentTask.value || isTerminalTask.value) {
    const discovered = await discoverActiveTask()
    if (discovered || !currentTask.value || isTerminalTask.value) {
      return
    }
  }

  await refreshSnapshot()
}

async function refreshTask() {
  if (!currentTask.value) return
  try {
    currentTask.value = await OperatorApi.getTask(currentTask.value.task_id)
  } catch {
    // Task may not be available yet during startup.
  }
}

async function refreshEvents() {
  if (!currentTask.value) return
  try {
    events.value = await OperatorApi.listEvents(currentTask.value.task_id, 100)
  } catch {
    // Events may not be available yet during startup.
  }
}

async function refreshApprovals() {
  if (!currentTask.value) return
  try {
    pendingApprovals.value = await OperatorApi.getPendingApprovals(currentTask.value.task_id)
  } catch {
    // Approvals may not be available yet during startup.
  }
}

async function refreshRemotePlatforms() {
  remoteStatus.value = 'checking'
  remoteError.value = null

  try {
    remotePlatforms.value = await OperatorRemoteApi.getPlatforms()
    remoteStatus.value = 'online'
  } catch (e: any) {
    remotePlatforms.value = []
    remoteStatus.value = 'offline'
    remoteError.value = e.message ?? String(e)
  }
}

onMounted(async () => {
  await refreshRemotePlatforms()
  await discoverActiveTask()
  startEventPolling()
})

onUnmounted(() => {
  stopEventPolling()
})
</script>

<template>
  <div class="game-operator-view">
    <div class="operator-header">
      <h1>{{ t('gameOperator.title') }}</h1>
      <p class="subtitle">{{ t('gameOperator.subtitle') }}</p>
      <div class="remote-status-bar" :class="[`remote-${remoteStatus}`]">
        <div class="remote-main">
          <span class="remote-dot"></span>
          <span>{{ remoteStatusLabel }}</span>
          <span v-if="remotePlatforms.length" class="remote-count">
            {{ remotePlatforms.length }} platforms
          </span>
        </div>
        <div class="remote-detail">
          {{ remoteError || latestEventTitle }}
        </div>
      </div>
    </div>

    <div v-if="!currentTask" class="project-selection">
      <div class="form-group">
        <label>{{ t('gameOperator.projectPath') }}</label>
        <div class="path-input">
          <input
            v-model="selectedProjectPath"
            type="text"
            :placeholder="t('gameOperator.projectPathPlaceholder')"
            class="path-field"
          />
          <button @click="handleSelectProject" class="btn-secondary">
            {{ t('gameOperator.selectProject') }}
          </button>
        </div>
      </div>

      <div class="form-group">
        <label>{{ t('gameOperator.taskGoal') }}</label>
        <textarea
          v-model="taskGoal"
          :placeholder="t('gameOperator.goalPlaceholder')"
          rows="3"
          class="goal-field"
        ></textarea>
      </div>

      <button
        @click="handleStartTask"
        :disabled="isLoading || !selectedProjectPath || !taskGoal"
        class="btn-primary"
      >
        {{ isLoading ? t('gameOperator.starting') : t('gameOperator.startTask') }}
      </button>

      <div v-if="error" class="error-message">
        {{ error }}
      </div>
    </div>

    <div v-else class="task-execution">
      <OperatorControlBar
        :task="currentTask"
        @pause="handlePause"
        @resume="handleResume"
        @stop="handleStop"
      />

      <div class="operator-content">
        <div class="left-panel">
          <PlanPanel :task="currentTask" :events="events" />
        </div>

        <div class="center-panel">
          <ProgressTimeline :events="events" />
        </div>

        <div class="right-panel">
          <div v-if="!isTerminalTask" class="redirect-section">
            <label>{{ t('gameOperator.redirectGoal') }}</label>
            <textarea
              v-model="redirectGoal"
              rows="3"
              class="redirect-field"
              :placeholder="t('gameOperator.goalPlaceholder')"
            ></textarea>
            <button
              class="btn-secondary redirect-btn"
              :disabled="isRedirecting || !redirectGoal.trim()"
              @click="handleRedirect"
            >
              {{ isRedirecting ? t('gameOperator.redirecting') : t('gameOperator.redirect') }}
            </button>
          </div>

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
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  min-width: 0;
  padding: 2rem;
  max-width: 1400px;
  margin: 0 auto;
  overflow-x: hidden;
  overflow-y: auto;
  container-type: inline-size;
}

.operator-header {
  margin-bottom: 2rem;
}

.operator-header h1 {
  font-size: 2rem;
  margin-bottom: 0.5rem;
  overflow-wrap: anywhere;
}

.subtitle {
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.remote-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-top: 1rem;
  padding: 0.75rem 0;
  border-top: 1px solid var(--border-color);
  border-bottom: 1px solid var(--border-color);
  font-size: 0.9rem;
}

.remote-main {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 0.5rem;
  font-weight: 600;
  flex-wrap: wrap;
}

.remote-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #F59E0B;
}

.remote-online .remote-dot {
  background: #10B981;
}

.remote-offline .remote-dot {
  background: #EF4444;
}

.remote-count,
.remote-detail {
  color: var(--text-secondary);
  font-weight: 400;
}

.remote-detail {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-selection {
  width: 100%;
  min-width: 0;
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
  min-width: 0;
  gap: 0.5rem;
}

.path-field,
.goal-field {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
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

.btn-primary:disabled,
.btn-secondary:disabled {
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
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  gap: 1.5rem;
}

.operator-content {
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  display: grid;
  grid-template-columns: minmax(240px, 300px) minmax(300px, 1fr) minmax(280px, 350px);
  min-width: 0;
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

.right-panel {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.left-panel,
.center-panel,
.right-panel {
  box-sizing: border-box;
  width: 100%;
  max-width: 100%;
  min-width: 0;
}

.redirect-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.redirect-section label {
  font-weight: 500;
}

.redirect-field {
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  min-height: 72px;
  padding: 0.5rem;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  resize: vertical;
}

.redirect-btn {
  align-self: flex-start;
}

@container (max-width: 1050px) {
  .operator-content {
    grid-template-columns: 1fr;
    min-height: 0;
  }

  .remote-status-bar {
    align-items: flex-start;
    flex-direction: column;
  }

  .remote-detail {
    width: 100%;
    overflow-wrap: anywhere;
    white-space: normal;
  }
}

@media (max-width: 800px) {
  .game-operator-view {
    padding: 1rem;
  }

  .operator-header {
    margin-bottom: 1.5rem;
  }

  .operator-header h1 {
    font-size: 1.5rem;
  }
}
</style>
