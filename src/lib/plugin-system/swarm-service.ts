import { invoke } from '@tauri-apps/api/core'

export type SwarmTopology = 'hierarchical' | 'mesh' | 'pipeline' | 'star' | 'adaptive'
export type ConsensusStrategy = 'first_wins' | 'majority' | 'best_score' | 'merge' | 'adversarial'
export type AgentRole = 'orchestrator' | 'executor' | 'specialist' | 'reviewer' | 'aggregator'
export type AgentSwarmStatus = 'idle' | 'assigned' | 'running' | 'waiting_sync' | 'completed' | 'failed' | 'evicted'
export type SwarmTaskStatus = 'pending' | 'in_progress' | 'waiting_consensus' | 'completed' | 'failed' | 'cancelled'

export interface SwarmAgent {
  id: string
  name: string
  role: AgentRole
  capabilities: string[]
  agent_config_name: string
  status: AgentSwarmStatus
  current_task?: string
  results: AgentResult[]
  token_budget?: number
  tokens_used: number
}

export interface AgentResult {
  task_id: string
  success: boolean
  output: Record<string, unknown>
  duration_ms: number
  tokens_used: number
  timestamp: string
}

export interface SwarmTask {
  id: string
  description: string
  assigned_agents: string[]
  topology: SwarmTopology
  consensus: ConsensusStrategy
  status: SwarmTaskStatus
  results: AgentResult[]
  created_at: string
  completed_at?: string
}

export interface SwarmHealth {
  total_agents: number
  active_agents: number
  idle_agents: number
  failed_agents: number
  pending_tasks: number
  running_tasks: number
  completed_tasks: number
  total_tokens_used: number
}

export class SwarmService {
  static async registerAgent(agent: SwarmAgent): Promise<void> {
    return invoke('swarm_register_agent', { agent })
  }

  static async listAgents(): Promise<SwarmAgent[]> {
    return invoke('swarm_list_agents')
  }

  static async createTask(description: string, topology?: SwarmTopology, consensus?: ConsensusStrategy): Promise<SwarmTask> {
    return invoke('swarm_create_task', { description, topology, consensus })
  }

  static async submitResult(taskId: string, agentId: string, result: AgentResult): Promise<string> {
    return invoke('swarm_submit_result', { taskId, agentId, result })
  }

  static async getTask(taskId: string): Promise<SwarmTask> {
    return invoke('swarm_get_task', { taskId })
  }

  static async getHealth(): Promise<SwarmHealth> {
    return invoke('swarm_get_health')
  }

  static async cancelTask(taskId: string): Promise<void> {
    return invoke('swarm_cancel_task', { taskId })
  }
}
