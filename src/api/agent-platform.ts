// Agent Platform API Service
// Connects to Tauri HTTP Server for Web/Mobile access

import type { AgentSummary, OneShotResponse, CostSummary, ExecuteResponse } from './types'

const API_BASE = 'http://localhost:3000/api'

// Agent API
export const agentApi = {
  async list(): Promise<AgentSummary[]> {
    const response = await fetch(`${API_BASE}/agents`)
    if (!response.ok) throw new Error('Failed to fetch agents')
    return response.json()
  },

  async get(id: string): Promise<AgentSummary> {
    const response = await fetch(`${API_BASE}/agents/${id}`)
    if (!response.ok) throw new Error(`Agent '${id}' not found`)
    return response.json()
  },

  async getHealth(id: string): Promise<{ health_score: number; success_rate: number }> {
    const response = await fetch(`${API_BASE}/agents/${id}/health`)
    if (!response.ok) throw new Error('Failed to fetch health')
    return response.json()
  }
}

// One-Shot API
export const oneShotApi = {
  async execute(request: {
    input: string
    context_hint?: string
    preferred_agent?: string
    max_cost?: number
    timeout_ms?: number
  }): Promise<OneShotResponse> {
    const response = await fetch(`${API_BASE}/oneshot`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request)
    })
    if (!response.ok) throw new Error('One-shot execution failed')
    return response.json()
  }
}

// Cost API
export const costApi = {
  async getSummary(): Promise<CostSummary> {
    const response = await fetch(`${API_BASE}/costs/summary`)
    if (!response.ok) throw new Error('Failed to fetch cost summary')
    return response.json()
  },

  async setBudget(budget: { daily_limit: number; monthly_limit: number }): Promise<string> {
    const response = await fetch(`${API_BASE}/costs/budget`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(budget)
    })
    if (!response.ok) throw new Error('Failed to set budget')
    return response.json()
  }
}

// Execute API
export const executeApi = {
  async execute(request: {
    agent_id: string
    description: string
    input: string
    input_type?: 'text' | 'file' | 'url' | 'data'
    timeout_ms?: number
  }): Promise<ExecuteResponse> {
    const response = await fetch(`${API_BASE}/execute`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        ...request,
        input_type: request.input_type || 'text'
      })
    })
    if (!response.ok) throw new Error('Execution failed')
    return response.json()
  }
}

// Health Check
export const healthApi = {
  async check(): Promise<{ status: string; version: string; uptime_seconds: number }> {
    const response = await fetch(`${API_BASE}/health`)
    if (!response.ok) throw new Error('Health check failed')
    return response.json()
  }
}

// WebSocket for real-time events
export class WebSocketService {
  private ws: WebSocket | null = null
  private listeners: Map<string, (data: any) => void> = new Map()

  connect(url: string = 'ws://localhost:3000/ws/events') {
    this.ws = new WebSocket(url)

    this.ws.onopen = () => {
      console.log('WebSocket connected')
    }

    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data)
      const handler = this.listeners.get(data.event_type)
      if (handler) handler(data.data)
    }

    this.ws.onclose = () => {
      console.log('WebSocket disconnected')
      // Auto reconnect after 5 seconds
      setTimeout(() => this.connect(url), 5000)
    }
  }

  on(eventType: string, handler: (data: any) => void) {
    this.listeners.set(eventType, handler)
  }

  send(type: string, data?: any) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type, data }))
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }
}