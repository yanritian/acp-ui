/// Agent Pet Types for Agent Teams Platform
/// 简化版宠物类型定义

/// 宠物情绪枚举
enum PetEmotion {
  happy,
  thinking,
  sleepy,
  confused,
  excited,
  bored,
  calm,
}

/// 宠物活动类型枚举
enum PetActivityType {
  thinking,
  executing,
  idle,
  waiting,
  completed,
  failed,
}

/// 宠物动画枚举
enum PetAnimation {
  idle,
  walking,
  thinking,
  happy,
  sleeping,
  eating,
  playing,
}

/// Agent宠物模型（简化版）
class AgentPet {
  final String petId;
  final String petName;
  final String agentId;
  final PetEmotion emotion;
  final double happiness;
  final double intimacy;
  final int level;
  final int experience;
  final int totalTasks;
  final int successfulTasks;
  final double averageDuration;
  final PetAnimation currentAnimation;
  final int lastInteractionTime;
  final List<String> achievements;
  final String statusText;
  final int? lastUpdateTime;
  final int totalInteractions;

  const AgentPet({
    required this.petId,
    required this.petName,
    required this.agentId,
    required this.emotion,
    required this.happiness,
    required this.intimacy,
    required this.level,
    required this.experience,
    required this.totalTasks,
    required this.successfulTasks,
    required this.averageDuration,
    required this.currentAnimation,
    required this.lastInteractionTime,
    required this.achievements,
    required this.statusText,
    this.lastUpdateTime,
    this.totalInteractions = 0,
  });

  AgentPet copyWith({
    String? petId,
    String? petName,
    String? agentId,
    PetEmotion? emotion,
    double? happiness,
    double? intimacy,
    int? level,
    int? experience,
    int? totalTasks,
    int? successfulTasks,
    double? averageDuration,
    PetAnimation? currentAnimation,
    int? lastInteractionTime,
    List<String>? achievements,
    String? statusText,
    int? lastUpdateTime,
    int? totalInteractions,
  }) {
    return AgentPet(
      petId: petId ?? this.petId,
      petName: petName ?? this.petName,
      agentId: agentId ?? this.agentId,
      emotion: emotion ?? this.emotion,
      happiness: happiness ?? this.happiness,
      intimacy: intimacy ?? this.intimacy,
      level: level ?? this.level,
      experience: experience ?? this.experience,
      totalTasks: totalTasks ?? this.totalTasks,
      successfulTasks: successfulTasks ?? this.successfulTasks,
      averageDuration: averageDuration ?? this.averageDuration,
      currentAnimation: currentAnimation ?? this.currentAnimation,
      lastInteractionTime: lastInteractionTime ?? this.lastInteractionTime,
      achievements: achievements ?? this.achievements,
      statusText: statusText ?? this.statusText,
      lastUpdateTime: lastUpdateTime ?? this.lastUpdateTime,
      totalInteractions: totalInteractions ?? this.totalInteractions,
    );
  }
}

/// 宠物互动记录
class PetInteraction {
  final String interactionId;
  final String petId;
  final String type;
  final int timestamp;
  final double happinessChange;
  final double intimacyChange;
  final PetEmotion resultingEmotion;

  const PetInteraction({
    required this.interactionId,
    required this.petId,
    required this.type,
    required this.timestamp,
    required this.happinessChange,
    required this.intimacyChange,
    required this.resultingEmotion,
  });
}

/// 宠物成长配置
class PetGrowthConfig {
  static const int maxLevel = 5;
  static const Map<int, int> levelExperienceRequirements = {
    1: 0,
    2: 300,
    3: 800,
    4: 2000,
    5: 5000,
  };

  static int calculateLevel(int experience) {
    if (experience >= 5000) return 5;
    if (experience >= 2000) return 4;
    if (experience >= 800) return 3;
    if (experience >= 300) return 2;
    return 1;
  }

  static int experienceToNextLevel(int currentLevel, int currentExperience) {
    if (currentLevel >= maxLevel) return 0;
    final nextLevel = currentLevel + 1;
    final required = levelExperienceRequirements[nextLevel] ?? 0;
    return required - currentExperience;
  }
}