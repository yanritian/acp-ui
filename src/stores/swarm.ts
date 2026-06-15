import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import {
  swarmRegisterWorker,
  swarmListWorkers,
  swarmGetWorkerStatus,
  swarmHealthCheck,
  swarmSendTask,
  swarmCancelTask,
  swarmShutdownWorker,
  type WorkerCapabilities,
  type WorkerStatus,
  type TaskHandle,
} from '@/lib/swarm-api'

// SwarmWorker type for real worker management (Day 1-4)
export interface SwarmWorker {
  id: string
  capabilities: WorkerCapabilities
  status: WorkerStatus
}

export const useSwarmStore = defineStore('swarm', () => {
  // State for real workers (Day 1-4)
  const workers = ref<SwarmWorker[]>([])
  const queenWorkerId = ref<string | null>(null)
  const currentTaskHandle = ref<TaskHandle | null>(null)
  const taskHandles = ref<TaskHandle[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Computed for workers
  const idleWorkers = computed(() => workers.value.filter(w => w.status?.health === 'healthy'))
  const busyWorkers = computed(() => workers.value.filter(w => w.status?.health === 'busy'))
  const offlineWorkers = computed(() => workers.value.filter(w => w.status?.health === 'offline'))
  const queenWorker = computed(() => workers.value.find(w => w.id === queenWorkerId.value))
  const healthyWorkerCount = computed(() => idleWorkers.value.length + busyWorkers.value.length)
  const swarmStatus = computed(() => {
    if (healthyWorkerCount.value === 0) return 'no_workers'
    if (!queenWorkerId.value) return 'no_queen'
    if (busyWorkers.value.length > 0) return 'executing'
    return 'ready'
  })

  // Actions for real workers (Day 1-4)
  async function loadWorkers() {
    loading.value = true
    try {
      const caps = await swarmListWorkers()
      const workerList: SwarmWorker[] = []

      for (const cap of caps) {
        try {
          const status = await swarmGetWorkerStatus(cap.workerId)
          workerList.push({ id: cap.workerId, capabilities: cap, status })
        } catch {
          workerList.push({
            id: cap.workerId,
            capabilities: cap,
            status: {
              workerId: cap.workerId,
              workerType: cap.workerType,
              health: 'offline',
              pid: null,
              memoryBytes: null,
              cpuPercent: null,
              tasksCompleted: 0,
              tasksFailed: 0,
              currentTask: null,
              startedAt: null,
              lastHeartbeat: null,
            },
          })
        }
      }

      workers.value = workerList
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function registerWorker(workerType: 'codex' | 'claude_code', workerId: string) {
    loading.value = true
    error.value = null

    try {
      const capabilities = await swarmRegisterWorker(workerType, workerId)
      const status = await swarmGetWorkerStatus(capabilities.workerId)

      workers.value.push({
        id: capabilities.workerId,
        capabilities,
        status,
      })

      // Auto-set queen for first claude_code worker
      if (workerType === 'claude_code' && !queenWorkerId.value) {
        queenWorkerId.value = capabilities.workerId
      }

      return capabilities
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function sendTaskToWorker(workerId: string, prompt: string) {
    const taskId = `task-${Date.now()}`
    try {
      const handle = await swarmSendTask(workerId, taskId, prompt)
      taskHandles.value.push(handle)
      currentTaskHandle.value = handle
      return handle
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function checkWorkerHealth(workerId: string) {
    try {
      return await swarmHealthCheck(workerId)
    } catch {
      return false
    }
  }

  async function cancelWorkerTask(workerId: string, taskId: string) {
    try {
      await swarmCancelTask(workerId, taskId)
      taskHandles.value = taskHandles.value.filter(h => h.taskId !== taskId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function shutdownWorkerById(workerId: string) {
    try {
      await swarmShutdownWorker(workerId)
      workers.value = workers.value.filter(w => w.id !== workerId)

      if (queenWorkerId.value === workerId) {
        queenWorkerId.value = workers.value.find(w => w.capabilities.workerType === 'claude_code')?.id || null
      }
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  function setQueen(workerId: string) {
    queenWorkerId.value = workerId
  }

  return {
    // State for workers (Day 1-4)
    workers, queenWorkerId, currentTaskHandle, taskHandles, loading, error,
    // Computed
    idleWorkers, busyWorkers, offlineWorkers, queenWorker, healthyWorkerCount, swarmStatus,
    // Actions
    loadWorkers, registerWorker, sendTaskToWorker, checkWorkerHealth, cancelWorkerTask, shutdownWorkerById, setQueen,
  }
})
