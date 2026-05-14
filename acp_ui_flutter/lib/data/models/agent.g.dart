// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'agent.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Agent _$AgentFromJson(Map<String, dynamic> json) => Agent(
      id: json['id'] as String,
      name: json['name'] as String,
      type: $enumDecode(_$AgentTypeEnumMap, json['type']),
      status: $enumDecodeNullable(_$AgentStatusEnumMap, json['status']) ??
          AgentStatus.idle,
      description: json['description'] as String?,
      capabilities: (json['capabilities'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
      cwd: json['cwd'] as String?,
      maxContextTokens: (json['maxContextTokens'] as num?)?.toInt() ?? 200000,
      usedContextTokens: (json['usedContextTokens'] as num?)?.toInt() ?? 0,
      createdAt: DateTime.parse(json['createdAt'] as String),
      lastActiveAt: json['lastActiveAt'] == null
          ? null
          : DateTime.parse(json['lastActiveAt'] as String),
    );

Map<String, dynamic> _$AgentToJson(Agent instance) => <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'type': _$AgentTypeEnumMap[instance.type]!,
      'status': _$AgentStatusEnumMap[instance.status]!,
      'description': instance.description,
      'capabilities': instance.capabilities,
      'cwd': instance.cwd,
      'maxContextTokens': instance.maxContextTokens,
      'usedContextTokens': instance.usedContextTokens,
      'createdAt': instance.createdAt.toIso8601String(),
      'lastActiveAt': instance.lastActiveAt?.toIso8601String(),
    };

const _$AgentTypeEnumMap = {
  AgentType.planner: 'planner',
  AgentType.architect: 'architect',
  AgentType.tddGuide: 'tddGuide',
  AgentType.codeReviewer: 'codeReviewer',
  AgentType.securityReviewer: 'securityReviewer',
  AgentType.buildErrorResolver: 'buildErrorResolver',
  AgentType.e2eRunner: 'e2eRunner',
  AgentType.refactorCleaner: 'refactorCleaner',
  AgentType.docUpdater: 'docUpdater',
  AgentType.generalPurpose: 'generalPurpose',
  AgentType.explore: 'explore',
  AgentType.custom: 'custom',
};

const _$AgentStatusEnumMap = {
  AgentStatus.idle: 'idle',
  AgentStatus.busy: 'busy',
  AgentStatus.error: 'error',
  AgentStatus.offline: 'offline',
};
