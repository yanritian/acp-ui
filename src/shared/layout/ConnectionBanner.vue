<script setup lang="ts">
defineProps<{
  isReconnecting: boolean
  reconnectingAgentName: string
  error: string | null
  canManuallyReconnect: boolean
}>()

const emit = defineEmits<{
  (e: 'manual-reconnect'): void
  (e: 'clear-error'): void
}>()
</script>

<template>
  <!-- Reconnect banner takes priority over the error banner: while a
       reconnect is in progress we don't want a contradictory red
       "Connection lost" pill. -->
  <div v-if="isReconnecting" class="reconnect-banner">
    <span class="reconnect-spinner" aria-hidden="true"></span>
    <span class="reconnect-text">
      Reconnecting to <strong>{{ reconnectingAgentName }}</strong>...
    </span>
  </div>

  <!-- Error display (suppressed while reconnecting). -->
  <div v-else-if="error" class="error-banner">
    <span class="error-icon">⚠</span>
    <span class="error-text">{{ error }}</span>
    <button
      v-if="canManuallyReconnect"
      class="error-action"
      @click="emit('manual-reconnect')"
      title="Reconnect"
    >Reconnect</button>
    <button class="error-close" @click="emit('clear-error')" title="Dismiss">×</button>
  </div>
</template>

<style scoped>
.error-banner {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1rem;
  background: #fee;
  color: #c00;
  border-bottom: 1px solid #fcc;
}

.error-icon {
  flex-shrink: 0;
}

.error-text {
  flex: 1;
}

.error-close {
  flex-shrink: 0;
  padding: 0.25rem 0.5rem;
  border: none;
  background: transparent;
  color: #c00;
  font-size: 1.25rem;
  line-height: 1;
  cursor: pointer;
  opacity: 0.6;
  border-radius: 4px;
}

.error-close:hover {
  opacity: 1;
  background: rgba(204, 0, 0, 0.1);
}

/* Inline "Reconnect" affordance shown next to a stale error when we have a
   saved session we could reattach to. */
.error-action {
  flex-shrink: 0;
  padding: 0.25rem 0.6rem;
  margin-right: 0.25rem;
  border: 1px solid #c00;
  border-radius: 4px;
  background: transparent;
  color: #c00;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
}

.error-action:hover {
  background: rgba(204, 0, 0, 0.1);
}

/* Foreground-reconnect banner. Distinct visual style from the red error
   banner so users immediately read it as transient progress, not failure. */
.reconnect-banner {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.75rem 1rem;
  background: #e0f2fe;
  color: #0369a1;
  border-bottom: 1px solid #bae6fd;
}

.reconnect-text {
  flex: 1;
  font-size: 0.9rem;
}

.reconnect-text strong {
  font-weight: 600;
}

.reconnect-spinner {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: reconnect-spin 0.9s linear infinite;
}

@keyframes reconnect-spin {
  to { transform: rotate(360deg); }
}

@media (prefers-color-scheme: dark) {
  .reconnect-banner {
    background: #082f49;
    color: #7dd3fc;
    border-bottom-color: #0c4a6e;
  }
}

/* Banners sit at the very top of the main area on mobile. */
@media (max-width: 800px) {
  .reconnect-banner,
  .error-banner {
    padding-top: calc(0.75rem + env(safe-area-inset-top, 0px));
  }
}
</style>
