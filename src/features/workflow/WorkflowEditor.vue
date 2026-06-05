<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from '@/locales'
import {
  WorkflowService,
  type WorkflowDefinition,
  type WorkflowStage,
  type WorkflowProgress,
  type StageStrategy,
  type FailurePolicy,
} from '@/lib/plugin-system'

const { t } = useI18n()

const workflows = ref<WorkflowDefinition[]>([])
const selectedId = ref<string | null>(null)
const progress = ref<WorkflowProgress | null>(null)
const expandedStage = ref<string | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const generating = ref(false)

// Create form state
const formName = ref('')
const formDesc = ref('')
const formMaxAgents = ref(3)
const formFailurePolicy = ref<FailurePolicy>('stop_all')
const formStages = ref<Array<{ name: string; strategy: StageStrategy; agents: string; dependsOn: string }>>([])
const submitting = ref(false)

const strategies: StageStrategy[] = ['parallel', 'sequential', 'map_reduce', 'competitive', 'adversarial_review']
const failurePolicies: FailurePolicy[] = ['stop_all', 'continue_others', 'retry_with_backoff', 'fallback_to_manual']

const statusColors: Record<string, string> = {
  draft: '#94a3b8', validating: '#3b82f6', ready: '#06b6d4', running: '#10b981',
  paused: '#f59e0b', completed: '#64748b', failed: '#ef4444', cancelled: '#64748b',
}
const strategyColors: Record<string, string> = {
  parallel: '#3b82f6', sequential: '#10b981', map_reduce: '#f59e0b',
  competitive: '#ec4899', adversarial_review: '#ef4444',
}

const selected = computed(() => workflows.value.find(w => w.id === selectedId.value) || null)

const progressPercent = computed(() => {
  if (!progress.value || !progress.value.total_stages) return 0
  return Math.round((progress.value.completed_stages / progress.value.total_stages) * 100)
})

const elapsedFormatted = computed(() => {
  if (!progress.value) return '--'
  const s = Math.floor(progress.value.elapsed_ms / 1000)
  if (s < 60) return `${s}s`
  const m = Math.floor(s / 60)
  return `${m}m ${s % 60}s`
})

async function loadWorkflows() {
  loading.value = true; error.value = null
  try { workflows.value = await WorkflowService.list() } catch (e) { error.value = String(e) }
  finally { loading.value = false }
}

async function selectWorkflow(id: string) {
  selectedId.value = id
  expandedStage.value = null
  try { progress.value = await WorkflowService.getProgress(id) } catch { progress.value = null }
}

watch(selectedId, (id) => { if (id) selectWorkflow(id) })

async function generateFromTask() {
  if (!formDesc.value.trim()) return
  generating.value = true
  try {
    const wf = await WorkflowService.generateFromTask(formDesc.value, [])
    workflows.value.unshift(wf)
    selectedId.value = wf.id
    formName.value = wf.name
    formDesc.value = ''
  } catch (e) { error.value = String(e) }
  finally { generating.value = false }
}

function addStage() {
  formStages.value.push({ name: '', strategy: 'sequential', agents: '', dependsOn: '' })
}

function removeStage(idx: number) {
  formStages.value.splice(idx, 1)
}

async function submitWorkflow() {
  if (!formName.value.trim() || !formStages.value.length) return
  submitting.value = true
  try {
    const stages: WorkflowStage[] = formStages.value.map((s, i) => ({
      id: `stage-${i}`,
      name: s.name,
      description: '',
      strategy: s.strategy,
      agents: s.agents.split(',').filter(Boolean).map(a => ({ agent_id: a.trim(), role: 'executor', prompt_template: '' })),
      depends_on: s.dependsOn.split(',').filter(Boolean).map(d => d.trim()),
      sync_points: [],
      status: 'draft' as const,
      results: [],
    }))
    const wf = await WorkflowService.create(formName.value, formDesc.value, stages, formMaxAgents.value)
    workflows.value.unshift(wf)
    selectedId.value = wf.id
    formName.value = ''; formDesc.value = ''; formStages.value = []
  } catch (e) { error.value = String(e) }
  finally { submitting.value = false }
}

function toggleStage(stageId: string) {
  expandedStage.value = expandedStage.value === stageId ? null : stageId
}

onMounted(() => { loadWorkflows() })
</script>

<template>
  <div class="wf-editor">
    <!-- Left: Workflow list -->
    <aside class="wf-sidebar">
      <h2 class="sb-title">{{ t('workflow.title') }}</h2>
      <div v-if="loading" class="sb-loading">{{ t('common.loading') }}...</div>
      <div v-else-if="!workflows.length" class="sb-empty">{{ t('workflow.noWorkflows') }}</div>
      <div v-else class="wf-list">
        <div
          v-for="wf in workflows" :key="wf.id"
          class="wf-item" :class="{ selected: selectedId === wf.id }"
          @click="selectWorkflow(wf.id)"
        >
          <span class="wf-status-dot" :style="{ background: statusColors[wf.status] || '#94a3b8' }"></span>
          <div class="wf-item-info">
            <span class="wf-item-name">{{ wf.name }}</span>
            <span class="wf-item-meta">{{ wf.stages.length }} stages &middot; {{ wf.status }}</span>
          </div>
        </div>
      </div>
    </aside>

    <!-- Center: Detail -->
    <main class="wf-center">
      <div v-if="!selected" class="center-empty">{{ t('workflow.selectWorkflow') }}</div>
      <template v-else>
        <div class="wf-detail-header">
          <div>
            <h2 class="wf-name">{{ selected.name }}</h2>
            <p class="wf-desc">{{ selected.description }}</p>
          </div>
          <span class="status-badge" :style="{ background: (statusColors[selected.status] || '#94a3b8') + '22', color: statusColors[selected.status] }">
            {{ selected.status }}
          </span>
        </div>

        <!-- Progress -->
        <div class="wf-progress" v-if="progress">
          <div class="progress-bar"><div class="progress-fill" :style="{ width: progressPercent + '%' }"></div></div>
          <div class="progress-stats">
            <span>{{ progress.completed_stages }}/{{ progress.total_stages }} {{ t('workflow.stages') }}</span>
            <span>{{ progress.tokens_used.toLocaleString() }} tokens</span>
            <span>{{ elapsedFormatted }}</span>
          </div>
        </div>

        <!-- Stage timeline -->
        <div class="stage-timeline">
          <div v-for="(stage, idx) in selected.stages" :key="stage.id" class="stage-node" @click="toggleStage(stage.id)">
            <div class="stage-connector">
              <span class="stage-dot" :style="{ background: statusColors[stage.status] || '#94a3b8' }"></span>
              <span v-if="idx < selected.stages.length - 1" class="stage-line"></span>
            </div>
            <div class="stage-content">
              <div class="stage-header">
                <span class="stage-name">{{ stage.name }}</span>
                <span class="strategy-badge" :style="{ background: (strategyColors[stage.strategy] || '#94a3b8') + '22', color: strategyColors[stage.strategy] }">
                  {{ stage.strategy }}
                </span>
                <span class="stage-agents">{{ stage.agents.length }} agents</span>
              </div>
              <div v-if="stage.depends_on.length" class="stage-deps">
                depends on: {{ stage.depends_on.join(', ') }}
              </div>
              <div v-if="expandedStage === stage.id" class="stage-expand">
                <div v-if="stage.results.length" class="results-list">
                  <div v-for="r in stage.results" :key="r.agent_id" class="result-row">
                    <span class="r-dot" :class="r.success ? 'ok' : 'fail'"></span>
                    <span>{{ r.agent_id }}</span>
                    <span class="r-tokens">{{ r.tokens_used.toLocaleString() }} tok</span>
                    <span class="r-dur">{{ r.duration_ms }}ms</span>
                  </div>
                </div>
                <div v-else class="no-results">{{ t('workflow.noResults') }}</div>
              </div>
            </div>
          </div>
        </div>
      </template>
    </main>

    <!-- Right: Create form -->
    <aside class="wf-form">
      <h3 class="form-title">{{ t('workflow.createNew') }}</h3>
      <div class="form-group">
        <label class="form-label">{{ t('workflow.name') }}</label>
        <input class="form-input" v-model="formName" :placeholder="t('workflow.namePlaceholder')" />
      </div>
      <div class="form-group">
        <label class="form-label">{{ t('workflow.description') }}</label>
        <textarea class="form-textarea" v-model="formDesc" rows="3" :placeholder="t('workflow.descPlaceholder')"></textarea>
      </div>
      <button class="btn btn-secondary full-w" :disabled="generating || !formDesc.trim()" @click="generateFromTask">
        {{ generating ? t('workflow.generating') : t('workflow.generateFromTask') }}
      </button>

      <div class="form-divider">{{ t('workflow.orManual') }}</div>

      <div class="stage-builder">
        <div v-for="(s, idx) in formStages" :key="idx" class="stage-form-row">
          <div class="sfp-header">
            <span class="sfp-idx">#{{ idx + 1 }}</span>
            <button class="btn-icon" @click="removeStage(idx)">&times;</button>
          </div>
          <input class="form-input" v-model="s.name" :placeholder="t('workflow.stageName')" />
          <select class="form-select" v-model="s.strategy">
            <option v-for="strat in strategies" :key="strat" :value="strat">{{ strat }}</option>
          </select>
          <input class="form-input" v-model="s.agents" :placeholder="t('workflow.agentsComma')" />
          <input class="form-input" v-model="s.dependsOn" :placeholder="t('workflow.dependsOnComma')" />
        </div>
        <button class="btn btn-ghost full-w" @click="addStage">+ {{ t('workflow.addStage') }}</button>
      </div>

      <div class="form-group">
        <label class="form-label">{{ t('workflow.maxConcurrent') }}: {{ formMaxAgents }}</label>
        <input type="range" min="1" max="10" v-model.number="formMaxAgents" class="form-range" />
      </div>
      <div class="form-group">
        <label class="form-label">{{ t('workflow.failurePolicy') }}</label>
        <select class="form-select full-w" v-model="formFailurePolicy">
          <option v-for="fp in failurePolicies" :key="fp" :value="fp">{{ fp }}</option>
        </select>
      </div>
      <button class="btn btn-primary full-w" :disabled="submitting || !formName.trim() || !formStages.length" @click="submitWorkflow">
        {{ submitting ? t('common.submitting') : t('workflow.submit') }}
      </button>

      <div v-if="error" class="form-error">{{ error }}</div>
    </aside>
  </div>
</template>

<style scoped>
.wf-editor {
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

  display: grid; grid-template-columns: 220px 1fr 280px;
  height: 100%; background: var(--bg-main); color: var(--text-primary);
  font-family: system-ui, -apple-system, sans-serif;
}

/* Sidebar */
.wf-sidebar { border-right: 1px solid var(--border); padding: 16px; overflow-y: auto; }
.sb-title { font-size: 16px; font-weight: 600; margin-bottom: 12px; }
.sb-title::before { content: '\u2699\FE0F '; }
.sb-loading, .sb-empty { color: var(--text-muted); font-size: 13px; padding: 24px 0; text-align: center; }

.wf-list { display: flex; flex-direction: column; gap: 4px; }
.wf-item {
  display: flex; align-items: center; gap: 8px; padding: 8px 10px;
  border-radius: var(--radius-md); cursor: pointer; transition: all 0.15s;
}
.wf-item:hover { background: var(--bg-subtle); }
.wf-item.selected { background: rgba(99, 102, 241, 0.15); }
.wf-status-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.wf-item-info { min-width: 0; }
.wf-item-name { display: block; font-size: 13px; font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.wf-item-meta { display: block; font-size: 11px; color: var(--text-muted); }

/* Center */
.wf-center { padding: 20px; overflow-y: auto; }
.center-empty { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-muted); font-size: 14px; }

.wf-detail-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px; }
.wf-name { font-size: 18px; font-weight: 600; }
.wf-desc { font-size: 13px; color: var(--text-secondary); margin-top: 4px; }
.status-badge { padding: 3px 10px; border-radius: var(--radius-full); font-size: 12px; font-weight: 500; white-space: nowrap; }

.wf-progress { margin-bottom: 20px; }
.progress-bar { height: 6px; background: var(--bg-subtle); border-radius: 3px; overflow: hidden; }
.progress-fill { height: 100%; background: linear-gradient(90deg, var(--accent), var(--success)); border-radius: 3px; transition: width 0.3s; }
.progress-stats { display: flex; gap: 16px; margin-top: 6px; font-size: 12px; color: var(--text-secondary); }

/* Stage timeline */
.stage-timeline { position: relative; }
.stage-node { display: flex; gap: 12px; cursor: pointer; }
.stage-connector { display: flex; flex-direction: column; align-items: center; width: 20px; flex-shrink: 0; }
.stage-dot { width: 12px; height: 12px; border-radius: 50%; border: 2px solid var(--bg-main); }
.stage-line { width: 2px; flex: 1; background: var(--border); min-height: 20px; }
.stage-content { flex: 1; padding-bottom: 16px; }
.stage-header { display: flex; align-items: center; gap: 8px; }
.stage-name { font-size: 14px; font-weight: 500; }
.strategy-badge { padding: 2px 8px; border-radius: var(--radius-full); font-size: 11px; font-weight: 500; }
.stage-agents { font-size: 11px; color: var(--text-muted); }
.stage-deps { font-size: 11px; color: var(--text-muted); margin-top: 4px; }

.stage-expand {
  margin-top: 8px; padding: 10px; background: var(--bg-surface); border-radius: var(--radius-md);
  border: 1px solid var(--border);
}
.results-list { display: flex; flex-direction: column; gap: 4px; }
.result-row { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--text-secondary); }
.r-dot { width: 6px; height: 6px; border-radius: 50%; }
.r-dot.ok { background: var(--success); }
.r-dot.fail { background: var(--error); }
.r-tokens, .r-dur { font-size: 11px; color: var(--text-muted); }
.no-results { font-size: 12px; color: var(--text-muted); }

/* Right form */
.wf-form { border-left: 1px solid var(--border); padding: 16px; overflow-y: auto; }
.form-title { font-size: 14px; font-weight: 600; margin-bottom: 12px; }

.form-group { margin-bottom: 12px; }
.form-label { display: block; font-size: 12px; font-weight: 500; color: var(--text-secondary); margin-bottom: 4px; }
.form-input {
  width: 100%; padding: 6px 10px; background: var(--bg-main); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px;
}
.form-input:focus { border-color: var(--accent); outline: none; }
.form-input::placeholder { color: var(--text-muted); }
.form-textarea {
  width: 100%; padding: 6px 10px; background: var(--bg-main); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px; resize: vertical;
}
.form-textarea:focus { border-color: var(--accent); outline: none; }
.form-textarea::placeholder { color: var(--text-muted); }
.form-select {
  padding: 6px 10px; background: var(--bg-main); border: 1px solid var(--border);
  border-radius: var(--radius-sm); color: var(--text-primary); font-size: 12px;
}
.form-select:focus { border-color: var(--accent); outline: none; }
.form-range { width: 100%; accent-color: var(--accent); }
.full-w { width: 100%; }

.form-divider {
  text-align: center; font-size: 11px; color: var(--text-muted);
  margin: 14px 0; border-top: 1px solid var(--border); padding-top: 10px;
}

.stage-builder { display: flex; flex-direction: column; gap: 10px; margin-bottom: 12px; }
.stage-form-row {
  padding: 8px; background: var(--bg-surface); border-radius: var(--radius-md);
  border: 1px solid var(--border); display: flex; flex-direction: column; gap: 6px;
}
.sfp-header { display: flex; justify-content: space-between; align-items: center; }
.sfp-idx { font-size: 11px; font-weight: 600; color: var(--accent); }
.btn-icon { background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 16px; }
.btn-icon:hover { color: var(--error); }

.btn {
  display: inline-flex; align-items: center; justify-content: center;
  padding: 6px 14px; font-size: 13px; font-weight: 500;
  border-radius: var(--radius-md); border: none; cursor: pointer; transition: all 0.15s;
}
.btn-primary { background: var(--accent); color: white; }
.btn-primary:hover { background: var(--accent-hover); }
.btn-secondary { background: var(--bg-subtle); color: var(--text-primary); }
.btn-secondary:hover { background: var(--border); }
.btn-ghost { background: transparent; color: var(--text-secondary); }
.btn-ghost:hover { color: var(--text-primary); }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }

.form-error { margin-top: 8px; padding: 8px; background: rgba(239, 68, 68, 0.1); border-radius: var(--radius-sm); font-size: 12px; color: var(--error); }
</style>
