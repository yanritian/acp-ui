// Agent Platform Store - Pinia state management

import { defineStore } from 'pinia'
import { agentApi, oneShotApi, costApi, executeApi } from '@/api/agent-platform'
import type { AgentSummary, OneShotResponse, CostSummary, ExecuteResponse } from '@/api/types'

interface AgentPlatformState {
  agents: AgentSummary[]
  selectedAgent: AgentSummary | null
  oneShotResult: OneShotResponse | null
  costSummary: CostSummary | null
  isExecuting: boolean
  error: string | null
}

export const useAgentPlatformStore = defineStore('agentPlatform', {
  state: (): AgentPlatformState => ({
    agents: [],
    selectedAgent: null,
    oneShotResult: null,
    costSummary: null,
    isExecuting: false,
    error: null
  }),

  getters: {
    availableAgents: (state) => state.agents.filter(a => a.status === 'available'),
    agentById: (state) => (id: string) => state.agents.find(a => a.id === id),
    topAgents: (state) => state.agents.sort((a, b) => b.health_score - a.health_score).slice(0, 3)
  },

  actions: {
    async fetchAgents() {
      try {
        this.agents = await agentApi.list()
        this.error = null
      } catch (e) {
        this.error = (e as Error).message
      }
    },

    selectAgent(agent: AgentSummary) {
      this.selectedAgent = agent
    },

    async executeOneShot(input: string, options?: {
      preferred_agent?: string
      max_cost?: number
    }) {
      this.isExecuting = true
      this.error = null

      try {
        this.oneShotResult = await oneShotApi.execute({
          input,
          ...options
        })
      } catch (e) {
        this.error = (e as Error).message
      } finally {
        this.isExecuting = false
      }
    },

    async fetchCostSummary() {
      try {
        this.costSummary = await costApi.getSummary()
      } catch (e) {
        this.error = (e as Error).message
      }
    },

    async executeWithAgent(agentId: string, description: string, input: string) {
      this.isExecuting = true
      this.error = null

      try {
        const result = await executeApi.execute({
          agent_id: agentId,
          description,
          input
        })
        return result
      } catch (e) {
        this.error = (e as Error).message
        return null
      } finally {
        this.isExecuting = false
      }
    },

    clearError() {
      this.error = null
    }
  }
})