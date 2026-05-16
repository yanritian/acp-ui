<script setup lang="ts">
import { computed } from 'vue'
import type { NodeProps } from '@vue-flow/core'
import type { AgentCapability, CollaborationNetworkConfig } from '@/lib/collaboration/types'

interface AgentNodeData {
  agentId: string
  agentName: string
  agentType: string
  status: 'idle' | 'active' | 'waiting' | 'error'
  capabilities: AgentCapability[]
  currentLoad: number
  maxLoad: number
  description?: string
  avatarUrl?: string
  config: CollaborationNetworkConfig
}

const props = defineProps<NodeProps<AgentNodeData>>()

// Computed
const statusColor = computed(() => {
  const colors: Record<string, string> = {
    idle: '#6B7280',
    active: '#3B82F6',
    waiting: '#F59E0B',
    error: '#EF4444',
  }
  return colors[props.data.status] || '#6B7280'
})

const statusIcon = computed(() => {
  const icons: Record<string, string> = {
    idle: '💤',
    active: '⚡',
    waiting: '⏳',
    error: '❌',
  }
  return icons[props.data.status] || '🤖'
})

const loadPercentage = computed(() => {
  return Math.round((props.data.currentLoad / props.data.maxLoad) * 100)
})

const loadColor = computed(() => {
  if (loadPercentage.value >= 80) return '#EF4444'
  if (loadPercentage.value >= 50) return '#F59E0B'
  return '#10B981'
})

const primaryCapabilities = computed(() => {
  return props.data.capabilities.slice(0, 3)
})

const agentTypeIcon = computed(() => {
  const icons: Record<string, string> = {
    planner: '📋',
    architect: '🏗️',
    tddGuide: '🧪',
    codeReviewer: '👀',
    securityReviewer: '🔒',
    buildErrorResolver: '🔧',
    e2eRunner: '🚀',
    refactorCleaner: '🧹',
    docUpdater: '📝',
    generalPurpose: '🤖',
    explore: '🔍',
    custom: '⚙️',
  }
  return icons[props.data.agentType] || '🤖'
})
</script>

<template>
  <div class="agent-node-wrapper" :class="`status-${data.status}`">
    <!-- Status indicator -->
    <div class="status-indicator" :style="{ backgroundColor: statusColor }">
      <span class="status-icon">{{ statusIcon }}</span>
    </div>

    <!-- Main content -->
    <div class="node-content">
      <!-- Header -->
      <div class="node-header">
        <div class="agent-type-icon">{{ agentTypeIcon }}</div>
        <div class="agent-name">{{ data.agentName }}</div>
      </div>

      <!-- Load bar -->
      <div class="load-section">
        <div class="load-label">
          <span>Load</span>
          <span class="load-value">{{ loadPercentage }}%</span>
        </div>
        <div class="load-bar">
          <div
            class="load-fill"
            :style="{
              width: `${loadPercentage}%`,
              backgroundColor: loadColor,
            }"
          />
        </div>
        <div class="load-numbers">
          <span>{{ data.currentLoad }}/{{ data.maxLoad }}</span>
        </div>
      </div>

      <!-- Capabilities (if enabled) -->
      <div v-if="data.config.showCapabilities && primaryCapabilities.length > 0" class="capabilities-section">
        <div class="capabilities-label">Capabilities</div>
        <div class="capabilities-list">
          <div v-for="cap in primaryCapabilities" :key="cap.id" class="capability-item">
            <span class="capability-icon">{{ cap.icon }}</span>
            <span class="capability-name">{{ cap.name }}</span>
          </div>
        </div>
      </div>

      <!-- Description (if provided and enabled) -->
      <div v-if="data.config.showMetrics && data.description" class="description-section">
        <div class="description-text">{{ data.description }}</div>
      </div>
    </div>

    <!-- Handles for edges -->
    <div class="vue-flow__handle vue-flow__handle-top" data-handleid="top" />
    <div class="vue-flow__handle vue-flow__handle-bottom" data-handleid="bottom" />
    <div class="vue-flow__handle vue-flow__handle-left" data-handleid="left" />
    <div class="vue-flow__handle vue-flow__handle-right" data-handleid="right" />
  </div>
</template>

<style scoped>
.agent-node-wrapper {
  min-width: 180px;
  max-width: 250px;
  background: white;
  border: 2px solid #E5E7EB;
  border-radius: 8px;
  padding: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
  position: relative;
}

.agent-node-wrapper:hover {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px);
}

.status-idle {
  opacity: 0.7;
}

.status-active {
  border-color: #3B82F6;
  animation: pulse-border 2s infinite;
}

.status-waiting {
  border-color: #F59E0B;
}

.status-error {
  border-color: #EF4444;
}

@keyframes pulse-border {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(59, 130, 246, 0.4);
  }
  50% {
    box-shadow: 0 0 0 8px rgba(59, 130, 246, 0.2);
  }
}

.status-indicator {
  position: absolute;
  top: -8px;
  right: -8px;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px solid white;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.status-icon {
  font-size: 16px;
}

.node-content {
  padding: 12px;
}

.node-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.agent-type-icon {
  font-size: 24px;
}

.agent-name {
  font-weight: 600;
  font-size: 14px;
  color: #1F2937;
}

.load-section {
  margin-bottom: 8px;
}

.load-label {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #6B7280;
  margin-bottom: 4px;
}

.load-value {
  font-weight: 600;
}

.load-bar {
  height: 4px;
  background: #E5E7EB;
  border-radius: 2px;
  overflow: hidden;
}

.load-fill {
  height: 100%;
  transition: width 0.3s ease;
}

.load-numbers {
  font-size: 11px;
  color: #9CA3AF;
  margin-top: 2px;
}

.capabilities-section {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #E5E7EB;
}

.capabilities-label {
  font-size: 11px;
  color: #9CA3AF;
  margin-bottom: 4px;
}

.capabilities-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.capability-item {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 2px 6px;
  background: #F3F4F6;
  border-radius: 4px;
  font-size: 10px;
}

.capability-icon {
  font-size: 12px;
}

.capability-name {
  color: #4B5563;
}

.description-section {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #E5E7EB;
}

.description-text {
  font-size: 11px;
  color: #6B7280;
  line-height: 1.4;
}

.vue-flow__handle {
  width: 8px;
  height: 8px;
  background: #6B7280;
  border: 2px solid white;
}

.vue-flow__handle-top {
  top: -4px;
}

.vue-flow__handle-bottom {
  bottom: -4px;
}

.vue-flow__handle-left {
  left: -4px;
}

.vue-flow__handle-right {
  right: -4px;
}
</style>