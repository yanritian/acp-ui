<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { AgentActivityType, AgentEmotionType } from '@/lib/agent-runtime/realtime-progress-types'
import { AGENT_PET_ANIMATIONS, EMOTION_EXPRESSIONS } from '@/lib/agent-runtime/realtime-progress-types'
import { useI18n } from '@/locales'

const { t } = useI18n()

const props = defineProps<{
  agentId: string
  agentName: string
  currentActivity: AgentActivityType
  emotion: AgentEmotionType
  petType?: 'cat' | 'dog' | 'robot' | 'dragon' | 'custom'
  customImageUrl?: string
}>()

const emit = defineEmits<{
  (e: 'interaction', type: string): void
}>()

// State
const isAnimating = ref(false)
const currentAnimation = ref('idle')
const animationStartTime = ref(0)
const petScale = ref(1)

// Computed
const petEmoji = computed(() => {
  // 根据宠物类型和情绪返回表情
  const emotionConfig = EMOTION_EXPRESSIONS[props.emotion]
  return emotionConfig.emoji
})

const petColor = computed(() => {
  return EMOTION_EXPRESSIONS[props.emotion].color
})

const animationClass = computed(() => {
  const anim = AGENT_PET_ANIMATIONS[currentAnimation.value as keyof typeof AGENT_PET_ANIMATIONS]
  return anim ? `anim-${currentAnimation.value}` : ''
})

const activityAnimation = computed(() => {
  // 根据活动类型决定动画
  const activityAnimations: Record<AgentActivityType, string> = {
    thinking: 'think',
    executing: 'working',
    outputting: 'happy',
    waiting: 'idle',
    idle: 'blink',
    error: 'confused'
  }
  return activityAnimations[props.currentActivity] || 'idle'
})

// Methods
function triggerAnimation(animationName: string) {
  const anim = AGENT_PET_ANIMATIONS[animationName as keyof typeof AGENT_PET_ANIMATIONS]
  if (!anim) return

  currentAnimation.value = animationName
  isAnimating.value = true
  animationStartTime.value = Date.now()

  // 动画结束后恢复idle
  setTimeout(() => {
    if (currentAnimation.value === animationName) {
      currentAnimation.value = 'idle'
      isAnimating.value = false
    }
  }, anim.duration)
}

function handlePet() {
  petScale.value = 1.2
  triggerAnimation('happy')
  emit('interaction', 'pet')

  setTimeout(() => {
    petScale.value = 1
  }, 200)
}

function handlePoke() {
  triggerAnimation('confused')
  emit('interaction', 'poke')
}

function handleFeed() {
  triggerAnimation('celebrate')
  emit('interaction', 'feed')
}

function handlePlay() {
  triggerAnimation('levelup')
  emit('interaction', 'play')
}

// Watch for activity changes
watch(() => props.currentActivity, (newActivity) => {
  triggerAnimation(activityAnimation.value)
})

// Watch for emotion changes
watch(() => props.emotion, () => {
  // 情绪变化时触发眨眼动画
  if (!isAnimating.value) {
    triggerAnimation('blink')
  }
})

// Lifecycle - 定时眨眼
let blinkInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  // 每5秒自动眨眼（如果没有其他动画）
  blinkInterval = setInterval(() => {
    if (!isAnimating.value && props.currentActivity === 'idle') {
      triggerAnimation('blink')
    }
  }, 5000)
})

onUnmounted(() => {
  if (blinkInterval) {
    clearInterval(blinkInterval)
  }
})
</script>

<template>
  <div class="agent-pet-avatar">
    <!-- Pet image/emoji -->
    <div
      :class="['pet-container', animationClass]"
      :style="{ transform: `scale(${petScale})`, borderColor: petColor }"
      @click="handlePet"
    >
      <div class="pet-emoji">{{ petEmoji }}</div>

      <!-- Activity indicator glow -->
      <div
        v-if="currentActivity !== 'idle'"
        class="activity-glow"
        :style="{ backgroundColor: petColor }"
      />

      <!-- Level badge (placeholder) -->
      <div class="level-badge">Lv.1</div>
    </div>

    <!-- Interaction buttons -->
    <div class="interaction-buttons">
      <button class="interact-btn pet-btn" @click.stop="handlePet" :title="t('agentPet.pet')">
        👋
      </button>
      <button class="interact-btn poke-btn" @click.stop="handlePoke" :title="t('agentPet.poke')">
        👉
      </button>
      <button class="interact-btn feed-btn" @click.stop="handleFeed" :title="t('agentPet.feed')">
        🍖
      </button>
      <button class="interact-btn play-btn" @click.stop="handlePlay" :title="t('agentPet.play')">
        🎾
      </button>
    </div>

    <!-- Pet name -->
    <div class="pet-name">{{ agentName }}</div>
  </div>
</template>

<style scoped>
.agent-pet-avatar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.pet-container {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 3px solid;
  border-radius: 50%;
  background: rgba(26, 26, 46, 0.6);
  cursor: pointer;
  transition: transform 0.2s ease, border-color 0.3s ease;
  position: relative;
  overflow: hidden;
}

.pet-container:hover {
  transform: scale(1.1);
}

.pet-emoji {
  font-size: 40px;
  user-select: none;
  transition: all 0.3s ease;
}

/* Animations */
.pet-container.anim-idle .pet-emoji {
  animation: idle-float 2s ease-in-out infinite;
}

.pet-container.anim-blink .pet-emoji {
  animation: blink-eyes 0.2s ease-in-out;
}

.pet-container.anim-think .pet-emoji {
  animation: think-wobble 0.8s ease-in-out;
}

.pet-container.anim-happy .pet-emoji {
  animation: happy-bounce 0.6s ease-in-out;
}

.pet-container.anim-confused .pet-emoji {
  animation: confused-shake 0.5s ease-in-out;
}

.pet-container.anim-tired .pet-emoji {
  animation: tired-droop 0.4s ease-in-out;
}

.pet-container.anim-working .pet-emoji {
  animation: working-pulse 1s ease-in-out infinite;
}

.pet-container.anim-levelup .pet-emoji {
  animation: levelup-grow 1.5s ease-in-out;
}

.pet-container.anim-celebrate .pet-emoji {
  animation: celebrate-spin 0.8s ease-in-out;
}

@keyframes idle-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

@keyframes blink-eyes {
  0%, 100% { transform: scaleY(1); }
  50% { transform: scaleY(0.1); }
}

@keyframes think-wobble {
  0%, 100% { transform: rotate(0deg); }
  25% { transform: rotate(-10deg); }
  75% { transform: rotate(10deg); }
}

@keyframes happy-bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-15px); }
}

@keyframes confused-shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-8px); }
  75% { transform: translateX(8px); }
}

@keyframes tired-droop {
  0% { transform: translateY(0); }
  100% { transform: translateY(5px) scaleY(0.9); }
}

@keyframes working-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

@keyframes levelup-grow {
  0% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.5); opacity: 0.8; }
  100% { transform: scale(1); opacity: 1; }
}

@keyframes celebrate-spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.activity-glow {
  position: absolute;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  opacity: 0.2;
  animation: glow-pulse 1s ease-in-out infinite;
}

@keyframes glow-pulse {
  0%, 100% { opacity: 0.2; }
  50% { opacity: 0.4; }
}

.level-badge {
  position: absolute;
  bottom: -2px;
  right: -2px;
  padding: 2px 6px;
  background: rgba(139, 92, 246, 0.8);
  border-radius: 6px;
  font-size: 10px;
  color: white;
  font-weight: 600;
}

.interaction-buttons {
  display: flex;
  gap: 4px;
}

.interact-btn {
  width: 32px;
  height: 32px;
  padding: 4px;
  border: 1px solid #3a3a5a;
  border-radius: 8px;
  background: rgba(26, 26, 46, 0.6);
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
}

.interact-btn:hover {
  background: rgba(58, 58, 90, 0.6);
  transform: scale(1.1);
}

.interact-btn.pet-btn:hover { border-color: #10b981; }
.interact-btn.poke-btn:hover { border-color: #f59e0b; }
.interact-btn.feed-btn:hover { border-color: #3b82f6; }
.interact-btn.play-btn:hover { border-color: #8b5cf6; }

.pet-name {
  font-size: 11px;
  color: #8b8b9b;
  font-weight: 500;
}
</style>