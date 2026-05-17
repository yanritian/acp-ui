/**
 * WebSocket同步服务
 * 三端实时数据同步（Web + Tauri桌面端 + Flutter移动端）
 */

import { ref, computed } from 'vue'
import type {
  SyncMessage,
  SyncMessageType,
  SyncSource,
  SyncTarget,
  ConnectionStatus,
  WebSocketConfig,
  SyncState,
  ClientIdentity,
  StatusUpdatePayload,
  EventPayload,
  CommandPayload,
  InteractionPayload
} from './sync-message-types'
import { DEFAULT_WS_CONFIG } from './sync-message-types'
import { useAgentRealtimeStore } from '@/stores/agent-realtime'
import { useAgentPetStore } from '@/stores/agent-pet'

export class WebSocketSyncService {
  private ws: WebSocket | null = null
  private config: WebSocketConfig
  private reconnectAttempts = 0
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null
  private messageQueue: SyncMessage[] = []
  private sequenceId = 0
  private clientId: string
  private clientType: SyncSource

  // Vue reactive state
  public state = ref<SyncState>({
    isConnected: false,
    connectionStatus: 'disconnected',
    lastSyncTime: 0,
    pendingMessages: [],
    sequenceId: 0,
    clientIdentity: null
  })

  // Stores reference
  private realtimeStore: ReturnType<typeof useAgentRealtimeStore> | null = null
  private petStore: ReturnType<typeof useAgentPetStore> | null = null

  constructor(
    clientType: SyncSource,
    config: Partial<WebSocketConfig> = {}
  ) {
    this.config = { ...DEFAULT_WS_CONFIG, ...config }
    this.clientType = clientType
    this.clientId = `${clientType}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  }

  // ===== 初始化 =====
  initialize() {
    this.realtimeStore = useAgentRealtimeStore()
    this.petStore = useAgentPetStore()
  }

  // ===== 连接 =====
  connect(): Promise<boolean> {
    return new Promise((resolve, reject) => {
      if (this.state.value.isConnected) {
        resolve(true)
        return
      }

      this.state.value.connectionStatus = 'connecting'

      try {
        this.ws = new WebSocket(this.config.url)

        this.ws.onopen = () => {
          this.state.value.isConnected = true
          this.state.value.connectionStatus = 'connected'
          this.reconnectAttempts = 0

          // 发送客户端标识
          this.sendClientIdentity()

          // 启动心跳
          this.startHeartbeat()

          // 发送队列中的消息
          this.flushMessageQueue()

          // 请求全量同步
          this.requestFullSync()

          resolve(true)
        }

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data)
        }

        this.ws.onerror = (error) => {
          this.state.value.connectionStatus = 'error'
          reject(error)
        }

        this.ws.onclose = () => {
          this.state.value.isConnected = false
          this.state.value.connectionStatus = 'disconnected'
          this.stopHeartbeat()
          this.handleReconnect()
        }
      } catch (error) {
        this.state.value.connectionStatus = 'error'
        reject(error)
      }
    })
  }

  // ===== 断开连接 =====
  disconnect() {
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
    this.stopHeartbeat()
    this.state.value.isConnected = false
    this.state.value.connectionStatus = 'disconnected'
  }

  // ===== 重连 =====
  private handleReconnect() {
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      this.state.value.connectionStatus = 'error'
      return
    }

    this.state.value.connectionStatus = 'reconnecting'
    this.reconnectAttempts++

    setTimeout(() => {
      this.connect().catch(() => {
        // 重连失败，继续尝试
      })
    }, this.config.reconnectInterval)
  }

  // ===== 心跳 =====
  private startHeartbeat() {
    this.heartbeatTimer = setInterval(() => {
      if (this.state.value.isConnected) {
        this.send({
          type: 'sync_request',
          source: this.clientType,
          target: 'server',
          payload: { requestType: 'heartbeat' },
          timestamp: Date.now(),
          sequenceId: this.nextSequenceId()
        })
      }
    }, this.config.heartbeatInterval)
  }

  private stopHeartbeat() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  // ===== 发送消息 =====
  private nextSequenceId(): number {
    this.sequenceId++
    this.state.value.sequenceId = this.sequenceId
    return this.sequenceId
  }

  send(message: SyncMessage) {
    if (!this.state.value.isConnected) {
      // 加入队列等待连接
      this.messageQueue.push(message)
      if (this.messageQueue.length > this.config.messageQueueSize) {
        this.messageQueue.shift()
      }
      this.state.value.pendingMessages = [...this.messageQueue]
      return
    }

    try {
      this.ws?.send(JSON.stringify(message))
    } catch (error) {
      console.error('Failed to send message:', error)
      this.messageQueue.push(message)
    }
  }

  private flushMessageQueue() {
    while (this.messageQueue.length > 0 && this.state.value.isConnected) {
      const message = this.messageQueue.shift()
      if (message) {
        this.send(message)
      }
    }
    this.state.value.pendingMessages = []
  }

  // ===== 发送客户端标识 =====
  private sendClientIdentity() {
    const identity: ClientIdentity = {
      id: this.clientId,
      type: this.clientType,
      version: '1.0.0',
      platform: this.clientType === 'desktop' ? 'tauri' : this.clientType,
      connectedAt: Date.now(),
      lastActiveAt: Date.now()
    }

    this.state.value.clientIdentity = identity

    this.send({
      type: 'sync_request',
      source: this.clientType,
      target: 'server',
      payload: { requestType: 'client_identity', identity },
      timestamp: Date.now(),
      sequenceId: this.nextSequenceId()
    })
  }

  // ===== 请求全量同步 =====
  private requestFullSync() {
    this.send({
      type: 'sync_request',
      source: this.clientType,
      target: 'server',
      payload: { requestType: 'full_sync' },
      timestamp: Date.now(),
      sequenceId: this.nextSequenceId()
    })
  }

  // ===== 处理接收消息 =====
  private handleMessage(data: string) {
    try {
      const message: SyncMessage = JSON.parse(data)
      this.state.value.lastSyncTime = Date.now()

      switch (message.type) {
        case 'status_update':
          this.handleStatusUpdate(message.payload as StatusUpdatePayload)
          break

        case 'event':
          this.handleEvent(message.payload as EventPayload)
          break

        case 'command':
          this.handleCommand(message.payload as CommandPayload)
          break

        case 'sync_response':
          this.handleSyncResponse(message.payload)
          break

        case 'permission_request':
          this.handlePermissionRequest(message.payload)
          break

        case 'interaction':
          this.handleInteraction(message.payload as InteractionPayload)
          break

        case 'level_up':
          this.handleLevelUp(message.payload)
          break

        case 'achievement':
          this.handleAchievement(message.payload)
          break
      }
    } catch (error) {
      console.error('Failed to handle message:', error)
    }
  }

  // ===== 处理状态更新 =====
  private handleStatusUpdate(payload: StatusUpdatePayload) {
    if (this.realtimeStore) {
      this.realtimeStore.updateAgentStatus(payload.agentId, payload.updates)
    }
  }

  // ===== 处理事件 =====
  private handleEvent(payload: EventPayload) {
    if (this.realtimeStore && payload.propagate) {
      this.realtimeStore.handleRealtimeEvent(payload.event)
    }
  }

  // ===== 处理命令 =====
  private handleCommand(payload: CommandPayload) {
    // 根据命令类型执行相应操作
    // 例如：start_task, pause_task, permission_grant 等
    console.log('Received command:', payload)
  }

  // ===== 处理同步响应 =====
  private handleSyncResponse(payload: any) {
    if (payload.agents && this.realtimeStore) {
      // 同步Agent列表
      payload.agents.forEach(agent => {
        this.realtimeStore!.registerAgent(agent.agentId, agent.agentName)
        this.realtimeStore!.updateAgentStatus(agent.agentId, agent)
      })
    }

    if (payload.pets && this.petStore) {
      // 同步宠物列表
      payload.pets.forEach(pet => {
        this.petStore!.createPet(pet.agentId, pet.name, pet.appearance?.type)
      })
    }

    if (payload.events && this.realtimeStore) {
      // 同步事件历史
      payload.events.forEach(event => {
        this.realtimeStore!.handleRealtimeEvent(event)
      })
    }
  }

  // ===== 处理权限请求 =====
  private handlePermissionRequest(payload: any) {
    if (this.realtimeStore) {
      this.realtimeStore.handleRealtimeEvent({
        id: `perm-${Date.now()}`,
        type: 'permission_request',
        agentId: payload.agentId,
        timestamp: Date.now(),
        data: { permission: payload.permission },
        severity: 'info'
      })
    }
  }

  // ===== 处理互动 =====
  private handleInteraction(payload: InteractionPayload) {
    if (this.petStore) {
      this.petStore.handleInteraction(payload.agentId, payload.interactionType)
    }
  }

  // ===== 处理升级 =====
  private handleLevelUp(payload: any) {
    if (this.petStore) {
      this.petStore.addExperience(payload.agentId, payload.experience)
    }
  }

  // ===== 处理成就 =====
  private handleAchievement(payload: any) {
    if (this.petStore) {
      this.petStore.unlockAchievement(payload.agentId, payload.achievementId)
    }
  }

  // ===== 公共API =====

  // 发送状态更新
  broadcastStatusUpdate(agentId: string, updates: Record<string, any>) {
    this.send({
      type: 'status_update',
      source: this.clientType,
      target: 'all',
      payload: { agentId, status: updates, updates },
      timestamp: Date.now(),
      sequenceId: this.nextSequenceId()
    })
  }

  // 发送事件
  broadcastEvent(event: any) {
    this.send({
      type: 'event',
      source: this.clientType,
      target: 'all',
      payload: { event, propagate: true },
      timestamp: Date.now(),
      sequenceId: this.nextSequenceId()
    })
  }

  // 发送互动
  sendInteraction(agentId: string, interactionType: 'pet' | 'poke' | 'feed' | 'play') {
    this.send({
      type: 'interaction',
      source: this.clientType,
      target: 'all',
      payload: { agentId, interactionType, source: this.clientType },
      timestamp: Date.now(),
      sequenceId: this.nextSequenceId()
    })
  }

  // 发送权限响应
  sendPermissionResponse(agentId: string, optionId: string) {
    this.send({
      type: 'permission_response',
      source: this.clientType,
      target: 'server',
      payload: { agentId, optionId, source: this.clientType },
      timestamp: Date.now(),
      sequenceId: this.nextSequenceId()
    })
  }

  // Computed
  get isConnected() {
    return computed(() => this.state.value.isConnected)
  }

  get connectionStatus() {
    return computed(() => this.state.value.connectionStatus)
  }
}

// ===== 创建单例 =====
let syncServiceInstance: WebSocketSyncService | null = null

export function createSyncService(clientType: SyncSource, config?: Partial<WebSocketConfig>) {
  if (!syncServiceInstance) {
    syncServiceInstance = new WebSocketSyncService(clientType, config)
  }
  return syncServiceInstance
}

export function getSyncService() {
  return syncServiceInstance
}