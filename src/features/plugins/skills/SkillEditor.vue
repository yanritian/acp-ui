<script setup lang="ts">
/**
 * Skill Editor - Edit skill content, manage versions, trigger evolution
 *
 * Features:
 * - Edit skill Markdown content
 * - View version history
 * - Rollback to previous version
 * - Manual self-improve trigger
 */

import { ref, onMounted, watch } from 'vue';
import { skillInvoker } from '@/lib/skill-system/skill-invoker';
import type { SkillVersionSnapshot } from '@/lib/skill-system/types';

// Props
const props = defineProps<{
  skillName: string;
}>();

// Emit
const emit = defineEmits<{
  saved: [name: string];
  evolved: [name: string, summary: string];
}>();

// State
const skillContent = ref<string>('');
const originalContent = ref<string>('');
const isEditing = ref(false);
const isSaving = ref(false);
const evolutionFeedback = ref<string>('');
const showVersionHistory = ref(false);
const versions = ref<SkillVersionSnapshot[]>([]);
const hasChanges = ref(false);

// Watch for changes
watch(skillContent, (newContent) => {
  hasChanges.value = newContent !== originalContent.value;
});

// Methods
async function loadSkill() {
  skillContent.value = await skillInvoker.viewSkill(props.skillName) || '';
  originalContent.value = skillContent.value;
}

async function saveSkill() {
  if (!hasChanges.value) return;

  isSaving.value = true;
  const result = await skillInvoker.manageSkill('update', {
    name: props.skillName,
    content: skillContent.value,
  });

  isSaving.value = false;
  if (result.includes('updated')) {
    originalContent.value = skillContent.value;
    hasChanges.value = false;
    emit('saved', props.skillName);
  }
}

async function triggerEvolution() {
  if (!evolutionFeedback.value) return;

  const result = await skillInvoker.selfImprove(props.skillName, evolutionFeedback.value);
  emit('evolved', props.skillName, result);
  evolutionFeedback.value = '';

  // Reload skill
  await loadSkill();
}

async function loadVersionHistory() {
  // In production, this would call a Tauri command
  // For now, mock data
  versions.value = [
    {
      semver: '1.0.0',
      content: skillContent.value,
      evolvedAt: new Date().toISOString(),
      reason: 'Initial creation',
      generation: 0,
    },
  ];
  showVersionHistory.value = true;
}

async function rollbackVersion(semver: string) {
  const version = versions.value.find(v => v.semver === semver);
  if (version) {
    skillContent.value = version.content;
    hasChanges.value = true;
  }
}

function cancelEdit() {
  skillContent.value = originalContent.value;
  hasChanges.value = false;
}

// Lifecycle
onMounted(() => {
  loadSkill();
});

// Expose for parent
defineExpose({
  loadSkill,
});
</script>

<template>
  <div class="skill-editor">
    <!-- Header -->
    <header class="skill-editor__header">
      <h3>Editing: {{ skillName }}</h3>
      <div class="skill-editor__status">
        <span v-if="hasChanges" class="status-changed">
          ⚠️ Unsaved changes
        </span>
        <span v-else class="status-clean">
          ✓ Saved
        </span>
      </div>
    </header>

    <!-- Editor -->
    <div class="skill-editor__content">
      <textarea
        v-model="skillContent"
        :disabled="isSaving"
        class="editor-textarea"
        placeholder="# Skill Name

## Description
Brief description of the skill

## Steps
1. Step one
2. Step two
3. Step three

## Parameters
- param1: Description
- param2: Description"
      ></textarea>
    </div>

    <!-- Actions -->
    <div class="skill-editor__actions">
      <button
        @click="saveSkill"
        :disabled="!hasChanges || isSaving"
        class="btn btn--primary"
      >
        {{ isSaving ? 'Saving...' : 'Save' }}
      </button>
      <button
        @click="cancelEdit"
        :disabled="!hasChanges"
        class="btn btn--secondary"
      >
        Cancel
      </button>
      <button @click="loadVersionHistory" class="btn btn--secondary">
        Version History
      </button>
    </div>

    <!-- Evolution Section -->
    <div class="skill-editor__evolution">
      <h4>Self-Improve</h4>
      <p class="evolution-hint">
        Provide feedback to trigger skill evolution. The skill will automatically
        improve based on your feedback.
      </p>
      <textarea
        v-model="evolutionFeedback"
        class="feedback-input"
        placeholder="Describe what could be improved..."
      ></textarea>
      <button
        @click="triggerEvolution"
        :disabled="!evolutionFeedback"
        class="btn btn--evolve"
      >
        🔄 Trigger Evolution
      </button>
    </div>

    <!-- Version History Dialog -->
    <div v-if="showVersionHistory" class="version-dialog">
      <div class="version-dialog__content">
        <h4>Version History</h4>

        <div class="version-list">
          <div
            v-for="version in versions"
            :key="version.semver"
            class="version-item"
          >
            <div class="version-header">
              <span class="version-semver">v{{ version.semver }}</span>
              <span class="version-gen">Gen {{ version.generation }}</span>
            </div>
            <div class="version-reason">{{ version.reason }}</div>
            <div class="version-date">{{ version.evolvedAt }}</div>
            <button
              @click="rollbackVersion(version.semver)"
              class="btn btn--small"
            >
              Rollback
            </button>
          </div>
        </div>

        <button @click="showVersionHistory = false" class="btn btn--secondary">
          Close
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-editor {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
}

.skill-editor__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-changed {
  color: #ff9800;
}

.status-clean {
  color: #4caf50;
}

.editor-textarea {
  width: 100%;
  min-height: 400px;
  font-family: monospace;
  padding: 1rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  resize: vertical;
}

.skill-editor__actions {
  display: flex;
  gap: 0.5rem;
}

.skill-editor__evolution {
  background: #f5f5f5;
  padding: 1rem;
  border-radius: 4px;
}

.evolution-hint {
  color: #666;
  font-size: 0.875rem;
}

.feedback-input {
  width: 100%;
  height: 80px;
  font-family: inherit;
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-bottom: 0.5rem;
}

.btn {
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
}

.btn--primary {
  background: #1976d2;
  color: white;
  border: none;
}

.btn--primary:disabled {
  background: #ccc;
}

.btn--secondary {
  background: white;
  border: 1px solid #ddd;
}

.btn--evolve {
  background: #7b1fa2;
  color: white;
  border: none;
}

.btn--small {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
}

.version-dialog {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
}

.version-dialog__content {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  width: 500px;
  max-height: 80vh;
  overflow: auto;
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  margin-bottom: 1rem;
}

.version-item {
  border: 1px solid #ddd;
  padding: 0.75rem;
  border-radius: 4px;
}

.version-header {
  display: flex;
  justify-content: space-between;
}

.version-semver {
  font-weight: 500;
}

.version-gen {
  color: #7b1fa2;
}

.version-reason {
  color: #666;
  font-size: 0.875rem;
  margin-top: 0.25rem;
}

.version-date {
  color: #999;
  font-size: 0.75rem;
}
</style>