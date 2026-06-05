<script setup lang="ts">
import { computed, ref } from 'vue'
import type { PermissionWaiting, PermissionOption } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  permission: PermissionWaiting
}>()

const emit = defineEmits<{
  (e: 'response', optionId: string): void
}>()

// State
const selectedOption = ref<string | null>(null)
const isResponding = ref(false)

// Computed
const waitingDuration = computed(() => {
  const duration = Date.now() - props.permission.waitingSince
  if (duration < 1000) return `${duration}ms`
  return `${(duration / 1000).toFixed(1)}s`
})

const permissionIcon = computed(() => {
  const icons: Record<string, string> = {
    tool_use: '🔧',
    file_write: '📝',
    command_execute: '⚡',
    network_access: '🌐'
  }
  return icons[props.permission.type] || '⏸️'
})

const permissionColor = computed(() => {
  const colors: Record<string, string> = {
    tool_use: '#F59E0B',
    file_write: '#3B82F6',
    command_execute: '#EF4444',
    network_access: '#8B5CF6'
  }
  return colors[props.permission.type] || '#6B7280'
})

// Methods
function handleOptionClick(option: PermissionOption) {
  selectedOption.value = option.id
  isResponding.value = true

  // 延迟发送以显示反馈
  setTimeout(() => {
    emit('response', option.id)
    isResponding.value = false
  }, 200)
}
</script>

<template>
  <div class="permission-waiting">
    <!-- Header -->
    <div class="permission-header">
      <div class="permission-info">
        <span class="permission-icon">{{ permissionIcon }}</span>
        <span class="permission-type">{{ permission.type }}</span>
        <span class="waiting-badge">{{ t('agentProgress.waitingPermission') }}</span>
      </div>
      <div class="waiting-duration">{{ waitingDuration }}</div>
    </div>

    <!-- Description -->
    <div class="permission-description">
      {{ permission.description }}
    </div>

    <!-- Timeout warning -->
    <div v-if="permission.timeout" class="timeout-warning">
      ⏱️ {{ t('agentProgress.timeoutWarning', { seconds: Math.round((permission.timeout - Date.now()) / 1000) }) }}
    </div>

    <!-- Options -->
    <div class="permission-options">
      <div class="options-label">{{ t('agentProgress.selectAction') }}</div>
      <div class="options-list">
        <button
          v-for="option in permission.options"
          :key="option.id"
          :class="[
            'option-button',
            option.action,
            { selected: selectedOption === option.id, responding: isResponding }
          ]"
          :style="{ borderColor: permissionColor }"
          @click="handleOptionClick(option)"
        >
          <span class="option-label">{{ option.label }}</span>
          <span v-if="option.recommended" class="recommended-badge">{{ t('agentProgress.recommended') }}</span>
        </button>
      </div>
    </div>

    <!-- Responding indicator -->
    <div v-if="isResponding" class="responding-indicator">
      <div class="responding-spinner"></div>
      <span>{{ t('agentProgress.responding') }}</span>
    </div>
  </div>
</template>

<style scoped>
.permission-waiting {
  padding: 16px;
  background: rgba(107, 114, 128, 0.1);
  border: 1px solid rgba(107, 114, 128, 0.3);
  border-radius: 12px;
}

.permission-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(107, 114, 128, 0.2);
}

.permission-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.permission-icon {
  font-size: 20px;
}

.permission-type {
  font-size: 14px;
  font-weight: 600;
  color: #e0e0e0;
}

.waiting-badge {
  padding: 4px 8px;
  background: rgba(107, 114, 128, 0.2);
  border-radius: 6px;
  font-size: 11px;
  color: #6b7280;
}

.waiting-duration {
  font-size: 12px;
  color: #8b8b9b;
  font-family: 'Monaco', monospace;
}

.permission-description {
  padding: 12px;
  background: rgba(26, 26, 46, 0.5);
  border-radius: 8px;
  border: 1px solid #3a3a5a;
  font-size: 13px;
  color: #e0e0e0;
  margin-bottom: 12px;
}

.timeout-warning {
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  font-size: 12px;
  color: #ef4444;
  margin-bottom: 12px;
}

.permission-options {
  margin-bottom: 12px;
}

.options-label {
  font-size: 12px;
  color: #8b8b9b;
  margin-bottom: 8px;
}

.options-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.option-button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: rgba(26, 26, 46, 0.5);
  border: 2px solid;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.option-button:hover {
  background: rgba(58, 58, 90, 0.3);
}

.option-button.selected {
  background: rgba(58, 58, 90, 0.4);
}

.option-button.responding {
  opacity: 0.6;
  pointer-events: none;
}

.option-button.allow {
  border-color: #10b981;
}

.option-button.deny {
  border-color: #ef4444;
}

.option-button.allow_always {
  border-color: #3b82f6;
}

.option-button.deny_always {
  border-color: #9ca3af;
}

.option-label {
  font-size: 13px;
  color: #e0e0e0;
}

.recommended-badge {
  padding: 2px 8px;
  background: rgba(16, 185, 129, 0.2);
  border-radius: 6px;
  font-size: 10px;
  color: #10b981;
}

.responding-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: rgba(107, 114, 128, 0.1);
  border-radius: 6px;
}

.responding-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid #6b7280;
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.responding-indicator span {
  font-size: 11px;
  color: #6b7280;
}
</style>