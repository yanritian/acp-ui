<script setup lang="ts">
import { ref, watch } from 'vue';
import { invokeOrProxy } from '@/lib/host';
import type { SkillExecutionResult } from '@/lib/skill-system/types';

const props = defineProps<{ skillName: string }>();
const emit = defineEmits<{ done: [result: SkillExecutionResult] }>();

const params = ref('{\n  \n}');
const executing = ref(false);
const result = ref<SkillExecutionResult | null>(null);
const error = ref<string | null>(null);

watch(() => props.skillName, () => { result.value = null; error.value = null; });

async function execute() {
  executing.value = true;
  error.value = null;
  result.value = null;
  try {
    const parsedParams = JSON.parse(params.value);
    result.value = await invokeOrProxy<SkillExecutionResult>('skill_invoke', {
      skillName: props.skillName,
      parameters: parsedParams,
      context: { cwd: '.' },
    });
    emit('done', result.value);
  } catch (e) {
    error.value = String(e);
    result.value = {
      success: false, output: '', error: String(e),
      durationMs: 0, toolCallsCount: 0, evolved: false,
    };
  } finally {
    executing.value = false;
  }
}
</script>

<template>
  <div class="skill-executor">
    <h4>Execute: {{ skillName }}</h4>
    <div class="form-group">
      <label>Parameters (JSON)</label>
      <textarea v-model="params" class="params-input" rows="6"></textarea>
    </div>
    <button @click="execute" :disabled="executing" class="btn btn--primary">
      {{ executing ? 'Executing...' : 'Execute' }}
    </button>

    <div v-if="error" class="error-msg">{{ error }}</div>

    <div v-if="result" class="result-panel">
      <div :class="['result-status', result.success ? 'success' : 'error']">
        {{ result.success ? 'Success' : 'Failed' }}
      </div>
      <div v-if="result.evolved" class="evolution-notice">
        Skill evolved: {{ result.evolutionSummary }}
      </div>
      <pre class="result-output">{{ result.output }}</pre>
      <div class="result-meta">
        Duration: {{ result.durationMs }}ms | Tool calls: {{ result.toolCallsCount }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-executor { padding: 1rem; }
.form-group { margin-bottom: 0.75rem; }
.form-group label { display: block; font-weight: 500; margin-bottom: 0.25rem; }
.params-input { width: 100%; font-family: monospace; padding: 0.5rem; border: 1px solid #ddd; border-radius: 4px; }
.error-msg { color: #c00; margin: 0.5rem 0; }
.result-panel { margin-top: 1rem; padding: 0.75rem; background: #f9f9f9; border-radius: 4px; }
.result-status.success { color: #4caf50; font-weight: 600; }
.result-status.error { color: #f44336; font-weight: 600; }
.evolution-notice { background: #fff3e0; padding: 0.5rem; border-radius: 4px; margin: 0.5rem 0; }
.result-output { white-space: pre-wrap; font-family: monospace; font-size: 0.85rem; max-height: 200px; overflow: auto; }
.result-meta { color: #666; font-size: 0.8rem; margin-top: 0.5rem; }
.btn { padding: 0.4rem 1rem; border-radius: 4px; cursor: pointer; }
.btn--primary { background: #1976d2; color: white; border: none; }
.btn--primary:disabled { opacity: 0.6; }
</style>
