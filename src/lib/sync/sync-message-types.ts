/**
 * WebSocket同步消息类型定义
 * 三端同步系统（Web + Tauri桌面端 + Flutter移动端）
 */

// ===== 同步源类型 =====
export type SyncSource = 'web' | 'desktop' | 'mobile' | 'server'

// ===== 同步目标类型 =====
export type SyncTarget = 'web' | 'desktop' | 'mobile' | 'server' | 'all'

// ===== 同步消息类型 =====
export type SyncMessageType =
  | 'status_update'      // Agent状态更新
  | 'event'              // 实时事件
  | 'command'            // 操作命令
  | 'sync_request'       // 同步请求
  | 'sync_response'      // 同步响应
  | 'permission_request' // 权限请求
  | 'permission_response'// 权限响应
  | 'interaction'        // 互动事件
  | 'level_up'           // 升级事件
  | 'achievement'        // 成就事件

// ===== 同步消息 =====
export interface SyncMessage {
  type: SyncMessageType
  source: SyncSource
  target: SyncTarget
  payload: any
  timestamp: number
  sequenceId: number
}

// ===== 状态更新消息 =====
export interface StatusUpdatePayload {
  agentId: string
  status: any
  updates: Record<string, any>
}

// ===== 事件消息 =====
export interface EventPayload {
  event: any
  propagate: boolean
}

// ===== 命令消息 =====
export interface CommandPayload {
  commandType: 'start_task' | 'pause_task' | 'resume_task' | 'cancel_task' | 'permission_grant'
  agentId?: string
  taskId?: string
  params?: Record<string, any>
}

// ===== 同步请求消息 =====
export interface SyncRequestPayload {
  requestType: 'full_sync' | 'partial_sync' | 'agent_list' | 'event_history'
  agentIds?: string[]
  since?: number // 时间戳
}

// ===== 同步响应消息 =====
export interface SyncResponsePayload {
  responseType: 'full_sync' | 'partial_sync' | 'agent_list' | 'event_history'
  agents?: any[]
  events?: any[]
  pets?: any[]
  timestamp: number
}

// ===== 权限请求消息 =====
export interface PermissionRequestPayload {
  agentId: string
  permission: any
  timeout?: number
}

// ===== 权限响应消息 =====
export interface PermissionResponsePayload {
  agentId: string
  optionId: string
  source: SyncSource
}

// ===== 互动消息 =====
export interface InteractionPayload {
  agentId: string
  interactionType: 'pet' | 'poke' | 'feed' | 'play'
  source: SyncSource
}

// ===== 升级消息 =====
export interface LevelUpPayload {
  agentId: string
  newLevel: number
  experience: number
}

// ===== 成就消息 =====
export interface AchievementPayload {
  agentId: string
  achievementId: string
  achievement: any
}

// ===== 连接状态 =====
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'error'

// ===== WebSocket配置 =====
export interface WebSocketConfig {
  url: string
  reconnectInterval: number // 毫秒
  maxReconnectAttempts: number
  heartbeatInterval: number // 毫秒
  messageQueueSize: number
}

// ===== 默认配置 =====
export const DEFAULT_WS_CONFIG: WebSocketConfig = {
  url: 'ws://localhost:8765/sync',
  reconnectInterval: 3000,
  maxReconnectAttempts: 10,
  heartbeatInterval: 30000,
  messageQueueSize: 100
}

// ===== 客户端标识 =====
export interface ClientIdentity {
  id: string
  type: SyncSource
  version: string
  platform: string
  connectedAt: number
  lastActiveAt: number
}

// ===== 同步状态 =====
export interface SyncState {
  isConnected: boolean
  connectionStatus: ConnectionStatus
  lastSyncTime: number
  pendingMessages: SyncMessage[]
  sequenceId: number
  clientIdentity: ClientIdentity | null
}