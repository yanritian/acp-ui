import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useTeamRuntimeStore } from './team-runtime'
import { useConfigStore } from './config'
import type { TeamRoutingMode } from '../lib/team-service/types'

export interface WorkflowStep {
  id: string
  name: string
  prompt: string
  agentName?: string
  dependsOn: string[]
}

export interface WorkflowDefinition {
  id: string
  name: string
  description: string
  steps: WorkflowStep[]
  createdAt: number
  updatedAt: number
}

export interface WorkflowExecution {
  workflowId: string
  status: 'idle' | 'running' | 'completed' | 'failed' | 'cancelled'
  currentStepIndex: number
  stepResults: Map<string, { status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped'; output?: string; error?: string }>
  startedAt?: number
  completedAt?: number
  runningTaskIds: string[]
}

const WEB_WORKFLOWS_KEY = 'acp-ui:workflows'
const WEB_EXECUTIONS_KEY = 'acp-ui:executions'

function loadWebWorkflows(): WorkflowDefinition[] {
  if (typeof localStorage === 'undefined') return []
  const raw = localStorage.getItem(WEB_WORKFLOWS_KEY)
  if (!raw) return []
  try { return JSON.parse(raw) } catch { return [] }
}

function saveWebWorkflows(wf: WorkflowDefinition[]): void {
  if (typeof localStorage === 'undefined') return
  try { localStorage.setItem(WEB_WORKFLOWS_KEY, JSON.stringify(wf)) } catch {}
}

function loadWebExecutions(): Map<string, WorkflowExecution> {
  if (typeof localStorage === 'undefined') return new Map()
  const raw = localStorage.getItem(WEB_EXECUTIONS_KEY)
  if (!raw) return new Map()
  try {
    const parsed = JSON.parse(raw) as Array<{
      workflowId: string
      status: WorkflowExecution['status']
      currentStepIndex: number
      stepResults: [string, { status: string; output?: string; error?: string }][]
      startedAt?: number
      completedAt?: number
      runningTaskIds: string[]
    }>
    const map = new Map<string, WorkflowExecution>()
    for (const e of parsed) {
      map.set(e.workflowId, {
        workflowId: e.workflowId,
        status: e.status,
        currentStepIndex: e.currentStepIndex,
        stepResults: new Map<string, { status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped'; output?: string; error?: string }>(
          e.stepResults.map(([k, v]) => [k, {
            status: v.status as 'pending' | 'running' | 'completed' | 'failed' | 'skipped',
            output: v.output,
            error: v.error,
          }])
        ),
        startedAt: e.startedAt,
        completedAt: e.completedAt,
        runningTaskIds: e.runningTaskIds,
      })
    }
    return map
  } catch {
    return new Map()
  }
}

function saveWebExecutions(ex: WorkflowExecution[]): void {
  if (typeof localStorage === 'undefined') return
  try {
    const serialized = ex.map(e => ({
      ...e,
      stepResults: Array.from(e.stepResults.entries()),
    }))
    localStorage.setItem(WEB_EXECUTIONS_KEY, JSON.stringify(serialized))
  } catch {}
}

export const useWorkflowsStore = defineStore('workflows', () => {
  const workflows = ref<WorkflowDefinition[]>(loadWebWorkflows())
  const executions = ref<Map<string, WorkflowExecution>>(loadWebExecutions())
  const error = ref<string | null>(null)

  const teamRuntime = useTeamRuntimeStore()
  const configStore = useConfigStore()

  const hasAgents = () => Object.keys(configStore.config.agents).length > 0

  function saveToStorage() {
    saveWebWorkflows(workflows.value)
    saveWebExecutions(Array.from(executions.value.values()))
  }

  function createWorkflow(name: string, description: string, steps: WorkflowStep[]): WorkflowDefinition {
    const wf: WorkflowDefinition = {
      id: crypto.randomUUID(),
      name,
      description,
      steps,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }
    workflows.value.unshift(wf)
    saveToStorage()
    return wf
  }

  function updateWorkflow(id: string, updates: Partial<WorkflowDefinition>): WorkflowDefinition | null {
    const idx = workflows.value.findIndex(w => w.id === id)
    if (idx === -1) return null
    workflows.value[idx] = { ...workflows.value[idx], ...updates, updatedAt: Date.now() }
    saveToStorage()
    return workflows.value[idx]
  }

  function deleteWorkflow(id: string): void {
    workflows.value = workflows.value.filter(w => w.id !== id)
    executions.value.delete(id)
    saveToStorage()
  }

  async function runWorkflow(workflowId: string): Promise<void> {
    const wf = workflows.value.find(w => w.id === workflowId)
    if (!wf) return

    const exec: WorkflowExecution = {
      workflowId,
      status: 'running',
      currentStepIndex: 0,
      stepResults: new Map(wf.steps.map(s => [s.id, { status: 'pending' as const }])),
      startedAt: Date.now(),
      runningTaskIds: [],
    }
    executions.value.set(workflowId, exec)

    const agentNames = Object.keys(configStore.config.agents)
    const cwd = configStore.config.agents[agentNames[0]]?.env?.PWD ?? '.'

    // Execute steps sequentially (respecting dependsOn)
    for (let i = 0; i < wf.steps.length; i++) {
      const step = wf.steps[i]

      // Check dependencies
      const depsMet = step.dependsOn.every(depId => {
        const depResult = exec.stepResults.get(depId)
        return depResult?.status === 'completed'
      })

      if (!depsMet) {
        exec.stepResults.set(step.id, { status: 'skipped', error: 'Dependencies not met' })
        continue
      }

      // Update step to running
      exec.stepResults.set(step.id, { status: 'running' })
      exec.currentStepIndex = i
      saveToStorage()

      try {
        const targetAgent = step.agentName || agentNames[0]
        if (!targetAgent || !configStore.config.agents[targetAgent]) {
          throw new Error(`Agent '${targetAgent}' not configured`)
        }

        const result = await teamRuntime.runTeamTask({
          title: `${wf.name} - ${step.name}`,
          prompt: step.prompt,
          source: 'workflow',
          routing: 'single' as TeamRoutingMode,
          agents: [{ agentName: targetAgent, cwd }],
        })

        exec.runningTaskIds.push(result.task.id)

        if (result.status === 'failed') {
          exec.stepResults.set(step.id, { status: 'failed', error: result.error })
          exec.status = 'failed'
          break
        }

        const outputText = result.outputs.map(o => o.content).join('\n')
        exec.stepResults.set(step.id, { status: 'completed', output: outputText })
      } catch (e) {
        exec.stepResults.set(step.id, { status: 'failed', error: e instanceof Error ? e.message : String(e) })
        exec.status = 'failed'
        break
      }
    }

    if (exec.status !== 'failed') {
      exec.status = 'completed'
    }
    exec.completedAt = Date.now()
    saveToStorage()
  }

  async function cancelWorkflow(workflowId: string): Promise<void> {
    const exec = executions.value.get(workflowId)
    if (!exec) return
    exec.status = 'cancelled'
    exec.completedAt = Date.now()
    // Cancel all running tasks for this workflow
    for (const taskId of exec.runningTaskIds) {
      await teamRuntime.cancelTask(taskId).catch(() => {})
    }
    saveToStorage()
  }

  function clearError() {
    error.value = null
  }

  return {
    workflows,
    executions,
    error,
    hasAgents,
    createWorkflow,
    updateWorkflow,
    deleteWorkflow,
    runWorkflow,
    cancelWorkflow,
    clearError,
  }
})
