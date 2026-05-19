import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:acp_ui_flutter/data/models/collaboration.dart';

/// Collaboration state
class CollaborationState {
  final List<CollaborationNode> nodes;
  final List<CollaborationEdge> edges;
  final List<CollaborationEvent> events;
  final List<CollaborationProtocol> protocols;
  final String? selectedNodeId;
  final String? selectedEdgeId;
  final String viewMode;
  final bool isLoading;
  final String? error;

  const CollaborationState({
    this.nodes = const [],
    this.edges = const [],
    this.events = const [],
    this.protocols = const [],
    this.selectedNodeId,
    this.selectedEdgeId,
    this.viewMode = 'network',
    this.isLoading = false,
    this.error,
  });

  CollaborationState copyWith({
    List<CollaborationNode>? nodes,
    List<CollaborationEdge>? edges,
    List<CollaborationEvent>? events,
    List<CollaborationProtocol>? protocols,
    String? selectedNodeId,
    String? selectedEdgeId,
    String? viewMode,
    bool? isLoading,
    String? error,
  }) {
    return CollaborationState(
      nodes: nodes ?? this.nodes,
      edges: edges ?? this.edges,
      events: events ?? this.events,
      protocols: protocols ?? this.protocols,
      selectedNodeId: selectedNodeId ?? this.selectedNodeId,
      selectedEdgeId: selectedEdgeId ?? this.selectedEdgeId,
      viewMode: viewMode ?? this.viewMode,
      isLoading: isLoading ?? this.isLoading,
      error: error ?? this.error,
    );
  }
}

/// Collaboration notifier
class CollaborationNotifier extends StateNotifier<CollaborationState> {
  CollaborationNotifier() : super(const CollaborationState());

  /// Add a node
  void addNode(CollaborationNode node) {
    state = state.copyWith(nodes: [...state.nodes, node]);
  }

  /// Update a node
  void updateNode(String nodeId, CollaborationNode updatedNode) {
    final nodes = state.nodes.map((n) {
      if (n.id == nodeId) return updatedNode;
      return n;
    }).toList();
    state = state.copyWith(nodes: nodes);
  }

  /// Remove a node
  void removeNode(String nodeId) {
    final nodes = state.nodes.where((n) => n.id != nodeId).toList();
    final edges = state.edges.where((e) =>
      e.sourceAgentId != nodeId && e.targetAgentId != nodeId
    ).toList();
    state = state.copyWith(nodes: nodes, edges: edges);
  }

  /// Add an edge
  void addEdge(CollaborationEdge edge) {
    state = state.copyWith(edges: [...state.edges, edge]);
  }

  /// Update an edge
  void updateEdge(String edgeId, CollaborationEdge updatedEdge) {
    final edges = state.edges.map((e) {
      if (e.id == edgeId) return updatedEdge;
      return e;
    }).toList();
    state = state.copyWith(edges: edges);
  }

  /// Remove an edge
  void removeEdge(String edgeId) {
    final edges = state.edges.where((e) => e.id != edgeId).toList();
    state = state.copyWith(edges: edges);
  }

  /// Add an event
  void addEvent(CollaborationEvent event) {
    final events = [event, ...state.events];
    // Keep only last 50 events
    final limitedEvents = events.length > 50 ? events.sublist(0, 50) : events;
    state = state.copyWith(events: limitedEvents);
  }

  /// Clear events
  void clearEvents() {
    state = state.copyWith(events: const []);
  }

  /// Add a protocol
  void addProtocol(CollaborationProtocol protocol) {
    state = state.copyWith(protocols: [...state.protocols, protocol]);
  }

  /// Remove a protocol
  void removeProtocol(String protocolId) {
    final protocols = state.protocols.where((p) => p.id != protocolId).toList();
    state = state.copyWith(protocols: protocols);
  }

  /// Select a node
  void selectNode(String? nodeId) {
    state = state.copyWith(selectedNodeId: nodeId);
  }

  /// Select an edge
  void selectEdge(String? edgeId) {
    state = state.copyWith(selectedEdgeId: edgeId);
  }

  /// Set view mode
  void setViewMode(String mode) {
    state = state.copyWith(viewMode: mode);
  }

  /// Set loading
  void setLoading(bool loading) {
    state = state.copyWith(isLoading: loading);
  }

  /// Set error
  void setError(String? error) {
    state = state.copyWith(error: error);
  }

  /// Clear all
  void clearAll() {
    state = const CollaborationState();
  }

  /// Initialize mock data for demo
  void initializeMockData() {
    // Create mock agents
    final mockAgents = [
      {'id': 'planner-001', 'name': 'Planner', 'type': 'planner', 'load': 1, 'max': 3},
      {'id': 'architect-001', 'name': 'Architect', 'type': 'architect', 'load': 2, 'max': 4},
      {'id': 'tddGuide-001', 'name': 'TDD Guide', 'type': 'tddGuide', 'load': 1, 'max': 3},
      {'id': 'codeReviewer-001', 'name': 'Code Reviewer', 'type': 'codeReviewer', 'load': 0, 'max': 2},
      {'id': 'securityReviewer-001', 'name': 'Security Reviewer', 'type': 'securityReviewer', 'load': 1, 'max': 2},
    ];

    final nodes = <CollaborationNode>[];
    for (var i = 0; i < mockAgents.length; i++) {
      final agent = mockAgents[i];
      nodes.add(CollaborationNode(
        id: 'node-${agent['id']}',
        agentId: agent['id'] as String,
        agentName: agent['name'] as String,
        agentType: agent['type'] as String,
        position: Position(x: 100.0 + i * 150, y: 100.0 + (i % 3) * 100),
        status: (agent['load'] as int) > 0 ? 'active' : 'idle',
        capabilities: _getMockCapabilities(agent['type'] as String),
        currentLoad: agent['load'] as int,
        maxLoad: agent['max'] as int,
        description: '${agent['name']} - Specialized agent',
      ));
    }

    // Create mock task flows
    final mockTasks = [
      {'id': 'task-1', 'source': 'planner-001', 'target': 'architect-001', 'desc': 'Design architecture', 'status': 'flowing'},
      {'id': 'task-2', 'source': 'architect-001', 'target': 'tddGuide-001', 'desc': 'Implement TDD', 'status': 'pending'},
      {'id': 'task-3', 'source': 'codeReviewer-001', 'target': 'securityReviewer-001', 'desc': 'Security audit', 'status': 'completed'},
    ];

    final edges = <CollaborationEdge>[];
    for (final task in mockTasks) {
      edges.add(CollaborationEdge(
        id: 'edge-${task['id']}',
        sourceAgentId: 'node-${task['source']}',
        targetAgentId: 'node-${task['target']}',
        taskId: task['id'] as String,
        taskDescription: task['desc'] as String,
        status: task['status'] as String,
        timestamp: DateTime.now().millisecondsSinceEpoch - (mockTasks.indexOf(task) * 5000),
        messageType: 'task_assign',
        animationProgress: task['status'] == 'flowing' ? 0.5 : 1.0,
      ));
    }

    // Create mock events
    final events = <CollaborationEvent>[
      CollaborationEvent(
        id: 'event-1',
        type: 'task_assign',
        timestamp: DateTime.now().millisecondsSinceEpoch - 10000,
        sourceAgentId: 'planner-001',
        targetAgentId: 'architect-001',
        severity: 'info',
        payload: null,
        details: EventDetails(summary: 'Architecture design task assigned'),
      ),
      CollaborationEvent(
        id: 'event-2',
        type: 'tool_call',
        timestamp: DateTime.now().millisecondsSinceEpoch - 5000,
        sourceAgentId: 'architect-001',
        severity: 'info',
        payload: null,
        details: EventDetails(summary: 'File read operation', duration: 120),
      ),
    ];

    state = state.copyWith(nodes: nodes, edges: edges, events: events);
  }

  List<AgentCapability> _getMockCapabilities(String agentType) {
    final capabilitiesByType = <String, List<AgentCapability>>{
      'planner': [
        AgentCapability(id: 'cap-1', name: 'Task Planning', description: 'Break down tasks', icon: '📋', category: 'planning', proficiency: 90),
        AgentCapability(id: 'cap-2', name: 'Resource Allocation', description: 'Assign agents', icon: '🎭', category: 'orchestration', proficiency: 85),
      ],
      'architect': [
        AgentCapability(id: 'cap-3', name: 'System Design', description: 'Design architecture', icon: '🏗️', category: 'planning', proficiency: 95),
      ],
      'tddGuide': [
        AgentCapability(id: 'cap-5', name: 'Test Writing', description: 'Write tests', icon: '🧪', category: 'testing', proficiency: 92),
      ],
      'codeReviewer': [
        AgentCapability(id: 'cap-7', name: 'Code Analysis', description: 'Analyze code', icon: '🔍', category: 'review', proficiency: 90),
      ],
      'securityReviewer': [
        AgentCapability(id: 'cap-9', name: 'Security Audit', description: 'Audit security', icon: '🔒', category: 'review', proficiency: 95),
      ],
    };
    return capabilitiesByType[agentType] ?? [
      AgentCapability(id: 'cap-default', name: 'General', description: 'Execute tasks', icon: '🤖', category: 'execution', proficiency: 70),
    ];
  }

  /// Calculate stats
  CollaborationNetworkStats getStats() {
    final activeNodes = state.nodes.where((n) => n.status == 'active').length;
    final flowingEdges = state.edges.where((e) => e.status == 'flowing').length;
    final completedEdges = state.edges.where((e) => e.status == 'completed').length;
    final failedEdges = state.edges.where((e) => e.status == 'failed').length;

    // Calculate average task duration
    final completedWithDuration = state.edges
      .where((e) => e.status == 'completed' && e.duration != null)
      .toList();
    final avgDuration = completedWithDuration.isEmpty
      ? 0.0
      : completedWithDuration.fold<int>(0, (sum, e) => sum + (e.duration ?? 0)) /
        completedWithDuration.length;

    // Calculate collaboration efficiency
    final totalEdges = state.edges.length;
    final efficiency = totalEdges == 0
      ? 100.0
      : ((completedEdges / totalEdges) * 100).round();

    return CollaborationNetworkStats(
      totalAgents: state.nodes.length,
      activeAgents: activeNodes,
      totalTasks: totalEdges,
      runningTasks: flowingEdges,
      completedTasks: completedEdges,
      failedTasks: failedEdges,
      averageTaskDuration: avgDuration.round(),
      collaborationEfficiency: efficiency.round(),
      messageCount: state.events.where((e) => e.type.contains('message')).length,
      toolCallCount: state.events.where((e) => e.type.contains('tool')).length,
      protocolInvocations: state.events.where((e) => e.type == 'protocol_invoked').length,
    );
  }

  int get totalTasks => state.edges.length;
}

/// Provider for collaboration state
final collaborationProvider =
  StateNotifierProvider<CollaborationNotifier, CollaborationState>(
    (ref) => CollaborationNotifier()
  );

/// Provider for stats
final collaborationStatsProvider = Provider<CollaborationNetworkStats>((ref) {
  final notifier = ref.watch(collaborationProvider.notifier);
  return notifier.getStats();
});

/// Provider for selected node
final selectedNodeProvider = Provider<CollaborationNode?>((ref) {
  final state = ref.watch(collaborationProvider);
  if (state.selectedNodeId == null) return null;
  return state.nodes.where((n) => n.id == state.selectedNodeId).firstOrNull;
});

/// Provider for selected node capabilities
final selectedNodeCapabilitiesProvider = Provider<List<AgentCapability>>((ref) {
  final selectedNode = ref.watch(selectedNodeProvider);
  if (selectedNode == null) return const [];
  return selectedNode.capabilities;
});

/// Provider for selected node protocols
final selectedNodeProtocolsProvider = Provider<List<CollaborationProtocol>>((ref) {
  final selectedNode = ref.watch(selectedNodeProvider);
  final state = ref.watch(collaborationProvider);
  if (selectedNode == null) return const [];
  return state.protocols
    .where((p) => p.participants.contains(selectedNode.agentId))
    .toList();
});

/// Provider for active nodes
final activeNodesProvider = Provider<List<CollaborationNode>>((ref) {
  final state = ref.watch(collaborationProvider);
  return state.nodes.where((n) => n.status == 'active').toList();
});

/// Provider for flowing edges
final flowingEdgesProvider = Provider<List<CollaborationEdge>>((ref) {
  final state = ref.watch(collaborationProvider);
  return state.edges.where((e) => e.status == 'flowing').toList();
});

/// Provider for recent events
final recentEventsProvider = Provider<List<CollaborationEvent>>((ref) {
  final state = ref.watch(collaborationProvider);
  final now = DateTime.now().millisecondsSinceEpoch;
  final fiveMinutes = 5 * 60 * 1000;
  return state.events.where((e) => now - e.timestamp < fiveMinutes).toList();
});