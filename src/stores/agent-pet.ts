import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type {
  AgentPet,
  AgentEmotionType,
  AgentActivityType
} from '@/lib/agent-runtime/realtime-progress-types'
import { LEVEL_CONFIG, EMOTION_EXPRESSIONS } from '@/lib/agent-runtime/realtime-progress-types'

export const useAgentPetStore = defineStore('agent-pet', () => {
  // ===== State =====
  const pets = ref<Map<string, AgentPet>>(new Map())
  const activePetId = ref<string | null>(null)

  // ===== Computed =====
  const activePet = computed(() => {
    if (!activePetId.value) return null
    return pets.value.get(activePetId.value)
  })

  const allPets = computed(() => {
    return Array.from(pets.value.values())
  })

  const happyPets = computed(() => {
    return allPets.value.filter(p => p.emotion.current === 'happy')
  })

  const focusedPets = computed(() => {
    return allPets.value.filter(p => p.emotion.current === 'focused')
  })

  const highLevelPets = computed(() => {
    return allPets.value.filter(p => p.growth.level >= 30)
  })

  // ===== Methods =====

  // 创建新宠物
  function createPet(agentId: string, agentName: string, petType: AgentPet['appearance']['type'] = 'cat') {
    const pet: AgentPet = {
      id: `pet-${agentId}`,
      agentId,
      name: agentName,
      appearance: {
        type: petType,
        color: EMOTION_EXPRESSIONS.focused.color,
        accessories: []
      },
      emotion: {
        current: 'focused',
        intensity: 50,
        lastChangeTime: Date.now(),
        transitionDuration: 300
      },
      growth: {
        level: 1,
        experience: 0,
        experienceToNextLevel: LEVEL_CONFIG.baseExperience,
        completedTasks: 0,
        successRate: 100,
        achievements: []
      },
      personality: {
        name: agentName,
        traits: ['careful', 'thorough'],
        workStyle: 'methodical',
        preferences: ['coding', 'testing']
      },
      interaction: {
        lastPetTime: 0,
        happinessLevel: 50,
        affinityScore: 30,
        petCount: 0,
        lastInteractionType: 'pet'
      },
      animation: {
        currentAnimation: 'idle',
        animationStartTime: Date.now(),
        animationDuration: 0,
        isAnimating: false
      }
    }
    pets.value.set(agentId, pet)
  }

  // 移除宠物
  function removePet(agentId: string) {
    pets.value.delete(agentId)
    if (activePetId.value === agentId) {
      activePetId.value = null
    }
  }

  // 设置活跃宠物
  function setActivePet(agentId: string) {
    if (pets.value.has(agentId)) {
      activePetId.value = agentId
    }
  }

  // 更新情绪状态
  function updateEmotion(agentId: string, emotion: AgentEmotionType, intensity?: number) {
    const pet = pets.value.get(agentId)
    if (!pet) return

    pet.emotion.current = emotion
    pet.emotion.lastChangeTime = Date.now()
    if (intensity !== undefined) {
      pet.emotion.intensity = Math.min(100, Math.max(0, intensity))
    }
    pet.appearance.color = EMOTION_EXPRESSIONS[emotion].color

    pets.value.set(agentId, { ...pet })
  }

  // 根据活动类型自动调整情绪
  function autoAdjustEmotion(agentId: string, activity: AgentActivityType) {
    const emotionMap: Record<AgentActivityType, AgentEmotionType> = {
      thinking: 'focused',
      executing: 'focused',
      outputting: 'happy',
      waiting: 'bored',
      idle: 'happy',
      error: 'confused'
    }
    updateEmotion(agentId, emotionMap[activity])
  }

  // 添加经验值
  function addExperience(agentId: string, amount: number) {
    const pet = pets.value.get(agentId)
    if (!pet) return

    pet.growth.experience += amount

    // 检查是否升级
    while (pet.growth.experience >= pet.growth.experienceToNextLevel) {
      pet.growth.experience -= pet.growth.experienceToNextLevel
      pet.growth.level++

      // 计算下一级所需经验
      pet.growth.experienceToNextLevel = Math.floor(
        LEVEL_CONFIG.baseExperience * Math.pow(LEVEL_CONFIG.experienceMultiplier, pet.growth.level - 1)
      )

      // 解锁成就
      if (pet.growth.level === 10) {
        unlockAchievement(agentId, 'ten_tasks')
      }
      if (pet.growth.level === 50) {
        unlockAchievement(agentId, 'fifty_tasks')
      }
      if (pet.growth.level === 100) {
        unlockAchievement(agentId, 'hundred_tasks')
      }
    }

    pets.value.set(agentId, { ...pet })
  }

  // 记录任务完成
  function recordTaskComplete(agentId: string, duration: number, success: boolean) {
    const pet = pets.value.get(agentId)
    if (!pet) return

    if (success) {
      pet.growth.completedTasks++
      pet.growth.successRate = (
        (pet.growth.successRate * (pet.growth.completedTasks - 1) + 100) /
        pet.growth.completedTasks
      )

      // 添加经验值
      const baseXP = 20
      const durationBonus = duration < 30000 ? 10 : 0 // 30秒内完成额外奖励
      addExperience(agentId, baseXP + durationBonus)

      // 解锁成就
      if (pet.growth.completedTasks === 1) {
        unlockAchievement(agentId, 'first_task')
      }
      if (pet.growth.completedTasks === 10) {
        unlockAchievement(agentId, 'ten_tasks')
      }
      if (pet.growth.completedTasks === 50) {
        unlockAchievement(agentId, 'fifty_tasks')
      }
      if (pet.growth.completedTasks === 100) {
        unlockAchievement(agentId, 'hundred_tasks')
      }

      // 快速执行成就 (简化条件：完成任务数>=10)
      if (pet.growth.completedTasks >= 10) {
        unlockAchievement(agentId, 'fast_executor')
      }
    } else {
      pet.growth.successRate = (
        (pet.growth.successRate * pet.growth.completedTasks) /
        (pet.growth.completedTasks + 1)
      )
    }

    pets.value.set(agentId, { ...pet })
  }

  // 解锁成就
  function unlockAchievement(agentId: string, achievementId: string) {
    const pet = pets.value.get(agentId)
    if (!pet) return

    if (!pet.growth.achievements.includes(achievementId)) {
      pet.growth.achievements.push(achievementId)

      // 成就解锁奖励经验
      const achievement = LEVEL_CONFIG.achievements[achievementId as keyof typeof LEVEL_CONFIG.achievements]
      if (achievement) {
        addExperience(agentId, 50)
      }

      pets.value.set(agentId, { ...pet })
    }
  }

  // 处理互动
  function handleInteraction(agentId: string, type: AgentPet['interaction']['lastInteractionType']) {
    const pet = pets.value.get(agentId)
    if (!pet) return

    const now = Date.now()

    // 更新互动统计
    pet.interaction.lastPetTime = now
    pet.interaction.lastInteractionType = type

    // 根据互动类型更新快乐度和亲密度
    const effects: Record<string, { happiness: number; affinity: number }> = {
      pet: { happiness: 5, affinity: 2 },
      poke: { happiness: -2, affinity: 0 },
      feed: { happiness: 10, affinity: 5 },
      play: { happiness: 15, affinity: 10 }
    }

    const effect = effects[type] || { happiness: 0, affinity: 0 }
    pet.interaction.happinessLevel = Math.min(100, Math.max(0, pet.interaction.happinessLevel + effect.happiness))
    pet.interaction.affinityScore = Math.min(100, Math.max(0, pet.interaction.affinityScore + effect.affinity))

    if (type === 'pet') {
      pet.interaction.petCount++
    }

    // 根据互动类型触发情绪变化
    const emotionMap: Record<string, AgentEmotionType> = {
      pet: 'happy',
      poke: 'confused',
      feed: 'happy',
      play: 'excited'
    }
    updateEmotion(agentId, emotionMap[type], Math.min(100, pet.emotion.intensity + 10))

    pets.value.set(agentId, { ...pet })

    // 返回更新后的状态
    return {
      happiness: pet.interaction.happinessLevel,
      affinity: pet.interaction.affinityScore,
      emotion: pet.emotion.current
    }
  }

  // 设置动画状态
  function setAnimation(agentId: string, animation: AgentPet['animation']['currentAnimation']) {
    const pet = pets.value.get(agentId)
    if (!pet) return

    pet.animation.currentAnimation = animation
    pet.animation.animationStartTime = Date.now()
    pet.animation.isAnimating = true

    // 设置动画持续时间
    const durations: Record<string, number> = {
      idle: 0,
      blink: 200,
      think: 800,
      happy: 600,
      confused: 500,
      tired: 400,
      working: 1000,
      levelup: 1500,
      celebrate: 800
    }
    pet.animation.animationDuration = durations[animation] || 500

    pets.value.set(agentId, { ...pet })
  }

  // 停止动画
  function stopAnimation(agentId: string) {
    const pet = pets.value.get(agentId)
    if (!pet) return

    pet.animation.isAnimating = false
    pet.animation.currentAnimation = 'idle'

    pets.value.set(agentId, { ...pet })
  }

  // 初始化模拟数据（演示用）
  function initializeMockPets() {
    createPet('planner-001', 'Planner', 'cat')
    createPet('architect-001', 'Architect', 'robot')
    createPet('tdd-guide-001', 'TDD Guide', 'dog')
    createPet('code-reviewer-001', 'Code Reviewer', 'dragon')

    setActivePet('planner-001')

    // 设置一些初始状态
    updateEmotion('planner-001', 'focused', 60)
    updateEmotion('architect-001', 'happy', 80)
  }

  return {
    // State
    pets,
    activePetId,

    // Computed
    activePet,
    allPets,
    happyPets,
    focusedPets,
    highLevelPets,

    // Methods
    createPet,
    removePet,
    setActivePet,
    updateEmotion,
    autoAdjustEmotion,
    addExperience,
    recordTaskComplete,
    unlockAchievement,
    handleInteraction,
    setAnimation,
    stopAnimation,
    initializeMockPets
  }
})