// Agent Platform API Service
// Connects to Tauri HTTP Server for Mobile access

import 'dart:convert';
import 'package:http/http.dart' as http;

const String API_BASE = 'http://localhost:3000/api';

class AgentApiService {
  // List all available agents
  Future<List<AgentSummary>> listAgents() async {
    final response = await http.get(Uri.parse('$API_BASE/agents'));
    if (response.statusCode != 200) {
      throw Exception('Failed to fetch agents: ${response.statusCode}');
    }
    final List<dynamic> data = jsonDecode(response.body);
    return data.map((json) => AgentSummary.fromJson(json)).toList();
  }

  // Get specific agent
  Future<AgentSummary> getAgent(String id) async {
    final response = await http.get(Uri.parse('$API_BASE/agents/$id'));
    if (response.statusCode != 200) {
      throw Exception('Agent $id not found');
    }
    return AgentSummary.fromJson(jsonDecode(response.body));
  }
}

class OneShotApiService {
  // Execute one-shot request
  Future<OneShotResponse> execute(OneShotRequest request) async {
    final response = await http.post(
      Uri.parse('$API_BASE/oneshot'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode(request.toJson()),
    );
    if (response.statusCode != 200) {
      throw Exception('One-shot execution failed: ${response.statusCode}');
    }
    return OneShotResponse.fromJson(jsonDecode(response.body));
  }
}

class CostApiService {
  // Get cost summary
  Future<CostSummary> getSummary() async {
    final response = await http.get(Uri.parse('$API_BASE/costs/summary'));
    if (response.statusCode != 200) {
      throw Exception('Failed to fetch cost summary');
    }
    return CostSummary.fromJson(jsonDecode(response.body));
  }

  // Set budget
  Future<void> setBudget(CostBudget budget) async {
    final response = await http.post(
      Uri.parse('$API_BASE/costs/budget'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode(budget.toJson()),
    );
    if (response.statusCode != 200) {
      throw Exception('Failed to set budget');
    }
  }
}

class ExecuteApiService {
  // Execute task with specific agent
  Future<ExecuteResponse> execute(ExecuteRequest request) async {
    final response = await http.post(
      Uri.parse('$API_BASE/execute'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode(request.toJson()),
    );
    if (response.statusCode != 200) {
      throw Exception('Execution failed: ${response.statusCode}');
    }
    return ExecuteResponse.fromJson(jsonDecode(response.body));
  }
}

// Models

class AgentSummary {
  final String id;
  final String name;
  final String adapterType;
  final String status;
  final double healthScore;
  final List<String> capabilities;

  AgentSummary({
    required this.id,
    required this.name,
    required this.adapterType,
    required this.status,
    required this.healthScore,
    required this.capabilities,
  });

  factory AgentSummary.fromJson(Map<String, dynamic> json) {
    return AgentSummary(
      id: json['id'],
      name: json['name'],
      adapterType: json['adapter_type'],
      status: json['status'],
      healthScore: json['health_score'],
      capabilities: List<String>.from(json['capabilities']),
    );
  }
}

class OneShotRequest {
  final String input;
  final String? contextHint;
  final String? preferredAgent;
  final double? maxCost;
  final int? timeoutMs;

  OneShotRequest({
    required this.input,
    this.contextHint,
    this.preferredAgent,
    this.maxCost,
    this.timeoutMs,
  });

  Map<String, dynamic> toJson() {
    return {
      'input': input,
      'context_hint': contextHint,
      'preferred_agent': preferredAgent,
      'max_cost': maxCost,
      'timeout_ms': timeoutMs,
    };
  }
}

class OneShotResponse {
  final String detectedRole;
  final String detectedScene;
  final String selectedAgent;
  final String selectionReason;
  final String resultPreview;
  final double estimatedCost;
  final TransparencyInfo transparency;

  OneShotResponse({
    required this.detectedRole,
    required this.detectedScene,
    required this.selectedAgent,
    required this.selectionReason,
    required this.resultPreview,
    required this.estimatedCost,
    required this.transparency,
  });

  factory OneShotResponse.fromJson(Map<String, dynamic> json) {
    return OneShotResponse(
      detectedRole: json['detected_role'],
      detectedScene: json['detected_scene'],
      selectedAgent: json['selected_agent'],
      selectionReason: json['selection_reason'],
      resultPreview: json['result_preview'],
      estimatedCost: json['estimated_cost'],
      transparency: TransparencyInfo.fromJson(json['transparency']),
    );
  }
}

class TransparencyInfo {
  final String dagVisualization;
  final int estimatedDurationMs;
  final String privacyLevel;

  TransparencyInfo({
    required this.dagVisualization,
    required this.estimatedDurationMs,
    required this.privacyLevel,
  });

  factory TransparencyInfo.fromJson(Map<String, dynamic> json) {
    return TransparencyInfo(
      dagVisualization: json['dag_visualization'],
      estimatedDurationMs: json['estimated_duration_ms'],
      privacyLevel: json['privacy_level'],
    );
  }
}

class CostSummary {
  final double dailyTotal;
  final double monthlyTotal;
  final String currency;
  final double budgetRemainingPercent;

  CostSummary({
    required this.dailyTotal,
    required this.monthlyTotal,
    required this.currency,
    required this.budgetRemainingPercent,
  });

  factory CostSummary.fromJson(Map<String, dynamic> json) {
    return CostSummary(
      dailyTotal: json['daily_total'],
      monthlyTotal: json['monthly_total'],
      currency: json['currency'],
      budgetRemainingPercent: json['budget_remaining_percent'],
    );
  }
}

class CostBudget {
  final double dailyLimit;
  final double monthlyLimit;

  CostBudget({
    required this.dailyLimit,
    required this.monthlyLimit,
  });

  Map<String, dynamic> toJson() {
    return {
      'daily_limit': dailyLimit,
      'monthly_limit': monthlyLimit,
    };
  }
}

class ExecuteRequest {
  final String agentId;
  final String description;
  final String input;
  final String inputType;
  final int? timeoutMs;

  ExecuteRequest({
    required this.agentId,
    required this.description,
    required this.input,
    this.inputType = 'text',
    this.timeoutMs,
  });

  Map<String, dynamic> toJson() {
    return {
      'agent_id': agentId,
      'description': description,
      'input': input,
      'input_type': inputType,
      'timeout_ms': timeoutMs,
    };
  }
}

class ExecuteResponse {
  final String taskId;
  final String status;
  final String outputPreview;
  final double cost;
  final int durationMs;

  ExecuteResponse({
    required this.taskId,
    required this.status,
    required this.outputPreview,
    required this.cost,
    required this.durationMs,
  });

  factory ExecuteResponse.fromJson(Map<String, dynamic> json) {
    return ExecuteResponse(
      taskId: json['task_id'],
      status: json['status'],
      outputPreview: json['output_preview'],
      cost: json['cost'],
      durationMs: json['duration_ms'],
    );
  }
}