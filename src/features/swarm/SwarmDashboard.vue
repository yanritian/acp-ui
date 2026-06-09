<!-- Swarm Dashboard - Real-time visualization of agent swarm execution -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Types
interface WorkerCapabilities {
  workerId: string
  workerType: string
  capabilities: string[]
  maxComplexity: number
  maxConcurrent: number
  supportsStreaming: boolean
  supportsCancel: boolean
  defaultTimeoutMs: number
}

interface WorkerStatus {
  workerId: string
  workerType: string
  health: 'healthy' | 'busy' | 'starting' | 'stopping' | 'unhealthy' | 'offline'
  pid: number | null
  memoryBytes: number | null
  cpuPercent: number | null
  tasksCompleted: number
  tasksFailed: number
  currentTask: string | null
  startedAt: { timestampMs: number } | null
  lastHeartbeat: { timestampMs: number } | null
}

interface TaskHandle {
  taskId: string
  workerId: string
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled' | 'timeout'
  startedAt: { timestampMs: number }
  output: string
  error: string | null
  pid: number | null
}

interface ShardGroup {
  taskId: string
  originalPrompt: string
  status: 'partitioning' | 'assigning' | 'executing' | 'aggregating' | 'completed' | 'failed' | 'cancelled'
  shardCount: number
  completedCount: number
  failedCount: number
  finalResult: string | null
  errors: string[]
}

interface QueenLease {
  leaseId: string
  queenId: string
  grantedAt: { timestampMs: number }
  ttlSeconds: number
  expiresAt: { timestampMs: number }
  isValid: boolean
  renewalCount: number
}

// State
const workers = ref<WorkerCapabilities[]>([])
const workerStatuses = ref<Map<string, WorkerStatus>>(new Map())
const activeTasks = ref<Map<string, TaskHandle>>(new Map())
const shardGroups = ref<Map<string, ShardGroup>>(new Map())
const queenLease = ref<QueenLease | null>(null)

// Form state
const workerTypeInput = ref<'codex' | 'claude_code'>('codex')
const workerIdInput = ref('')
const taskPromptInput = ref('')
const topologyInput = ref<'star' | 'chain'>('star')

// UI state
const isRegisteringWorker = ref(false)
const isExecutingTask = ref(false)
const selectedWorkerId = ref<string | null>(null)
const eventLog = ref<string[]>([])

// Computed
const healthyWorkers = computed(() =>
  workers.value.filter(w => {
    const status = workerStatuses.value.get(w.workerId)
    return status && (status.health === 'healthy' || status.health === 'busy')
  })
)

const queenWorker = computed(() => {
  if (!queenLease.value?.isValid) return null
  return workers.value.find(w => w.workerId === queenLease.value?.queenId)
})

const swarmStatus = computed(() => {
  if (!queenLease.value?.isValid) return 'no_queen'
  const runningTasks = Array.from(activeTasks.value.values())
    .filter(t => t.status === 'running').length
  return runningTasks > 0 ? 'executing' : 'ready'
})

// Methods
async function registerWorker() {
  if (!workerIdInput.value) {
    eventLog.value.push('[Error] Worker ID required')
    return
  }

  isRegisteringWorker.value = true
  try {
    const capabilities = await invoke<WorkerCapabilities>('swarm_register_worker', {
      workerType: workerTypeInput.value,
      workerId: workerIdInput.value
    })
    workers.value.push(capabilities)
    eventLog.value.push(`[OK] Registered ${capabilities.workerType} (${capabilities.workerId})`)
    workerIdInput.value = ''
  } catch (e) {
    eventLog.value.push(`[Error] Failed to register: ${e}`)
  } finally {
    isRegisteringWorker.value = false
  }
}

async function refreshWorkerStatus() {
  for (const worker of workers.value) {
    try {
      const status = await invoke<WorkerStatus>('swarm_get_worker_status', {
        workerId: worker.workerId
      })
      workerStatuses.value.set(worker.workerId, status)
    } catch (e) {
      eventLog.value.push(`[Warn] Failed to get status for ${worker.workerId}: ${e}`)
    }
  }
}

async function healthCheck(workerId: string) {
  try {
    const healthy = await invoke<boolean>('swarm_health_check', { workerId })
    eventLog.value.push(`[Health] ${workerId}: ${healthy ? 'OK' : 'FAIL'}`)
  } catch (e) {
    eventLog.value.push(`[Error] Health check failed: ${e}`)
  }
}

async function executeTask() {
  if (!taskPromptInput.value) {
    eventLog.value.push('[Error] Task prompt required')
    return
  }

  const availableWorkers = healthyWorkers.value
  if (availableWorkers.length === 0) {
    eventLog.value.push('[Error] No healthy workers available')
    return
  }

  isExecutingTask.value = true

  // Demo: Use Queen to decompose and Workers to execute
  const queen = queenWorker.value || availableWorkers[0]
  const taskId = `task-${Date.now()}`

  try {
    // Step 1: Send task to Queen for decomposition
    eventLog.value.push(`[Queen] Sending task to ${queen.workerId} for decomposition...`)

    const handle = await invoke<TaskHandle>('swarm_send_task', {
      workerId: queen.workerId,
      taskId: `${taskId}-decompose`,
      prompt: `Analyze and decompose this task into subtasks: ${taskPromptInput.value}`,
      workingDir: null,
      timeoutMs: 30000
    })

    activeTasks.value.set(handle.taskId, handle)
    eventLog.value.push(`[Task] ${handle.taskId} started on ${handle.workerId}`)

    // Step 2: Distribute subtasks to other workers (simplified)
    if (topologyInput.value === 'star' && availableWorkers.length > 1) {
      // Assign subtasks to workers
      for (let i = 1; i < Math.min(availableWorkers.length, 4); i++) {
        const subWorker = availableWorkers[i]
        const subTaskId = `${taskId}-sub-${i}`

        const subHandle = await invoke<TaskHandle>('swarm_send_task', {
          workerId: subWorker.workerId,
          taskId: subTaskId,
          prompt: `Execute subtask ${i} of: ${taskPromptInput.value}`,
          workingDir: null,
          timeoutMs: 60000
        })

        activeTasks.value.set(subHandle.taskId, subHandle)
        eventLog.value.push(`[Worker] ${subWorker.workerId} assigned subtask ${i}`)
      }
    }

    eventLog.value.push(`[Swarm] Task ${taskId} executing with ${activeTasks.value.size} workers`)
  } catch (e) {
    eventLog.value.push(`[Error] Task execution failed: ${e}`)
  } finally {
    isExecutingTask.value = false
  }
}

async function getTaskOutput(taskId: string, workerId: string) {
  try {
    const output = await invoke<string>('swarm_get_task_output', {
      workerId,
      taskId
    })
    eventLog.value.push(`[Output] ${taskId}: ${output.substring(0, 200)}...`)
    return output
  } catch (e) {
    eventLog.value.push(`[Error] Failed to get output: ${e}`)
    return null
  }
}

async function cancelTask(taskId: string, workerId: string) {
  try {
    await invoke('swarm_cancel_task', { workerId, taskId })
    const task = activeTasks.value.get(taskId)
    if (task) {
      task.status = 'cancelled'
    }
    eventLog.value.push(`[Cancel] ${taskId} cancelled`)
  } catch (e) {
    eventLog.value.push(`[Error] Cancel failed: ${e}`)
  }
}

async function shutdownWorker(workerId: string) {
  try {
    await invoke('swarm_shutdown_worker', { workerId })
    workers.value = workers.value.filter(w => w.workerId !== workerId)
    workerStatuses.value.delete(workerId)
    eventLog.value.push(`[Shutdown] ${workerId} stopped`)
  } catch (e) {
    eventLog.value.push(`[Error] Shutdown failed: ${e}`)
  }
}

// Event listeners
let unlisten: (() => void)[] = []

onMounted(async () => {
  // Listen to swarm events
  unlisten.push(
    await listen('swarm-task-completed', (event) => {
      const payload = event.payload as { taskId: string; workerId: string; output: string }
      eventLog.value.push(`[Complete] ${payload.taskId} by ${payload.workerId}`)
      const task = activeTasks.value.get(payload.taskId)
      if (task) {
        task.status = 'completed'
        task.output = payload.output
      }
    })
  )

  unlisten.push(
    await listen('swarm-worker-status', (event) => {
      const status = event.payload as WorkerStatus
      workerStatuses.value.set(status.workerId, status)
    })
  )

  unlisten.push(
    await listen('queen-lease-update', (event) => {
      queenLease.value = event.payload as QueenLease
    })
  )

  // Refresh status periodically
  const intervalId = setInterval(refreshWorkerStatus, 5000)
  unlisten.push(() => clearInterval(intervalId))

  // Initial load
  try {
    const registered = await invoke<WorkerCapabilities[]>('swarm_list_workers')
    workers.value = registered
  } catch (e) {
    // No workers registered yet
  }
})

onUnmounted(() => {
  unlisten.forEach(fn => fn())
})

// Helper functions
function getHealthColor(health: string): string {
  const colors: Record<string, string> = {
    healthy: 'text-green-500',
    busy: 'text-blue-500',
    starting: 'text-yellow-500',
    stopping: 'text-orange-500',
    unhealthy: 'text-red-500',
    offline: 'text-gray-400'
  }
  return colors[health] || 'text-gray-400'
}

function formatCapabilities(caps: string[]): string {
  return caps.slice(0, 3).join(', ') + (caps.length > 3 ? '...' : '')
}

function formatBytes(bytes: number | null): string {
  if (!bytes) return 'N/A'
  if (bytes < 1024) return `${bytes}B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`
  return `${(bytes / 1024 / 1024).toFixed(1)}MB`
}
</script>

<template>
  <div class="swarm-dashboard p-6 bg-gray-50 min-h-screen">
    <!-- Header -->
    <header class="mb-6">
      <h1 class="text-2xl font-bold text-gray-800">Swarm Dashboard</h1>
      <p class="text-gray-600">Agent蜂群编排面板 - Real-time visualization</p>
    </header>

    <!-- Swarm Status Banner -->
    <div class="mb-4 p-3 rounded-lg" :class="{
      'bg-green-100 border-green-300': swarmStatus === 'ready',
      'bg-blue-100 border-blue-300': swarmStatus === 'executing',
      'bg-yellow-100 border-yellow-300': swarmStatus === 'no_queen'
    }">
      <div class="flex items-center gap-2">
        <span class="font-semibold">Swarm Status:</span>
        <span class="uppercase">{{ swarmStatus }}</span>
        <span v-if="queenWorker" class="ml-4">
          Queen: <span class="font-mono">{{ queenWorker.workerId }}</span>
          ({{ queenWorker.workerType }})
        </span>
      </div>
    </div>

    <!-- Worker Registration -->
    <section class="mb-6 bg-white rounded-lg shadow p-4">
      <h2 class="text-lg font-semibold mb-3">Register Worker</h2>
      <div class="flex gap-3 items-end">
        <div>
          <label class="block text-sm text-gray-600 mb-1">Worker Type</label>
          <select v-model="workerTypeInput" class="border rounded px-3 py-2">
            <option value="codex">Codex (Executor)</option>
            <option value="claude_code">Claude Code (Orchestrator)</option>
          </select>
        </div>
        <div>
          <label class="block text-sm text-gray-600 mb-1">Worker ID</label>
          <input
            v-model="workerIdInput"
            type="text"
            placeholder="e.g., codex-1"
            class="border rounded px-3 py-2 w-32"
          />
        </div>
        <button
          @click="registerWorker"
          :disabled="isRegisteringWorker"
          class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:opacity-50"
        >
          {{ isRegisteringWorker ? 'Registering...' : 'Register' }}
        </button>
      </div>
    </section>

    <!-- Worker Cards -->
    <section class="mb-6">
      <h2 class="text-lg font-semibold mb-3">Workers ({{ workers.length }})</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="worker in workers"
          :key="worker.workerId"
          class="bg-white rounded-lg shadow p-4 cursor-pointer hover:shadow-md"
          :class="{ 'ring-2 ring-blue-500': selectedWorkerId === worker.workerId }"
          @click="selectedWorkerId = worker.workerId"
        >
          <!-- Worker Header -->
          <div class="flex justify-between items-center mb-2">
            <span class="font-mono font-semibold">{{ worker.workerId }}</span>
            <span
              :class="getHealthColor(workerStatuses.get(worker.workerId)?.health || 'offline')"
              class="font-medium"
            >
              {{ workerStatuses.get(worker.workerId)?.health || 'offline' }}
            </span>
          </div>

          <!-- Worker Type Badge -->
          <div class="mb-2">
            <span
              class="px-2 py-1 rounded text-xs font-medium"
              :class="{
                'bg-purple-100 text-purple-800': worker.workerType === 'claude_code',
                'bg-green-100 text-green-800': worker.workerType === 'codex'
              }"
            >
              {{ worker.workerType }}
            </span>
            <span v-if="queenWorker?.workerId === worker.workerId" class="ml-2 px-2 py-1 rounded text-xs bg-yellow-100 text-yellow-800">
              QUEEN
            </span>
          </div>

          <!-- Capabilities -->
          <div class="text-sm text-gray-600 mb-2">
            {{ formatCapabilities(worker.capabilities) }}
          </div>

          <!-- Stats -->
          <div class="grid grid-cols-2 gap-2 text-sm">
            <div>
              <span class="text-gray-500">Tasks:</span>
              <span class="ml-1">{{ workerStatuses.get(worker.workerId)?.tasksCompleted || 0 }}/{{ workerStatuses.get(worker.workerId)?.tasksFailed || 0 }}</span>
            </div>
            <div>
              <span class="text-gray-500">PID:</span>
              <span class="ml-1">{{ workerStatuses.get(worker.workerId)?.pid || 'N/A' }}</span>
            </div>
            <div>
              <span class="text-gray-500">Memory:</span>
              <span class="ml-1">{{ formatBytes(workerStatuses.get(worker.workerId)?.memoryBytes ?? null) }}</span>
            </div>
            <div>
              <span class="text-gray-500">CPU:</span>
              <span class="ml-1">{{ workerStatuses.get(worker.workerId)?.cpuPercent?.toFixed(1) || 'N/A' }}%</span>
            </div>
          </div>

          <!-- Current Task -->
          <div v-if="workerStatuses.get(worker.workerId)?.currentTask" class="mt-2 p-2 bg-blue-50 rounded text-sm">
            <span class="text-gray-500">Current:</span>
            <span class="ml-1 font-mono">{{ workerStatuses.get(worker.workerId)?.currentTask }}</span>
          </div>

          <!-- Actions -->
          <div class="mt-3 flex gap-2">
            <button
              @click.stop="healthCheck(worker.workerId)"
              class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded"
            >
              Health
            </button>
            <button
              @click.stop="shutdownWorker(worker.workerId)"
              class="text-xs px-2 py-1 bg-red-100 hover:bg-red-200 text-red-700 rounded"
            >
              Shutdown
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Task Execution -->
    <section class="mb-6 bg-white rounded-lg shadow p-4">
      <h2 class="text-lg font-semibold mb-3">Execute Task</h2>
      <div class="flex gap-3 items-start">
        <div class="flex-1">
          <label class="block text-sm text-gray-600 mb-1">Task Prompt</label>
          <textarea
            v-model="taskPromptInput"
            placeholder="e.g., Write a React login page with validation"
            class="border rounded px-3 py-2 w-full h-20"
          />
        </div>
        <div>
          <label class="block text-sm text-gray-600 mb-1">Topology</label>
          <select v-model="topologyInput" class="border rounded px-3 py-2">
            <option value="star">Star (Queen + Workers)</option>
            <option value="chain">Chain (Sequential)</option>
          </select>
        </div>
        <button
          @click="executeTask"
          :disabled="isExecutingTask || healthyWorkers.length === 0"
          class="bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600 disabled:opacity-50"
        >
          {{ isExecutingTask ? 'Executing...' : 'Execute' }}
        </button>
      </div>
      <div v-if="healthyWorkers.length === 0" class="mt-2 text-sm text-yellow-600">
        ⚠️ No healthy workers available. Register workers first.
      </div>
    </section>

    <!-- Active Tasks -->
    <section class="mb-6">
      <h2 class="text-lg font-semibold mb-3">Active Tasks ({{ activeTasks.size }})</h2>
      <div v-if="activeTasks.size === 0" class="text-gray-500 text-sm">
        No active tasks
      </div>
      <div v-else class="space-y-2">
        <div
          v-for="[taskId, task] in activeTasks"
          :key="taskId"
          class="bg-white rounded shadow p-3"
        >
          <div class="flex justify-between items-center">
            <span class="font-mono">{{ taskId }}</span>
            <span
              class="px-2 py-1 rounded text-xs"
              :class="{
                'bg-yellow-100 text-yellow-800': task.status === 'running',
                'bg-green-100 text-green-800': task.status === 'completed',
                'bg-red-100 text-red-800': task.status === 'failed',
                'bg-gray-100 text-gray-800': task.status === 'cancelled'
              }"
            >
              {{ task.status }}
            </span>
          </div>
          <div class="text-sm text-gray-600 mt-1">
            Worker: <span class="font-mono">{{ task.workerId }}</span>
            <span v-if="task.pid" class="ml-2">PID: {{ task.pid }}</span>
          </div>
          <div v-if="task.output" class="mt-2 p-2 bg-gray-50 rounded text-sm max-h-40 overflow-auto">
            {{ task.output }}
          </div>
          <div v-if="task.error" class="mt-2 p-2 bg-red-50 rounded text-sm text-red-700">
            Error: {{ task.error }}
          </div>
          <div class="mt-2 flex gap-2">
            <button
              @click="getTaskOutput(taskId, task.workerId)"
              class="text-xs px-2 py-1 bg-blue-100 hover:bg-blue-200 rounded"
            >
              Get Output
            </button>
            <button
              v-if="task.status === 'running'"
              @click="cancelTask(taskId, task.workerId)"
              class="text-xs px-2 py-1 bg-red-100 hover:bg-red-200 text-red-700 rounded"
            >
              Cancel
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Event Log -->
    <section class="bg-white rounded-lg shadow p-4">
      <h2 class="text-lg font-semibold mb-3">Event Log</h2>
      <div class="bg-gray-900 text-green-400 p-3 rounded font-mono text-sm max-h-60 overflow-auto">
        <div v-for="(log, i) in eventLog.slice(-50)" :key="i" class="mb-1">
          {{ log }}
        </div>
        <div v-if="eventLog.length === 0" class="text-gray-500">
          No events yet
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.swarm-dashboard {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}
</style>