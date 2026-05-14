import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/agent/agent_pool.dart';
import '../../core/agent/agent_bridge.dart';
import '../models/agent.dart';

/// Agent Pool Provider
final agentPoolProvider = Provider<AgentPool>((ref) {
  return AgentPool(
    baseUrl: 'ws://localhost:8080',
    maxAgents: 10,
  );
});

/// Active Agents Provider
final activeAgentsProvider = Provider<List<AgentBridge>>((ref) {
  final pool = ref.watch(agentPoolProvider);
  return pool.getActiveAgents();
});

/// Selected Agent Provider
final selectedAgentProvider = StateProvider<Agent?>((ref) {
  return null;
});

/// Agent Status Provider
final agentStatusProvider = StateNotifierProvider<AgentStatusNotifier, Map<String, AgentStatus>>((ref) {
  return AgentStatusNotifier();
});

class AgentStatusNotifier extends StateNotifier<Map<String, AgentStatus>> {
  AgentStatusNotifier() : super({});

  void updateStatus(String agentId, AgentStatus status) {
    state = {...state, agentId: status};
  }

  void setIdle(String agentId) => updateStatus(agentId, AgentStatus.idle);
  void setBusy(String agentId) => updateStatus(agentId, AgentStatus.busy);
  void setError(String agentId) => updateStatus(agentId, AgentStatus.error);
  void setOffline(String agentId) => updateStatus(agentId, AgentStatus.offline);
}

/// Agent List Provider (mock data for now)
final agentListProvider = Provider<List<Agent>>((ref) {
  return [
    Agent(
      id: 'planner-1',
      name: 'planner',
      type: AgentType.planner,
      status: AgentStatus.idle,
      description: 'Implementation planning agent',
      capabilities: ['planning', 'breaking_down_tasks'],
      createdAt: DateTime.now(),
    ),
    Agent(
      id: 'architect-1',
      name: 'architect',
      type: AgentType.architect,
      status: AgentStatus.idle,
      description: 'System design agent',
      capabilities: ['architecture', 'system_design'],
      createdAt: DateTime.now(),
    ),
    Agent(
      id: 'code-reviewer-1',
      name: 'code-reviewer',
      type: AgentType.codeReviewer,
      status: AgentStatus.idle,
      description: 'Code review specialist',
      capabilities: ['code_review', 'linting', 'security_scan'],
      createdAt: DateTime.now(),
    ),
    Agent(
      id: 'tdd-guide-1',
      name: 'tdd-guide',
      type: AgentType.tddGuide,
      status: AgentStatus.idle,
      description: 'Test-driven development guide',
      capabilities: ['testing', 'tdd', 'coverage'],
      createdAt: DateTime.now(),
    ),
    Agent(
      id: 'security-reviewer-1',
      name: 'security-reviewer',
      type: AgentType.securityReviewer,
      status: AgentStatus.idle,
      description: 'Security vulnerability detector',
      capabilities: ['security', 'owasp', 'vulnerability_scan'],
      createdAt: DateTime.now(),
    ),
  ];
});