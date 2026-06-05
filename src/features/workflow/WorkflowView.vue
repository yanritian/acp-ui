<script setup lang="ts">
import { ref, computed } from 'vue'
import { useWorkflowsStore, type WorkflowStep } from '@/stores/workflows'
import { useI18n } from '@/locales'

const { t } = useI18n()

const workflowStore = useWorkflowsStore()

const showCreateModal = ref(false)
const newWorkflowName = ref('')
const newWorkflowDescription = ref('')
const newSteps = ref<WorkflowStep[]>([])
const editingStepIndex = ref<number | null>(null)

const runningExecutions = computed(() =>
  workflowStore.workflows
    .filter(w => {
      const exec = workflowStore.executions.get(w.id)
      return exec?.status === 'running'
    })
    .map(w => ({
      ...w,
      execution: workflowStore.executions.get(w.id)!,
    }))
)

function getExecution(wfId: string) {
  return workflowStore.executions.get(wfId)
}

function addStep() {
  newSteps.value.push({
    id: crypto.randomUUID(),
    name: '',
    prompt: '',
    agentName: '',
    dependsOn: [],
  })
  editingStepIndex.value = newSteps.value.length - 1
}

function removeStep(index: number) {
  newSteps.value.splice(index, 1)
  if (editingStepIndex.value === index) editingStepIndex.value = null
  else if (editingStepIndex.value && editingStepIndex.value > index) editingStepIndex.value--
}

function handleCreate() {
  if (!newWorkflowName.value.trim() || newSteps.value.length === 0) return

  const validated = newSteps.value.map(s => ({
    ...s,
    agentName: s.agentName || undefined,
    dependsOn: s.dependsOn || [],
  }))

  workflowStore.createWorkflow(newWorkflowName.value.trim(), newWorkflowDescription.value.trim(), validated)
  resetForm()
}

function resetForm() {
  newWorkflowName.value = ''
  newWorkflowDescription.value = ''
  newSteps.value = []
  editingStepIndex.value = null
  showCreateModal.value = false
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleString()
}

function getExecutionStatus(wfId: string): string {
  const exec = getExecution(wfId)
  if (!exec) return 'idle'
  return exec.status
}

function getStatusIcon(status: string): string {
  switch (status) {
    case 'running': return '🔄'
    case 'completed': return '✅'
    case 'failed': return '❌'
    case 'cancelled': return '⏹️'
    default: return '⏸️'
  }
}
</script>

<template>
  <div class="workflow-view">
    <div class="workflow-header">
      <h3>{{ t('workflow.title') }}</h3>
      <button class="create-btn" @click="showCreateModal = true" :disabled="!workflowStore.hasAgents()">
        {{ t('workflow.createWorkflow') }}
      </button>
    </div>

    <!-- Running executions -->
    <div v-if="runningExecutions.length > 0" class="workflow-section">
      <h4>{{ t('workflow.running') }}</h4>
      <div class="workflow-list">
        <div v-for="item in runningExecutions" :key="item.id" class="workflow-card running">
          <div class="card-title">
            <span>{{ getStatusIcon('running') }}</span>
            <span class="card-name">{{ item.name }}</span>
          </div>
          <div class="card-steps">
            <span v-for="step in item.steps" :key="step.id" class="step-badge"
                  :class="item.execution.stepResults.get(step.id)?.status">
              {{ step.name || t('workflow.unnamed') }}
            </span>
          </div>
          <div class="card-actions">
            <button @click="workflowStore.cancelWorkflow(item.id)">{{ t('workflow.cancel') }}</button>
          </div>
        </div>
      </div>
    </div>

    <!-- All workflows -->
    <div class="workflow-section">
      <h4>{{ t('workflow.allWorkflows') }}</h4>
      <div class="workflow-list">
        <div v-for="wf in workflowStore.workflows" :key="wf.id" class="workflow-card" :class="getExecutionStatus(wf.id)">
          <div class="card-title">
            <span>{{ getStatusIcon(getExecutionStatus(wf.id)) }}</span>
            <span class="card-name">{{ wf.name }}</span>
          </div>
          <div class="card-meta">
            <span>{{ wf.steps.length }} {{ t('workflow.steps') }}</span>
            <span>{{ formatTime(wf.createdAt) }}</span>
          </div>
          <div class="card-actions">
            <button v-if="getExecutionStatus(wf.id) === 'idle'" @click="workflowStore.runWorkflow(wf.id)" :disabled="!workflowStore.hasAgents()">
              {{ t('workflow.run') }}
            </button>
            <button v-if="getExecutionStatus(wf.id) === 'running'" @click="workflowStore.cancelWorkflow(wf.id)">
              {{ t('workflow.cancel') }}
            </button>
            <button @click="workflowStore.deleteWorkflow(wf.id)">{{ t('workflow.delete') }}</button>
          </div>
        </div>
      </div>

      <div v-if="workflowStore.workflows.length === 0" class="empty-state">
        <p>{{ t('workflow.noWorkflows') }}</p>
        <p class="hint">{{ t('workflow.noWorkflowsHint') }}</p>
      </div>
    </div>

    <!-- Create modal -->
    <div v-if="showCreateModal" class="modal-overlay" @click.self="resetForm">
      <div class="modal-content">
        <div class="modal-header">
          <h3>{{ t('workflow.newName') }}</h3>
          <button @click="resetForm">✕</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>{{ t('workflow.newDescription') }}</label>
            <input v-model="newWorkflowName" type="text" :placeholder="t('workflow.namePlaceholder')" />
          </div>
          <div class="form-group">
            <label>{{ t('workflow.descriptionPlaceholder') }}</label>
            <textarea v-model="newWorkflowDescription" :placeholder="t('workflow.descriptionPlaceholder')" rows="2"></textarea>
          </div>
          <div class="form-group">
            <label>{{ t('workflow.steps') }}</label>
            <div v-for="(step, i) in newSteps" :key="step.id" class="step-editor">
              <div class="step-header">
                <span>{{ t('workflow.steps') }} {{ i + 1 }}</span>
                <button class="step-remove" @click="removeStep(i)">✕</button>
              </div>
              <input v-model="step.name" :placeholder="t('workflow.stepPlaceholder')" />
              <textarea v-model="step.prompt" :placeholder="t('workflow.promptPlaceholder')" rows="2"></textarea>
              <div class="step-options">
                <select v-model="step.agentName">
                  <option value="">{{ t('workflow.defaultAgent') }}</option>
                </select>
              </div>
            </div>
            <button class="add-step-btn" @click="addStep">{{ t('workflow.addStep') }}</button>
          </div>
          <div class="form-actions">
            <button class="cancel-btn" @click="resetForm">{{ t('workflow.cancel') }}</button>
            <button class="submit-btn" @click="handleCreate" :disabled="!newWorkflowName.trim() || newSteps.length === 0">
              {{ t('workflow.createWorkflow').replace('+ ', '') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workflow-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
}

.workflow-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.create-btn {
  padding: 8px 16px;
  border-radius: 4px;
  border: none;
  background: var(--bg-primary);
  color: white;
  cursor: pointer;
}

.create-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.workflow-section h4 {
  margin: 0 0 8px 0;
  color: var(--text-muted);
  font-size: 14px;
}

.workflow-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.workflow-card {
  padding: 10px 14px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-surface);
}

.workflow-card.running {
  border-color: #2196f3;
  border-left: 3px solid #2196f3;
}

.workflow-card.completed {
  border-color: #4caf50;
}

.workflow-card.failed {
  border-color: #ef4444;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.card-name {
  font-weight: 500;
}

.card-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.card-steps {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 8px;
}

.step-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  border: 1px solid var(--border-color);
}

.step-badge.completed { border-color: #4caf50; color: #4caf50; }
.step-badge.running { border-color: #2196f3; color: #2196f3; }
.step-badge.failed { border-color: #ef4444; color: #ef4444; }
.step-badge.skipped { border-color: #9e9e9e; color: #9e9e9e; }

.card-actions {
  display: flex;
  gap: 6px;
}

.card-actions button {
  padding: 3px 10px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: transparent;
  cursor: pointer;
  font-size: 12px;
}

.card-actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.empty-state {
  text-align: center;
  padding: 32px;
  color: var(--text-muted);
}

.empty-state .hint {
  font-size: 13px;
  margin-top: 4px;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-content {
  background: var(--bg-surface);
  border-radius: 12px;
  max-width: 520px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header button {
  padding: 4px 8px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 20px;
}

.modal-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-group label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 4px;
}

.form-group input,
.form-group textarea,
.form-group select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-main);
  color: var(--text-primary);
  font-size: 14px;
  box-sizing: border-box;
}

.step-editor {
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 8px;
  margin-bottom: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.step-header {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  font-weight: 500;
}

.step-remove {
  border: none;
  background: none;
  cursor: pointer;
  color: #ef4444;
}

.step-options {
  font-size: 12px;
  color: var(--text-muted);
}

.add-step-btn {
  padding: 6px;
  border: 1px dashed var(--border-color);
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  color: var(--text-muted);
  font-size: 13px;
}

.form-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 8px;
}

.cancel-btn {
  padding: 8px 16px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: transparent;
  cursor: pointer;
}

.submit-btn {
  padding: 8px 16px;
  border-radius: 4px;
  border: none;
  background: var(--bg-primary);
  color: white;
  cursor: pointer;
}

.submit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
