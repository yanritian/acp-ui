import 'package:json_annotation/json_annotation.dart';

part 'session.g.dart';

/// Session status enum
enum SessionStatus {
  active,
  paused,
  completed,
  compacted,
}

/// Message role enum
enum MessageRole {
  user,
  assistant,
  system,
  tool,
}

/// Session message model
@JsonSerializable()
class SessionMessage {
  final String id;
  final MessageRole role;
  final String content;
  final DateTime timestamp;
  final bool compressed;

  const SessionMessage({
    required this.id,
    required this.role,
    required this.content,
    required this.timestamp,
    this.compressed = false,
  });

  factory SessionMessage.fromJson(Map<String, dynamic> json) =>
      _$SessionMessageFromJson(json);
  Map<String, dynamic> toJson() => _$SessionMessageToJson(this);
}

/// Session model
@JsonSerializable()
class Session {
  final String id;
  final String agentId;
  final DateTime createdAt;
  final DateTime lastActive;
  final List<SessionMessage> messages;
  final String? branchLock;
  final SessionStatus status;
  final int compactionCount;

  const Session({
    required this.id,
    required this.agentId,
    required this.createdAt,
    required this.lastActive,
    this.messages = const [],
    this.branchLock,
    this.status = SessionStatus.active,
    this.compactionCount = 0,
  });

  factory Session.fromJson(Map<String, dynamic> json) => _$SessionFromJson(json);
  Map<String, dynamic> toJson() => _$SessionToJson(this);

  /// Add a message to session (returns new session)
  Session addMessage(SessionMessage message) {
    return Session(
      id: id,
      agentId: agentId,
      createdAt: createdAt,
      lastActive: message.timestamp,
      messages: [...messages, message],
      branchLock: branchLock,
      status: status,
      compactionCount: compactionCount,
    );
  }

  /// Check if needs compaction (more than threshold messages)
  bool needsCompaction(int threshold) {
    return messages.length > threshold;
  }

  /// Copy with new values
  Session copyWith({
    String? id,
    String? agentId,
    DateTime? createdAt,
    DateTime? lastActive,
    List<SessionMessage>? messages,
    String? branchLock,
    SessionStatus? status,
    int? compactionCount,
  }) {
    return Session(
      id: id ?? this.id,
      agentId: agentId ?? this.agentId,
      createdAt: createdAt ?? this.createdAt,
      lastActive: lastActive ?? this.lastActive,
      messages: messages ?? this.messages,
      branchLock: branchLock ?? this.branchLock,
      status: status ?? this.status,
      compactionCount: compactionCount ?? this.compactionCount,
    );
  }
}