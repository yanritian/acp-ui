<script setup lang="ts">
import { useI18n } from '@/locales'
import { useTeamRuntimeStore } from '@/stores/team-runtime'
import TrafficMonitor from '@/features/monitoring/TrafficMonitor.vue'

const { t } = useI18n()
const teamRuntime = useTeamRuntimeStore()
</script>

<template>
  <div class="view-container">
    <h3>{{ t('navigation.monitor') }}</h3>
    <div v-if="teamRuntime.events.length > 0" class="event-list">
      <div v-for="event in teamRuntime.events.slice(0, 50)" :key="event.id" class="event-item">
        <span class="event-type">{{ event.type }}</span>
        <span class="event-message">{{ event.message }}</span>
        <span class="event-time">{{ new Date(event.timestamp).toLocaleTimeString() }}</span>
      </div>
    </div>
    <div v-else class="empty-state">
      <p>{{ t('common.noEvents') }}</p>
    </div>
    <TrafficMonitor />
  </div>
</template>

<style scoped>
.view-container {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.event-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 400px;
  overflow-y: auto;
}

.event-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 10px;
  border-radius: 4px;
  font-size: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border-color);
}

.event-type {
  font-weight: 600;
  color: var(--primary);
  min-width: 100px;
  font-size: 11px;
}

.event-message { flex: 1; }
.event-time { color: var(--text-muted); min-width: 80px; text-align: right; }

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-muted);
}

.empty-state .hint {
  font-size: 12px;
  margin-top: 4px;
}
</style>
