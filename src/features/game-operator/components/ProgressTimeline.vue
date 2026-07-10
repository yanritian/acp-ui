<script setup lang="ts">
import type { OperatorEvent } from '@/types/operator'

const props = defineProps<{
  events: OperatorEvent[]
}>()

const levelColors: Record<string, string> = {
  info: '#3B82F6',
  warning: '#F59E0B',
  error: '#EF4444',
  debug: '#6B7280',
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('zh-CN', { hour12: false })
}
</script>

<template>
  <div class="progress-timeline">
    <h3>Progress Timeline</h3>
    <div v-if="events.length === 0" class="empty-state">
      No events yet
    </div>
    <div v-else class="event-list">
      <div
        v-for="event in events"
        :key="event.event_id"
        class="event-item"
        :class="[`level-${event.level}`]"
      >
        <div class="event-timestamp">
          {{ formatTimestamp(event.timestamp) }}
        </div>
        <div
          class="event-indicator"
          :style="{ background: levelColors[event.level] }"
        ></div>
        <div class="event-content">
          <div class="event-title">{{ event.title }}</div>
          <div v-if="event.message" class="event-message">
            {{ event.message }}
          </div>
          <div class="event-meta">
            <span class="event-type">{{ event.type }}</span>
            <span class="event-source">{{ event.source }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.progress-timeline {
  height: auto;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.progress-timeline h3 {
  margin-bottom: 1rem;
  font-size: 1.1rem;
}

.empty-state {
  padding: 2rem;
  text-align: center;
  color: var(--text-muted);
}

.event-list {
  flex: 1;
  min-width: 0;
  overflow-y: visible;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.event-item {
  display: flex;
  min-width: 0;
  gap: 0.75rem;
  padding: 0.75rem;
  background: var(--bg-main);
  border-radius: 4px;
  border-left: 3px solid var(--border-color);
}

.event-item.level-error {
  border-left-color: #EF4444;
}

.event-item.level-warning {
  border-left-color: #F59E0B;
}

.event-timestamp {
  font-size: 0.85rem;
  color: var(--text-muted);
  font-family: monospace;
  min-width: 80px;
}

.event-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-top: 0.35rem;
  flex-shrink: 0;
}

.event-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.event-title {
  font-weight: 500;
  overflow-wrap: anywhere;
}

.event-message {
  font-size: 0.9rem;
  color: var(--text-secondary);
  overflow-wrap: anywhere;
}

.event-meta {
  display: flex;
  flex-wrap: wrap;
  min-width: 0;
  gap: 0.5rem;
  font-size: 0.8rem;
  color: var(--text-muted);
}

.event-type {
  font-family: monospace;
  overflow-wrap: anywhere;
}

</style>
