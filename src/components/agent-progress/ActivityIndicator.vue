<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import type { AgentActivityType } from '@/lib/agent-runtime/realtime-progress-types'
import { ACTIVITY_INDICATORS } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  type: AgentActivityType
  progress?: number
  description?: string
}>()

// 动画状态
const isPulsing = ref(false)
const pulseInterval = ref<number | null>(null)

// Computed
const indicator = computed(() => ACTIVITY_INDICATORS[props.type])
const progressWidth = computed(() => `${props.progress || 0}%`)
const indicatorDescription = computed(() => t(indicator.value.descriptionKey))

// 动画方法
function startPulseAnimation() {
  if (indicator.value.animation === 'pulse') {
    isPulsing.value = true
    pulseInterval.value = window.setInterval(() => {
      isPulsing.value = !isPulsing.value
    }, 600)
  }
}

function stopPulseAnimation() {
  if (pulseInterval.value) {
    clearInterval(pulseInterval.value)
    pulseInterval.value = null
  }
  isPulsing.value = false
}

// Lifecycle
onMounted(() => {
  startPulseAnimation()
})

onUnmounted(() => {
  stopPulseAnimation()
})

// Watch for type changes
watch(() => props.type, () => {
  stopPulseAnimation()
  startPulseAnimation()
})
</script>

<template>
  <div class="activity-indicator">
    <div
      :class="['indicator-badge', `status-${type}`, { pulsing: isPulsing }]"
      :style="{ backgroundColor: indicator.color }"
    >
      <span class="indicator-icon">{{ indicator.icon }}</span>
      <span class="indicator-text">{{ indicatorDescription }}</span>
    </div>

    <!-- Progress bar for executing -->
    <div v-if="type === 'executing' && progress" class="progress-container">
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: progressWidth, backgroundColor: indicator.color }"
        />
      </div>
      <span class="progress-text">{{ progress }}%</span>
    </div>

    <!-- Additional description -->
    <div v-if="description" class="activity-description">
      {{ description }}
    </div>
  </div>
</template>

<script lang="ts">
import { watch } from 'vue'
</script>

<style scoped>
.activity-indicator {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.indicator-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 8px;
  color: white;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.indicator-badge.pulsing {
  animation: pulse 0.6s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.05);
  }
}

.indicator-badge.status-error {
  animation: shake 0.3s ease-in-out;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-5px); }
  75% { transform: translateX(5px); }
}

.indicator-icon {
  font-size: 18px;
}

.indicator-text {
  font-size: 13px;
}

.progress-container {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 200px;
}

.progress-bar {
  width: 160px;
  height: 8px;
  background: rgba(139, 139, 155, 0.2);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 12px;
  color: #e0e0e0;
  font-family: 'Monaco', monospace;
  min-width: 32px;
}

.activity-description {
  font-size: 12px;
  color: #8b8b9b;
  max-width: 300px;
  text-align: right;
  word-break: break-word;
}
</style>