/// Team DAG - DAG 执行引擎
///
/// 支持并行/顺序/混合执行策略

import 'package:flutter/foundation.dart';

/// 执行策略
enum ExecutionStrategy {
  parallel,
  sequential,
  hybrid,
}

/// 节点状态
enum NodeStatus {
  pending,
  waitingSync,
  running,
  completed,
  failed,
  skipped,
}

/// 执行状态
enum ExecutionStatus {
  planning,
  pendingSync,
  running,
  paused,
  completed,
  failed,
  cancelled,
}

/// 团队成员
class TeamMember {
  final String agentId;
  final String templateId;
  final String role;
  final List<String> capabilities;
  final List<String> dependencies;

  TeamMember({
    required this.agentId,
    required this.templateId,
    required this.role,
    this.capabilities = [],
    this.dependencies = [],
  });

  Map<String, dynamic> toJson() => {
    'agent_id': agentId,
    'template_id': templateId,
    'role': role,
    'capabilities': capabilities,
    'dependencies': dependencies,
  };
}

/// 同步点
class SyncPoint {
  final String id;
  final String name;
  final List<String> waitFor;
  final String triggerAction;
  final int timeoutMs;

  SyncPoint({
    required this.id,
    required this.name,
    this.waitFor = [],
    this.triggerAction = '',
    this.timeoutMs = 30000,
  });

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'wait_for': waitFor,
    'trigger_action': triggerAction,
    'timeout_ms': timeoutMs,
  };
}

/// DAG 节点
class DAGNode {
  final String id;
  final String memberId;
  final List<String> dependencies;
  final NodeStatus status;
  final DateTime? startedAt;
  final DateTime? completedAt;
  final String? result;

  DAGNode({
    required this.id,
    required this.memberId,
    this.dependencies = [],
    this.status = NodeStatus.pending,
    this.startedAt,
    this.completedAt,
    this.result,
  });

  factory DAGNode.fromJson(Map<String, dynamic> json) {
    return DAGNode(
      id: json['id'] as String,
      memberId: json['member_id'] as String,
      dependencies: (json['dependencies'] as List?)?.map((e) => e as String).toList() ?? [],
      status: _parseNodeStatus(json['status'] as String),
      startedAt: json['started_at'] != null
          ? DateTime.parse(json['started_at'] as String)
          : null,
      completedAt: json['completed_at'] != null
          ? DateTime.parse(json['completed_at'] as String)
          : null,
      result: json['result'] as String?,
    );
  }

  static NodeStatus _parseNodeStatus(String s) {
    return NodeStatus.values.firstWhere(
      (e) => e.name == s.replaceAll('_', ''),
      orElse: () => NodeStatus.pending,
    );
  }
}

/// 执行计划
class ExecutionPlan {
  final String id;
  final String teamId;
  final ExecutionStrategy strategy;
  final List<DAGNode> nodes;
  final List<SyncPoint> syncPoints;
  final String? currentSyncPoint;
  final ExecutionStatus status;
  final DateTime createdAt;
  final DateTime? startedAt;
  final DateTime? completedAt;

  ExecutionPlan({
    required this.id,
    required this.teamId,
    required this.strategy,
    this.nodes = [],
    this.syncPoints = [],
    this.currentSyncPoint,
    this.status = ExecutionStatus.planning,
    required this.createdAt,
    this.startedAt,
    this.completedAt,
  });

  factory ExecutionPlan.fromJson(Map<String, dynamic> json) {
    return ExecutionPlan(
      id: json['id'] as String,
      teamId: json['team_id'] as String,
      strategy: _parseStrategy(json['strategy'] as String),
      nodes: (json['nodes'] as List?)?.map((e) => DAGNode.fromJson(e as Map<String, dynamic>)).toList() ?? [],
      syncPoints: (json['sync_points'] as List?)?.map((e) => SyncPoint(
        id: e['id'] as String,
        name: e['name'] as String,
        waitFor: (e['wait_for'] as List?)?.map((x) => x as String).toList() ?? [],
        triggerAction: e['trigger_action'] as String? ?? '',
        timeoutMs: e['timeout_ms'] as int? ?? 30000,
      )).toList() ?? [],
      currentSyncPoint: json['current_sync_point'] as String?,
      status: _parseExecutionStatus(json['status'] as String),
      createdAt: DateTime.parse(json['created_at'] as String),
      startedAt: json['started_at'] != null
          ? DateTime.parse(json['started_at'] as String)
          : null,
      completedAt: json['completed_at'] != null
          ? DateTime.parse(json['completed_at'] as String)
          : null,
    );
  }

  static ExecutionStrategy _parseStrategy(String s) {
    return ExecutionStrategy.values.firstWhere(
      (e) => e.name == s,
      orElse: () => ExecutionStrategy.hybrid,
    );
  }

  static ExecutionStatus _parseExecutionStatus(String s) {
    return ExecutionStatus.values.firstWhere(
      (e) => e.name == s.replaceAll('_', ''),
      orElse: () => ExecutionStatus.planning,
    );
  }

  double get progress {
    if (nodes.isEmpty) return 0.0;
    final completed = nodes.where((n) => n.status == NodeStatus.completed).length;
    return completed / nodes.length;
  }
}

/// DAG 执行引擎
class DAGEngine {
  final Map<String, ExecutionPlan> _plans = {};

  /// 创建执行计划
  ///
  /// 通过 WebSocket 调用 Tauri 命令 create_dag_plan
  Future<ExecutionPlan> createPlan(
    String teamId,
    List<TeamMember> members,
    List<SyncPoint> syncPoints,
    ExecutionStrategy strategy,
  ) async {
    // TODO: WebSocket 调用
    // final response = await _wsService.sendCommand({
    //   'command': 'create_dag_plan',
    //   'args': {
    //     'team_id': teamId,
    //     'members': members.map((m) => m.toJson()).toList(),
    //     'sync_points': syncPoints.map((s) => s.toJson()).toList(),
    //     'strategy': strategy.name,
    //   },
    // });
    // final plan = ExecutionPlan.fromJson(response);
    // _plans[plan.id] = plan;
    // return plan;

    // 本地创建（备用）
    final planId = 'plan-${DateTime.now().millisecondsSinceEpoch}';
    final nodes = members.map((m) => DAGNode(
      id: 'node-${m.agentId}',
      memberId: m.agentId,
      dependencies: m.dependencies,
      status: NodeStatus.pending,
    )).toList();

    final plan = ExecutionPlan(
      id: planId,
      teamId: teamId,
      strategy: strategy,
      nodes: nodes,
      syncPoints: syncPoints,
      createdAt: DateTime.now(),
    );
    _plans[planId] = plan;
    return plan;
  }

  /// 获取计划进度
  Future<double> getProgress(String planId) async {
    // TODO: WebSocket 调用
    // final response = await _wsService.sendCommand({
    //   'command': 'get_dag_plan_progress',
    //   'args': {'plan_id': planId},
    // });
    // return (response as num).toDouble();

    final plan = _plans[planId];
    return plan?.progress ?? 0.0;
  }

  /// 获取计划
  ExecutionPlan? getPlan(String planId) {
    return _plans[planId];
  }

  /// 获取就绪节点
  List<DAGNode> getReadyNodes(ExecutionPlan plan) {
    final completedIds = plan.nodes
        .where((n) => n.status == NodeStatus.completed)
        .map((n) => n.memberId)
        .toSet();

    return plan.nodes.where((n) {
      if (n.status != NodeStatus.pending) return false;
      return n.dependencies.every((d) => completedIds.contains(d));
    }).toList();
  }

  /// 开始执行节点
  void startNode(String planId, String nodeId) {
    final plan = _plans[planId];
    if (plan == null) return;

    final nodeIdx = plan.nodes.indexWhere((n) => n.id == nodeId);
    if (nodeIdx == -1) return;

    final node = plan.nodes[nodeIdx];
    plan.nodes[nodeIdx] = DAGNode(
      id: node.id,
      memberId: node.memberId,
      dependencies: node.dependencies,
      status: NodeStatus.running,
      startedAt: DateTime.now(),
    );
  }

  /// 完成节点
  void completeNode(String planId, String nodeId, String result) {
    final plan = _plans[planId];
    if (plan == null) return;

    final nodeIdx = plan.nodes.indexWhere((n) => n.id == nodeId);
    if (nodeIdx == -1) return;

    final node = plan.nodes[nodeIdx];
    plan.nodes[nodeIdx] = DAGNode(
      id: node.id,
      memberId: node.memberId,
      dependencies: node.dependencies,
      status: NodeStatus.completed,
      startedAt: node.startedAt,
      completedAt: DateTime.now(),
      result: result,
    );
  }

  /// 检查计划是否完成
  bool isPlanComplete(ExecutionPlan plan) {
    return plan.nodes.every((n) =>
      n.status == NodeStatus.completed || n.status == NodeStatus.skipped
    );
  }

  /// 列出所有计划
  List<ExecutionPlan> listPlans() {
    return _plans.values.toList();
  }
}