import 'dart:async';

/// Healing Event type
enum HealingEventType {
  errorRecovery,
  memoryCompaction,
  agentRestart,
  contextReset,
  cacheClear,
}

/// Healing Event
class HealingEvent {
  final String id;
  final HealingEventType type;
  final String status;
  final DateTime timestamp;
  final String? message;

  HealingEvent({
    required this.id,
    required this.type,
    required this.status,
    required this.timestamp,
    this.message,
  });
}

/// Evolution Pattern
class EvolutionPattern {
  final String id;
  final String name;
  final String description;
  final int frequency;
  final double successRate;

  EvolutionPattern({
    required this.id,
    required this.name,
    required this.description,
    this.frequency = 0,
    this.successRate = 0.0,
  });
}

/// Self Healing System (Claw Code inspired)
class SelfHealingSystem {
  final List<HealingEvent> _healingLog = [];
  final Map<String, int> _errorCounts = {};
  final StreamController<HealingEvent> _healingController = StreamController<HealingEvent>.broadcast();

  /// Record a healing event
  HealingEvent recordEvent(HealingEventType type, String status, {String? message}) {
    final event = HealingEvent(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      type: type,
      status: status,
      timestamp: DateTime.now(),
      message: message,
    );

    _healingLog.add(event);
    _healingController.add(event);
    return event;
  }

  /// Record an error
  void recordError(String errorType) {
    _errorCounts[errorType] = (_errorCounts[errorType] ?? 0) + 1;
  }

  /// Check if error threshold exceeded
  bool isErrorThresholdExceeded(String errorType, int threshold) {
    return (_errorCounts[errorType] ?? 0) >= threshold;
  }

  /// Trigger automatic healing
  HealingEvent triggerHealing(String errorType) {
    HealingEventType healingType;

    switch (errorType) {
      case 'memory_full':
        healingType = HealingEventType.memoryCompaction;
        break;
      case 'agent_timeout':
        healingType = HealingEventType.agentRestart;
        break;
      case 'context_overflow':
        healingType = HealingEventType.contextReset;
        break;
      case 'cache_corruption':
        healingType = HealingEventType.cacheClear;
        break;
      default:
        healingType = HealingEventType.errorRecovery;
    }

    return recordEvent(healingType, 'success', message: 'Healed $errorType');
  }

  /// Get healing log
  List<HealingEvent> getHealingLog() => List.unmodifiable(_healingLog);

  /// Get healing stream
  Stream<HealingEvent> get healingStream => _healingController.stream;

  /// Get statistics
  Map<String, dynamic> getStats() {
    final successCount = _healingLog.where((e) => e.status == 'success').length;
    final totalCount = _healingLog.length;

    return {
      'totalEvents': totalCount,
      'successRate': totalCount > 0 ? successCount / totalCount * 100 : 0.0,
      'errorCounts': _errorCounts,
    };
  }
}

/// Evolution Engine
class EvolutionEngine {
  final SelfHealingSystem _healingSystem;
  final List<EvolutionPattern> _patterns = [];
  final Map<String, int> _patternFrequency = {};

  EvolutionEngine({required SelfHealingSystem healingSystem})
      : _healingSystem = healingSystem;

  /// Learn a new pattern from healing events
  EvolutionPattern learnPattern(String name, String description) {
    final pattern = EvolutionPattern(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      name: name,
      description: description,
    );

    _patterns.add(pattern);
    return pattern;
  }

  /// Update pattern frequency
  void updatePatternFrequency(String patternId) {
    _patternFrequency[patternId] = (_patternFrequency[patternId] ?? 0) + 1;
  }

  /// Get learned patterns
  List<EvolutionPattern> getPatterns() => List.unmodifiable(_patterns);

  /// Get pattern by ID
  EvolutionPattern? getPattern(String patternId) {
    final pattern = _patterns.where((p) => p.id == patternId).firstOrNull;
    if (pattern == null) return null;

    return EvolutionPattern(
      id: pattern.id,
      name: pattern.name,
      description: pattern.description,
      frequency: _patternFrequency[pattern.id] ?? 0,
      successRate: calculateSuccessRate(pattern.id),
    );
  }

  /// Calculate success rate for a pattern
  double calculateSuccessRate(String patternId) {
    // Simplified calculation
    final healingEvents = _healingSystem.getHealingLog();
    final successCount = healingEvents.where((e) => e.status == 'success').length;

    return healingEvents.isNotEmpty ? successCount / healingEvents.length * 100 : 0.0;
  }

  /// Get evolution score
  double getEvolutionScore() {
    if (_patterns.isEmpty) return 0.0;

    final avgSuccessRate = _patterns
        .map((p) => calculateSuccessRate(p.id))
        .reduce((a, b) => a + b) / _patterns.length;

    return avgSuccessRate;
  }

  /// Suggest improvement based on patterns
  String? suggestImprovement() {
    if (_patterns.isEmpty) return null;

    // Find most frequent pattern
    final mostFrequent = _patterns.reduce((a, b) {
      final freqA = _patternFrequency[a.id] ?? 0;
      final freqB = _patternFrequency[b.id] ?? 0;
      return freqA > freqB ? a : b;
    });

    return 'Consider optimizing: ${mostFrequent.name}';
  }
}