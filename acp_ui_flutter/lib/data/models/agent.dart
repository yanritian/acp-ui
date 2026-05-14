import 'package:json_annotation/json_annotation.dart';

part 'agent.g.dart';

/// Agent status enum
enum AgentStatus {
  idle,
  busy,
  error,
  offline,
}

/// Agent type enum
enum AgentType {
  planner,
  architect,
  tddGuide,
  codeReviewer,
  securityReviewer,
  buildErrorResolver,
  e2eRunner,
  refactorCleaner,
  docUpdater,
  generalPurpose,
  explore,
  custom,
}

/// Agent model
@JsonSerializable()
class Agent {
  final String id;
  final String name;
  final AgentType type;
  final AgentStatus status;
  final String? description;
  final List<String> capabilities;
  final String? cwd;
  final int maxContextTokens;
  final int usedContextTokens;
  final DateTime createdAt;
  final DateTime? lastActiveAt;

  const Agent({
    required this.id,
    required this.name,
    required this.type,
    this.status = AgentStatus.idle,
    this.description,
    this.capabilities = const [],
    this.cwd,
    this.maxContextTokens = 200000,
    this.usedContextTokens = 0,
    required this.createdAt,
    this.lastActiveAt,
  });

  factory Agent.fromJson(Map<String, dynamic> json) => _$AgentFromJson(json);
  Map<String, dynamic> toJson() => _$AgentToJson(this);

  /// Context usage percentage
  double contextUsagePercent() {
    if (maxContextTokens == 0) return 0;
    return usedContextTokens / maxContextTokens * 100;
  }

  /// Check if context is near limit (80%)
  bool isContextNearLimit() {
    return contextUsagePercent() >= 80;
  }

  /// Copy with new values
  Agent copyWith({
    String? id,
    String? name,
    AgentType? type,
    AgentStatus? status,
    String? description,
    List<String>? capabilities,
    String? cwd,
    int? maxContextTokens,
    int? usedContextTokens,
    DateTime? createdAt,
    DateTime? lastActiveAt,
  }) {
    return Agent(
      id: id ?? this.id,
      name: name ?? this.name,
      type: type ?? this.type,
      status: status ?? this.status,
      description: description ?? this.description,
      capabilities: capabilities ?? this.capabilities,
      cwd: cwd ?? this.cwd,
      maxContextTokens: maxContextTokens ?? this.maxContextTokens,
      usedContextTokens: usedContextTokens ?? this.usedContextTokens,
      createdAt: createdAt ?? this.createdAt,
      lastActiveAt: lastActiveAt ?? this.lastActiveAt,
    );
  }
}