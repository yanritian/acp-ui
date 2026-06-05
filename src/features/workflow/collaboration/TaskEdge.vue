<script setup lang="ts">
import { computed, ref } from 'vue'
import type { EdgeProps } from '@vue-flow/core'
import { BaseEdge, getSmoothStepPath, EdgeLabelRenderer } from '@vue-flow/core'

interface TaskEdgeData {
  taskId: string
  taskDescription: string
  status: 'pending' | 'flowing' | 'completed' | 'failed'
  timestamp: number
  messageType?: string
  payloadPreview?: string
  duration?: number
  animationProgress: number
}

const props = defineProps<EdgeProps<TaskEdgeData>>()

// State
const isExpanded = ref(false)

// Computed
const edgeColor = computed(() => {
  const colors: Record<string, string> = {
    pending: '#FFA500',
    flowing: '#3B82F6',
    completed: '#10B981',
    failed: '#EF4444',
  }
  return colors[props.data.status] || '#6B7280'
})

const statusText = computed(() => {
  const texts: Record<string, string> = {
    pending: '⏳ Pending',
    flowing: '⚡ Flowing',
    completed: '✅ Completed',
    failed: '❌ Failed',
  }
  return texts[props.data.status] || ''
})

const timeAgo = computed(() => {
  const diff = Date.now() - props.data.timestamp
  const seconds = Math.floor(diff / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  return `${hours}h`
})

const path = computed(() => {
  const [pathString] = getSmoothStepPath({
    sourceX: props.sourceX,
    sourceY: props.sourceY,
    targetX: props.targetX,
    targetY: props.targetY,
    sourcePosition: props.sourcePosition,
    targetPosition: props.targetPosition,
    borderRadius: 16,
  })
  return pathString
})

const labelX = computed(() => {
  return (props.sourceX + props.targetX) / 2
})

const labelY = computed(() => {
  return (props.sourceY + props.targetY) / 2
})

const messageTypeIcon = computed(() => {
  const icons: Record<string, string> = {
    task_assign: '📋',
    task_transfer: '🔄',
    request: '📤',
    response: '📥',
    notification: '📢',
  }
  return icons[props.data.messageType || ''] || '🔗'
})
</script>

<template>
  <BaseEdge
    :id="id"
    :path="path"
    :style="{
      stroke: edgeColor,
      strokeWidth: 2,
      strokeDasharray: data.status === 'flowing' ? '8 4' : '',
      animation: data.status === 'flowing' ? 'flow-animation 1s linear infinite' : '',
    }"
    :marker-end="markerEnd"
  />

  <EdgeLabelRenderer>
    <div
      class="task-edge-label"
      :style="{
        transform: `translate(${labelX}px, ${labelY}px) translate(-50%, -50%)`,
        pointerEvents: 'all',
      }"
      @click="isExpanded = !isExpanded"
    >
      <!-- Compact view -->
      <div v-if="!isExpanded" class="label-compact">
        <div class="label-header">
          <span class="message-type-icon">{{ messageTypeIcon }}</span>
          <span class="status-badge" :style="{ backgroundColor: edgeColor }">{{ statusText }}</span>
        </div>
        <div class="label-time">{{ timeAgo }}</div>
      </div>

      <!-- Expanded view -->
      <div v-else class="label-expanded">
        <div class="label-header-expanded">
          <span class="message-type-icon">{{ messageTypeIcon }}</span>
          <div class="task-info">
            <div class="task-id">{{ data.taskId }}</div>
            <div class="task-description">{{ data.taskDescription }}</div>
          </div>
          <button class="close-button" @click="isExpanded = false">×</button>
        </div>

        <div class="label-details">
          <div class="detail-row">
            <span class="detail-label">Status:</span>
            <span class="status-badge" :style="{ backgroundColor: edgeColor }">{{ statusText }}</span>
          </div>

          <div v-if="data.duration" class="detail-row">
            <span class="detail-label">Duration:</span>
            <span class="detail-value">{{ data.duration }}ms</span>
          </div>

          <div v-if="data.payloadPreview" class="detail-row">
            <span class="detail-label">Preview:</span>
            <div class="payload-preview">{{ data.payloadPreview }}</div>
          </div>

          <div class="detail-row">
            <span class="detail-label">Time:</span>
            <span class="detail-value">{{ timeAgo }}</span>
          </div>
        </div>
      </div>
    </div>
  </EdgeLabelRenderer>
</template>

<style scoped>
.task-edge-label {
  position: absolute;
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  padding: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  cursor: pointer;
  transition: all 0.2s ease;
  z-index: 10;
}

.task-edge-label:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.label-compact {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.label-header {
  display: flex;
  align-items: center;
  gap: 4px;
}

.message-type-icon {
  font-size: 16px;
}

.status-badge {
  padding: 2px 6px;
  border-radius: 4px;
  color: white;
  font-size: 10px;
  font-weight: 600;
}

.label-time {
  font-size: 11px;
  color: #9CA3AF;
}

.label-expanded {
  min-width: 200px;
  max-width: 300px;
}

.label-header-expanded {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 8px;
}

.task-info {
  flex: 1;
}

.task-id {
  font-size: 12px;
  font-weight: 600;
  color: #3B82F6;
  margin-bottom: 2px;
}

.task-description {
  font-size: 11px;
  color: #6B7280;
  line-height: 1.3;
}

.close-button {
  background: none;
  border: none;
  font-size: 18px;
  color: #9CA3AF;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.close-button:hover {
  color: #4B5563;
}

.label-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-top: 8px;
  border-top: 1px solid #E5E7EB;
}

.detail-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.detail-label {
  font-size: 11px;
  color: #9CA3AF;
  min-width: 60px;
}

.detail-value {
  font-size: 11px;
  color: #4B5563;
}

.payload-preview {
  font-size: 10px;
  color: #6B7280;
  background: #F9FAFB;
  padding: 4px;
  border-radius: 4px;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>

<style>
@keyframes flow-animation {
  0% {
    stroke-dashoffset: 16;
  }
  100% {
    stroke-dashoffset: 0;
  }
}
</style>