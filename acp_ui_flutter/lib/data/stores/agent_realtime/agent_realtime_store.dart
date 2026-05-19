/// Agent Realtime Store
/// 实时进度状态管理

import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../models/agent_realtime/agent_realtime_types.dart';

/// Agent Realtime State
class AgentRealtimeState {
  final Map<String, AgentRealtimeStatus> agents;
  final String? activeAgentId;
  final List<RealtimeEvent> globalEvents;
  final int totalCompletedTasks;
  final int totalFailedTasks;
  final double globalSuccessRate;
  final bool isConnected;

  const AgentRealtimeState({
    this.agents = const {},
    this.activeAgentId,
    this.globalEvents = const [],
    this.totalCompletedTasks = 0,
    this.totalFailedTasks = 0,
    this.globalSuccessRate = 1.0,
    this.isConnected = false,
  });

  AgentRealtimeState copyWith({
    Map<String, AgentRealtimeStatus>? agents,
    String? activeAgentId,
    List<RealtimeEvent>? globalEvents,
    int? totalCompletedTasks,
    int? totalFailedTasks,
    double? globalSuccessRate,
    bool? isConnected,
  }) {
    return AgentRealtimeState(
      agents: agents ?? this.agents,
      activeAgentId: activeAgentId ?? this.activeAgentId,
      globalEvents: globalEvents ?? this.globalEvents,
      totalCompletedTasks: totalCompletedTasks ?? this.totalCompletedTasks,
      totalFailedTasks: totalFailedTasks ?? this.totalFailedTasks,
      globalSuccessRate: globalSuccessRate ?? this.globalSuccessRate,
      isConnected: isConnected ?? this.isConnected,
    );
  }

  AgentRealtimeStatus? get activeAgent =>
      activeAgentId != null ? agents[activeAgentId] : null;

  List<AgentRealtimeStatus> get allAgents => agents.values.toList();

  int get thinkingCount =>
      agents.values.where((a) => a.currentActivity.type == AgentActivityType.thinking).length;

  int get executingCount =>
      agents.values.where((a) => a.currentActivity.type == AgentActivityType.executing).length;
}

/// Agent Realtime Store
class AgentRealtimeStore extends StateNotifier<AgentRealtimeState> {
  AgentRealtimeStore() : super(const AgentRealtimeState()) {
    _initializeMockData();
  }

  void _initializeMockData() {
    // Create mock agents
    final mockAgents = <String, AgentRealtimeStatus>{
      'planner-001': _createMockAgent('planner-001', 'Planner', AgentActivityType.thinking),
      'architect-001': _createMockAgent('architect-001', 'Architect', AgentActivityType.executing),
      'tddGuide-001': _createMockAgent('tddGuide-001', 'TDD Guide', AgentActivityType.idle),
      'codeReviewer-001': _createMockAgent('codeReviewer-001', 'Code Reviewer', AgentActivityType.idle),
    };

    state = state.copyWith(
      agents: mockAgents,
      activeAgentId: 'planner-001',
      isConnected: true,
      totalCompletedTasks: 1,
      globalSuccessRate: 1.0,
    );
  }

  AgentRealtimeStatus _createMockAgent(String id, String name, AgentActivityType activityType) {
    return AgentRealtimeStatus(
      agentId: id,
      agentName: name,
      currentActivity: CurrentActivity(
        type: activityType,
        startTime: DateTime.now().millisecondsSinceEpoch,
        duration: 0,
        progress: 50,
        description: activityType == AgentActivityType.thinking ? '分析任务需求...' : null,
      ),
      thinking: ThinkingState(
        content: '首先需要了解用户的具体需求...',
        startTime: DateTime.now().millisecondsSinceEpoch,
        depth: 2,
        chunks: [
          ThinkingChunk(
            id: 'chunk-1',
            content: '首先需要了解用户的具体需求...',
            timestamp: DateTime.now().millisecondsSinceEpoch,
            duration: 150,
            depth: 2,
          ),
        ],
        isStreaming: activityType == AgentActivityType.thinking,
      ),
      output: OutputState(
        content: '',
        chunks: const [],
        totalLength: 0,
        currentPosition: 0,
        isStreaming: false,
      ),
      stats: AgentStats(
        tasksCompleted: activityType == AgentActivityType.idle ? 0 : 1,
        tasksFailed: 0,
        averageDuration: 5000,
        successRate: 1.0,
        totalThinkingTime: 150,
        totalToolCalls: 0,
      ),
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );
  }

  void setActiveAgent(String agentId) {
    state = state.copyWith(activeAgentId: agentId);
  }

  void registerAgent(AgentRealtimeStatus agent) {
    final newAgents = Map<String, AgentRealtimeStatus>.from(state.agents);
    newAgents[agent.agentId] = agent;
    state = state.copyWith(agents: newAgents);
  }

  void removeAgent(String agentId) {
    final newAgents = Map<String, AgentRealtimeStatus>.from(state.agents);
    newAgents.remove(agentId);
    state = state.copyWith(
      agents: newAgents,
      activeAgentId: state.activeAgentId == agentId ? null : state.activeAgentId,
    );
  }

  void updateAgentStatus(String agentId, AgentRealtimeStatus status) {
    final newAgents = Map<String, AgentRealtimeStatus>.from(state.agents);
    newAgents[agentId] = status;
    state = state.copyWith(agents: newAgents);
  }

  void addGlobalEvent(RealtimeEvent event) {
    final newEvents = List<RealtimeEvent>.from(state.globalEvents);
    newEvents.insert(0, event);
    if (newEvents.length > 50) newEvents.removeLast();
    state = state.copyWith(globalEvents: newEvents);
  }

  void setConnectionStatus(bool connected) {
    state = state.copyWith(isConnected: connected);
  }

  void respondToPermission(String agentId, String optionId) {
    final agent = state.agents[agentId];
    if (agent == null || agent.permissionWaiting == null) return;

    final newAgents = Map<String, AgentRealtimeStatus>.from(state.agents);
    newAgents[agentId] = AgentRealtimeStatus(
      agentId: agent.agentId,
      agentName: agent.agentName,
      currentActivity: CurrentActivity(
        type: AgentActivityType.executing,
        startTime: DateTime.now().millisecondsSinceEpoch,
        duration: 0,
        progress: 0,
      ),
      thinking: agent.thinking,
      output: agent.output,
      stats: agent.stats,
      lastUpdateTime: DateTime.now().millisecondsSinceEpoch,
    );
    state = state.copyWith(agents: newAgents);
  }
}

/// Provider
final agentRealtimeProvider =
    StateNotifierProvider<AgentRealtimeStore, AgentRealtimeState>(
  (ref) => AgentRealtimeStore(),
);