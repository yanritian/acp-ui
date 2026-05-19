/// Agent宠物类型定义 (Phase 3)
/// 类人宠物概念的Agent拟人化

import 'agent_realtime_types.dart';

/// 宠物外观类型
enum PetAppearanceType {
  cat,
  dog,
  robot,
  dragon,
  custom,
}

/// 宠物外观
class PetAppearance {
  final PetAppearanceType type;
  final String color;
  final List<String> accessories;
  final String? customImageUrl;

  PetAppearance({
    required this.type,
    required this.color,
    required this.accessories,
    this.customImageUrl,
  });
}

/// 宠物情绪状态
class PetEmotion {
  final AgentEmotionType current;
  final int intensity; // 0-100
  final int lastChangeTime;
  final int transitionDuration;

  PetEmotion({
    required this.current,
    required this.intensity,
    required this.lastChangeTime,
    required this.transitionDuration,
  });
}

/// 宠物成长系统
class PetGrowth {
  final int level;
  final int experience;
  final int experienceToNextLevel;
  final int completedTasks;
  final double successRate;
  final List<String> achievements;

  PetGrowth({
    required this.level,
    required this.experience,
    required this.experienceToNextLevel,
    required this.completedTasks,
    required this.successRate,
    required this.achievements,
  });
}

/// 工作风格
enum WorkStyle {
  methodical,
  creative,
  aggressive,
  careful,
}

/// 宠物个性
class PetPersonality {
  final String name;
  final List<String> traits;
  final WorkStyle workStyle;
  final List<String> preferences;
  final String? catchphrase;

  PetPersonality({
    required this.name,
    required this.traits,
    required this.workStyle,
    required this.preferences,
    this.catchphrase,
  });
}

/// 互动类型
enum InteractionType {
  pet,
  poke,
  feed,
  play,
}

/// 宠物互动状态
class PetInteraction {
  final int lastPetTime;
  final int happinessLevel; // 0-100
  final int affinityScore; // 与用户的亲密度 0-100
  final int petCount;
  final InteractionType lastInteractionType;

  PetInteraction({
    required this.lastPetTime,
    required this.happinessLevel,
    required this.affinityScore,
    required this.petCount,
    required this.lastInteractionType,
  });
}

/// 动画类型
enum PetAnimationType {
  idle,
  blink,
  think,
  happy,
  confused,
  tired,
  working,
  levelup,
  celebrate,
}

/// 宠物动画状态
class PetAnimation {
  final PetAnimationType currentAnimation;
  final int animationStartTime;
  final int animationDuration;
  final bool isAnimating;

  PetAnimation({
    required this.currentAnimation,
    required this.animationStartTime,
    required this.animationDuration,
    required this.isAnimating,
  });
}

/// Agent宠物模型
class AgentPet {
  final String id;
  final String agentId;
  final String name;
  final String? avatarUrl;

  // 外观特征
  final PetAppearance appearance;

  // 情绪状态
  final PetEmotion emotion;

  // 成长系统
  final PetGrowth growth;

  // 个性化
  final PetPersonality personality;

  // 互动状态
  final PetInteraction interaction;

  // 动画状态
  final PetAnimation animation;

  AgentPet({
    required this.id,
    required this.agentId,
    required this.name,
    this.avatarUrl,
    required this.appearance,
    required this.emotion,
    required this.growth,
    required this.personality,
    required this.interaction,
    required this.animation,
  });
}

/// Agent宠物动画配置
const Map<PetAnimationType, PetAnimationConfig> petAnimations = {
  PetAnimationType.idle: PetAnimationConfig(
    frames: 1,
    duration: 0,
    expression: '😐',
    description: '平静状态',
  ),
  PetAnimationType.blink: PetAnimationConfig(
    frames: 2,
    duration: 200,
    expression: '😐',
    description: '眨眼动画',
  ),
  PetAnimationType.think: PetAnimationConfig(
    frames: 3,
    duration: 800,
    expression: '🤔',
    description: '思考动画',
  ),
  PetAnimationType.happy: PetAnimationConfig(
    frames: 4,
    duration: 600,
    expression: '😊',
    description: '开心动画',
  ),
  PetAnimationType.confused: PetAnimationConfig(
    frames: 3,
    duration: 500,
    expression: '😕',
    description: '困惑动画',
  ),
  PetAnimationType.tired: PetAnimationConfig(
    frames: 2,
    duration: 400,
    expression: '😫',
    description: '疲惫动画',
  ),
  PetAnimationType.working: PetAnimationConfig(
    frames: 3,
    duration: 1000,
    expression: '😐',
    description: '专注工作',
  ),
  PetAnimationType.levelup: PetAnimationConfig(
    frames: 6,
    duration: 1500,
    expression: '🎉',
    description: '升级动画',
  ),
  PetAnimationType.celebrate: PetAnimationConfig(
    frames: 5,
    duration: 800,
    expression: '✨',
    description: '庆祝动画',
  ),
};

/// 动画配置
class PetAnimationConfig {
  final int frames;
  final int duration;
  final String expression;
  final String description;

  const PetAnimationConfig({
    required this.frames,
    required this.duration,
    required this.expression,
    required this.description,
  });
}

/// 情绪表情映射
const Map<AgentEmotionType, EmotionExpression> emotionExpressions = {
  AgentEmotionType.happy: EmotionExpression(
    emoji: '😊',
    color: '#10B981',
    animation: 'bounce',
  ),
  AgentEmotionType.focused: EmotionExpression(
    emoji: '😐',
    color: '#3B82F6',
    animation: 'pulse',
  ),
  AgentEmotionType.confused: EmotionExpression(
    emoji: '😕',
    color: '#F59E0B',
    animation: 'shake',
  ),
  AgentEmotionType.tired: EmotionExpression(
    emoji: '😫',
    color: '#6B7280',
    animation: 'droop',
  ),
  AgentEmotionType.bored: EmotionExpression(
    emoji: '🙄',
    color: '#9CA3AF',
    animation: 'look-around',
  ),
  AgentEmotionType.excited: EmotionExpression(
    emoji: '🎉',
    color: '#8B5CF6',
    animation: 'jump',
  ),
};

/// 情绪表情
class EmotionExpression {
  final String emoji;
  final String color;
  final String animation;

  const EmotionExpression({
    required this.emoji,
    required this.color,
    required this.animation,
  });
}

/// 成长等级配置
class LevelConfig {
  static const int maxLevel = 100;
  static const int baseExperience = 100;
  static const double experienceMultiplier = 1.5;

  static const Map<String, AchievementConfig> achievements = {
    'first_task': AchievementConfig(
      name: '初出茅庐',
      description: '完成第一个任务',
      icon: '🌟',
      requirement: 1,
    ),
    'ten_tasks': AchievementConfig(
      name: '小有成就',
      description: '完成10个任务',
      icon: '⭐',
      requirement: 10,
    ),
    'fifty_tasks': AchievementConfig(
      name: '经验丰富',
      description: '完成50个任务',
      icon: '🏅',
      requirement: 50,
    ),
    'hundred_tasks': AchievementConfig(
      name: '大师级',
      description: '完成100个任务',
      icon: '🏆',
      requirement: 100,
    ),
    'perfect_streak': AchievementConfig(
      name: '完美执行',
      description: '连续10个任务成功',
      icon: '💎',
      requirement: 10,
    ),
    'fast_executor': AchievementConfig(
      name: '闪电执行',
      description: '平均执行时间<30秒',
      icon: '⚡',
      requirement: 30,
    ),
    'deep_thinker': AchievementConfig(
      name: '深度思考',
      description: '思考时间>5分钟',
      icon: '🧠',
      requirement: 300,
    ),
  };
}

/// 成就配置
class AchievementConfig {
  final String name;
  final String description;
  final String icon;
  final int requirement;

  AchievementConfig({
    required this.name,
    required this.description,
    required this.icon,
    required this.requirement,
  });
}