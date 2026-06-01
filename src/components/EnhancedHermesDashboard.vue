<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useCollaborationStore } from '@/stores/collaboration'
import CollaborationNetworkFlow from '@/components/collaboration/CollaborationNetworkFlow.vue'
import CollaborationTimeline from '@/components/collaboration/CollaborationTimeline.vue'
import CollaborationKanban from '@/components/collaboration/CollaborationKanban.vue'
import CapabilityProtocolView from '@/components/collaboration/CapabilityProtocolView.vue'
import { orchestrator, type Event } from '@/lib/orchestrator'
import { agentMatcher } from '@/lib/agent-matcher'
import { useI18n } from '@/locales'

const { t } = useI18n()

// Store
const collaborationStore = useCollaborationStore()

// State
const activeView = ref<'network' | 'timeline' | 'kanban'>('network')
const showCapabilityPanel = ref(false)
const autoRefresh = ref(true)
let refreshIntervalId: ReturnType<typeof setInterval> | null = null

// Computed
const stats = computed(() => collaborationStore.stats)
const selectedNode = computed(() => collaborationStore.selectedNode)
const hasData = computed(() => {
  return collaborationStore.nodes.length > 0 || collaborationStore.edges.length > 0
})

// Initialize mock data for demo
function initializeMockData() {
  // Register mock agents first
  const mockAgentTypes = [
    { agentId: 'planner-001', capabilities: ['planning', 'task-breakdown', 'resource-allocation'], specialization: ['orchestration'], currentLoad: 1, maxLoad: 3, performanceScore: 90 },
    { agentId: 'architect-001', capabilities: ['architecture', 'design', 'code-review'], specialization: ['system-design'], currentLoad: 2, maxLoad: 4, performanceScore: 95 },
    { agentId: 'tddGuide-001', capabilities: ['testing', 'tdd', 'refactoring'], specialization: ['quality'], currentLoad: 1, maxLoad: 3, performanceScore: 92 },
    { agentId: 'codeReviewer-001', capabilities: ['review', 'lint', 'security'], specialization: ['qa'], currentLoad: 0, maxLoad: 2, performanceScore: 88 },
    { agentId: 'securityReviewer-001', capabilities: ['security', 'audit', 'compliance'], specialization: ['security'], currentLoad: 1, maxLoad: 2, performanceScore: 95 },
  ]

  mockAgentTypes.forEach(agent => {
    agentMatcher.registerAgent(agent)
  })

  // Create sample agents from registered agents
  const agents = agentMatcher.getAllAgents()

  agents.forEach((agent, index) => {
    const node = {
      id: `node-${agent.agentId}`,
      agentId: agent.agentId,
      agentName: agent.agentId.split('-')[0],
      agentType: agent.agentId.split('-')[0] as any,
      position: {
        x: 100 + index * 150,
        y: 100 + (index % 3) * 100,
      },
      status: (agent.currentLoad > 0 ? 'active' : 'idle') as 'idle' | 'active' | 'waiting' | 'error',
      capabilities: generateMockCapabilities(agent.agentId),
      currentLoad: agent.currentLoad,
      maxLoad: agent.maxLoad,
      description: `${agent.agentId} - Specialized agent`,
    }

    collaborationStore.addNode(node)
  })

  // Create sample task flows
  const sampleTasks = [
    {
      id: 'task-1',
      source: 'planner-001',
      target: 'architect-001',
      description: 'Design system architecture',
      status: 'flowing',
    },
    {
      id: 'task-2',
      source: 'architect-001',
      target: 'tddGuide-001',
      description: 'Implement TDD workflow',
      status: 'pending',
    },
    {
      id: 'task-3',
      source: 'codeReviewer-001',
      target: 'securityReviewer-001',
      description: 'Security audit',
      status: 'completed',
    },
  ]

  sampleTasks.forEach(task => {
    const edge = {
      id: `edge-${task.id}`,
      sourceAgentId: `node-${task.source}`,
      targetAgentId: `node-${task.target}`,
      taskId: task.id,
      taskDescription: task.description,
      status: task.status as 'pending' | 'flowing' | 'completed' | 'failed',
      timestamp: Date.now() - Math.random() * 10000,
      animationProgress: task.status === 'flowing' ? 0.5 : 1,
      messageType: 'task_assign' as 'task_assign' | 'task_transfer' | 'request' | 'response' | 'notification',
      payloadPreview: task.description.slice(0, 50),
      duration: task.status === 'completed' ? Math.round(Math.random() * 5000) : undefined,
    }

    collaborationStore.addEdge(edge)
  })

  // Create sample events
  const sampleEvents = [
    {
      type: 'task_assign',
      summary: 'Architecture design task assigned',
      source: 'planner-001',
      target: 'architect-001',
    },
    {
      type: 'tool_call',
      summary: 'File read operation',
      source: 'architect-001',
      duration: 120,
    },
    {
      type: 'capability_match',
      summary: 'Security review capability matched',
      source: 'securityReviewer-001',
    },
  ]

  sampleEvents.forEach((event, index) => {
    collaborationStore.addEvent({
      id: `event-${index}`,
      type: event.type as 'task_assign' | 'task_transfer' | 'task_complete' | 'task_fail' | 'message_sent' | 'message_received' | 'tool_call' | 'tool_result' | 'status_change' | 'protocol_invoked' | 'capability_match' | 'load_balance',
      timestamp: Date.now() - index * 1000,
      sourceAgentId: event.source,
      targetAgentId: event.target,
      payload: {},
      details: {
        summary: event.summary,
        duration: event.duration,
      },
      severity: 'info' as 'info' | 'warning' | 'error' | 'critical',
    })
  })
}

function generateMockCapabilities(agentType: string) {
  const capabilitiesByType: Record<string, any[]> = {
    planner: [
      { id: 'cap-1', name: 'Task Planning', description: 'Break down complex tasks', icon: '📋', category: 'planning', proficiency: 90 },
      { id: 'cap-2', name: 'Resource Allocation', description: 'Assign agents to tasks', icon: '🎭', category: 'orchestration', proficiency: 85 },
    ],
    architect: [
      { id: 'cap-3', name: 'System Design', description: 'Design system architecture', icon: '🏗️', category: 'planning', proficiency: 95 },
      { id: 'cap-4', name: 'Code Review', description: 'Review code quality', icon: '👀', category: 'review', proficiency: 80 },
    ],
    tddGuide: [
      { id: 'cap-5', name: 'Test Writing', description: 'Write comprehensive tests', icon: '🧪', category: 'testing', proficiency: 92 },
      { id: 'cap-6', name: 'Refactoring', description: 'Refactor legacy code', icon: '🧹', category: 'execution', proficiency: 88 },
    ],
    codeReviewer: [
      { id: 'cap-7', name: 'Code Analysis', description: 'Analyze code patterns', icon: '🔍', category: 'review', proficiency: 90 },
      { id: 'cap-8', name: 'Best Practices', description: 'Enforce coding standards', icon: '✓', category: 'review', proficiency: 85 },
    ],
    securityReviewer: [
      { id: 'cap-9', name: 'Security Audit', description: 'Identify security vulnerabilities', icon: '🔒', category: 'review', proficiency: 95 },
      { id: 'cap-10', name: 'Compliance Check', description: 'Verify compliance requirements', icon: '📋', category: 'review', proficiency: 90 },
    ],
  }

  return capabilitiesByType[agentType.split('-')[0]] || [
    { id: 'cap-default', name: 'General Execution', description: 'Execute general tasks', icon: '🤖', category: 'execution', proficiency: 70 },
  ]
}

// Event handlers
function handleOrchestratorEvent(event: Event) {
  // Convert orchestrator events to collaboration events
  collaborationStore.addEvent({
    id: `orch-event-${Date.now()}`,
    type: event.type as 'task_assign' | 'task_transfer' | 'task_complete' | 'task_fail' | 'message_sent' | 'message_received' | 'tool_call' | 'tool_result' | 'status_change' | 'protocol_invoked' | 'capability_match' | 'load_balance',
    timestamp: typeof event.timestamp === 'number' ? event.timestamp : Date.now(),
    sourceAgentId: 'orchestrator',
    payload: event.payload,
    details: {
      summary: (event as any).message || event.type,
    },
    severity: 'info' as 'info' | 'warning' | 'error' | 'critical',
  })
}

function handleNodeClick(nodeId: string) {
  collaborationStore.selectNode(nodeId)
  showCapabilityPanel.value = true
}

function handleEdgeClick(edgeId: string) {
  collaborationStore.selectEdge(edgeId)
}

function handleViewChange(view: 'network' | 'timeline' | 'kanban') {
  activeView.value = view
  collaborationStore.setViewMode(view)
}

function refreshData() {
  collaborationStore.setLoading(true)
  // Simulate data refresh
  setTimeout(() => {
    collaborationStore.setLoading(false)
  }, 1000)
}

function toggleAutoRefresh() {
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId)
    refreshIntervalId = null
  }

  if (autoRefresh.value) {
    refreshIntervalId = setInterval(refreshData, 5000)
  }
}

// Lifecycle
onMounted(() => {
  // Subscribe to orchestrator events
  orchestrator.subscribe('task_complete', handleOrchestratorEvent)
  orchestrator.subscribe('task_failed', handleOrchestratorEvent)
  orchestrator.subscribe('review_result', handleOrchestratorEvent)
  orchestrator.subscribe('test_result', handleOrchestratorEvent)

  // Start auto refresh if enabled
  if (autoRefresh.value) {
    refreshIntervalId = setInterval(refreshData, 5000)
  }
})

onUnmounted(() => {
  orchestrator.unsubscribe('task_complete', handleOrchestratorEvent)
  orchestrator.unsubscribe('task_failed', handleOrchestratorEvent)
  orchestrator.unsubscribe('review_result', handleOrchestratorEvent)
  orchestrator.unsubscribe('test_result', handleOrchestratorEvent)

  if (refreshIntervalId) {
    clearInterval(refreshIntervalId)
  }
})
</script>

<template>
  <div class="enhanced-hermes-dashboard">
    <!-- Header -->
    <header class="dashboard-header">
      <div class="header-left">
        <h1 class="dashboard-title">{{ t('collaboration.title') }}</h1>
        <p class="dashboard-subtitle">{{ t('collaboration.subtitle') }}</p>
      </div>

      <div class="header-right">
        <!-- Stats badges -->
        <div class="stats-badges">
          <div class="stat-badge">
            <span class="stat-icon">🤖</span>
            <span class="stat-value">{{ stats.activeAgents }}/{{ stats.totalAgents }}</span>
            <span class="stat-label">{{ t('collaboration.agents') }}</span>
          </div>

          <div class="stat-badge">
            <span class="stat-icon">⚡</span>
            <span class="stat-value">{{ stats.runningTasks }}</span>
            <span class="stat-label">{{ t('collaboration.running') }}</span>
          </div>

          <div class="stat-badge">
            <span class="stat-icon">✅</span>
            <span class="stat-value">{{ stats.completedTasks }}</span>
            <span class="stat-label">{{ t('collaboration.completed') }}</span>
          </div>

          <div class="stat-badge">
            <span class="stat-icon">📊</span>
            <span class="stat-value">{{ stats.collaborationEfficiency }}%</span>
            <span class="stat-label">{{ t('collaboration.efficiency') }}</span>
          </div>
        </div>

        <!-- Controls -->
        <div class="header-controls">
          <label class="auto-refresh-toggle">
            <input type="checkbox" v-model="autoRefresh" @change="toggleAutoRefresh" />
            <span>{{ t('common.autoRefresh') }}</span>
          </label>

          <button class="refresh-button" @click="refreshData" :disabled="collaborationStore.isLoading">
            {{ collaborationStore.isLoading ? t('common.loading') : t('common.refresh') }}
          </button>
        </div>
      </div>
    </header>

    <!-- Main content -->
    <div class="dashboard-content">
      <!-- Left sidebar - View mode selector -->
      <aside class="sidebar">
        <div class="sidebar-header">{{ t('collaboration.viewMode') }}</div>

        <div class="view-mode-buttons">
          <button
            :class="['mode-button', { active: activeView === 'network' }]"
            @click="handleViewChange('network')"
          >
            <span class="mode-icon">🕸️</span>
            <span class="mode-text">{{ t('collaboration.network') }}</span>
          </button>

          <button
            :class="['mode-button', { active: activeView === 'timeline' }]"
            @click="handleViewChange('timeline')"
          >
            <span class="mode-icon">⏱️</span>
            <span class="mode-text">{{ t('collaboration.timeline') }}</span>
          </button>

          <button
            :class="['mode-button', { active: activeView === 'kanban' }]"
            @click="handleViewChange('kanban')"
          >
            <span class="mode-icon">📋</span>
            <span class="mode-text">{{ t('collaboration.kanban') }}</span>
          </button>
        </div>

        <!-- Recent events -->
        <div class="sidebar-section">
          <div class="section-header">{{ t('collaboration.recentEvents') }}</div>
          <div class="events-list">
            <div
              v-for="event in collaborationStore.recentEvents.slice(0, 5)"
              :key="event.id"
              class="event-item"
            >
              <span class="event-icon">{{ getEventIcon(event.type) }}</span>
              <div class="event-content">
                <div class="event-summary">{{ event.details.summary }}</div>
                <div class="event-meta">
                  <span>{{ event.sourceAgentId }}</span>
                  <span>{{ formatTime(event.timestamp) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </aside>

      <!-- Main view area -->
      <main class="main-view">
        <!-- Empty state -->
        <div v-if="!hasData" class="empty-state">
          <div class="empty-icon">🕸️</div>
          <div class="empty-title">{{ t('collaboration.noData') }}</div>
          <div class="empty-description">
            {{ t('collaboration.noDataHint') }}
          </div>
        </div>

        <!-- Network view -->
        <CollaborationNetworkFlow
          v-else-if="activeView === 'network'"
          :nodes="collaborationStore.nodes"
          :edges="collaborationStore.edges"
          :config="collaborationStore.config"
          @nodeClick="handleNodeClick"
          @edgeClick="handleEdgeClick"
        />

        <!-- Timeline view -->
        <CollaborationTimeline
          v-else-if="activeView === 'timeline'"
          :timelineData="collaborationStore.timelineData"
          :autoRefresh="autoRefresh"
        />

        <!-- Kanban view -->
        <CollaborationKanban
          v-else-if="activeView === 'kanban'"
          :kanbanData="collaborationStore.kanbanData"
        />
      </main>

      <!-- Right panel - Capability details -->
      <aside v-if="showCapabilityPanel && selectedNode" class="capability-panel">
        <CapabilityProtocolView
          :agentId="selectedNode.agentId"
          :agentName="selectedNode.agentName"
          :capabilities="selectedNode.capabilities"
          :protocols="collaborationStore.selectedNodeProtocols"
        />

        <button class="close-panel-button" @click="showCapabilityPanel = false">
          ✕ {{ t('collaboration.close') }}
        </button>
      </aside>
    </div>
  </div>
</template>

<script lang="ts">
function getEventIcon(type: string): string {
  const icons: Record<string, string> = {
    task_assign: '📋',
    task_transfer: '🔄',
    task_complete: '✅',
    task_fail: '❌',
    message_sent: '📤',
    message_received: '📥',
    tool_call: '🔧',
    tool_result: '📊',
    status_change: '🔄',
    protocol_invoked: '📜',
    capability_match: '🎯',
    load_balance: '⚖️',
  }
  return icons[type] || '⚡'
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  return `${date.getHours()}:${date.getMinutes()}:${date.getSeconds()}`
}
</script>

<style scoped>
.enhanced-hermes-dashboard {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #F8FAFC 0%, #EEF2FF 50%, #F8FAFC 100%);
  font-family: system-ui, -apple-system, sans-serif;
}

.dashboard-header {
  padding: 12px 20px;
  background: white;
  border-bottom: 1px solid #E2E8F0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.header-left {
  flex: 1;
}

.dashboard-title {
  font-size: 20px;
  font-weight: 600;
  color: #1E293B;
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.dashboard-title::before {
  content: '📡';
  font-size: 24px;
}

.dashboard-subtitle {
  font-size: 13px;
  color: #64748B;
  margin-top: 2px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stats-badges {
  display: flex;
  gap: 8px;
}

.stat-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #F1F5F9;
  border-radius: 6px;
  border: 1px solid #E2E8F0;
}

.stat-icon {
  font-size: 16px;
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: #1E293B;
}

.stat-label {
  font-size: 11px;
  color: #64748B;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.auto-refresh-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #64748B;
  cursor: pointer;
}

.auto-refresh-toggle input {
  accent-color: #3B82F6;
}

.refresh-button {
  padding: 6px 14px;
  background: #3B82F6;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.refresh-button:hover:not(:disabled) {
  background: #2563EB;
}

.refresh-button:disabled {
  background: #94A3B8;
  cursor: not-allowed;
}

.dashboard-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}

.sidebar {
  width: 220px;
  background: white;
  border-right: 1px solid #E2E8F0;
  padding: 12px;
  overflow-y: auto;
  flex-shrink: 0;
}

.sidebar-header {
  font-size: 13px;
  font-weight: 600;
  color: #475569;
  margin-bottom: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.view-mode-buttons {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 16px;
}

.mode-button {
  padding: 10px 12px;
  background: #F8FAFC;
  border: 1px solid #E2E8F0;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.15s;
  font-size: 13px;
}

.mode-button:hover {
  background: #F1F5F9;
  border-color: #CBD5E1;
}

.mode-button.active {
  background: linear-gradient(135deg, #3B82F6 0%, #2563EB 100%);
  border-color: #3B82F6;
  color: white;
  box-shadow: 0 2px 4px rgba(59, 130, 246, 0.3);
}

.mode-icon {
  font-size: 18px;
}

.mode-text {
  font-weight: 500;
}

.sidebar-section {
  margin-top: 12px;
}

.section-header {
  font-size: 12px;
  font-weight: 600;
  color: #64748B;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid #E2E8F0;
}

.events-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.event-item {
  padding: 8px;
  background: #F8FAFC;
  border-radius: 6px;
  display: flex;
  gap: 8px;
  border: 1px solid #E2E8F0;
  transition: background 0.15s;
}

.event-item:hover {
  background: #F1F5F9;
}

.event-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.event-content {
  flex: 1;
  min-width: 0;
}

.event-summary {
  font-size: 12px;
  color: #334155;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.event-meta {
  font-size: 10px;
  color: #94A3B8;
  display: flex;
  gap: 6px;
  margin-top: 2px;
}

.main-view {
  flex: 1;
  overflow: hidden;
  position: relative;
  min-height: 400px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 40px;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: #475569;
  margin-bottom: 8px;
}

.empty-description {
  font-size: 14px;
  color: #94A3B8;
  text-align: center;
  max-width: 300px;
}

.capability-panel {
  width: 280px;
  background: white;
  border-left: 1px solid #E2E8F0;
  overflow-y: auto;
  position: relative;
  flex-shrink: 0;
}

.close-panel-button {
  position: absolute;
  top: 12px;
  right: 12px;
  background: #F1F5F9;
  border: 1px solid #E2E8F0;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  cursor: pointer;
  color: #64748B;
  transition: all 0.15s;
}

.close-panel-button:hover {
  background: #E2E8F0;
  color: #334155;
}
</style>