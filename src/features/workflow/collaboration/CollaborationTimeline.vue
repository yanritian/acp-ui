<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import type {
  TimelineData,
  TimelineAgent,
} from '@/lib/collaboration/types'

// Props
interface Props {
  timelineData: TimelineData
  autoRefresh?: boolean
  refreshInterval?: number
  timeRange?: { start: number; end: number }
}

const props = withDefaults(defineProps<Props>(), {
  autoRefresh: true,
  refreshInterval: 2000,
  timeRange: undefined,
})

// Emits
const emit = defineEmits<{
  agentClick: [agentId: string]
  activityClick: [activityId: string]
  milestoneClick: [milestoneId: string]
  eventClick: [eventId: string]
  timeRange: [range: { start: number; end: number }]
}>()

// State
const selectedAgentId = ref<string | null>(null)
const hoveredActivityId = ref<string | null>(null)
const currentTimePosition = ref<number>(0)
let refreshIntervalId: ReturnType<typeof setInterval> | null = null

// Computed
const effectiveTimeRange = computed(() => {
  if (props.timeRange) return props.timeRange
  return {
    start: props.timelineData.startTime,
    end: props.timelineData.endTime,
  }
})

const timelineDuration = computed(() => {
  return effectiveTimeRange.value.end - effectiveTimeRange.value.start
})

const milestonePositions = computed(() => {
  return props.timelineData.milestones.map(milestone => ({
    milestone,
    position: ((milestone.timestamp - effectiveTimeRange.value.start) / timelineDuration.value) * 100,
  }))
})

const eventPositions = computed(() => {
  return props.timelineData.events.map(event => ({
    event,
    position: ((event.timestamp - effectiveTimeRange.value.start) / timelineDuration.value) * 100,
  }))
})

// Helper functions
function getAgentActivitiesPosition(agent: TimelineAgent) {
  return agent.activities.map(activity => ({
    activity,
    startPosition: ((activity.startTime - effectiveTimeRange.value.start) / timelineDuration.value) * 100,
    width: ((activity.endTime - activity.startTime) / timelineDuration.value) * 100,
  }))
}

function getActivityColor(type: string): string {
  const colors: Record<string, string> = {
    task: '#3B82F6',
    waiting: '#F59E0B',
    idle: '#6B7280',
    error: '#EF4444',
  }
  return colors[type] || '#6B7280'
}

function getMilestoneIcon(type: string): string {
  const icons: Record<string, string> = {
    start: '🚀',
    checkpoint: '📍',
    completion: '🎉',
    error: '⚠️',
  }
  return icons[type] || '📌'
}

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
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function formatDuration(startTime: number, endTime: number): string {
  const duration = endTime - startTime
  const seconds = Math.floor(duration / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  return `${hours}h ${minutes % 60}m`
}

// Lifecycle
onMounted(() => {
  if (props.autoRefresh) {
    refreshIntervalId = setInterval(() => {
      currentTimePosition.value = Date.now()
    }, props.refreshInterval)
  }
})

onUnmounted(() => {
  if (refreshIntervalId) {
    clearInterval(refreshIntervalId)
  }
})
</script>

<template>
  <div class="collaboration-timeline">
    <!-- Header -->
    <div class="timeline-header">
      <div class="header-title">Collaboration Timeline</div>
      <div class="header-controls">
        <div class="time-range-display">
          {{ formatTime(effectiveTimeRange.start) }} - {{ formatTime(effectiveTimeRange.end) }}
        </div>
      </div>
    </div>

    <!-- Main timeline -->
    <div class="timeline-container">
      <!-- Time axis -->
      <div class="time-axis">
        <div class="axis-labels">
          <div
            v-for="milestonePos in milestonePositions"
            :key="milestonePos.milestone.id"
            class="milestone-marker"
            :style="{ left: `${milestonePos.position}%` }"
            @click="emit('milestoneClick', milestonePos.milestone.id)"
          >
            <span class="milestone-icon">{{ getMilestoneIcon(milestonePos.milestone.type) }}</span>
            <span class="milestone-name">{{ milestonePos.milestone.name }}</span>
          </div>
        </div>
      </div>

      <!-- Agent lanes -->
      <div class="agent-lanes">
        <div
          v-for="agent in timelineData.agents"
          :key="agent.agentId"
          class="agent-lane"
          :class="{ selected: selectedAgentId === agent.agentId }"
          @click="emit('agentClick', agent.agentId)"
        >
          <!-- Agent label -->
          <div class="agent-label" :style="{ backgroundColor: agent.color }">
            <div class="agent-name">{{ agent.agentName }}</div>
          </div>

          <!-- Agent activities -->
          <div class="agent-activities">
            <div
              v-for="activityPos in getAgentActivitiesPosition(agent)"
              :key="activityPos.activity.id"
              class="activity-bar"
              :style="{
                left: `${activityPos.startPosition}%`,
                width: `${activityPos.width}%`,
                backgroundColor: getActivityColor(activityPos.activity.type),
              }"
              :class="{ hovered: hoveredActivityId === activityPos.activity.id }"
              @mouseenter="hoveredActivityId = activityPos.activity.id"
              @mouseleave="hoveredActivityId = null"
              @click.stop="emit('activityClick', activityPos.activity.id)"
            >
              <div class="activity-tooltip">
                <div class="tooltip-header">{{ activityPos.activity.description }}</div>
                <div class="tooltip-details">
                  <div>Type: {{ activityPos.activity.type }}</div>
                  <div>Duration: {{ formatDuration(activityPos.activity.startTime, activityPos.activity.endTime) }}</div>
                  <div v-if="activityPos.activity.taskId">Task: {{ activityPos.activity.taskId }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Event markers -->
      <div class="event-markers">
        <div
          v-for="eventPos in eventPositions"
          :key="eventPos.event.id"
          class="event-marker"
          :style="{ left: `${eventPos.position}%` }"
          @click="emit('eventClick', eventPos.event.id)"
        >
          <span class="event-icon">{{ getEventIcon(eventPos.event.type) }}</span>
          <div class="event-tooltip">
            <div class="tooltip-type">{{ eventPos.event.type }}</div>
            <div class="tooltip-summary">{{ eventPos.event.details.summary }}</div>
            <div class="tooltip-time">{{ formatTime(eventPos.event.timestamp) }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Timeline controls -->
    <div class="timeline-controls">
      <button class="control-button" @click="$emit('timeRange', { start: effectiveTimeRange.start - 60000, end: effectiveTimeRange.end })">
        ← Earlier
      </button>
      <button class="control-button" @click="$emit('timeRange', { start: effectiveTimeRange.start + 60000, end: effectiveTimeRange.end })">
        Later →
      </button>
      <button class="control-button" @click="$emit('timeRange', { start: Date.now() - 300000, end: Date.now() })">
        Now
      </button>
    </div>
  </div>
</template>

<style scoped>
.collaboration-timeline {
  width: 100%;
  height: 100%;
  background: white;
  border-radius: 8px;
  padding: 16px;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #E5E7EB;
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: #1F2937;
}

.time-range-display {
  font-size: 14px;
  color: #6B7280;
}

.timeline-container {
  position: relative;
  overflow: hidden;
}

.time-axis {
  height: 40px;
  border-bottom: 2px solid #E5E7EB;
  position: relative;
}

.milestone-marker {
  position: absolute;
  top: 0;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: white;
  border: 1px solid #D1D5DB;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.milestone-marker:hover {
  background: #F9FAFB;
  border-color: #9CA3AF;
}

.milestone-icon {
  font-size: 16px;
}

.milestone-name {
  font-size: 12px;
  color: #4B5563;
}

.agent-lanes {
  margin-top: 20px;
}

.agent-lane {
  display: flex;
  margin-bottom: 16px;
  padding: 8px;
  border-radius: 8px;
  transition: all 0.2s ease;
  cursor: pointer;
}

.agent-lane:hover {
  background: #F9FAFB;
}

.agent-lane.selected {
  background: #EBF5FF;
}

.agent-label {
  width: 150px;
  padding: 8px 12px;
  border-radius: 8px;
  color: white;
  font-weight: 600;
  display: flex;
  align-items: center;
}

.agent-name {
  font-size: 14px;
}

.agent-activities {
  flex: 1;
  height: 32px;
  margin-left: 16px;
  background: #F9FAFB;
  border-radius: 4px;
  position: relative;
}

.activity-bar {
  position: absolute;
  height: 100%;
  border-radius: 4px;
  transition: all 0.2s ease;
  cursor: pointer;
}

.activity-bar:hover {
  opacity: 0.8;
  transform: translateY(-2px);
}

.activity-bar.hovered {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

.activity-tooltip {
  position: absolute;
  top: -80px;
  left: 50%;
  transform: translateX(-50%);
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  padding: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: none;
  min-width: 150px;
  z-index: 100;
}

.activity-bar:hover .activity-tooltip {
  display: block;
}

.tooltip-header {
  font-size: 12px;
  font-weight: 600;
  color: #1F2937;
  margin-bottom: 4px;
}

.tooltip-details {
  font-size: 11px;
  color: #6B7280;
}

.event-markers {
  position: absolute;
  top: 60px;
  width: 100%;
  height: 100%;
}

.event-marker {
  position: absolute;
  transform: translateX(-50%);
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: white;
  border: 2px solid #3B82F6;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.event-marker:hover {
  background: #EBF5FF;
  transform: translateX(-50%) scale(1.2);
}

.event-icon {
  font-size: 14px;
}

.event-tooltip {
  position: absolute;
  bottom: 30px;
  left: 50%;
  transform: translateX(-50%);
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  padding: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: none;
  min-width: 150px;
  z-index: 100;
}

.event-marker:hover .event-tooltip {
  display: block;
}

.tooltip-type {
  font-size: 12px;
  font-weight: 600;
  color: #3B82F6;
  margin-bottom: 4px;
}

.tooltip-summary {
  font-size: 11px;
  color: #4B5563;
  margin-bottom: 4px;
}

.tooltip-time {
  font-size: 10px;
  color: #9CA3AF;
}

.timeline-controls {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #E5E7EB;
}

.control-button {
  padding: 8px 16px;
  background: #F3F4F6;
  border: 1px solid #D1D5DB;
  border-radius: 6px;
  color: #4B5563;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.control-button:hover {
  background: #E5E7EB;
  border-color: #9CA3AF;
}
</style>