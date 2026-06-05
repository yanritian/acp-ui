<script setup lang="ts">
import { ref, computed } from 'vue'
import type { KanbanData, KanbanTask, KanbanAgent } from '@/lib/collaboration/types'

// Props
interface Props {
  kanbanData: KanbanData
  enableDrag?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  enableDrag: true,
})

// Emits
const emit = defineEmits<{
  taskDrag: [taskId: string, fromColumnId: string, toColumnId: string]
  taskClick: [taskId: string]
  agentClick: [agentId: string]
}>()

// State
const draggedTaskId = ref<string | null>(null)
const dragFromColumnId = ref<string | null>(null)
const viewMode = ref<'column' | 'agent'>('column')

// Computed
const columnsWithTasks = computed(() => {
  return props.kanbanData.columns
})

const agentsWithTasks = computed(() => {
  return props.kanbanData.agents
})

// Helper functions
function getPriorityColor(priority: string): string {
  const colors: Record<string, string> = {
    low: '#6B7280',
    medium: '#F59E0B',
    high: '#EF4444',
    critical: '#7C3AED',
  }
  return colors[priority] || '#6B7280'
}

function getPriorityIcon(priority: string): string {
  const icons: Record<string, string> = {
    low: '⬇️',
    medium: '➡️',
    high: '⬆️',
    critical: '🔴',
  }
  return icons[priority] || '•'
}

function getProgressColor(progress: number): string {
  if (progress >= 80) return '#10B981'
  if (progress >= 50) return '#3B82F6'
  if (progress >= 20) return '#F59E0B'
  return '#6B7280'
}

function getAssignedAgent(task: KanbanTask): KanbanAgent | undefined {
  if (!task.assignedAgentId) return undefined
  return props.kanbanData.agents.find(a => a.agentId === task.assignedAgentId)
}

// Drag handlers
function onDragStart(event: DragEvent, taskId: string, fromColumnId: string) {
  if (!props.enableDrag) return
  draggedTaskId.value = taskId
  dragFromColumnId.value = fromColumnId
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
    event.dataTransfer.setData('text/plain', taskId)
  }
}

function onDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

function onDrop(event: DragEvent, toColumnId: string) {
  event.preventDefault()
  if (!draggedTaskId.value || !dragFromColumnId.value) return

  emit('taskDrag', draggedTaskId.value, dragFromColumnId.value, toColumnId)

  // Reset drag state
  draggedTaskId.value = null
  dragFromColumnId.value = null
}
</script>

<template>
  <div class="collaboration-kanban">
    <!-- Header -->
    <div class="kanban-header">
      <div class="header-title">Collaboration Kanban</div>
      <div class="view-mode-toggle">
        <button
          :class="['mode-button', { active: viewMode === 'column' }]"
          @click="viewMode = 'column'"
        >
          📊 By Status
        </button>
        <button
          :class="['mode-button', { active: viewMode === 'agent' }]"
          @click="viewMode = 'agent'"
        >
          🤖 By Agent
        </button>
      </div>
    </div>

    <!-- Kanban board - Column view -->
    <div v-if="viewMode === 'column'" class="kanban-board column-view">
      <div
        v-for="column in columnsWithTasks"
        :key="column.id"
        class="kanban-column"
        @dragover="onDragOver"
        @drop="onDrop($event, column.id)"
      >
        <!-- Column header -->
        <div class="column-header" :style="{ backgroundColor: column.color }">
          <div class="column-name">{{ column.name }}</div>
          <div class="column-count">{{ column.tasks.length }}</div>
        </div>

        <!-- Column tasks -->
        <div class="column-content">
          <div
            v-for="task in column.tasks"
            :key="task.id"
            class="kanban-task-card"
            :draggable="enableDrag"
            @dragstart="onDragStart($event, task.id, column.id)"
            @click="emit('taskClick', task.taskId)"
          >
            <!-- Task header -->
            <div class="task-header">
              <span class="task-id">{{ task.taskId }}</span>
              <span class="priority-badge" :style="{ backgroundColor: getPriorityColor(task.priority) }">
                {{ getPriorityIcon(task.priority) }}
              </span>
            </div>

            <!-- Task title -->
            <div class="task-title">{{ task.title }}</div>

            <!-- Task progress -->
            <div class="task-progress">
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  :style="{
                    width: `${task.progress}%`,
                    backgroundColor: getProgressColor(task.progress),
                  }"
                />
              </div>
              <span class="progress-text">{{ task.progress }}%</span>
            </div>

            <!-- Task assignee -->
            <div v-if="getAssignedAgent(task)" class="task-assignee" @click.stop="emit('agentClick', getAssignedAgent(task)!.agentId)">
              <span class="assignee-icon">🤖</span>
              <span class="assignee-name">{{ getAssignedAgent(task)?.agentName }}</span>
            </div>

            <!-- Task tags -->
            <div v-if="task.tags.length > 0" class="task-tags">
              <span v-for="tag in task.tags.slice(0, 3)" :key="tag" class="task-tag">
                {{ tag }}
              </span>
            </div>

            <!-- Task time -->
            <div class="task-time">
              <span v-if="task.estimatedTime" class="time-label">
                Est: {{ task.estimatedTime }}min
              </span>
              <span v-if="task.actualTime" class="time-label">
                Actual: {{ task.actualTime }}min
              </span>
            </div>
          </div>

          <!-- Empty column message -->
          <div v-if="column.tasks.length === 0" class="empty-column">
            <div class="empty-icon">📭</div>
            <div class="empty-text">No tasks</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Kanban board - Agent view -->
    <div v-if="viewMode === 'agent'" class="kanban-board agent-view">
      <div
        v-for="agent in agentsWithTasks"
        :key="agent.agentId"
        class="kanban-agent-column"
        @click="emit('agentClick', agent.agentId)"
      >
        <!-- Agent header -->
        <div class="agent-header">
          <span class="agent-icon">🤖</span>
          <div class="agent-info">
            <div class="agent-name">{{ agent.agentName }}</div>
            <div class="agent-load">
              <span class="load-current">{{ agent.currentLoad }}</span>
              <span class="load-divider">/</span>
              <span class="load-max">{{ agent.capacity }}</span>
            </div>
          </div>
        </div>

        <!-- Agent tasks -->
        <div class="agent-content">
          <div
            v-for="task in agent.tasks"
            :key="task.id"
            class="kanban-task-card"
            :draggable="enableDrag"
            @dragstart="onDragStart($event, task.id, agent.agentId)"
            @click="emit('taskClick', task.taskId)"
          >
            <!-- Same task card content as above -->
            <div class="task-header">
              <span class="task-id">{{ task.taskId }}</span>
              <span class="priority-badge" :style="{ backgroundColor: getPriorityColor(task.priority) }">
                {{ getPriorityIcon(task.priority) }}
              </span>
            </div>

            <div class="task-title">{{ task.title }}</div>

            <div class="task-progress">
              <div class="progress-bar">
                <div
                  class="progress-fill"
                  :style="{
                    width: `${task.progress}%`,
                    backgroundColor: getProgressColor(task.progress),
                  }"
                />
              </div>
              <span class="progress-text">{{ task.progress }}%</span>
            </div>

            <div v-if="task.tags.length > 0" class="task-tags">
              <span v-for="tag in task.tags.slice(0, 3)" :key="tag" class="task-tag">
                {{ tag }}
              </span>
            </div>
          </div>

          <!-- Empty agent message -->
          <div v-if="agent.tasks.length === 0" class="empty-column">
            <div class="empty-icon">🤖</div>
            <div class="empty-text">No tasks assigned</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.collaboration-kanban {
  width: 100%;
  height: 100%;
  background: #F9FAFB;
  padding: 16px;
}

.kanban-header {
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

.view-mode-toggle {
  display: flex;
  gap: 8px;
}

.mode-button {
  padding: 8px 16px;
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 6px;
  color: #6B7280;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-button:hover {
  background: #F9FAFB;
  border-color: #D1D5DB;
}

.mode-button.active {
  background: #3B82F6;
  border-color: #3B82F6;
  color: white;
}

.kanban-board {
  display: flex;
  gap: 16px;
  overflow-x: auto;
}

.column-view .kanban-column {
  min-width: 280px;
  max-width: 320px;
  flex-shrink: 0;
}

.agent-view .kanban-agent-column {
  min-width: 280px;
  max-width: 320px;
  flex-shrink: 0;
}

.kanban-column,
.kanban-agent-column {
  background: white;
  border-radius: 8px;
  border: 1px solid #E5E7EB;
  overflow: hidden;
}

.column-header,
.agent-header {
  padding: 12px 16px;
  color: white;
  font-weight: 600;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.agent-header {
  background: #3B82F6;
  padding: 12px;
}

.agent-icon {
  font-size: 24px;
}

.agent-info {
  flex: 1;
  margin-left: 12px;
}

.agent-name {
  font-size: 14px;
  margin-bottom: 4px;
}

.agent-load {
  font-size: 12px;
  display: flex;
  gap: 4px;
}

.load-divider {
  opacity: 0.5;
}

.column-count {
  padding: 4px 12px;
  background: rgba(255, 255, 255, 0.3);
  border-radius: 12px;
  font-size: 12px;
}

.column-content,
.agent-content {
  padding: 12px;
  min-height: 200px;
  max-height: 600px;
  overflow-y: auto;
}

.kanban-task-card {
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.kanban-task-card:hover {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.kanban-task-card[draggable="true"] {
  cursor: grab;
}

.kanban-task-card:active {
  cursor: grabbing;
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.task-id {
  font-size: 11px;
  color: #3B82F6;
  font-weight: 600;
}

.priority-badge {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}

.task-title {
  font-size: 14px;
  color: #1F2937;
  margin-bottom: 8px;
  line-height: 1.4;
}

.task-progress {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.progress-bar {
  flex: 1;
  height: 4px;
  background: #E5E7EB;
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 11px;
  color: #6B7280;
  font-weight: 600;
}

.task-assignee {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: #EBF5FF;
  border-radius: 4px;
  margin-bottom: 8px;
  cursor: pointer;
}

.assignee-icon {
  font-size: 14px;
}

.assignee-name {
  font-size: 12px;
  color: #3B82F6;
}

.task-tags {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}

.task-tag {
  padding: 2px 8px;
  background: #F3F4F6;
  border-radius: 4px;
  font-size: 10px;
  color: #4B5563;
}

.task-time {
  display: flex;
  gap: 8px;
  font-size: 11px;
  color: #9CA3AF;
}

.empty-column {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: #9CA3AF;
}

.empty-icon {
  font-size: 32px;
  margin-bottom: 8px;
}

.empty-text {
  font-size: 13px;
}
</style>