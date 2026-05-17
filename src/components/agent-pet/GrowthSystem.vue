<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import { LEVEL_CONFIG } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  level: number
  experience: number
  experienceToNextLevel: number
  completedTasks: number
  successRate: number
  achievements: string[]
}>()

const emit = defineEmits<{
  (e: 'achievement_click', achievementId: string): void
}>()

// State
const showLevelUpAnimation = ref(false)
const showAchievementPopup = ref<string | null>(null)

// Computed
const experienceProgress = computed(() => {
  return Math.min((props.experience / props.experienceToNextLevel) * 100, 100)
})

const levelBadgeColor = computed(() => {
  // 根据等级返回不同颜色
  if (props.level < 10) return '#6B7280'
  if (props.level < 30) return '#3B82F6'
  if (props.level < 50) return '#10B981'
  if (props.level < 80) return '#F59E0B'
  return '#8B5CF6' // 80+
})

const levelTitle = computed(() => {
  // 根据等级返回称号
  if (props.level < 10) return t('growthSystem.levelNovice')
  if (props.level < 30) return t('growthSystem.levelApprentice')
  if (props.level < 50) return t('growthSystem.levelSkilled')
  if (props.level < 80) return t('growthSystem.levelExpert')
  return t('growthSystem.levelMaster')
})

const unlockedAchievements = computed(() => {
  return Object.entries(LEVEL_CONFIG.achievements)
    .filter(([id, achievement]) => props.achievements.includes(id))
    .map(([id, achievement]) => ({ id, ...achievement }))
})

const nextAchievements = computed(() => {
  return Object.entries(LEVEL_CONFIG.achievements)
    .filter(([id]) => !props.achievements.includes(id))
    .slice(0, 3)
    .map(([id, achievement]) => ({ id, ...achievement }))
})

// Watch for level changes
watch(() => props.level, (newLevel, oldLevel) => {
  if (newLevel > oldLevel) {
    showLevelUpAnimation.value = true
    setTimeout(() => {
      showLevelUpAnimation.value = false
    }, 2000)
  }
})

// Watch for new achievements
watch(() => props.achievements, (newAchievements, oldAchievements) => {
  const newAchievement = newAchievements.find(a => !oldAchievements.includes(a))
  if (newAchievement) {
    showAchievementPopup.value = newAchievement
    setTimeout(() => {
      showAchievementPopup.value = null
    }, 3000)
  }
}, { deep: true })

// Methods
function handleAchievementClick(achievementId: string) {
  emit('achievement_click', achievementId)
}

// 格式化数字
function formatNumber(num: number): string {
  if (num >= 1000) return `${(num / 1000).toFixed(1)}k`
  return String(num)
}
</script>

<template>
  <div class="growth-system">
    <!-- Level display -->
    <div class="level-section">
      <div :class="['level-badge', { levelup: showLevelUpAnimation }]" :style="{ backgroundColor: levelBadgeColor }">
        <div class="level-number">Lv.{{ level }}</div>
        <div class="level-title">{{ levelTitle }}</div>
      </div>

      <!-- Level up animation -->
      <div v-if="showLevelUpAnimation" class="levelup-overlay">
        <div class="levelup-content">
          <div class="levelup-icon">🎉</div>
          <div class="levelup-text">{{ t('growthSystem.levelUp') }}</div>
          <div class="levelup-new-level">Lv.{{ level }}</div>
        </div>
      </div>
    </div>

    <!-- Experience bar -->
    <div class="experience-section">
      <div class="experience-header">
        <span class="experience-label">{{ t('growthSystem.experience') }}</span>
        <span class="experience-value">{{ formatNumber(experience) }} / {{ formatNumber(experienceToNextLevel) }}</span>
      </div>
      <div class="experience-bar">
        <div
          class="experience-fill"
          :style="{ width: `${experienceProgress}%`, backgroundColor: levelBadgeColor }"
        />
      </div>
      <div class="experience-progress">{{ experienceProgress.toFixed(1) }}%</div>
    </div>

    <!-- Stats -->
    <div class="stats-section">
      <div class="stat-row">
        <div class="stat-item">
          <span class="stat-icon">✅</span>
          <span class="stat-name">{{ t('growthSystem.completedTasks') }}</span>
          <span class="stat-value">{{ completedTasks }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-icon">📊</span>
          <span class="stat-name">{{ t('growthSystem.successRate') }}</span>
          <span class="stat-value">{{ successRate }}%</span>
        </div>
      </div>
    </div>

    <!-- Achievements -->
    <div class="achievements-section">
      <div class="achievements-header">
        <span class="achievements-label">{{ t('growthSystem.achievements') }}</span>
        <span class="achievements-count">{{ achievements.length }} / {{ Object.keys(LEVEL_CONFIG.achievements).length }}</span>
      </div>

      <!-- Unlocked achievements -->
      <div v-if="unlockedAchievements.length > 0" class="unlocked-achievements">
        <div
          v-for="achievement in unlockedAchievements"
          :key="achievement.id"
          class="achievement-badge unlocked"
          @click="handleAchievementClick(achievement.id)"
        >
          <span class="achievement-icon">{{ achievement.icon }}</span>
          <span class="achievement-name">{{ achievement.name }}</span>
        </div>
      </div>

      <!-- Next achievements (locked) -->
      <div v-if="nextAchievements.length > 0" class="locked-achievements">
        <div class="locked-label">{{ t('growthSystem.nextGoal') }}</div>
        <div
          v-for="achievement in nextAchievements"
          :key="achievement.id"
          class="achievement-badge locked"
        >
          <span class="achievement-icon">🔒</span>
          <span class="achievement-name">{{ achievement.name }}</span>
          <span class="achievement-requirement">({{ achievement.requirement }})</span>
        </div>
      </div>
    </div>

    <!-- Achievement popup -->
    <div v-if="showAchievementPopup" class="achievement-popup">
      <div class="popup-content">
        <div class="popup-icon">
          {{ LEVEL_CONFIG.achievements[showAchievementPopup]?.icon }}
        </div>
        <div class="popup-text">{{ t('growthSystem.unlockAchievement') }}</div>
        <div class="popup-name">
          {{ LEVEL_CONFIG.achievements[showAchievementPopup]?.name }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.growth-system {
  padding: 16px;
  background: rgba(26, 26, 46, 0.6);
  border-radius: 12px;
  border: 1px solid #3a3a5a;
}

.level-section {
  position: relative;
  margin-bottom: 16px;
}

.level-badge {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 24px;
  border-radius: 12px;
  color: white;
  transition: all 0.3s ease;
}

.level-badge.levelup {
  animation: levelup-pulse 2s ease-in-out;
}

@keyframes levelup-pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.1); box-shadow: 0 0 30px currentColor; }
}

.level-number {
  font-size: 24px;
  font-weight: 700;
}

.level-title {
  font-size: 12px;
  opacity: 0.8;
}

.levelup-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.8);
  z-index: 1000;
  animation: fade-in 0.3s ease;
}

.levelup-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  animation: scale-in 0.5s ease;
}

.levelup-icon {
  font-size: 64px;
}

.levelup-text {
  font-size: 20px;
  color: #10b981;
  font-weight: 600;
}

.levelup-new-level {
  font-size: 32px;
  color: white;
  font-weight: 700;
}

.experience-section {
  margin-bottom: 16px;
}

.experience-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.experience-label {
  font-size: 12px;
  color: #8b8b9b;
}

.experience-value {
  font-size: 12px;
  color: #e0e0e0;
  font-family: 'Monaco', monospace;
}

.experience-bar {
  height: 10px;
  background: rgba(139, 139, 155, 0.2);
  border-radius: 5px;
  overflow: hidden;
}

.experience-fill {
  height: 100%;
  border-radius: 5px;
  transition: width 0.5s ease, backgroundColor 0.3s ease;
}

.experience-progress {
  text-align: right;
  font-size: 11px;
  color: #8b8b9b;
  margin-top: 4px;
}

.stats-section {
  margin-bottom: 16px;
  padding: 12px;
  background: rgba(58, 58, 90, 0.2);
  border-radius: 8px;
}

.stat-row {
  display: flex;
  justify-content: space-around;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stat-icon {
  font-size: 16px;
}

.stat-name {
  font-size: 11px;
  color: #8b8b9b;
}

.stat-value {
  font-size: 14px;
  color: #e0e0e0;
  font-weight: 600;
}

.achievements-section {
  border-top: 1px solid rgba(58, 58, 90, 0.3);
  padding-top: 12px;
}

.achievements-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.achievements-label {
  font-size: 12px;
  color: #8b8b9b;
  font-weight: 500;
}

.achievements-count {
  font-size: 11px;
  color: #6b7280;
}

.unlocked-achievements,
.locked-achievements {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.achievement-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 11px;
  transition: all 0.2s ease;
}

.achievement-badge.unlocked {
  background: rgba(139, 92, 246, 0.2);
  border: 1px solid rgba(139, 92, 246, 0.3);
  cursor: pointer;
}

.achievement-badge.unlocked:hover {
  background: rgba(139, 92, 246, 0.3);
  transform: scale(1.05);
}

.achievement-badge.locked {
  background: rgba(107, 114, 128, 0.1);
  border: 1px solid rgba(107, 114, 128, 0.2);
  opacity: 0.6;
}

.locked-label {
  font-size: 10px;
  color: #6b7280;
  margin-bottom: 4px;
}

.achievement-icon {
  font-size: 14px;
}

.achievement-name {
  color: #e0e0e0;
  font-weight: 500;
}

.achievement-requirement {
  font-size: 10px;
  color: #6b7280;
}

.achievement-popup {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  padding: 20px;
  background: rgba(26, 26, 46, 0.95);
  border: 2px solid #8b5cf6;
  border-radius: 16px;
  z-index: 1000;
  animation: popup-appear 0.5s ease;
}

.popup-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.popup-icon {
  font-size: 48px;
}

.popup-text {
  font-size: 14px;
  color: #10b981;
  font-weight: 600;
}

.popup-name {
  font-size: 18px;
  color: #e0e0e0;
  font-weight: 700;
}

@keyframes popup-appear {
  from { opacity: 0; transform: translate(-50%, -50%) scale(0.5); }
  to { opacity: 1; transform: translate(-50%, -50%) scale(1); }
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scale-in {
  from { transform: scale(0.5); }
  to { transform: scale(1); }
}
</style>