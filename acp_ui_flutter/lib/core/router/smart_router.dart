/// Smart Router - 任务复杂度分析
///
/// 调用 Tauri 命令进行三层渐进复杂度评估

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

/// 路由决策
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
      (e) => e.name == s.replaceAll('_', ''),
      orElse: () => TaskComplexity.medium,
    );
  }

  static RouteTarget _parseRouteTarget(String s) {
    return RouteTarget.values.firstWhere(
      (e) => e.name == s.replaceAll('-', '_'),
      orElse: () => RouteTarget.team,
    );
  }

  static EvaluationMethod _parseEvaluationMethod(String s) {
    return EvaluationMethod.values.firstWhere(
      (e) => e.name == s.replaceAll('_', ''),
      orElse: () => EvaluationMethod.heuristic,
    );
  }
}

/// Smart Router 服务
class SmartRouter {
  /// 分析任务复杂度
  ///
  /// 通过 WebSocket 或 HTTP 调用 Tauri 后端的 analyze_task_complexity 命令
  Future<RouteDecision> analyze(
    String input,
    InputType inputType,
  ) async {
    // WebSocket 方式调用 Tauri 命令
    // 实际实现需要通过 WebSocketService
    final request = {
      'command': 'analyze_task_complexity',
      'args': {
        'input': input,
        'input_type': inputType.name,
      },
    };

    // TODO: 通过 WebSocket 发送请求并接收响应
    // final response = await _wsService.sendCommand(request);

    // 临时返回模拟决策（实际需要 WebSocket 连接）
    return RouteDecision(
      id: 'mock-${DateTime.now().millisecondsSinceEpoch}',
      taskType: TaskType.fullstack,
      inputType: inputType,
      complexity: TaskComplexity.medium,
      routeTarget: RouteTarget.claudeCodeSonnet,
      reason: 'Mock decision - WebSocket not connected',
      evaluationMethod: EvaluationMethod.heuristic,
      createdAt: DateTime.now(),
    );
  }

  /// 本地启发式分析（备用）
  RouteDecision localAnalyze(String input, InputType inputType) {
    // 简单的启发式规则
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
    if (input.contains('修改一行') || input.contains('修复bug')) {
      return TaskComplexity.simple;
    }
    if (input.contains('重构') || input.contains('添加功能')) {
      return TaskComplexity.medium;
    }
    if (input.contains('跨模块') || input.contains('新功能')) {
      return TaskComplexity.complex;
    }
    if (input.contains('多agent') || input.contains('系统设计')) {
      return TaskComplexity.veryComplex;
    }
    return TaskComplexity.medium;
  }

  TaskType _determineTaskType(String input) {
    if (input.contains('vue') || input.contains('前端') || input.contains('ui')) {
      return TaskType.frontend;
    }
    if (input.contains('api') || input.contains('后端') || input.contains('数据库')) {
      return TaskType.backend;
    }
    if (input.contains('测试') || input.contains('qa')) {
      return TaskType.testing;
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