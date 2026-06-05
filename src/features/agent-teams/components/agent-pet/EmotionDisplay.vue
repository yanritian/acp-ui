<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { AgentEmotionType } from '@/lib/agent-runtime/realtime-progress-types'
import { EMOTION_EXPRESSIONS } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  emotion: AgentEmotionType
  intensity: number
  transitionDuration?: number
}>()

// State
const previousEmotion = ref<AgentEmotionType>(props.emotion)
const isTransitioning = ref(false)

// Computed
const currentConfig = computed(() => EMOTION_EXPRESSIONS[props.emotion])
const previousConfig = computed(() => EMOTION_EXPRESSIONS[previousEmotion.value])

const intensityStyle = computed(() => {
  // 根据强度调整颜色透明度
  const opacity = 0.3 + (props.intensity / 100) * 0.7
  return { opacity }
})

// Watch for emotion changes
watch(() => props.emotion, (newEmotion, oldEmotion) => {
  if (oldEmotion !== newEmotion) {
    previousEmotion.value = oldEmotion
    isTransitioning.value = true

    // 过渡动画持续时间
    const duration = props.transitionDuration || 300
    setTimeout(() => {
      isTransitioning.value = false
    }, duration)
  }
})

// 情绪描述映射
const emotionDescriptions = computed(() => ({
  happy: t('emotionDisplay.emotionHappy'),
  focused: t('emotionDisplay.emotionFocused'),
  confused: t('emotionDisplay.emotionConfused'),
  tired: t('emotionDisplay.emotionTired'),
  bored: t('emotionDisplay.emotionBored'),
  excited: t('emotionDisplay.emotionExcited')
}))
</script>

<template>
  <div class="emotion-display">
    <!-- Emotion icon -->
    <div :class="['emotion-icon-wrapper', { transitioning: isTransitioning }]">
      <div
        v-if="isTransitioning"
        class="emotion-icon previous"
        :style="{ color: previousConfig.color }"
      >
        {{ previousConfig.emoji }}
      </div>
      <div
        class="emotion-icon current"
        :style="{ color: currentConfig.color, ...intensityStyle }"
      >
        {{ currentConfig.emoji }}
      </div>
    </div>

    <!-- Emotion bar -->
    <div class="emotion-bar-container">
      <div class="emotion-bar-label">{{ t('emotionDisplay.emotionIntensity') }}</div>
      <div class="emotion-bar">
        <div
          class="emotion-bar-fill"
          :style="{ width: `${intensity}%`, backgroundColor: currentConfig.color }"
        />
      </div>
      <div class="emotion-bar-value">{{ intensity }}%</div>
    </div>

    <!-- Emotion description -->
    <div class="emotion-description" :style="{ color: currentConfig.color }">
      {{ emotionDescriptions[emotion] }}
    </div>

    <!-- Emotion history (optional) -->
    <div class="emotion-history">
      <span class="history-label">{{ t('emotionDisplay.recentEmotion') }}:</span>
      <span class="history-item" :style="{ color: previousConfig.color }">
        {{ previousConfig.emoji }} {{ previousEmotion }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.emotion-display {
  padding: 12px;
  background: rgba(26, 26, 46, 0.6);
  border-radius: 10px;
  border: 1px solid #3a3a5a;
}

.emotion-icon-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
  height: 50px;
  margin-bottom: 12px;
}

.emotion-icon-wrapper.transitioning .emotion-icon.current {
  animation: fade-in 0.3s ease-in-out;
}

.emotion-icon-wrapper.transitioning .emotion-icon.previous {
  animation: fade-out 0.3s ease-in-out;
}

.emotion-icon {
  font-size: 36px;
  position: absolute;
  transition: all 0.3s ease;
}

@keyframes fade-in {
  from { opacity: 0; transform: scale(0.8); }
  to { opacity: 1; transform: scale(1); }
}

@keyframes fade-out {
  from { opacity: 1; transform: scale(1); }
  to { opacity: 0; transform: scale(0.8); }
}

.emotion-bar-container {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.emotion-bar-label {
  font-size: 11px;
  color: #8b8b9b;
  min-width: 60px;
}

.emotion-bar {
  flex: 1;
  height: 8px;
  background: rgba(139, 139, 155, 0.2);
  border-radius: 4px;
  overflow: hidden;
}

.emotion-bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease, backgroundColor 0.3s ease;
}

.emotion-bar-value {
  font-size: 12px;
  color: #e0e0e0;
  font-family: 'Monaco', monospace;
  min-width: 40px;
  text-align: right;
}

.emotion-description {
  font-size: 13px;
  font-weight: 500;
  text-align: center;
  margin-bottom: 8px;
}

.emotion-history {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid rgba(58, 58, 90, 0.3);
}

.history-label {
  font-size: 10px;
  color: #8b8b9b;
}

.history-item {
  font-size: 11px;
  font-weight: 500;
}
</style>