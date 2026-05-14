import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/self_improvement/self_healing.dart';

/// Self Healing System Provider
final selfHealingProvider = Provider<SelfHealingSystem>((ref) {
  return SelfHealingSystem();
});

/// Evolution Engine Provider
final evolutionEngineProvider = Provider<EvolutionEngine>((ref) {
  final healing = ref.watch(selfHealingProvider);
  return EvolutionEngine(healingSystem: healing);
});

/// Healing Log Provider
final healingLogProvider = Provider<List<HealingEvent>>((ref) {
  final system = ref.watch(selfHealingProvider);
  return system.getHealingLog();
});

/// Healing Stream Provider
final healingStreamProvider = StreamProvider<HealingEvent>((ref) {
  final system = ref.watch(selfHealingProvider);
  return system.healingStream;
});

/// Evolution Patterns Provider
final evolutionPatternsProvider = Provider<List<EvolutionPattern>>((ref) {
  final engine = ref.watch(evolutionEngineProvider);
  return engine.getPatterns();
});

/// Evolution Score Provider
final evolutionScoreProvider = Provider<double>((ref) {
  final engine = ref.watch(evolutionEngineProvider);
  return engine.getEvolutionScore();
});

/// Healing Stats Provider
final healingStatsProvider = Provider<Map<String, dynamic>>((ref) {
  final system = ref.watch(selfHealingProvider);
  return system.getStats();
});