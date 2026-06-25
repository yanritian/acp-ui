// Agent Platform Types

export interface AgentSummary {
  id: string
  name: string
  adapter_type: string
  status: string
  health_score: number
  capabilities: string[]
}

export interface OneShotResponse {
  detected_role: string
  detected_scene: string
  selected_agent: string
  selection_reason: string
  result_preview: string
  estimated_cost: number
  transparency: {
    dag_visualization: string
    estimated_duration_ms: number
    privacy_level: string
  }
}

export interface CostSummary {
  daily_total: number
  monthly_total: number
  currency: string
  budget_remaining_percent: number
}

export interface ExecuteResponse {
  task_id: string
  status: string
  output_preview: string
  cost: number
  duration_ms: number
}

export interface UserRole {
  role: 'developer' | 'marketer' | 'designer' | 'finance' | 'gamer' | 'writer' | 'analyst' | 'general'
  confidence: number
}

export interface Scene {
  type: string
  keywords: string[]
}

export interface AgentEvent {
  event_type: string
  data: any
  timestamp: string
}