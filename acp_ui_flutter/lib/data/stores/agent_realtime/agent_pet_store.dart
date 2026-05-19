/// Agent Pet Store
/// 宠物状态管理

import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../models/agent_pet/agent_pet_types.dart';

/// Agent Pet State
class AgentPetState {
  final Map<String, AgentPet> pets;
  final String? activePetId;
  final List<PetInteraction> recentInteractions;
  final int totalInteractions;
  final double averageHappiness;
  final double averageIntimacy;
  final List<String> unlockedAchievements;

  const AgentPetState({
    this.pets = const {},
    this.activePetId,
    this.recentInteractions = const [],
    this.totalInteractions = 0,
    this.averageHappiness = 50.0,
    this.averageIntimacy = 30.0,
    this.unlockedAchievements = const [],
  });

  AgentPetState copyWith({
    Map<String, AgentPet>? pets,
    String? activePetId,
    List<PetInteraction>? recentInteractions,
    int? totalInteractions,
    double? averageHappiness,
    double? averageIntimacy,
    List<String>? unlockedAchievements,
  }) {
    return AgentPetState(
      pets: pets ?? this.pets,
      activePetId: activePetId ?? this.activePetId,
      recentInteractions: recentInteractions ?? this.recentInteractions,
      totalInteractions: totalInteractions ?? this.totalInteractions,
      averageHappiness: averageHappiness ?? this.averageHappiness,
      averageIntimacy: averageIntimacy ?? this.averageIntimacy,
      unlockedAchievements: unlockedAchievements ?? this.unlockedAchievements,
    );
  }

  AgentPet? get activePet => activePetId != null ? pets[activePetId] : null;

  List<AgentPet> get allPets => pets.values.toList();

  List<AgentPet> get happyPets =>
      pets.values.where((p) => p.emotion == PetEmotion.happy).toList();

  List<AgentPet> get thinkingPets =>
      pets.values.where((p) => p.emotion == PetEmotion.thinking).toList();
}

/// Agent Pet Store
class AgentPetStore extends StateNotifier<AgentPetState> {
  AgentPetStore() : super(const AgentPetState()) {
    _initializeMockPets();
  }

  void _initializeMockPets() {
    final mockPets = <String, AgentPet>{
      'planner-pet': _createMockPet(
        'planner-pet',
        'Planner',
        PetEmotion.thinking,
        'planner-001',
      ),
      'architect-pet': _createMockPet(
        'architect-pet',
        'Architect',
        PetEmotion.happy,
        'architect-001',
      ),
      'tdd-guide-pet': _createMockPet(
        'tdd-guide-pet',
        'TDD Guide',
        PetEmotion.calm,
        'tddGuide-001',
      ),
      'code-reviewer-pet': _createMockPet(
        'code-reviewer-pet',
        'Code Reviewer',
        PetEmotion.sleepy,
        'codeReviewer-001',
      ),
    };

    state = state.copyWith(
      pets: mockPets,
      activePetId: 'planner-pet',
      totalInteractions: 5,
      averageHappiness: 65.0,
      averageIntimacy: 45.0,
      unlockedAchievements: ['first_task', 'speed_demon'],
    );
  }

  AgentPet _createMockPet(
    String id,
    String name,
    PetEmotion emotion,
    String agentId,
  ) {
    return AgentPet(
      petId: id,
      petName: name,
      agentId: agentId,
      emotion: emotion,
      happiness: 65.0,
      intimacy: 45.0,
      level: 2,
      experience: 350,
      totalTasks: 12,
      successfulTasks: 10,
      averageDuration: 45000,
      currentAnimation: PetAnimation.idle,
      lastInteractionTime: DateTime.now().millisecondsSinceEpoch - 300000,
      achievements: ['first_task'],
      statusText: _getStatusText(emotion),
    );
  }

  String _getStatusText(PetEmotion emotion) {
    switch (emotion) {
      case PetEmotion.happy:
        return '任务完成！';
      case PetEmotion.thinking:
        return '正在思考...';
      case PetEmotion.sleepy:
        return '休息中';
      case PetEmotion.confused:
        return '遇到困难';
      case PetEmotion.excited:
        return '开始新任务！';
      case PetEmotion.bored:
        return '等待任务';
      case PetEmotion.calm:
        return '准备就绪';
    }
  }

  void setActivePet(String petId) {
    state = state.copyWith(activePetId: petId);
  }

  void registerPet(AgentPet pet) {
    final newPets = Map<String, AgentPet>.from(state.pets);
    newPets[pet.petId] = pet;
    state = state.copyWith(pets: newPets);
  }

  void removePet(String petId) {
    final newPets = Map<String, AgentPet>.from(state.pets);
    newPets.remove(petId);
    state = state.copyWith(
      pets: newPets,
      activePetId: state.activePetId == petId ? null : state.activePetId,
    );
  }

  void updatePetEmotion(String petId, PetEmotion emotion) {
    final pet = state.pets[petId];
    if (pet == null) return;

    final newPets = Map<String, AgentPet>.from(state.pets);
    newPets[petId] = pet.copyWith(
      emotion: emotion,
      statusText: _getStatusText(emotion),
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );
    state = state.copyWith(pets: newPets);
  }

  void interactWithPet(String petId, String interactionType) {
    final pet = state.pets[petId];
    if (pet == null) return;

    double happinessChange = 0;
    double intimacyChange = 0;

    switch (interactionType) {
      case 'pet':
        happinessChange = 5;
        intimacyChange = 2;
        break;
      case 'poke':
        happinessChange = -2;
        intimacyChange = 0;
        break;
      case 'feed':
        happinessChange = 10;
        intimacyChange = 5;
        break;
      case 'play':
        happinessChange = 15;
        intimacyChange = 10;
        break;
    }

    final newHappiness = (pet.happiness + happinessChange).clamp(0.0, 100.0);
    final newIntimacy = (pet.intimacy + intimacyChange).clamp(0.0, 100.0);

    final interaction = PetInteraction(
      interactionId: 'interaction-${DateTime.now().millisecondsSinceEpoch}',
      petId: petId,
      type: interactionType,
      timestamp: DateTime.now().millisecondsSinceEpoch,
      happinessChange: happinessChange,
      intimacyChange: intimacyChange,
      resultingEmotion: _getEmotionFromHappiness(newHappiness),
    );

    final newPets = Map<String, AgentPet>.from(state.pets);
    newPets[petId] = pet.copyWith(
      happiness: newHappiness,
      intimacy: newIntimacy,
      emotion: interaction.resultingEmotion,
      lastInteractionTime: interaction.timestamp,
      totalInteractions: pet.totalInteractions + 1,
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );

    final newInteractions = List<PetInteraction>.from(state.recentInteractions);
    newInteractions.insert(0, interaction);
    if (newInteractions.length > 20) newInteractions.removeLast();

    final allPets = newPets.values.toList();
    final avgHappiness = allPets.isEmpty
        ? 0.0
        : allPets.map((p) => p.happiness).reduce((a, b) => a + b) / allPets.length;
    final avgIntimacy = allPets.isEmpty
        ? 0.0
        : allPets.map((p) => p.intimacy).reduce((a, b) => a + b) / allPets.length;

    state = state.copyWith(
      pets: newPets,
      recentInteractions: newInteractions,
      totalInteractions: state.totalInteractions + 1,
      averageHappiness: avgHappiness,
      averageIntimacy: avgIntimacy,
    );
  }

  PetEmotion _getEmotionFromHappiness(double happiness) {
    if (happiness >= 80) return PetEmotion.happy;
    if (happiness >= 60) return PetEmotion.calm;
    if (happiness >= 40) return PetEmotion.bored;
    if (happiness >= 20) return PetEmotion.confused;
    return PetEmotion.sleepy;
  }

  void addExperience(String petId, int exp) {
    final pet = state.pets[petId];
    if (pet == null) return;

    final newExp = pet.experience + exp;
    final newLevel = _calculateLevel(newExp);

    final newPets = Map<String, AgentPet>.from(state.pets);
    newPets[petId] = pet.copyWith(
      experience: newExp,
      level: newLevel,
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );

    // Check for level-up achievement
    if (newLevel > pet.level) {
      _unlockAchievement('level_$newLevel');
    }

    state = state.copyWith(pets: newPets);
  }

  int _calculateLevel(int experience) {
    if (experience >= 5000) return 5;
    if (experience >= 2000) return 4;
    if (experience >= 800) return 3;
    if (experience >= 300) return 2;
    return 1;
  }

  void _unlockAchievement(String achievementId) {
    if (state.unlockedAchievements.contains(achievementId)) return;

    final newAchievements = List<String>.from(state.unlockedAchievements);
    newAchievements.add(achievementId);
    state = state.copyWith(unlockedAchievements: newAchievements);
  }

  void updatePetAnimation(String petId, PetAnimation animation) {
    final pet = state.pets[petId];
    if (pet == null) return;

    final newPets = Map<String, AgentPet>.from(state.pets);
    newPets[petId] = pet.copyWith(
      currentAnimation: animation,
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );
    state = state.copyWith(pets: newPets);
  }

  void completeTask(String petId, bool success, int duration) {
    final pet = state.pets[petId];
    if (pet == null) return;

    final newTasks = pet.totalTasks + 1;
    final newSuccessfulTasks =
        success ? pet.successfulTasks + 1 : pet.successfulTasks;
    final newAvgDuration =
        ((pet.averageDuration * pet.totalTasks) + duration) / newTasks;

    // Add experience for task completion
    final expGain = success ? (duration < 30000 ? 20 : 10) : 5;
    final newExp = pet.experience + expGain;
    final newLevel = _calculateLevel(newExp);

    // Update emotion based on success
    final newEmotion = success ? PetEmotion.happy : PetEmotion.confused;

    final newPets = Map<String, AgentPet>.from(state.pets);
    newPets[petId] = pet.copyWith(
      totalTasks: newTasks,
      successfulTasks: newSuccessfulTasks,
      averageDuration: newAvgDuration,
      experience: newExp,
      level: newLevel,
      emotion: newEmotion,
      statusText: success ? '任务完成！' : '遇到问题',
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );

    // Unlock achievements
    if (newTasks == 1) _unlockAchievement('first_task');
    if (newTasks == 10) _unlockAchievement('ten_tasks');
    if (newTasks == 50) _unlockAchievement('fifty_tasks');
    if (newTasks == 100) _unlockAchievement('hundred_tasks');
    if (success && duration < 30000) _unlockAchievement('speed_demon');

    state = state.copyWith(pets: newPets);
  }
}

/// Provider
final agentPetProvider =
    StateNotifierProvider<AgentPetStore, AgentPetState>(
  (ref) => AgentPetStore(),
);