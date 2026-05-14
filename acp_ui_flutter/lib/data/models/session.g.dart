// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'session.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SessionMessage _$SessionMessageFromJson(Map<String, dynamic> json) =>
    SessionMessage(
      id: json['id'] as String,
      role: $enumDecode(_$MessageRoleEnumMap, json['role']),
      content: json['content'] as String,
      timestamp: DateTime.parse(json['timestamp'] as String),
      compressed: json['compressed'] as bool? ?? false,
    );

Map<String, dynamic> _$SessionMessageToJson(SessionMessage instance) =>
    <String, dynamic>{
      'id': instance.id,
      'role': _$MessageRoleEnumMap[instance.role]!,
      'content': instance.content,
      'timestamp': instance.timestamp.toIso8601String(),
      'compressed': instance.compressed,
    };

const _$MessageRoleEnumMap = {
  MessageRole.user: 'user',
  MessageRole.assistant: 'assistant',
  MessageRole.system: 'system',
  MessageRole.tool: 'tool',
};

Session _$SessionFromJson(Map<String, dynamic> json) => Session(
      id: json['id'] as String,
      agentId: json['agentId'] as String,
      createdAt: DateTime.parse(json['createdAt'] as String),
      lastActive: DateTime.parse(json['lastActive'] as String),
      messages: (json['messages'] as List<dynamic>?)
              ?.map((e) => SessionMessage.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      branchLock: json['branchLock'] as String?,
      status: $enumDecodeNullable(_$SessionStatusEnumMap, json['status']) ??
          SessionStatus.active,
      compactionCount: (json['compactionCount'] as num?)?.toInt() ?? 0,
    );

Map<String, dynamic> _$SessionToJson(Session instance) => <String, dynamic>{
      'id': instance.id,
      'agentId': instance.agentId,
      'createdAt': instance.createdAt.toIso8601String(),
      'lastActive': instance.lastActive.toIso8601String(),
      'messages': instance.messages,
      'branchLock': instance.branchLock,
      'status': _$SessionStatusEnumMap[instance.status]!,
      'compactionCount': instance.compactionCount,
    };

const _$SessionStatusEnumMap = {
  SessionStatus.active: 'active',
  SessionStatus.paused: 'paused',
  SessionStatus.completed: 'completed',
  SessionStatus.compacted: 'compacted',
};
