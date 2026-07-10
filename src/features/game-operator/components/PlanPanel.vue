<script setup lang="ts">
import { computed } from 'vue'
import type { OperatorTask, OperatorEvent } from '@/types/operator'

const props = defineProps<{
  task: OperatorTask
  events: OperatorEvent[]
}>()

interface PlanStepView {
  id: number
  text: string
  status: 'done' | 'running' | 'pending' | 'failed'
}

const steps = computed<PlanStepView[]>(() => {
  const planReadyEvent = props.events.find(event => event.type === 'plan_ready')
  const rawSteps = planReadyEvent?.payload?.steps

  if (Array.isArray(rawSteps)) {
    return rawSteps.map((step: any, index: number) => {
      const id = Number(step.id ?? index + 1)
      return {
        id,
        text: step.description || step.text || `Step ${index + 1}`,
        status: getStepStatus(id),
      }
    })
  }

  const defaultSteps: PlanStepView[] = [
    { id: 1, text: 'Analyze project structure', status: 'done' },
    { id: 2, text: 'Find player controller', status: 'done' },
    { id: 3, text: 'Generate implementation plan', status: 'running' },
    { id: 4, text: 'Wait for user approval', status: 'pending' },
    { id: 5, text: 'Apply changes', status: 'pending' },
  ]

  if (props.task.status === 'waiting_approval') {
    defaultSteps[2].status = 'done'
    defaultSteps[3].status = 'running'
  } else if (props.task.status === 'running') {
    defaultSteps[2].status = 'done'
    defaultSteps[3].status = 'done'
    defaultSteps[4].status = 'running'
  } else if (props.task.status === 'paused') {
    defaultSteps[2].status = 'done'
    defaultSteps[3].status = 'done'
    defaultSteps[4].status = 'pending'
  } else if (props.task.status === 'completed') {
    defaultSteps.forEach(step => {
      step.status = 'done'
    })
  } else if (props.task.status === 'failed' || props.task.status === 'cancelled') {
    defaultSteps[4].status = 'failed'
  }

  return defaultSteps
})

function getStepStatus(stepId: number): PlanStepView['status'] {
  const startedEvent = props.events.find(event =>
    event.type === 'tool_call_started' && event.payload?.step_id === stepId
  )
  const succeededEvent = props.events.find(event =>
    event.type === 'tool_call_succeeded' && event.payload?.step_id === stepId
  )
  const failedEvent = props.events.find(event =>
    event.type === 'tool_call_failed' && event.payload?.step_id === stepId
  )

  if (failedEvent) return 'failed'
  if (succeededEvent) return 'done'
  if (startedEvent) return 'running'
  return 'pending'
}

function statusLabel(status: PlanStepView['status']): string {
  if (status === 'done') return 'Done'
  if (status === 'running') return 'Now'
  if (status === 'failed') return 'Failed'
  return 'Next'
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
          {{ statusLabel(step.status) }}
        </div>
        <div class="step-text">{{ step.text }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.plan-panel {
  height: auto;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.plan-panel h3 {
  margin-bottom: 1rem;
  font-size: 1.1rem;
}

.plan-goal {
  min-width: 0;
  padding: 0.75rem;
  background: var(--bg-main);
  border-radius: 4px;
  margin-bottom: 1rem;
  overflow-wrap: anywhere;
}

.plan-steps {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.plan-step {
  display: flex;
  align-items: flex-start;
  min-width: 0;
  gap: 0.75rem;
  padding: 0.5rem;
  background: var(--bg-main);
  border-radius: 4px;
}

.plan-step.status-done {
  opacity: 0.65;
}

.plan-step.status-running {
  background: #EFF6FF;
  border: 1px solid #3B82F6;
}

.plan-step.status-failed {
  background: #FEF2F2;
  border: 1px solid #EF4444;
}

.step-indicator {
  width: 52px;
  min-width: 52px;
  min-height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  background: var(--bg-hover);
  font-size: 0.75rem;
  font-weight: 600;
}

.status-done .step-indicator {
  background: #10B981;
  color: white;
}

.status-running .step-indicator {
  background: #3B82F6;
  color: white;
}

.status-failed .step-indicator {
  background: #EF4444;
  color: white;
}

.step-text {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}

</style>
