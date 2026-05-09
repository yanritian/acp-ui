<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMemoryStore, type MemoryScope } from '../stores/memory'
import { useConfigStore } from '../stores/config'

const memoryStore = useMemoryStore()
const configStore = useConfigStore()

const newMemoryContent = ref('')
const newMemoryTags = ref('')
const newMemoryScope = ref<MemoryScope>('global')
const showAddForm = ref(false)
const selectedScope = ref<MemoryScope | 'all'>('all')

const scopes: { value: MemoryScope | 'all', label: string }[] = [
  { value: 'all', label: '全部' },
  { value: 'global', label: '全局' },
  { value: 'agent', label: 'Agent' },
  { value: 'session', label: '会话' },
  { value: 'task', label: '任务' },
]

const agentEntries = computed(() => Object.entries(configStore.config.agents))
const records = computed(() => {
  let filtered = memoryStore.searchResults.length > 0 ? memoryStore.searchResults : memoryStore.memories
  if (selectedScope.value !== 'all') {
    filtered = filtered.filter(m => m.scope === selectedScope.value)
  }
  return filtered
})

async function loadMemories(agentId: string | null) {
  await memoryStore.loadAgentMemories(agentId)
}

async function handleSearch() {
  if (memoryStore.searchKeyword.trim()) {
    await memoryStore.searchMemories(memoryStore.searchKeyword)
  } else {
    await loadMemories(null)
  }
}

async function handleAddMemory() {
  const content = newMemoryContent.value.trim()
  if (!content) return

  const tags = newMemoryTags.value.trim() || null
  await memoryStore.saveMemory(
    content,
    tags,
    newMemoryScope.value,
    'fact',  // memoryType
    memoryStore.selectedAgentId,
    null,    // sessionId
    null,    // taskId
    0.5,     // importance
    null     // expiresAt
  )
  newMemoryContent.value = ''
  newMemoryTags.value = ''
  newMemoryScope.value = 'global'
  showAddForm.value = false
}

function formatTime(s: string): string {
  return new Date(s).toLocaleString()
}

function getTags(tags: string | null): string[] {
  if (!tags) return []
  try {
    return JSON.parse(tags)
  } catch {
    return tags.split(',').map(t => t.trim())
  }
}

function getScopeLabel(scope: MemoryScope): string {
  const labels: Record<MemoryScope, string> = {
    global: '全局',
    agent: 'Agent',
    session: '会话',
    task: '任务'
  }
  return labels[scope]
}

onMounted(() => {
  loadMemories(null)
})
</script>

<template>
  <div class="memory-view">
    <!-- Search Bar -->
    <div class="search-bar">
      <input
        v-model="memoryStore.searchKeyword"
        type="text"
        placeholder="搜索记忆..."
        @keyup.enter="handleSearch"
      />
      <button @click="handleSearch">搜索</button>
      <button @click="loadMemories(null)">全部记忆</button>
    </div>

    <!-- Agent Filter -->
    <div class="agent-filter">
      <span class="filter-label">按 Agent 过滤:</span>
      <button
        :class="['agent-btn', { active: !memoryStore.selectedAgentId }]"
        @click="loadMemories(null)"
      >
        全局
      </button>
      <button
        v-for="[name] in agentEntries"
        :key="name"
        :class="['agent-btn', { active: memoryStore.selectedAgentId === name }]"
        @click="loadMemories(name)"
      >
        {{ name }}
      </button>
    </div>

    <!-- Scope Filter -->
    <div class="scope-filter">
      <span class="filter-label">按作用域过滤:</span>
      <button
        v-for="s in scopes"
        :key="s.value"
        :class="['scope-btn', { active: selectedScope === s.value }]"
        @click="selectedScope = s.value"
      >
        {{ s.label }}
      </button>
    </div>

    <!-- Add Memory Button -->
    <div class="add-section">
      <button class="btn-add" @click="showAddForm = !showAddForm">
        {{ showAddForm ? '取消' : '+ 添加记忆' }}
      </button>
    </div>

    <!-- Add Memory Form -->
    <div v-if="showAddForm" class="add-form">
      <textarea
        v-model="newMemoryContent"
        class="memory-input"
        placeholder="记忆内容..."
        rows="3"
      ></textarea>
      <input
        v-model="newMemoryTags"
        class="memory-input"
        placeholder="标签 (逗号分隔, 如: 技术,架构)"
      />
      <div class="scope-select">
        <label>作用域:</label>
        <select v-model="newMemoryScope">
          <option value="global">全局</option>
          <option value="agent">Agent</option>
          <option value="session">会话</option>
          <option value="task">任务</option>
        </select>
      </div>
      <div class="form-actions">
        <button class="btn-submit" @click="handleAddMemory" :disabled="!newMemoryContent.trim()">
          保存
        </button>
      </div>
    </div>

    <!-- Error -->
    <div v-if="memoryStore.error" class="error-banner">
      <span>{{ memoryStore.error }}</span>
      <button @click="memoryStore.clearError">✕</button>
    </div>

    <!-- Loading -->
    <div v-if="memoryStore.loading" class="loading">加载中...</div>

    <!-- Memory List -->
    <div class="memory-list">
      <div v-if="records.length === 0 && !memoryStore.loading" class="empty-state">
        <p>暂无记忆</p>
        <p class="hint">添加记忆来记录重要的项目信息和决策</p>
      </div>

      <div
        v-for="memory in records"
        :key="memory.id"
        class="memory-card"
      >
        <div class="memory-header">
          <span class="memory-scope">{{ getScopeLabel(memory.scope) }}</span>
          <span class="memory-agent">{{ memory.agentId || '全局' }}</span>
          <span class="memory-time">{{ formatTime(memory.createdAt) }}</span>
          <button class="memory-delete" @click="memoryStore.deleteMemory(memory.id)">✕</button>
        </div>
        <div class="memory-content">{{ memory.content }}</div>
        <div v-if="memory.tags" class="memory-tags">
          <span v-for="tag in getTags(memory.tags)" :key="tag" class="tag">{{ tag }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.memory-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}

.search-bar {
  display: flex;
  gap: 8px;
}

.search-bar input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.search-bar button {
  padding: 8px 16px;
  border-radius: 4px;
  border: none;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.agent-filter {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-label {
  font-size: 13px;
  color: var(--text-muted);
}

.agent-btn {
  padding: 4px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 12px;
}

.agent-btn.active {
  background: #4a90d9;
  color: white;
  border-color: #4a90d9;
}

.scope-filter {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.scope-btn {
  padding: 4px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  font-size: 12px;
}

.scope-btn.active {
  background: #28a745;
  color: white;
  border-color: #28a745;
}

.add-section {
  display: flex;
  justify-content: flex-end;
}

.btn-add {
  padding: 8px 16px;
  border: 1px dashed #4a90d9;
  border-radius: 4px;
  background: transparent;
  color: #4a90d9;
  cursor: pointer;
}

.add-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 8px;
}

.memory-input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-family: inherit;
  font-size: 14px;
}

.scope-select {
  display: flex;
  align-items: center;
  gap: 8px;
}

.scope-select label {
  font-size: 13px;
  color: #666;
}

.scope-select select {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
}

.btn-submit {
  padding: 8px 20px;
  border: none;
  border-radius: 4px;
  background: #4a90d9;
  color: white;
  cursor: pointer;
}

.btn-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #fee;
  color: #c00;
  font-size: 13px;
}

.error-banner button {
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
}

.loading {
  text-align: center;
  padding: 24px;
  color: #666;
}

.memory-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-state {
  text-align: center;
  padding: 48px;
  color: #666;
}

.empty-state .hint {
  font-size: 13px;
  margin-top: 8px;
}

.memory-card {
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #ddd;
  background: white;
}

.memory-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 12px;
  color: #666;
}

.memory-agent {
  font-weight: 600;
  color: #4a90d9;
}

.memory-scope {
  padding: 2px 8px;
  background: #28a745;
  color: white;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
}

.memory-delete {
  padding: 2px 6px;
  border: none;
  background: transparent;
  color: #999;
  cursor: pointer;
  font-size: 14px;
}

.memory-delete:hover {
  color: #ef4444;
}

.memory-content {
  white-space: pre-wrap;
  word-break: break-word;
  margin-bottom: 8px;
}

.memory-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.tag {
  padding: 2px 8px;
  background: #e3f2fd;
  color: #1976d2;
  border-radius: 12px;
  font-size: 11px;
}
</style>
