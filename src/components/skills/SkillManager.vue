<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { SkillMeta, SkillExecutionResult, SkillVersionHistory } from '@/lib/skill-system/types';
import { CORE_SKILLS } from '@/lib/skill-system/types';

const skills = ref<SkillMeta[]>([]);
const selectedSkill = ref<string | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const searchQuery = ref('');
const showExecutor = ref(false);
const showVersionHistory = ref(false);
const executionResult = ref<SkillExecutionResult | null>(null);

const filteredSkills = computed(() => {
  if (!searchQuery.value) return skills.value;
  const q = searchQuery.value.toLowerCase();
  return skills.value.filter(
    s => s.name.toLowerCase().includes(q) ||
         (s.description?.toLowerCase().includes(q) ?? false)
  );
});

const coreSkills = computed(() =>
  filteredSkills.value.filter(s => CORE_SKILLS.includes(s.name))
);

const userSkills = computed(() =>
  filteredSkills.value.filter(s => !CORE_SKILLS.includes(s.name))
);

async function loadSkills() {
  loading.value = true;
  error.value = null;
  try {
    skills.value = await invoke<SkillMeta[]>('skills_list');
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function selectSkill(name: string) {
  selectedSkill.value = name;
  showExecutor.value = false;
  showVersionHistory.value = false;
}

function openExecutor() {
  if (!selectedSkill.value) return;
  showExecutor.value = true;
  showVersionHistory.value = false;
}

function openVersionHistory() {
  if (!selectedSkill.value) return;
  showVersionHistory.value = true;
  showExecutor.value = false;
}

async function installBuiltins() {
  loading.value = true;
  try {
    await invoke('skill_manage', { action: 'install_builtins', name: '_all_' });
    await loadSkills();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function deleteSkill(name: string) {
  if (!confirm(`Delete skill '${name}'?`)) return;
  try {
    await invoke('skill_manage', { action: 'delete', name });
    if (selectedSkill.value === name) selectedSkill.value = null;
    await loadSkills();
  } catch (e) {
    error.value = String(e);
  }
}

function onExecutionDone(result: SkillExecutionResult) {
  executionResult.value = result;
}

onMounted(() => loadSkills());
</script>

<template>
  <div class="skill-manager">
    <header class="skill-manager__header">
      <h2>Skill Manager</h2>
      <div class="skill-manager__actions">
        <input v-model="searchQuery" placeholder="Search skills..." class="search-input" />
        <button @click="installBuiltins" :disabled="loading" class="btn btn--secondary">
          Install Built-ins
        </button>
        <button @click="loadSkills" class="btn btn--secondary">Refresh</button>
      </div>
    </header>

    <div v-if="error" class="error-banner">{{ error }}</div>
    <div v-if="loading" class="loading">Loading skills...</div>

    <div class="skill-manager__layout">
      <div class="skill-manager__list">
        <section class="skill-section">
          <h3>Core Skills ({{ coreSkills.length }}/16)</h3>
          <div class="skill-grid">
            <div
              v-for="skill in coreSkills" :key="skill.name"
              class="skill-card skill-card--core"
              :class="{ 'skill-card--selected': selectedSkill === skill.name }"
              @click="selectSkill(skill.name)"
            >
              <span class="skill-name">{{ skill.name }}</span>
              <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
            </div>
          </div>
        </section>

        <section class="skill-section">
          <h3>User Skills ({{ userSkills.length }})</h3>
          <div class="skill-grid">
            <div
              v-for="skill in userSkills" :key="skill.name"
              class="skill-card skill-card--user"
              :class="{ 'skill-card--selected': selectedSkill === skill.name }"
              @click="selectSkill(skill.name)"
            >
              <span class="skill-name">{{ skill.name }}</span>
              <button @click.stop="deleteSkill(skill.name)" class="delete-btn">x</button>
            </div>
          </div>
        </section>
      </div>

      <div v-if="selectedSkill" class="skill-manager__detail">
        <div class="detail-tabs">
          <button :class="{ active: !showExecutor && !showVersionHistory }" @click="selectSkill(selectedSkill!)">Info</button>
          <button :class="{ active: showExecutor }" @click="openExecutor">Execute</button>
          <button :class="{ active: showVersionHistory }" @click="openVersionHistory">Versions</button>
        </div>

        <SkillExecutor
          v-if="showExecutor"
          :skill-name="selectedSkill"
          @done="onExecutionDone"
        />
        <SkillVersionHistory
          v-else-if="showVersionHistory"
          :skill-name="selectedSkill"
        />
        <div v-else class="detail-info">
          <h3>{{ selectedSkill }}</h3>
          <p>{{ skills.find(s => s.name === selectedSkill)?.description || 'No description' }}</p>
          <div class="detail-meta">
            <span>Version: {{ skills.find(s => s.name === selectedSkill)?.version || 'N/A' }}</span>
            <span>Category: {{ skills.find(s => s.name === selectedSkill)?.category || 'N/A' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-manager { padding: 1rem; }
.skill-manager__header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
.skill-manager__actions { display: flex; gap: 0.5rem; align-items: center; }
.search-input { padding: 0.4rem 0.8rem; border: 1px solid #ddd; border-radius: 4px; }
.error-banner { background: #fee; color: #c00; padding: 0.5rem; border-radius: 4px; margin-bottom: 1rem; }
.loading { color: #666; padding: 1rem; }
.skill-manager__layout { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.skill-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 0.5rem; }
.skill-card { padding: 0.5rem; border-radius: 4px; cursor: pointer; display: flex; justify-content: space-between; align-items: center; }
.skill-card--core { background: #e3f2fd; border: 1px solid #1976d2; }
.skill-card--user { background: #f3e5f5; border: 1px solid #7b1fa2; }
.skill-card--selected { box-shadow: 0 0 0 2px #1976d2; }
.skill-name { font-weight: 500; font-size: 0.9rem; }
.skill-version { font-size: 0.7rem; color: #666; }
.delete-btn { background: none; border: none; color: #c00; cursor: pointer; font-size: 0.8rem; }
.detail-tabs { display: flex; gap: 0.25rem; margin-bottom: 1rem; }
.detail-tabs button { padding: 0.4rem 1rem; border: 1px solid #ddd; background: white; cursor: pointer; border-radius: 4px 4px 0 0; }
.detail-tabs button.active { background: #1976d2; color: white; border-color: #1976d2; }
.detail-info h3 { margin: 0 0 0.5rem; }
.detail-meta { display: flex; gap: 1rem; color: #666; font-size: 0.85rem; margin-top: 0.5rem; }
.btn { padding: 0.4rem 0.8rem; border-radius: 4px; cursor: pointer; font-size: 0.85rem; }
.btn--primary { background: #1976d2; color: white; border: none; }
.btn--secondary { background: white; border: 1px solid #ddd; }
</style>
