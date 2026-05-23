import 'package:flutter_test/flutter_test.dart';
import 'package:acp_ui_flutter/core/self_improvement/self_healing.dart';

void main() {
  group('SelfHealingSystem', () {
    test('should create healing system', () {
      final system = SelfHealingSystem();
      expect(system.getHealingLog(), isEmpty);
    });

    test('should record healing event', () {
      final system = SelfHealingSystem();
      final event = system.recordEvent(
        HealingEventType.errorRecovery,
        'success',
        message: 'Test event',
      );

      expect(event.type, HealingEventType.errorRecovery);
      expect(event.status, 'success');
      expect(event.message, 'Test event');
      expect(system.getHealingLog().length, 1);
    });

    test('should record error and track count', () {
      final system = SelfHealingSystem();
      system.recordError('memory_full');
      system.recordError('memory_full');
      system.recordError('memory_full');

      expect(system.isErrorThresholdExceeded('memory_full', 3), true);
      expect(system.isErrorThresholdExceeded('memory_full', 4), false);
    });

    test('should trigger correct healing type', () {
      final system = SelfHealingSystem();

      final memoryEvent = system.triggerHealing('memory_full');
      expect(memoryEvent.type, HealingEventType.memoryCompaction);

      final agentEvent = system.triggerHealing('agent_timeout');
      expect(agentEvent.type, HealingEventType.agentRestart);

      final contextEvent = system.triggerHealing('context_overflow');
      expect(contextEvent.type, HealingEventType.contextReset);

      final cacheEvent = system.triggerHealing('cache_corruption');
      expect(cacheEvent.type, HealingEventType.cacheClear);

      final defaultEvent = system.triggerHealing('unknown_error');
      expect(defaultEvent.type, HealingEventType.errorRecovery);
    });

    test('should calculate stats correctly', () {
      final system = SelfHealingSystem();

      system.recordEvent(HealingEventType.errorRecovery, 'success');
      system.recordEvent(HealingEventType.errorRecovery, 'success');
      system.recordEvent(HealingEventType.errorRecovery, 'failed');

      final stats = system.getStats();

      expect(stats['totalEvents'], 3);
      expect(stats['successRate'], 66.66666666666666);
    });

    test('should return zero stats for empty log', () {
      final system = SelfHealingSystem();
      final stats = system.getStats();

      expect(stats['totalEvents'], 0);
      expect(stats['successRate'], 0.0);
    });
  });

  group('EvolutionEngine', () {
    test('should create engine with healing system', () {
      final healing = SelfHealingSystem();
      final engine = EvolutionEngine(healingSystem: healing);

      expect(engine.getPatterns(), isEmpty);
      expect(engine.getEvolutionScore(), 0.0);
    });

    test('should learn new pattern', () {
      final healing = SelfHealingSystem();
      final engine = EvolutionEngine(healingSystem: healing);

      final pattern = engine.learnPattern('Memory Optimization', 'Compact memory when full');

      expect(pattern.name, 'Memory Optimization');
      expect(engine.getPatterns().length, 1);
    });

    test('should update pattern frequency', () {
      final healing = SelfHealingSystem();
      final engine = EvolutionEngine(healingSystem: healing);

      final pattern = engine.learnPattern('Test Pattern', 'Description');
      engine.updatePatternFrequency(pattern.id);
      engine.updatePatternFrequency(pattern.id);

      final updatedPattern = engine.getPattern(pattern.id);
      expect(updatedPattern!.frequency, 2);
    });

    test('should calculate success rate from healing events', () {
      final healing = SelfHealingSystem();
      healing.recordEvent(HealingEventType.memoryCompaction, 'success');
      healing.recordEvent(HealingEventType.memoryCompaction, 'success');

      final engine = EvolutionEngine(healingSystem: healing);
      final pattern = engine.learnPattern('Test', 'Test');

      final rate = engine.calculateSuccessRate(pattern.id);
      expect(rate, 100.0);
    });

    test('should suggest improvement from patterns', () {
      final healing = SelfHealingSystem();
      final engine = EvolutionEngine(healingSystem: healing);

      // No patterns - no suggestion
      expect(engine.suggestImprovement(), isNull);

      engine.learnPattern('Pattern A', 'Description A');
      engine.updatePatternFrequency(engine.getPatterns().first.id);

      final suggestion = engine.suggestImprovement();
      expect(suggestion, contains('Pattern A'));
    });

    test('should get pattern by ID', () {
      final healing = SelfHealingSystem();
      final engine = EvolutionEngine(healingSystem: healing);

      final pattern = engine.learnPattern('Test', 'Test');
      final found = engine.getPattern(pattern.id);

      expect(found, isNotNull);
      expect(found!.name, 'Test');

      expect(engine.getPattern('non-existent'), isNull);
    });
  });

  group('HealingEvent', () {
    test('should create event with all fields', () {
      final event = HealingEvent(
        id: 'event-1',
        type: HealingEventType.agentRestart,
        status: 'success',
        timestamp: DateTime(2024, 1, 1),
        message: 'Agent restarted',
      );

      expect(event.id, 'event-1');
      expect(event.type, HealingEventType.agentRestart);
      expect(event.status, 'success');
      expect(event.message, 'Agent restarted');
    });
  });

  group('EvolutionPattern', () {
    test('should create pattern with defaults', () {
      final pattern = EvolutionPattern(
        id: 'pattern-1',
        name: 'Test Pattern',
        description: 'Test description',
      );

      expect(pattern.frequency, 0);
      expect(pattern.successRate, 0.0);
    });

    test('should create pattern with custom values', () {
      final pattern = EvolutionPattern(
        id: 'pattern-1',
        name: 'Test',
        description: 'Desc',
        frequency: 10,
        successRate: 85.5,
      );

      expect(pattern.frequency, 10);
      expect(pattern.successRate, 85.5);
    });
  });
}