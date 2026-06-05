<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from '@/locales'
import { useSwarmStore } from '@/stores/swarm'
import type { SwarmTopology, ConsensusStrategy, AgentRole } from '@/lib/plugin-system'

const { t } = useI18n()
const store = useSwarmStore()

const newTaskDesc = ref('')
const newTopology = ref<SwarmTopology>('hierarchical')
const newConsensus = ref<ConsensusStrategy>('first_wins')
const submitting = ref(false)
const selectedAgent = ref<string | null>(null)

const topologies: SwarmTopology[] = ['hierarchical', 'mesh', 'pipeline', 'star', 'adaptive']
const consensusOptions: ConsensusStrategy[] = ['first_wins', 'majority', 'best_score', 'merge', 'adversarial']

const roleBadge: Record<AgentRole, string> = {
  orchestrator: '#6366f1', executor: '#10b981', specialist: '#f59e0b',
  reviewer: '#ec4899', aggregator: '#06b6d4',
}
const statusColor: Record<string, string> = {
  idle: '#94a3b8', assigned: '#3b82f6', running: '#10b981',
  waiting_sync: '#f59e0b', completed: '#64748b', failed: '#ef4444', evicted: '#ef4444',
}
const taskStatusColor: Record<string, string> = {
  pending: '#94a3b8', in_progress: '#3b82f6', waiting_consensus: '#f59e0b',
  completed: '#10b981', failed: '#ef4444', cancelled: '#64748b',
}

const health = computed(() => store.health)
const totalAgents = computed(() => store.agents.length)
const activeCount = computed(() => store.activeAgents.length)
const runningTaskCount = computed(() => store.runningTasks.length)

function tokenUsagePercent(used: number, budget?: number): number {
  if (!budget) return used > 0 ? 60 : 0
  return Math.min(100, Math.round((used / budget) * 100))
}

async function submitTask() {
  if (!newTaskDesc.value.trim()) return
  submitting.value = true
  try {
    await store.createTask(newTaskDesc.value, newTopology.value, newConsensus.value)
    newTaskDesc.value = ''
  } catch (e) { console.error('Task creation failed:', e) }
  finally { submitting.value = false }
}

async function cancelTask(id: string) {
  await store.cancelTask(id)
}

let pollId: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  await Promise.all([store.loadAgents(), store.loadHealth()])
  pollId = setInterval(() => { store.loadAgents(); store.loadHealth() }, 3000)
})

onUnmounted(() => { if (pollId) clearInterval(pollId) })
</script>

<template>
  <div class="swarm-dash">
    <header class="sd-header">
      <h1 class="sd-title">{{ t('swarm.title') }}</h1>
      <div class="sd-health" v-if="health">
        <span class="health-chip">{{ health.total_agents }} {{ t('swarm.agents') }}</span>
        <span class="health-chip active">{{ health.active_agents }} {{ t('swarm.active') }}</span>
        <span class="health-chip running">{{ health.running_tasks }} {{ t('swarm.tasksRunning') }}</span>
      </div>
    </header>

    <div v-if="store.loading" class="sd-loading">{{ t('common.loading') }}...</div>
    <div v-else-if="store.error" class="sd-error">{{ store.error }}</div>

    <div v-else class="sd-body">
      <!-- Left: Agent List -->
      <section class="sd-panel">
        <h2 class="panel-title">{{ t('swarm.agentList') }}</h2>
        <div v-if="!store.agents.length" class="empty-text">{{ t('swarm.noAgents') }}</div>
        <div class="agent-scroll">
          <div
            v-for="agent in store.agents" :key="agent.id"
            class="agent-row" :class="{ selected: selectedAgent === agent.id }"
            @click="selectedAgent = agent.id === selectedAgent ? null : agent.id"
          >
            <div class="agent-top">
              <span class="agent-name">{{ agent.name }}</span>
              <span class="role-badge" :style="{ background: roleBadge[agent.role] || '#64748b' }">{{ agent.role }}</span>
            </div>
            <div class="agent-mid">
              <span class="status-pill" :style="{ color: statusColor[agent.status] || '#94a3b8' }">
                <span class="s-dot" :style="{ background: statusColor[agent.status] || '#94a3b8' }"></span>
                {{ agent.status }}
              </span>
              <span class="agent-task" v-if="agent.current_task">{{ agent.current_task }}</span>
            </div>
            <div class="token-bar-wrap">
              <div class="token-bar">
                <div class="token-fill" :style="{ width: tokenUsagePercent(agent.tokens_used, agent.token_budget) + '%' }"></div>
              </div>
              <span class="token-label">{{ agent.tokens_used.toLocaleString() }}{{ agent.token_budget ? ' / ' + agent.token_budget.toLocaleString() : '' }} tokens</span>
            </div>
          </div>
        </div>
      </section>

      <!-- Right: Tasks -->
      <section class="sd-panel">
        <h2 class="panel-title">{{ t('swarm.taskList') }}</h2>
        <div v-if="!store.tasks.length" class="empty-text">{{ t('swarm.noTasks') }}</div>
        <div class="task-scroll">
          <div v-for="task in store.tasks" :key="task.id" class="task-card">
            <div class="task-top">
              <span class="task-desc">{{ task.description }}</span>
              <span class="task-status-pill" :style="{ background: (taskStatusColor[task.status] || '#94a3b8') + '22', color: taskStatusColor[task.status] || '#94a3b8' }">
                {{ task.status }}
              </span>
            </div>
            <div class="task-meta">
              <span class="meta-chip">{{ task.topology }}</span>
              <span class="meta-chip">{{ task.consensus }}</span>
              <span class="meta-chip agents">{{ task.assigned_agents.length }} agents</span>
            </div>
            <div class="task-progress-bar">
              <div class="task-progress-fill" :style="{
                width: (task.status === 'completed' ? 100 : task.status === 'failed' || task.status === 'cancelled' ? 0 : 50) + '%'
              }"></div>
            </div>
            <div class="task-actions" v-if="task.status === 'in_progress' || task.status === 'pending'">
              <button class="btn btn-sm btn-danger" @click="cancelTask(task.id)">{{ t('swarm.cancel') }}</button>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- Bottom: Create Task Form -->
    <footer class="sd-footer">
      <h3 class="footer-title">{{ t('swarm.createTask') }}</h3>
      <div class="task-form">
        <input
          class="form-input flex-grow"
          v-model="newTaskDesc"
          :placeholder="t('swarm.taskDescPlaceholder')"
          @keydown.enter="submitTask"
        />
        <select class="form-select" v-model="newTopology">
          <option v-for="topo in topologies" :key="topo" :value="topo">{{ topo }}</option>
        </select>
        <select class="form-select" v-model="newConsensus">
          <option v-for="cs in consensusOptions" :key="cs" :value="cs">{{ cs }}</option>
        </select>
        <button class="btn btn-primary" :disabled="submitting || !newTaskDesc.trim()" @click="submitTask">
          {{ submitting ? t('common.submitting') : t('swarm.submit') }}
        </button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.swarm-dash {
  --bg-main: #0f172a;
  --bg-surface: #1e293b;
  --bg-subtle: #334155;
  --text-primary: #f1f5f9;
  --text-secondary: #94a3b8;
  --text-muted: #64748b;
  --accent: #6366f1;
  --accent-hover: #4f46e5;
  --success: #10b981;
  --error: #ef4444;
  --border: #334155;
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-full: 9999px;

  display: flex; flex-direction: column; height: 100%;
  background: var(--bg-main); color: var(--text-primary);
  font-family: system-ui, -apple-system, sans-serif;
}

.sd-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px; border-bottom: 1px solid var(--border);
}
.sd-title { font-size: 20px; font-weight: 600; }
.sd-title::before { content: '\uD83D\uDC1D '; }
.sd-health { display: flex; gap: 8px; }
.health-chip {
  padding: 4px 12px; border-radius: var(--radius-full);
  background: var(--bg-surface); font-size: 12px; color: var(--text-secondary);
}
.health-chip.active { color: var(--accent); }
.health-chip.running { color: var(--success); }

.sd-loading, .sd-error { text-align: center; padding: 48px; color: var(--text-muted); }
.sd-error { color: var(--error); }

.sd-body { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; flex: 1; padding: 16px 20px; overflow: hidden; }

.sd-panel {
  background: var(--bg-surface); border-radius: var(--radius-lg);
  border: 1px solid var(--border); display: flex; flex-direction: column; overflow: hidden;
}
.panel-title { padding: 12px 16px; font-size: 14px; font-weight: 600; border-bottom: 1px solid var(--border); }

.agent-scroll, .task-scroll { flex: 1; overflow-y: auto; padding: 8px; }

.agent-row {
  padding: 10px 12px; border-radius: var(--radius-md); cursor: pointer;
  border: 1px solid transparent; margin-bottom: 6px; transition: all 0.15s;
}
.agent-row:hover { background: var(--bg-subtle); }
.agent-row.selected { border-color: var(--accent); background: rgba(99, 102, 241, 0.1); }

.agent-top { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
.agent-name { font-size: 13px; font-weight: 500; }
.role-badge {
  padding: 2px 8px; border-radius: var(--radius-full);
  font-size: 10px; font-weight: 600; color: white; text-transform: uppercase; letter-spacing: 0.3px;
}

.agent-mid { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; }
.status-pill { display: flex; align-items: center; gap: 4px; font-size: 11px; }
.s-dot { width: 6px; height: 6px; border-radius: 50%; }
.agent-task { font-size: 11px; color: var(--text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.token-bar-wrap { display: flex; align-items: center; gap: 8px; }
.token-bar { flex: 1; height: 4px; background: var(--bg-main); border-radius: 2px; overflow: hidden; }
.token-fill { height: 100%; background: var(--accent); border-radius: 2px; transition: width 0.3s; }
.token-label { font-size: 10px; color: var(--text-muted); white-space: nowrap; }

.task-card {
  padding: 12px; border-radius: var(--radius-md);
  border: 1px solid var(--border); margin-bottom: 8px;
}
.task-top { display: flex; justify-content: space-between; align-items: flex-start; gap: 8px; margin-bottom: 8px; }
.task-desc { font-size: 13px; font-weight: 500; flex: 1; }
.task-status-pill { padding: 2px 8px; border-radius: var(--radius-full); font-size: 11px; font-weight: 500; white-space: nowrap; }

.task-meta { display: flex; gap: 6px; margin-bottom: 8px; flex-wrap: wrap; }
.meta-chip {
  padding: 2px 8px; background: var(--bg-subtle); border-radius: var(--radius-sm);
  font-size: 11px; color: var(--text-secondary);
}
.meta-chip.agents { color: var(--accent); }

.task-progress-bar { height: 4px; background: var(--bg-main); border-radius: 2px; overflow: hidden; }
.task-progress-fill { height: 100%; background: var(--success); border-radius: 2px; transition: width 0.3s; }

.task-actions { margin-top: 8px; display: flex; justify-content: flex-end; }

.sd-footer {
  padding: 12px 20px; border-top: 1px solid var(--border); background: var(--bg-surface);
}
.footer-title { font-size: 13px; font-weight: 600; margin-bottom: 8px; }
.task-form { display: flex; gap: 8px; align-items: center; }

.form-input {
  padding: 6px 12px; background: var(--bg-main); border: 1px solid var(--border);
  border-radius: var(--radius-md); color: var(--text-primary); font-size: 13px;
}
.form-input:focus { border-color: var(--accent); outline: none; }
.form-input::placeholder { color: var(--text-muted); }
.flex-grow { flex: 1; }

.form-select {
  padding: 6px 10px; background: var(--bg-main); border: 1px solid var(--border);
  border-radius: var(--radius-md); color: var(--text-primary); font-size: 12px;
}
.form-select:focus { border-color: var(--accent); outline: none; }

.btn {
  display: inline-flex; align-items: center; justify-content: center;
  padding: 6px 14px; font-size: 13px; font-weight: 500;
  border-radius: var(--radius-md); border: none; cursor: pointer; transition: all 0.15s;
}
.btn-primary { background: var(--accent); color: white; }
.btn-primary:hover { background: var(--accent-hover); }
.btn-danger { background: var(--error); color: white; }
.btn-sm { padding: 4px 10px; font-size: 12px; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }

.empty-text { text-align: center; color: var(--text-muted); padding: 32px 16px; font-size: 13px; }
</style>
