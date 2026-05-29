<script setup lang="ts">
/**
 * Skill Explorer - Browse, search, and invoke skills
 *
 * Features:
 * - List all skills with metadata
 * - Fuzzy search by name/description
 * - Quick invoke with parameters
 * - View skill content
 * - Rate skills for self-evolution
 */

import { ref, computed, onMounted } from 'vue';
import { skillInvoker, invokeSkill, createSkill } from '@/lib/skill-system/skill-invoker';
import type { SkillMeta, SkillExecutionResult, CreateSkillResult } from '@/lib/skill-system/types';
import { CORE_SKILLS } from '@/lib/skill-system/types';

// State
const skills = ref<SkillMeta[]>([]);
const selectedSkill = ref<string | null>(null);
const skillContent = ref<string>('');
const searchQuery = ref<string>('');
const invokeParams = ref<string>('{\n  \n}');
const invokeResult = ref<SkillExecutionResult | null>(null);
const showCreateDialog = ref(false);
const createResult = ref<CreateSkillResult | null>(null);

// New skill form
const newSkillName = ref('');
const newSkillDescription = ref('');
const newSkillNaturalSpec = ref('');

// Computed
const filteredSkills = computed(() => {
  if (!searchQuery.value) return skills.value;
  const query = searchQuery.value.toLowerCase();
  return skills.value.filter(
    s => s.name.toLowerCase().includes(query) ||
         (s.description?.toLowerCase().includes(query) ?? false)
  );
});

const coreSkills = computed(() => {
  return skills.value.filter(s => CORE_SKILLS.includes(s.name));
});

const userSkills = computed(() => {
  return skills.value.filter(s => !CORE_SKILLS.includes(s.name));
});

// Methods
async function loadSkills() {
  skills.value = await skillInvoker.listSkills();
}

async function viewSkill(name: string) {
  selectedSkill.value = name;
  skillContent.value = await skillInvoker.viewSkill(name) || 'Failed to load skill content';
}

async function invokeSelectedSkill() {
  if (!selectedSkill.value) return;

  try {
    const params = JSON.parse(invokeParams.value);
    invokeResult.value = await invokeSkill(selectedSkill.value, params);
  } catch (e) {
    invokeResult.value = {
      success: false,
      output: '',
      error: `Invalid JSON params: ${e}`,
      durationMs: 0,
      toolCallsCount: 0,
      evolved: false,
    };
  }
}

async function rateSkill(rating: number) {
  if (!selectedSkill.value) return;
  await skillInvoker.rateSkill({ skillName: selectedSkill.value, rating });
  // Show feedback
  alert(`Rated ${selectedSkill.value} as ${rating}/5. This feedback will help skill self-evolution.`);
}

async function createNewSkill() {
  createResult.value = await createSkill(
    newSkillName.value,
    newSkillDescription.value,
    newSkillNaturalSpec.value,
  );

  if (createResult.value.created) {
    await loadSkills();
    showCreateDialog.value = false;
    // Reset form
    newSkillName.value = '';
    newSkillDescription.value = '';
    newSkillNaturalSpec.value = '';
  }
}

async function installBuiltins() {
  const result = await skillInvoker.installBuiltinSkills();
  alert(result);
  await loadSkills();
}

// Lifecycle
onMounted(() => {
  loadSkills();
});
</script>

<template>
  <div class="skill-explorer">
    <!-- Header -->
    <header class="skill-explorer__header">
      <h2>Skill Explorer</h2>
      <div class="skill-explorer__actions">
        <button @click="installBuiltins" class="btn btn--secondary">
          Install Built-ins
        </button>
        <button @click="showCreateDialog = true" class="btn btn--primary">
          + New Skill
        </button>
      </div>
    </header>

    <!-- Search -->
    <div class="skill-explorer__search">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search skills..."
        class="search-input"
      />
    </div>

    <!-- Skill Lists -->
    <div class="skill-explorer__lists">
      <!-- Core Skills -->
      <section class="skill-section">
        <h3>Core Skills ({{ coreSkills.length }}/16)</h3>
        <div class="skill-grid">
          <div
            v-for="skill in coreSkills"
            :key="skill.name"
            class="skill-card skill-card--core"
            :class="{ 'skill-card--selected': selectedSkill === skill.name }"
            @click="viewSkill(skill.name)"
          >
            <span class="skill-name">{{ skill.name }}</span>
            <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
          </div>
        </div>
      </section>

      <!-- User Skills -->
      <section class="skill-section">
        <h3>User Skills ({{ userSkills.length }})</h3>
        <div class="skill-grid">
          <div
            v-for="skill in userSkills"
            :key="skill.name"
            class="skill-card skill-card--user"
            :class="{ 'skill-card--selected': selectedSkill === skill.name }"
            @click="viewSkill(skill.name)"
          >
            <span class="skill-name">{{ skill.name }}</span>
            <span v-if="skill.generation" class="skill-gen">Gen {{ skill.generation }}</span>
          </div>
        </div>
      </section>
    </div>

    <!-- Skill Detail -->
    <div v-if="selectedSkill" class="skill-explorer__detail">
      <h3>{{ selectedSkill }}</h3>

      <!-- Content -->
      <div class="skill-content">
        <pre>{{ skillContent }}</pre>
      </div>

      <!-- Invoke -->
      <div class="skill-invoke">
        <h4>Invoke Skill</h4>
        <textarea
          v-model="invokeParams"
          placeholder="JSON parameters"
          class="params-input"
        ></textarea>
        <button @click="invokeSelectedSkill" class="btn btn--primary">
          ⚡ Invoke
        </button>
      </div>

      <!-- Result -->
      <div v-if="invokeResult" class="skill-result">
        <div :class="['result-status', invokeResult.success ? 'success' : 'error']">
          {{ invokeResult.success ? '✓ Success' : '✗ Failed' }}
        </div>
        <div v-if="invokeResult.evolved" class="result-evolution">
          🔄 Skill evolved: {{ invokeResult.evolutionSummary }}
        </div>
        <pre class="result-output">{{ invokeResult.output }}</pre>
        <div class="result-meta">
          Duration: {{ invokeResult.durationMs }}ms | Tool calls: {{ invokeResult.toolCallsCount }}
        </div>
      </div>

      <!-- Rating -->
      <div class="skill-rating">
        <h4>Rate this skill</h4>
        <div class="rating-buttons">
          <button @click="rateSkill(1)" class="rating-btn">1</button>
          <button @click="rateSkill(2)" class="rating-btn">2</button>
          <button @click="rateSkill(3)" class="rating-btn">3</button>
          <button @click="rateSkill(4)" class="rating-btn">4</button>
          <button @click="rateSkill(5)" class="rating-btn">5</button>
        </div>
      </div>
    </div>

    <!-- Create Dialog -->
    <div v-if="showCreateDialog" class="create-dialog">
      <div class="create-dialog__content">
        <h3>Create Skill from Natural Language</h3>

        <div class="form-group">
          <label>Name (kebab-case)</label>
          <input v-model="newSkillName" placeholder="my-custom-task" />
        </div>

        <div class="form-group">
          <label>Description</label>
          <input v-model="newSkillDescription" placeholder="Brief description" />
        </div>

        <div class="form-group">
          <label>Natural Language Specification</label>
          <textarea
            v-model="newSkillNaturalSpec"
            placeholder="When user asks to deploy, first build, then test, then push to production"
          ></textarea>
        </div>

        <div class="create-dialog__actions">
          <button @click="createNewSkill" class="btn btn--primary">
            ✨ Create
          </button>
          <button @click="showCreateDialog = false" class="btn btn--secondary">
            Cancel
          </button>
        </div>

        <div v-if="createResult && !createResult.created" class="create-error">
          {{ createResult.message }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-explorer {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: 1rem;
  padding: 1rem;
}

.skill-explorer__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.skill-explorer__search {
  margin-bottom: 1rem;
}

.search-input {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.skill-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 0.5rem;
}

.skill-card {
  padding: 0.5rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.skill-card--core {
  background: #e3f2fd;
  border: 1px solid #1976d2;
}

.skill-card--user {
  background: #f3e5f5;
  border: 1px solid #7b1fa2;
}

.skill-card--selected {
  box-shadow: 0 0 0 2px #1976d2;
}

.skill-name {
  font-weight: 500;
}

.skill-version, .skill-gen {
  font-size: 0.75rem;
  color: #666;
}

.skill-content {
  background: #f5f5f5;
  padding: 1rem;
  border-radius: 4px;
  overflow: auto;
  max-height: 300px;
}

.skill-content pre {
  white-space: pre-wrap;
  font-family: monospace;
}

.params-input {
  width: 100%;
  height: 100px;
  font-family: monospace;
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.result-status.success {
  color: #4caf50;
}

.result-status.error {
  color: #f44336;
}

.result-evolution {
  background: #fff3e0;
  padding: 0.5rem;
  border-radius: 4px;
  margin-bottom: 0.5rem;
}

.rating-buttons {
  display: flex;
  gap: 0.5rem;
}

.rating-btn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: 1px solid #ddd;
  background: white;
  cursor: pointer;
}

.rating-btn:hover {
  background: #e3f2fd;
}

.create-dialog {
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

.create-dialog__content {
  background: white;
  padding: 2rem;
  border-radius: 8px;
  width: 400px;
}

.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  margin-bottom: 0.25rem;
  font-weight: 500;
}

.form-group input, .form-group textarea {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.create-dialog__actions {
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
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

.btn--secondary {
  background: white;
  border: 1px solid #ddd;
}
</style>