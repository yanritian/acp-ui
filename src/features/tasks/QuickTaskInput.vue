<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from '@/locales'
import { trackBehavior } from '@/lib/self-improvement'

const props = defineProps<{
  selectedAgent: string | null
  agents: string[]
}>()

const emit = defineEmits<{
  (e: 'submit', task: string): void
  (e: 'agent-change', agentName: string): void
}>()

const { t } = useI18n()

// State
const taskInput = ref('')
const localSelectedAgent = ref(props.selectedAgent || props.agents[0] || '')
const submitting = ref(false)

// Quick task suggestions
const quickTasks = [
  { label: '列出当前目录', task: '列出当前目录的文件' },
  { label: '生成 README', task: '为当前项目生成 README.md' },
  { label: '分析代码结构', task: '分析当前项目的代码结构' },
]

// Computed
const canSubmit = computed(() =>
  taskInput.value.trim().length > 0 && localSelectedAgent.value
)

function handleAgentChange(event: Event) {
  const target = event.target as HTMLSelectElement
  localSelectedAgent.value = target.value
  emit('agent-change', target.value)
}

function handleSubmit() {
  if (!canSubmit.value) return

  submitting.value = true
  emit('submit', taskInput.value.trim())

  // Clear input after submit
  setTimeout(() => {
    taskInput.value = ''
    submitting.value = false
  }, 300)
}

function handleQuickTask(task: string) {
  taskInput.value = task
  handleSubmit()
  trackBehavior('dashboard-quick-task-used', { task })
}
</script>

<template>
  <div class="quick-task-input">
    <!-- Agent Selector -->
    <div class="agent-selector">
      <label>{{ t('dashboard.selectAgent') }}</label>
      <select
        v-model="localSelectedAgent"
        @change="handleAgentChange"
        class="agent-select"
      >
        <option v-if="agents.length === 0" value="" disabled>
          {{ t('dashboard.noAgentsAvailable') }}
        </option>
        <option v-for="agent in agents" :key="agent" :value="agent">
          {{ agent }}
        </option>
      </select>
    </div>

    <!-- Task Input -->
    <div class="task-input-area">
      <label>{{ t('dashboard.taskDescription') }}</label>
      <textarea
        v-model="taskInput"
        class="task-textarea"
        rows="3"
        :placeholder="t('dashboard.taskPlaceholder')"
        @keydown.ctrl.enter="handleSubmit"
      ></textarea>
    </div>

    <!-- Quick Tasks -->
    <div class="quick-tasks">
      <span class="quick-label">{{ t('dashboard.quickSuggestions') }}</span>
      <div class="quick-buttons">
        <button
          v-for="qt in quickTasks"
          :key="qt.label"
          class="quick-btn"
          @click="handleQuickTask(qt.task)"
        >
          {{ qt.label }}
        </button>
      </div>
    </div>

    <!-- Submit Button -->
    <div class="submit-area">
      <button
        class="submit-btn"
        :disabled="!canSubmit || submitting"
        @click="handleSubmit"
      >
        {{ submitting ? t('dashboard.submitting') : t('dashboard.executeTask') }}
      </button>
      <span class="hint">{{ t('dashboard.ctrlEnterHint') }}</span>
    </div>
  </div>
</template>

<style scoped>
.quick-task-input {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.agent-selector,
.task-input-area {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary, #666);
}

.agent-select {
  padding: 12px 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  background: var(--bg-surface, #fff);
  font-size: 14px;
  cursor: pointer;
}

.agent-select:focus {
  border-color: var(--bg-primary, #0066cc);
  outline: none;
}

.task-textarea {
  padding: 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 8px;
  font-size: 14px;
  font-family: inherit;
  resize: vertical;
  min-height: 80px;
}

.task-textarea:focus {
  border-color: var(--bg-primary, #0066cc);
  outline: none;
}

.task-textarea::placeholder {
  color: var(--text-muted, #999);
}

.quick-tasks {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.quick-label {
  font-size: 12px;
  color: var(--text-muted, #999);
}

.quick-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.quick-btn {
  padding: 8px 16px;
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  background: transparent;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.quick-btn:hover {
  background: var(--bg-hover, #f0f0f0);
  border-color: var(--bg-primary, #0066cc);
}

.submit-area {
  display: flex;
  align-items: center;
  gap: 12px;
}

.submit-btn {
  padding: 12px 32px;
  background: var(--bg-primary, #0066cc);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease;
}

.submit-btn:hover:not(:disabled) {
  background: var(--bg-primary-hover, #0052a3);
}

.submit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.hint {
  font-size: 12px;
  color: var(--text-muted, #999);
}
</style>