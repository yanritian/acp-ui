import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  AgentRealtimeStatus,
  AgentActivityType,
  RealtimeEvent,
  ThinkingChunk,
  ToolExecution,
  OutputChunk,
  PermissionWaiting
} from '@/lib/agent-runtime/realtime-progress-types'

export const useAgentRealtimeStore = defineStore('agent-realtime', () => {
  // ===== State =====
  const agents = ref<Map<string, AgentRealtimeStatus>>(new Map())
  const activeAgentId = ref<string | null>(null)
  const globalEvents = ref<RealtimeEvent[]>([])
  const isConnected = ref(false)
  const connectionStatus = ref<'idle' | 'connecting' | 'connected' | 'error'>('idle')

  // ===== Computed =====
  const activeAgent = computed(() => {
    if (!activeAgentId.value) return null
    return agents.value.get(activeAgentId.value)
  })

  const allAgents = computed(() => {
    return Array.from(agents.value.values())
  })

  const thinkingAgents = computed(() => {
    return allAgents.value.filter(a => a.currentActivity.type === 'thinking')
  })

  const executingAgents = computed(() => {
    return allAgents.value.filter(a => a.currentActivity.type === 'executing')
  })

  const outputtingAgents = computed(() => {
    return allAgents.value.filter(a => a.currentActivity.type === 'outputting')
  })

  const waitingAgents = computed(() => {
    return allAgents.value.filter(a => a.permissionWaiting !== null)
  })

  const idleAgents = computed(() => {
    return allAgents.value.filter(a => a.currentActivity.type === 'idle')
  })

  const errorAgents = computed(() => {
    return allAgents.value.filter(a => a.currentActivity.type === 'error')
  })

  const recentEvents = computed(() => {
    return globalEvents.value.slice(-50).sort((a, b) => b.timestamp - a.timestamp)
  })

  const globalStats = computed(() => {
    const all = allAgents.value
    return {
      totalAgents: all.length,
      thinking: thinkingAgents.value.length,
      executing: executingAgents.value.length,
      outputting: outputtingAgents.value.length,
      waiting: waitingAgents.value.length,
      idle: idleAgents.value.length,
      error: errorAgents.value.length,
      totalCompleted: all.reduce((sum, a) => sum + a.stats.tasksCompleted, 0),
      totalFailed: all.reduce((sum, a) => sum + a.stats.tasksFailed, 0),
      averageSuccessRate: all.length > 0
        ? all.reduce((sum, a) => sum + a.stats.successRate, 0) / all.length
        : 100
    }
  })

  // ===== Methods =====

  // 注册新Agent
  function registerAgent(agentId: string, agentName: string) {
    const status: AgentRealtimeStatus = {
      agentId,
      agentName,
      currentActivity: {
        type: 'idle',
        startTime: Date.now(),
        duration: 0,
        progress: 0
      },
      thinking: {
        content: '',
        startTime: 0,
        depth: 0,
        chunks: [],
        isStreaming: false
      },
      toolExecution: null,
      output: {
        content: '',
        chunks: [],
        totalLength: 0,
        currentPosition: 0,
        isStreaming: false
      },
      permissionWaiting: null,
      stats: {
        tasksCompleted: 0,
        tasksFailed: 0,
        averageDuration: 0,
        successRate: 100,
        totalThinkingTime: 0,
        totalToolCalls: 0
      },
      lastUpdateTime: Date.now()
    }
    agents.value.set(agentId, status)
    addGlobalEvent('status_change', agentId, { newStatus: 'idle' }, 'info')
  }

  // 移除Agent
  function removeAgent(agentId: string) {
    agents.value.delete(agentId)
    if (activeAgentId.value === agentId) {
      activeAgentId.value = null
    }
  }

  // 设置活跃Agent
  function setActiveAgent(agentId: string) {
    if (agents.value.has(agentId)) {
      activeAgentId.value = agentId
    }
  }

  // 更新Agent状态
  function updateAgentStatus(agentId: string, updates: Partial<AgentRealtimeStatus>) {
    const agent = agents.value.get(agentId)
    if (!agent) return

    const updatedAgent = {
      ...agent,
      ...updates,
      lastUpdateTime: Date.now()
    }
    agents.value.set(agentId, updatedAgent)
  }

  // 处理实时事件
  function handleRealtimeEvent(event: RealtimeEvent) {
    const agent = agents.value.get(event.agentId)
    if (!agent) return

    // 添加到全局事件列表
    globalEvents.value.push(event)
    if (globalEvents.value.length > 100) {
      globalEvents.value.shift()
    }

    // 根据事件类型更新Agent状态
    switch (event.type) {
      case 'thinking_start':
        agent.currentActivity = {
          type: 'thinking',
          startTime: event.timestamp,
          duration: 0,
          progress: 0,
          description: event.data.description
        }
        agent.thinking.startTime = event.timestamp
        agent.thinking.isStreaming = true
        agent.thinking.content = ''
        agent.thinking.chunks = []
        break

      case 'thinking_chunk':
        if (event.data.chunk) {
          const chunk = event.data.chunk as ThinkingChunk
          agent.thinking.chunks.push(chunk)
          agent.thinking.content += chunk.content
          agent.thinking.depth = Math.max(agent.thinking.depth, chunk.depth)
        }
        break

      case 'thinking_end':
        agent.thinking.isStreaming = false
        const thinkingDuration = Date.now() - agent.thinking.startTime
        agent.stats.totalThinkingTime += thinkingDuration
        break

      case 'tool_call_start':
        agent.currentActivity = {
          type: 'executing',
          startTime: event.timestamp,
          duration: 0,
          progress: 0,
          description: event.data.toolExecution?.toolName
        }
        agent.toolExecution = event.data.toolExecution as ToolExecution
        agent.stats.totalToolCalls++
        break

      case 'tool_call_progress':
        if (agent.toolExecution && event.data.progress) {
          agent.toolExecution.progress = event.data.progress
          agent.currentActivity.progress = event.data.progress
        }
        break

      case 'tool_call_result':
        if (agent.toolExecution) {
          agent.toolExecution.status = 'completed'
          agent.toolExecution.result = event.data.result
          agent.toolExecution.duration = Date.now() - agent.toolExecution.startTime
        }
        break

      case 'tool_call_error':
        if (agent.toolExecution) {
          agent.toolExecution.status = 'failed'
          agent.toolExecution.error = event.data.error
        }
        break

      case 'output_start':
        agent.currentActivity = {
          type: 'outputting',
          startTime: event.timestamp,
          duration: 0,
          progress: 0
        }
        agent.output.isStreaming = true
        agent.output.content = ''
        agent.output.chunks = []
        agent.output.currentPosition = 0
        break

      case 'output_chunk':
        if (event.data.chunk) {
          const chunk = event.data.chunk as OutputChunk
          agent.output.chunks.push(chunk)
          agent.output.content += chunk.content
          agent.output.currentPosition = agent.output.content.length
        }
        break

      case 'output_end':
        agent.output.isStreaming = false
        agent.output.totalLength = agent.output.content.length
        agent.stats.tasksCompleted++
        updateSuccessRate(agent)
        break

      case 'permission_request':
        agent.permissionWaiting = event.data.permission as PermissionWaiting
        agent.currentActivity.type = 'waiting'
        break

      case 'permission_response':
        agent.permissionWaiting = null
        break

      case 'status_change':
        agent.currentActivity.type = event.data.newStatus as AgentActivityType
        break

      case 'level_up':
        // 由agent-pet store处理
        break

      case 'achievement_unlocked':
        // 由agent-pet store处理
        break
    }

    // 更新持续时间
    agent.currentActivity.duration = Date.now() - agent.currentActivity.startTime

    // 更新Map
    agents.value.set(event.agentId, { ...agent, lastUpdateTime: Date.now() })
  }

  // 处理权限响应
  function respondToPermission(agentId: string, optionId: string) {
    const agent = agents.value.get(agentId)
    if (!agent || !agent.permissionWaiting) return

    addGlobalEvent('permission_response', agentId, { optionId }, 'info')
    agent.permissionWaiting = null
    agents.value.set(agentId, { ...agent, lastUpdateTime: Date.now() })
  }

  // 添加全局事件
  function addGlobalEvent(
    type: string,
    agentId: string,
    data: any,
    severity: 'info' | 'warning' | 'error' | 'success' | 'critical'
  ) {
    const event: RealtimeEvent = {
      id: `event-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
      type: type as any,
      agentId,
      timestamp: Date.now(),
      data,
      severity
    }
    globalEvents.value.push(event)
    if (globalEvents.value.length > 100) {
      globalEvents.value.shift()
    }
  }

  // 更新成功率
  function updateSuccessRate(agent: AgentRealtimeStatus) {
    const total = agent.stats.tasksCompleted + agent.stats.tasksFailed
    agent.stats.successRate = total > 0
      ? (agent.stats.tasksCompleted / total) * 100
      : 100
  }

  // 设置连接状态
  function setConnectionStatus(status: 'idle' | 'connecting' | 'connected' | 'error') {
    connectionStatus.value = status
    isConnected.value = status === 'connected'
  }

  // 清空所有数据
  function clearAll() {
    agents.value.clear()
    activeAgentId.value = null
    globalEvents.value = []
  }

  // 初始化模拟数据（演示用）
  function initializeMockData() {
    // 注册几个Agent
    registerAgent('planner-001', 'Planner')
    registerAgent('architect-001', 'Architect')
    registerAgent('tdd-guide-001', 'TDD Guide')
    registerAgent('code-reviewer-001', 'Code Reviewer')

    // 设置活跃Agent
    setActiveAgent('planner-001')

    // 模拟一些事件
    setTimeout(() => {
      handleRealtimeEvent({
        id: 'mock-1',
        type: 'thinking_start',
        agentId: 'planner-001',
        timestamp: Date.now(),
        data: { description: '分析任务需求...' },
        severity: 'info'
      })
    }, 500)

    setTimeout(() => {
      handleRealtimeEvent({
        id: 'mock-2',
        type: 'thinking_chunk',
        agentId: 'planner-001',
        timestamp: Date.now(),
        data: {
          chunk: {
            id: 'chunk-1',
            content: '首先需要了解用户的具体需求...',
            timestamp: Date.now(),
            duration: 150,
            depth: 2
          }
        },
        severity: 'info'
      })
    }, 1000)

    setTimeout(() => {
      handleRealtimeEvent({
        id: 'mock-3',
        type: 'tool_call_start',
        agentId: 'architect-001',
        timestamp: Date.now(),
        data: {
          toolExecution: {
            id: 'tool-1',
            toolName: 'readFile',
            parameters: { path: '/src/main.ts' },
            status: 'running',
            progress: 0,
            startTime: Date.now()
          }
        },
        severity: 'info'
      })
    }, 2000)
  }

  return {
    // State
    agents,
    activeAgentId,
    globalEvents,
    isConnected,
    connectionStatus,

    // Computed
    activeAgent,
    allAgents,
    thinkingAgents,
    executingAgents,
    outputtingAgents,
    waitingAgents,
    idleAgents,
    errorAgents,
    recentEvents,
    globalStats,

    // Methods
    registerAgent,
    removeAgent,
    setActiveAgent,
    updateAgentStatus,
    handleRealtimeEvent,
    respondToPermission,
    addGlobalEvent,
    setConnectionStatus,
    clearAll,
    initializeMockData
  }
})