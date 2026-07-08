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
    e.type === 'project_analyzed'
  )
})

// Extract steps from plan events (simplified - real implementation would parse payload)
const steps = computed(() => {
  // TODO: Parse actual plan from events
  return [
    { id: 1, text: 'Analyze project structure', status: 'done' },
    { id: 2, text: 'Find player controller', status: 'done' },
    { id: 3, text: 'Generate implementation plan', status: 'running' },
    { id: 4, text: 'Wait for user approval', status: 'pending' },
    { id: 5, text: 'Apply changes', status: 'pending' },
  ]
})
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
