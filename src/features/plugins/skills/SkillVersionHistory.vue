<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { invokeOrProxy } from '@/lib/host';
import type { SkillVersionSnapshot } from '@/lib/skill-system/types';

const props = defineProps<{ skillName: string }>();

const versions = ref<SkillVersionSnapshot[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const rollbackResult = ref<string | null>(null);

async function loadVersions() {
  loading.value = true;
  error.value = null;
  try {
    const history = await invokeOrProxy<{ skill: string; versions: SkillVersionSnapshot[] }>(
      'skill_manage',
      { action: 'version_list', name: props.skillName }
    );
    versions.value = history.versions || [];
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function rollback(semver: string) {
  if (!confirm(`Rollback ${props.skillName} to v${semver}?`)) return;
  try {
    const result = await invokeOrProxy<{ message: string }>(
      'skill_manage',
      { action: 'version_rollback', name: props.skillName, version: semver }
    );
    rollbackResult.value = result.message;
    await loadVersions();
  } catch (e) {
    error.value = String(e);
  }
}

watch(() => props.skillName, () => loadVersions());
onMounted(() => loadVersions());
</script>

<template>
  <div class="version-history">
    <h4>Version History: {{ skillName }}</h4>
    <div v-if="loading" class="loading">Loading...</div>
    <div v-if="error" class="error-msg">{{ error }}</div>
    <div v-if="rollbackResult" class="success-msg">{{ rollbackResult }}</div>

    <div v-if="!loading && versions.length === 0" class="empty">
      No version history available.
    </div>

    <div class="version-list">
      <div v-for="v in versions" :key="v.semver" class="version-item">
        <div class="version-header">
          <span class="version-semver">v{{ v.semver }}</span>
          <span class="version-gen">Gen {{ v.generation }}</span>
          <span class="version-date">{{ new Date(v.evolvedAt).toLocaleString() }}</span>
        </div>
        <div class="version-reason">{{ v.reason }}</div>
        <div class="version-actions">
          <button @click="rollback(v.semver)" class="btn btn--small">Rollback</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.version-history { padding: 1rem; }
.loading, .empty { color: #666; padding: 0.5rem; }
.error-msg { color: #c00; padding: 0.5rem; }
.success-msg { color: #4caf50; padding: 0.5rem; background: #e8f5e9; border-radius: 4px; }
.version-list { display: flex; flex-direction: column; gap: 0.5rem; }
.version-item { padding: 0.75rem; border: 1px solid #ddd; border-radius: 4px; }
.version-header { display: flex; gap: 1rem; align-items: center; margin-bottom: 0.25rem; }
.version-semver { font-weight: 600; color: #1976d2; }
.version-gen { font-size: 0.8rem; color: #666; }
.version-date { font-size: 0.8rem; color: #999; }
.version-reason { font-size: 0.85rem; color: #333; margin-bottom: 0.5rem; }
.version-actions { display: flex; gap: 0.5rem; }
.btn { padding: 0.3rem 0.6rem; border-radius: 4px; cursor: pointer; font-size: 0.8rem; }
.btn--small { background: white; border: 1px solid #ddd; }
.btn--small:hover { background: #f5f5f5; }
</style>
