import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  CollaborationNode,
  CollaborationEdge,
  CollaborationEvent,
  CollaborationProtocol,
  AgentCapability,
  TimelineData,
  KanbanData,
  CollaborationNetworkStats,
  CollaborationNetworkConfig,
} from '@/lib/collaboration/types'

export const useCollaborationStore = defineStore('collaboration', () => {
  // State
  const nodes = ref<CollaborationNode[]>([])
  const edges = ref<CollaborationEdge[]>([])
  const events = ref<CollaborationEvent[]>([])
  const protocols = ref<CollaborationProtocol[]>([])
  const selectedNodeId = ref<string | null>(null)
  const selectedEdgeId = ref<string | null>(null)
  const hoveredNodeId = ref<string | null>(null)
  const hoveredEdgeId = ref<string | null>(null)
  const viewMode = ref<'network' | 'timeline' | 'kanban'>('network')
  const config = ref<CollaborationNetworkConfig>({
    layoutAlgorithm: 'force-directed',
    animationEnabled: true,
    animationSpeed: 0.5,
    nodeSize: 'medium',
    edgeStyle: 'curved',
    showCapabilities: true,
    showProtocols: true,
    showMetrics: true,
    autoRefresh: true,
    refreshInterval: 2000,
  })
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Computed
  const activeNodes = computed(() => {
    return nodes.value.filter(n => n.status === 'active')
  })

  const waitingNodes = computed(() => {
    return nodes.value.filter(n => n.status === 'waiting')
  })

  const errorNodes = computed(() => {
    return nodes.value.filter(n => n.status === 'error')
  })

  const flowingEdges = computed(() => {
    return edges.value.filter(e => e.status === 'flowing')
  })

  const recentEvents = computed(() => {
    const now = Date.now()
    const fiveMinutes = 5 * 60 * 1000
    return events.value.filter(e => now - e.timestamp < fiveMinutes)
  })

  const stats = computed<CollaborationNetworkStats>(() => {
    return {
      totalAgents: nodes.value.length,
      activeAgents: activeNodes.value.length,
      totalTasks: edges.value.length,
      runningTasks: flowingEdges.value.length,
      completedTasks: edges.value.filter(e => e.status === 'completed').length,
      failedTasks: edges.value.filter(e => e.status === 'failed').length,
      averageTaskDuration: calculateAverageTaskDuration(),
      collaborationEfficiency: calculateCollaborationEfficiency(),
      messageCount: events.value.filter(e => e.type.includes('message')).length,
      toolCallCount: events.value.filter(e => e.type.includes('tool')).length,
      protocolInvocations: events.value.filter(e => e.type === 'protocol_invoked').length,
    }
  })

  const timelineData = computed<TimelineData>(() => {
    const now = Date.now()
    const oneHour = 60 * 60 * 1000
    return {
      startTime: now - oneHour,
      endTime: now,
      agents: generateTimelineAgents(),
      events: recentEvents.value,
      milestones: generateMilestones(),
    }
  })

  const kanbanData = computed<KanbanData>(() => {
    return {
      columns: generateKanbanColumns(),
      agents: generateKanbanAgents(),
    }
  })

  const selectedNode = computed(() => {
    if (!selectedNodeId.value) return null
    return nodes.value.find(n => n.id === selectedNodeId.value)
  })

  const selectedNodeCapabilities = computed<AgentCapability[]>(() => {
    if (!selectedNode.value) return []
    return selectedNode.value.capabilities
  })

  const selectedNodeProtocols = computed<CollaborationProtocol[]>(() => {
    if (!selectedNode.value) return []
    return protocols.value.filter(p => p.participants.includes(selectedNode.value!.agentId))
  })

  // Helper functions
  function calculateAverageTaskDuration(): number {
    const completedEdges = edges.value.filter(e => e.status === 'completed' && e.duration)
    if (completedEdges.length === 0) return 0
    const totalDuration = completedEdges.reduce((sum, e) => sum + (e.duration || 0), 0)
    return Math.round(totalDuration / completedEdges.length)
  }

  function calculateCollaborationEfficiency(): number {
    const total = edges.value.length
    if (total === 0) return 100
    const successful = edges.value.filter(e => e.status === 'completed').length
    return Math.round((successful / total) * 100)
  }

  function generateTimelineAgents() {
    return nodes.value.map(node => ({
      agentId: node.agentId,
      agentName: node.agentName,
      activities: generateAgentActivities(node),
      color: getNodeColor(node.status),
    }))
  }

  function generateAgentActivities(node: CollaborationNode) {
    // Generate activities from events
    const agentEvents = events.value.filter(e =>
      e.sourceAgentId === node.agentId || e.targetAgentId === node.agentId
    )

    return agentEvents.map(event => ({
      id: event.id,
      startTime: event.timestamp,
      endTime: event.timestamp + (event.details.duration || 1000),
      type: getActivityType(event.type),
      taskId: event.taskId,
      description: event.details.summary,
    }))
  }

  function getNodeColor(status: string): string {
    const colors: Record<string, string> = {
      idle: '#6B7280',
      active: '#3B82F6',
      waiting: '#F59E0B',
      error: '#EF4444',
    }
    return colors[status] || '#6B7280'
  }

  function getActivityType(eventType: string): 'task' | 'waiting' | 'idle' | 'error' {
    if (eventType.includes('fail') || eventType.includes('error')) return 'error'
    if (eventType.includes('task')) return 'task'
    if (eventType.includes('waiting')) return 'waiting'
    return 'idle'
  }

  function generateMilestones() {
    // Generate milestones from important events
    const importantEvents = events.value.filter(e =>
      e.type === 'task_complete' || e.type === 'task_fail' || e.type === 'protocol_invoked'
    )

    return importantEvents.slice(0, 5).map(event => ({
      id: `milestone-${event.id}`,
      timestamp: event.timestamp,
      name: event.details.summary,
      description: event.details.description,
      type: (event.type === 'task_complete' ? 'completion' : event.type === 'task_fail' ? 'error' : 'checkpoint') as 'completion' | 'error' | 'checkpoint',
      relatedTaskIds: event.taskId ? [event.taskId] : undefined,
    }))
  }

  function generateKanbanColumns() {
    return [
      {
        id: 'pending',
        name: 'Pending',
        status: 'pending' as 'pending' | 'in_progress' | 'completed' | 'failed' | 'testing' | 'reviewing',
        tasks: edges.value.filter(e => e.status === 'pending').map(edgeToKanbanTask),
        color: '#FFA500',
      },
      {
        id: 'in_progress',
        name: 'In Progress',
        status: 'in_progress' as 'pending' | 'in_progress' | 'completed' | 'failed' | 'testing' | 'reviewing',
        tasks: edges.value.filter(e => e.status === 'flowing').map(edgeToKanbanTask),
        color: '#3B82F6',
      },
      {
        id: 'completed',
        name: 'Completed',
        status: 'completed' as 'pending' | 'in_progress' | 'completed' | 'failed' | 'testing' | 'reviewing',
        tasks: edges.value.filter(e => e.status === 'completed').map(edgeToKanbanTask),
        color: '#10B981',
      },
      {
        id: 'failed',
        name: 'Failed',
        status: 'failed' as 'pending' | 'in_progress' | 'completed' | 'failed' | 'testing' | 'reviewing',
        tasks: edges.value.filter(e => e.status === 'failed').map(edgeToKanbanTask),
        color: '#EF4444',
      },
    ]
  }

  function generateKanbanAgents() {
    return nodes.value.map(node => ({
      agentId: node.agentId,
      agentName: node.agentName,
      tasks: edges.value
        .filter(e => e.sourceAgentId === node.agentId || e.targetAgentId === node.agentId)
        .map(edgeToKanbanTask),
      capacity: node.maxLoad,
      currentLoad: node.currentLoad,
    }))
  }

  function edgeToKanbanTask(edge: CollaborationEdge) {
    return {
      id: `kanban-${edge.id}`,
      taskId: edge.taskId,
      title: edge.taskDescription,
      assignedAgentId: edge.targetAgentId,
      priority: 'medium' as 'low' | 'medium' | 'high' | 'critical',
      progress: edge.status === 'completed' ? 100 : edge.status === 'flowing' ? 50 : 0,
      tags: [edge.messageType || 'task'],
      createdAt: edge.timestamp,
      updatedAt: edge.timestamp,
      estimatedTime: edge.duration ? Math.round(edge.duration / 1000 / 60) : undefined,
      actualTime: edge.duration ? Math.round(edge.duration / 1000 / 60) : undefined,
    }
  }

  // Actions
  function addNode(node: CollaborationNode) {
    nodes.value.push(node)
  }

  function updateNode(nodeId: string, updates: Partial<CollaborationNode>) {
    const node = nodes.value.find(n => n.id === nodeId)
    if (node) {
      Object.assign(node, updates)
    }
  }

  function removeNode(nodeId: string) {
    nodes.value = nodes.value.filter(n => n.id !== nodeId)
    edges.value = edges.value.filter(e =>
      e.sourceAgentId !== nodeId && e.targetAgentId !== nodeId
    )
  }

  function addEdge(edge: CollaborationEdge) {
    edges.value.push(edge)
  }

  function updateEdge(edgeId: string, updates: Partial<CollaborationEdge>) {
    const edge = edges.value.find(e => e.id === edgeId)
    if (edge) {
      Object.assign(edge, updates)
    }
  }

  function removeEdge(edgeId: string) {
    edges.value = edges.value.filter(e => e.id !== edgeId)
  }

  function addEvent(event: CollaborationEvent) {
    events.value.unshift(event)
    // Keep only last 100 events
    if (events.value.length > 100) {
      events.value = events.value.slice(0, 100)
    }
  }

  function clearEvents() {
    events.value = []
  }

  function addProtocol(protocol: CollaborationProtocol) {
    protocols.value.push(protocol)
  }

  function removeProtocol(protocolId: string) {
    protocols.value = protocols.value.filter(p => p.id !== protocolId)
  }

  function selectNode(nodeId: string | null) {
    selectedNodeId.value = nodeId
  }

  function selectEdge(edgeId: string | null) {
    selectedEdgeId.value = edgeId
  }

  function hoverNode(nodeId: string | null) {
    hoveredNodeId.value = nodeId
  }

  function hoverEdge(edgeId: string | null) {
    hoveredEdgeId.value = edgeId
  }

  function setViewMode(mode: 'network' | 'timeline' | 'kanban') {
    viewMode.value = mode
  }

  function updateConfig(newConfig: Partial<CollaborationNetworkConfig>) {
    Object.assign(config.value, newConfig)
  }

  function setLoading(loading: boolean) {
    isLoading.value = loading
  }

  function setError(errorMsg: string | null) {
    error.value = errorMsg
  }

  function clearAll() {
    nodes.value = []
    edges.value = []
    events.value = []
    protocols.value = []
    selectedNodeId.value = null
    selectedEdgeId.value = null
    error.value = null
  }

  return {
    // State
    nodes,
    edges,
    events,
    protocols,
    selectedNodeId,
    selectedEdgeId,
    hoveredNodeId,
    hoveredEdgeId,
    viewMode,
    config,
    isLoading,
    error,

    // Computed
    activeNodes,
    waitingNodes,
    errorNodes,
    flowingEdges,
    recentEvents,
    stats,
    timelineData,
    kanbanData,
    selectedNode,
    selectedNodeCapabilities,
    selectedNodeProtocols,

    // Actions
    addNode,
    updateNode,
    removeNode,
    addEdge,
    updateEdge,
    removeEdge,
    addEvent,
    clearEvents,
    addProtocol,
    removeProtocol,
    selectNode,
    selectEdge,
    hoverNode,
    hoverEdge,
    setViewMode,
    updateConfig,
    setLoading,
    setError,
    clearAll,
  }
})