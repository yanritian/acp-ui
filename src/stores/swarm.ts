import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { SwarmService, type SwarmAgent, type SwarmTask, type SwarmHealth, type SwarmTopology, type ConsensusStrategy } from '@/lib/plugin-system/swarm-service'

export const useSwarmStore = defineStore('swarm', () => {
  const agents = ref<SwarmAgent[]>([])
  const tasks = ref<SwarmTask[]>([])
  const health = ref<SwarmHealth | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const activeAgents = computed(() => agents.value.filter(a => a.status === 'running' || a.status === 'assigned'))
  const idleAgents = computed(() => agents.value.filter(a => a.status === 'idle'))
  const runningTasks = computed(() => tasks.value.filter(t => t.status === 'in_progress' || t.status === 'waiting_consensus'))

  async function loadAgents() {
    loading.value = true
    try {
      agents.value = await SwarmService.listAgents()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function registerAgent(agent: SwarmAgent) {
    try {
      await SwarmService.registerAgent(agent)
      await loadAgents()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function createTask(description: string, topology?: SwarmTopology, consensus?: ConsensusStrategy) {
    try {
      const task = await SwarmService.createTask(description, topology, consensus)
      tasks.value.push(task)
      return task
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function loadHealth() {
    try {
      health.value = await SwarmService.getHealth()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function cancelTask(taskId: string) {
    try {
      await SwarmService.cancelTask(taskId)
      await loadAgents()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  return {
    agents, tasks, health, loading, error,
    activeAgents, idleAgents, runningTasks,
    loadAgents, registerAgent, createTask, loadHealth, cancelTask,
  }
})
