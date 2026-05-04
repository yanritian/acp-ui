import type { RuntimeOutput, RuntimeTask, RuntimeTaskStatus } from '../agent-runtime/types'

export type TeamRoutingMode = 'single' | 'broadcast' | 'round-robin' | 'load-balanced'

export interface TeamAgentSelection {
  agentName: string
  cwd: string
}

export interface TeamRunRequest {
  title: string
  prompt: string
  source: RuntimeTask['source']
  routing: TeamRoutingMode
  agents: TeamAgentSelection[]
  memoryScope?: 'none' | 'global' | 'agent' | 'session'
}

export interface TeamRunResult {
  task: RuntimeTask
  outputs: RuntimeOutput[]
  status: RuntimeTaskStatus
  error?: string
}
