import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { AgentTeamsService } from '../lib/team-service/agent-teams-service'
import type { TeamRunRequest, TeamRunResult } from '../lib/team-service/types'
import type { RuntimeOutput, RuntimeTask, RuntimeEvent, RuntimeSession } from '../lib/agent-runtime/types'

export const useTeamRuntimeStore = defineStore('teamRuntime', () => {
  const agents = ref<Map<string, RuntimeSession>>(new Map())
  const tasks = ref<Map<string, RuntimeTask>>(new Map())
  const outputs = ref<Map<string, RuntimeOutput[]>>(new Map())
  const events = ref<RuntimeEvent[]>([])
  const error = ref<string | null>(null)
  const isRunning = ref(false)

  const service = new AgentTeamsService()

  const taskList = computed(() => Array.from(tasks.value.values()).sort((a, b) => b.createdAt - a.createdAt))

  const activeTaskCount = computed(() =>
    Array.from(tasks.value.values()).filter(t => t.status === 'running').length
  )

  const completedTaskCount = computed(() =>
    Array.from(tasks.value.values()).filter(t => t.status === 'completed' || t.status === 'failed').length
  )

  function getTaskOutputs(taskId: string): RuntimeOutput[] {
    return outputs.value.get(taskId) ?? []
  }

  function addEvent(event: Omit<RuntimeEvent, 'id' | 'timestamp'>) {
    events.value.unshift({
      ...event,
      id: crypto.randomUUID(),
      timestamp: Date.now(),
    })
  }

  async function runTeamTask(request: TeamRunRequest): Promise<TeamRunResult> {
    isRunning.value = true
    error.value = null

    try {
      const result = await service.runTeamTask(request)

      // Update stores
      tasks.value.set(result.task.id, result.task)
      outputs.value.set(result.task.id, result.outputs)

      addEvent({
        type: result.status === 'failed' ? 'task-failed' : 'task-completed',
        taskId: result.task.id,
        message: `${request.title}: ${result.status}`,
        payload: { error: result.error },
      })

      if (result.status === 'failed') {
        error.value = result.error ?? 'Task failed'
      }

      return result
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = msg
      addEvent({ type: 'task-failed', taskId: 'unknown', message: msg })
      throw e
    } finally {
      isRunning.value = false
    }
  }

  async function cancelTask(taskId: string): Promise<void> {
    const task = tasks.value.get(taskId)
    if (!task) return

    tasks.value.set(taskId, { ...task, status: 'cancelled', completedAt: Date.now() })

    await service.cancelTask(taskId).catch(() => {})

    addEvent({
      type: 'task-failed',
      taskId,
      message: `Task cancelled: ${task.title}`,
    })
  }

  function clearError() {
    error.value = null
  }

  function clearCompleted(): void {
    for (const [id, task] of tasks.value) {
      if (task.status === 'completed' || task.status === 'failed' || task.status === 'cancelled') {
        tasks.value.delete(id)
        outputs.value.delete(id)
      }
    }
  }

  return {
    agents,
    tasks,
    outputs,
    events,
    error,
    isRunning,
    taskList,
    activeTaskCount,
    completedTaskCount,
    getTaskOutputs,
    runTeamTask,
    cancelTask,
    clearError,
    clearCompleted,
  }
})
