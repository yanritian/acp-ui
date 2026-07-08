<script setup lang="ts">
import type { OperatorTask, OperatorEvent } from '@/types/operator'
import { computed } from 'vue'

const props = defineProps<{
  task: OperatorTask
  events: OperatorEvent[]
}>()

// Extract plan-related events
const planEvents = computed(() => {
  return props.events.filter(e =>
    e.type === 'plan_started' ||
    e.type === 'plan_ready' ||
    e.type === 'project_analyzed' ||
    e.type === 'tool_call_started' ||
    e.type === 'tool_call_succeeded' ||
    e.type === 'tool_call_failed'
  )
})

// Parse steps from events payload or generate default steps based on task status
const steps = computed(() => {
  // Try to extract steps from plan_ready event payload
  const planReadyEvent = props.events.find(e => e.type === 'plan_ready')
  if (planReadyEvent?.payload?.steps) {
    return planReadyEvent.payload.steps.map((s: any, idx: number) => ({
      id: s.id || idx + 1,
      text: s.description || s.text || `Step ${idx + 1}`,
      status: getStepStatus(s.id || idx + 1)
    }))
  }

  // Generate default steps based on task status
  const taskStatus = props.task.status
  const defaultSteps = [
    { id: 1, text: 'Analyze project structure', status: 'done' },
    { id: 2, text: 'Find player controller', status: 'done' },
    { id: 3, text: 'Generate implementation plan', status: 'running' },
    { id: 4, text: 'Wait for user approval', status: 'pending' },
    { id: 5, text: 'Apply changes', status: 'pending' },
  ]

  // Update step status based on task status
  if (taskStatus === 'waiting_approval') {
    defaultSteps[2].status = 'done'
    defaultSteps[3].status = 'running'
  } else if (taskStatus === 'running') {
    defaultSteps[2].status = 'done'
    defaultSteps[3].status = 'done'
    defaultSteps[4].status = 'running'
  } else if (taskStatus === 'completed') {
    defaultSteps.forEach(s => s.status = 'done')
  } else if (taskStatus === 'failed') {
    defaultSteps[4].status = 'failed'
  }

  return defaultSteps
})

// Helper to get step status from events
function getStepStatus(stepId: number): string {
  const startedEvent = props.events.find(e =>
    e.type === 'tool_call_started' && e.payload?.step_id === stepId
  )
  const succeededEvent = props.events.find(e =>
    e.type === 'tool_call_succeeded' && e.payload?.step_id === stepId
  )
  const failedEvent = props.events.find(e =>
    e.type === 'tool_call_failed' && e.payload?.step_id === stepId
  )

  if (failedEvent) return 'failed'
  if (succeededEvent) return 'done'
  if (startedEvent) return 'running'
  return 'pending'
}
</script>

<template>
  <div class="plan-panel">
    <h3>Execution Plan</h3>
    <div class="plan-goal">
      <strong>Goal:</strong> {{ task.goal }}
    </div>
    <div class="plan-steps">
      <div
        v-for="step in steps"
        :key="step.id"
        class="plan-step"
        :class="[`status-${step.status}`]"
      >
        <div class="step-indicator">
          <span v-if="step.status === 'done'">✓</span>
          <span v-else-if="step.status === 'running'">⏳</span>
          <span v-else>○</span>
        </div>
        <div class="step-text">{{ step.text }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.plan-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.plan-panel h3 {
  margin-bottom: 1rem;
  font-size: 1.1rem;
}

.plan-goal {
  padding: 0.75rem;
  background: var(--bg-main);
  border-radius: 4px;
  margin-bottom: 1rem;
}

.plan-steps {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.plan-step {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem;
  background: var(--bg-main);
  border-radius: 4px;
}

.plan-step.status-done {
  opacity: 0.6;
}

.plan-step.status-running {
  background: #EFF6FF;
  border: 1px solid #3B82F6;
}

.step-indicator {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--bg-hover);
  font-size: 0.9rem;
}

.status-done .step-indicator {
  background: #10B981;
  color: white;
}

.status-running .step-indicator {
  background: #3B82F6;
  color: white;
}

.step-text {
  flex: 1;
}
</style>
