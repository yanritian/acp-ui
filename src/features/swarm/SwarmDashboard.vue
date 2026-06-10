<!-- Swarm Dashboard - Real-time visualization of agent swarm execution -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useSwarmStore } from '@/stores/swarm'
import {
  swarmSendTask,
  swarmGetTaskOutput,
  type WorkerCapabilities,
  type WorkerStatus,
  type TaskHandle,
} from '@/lib/swarm-api'
import { isTauriHost } from '@/lib/platform'

const swarm = useSwarmStore()

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

// Local state not in store
const activeTasks = ref<Map<string, TaskHandle>>(new Map())

// Computed
const healthyWorkers = computed(() =>
  swarm.workers.filter(w =>
    w.status?.health === 'healthy' || w.status?.health === 'busy'
  )
)

const queenWorkerCaps = computed(() => {
  if (!swarm.queenWorkerId) return null
  return swarm.workers.find(w => w.id === swarm.queenWorkerId)?.capabilities ?? null
})

const swarmStatusLabel = computed(() => {
  return swarm.swarmStatus
})

// Methods
function log(msg: string) {
  eventLog.value.push(msg)
}

async function registerWorker() {
  if (!workerIdInput.value) {
    log('[Error] Worker ID required')
    return
  }

  isRegisteringWorker.value = true
  try {
    const caps = await swarm.registerWorker(workerTypeInput.value, workerIdInput.value)
    log(`[OK] Registered ${caps.workerType} (${caps.workerId})`)
    workerIdInput.value = ''
  } catch (e) {
    log(`[Error] Failed to register: ${e}`)
  } finally {
    isRegisteringWorker.value = false
  }
}

async function refreshWorkerStatuses() {
  await swarm.loadWorkers()
}

async function healthCheck(workerId: string) {
  try {
    const ok = await swarm.checkWorkerHealth(workerId)
    log(`[Health] ${workerId}: ${ok ? 'OK' : 'FAIL'}`)
  } catch (e) {
    log(`[Error] Health check failed: ${e}`)
  }
}

async function executeTask() {
  if (!taskPromptInput.value) {
    log('[Error] Task prompt required')
    return
  }

  const available = healthyWorkers.value
  if (available.length === 0) {
    log('[Error] No healthy workers available')
    return
  }

  isExecutingTask.value = true

  const queen = queenWorkerCaps.value || available[0].capabilities
  const taskId = `task-${Date.now()}`

  try {
    log(`[Queen] Sending task to ${queen.workerId} for decomposition...`)

    const handle = await swarmSendTask(
      queen.workerId,
      `${taskId}-decompose`,
      `Analyze and decompose this task into subtasks: ${taskPromptInput.value}`
    )

    activeTasks.value.set(handle.taskId, handle)
    log(`[Task] ${handle.taskId} started on ${handle.workerId}`)

    // Star topology: distribute subtasks
    if (topologyInput.value === 'star' && available.length > 1) {
      for (let i = 1; i < Math.min(available.length, 4); i++) {
        const subWorker = available[i].capabilities
        const subTaskId = `${taskId}-sub-${i}`

        const subHandle = await swarmSendTask(
          subWorker.workerId,
          subTaskId,
          `Execute subtask ${i} of: ${taskPromptInput.value}`
        )

        activeTasks.value.set(subHandle.taskId, subHandle)
        log(`[Worker] ${subWorker.workerId} assigned subtask ${i}`)
      }
    }

    log(`[Swarm] Task ${taskId} executing with ${activeTasks.value.size} workers`)
  } catch (e) {
    log(`[Error] Task execution failed: ${e}`)
  } finally {
    isExecutingTask.value = false
  }
}

async function getTaskOutput(taskId: string, workerId: string) {
  try {
    const output = await swarmGetTaskOutput(workerId, taskId)
    log(`[Output] ${taskId}: ${output.substring(0, 200)}...`)
    return output
  } catch (e) {
    log(`[Error] Failed to get output: ${e}`)
    return null
  }
}

async function cancelTask(taskId: string, workerId: string) {
  try {
    await swarm.cancelWorkerTask(workerId, taskId)
    const task = activeTasks.value.get(taskId)
    if (task) {
      task.status = 'cancelled'
    }
    log(`[Cancel] ${taskId} cancelled`)
  } catch (e) {
    log(`[Error] Cancel failed: ${e}`)
  }
}

async function shutdownWorker(workerId: string) {
  try {
    await swarm.shutdownWorkerById(workerId)
    log(`[Shutdown] ${workerId} stopped`)
  } catch (e) {
    log(`[Error] Shutdown failed: ${e}`)
  }
}

// Event listeners — only in Tauri mode
let unlistenFns: (() => void)[] = []

onMounted(async () => {
  // Initial load
  await swarm.loadWorkers()

  // Tauri-only event listeners (Web mode uses polling fallback)
  if (isTauriHost()) {
    try {
      const { listen } = await import('@tauri-apps/api/event')

      unlistenFns.push(
        await listen('swarm-task-completed', (event) => {
          const payload = event.payload as { taskId: string; workerId: string; output: string }
          log(`[Complete] ${payload.taskId} by ${payload.workerId}`)
          const task = activeTasks.value.get(payload.taskId)
          if (task) {
            task.status = 'completed'
            task.output = payload.output
          }
        })
      )

      unlistenFns.push(
        await listen('swarm-worker-status', (event) => {
          const status = event.payload as WorkerStatus
          const worker = swarm.workers.find(w => w.id === status.workerId)
          if (worker) {
            worker.status = status
          }
        })
      )
    } catch {
      // Event system not available — fall back to polling
    }
  }

  // Periodic status refresh (both modes)
  const intervalId = setInterval(refreshWorkerStatuses, 5000)
  unlistenFns.push(() => clearInterval(intervalId))
})

onUnmounted(() => {
  unlistenFns.forEach(fn => fn())
})

// Helper functions
function getHealthColor(health: string | undefined): string {
  const colors: Record<string, string> = {
    healthy: 'text-green-500',
    busy: 'text-blue-500',
    starting: 'text-yellow-500',
    stopping: 'text-orange-500',
    unhealthy: 'text-red-500',
    offline: 'text-gray-400',
  }
  return colors[health || 'offline'] || 'text-gray-400'
}

function formatCapabilities(caps: string[]): string {
  return caps.slice(0, 3).join(', ') + (caps.length > 3 ? '...' : '')
}

function formatBytes(bytes: number | null | undefined): string {
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
      'bg-green-100 border-green-300': swarmStatusLabel === 'ready',
      'bg-blue-100 border-blue-300': swarmStatusLabel === 'executing',
      'bg-yellow-100 border-yellow-300': swarmStatusLabel === 'no_queen' || swarmStatusLabel === 'no_workers',
    }">
      <div class="flex items-center gap-2">
        <span class="font-semibold">Swarm Status:</span>
        <span class="uppercase">{{ swarmStatusLabel }}</span>
        <span v-if="queenWorkerCaps" class="ml-4">
          Queen: <span class="font-mono">{{ queenWorkerCaps.workerId }}</span>
          ({{ queenWorkerCaps.workerType }})
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
          :disabled="isRegisteringWorker || swarm.loading"
          class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:opacity-50"
        >
          {{ isRegisteringWorker ? 'Registering...' : 'Register' }}
        </button>
      </div>
    </section>

    <!-- Worker Cards -->
    <section class="mb-6">
      <h2 class="text-lg font-semibold mb-3">Workers ({{ swarm.workers.length }})</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="worker in swarm.workers"
          :key="worker.id"
          class="bg-white rounded-lg shadow p-4 cursor-pointer hover:shadow-md"
          :class="{ 'ring-2 ring-blue-500': selectedWorkerId === worker.id }"
          @click="selectedWorkerId = worker.id"
        >
          <!-- Worker Header -->
          <div class="flex justify-between items-center mb-2">
            <span class="font-mono font-semibold">{{ worker.id }}</span>
            <span
              :class="getHealthColor(worker.status?.health)"
              class="font-medium"
            >
              {{ worker.status?.health || 'offline' }}
            </span>
          </div>

          <!-- Worker Type Badge -->
          <div class="mb-2">
            <span
              class="px-2 py-1 rounded text-xs font-medium"
              :class="{
                'bg-purple-100 text-purple-800': worker.capabilities.workerType === 'claude_code',
                'bg-green-100 text-green-800': worker.capabilities.workerType === 'codex',
              }"
            >
              {{ worker.capabilities.workerType }}
            </span>
            <span v-if="swarm.queenWorkerId === worker.id" class="ml-2 px-2 py-1 rounded text-xs bg-yellow-100 text-yellow-800">
              QUEEN
            </span>
          </div>

          <!-- Capabilities -->
          <div class="text-sm text-gray-600 mb-2">
            {{ formatCapabilities(worker.capabilities.capabilities) }}
          </div>

          <!-- Stats -->
          <div class="grid grid-cols-2 gap-2 text-sm">
            <div>
              <span class="text-gray-500">Tasks:</span>
              <span class="ml-1">{{ worker.status?.tasksCompleted || 0 }}/{{ worker.status?.tasksFailed || 0 }}</span>
            </div>
            <div>
              <span class="text-gray-500">PID:</span>
              <span class="ml-1">{{ worker.status?.pid || 'N/A' }}</span>
            </div>
            <div>
              <span class="text-gray-500">Memory:</span>
              <span class="ml-1">{{ formatBytes(worker.status?.memoryBytes) }}</span>
            </div>
            <div>
              <span class="text-gray-500">CPU:</span>
              <span class="ml-1">{{ worker.status?.cpuPercent?.toFixed(1) || 'N/A' }}%</span>
            </div>
          </div>

          <!-- Current Task -->
          <div v-if="worker.status?.currentTask" class="mt-2 p-2 bg-blue-50 rounded text-sm">
            <span class="text-gray-500">Current:</span>
            <span class="ml-1 font-mono">{{ worker.status.currentTask }}</span>
          </div>

          <!-- Actions -->
          <div class="mt-3 flex gap-2">
            <button
              @click.stop="healthCheck(worker.id)"
              class="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded"
            >
              Health
            </button>
            <button
              @click.stop="shutdownWorker(worker.id)"
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
        No healthy workers available. Register workers first.
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
                'bg-gray-100 text-gray-800': task.status === 'cancelled',
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
        <div v-for="(msg, i) in eventLog.slice(-50)" :key="i" class="mb-1">
          {{ msg }}
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
