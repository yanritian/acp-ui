/// Smart Router - 任务复杂度分析与路由决策
///
/// 调用 Tauri 后端的 analyze_task_complexity 命令

import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';

/// 任务复杂度级别
enum TaskComplexity {
  simple,
  medium,
  complex,
  veryComplex,
}

/// 任务类型
enum TaskType {
  frontend,
  backend,
  fullstack,
  devops,
  testing,
  documentation,
  refactoring,
  bugfix,
}

/// 输入类型
enum InputType {
  text,
  image,
  document,
  code,
  command,
}

/// 路由目标
enum RouteTarget {
  claudeCodeHaiku,
  claudeCodeSonnet,
  codexHaiku,
  codexSonnet,
  team,
  humanReview,
}

/// 评估方法
enum EvaluationMethod {
  heuristic,
  structural,
  llmAssisted,
}

/// 路由决策结果
class RouteDecision {
  final String id;
  final TaskType taskType;
  final InputType inputType;
  final TaskComplexity complexity;
  final RouteTarget routeTarget;
  final String reason;
  final int agentLoad;
  final double historicalSuccessRate;
  final EvaluationMethod evaluationMethod;
  final DateTime createdAt;

  RouteDecision({
    required this.id,
    required this.taskType,
    required this.inputType,
    required this.complexity,
    required this.routeTarget,
    required this.reason,
    this.agentLoad = 0,
    this.historicalSuccessRate = 0.8,
    required this.evaluationMethod,
    required this.createdAt,
  });

  factory RouteDecision.fromJson(Map<String, dynamic> json) {
    return RouteDecision(
      id: json['id'] as String,
      taskType: _parseTaskType(json['task_type'] as String),
      inputType: _parseInputType(json['input_type'] as String),
      complexity: _parseComplexity(json['complexity'] as String),
      routeTarget: _parseRouteTarget(json['route_target'] as String),
      reason: json['reason'] as String,
      agentLoad: json['agent_load'] as int? ?? 0,
      historicalSuccessRate: (json['historical_success_rate'] as num?)?.toDouble() ?? 0.8,
      evaluationMethod: _parseEvaluationMethod(json['evaluation_method'] as String),
      createdAt: DateTime.parse(json['created_at'] as String),
    );
  }

  static TaskType _parseTaskType(String s) {
    return TaskType.values.firstWhere(
      (e) => e.name == s,
      orElse: () => TaskType.fullstack,
    );
  }

  static InputType _parseInputType(String s) {
    return InputType.values.firstWhere(
      (e) => e.name == s,
      orElse: () => InputType.text,
    );
  }

  static TaskComplexity _parseComplexity(String s) {
    return TaskComplexity.values.firstWhere(
      (e) => e.name.replaceAll('_', '') == s.replaceAll('_', ''),
      orElse: () => TaskComplexity.medium,
    );
  }

  static RouteTarget _parseRouteTarget(String s) {
    return RouteTarget.values.firstWhere(
      (e) => e.name.replaceAll('_', '') == s.replaceAll('-', ''),
      orElse: () => RouteTarget.team,
    );
  }

  static EvaluationMethod _parseEvaluationMethod(String s) {
    return EvaluationMethod.values.firstWhere(
      (e) => e.name.replaceAll('_', '') == s.replaceAll('_', ''),
      orElse: () => EvaluationMethod.heuristic,
    );
  }

  Map<String, dynamic> toJson() => {
    'id': id,
    'task_type': taskType.name,
    'input_type': inputType.name,
    'complexity': complexity.name,
    'route_target': routeTarget.name.replaceAll('_', '-'),
    'reason': reason,
    'agent_load': agentLoad,
    'historical_success_rate': historicalSuccessRate,
    'evaluation_method': evaluationMethod.name,
    'created_at': createdAt.toIso8601String(),
  };
}

/// Smart Router 服务
class SmartRouter {
  final WebSocketService? _wsService;

  SmartRouter({WebSocketService? wsService}) : _wsService = wsService;

  /// 分析任务复杂度
  Future<RouteDecision> analyze(
    String input,
    InputType inputType,
  ) async {
    if (_wsService != null && _wsService!.isConnectedGetter) {
      // WebSocket 方式调用 Tauri 命令
      final request = {
        'command': 'analyze_task_complexity',
        'args': {
          'input': input,
          'input_type': inputType.name,
        },
      };

      // TODO: 实现 WebSocket 响应等待
      // final response = await _wsService.sendAndWait(request);
      // return RouteDecision.fromJson(response);
    }

    // 本地启发式分析（备用）
    return _localAnalyze(input, inputType);
  }

  /// 本地启发式分析
  RouteDecision _localAnalyze(String input, InputType inputType) {
    final complexity = _determineComplexity(input);
    final taskType = _determineTaskType(input);
    final target = _selectTarget(taskType, complexity);

    return RouteDecision(
      id: 'local-${DateTime.now().millisecondsSinceEpoch}',
      taskType: taskType,
      inputType: inputType,
      complexity: complexity,
      routeTarget: target,
      reason: 'Local heuristic analysis',
      evaluationMethod: EvaluationMethod.heuristic,
      createdAt: DateTime.now(),
    );
  }

  TaskComplexity _determineComplexity(String input) {
    final lower = input.toLowerCase();
    if (lower.contains('修改一行') || lower.contains('fix') || lower.contains('bug')) {
      return TaskComplexity.simple;
    }
    if (lower.contains('refactor') || lower.contains('重构') || lower.contains('添加')) {
      return TaskComplexity.medium;
    }
    if (lower.contains('跨模块') || lower.contains('新功能') || lower.contains('feature')) {
      return TaskComplexity.complex;
    }
    if (lower.contains('多agent') || lower.contains('system') || lower.contains('架构')) {
      return TaskComplexity.veryComplex;
    }
    return TaskComplexity.medium;
  }

  TaskType _determineTaskType(String input) {
    final lower = input.toLowerCase();
    if (lower.contains('vue') || lower.contains('ui') || lower.contains('前端')) {
      return TaskType.frontend;
    }
    if (lower.contains('api') || lower.contains('backend') || lower.contains('数据库')) {
      return TaskType.backend;
    }
    if (lower.contains('test') || lower.contains('测试')) {
      return TaskType.testing;
    }
    if (lower.contains('doc') || lower.contains('文档')) {
      return TaskType.documentation;
    }
    return TaskType.fullstack;
  }

  RouteTarget _selectTarget(TaskType taskType, TaskComplexity complexity) {
    if (complexity == TaskComplexity.veryComplex) {
      return RouteTarget.team;
    }
    switch (taskType) {
      case TaskType.frontend:
        return complexity == TaskComplexity.simple
            ? RouteTarget.claudeCodeHaiku
            : RouteTarget.claudeCodeSonnet;
      case TaskType.backend:
        return complexity == TaskComplexity.simple
            ? RouteTarget.codexHaiku
            : RouteTarget.codexSonnet;
      default:
        return RouteTarget.claudeCodeSonnet;
    }
  }
}

/// WebSocketService 类型占位（实际从 services 导入）
class WebSocketService {
  final String url;
  WebSocketService({required this.url});
  bool isConnectedGetter = false;
}