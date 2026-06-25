// Agent Platform Cubit - State Management using flutter_bloc

import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:equatable/equatable.dart';
import '../../../data/services/agent_platform_api.dart';

// States

abstract class AgentPlatformState extends Equatable {
  const AgentPlatformState();

  @override
  List<Object?> get props => [];
}

class AgentPlatformInitial extends AgentPlatformState {}

class AgentPlatformLoading extends AgentPlatformState {}

class AgentsLoaded extends AgentPlatformState {
  final List<AgentSummary> agents;
  final AgentSummary? selectedAgent;

  const AgentsLoaded({required this.agents, this.selectedAgent});

  @override
  List<Object?> get props => [agents, selectedAgent];
}

class OneShotExecuting extends AgentPlatformState {
  final String input;

  const OneShotExecuting({required this.input});

  @override
  List<Object?> get props => [input];
}

class OneShotCompleted extends AgentPlatformState {
  final OneShotResponse result;
  final List<AgentSummary> agents;

  const OneShotCompleted({required this.result, required this.agents});

  @override
  List<Object?> get props => [result, agents];
}

class CostLoaded extends AgentPlatformState {
  final CostSummary summary;

  const CostLoaded({required this.summary});

  @override
  List<Object?> get props => [summary];
}

class AgentPlatformError extends AgentPlatformState {
  final String message;

  const AgentPlatformError({required this.message});

  @override
  List<Object?> get props => [message];
}

// Cubit

class AgentPlatformCubit extends Cubit<AgentPlatformState> {
  final AgentApiService _agentApi;
  final OneShotApiService _oneShotApi;
  final CostApiService _costApi;

  List<AgentSummary> _agents = [];

  AgentPlatformCubit({
    AgentApiService? agentApi,
    OneShotApiService? oneShotApi,
    CostApiService? costApi,
  }) : _agentApi = agentApi ?? AgentApiService(),
       _oneShotApi = oneShotApi ?? OneShotApiService(),
       _costApi = costApi ?? CostApiService(),
       super(AgentPlatformInitial()) {}

  // Fetch all agents
  Future<void> fetchAgents() async {
    emit(AgentPlatformLoading());
    try {
      _agents = await _agentApi.listAgents();
      emit(AgentsLoaded(agents: _agents));
    } catch (e) {
      emit(AgentPlatformError(message: e.toString()));
    }
  }

  // Select an agent
  void selectAgent(AgentSummary agent) {
    final currentState = state;
    if (currentState is AgentsLoaded) {
      emit(AgentsLoaded(agents: _agents, selectedAgent: agent));
    }
  }

  // Execute one-shot request
  Future<void> executeOneShot({
    required String input,
    String? preferredAgent,
    double? maxCost,
  }) async {
    emit(OneShotExecuting(input: input));
    try {
      final request = OneShotRequest(
        input: input,
        preferredAgent: preferredAgent,
        maxCost: maxCost,
      );
      final result = await _oneShotApi.execute(request);
      emit(OneShotCompleted(result: result, agents: _agents));
    } catch (e) {
      emit(AgentPlatformError(message: e.toString()));
    }
  }

  // Fetch cost summary
  Future<void> fetchCostSummary() async {
    emit(AgentPlatformLoading());
    try {
      final summary = await _costApi.getSummary();
      emit(CostLoaded(summary: summary));
    } catch (e) {
      emit(AgentPlatformError(message: e.toString()));
    }
  }

  // Clear error
  void clearError() {
    if (_agents.isNotEmpty) {
      emit(AgentsLoaded(agents: _agents));
    } else {
      emit(AgentPlatformInitial());
    }
  }

  // Get available agents
  List<AgentSummary> get availableAgents =>
      _agents.where((a) => a.status == 'available').toList();

  // Get top agents by health score
  List<AgentSummary> get topAgents {
    final sorted = List<AgentSummary>.from(_agents);
    sorted.sort((a, b) => b.healthScore.compareTo(a.healthScore));
    return sorted.take(3).toList();
  }
}