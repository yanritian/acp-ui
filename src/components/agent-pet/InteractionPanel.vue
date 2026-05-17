<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  happinessLevel: number
  affinityScore: number
  petCount: number
  lastInteractionType: string
}>()

const emit = defineEmits<{
  (e: 'pet'): void
  (e: 'poke'): void
  (e: 'feed'): void
  (e: 'play'): void
}>()

// State
const isInteracting = ref<string | null>(null)

// Computed
const happinessColor = computed(() => {
  if (props.happinessLevel < 30) return '#ef4444'
  if (props.happinessLevel < 60) return '#f59e0b'
  return '#10b981'
})

const affinityColor = computed(() => {
  if (props.affinityScore < 30) return '#6b7280'
  if (props.affinityScore < 60) return '#3b82f6'
  return '#8b5cf6'
})

const interactionHistory = computed(() => {
  return {
    pet: props.petCount,
    lastType: props.lastInteractionType
  }
})

// Methods
function handleInteraction(type: string) {
  isInteracting.value = type

  // 发射事件
  emit(type as any)

  // 动画反馈
  setTimeout(() => {
    isInteracting.value = null
  }, 300)
}

// 互动效果描述
const interactionEffects = computed(() => ({
  pet: { icon: '👋', effect: t('interactionPanel.petEffect'), description: t('interactionPanel.pet') },
  poke: { icon: '👉', effect: t('interactionPanel.pokeEffect'), description: t('interactionPanel.poke') },
  feed: { icon: '🍖', effect: t('interactionPanel.feedEffect'), description: t('interactionPanel.feed') },
  play: { icon: '🎾', effect: t('interactionPanel.playEffect'), description: t('interactionPanel.play') }
}))
</script>

<template>
  <div class="interaction-panel">
    <!-- Happiness & Affinity bars -->
    <div class="status-bars">
      <div class="bar-item">
        <div class="bar-header">
          <span class="bar-icon">😊</span>
          <span class="bar-label">{{ t('interactionPanel.happiness') }}</span>
          <span class="bar-value">{{ happinessLevel }}%</span>
        </div>
        <div class="bar-track">
          <div
            class="bar-fill"
            :style="{ width: `${happinessLevel}%`, backgroundColor: happinessColor }"
          />
        </div>
      </div>

      <div class="bar-item">
        <div class="bar-header">
          <span class="bar-icon">💜</span>
          <span class="bar-label">{{ t('interactionPanel.affinity') }}</span>
          <span class="bar-value">{{ affinityScore }}%</span>
        </div>
        <div class="bar-track">
          <div
            class="bar-fill"
            :style="{ width: `${affinityScore}%`, backgroundColor: affinityColor }"
          />
        </div>
      </div>
    </div>

    <!-- Interaction buttons -->
    <div class="interaction-buttons">
      <div class="buttons-label">{{ t('interactionPanel.interactionActions') }}</div>
      <div class="buttons-grid">
        <button
          v-for="(effect, type) in interactionEffects"
          :key="type"
          :class="['interact-btn', type, { active: isInteracting === type }]"
          @click="handleInteraction(type)"
        >
          <span class="btn-icon">{{ effect.icon }}</span>
          <span class="btn-name">{{ effect.description }}</span>
          <span class="btn-effect">{{ effect.effect }}</span>
        </button>
      </div>
    </div>

    <!-- Interaction history -->
    <div class="interaction-history">
      <div class="history-header">
        <span class="history-icon">📊</span>
        <span class="history-label">{{ t('interactionPanel.interactionStats') }}</span>
      </div>
      <div class="history-stats">
        <div class="history-item">
          <span class="history-stat-icon">👋</span>
          <span class="history-stat-name">{{ t('interactionPanel.petCount') }}</span>
          <span class="history-stat-value">{{ interactionHistory.pet }}</span>
        </div>
        <div class="history-item">
          <span class="history-stat-icon">🕐</span>
          <span class="history-stat-name">{{ t('interactionPanel.lastInteraction') }}</span>
          <span class="history-stat-value">{{ interactionHistory.lastType || t('interactionPanel.none') }}</span>
        </div>
      </div>
    </div>

    <!-- Interaction tips -->
    <div class="interaction-tips">
      <div class="tip-icon">💡</div>
      <div class="tip-text">
        {{ t('interactionPanel.interactionTip') }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.interaction-panel {
  padding: 16px;
  background: rgba(26, 26, 46, 0.6);
  border-radius: 12px;
  border: 1px solid #3a3a5a;
}

.status-bars {
  margin-bottom: 16px;
}

.bar-item {
  margin-bottom: 12px;
}

.bar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.bar-icon {
  font-size: 14px;
}

.bar-label {
  font-size: 12px;
  color: #8b8b9b;
  flex: 1;
}

.bar-value {
  font-size: 12px;
  color: #e0e0e0;
  font-weight: 600;
  font-family: 'Monaco', monospace;
}

.bar-track {
  height: 8px;
  background: rgba(139, 139, 155, 0.2);
  border-radius: 4px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.5s ease, backgroundColor 0.3s ease;
}

.interaction-buttons {
  margin-bottom: 16px;
}

.buttons-label {
  font-size: 11px;
  color: #6b7280;
  margin-bottom: 8px;
}

.buttons-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.interact-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 8px;
  background: rgba(26, 26, 46, 0.5);
  border: 2px solid #3a3a5a;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.interact-btn:hover {
  background: rgba(58, 58, 90, 0.5);
  transform: translateY(-2px);
}

.interact-btn.active {
  transform: scale(1.05);
  animation: btn-pulse 0.3s ease;
}

@keyframes btn-pulse {
  0%, 100% { transform: scale(1.05); }
  50% { transform: scale(1.1); }
}

.interact-btn.pet:hover { border-color: #10b981; }
.interact-btn.poke:hover { border-color: #f59e0b; }
.interact-btn.feed:hover { border-color: #3b82f6; }
.interact-btn.play:hover { border-color: #8b5cf6; }

.btn-icon {
  font-size: 24px;
  margin-bottom: 4px;
}

.btn-name {
  font-size: 12px;
  color: #e0e0e0;
  font-weight: 500;
}

.btn-effect {
  font-size: 10px;
  color: #6b7280;
}

.interaction-history {
  padding: 12px;
  background: rgba(58, 58, 90, 0.2);
  border-radius: 8px;
  margin-bottom: 12px;
}

.history-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.history-icon {
  font-size: 14px;
}

.history-label {
  font-size: 11px;
  color: #8b8b9b;
}

.history-stats {
  display: flex;
  gap: 16px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.history-stat-icon {
  font-size: 12px;
}

.history-stat-name {
  font-size: 10px;
  color: #6b7280;
}

.history-stat-value {
  font-size: 12px;
  color: #e0e0e0;
  font-weight: 600;
}

.interaction-tips {
  display: flex;
  gap: 8px;
  padding: 10px;
  background: rgba(139, 92, 246, 0.1);
  border: 1px solid rgba(139, 92, 246, 0.2);
  border-radius: 8px;
}

.tip-icon {
  font-size: 14px;
}

.tip-text {
  font-size: 11px;
  color: #8b8b9b;
  line-height: 1.4;
}
</style>