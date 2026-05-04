<script setup lang="ts">
import { ref, computed } from 'vue'
import { useConfigStore } from '../../stores/config'

const configStore = useConfigStore()

const emit = defineEmits<{
  create: [agentName: string, cwd: string]
  cancel: []
}>()

const agents = computed(() => Object.keys(configStore.config.agents))
const selectedAgent = ref(agents.value[0] ?? '')
const cwd = ref('')
const cwdError = ref('')

const isAbsoluteCwd = computed(() => {
  const value = cwd.value.trim()
  if (!value) return false
  return value.startsWith('/') || /^[A-Za-z]:[\\/]/.test(value)
})

const isValid = computed(() => selectedAgent.value && isAbsoluteCwd.value)

function validateCwd() {
  const value = cwd.value.trim()
  if (!value) {
    cwdError.value = '请输入工作目录'
  } else if (!isAbsoluteCwd.value) {
    cwdError.value = `工作目录必须是绝对路径，当前值: ${value}`
  } else {
    cwdError.value = ''
  }
}

function handleCreate() {
  if (!isValid.value) {
    validateCwd()
    return
  }
  emit('create', selectedAgent.value, cwd.value.trim())
}
</script>

<template>
  <div class="new-session-overlay" @click.self="emit('cancel')">
    <div class="dialog">
      <h3>新建会话</h3>

      <div class="field">
        <label>Agent:</label>
        <select v-model="selectedAgent">
          <option v-for="name in agents" :key="name" :value="name">{{ name }}</option>
        </select>
        <p v-if="agents.length === 0" class="hint">请先在设置中添加 Agent</p>
      </div>

      <div class="field">
        <label>工作目录 (绝对路径):</label>
        <input
          v-model="cwd"
          type="text"
          placeholder="例如: D:\work\project 或 /home/user/project"
          @blur="validateCwd"
          @keyup.enter="handleCreate"
        />
        <p v-if="cwdError" class="error">{{ cwdError }}</p>
      </div>

      <div class="actions">
        <button class="btn-cancel" @click="emit('cancel')">取消</button>
        <button class="btn-create" :disabled="!isValid" @click="handleCreate">创建</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.new-session-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog {
  background: var(--bg-surface, #fff);
  border-radius: 12px;
  padding: 24px;
  min-width: 400px;
  max-width: 520px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.dialog h3 {
  margin: 0 0 20px;
  color: var(--text-primary);
}

.field {
  margin-bottom: 16px;
}

.field label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.field select,
.field input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color, #ddd);
  border-radius: 6px;
  background: var(--bg-main, #fff);
  color: var(--text-primary);
  font-size: 14px;
  box-sizing: border-box;
}

.field input:focus,
.field select:focus {
  border-color: var(--primary, #6366f1);
  outline: none;
}

.field .hint {
  font-size: 12px;
  color: var(--text-muted, #999);
  margin: 4px 0 0;
}

.field .error {
  font-size: 12px;
  color: #ef4444;
  margin: 4px 0 0;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
}

.btn-cancel,
.btn-create {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.btn-cancel {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color, #ddd);
}

.btn-create {
  background: var(--bg-primary, #6366f1);
  color: white;
}

.btn-create:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
