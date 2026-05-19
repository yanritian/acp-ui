/// Collaboration types for Flutter
/// Based on TypeScript types from src/lib/collaboration/types.ts

/// Node type enumeration
enum NodeType {
  planner,
  architect,
  developer,
  tddGuide,
  codeReviewer,
  securityReviewer,
  performanceReviewer,
  docUpdater,
}

/// Node status enumeration
enum NodeStatus {
  active,
  idle,
  busy,
  error,
}

/// Edge status enumeration
enum EdgeStatus {
  flowing,
  pending,
  completed,
}

/// Agent capability definition
class AgentCapability {
  final String id;
  final String name;
  final String description;
  final String icon;
  final String category;
  final List<String>? prerequisites;
  final List<String>? outputs;
  final int? proficiency;
  final List<String>? examples;

  const AgentCapability({
    required this.id,
    required this.name,
    required this.description,
    required this.icon,
    required this.category,
    this.prerequisites,
    this.outputs,
    this.proficiency,
    this.examples,
  });

  factory AgentCapability.fromJson(Map<String, dynamic> json) => AgentCapability(
    id: json['id'] as String,
    name: json['name'] as String,
    description: json['description'] as String,
    icon: json['icon'] as String,
    category: json['category'] as String,
    prerequisites: (json['prerequisites'] as List?)?.map((e) => e as String).toList(),
    outputs: (json['outputs'] as List?)?.map((e) => e as String).toList(),
    proficiency: json['proficiency'] as int?,
    examples: (json['examples'] as List?)?.map((e) => e as String).toList(),
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'description': description,
    'icon': icon,
    'category': category,
    'prerequisites': prerequisites,
    'outputs': outputs,
    'proficiency': proficiency,
    'examples': examples,
  };
}

/// Collaboration node - represents an Agent
class CollaborationNode {
  final String id;
  final String agentId;
  final String agentName;
  final String agentType;
  final Position position;
  final String status;
  final List<AgentCapability> capabilities;
  final int currentLoad;
  final int maxLoad;
  final String? description;
  final String? avatarUrl;

  const CollaborationNode({
    required this.id,
    required this.agentId,
    required this.agentName,
    required this.agentType,
    required this.position,
    required this.status,
    required this.capabilities,
    required this.currentLoad,
    required this.maxLoad,
    this.description,
    this.avatarUrl,
  });

  factory CollaborationNode.fromJson(Map<String, dynamic> json) => CollaborationNode(
    id: json['id'] as String,
    agentId: json['agentId'] as String,
    agentName: json['agentName'] as String,
    agentType: json['agentType'] as String,
    position: Position.fromJson(json['position'] as Map<String, dynamic>),
    status: json['status'] as String,
    capabilities: (json['capabilities'] as List)
      .map((e) => AgentCapability.fromJson(e as Map<String, dynamic>))
      .toList(),
    currentLoad: json['currentLoad'] as int,
    maxLoad: json['maxLoad'] as int,
    description: json['description'] as String?,
    avatarUrl: json['avatarUrl'] as String?,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'agentId': agentId,
    'agentName': agentName,
    'agentType': agentType,
    'position': position.toJson(),
    'status': status,
    'capabilities': capabilities.map((e) => e.toJson()).toList(),
    'currentLoad': currentLoad,
    'maxLoad': maxLoad,
    'description': description,
    'avatarUrl': avatarUrl,
  };
}

/// Position for graph layout
class Position {
  final double x;
  final double y;

  const Position({required this.x, required this.y});

  factory Position.fromJson(Map<String, dynamic> json) => Position(
    x: (json['x'] as num).toDouble(),
    y: (json['y'] as num).toDouble(),
  );

  Map<String, dynamic> toJson() => {'x': x, 'y': y};
}

/// Collaboration edge - represents task flow or message transfer
class CollaborationEdge {
  final String id;
  final String sourceAgentId;
  final String targetAgentId;
  final String taskId;
  final String taskDescription;
  final String status;
  final int timestamp;
  final double animationProgress;
  final String? messageType;
  final String? payloadPreview;
  final int? duration;

  const CollaborationEdge({
    required this.id,
    required this.sourceAgentId,
    required this.targetAgentId,
    required this.taskId,
    required this.taskDescription,
    required this.status,
    required this.timestamp,
    required this.animationProgress,
    this.messageType,
    this.payloadPreview,
    this.duration,
  });

  factory CollaborationEdge.fromJson(Map<String, dynamic> json) => CollaborationEdge(
    id: json['id'] as String,
    sourceAgentId: json['sourceAgentId'] as String,
    targetAgentId: json['targetAgentId'] as String,
    taskId: json['taskId'] as String,
    taskDescription: json['taskDescription'] as String,
    status: json['status'] as String,
    timestamp: json['timestamp'] as int,
    animationProgress: (json['animationProgress'] as num).toDouble(),
    messageType: json['messageType'] as String?,
    payloadPreview: json['payloadPreview'] as String?,
    duration: json['duration'] as int?,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'sourceAgentId': sourceAgentId,
    'targetAgentId': targetAgentId,
    'taskId': taskId,
    'taskDescription': taskDescription,
    'status': status,
    'timestamp': timestamp,
    'animationProgress': animationProgress,
    'messageType': messageType,
    'payloadPreview': payloadPreview,
    'duration': duration,
  };
}

/// Collaboration event - detailed record of all collaboration activities
class CollaborationEvent {
  final String id;
  final String type;
  final int timestamp;
  final String sourceAgentId;
  final String? targetAgentId;
  final String? taskId;
  final String? protocolId;
  final String? capabilityId;
  final dynamic payload;
  final EventDetails details;
  final String severity;

  const CollaborationEvent({
    required this.id,
    required this.type,
    required this.timestamp,
    required this.sourceAgentId,
    this.targetAgentId,
    this.taskId,
    this.protocolId,
    this.capabilityId,
    required this.payload,
    required this.details,
    required this.severity,
  });

  factory CollaborationEvent.fromJson(Map<String, dynamic> json) => CollaborationEvent(
    id: json['id'] as String,
    type: json['type'] as String,
    timestamp: json['timestamp'] as int,
    sourceAgentId: json['sourceAgentId'] as String,
    targetAgentId: json['targetAgentId'] as String?,
    taskId: json['taskId'] as String?,
    protocolId: json['protocolId'] as String?,
    capabilityId: json['capabilityId'] as String?,
    payload: json['payload'],
    details: EventDetails.fromJson(json['details'] as Map<String, dynamic>),
    severity: json['severity'] as String,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'type': type,
    'timestamp': timestamp,
    'sourceAgentId': sourceAgentId,
    'targetAgentId': targetAgentId,
    'taskId': taskId,
    'protocolId': protocolId,
    'capabilityId': capabilityId,
    'payload': payload,
    'details': details.toJson(),
    'severity': severity,
  };
}

/// Event details
class EventDetails {
  final String summary;
  final String? description;
  final int? duration;
  final bool? success;
  final String? error;
  final EventMetrics? metrics;
  final List<EventAttachment>? attachments;

  const EventDetails({
    required this.summary,
    this.description,
    this.duration,
    this.success,
    this.error,
    this.metrics,
    this.attachments,
  });

  factory EventDetails.fromJson(Map<String, dynamic> json) => EventDetails(
    summary: json['summary'] as String,
    description: json['description'] as String?,
    duration: json['duration'] as int?,
    success: json['success'] as bool?,
    error: json['error'] as String?,
    metrics: json['metrics'] != null
      ? EventMetrics.fromJson(json['metrics'] as Map<String, dynamic>)
      : null,
    attachments: (json['attachments'] as List?)
      ?.map((e) => EventAttachment.fromJson(e as Map<String, dynamic>))
      .toList(),
  );

  Map<String, dynamic> toJson() => {
    'summary': summary,
    'description': description,
    'duration': duration,
    'success': success,
    'error': error,
    'metrics': metrics?.toJson(),
    'attachments': attachments?.map((e) => e.toJson()).toList(),
  };
}

/// Event metrics
class EventMetrics {
  final int? executionTime;
  final int? resourceUsage;
  final int? messageCount;
  final int? toolCallCount;
  final int? retryCount;

  const EventMetrics({
    this.executionTime,
    this.resourceUsage,
    this.messageCount,
    this.toolCallCount,
    this.retryCount,
  });

  factory EventMetrics.fromJson(Map<String, dynamic> json) => EventMetrics(
    executionTime: json['executionTime'] as int?,
    resourceUsage: json['resourceUsage'] as int?,
    messageCount: json['messageCount'] as int?,
    toolCallCount: json['toolCallCount'] as int?,
    retryCount: json['retryCount'] as int?,
  );

  Map<String, dynamic> toJson() => {
    'executionTime': executionTime,
    'resourceUsage': resourceUsage,
    'messageCount': messageCount,
    'toolCallCount': toolCallCount,
    'retryCount': retryCount,
  };
}

/// Event attachment
class EventAttachment {
  final String id;
  final String type;
  final String name;
  final String? content;
  final String? url;
  final int? size;

  const EventAttachment({
    required this.id,
    required this.type,
    required this.name,
    this.content,
    this.url,
    this.size,
  });

  factory EventAttachment.fromJson(Map<String, dynamic> json) => EventAttachment(
    id: json['id'] as String,
    type: json['type'] as String,
    name: json['name'] as String,
    content: json['content'] as String?,
    url: json['url'] as String?,
    size: json['size'] as int?,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'type': type,
    'name': name,
    'content': content,
    'url': url,
    'size': size,
  };
}

/// Collaboration protocol definition
class CollaborationProtocol {
  final String id;
  final String name;
  final String version;
  final List<String> participants;
  final ContractDefinition inputContract;
  final ContractDefinition outputContract;
  final List<Condition> executionConditions;
  final List<Constraint> constraints;
  final List<ProtocolExample> examples;
  final ProtocolMetadata? metadata;

  const CollaborationProtocol({
    required this.id,
    required this.name,
    required this.version,
    required this.participants,
    required this.inputContract,
    required this.outputContract,
    required this.executionConditions,
    required this.constraints,
    required this.examples,
    this.metadata,
  });

  factory CollaborationProtocol.fromJson(Map<String, dynamic> json) => CollaborationProtocol(
    id: json['id'] as String,
    name: json['name'] as String,
    version: json['version'] as String,
    participants: (json['participants'] as List).map((e) => e as String).toList(),
    inputContract: ContractDefinition.fromJson(json['inputContract'] as Map<String, dynamic>),
    outputContract: ContractDefinition.fromJson(json['outputContract'] as Map<String, dynamic>),
    executionConditions: (json['executionConditions'] as List)
      .map((e) => Condition.fromJson(e as Map<String, dynamic>))
      .toList(),
    constraints: (json['constraints'] as List)
      .map((e) => Constraint.fromJson(e as Map<String, dynamic>))
      .toList(),
    examples: (json['examples'] as List)
      .map((e) => ProtocolExample.fromJson(e as Map<String, dynamic>))
      .toList(),
    metadata: json['metadata'] != null
      ? ProtocolMetadata.fromJson(json['metadata'] as Map<String, dynamic>)
      : null,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'version': version,
    'participants': participants,
    'inputContract': inputContract.toJson(),
    'outputContract': outputContract.toJson(),
    'executionConditions': executionConditions.map((e) => e.toJson()).toList(),
    'constraints': constraints.map((e) => e.toJson()).toList(),
    'examples': examples.map((e) => e.toJson()).toList(),
    'metadata': metadata?.toJson(),
  };
}

/// Contract definition
class ContractDefinition {
  final String type;
  final Map<String, dynamic> schema;
  final bool required;
  final String? description;
  final List<ValidationRule>? validationRules;

  const ContractDefinition({
    required this.type,
    required this.schema,
    required this.required,
    this.description,
    this.validationRules,
  });

  factory ContractDefinition.fromJson(Map<String, dynamic> json) => ContractDefinition(
    type: json['type'] as String,
    schema: json['schema'] as Map<String, dynamic>,
    required: json['required'] as bool,
    description: json['description'] as String?,
    validationRules: (json['validationRules'] as List?)
      ?.map((e) => ValidationRule.fromJson(e as Map<String, dynamic>))
      .toList(),
  );

  Map<String, dynamic> toJson() => {
    'type': type,
    'schema': schema,
    'required': required,
    'description': description,
    'validationRules': validationRules?.map((e) => e.toJson()).toList(),
  };
}

/// Execution condition
class Condition {
  final String id;
  final String type;
  final String expression;
  final String description;
  final int? priority;

  const Condition({
    required this.id,
    required this.type,
    required this.expression,
    required this.description,
    this.priority,
  });

  factory Condition.fromJson(Map<String, dynamic> json) => Condition(
    id: json['id'] as String,
    type: json['type'] as String,
    expression: json['expression'] as String,
    description: json['description'] as String,
    priority: json['priority'] as int?,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'type': type,
    'expression': expression,
    'description': description,
    'priority': priority,
  };
}

/// Constraint
class Constraint {
  final String id;
  final String type;
  final dynamic value;
  final String? unit;
  final String description;
  final String enforcement;

  const Constraint({
    required this.id,
    required this.type,
    required this.value,
    this.unit,
    required this.description,
    required this.enforcement,
  });

  factory Constraint.fromJson(Map<String, dynamic> json) => Constraint(
    id: json['id'] as String,
    type: json['type'] as String,
    value: json['value'],
    unit: json['unit'] as String?,
    description: json['description'] as String,
    enforcement: json['enforcement'] as String,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'type': type,
    'value': value,
    'unit': unit,
    'description': description,
    'enforcement': enforcement,
  };
}

/// Protocol example
class ProtocolExample {
  final String id;
  final String name;
  final String scenario;
  final Map<String, dynamic> input;
  final Map<String, dynamic> output;
  final List<String>? notes;

  const ProtocolExample({
    required this.id,
    required this.name,
    required this.scenario,
    required this.input,
    required this.output,
    this.notes,
  });

  factory ProtocolExample.fromJson(Map<String, dynamic> json) => ProtocolExample(
    id: json['id'] as String,
    name: json['name'] as String,
    scenario: json['scenario'] as String,
    input: json['input'] as Map<String, dynamic>,
    output: json['output'] as Map<String, dynamic>,
    notes: (json['notes'] as List?)?.map((e) => e as String).toList(),
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'scenario': scenario,
    'input': input,
    'output': output,
    'notes': notes,
  };
}

/// Validation rule
class ValidationRule {
  final String id;
  final String type;
  final String rule;
  final String errorMessage;

  const ValidationRule({
    required this.id,
    required this.type,
    required this.rule,
    required this.errorMessage,
  });

  factory ValidationRule.fromJson(Map<String, dynamic> json) => ValidationRule(
    id: json['id'] as String,
    type: json['type'] as String,
    rule: json['rule'] as String,
    errorMessage: json['errorMessage'] as String,
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'type': type,
    'rule': rule,
    'errorMessage': errorMessage,
  };
}

/// Protocol metadata
class ProtocolMetadata {
  final String? author;
  final int? createdAt;
  final int? updatedAt;
  final List<String>? tags;

  const ProtocolMetadata({
    this.author,
    this.createdAt,
    this.updatedAt,
    this.tags,
  });

  factory ProtocolMetadata.fromJson(Map<String, dynamic> json) => ProtocolMetadata(
    author: json['author'] as String?,
    createdAt: json['createdAt'] as int?,
    updatedAt: json['updatedAt'] as int?,
    tags: (json['tags'] as List?)?.map((e) => e as String).toList(),
  );

  Map<String, dynamic> toJson() => {
    'author': author,
    'createdAt': createdAt,
    'updatedAt': updatedAt,
    'tags': tags,
  };
}

/// Collaboration network stats
class CollaborationNetworkStats {
  final int totalAgents;
  final int activeAgents;
  final int totalTasks;
  final int runningTasks;
  final int completedTasks;
  final int failedTasks;
  final int averageTaskDuration;
  final int collaborationEfficiency;
  final int messageCount;
  final int toolCallCount;
  final int protocolInvocations;

  const CollaborationNetworkStats({
    required this.totalAgents,
    required this.activeAgents,
    required this.totalTasks,
    required this.runningTasks,
    required this.completedTasks,
    required this.failedTasks,
    required this.averageTaskDuration,
    required this.collaborationEfficiency,
    required this.messageCount,
    required this.toolCallCount,
    required this.protocolInvocations,
  });

  factory CollaborationNetworkStats.fromJson(Map<String, dynamic> json) =>
    CollaborationNetworkStats(
      totalAgents: json['totalAgents'] as int,
      activeAgents: json['activeAgents'] as int,
      totalTasks: json['totalTasks'] as int,
      runningTasks: json['runningTasks'] as int,
      completedTasks: json['completedTasks'] as int,
      failedTasks: json['failedTasks'] as int,
      averageTaskDuration: json['averageTaskDuration'] as int,
      collaborationEfficiency: json['collaborationEfficiency'] as int,
      messageCount: json['messageCount'] as int,
      toolCallCount: json['toolCallCount'] as int,
      protocolInvocations: json['protocolInvocations'] as int,
    );
}