/// Circuit Breaker - 三态熔断器
///
/// Closed → Open → HalfOpen 状态转换

import 'package:flutter/foundation.dart';

/// 熔断器状态
enum CircuitState {
  closed,
  open,
  halfOpen,
}

/// 熔断器记录
class CircuitBreakerRecord {
  final String id;
  final String target;
  final CircuitState state;
  final int failureCount;
  final int successCount;
  final DateTime? lastFailureAt;
  final DateTime? coolDownUntil;
  final DateTime createdAt;
  final DateTime updatedAt;

  CircuitBreakerRecord({
    required this.id,
    required this.target,
    required this.state,
    this.failureCount = 0,
    this.successCount = 0,
    this.lastFailureAt,
    this.coolDownUntil,
    required this.createdAt,
    required this.updatedAt,
  });

  factory CircuitBreakerRecord.fromJson(Map<String, dynamic> json) {
    return CircuitBreakerRecord(
      id: json['id'] as String,
      target: json['target'] as String,
      state: _parseState(json['state'] as String),
      failureCount: json['failure_count'] as int? ?? 0,
      successCount: json['success_count'] as int? ?? 0,
      lastFailureAt: json['last_failure_at'] != null
          ? DateTime.parse(json['last_failure_at'] as String)
          : null,
      coolDownUntil: json['cool_down_until'] != null
          ? DateTime.parse(json['cool_down_until'] as String)
          : null,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  static CircuitState _parseState(String s) {
    switch (s) {
      case 'closed':
        return CircuitState.closed;
      case 'open':
        return CircuitState.open;
      case 'half_open':
      case 'halfopen':
        return CircuitState.halfOpen;
      default:
        return CircuitState.closed;
    }
  }

  bool get isClosed => state == CircuitState.closed;
  bool get isOpen => state == CircuitState.open;
  bool get isHalfOpen => state == CircuitState.halfOpen;
}

/// 熔断器管理器
class CircuitBreakerManager {
  final Map<String, CircuitBreakerRecord> _breakers = {};

  /// 检查请求是否允许
  ///
  /// 通过 WebSocket 调用 Tauri 命令 is_circuit_breaker_allowed
  Future<bool> isAllowed(String target) async {
    // TODO: WebSocket 调用
    // final response = await _wsService.sendCommand({
    //   'command': 'is_circuit_breaker_allowed',
    //   'args': {'target': target},
    // });
    // return response['allowed'] as bool;

    // 本地检查（备用）
    final breaker = _breakers[target];
    if (breaker == null) return true;

    switch (breaker.state) {
      case CircuitState.closed:
        return true;
      case CircuitState.open:
        if (breaker.coolDownUntil != null &&
            DateTime.now().isAfter(breaker.coolDownUntil!)) {
          return true; // 可以尝试恢复
        }
        return false;
      case CircuitState.halfOpen:
        return breaker.successCount < 5; // 限制半开状态调用次数
    }
  }

  /// 获取熔断器状态
  Future<CircuitBreakerRecord> getStatus(String target) async {
    // TODO: WebSocket 调用
    // final response = await _wsService.sendCommand({
    //   'command': 'get_circuit_breaker_status',
    //   'args': {'target': target},
    // });
    // return CircuitBreakerRecord.fromJson(response);

    return _breakers[target] ?? CircuitBreakerRecord(
      id: 'default',
      target: target,
      state: CircuitState.closed,
      createdAt: DateTime.now(),
      updatedAt: DateTime.now(),
    );
  }

  /// 获取所有熔断器状态
  Future<List<CircuitBreakerRecord>> getAllStatuses() async {
    // TODO: WebSocket 调用
    // final response = await _wsService.sendCommand({
    //   'command': 'get_all_circuit_breakers',
    // });
    // return (response as List).map((e) => CircuitBreakerRecord.fromJson(e)).toList();

    return _breakers.values.toList();
  }

  /// 重置熔断器
  Future<void> reset(String target) async {
    // TODO: WebSocket 调用
    // await _wsService.sendCommand({
    //   'command': 'reset_circuit_breaker',
    //   'args': {'target': target},
    // });

    _breakers.remove(target);
  }

  /// 记录成功（本地备用）
  void recordSuccess(String target) {
    final breaker = _breakers[target];
    if (breaker != null && breaker.state == CircuitState.halfOpen) {
      _breakers[target] = CircuitBreakerRecord(
        id: breaker.id,
        target: target,
        state: breaker.successCount >= 2 ? CircuitState.closed : CircuitState.halfOpen,
        successCount: breaker.successCount + 1,
        createdAt: breaker.createdAt,
        updatedAt: DateTime.now(),
      );
    }
  }

  /// 记录失败（本地备用）
  void recordFailure(String target) {
    final breaker = _breakers[target] ?? CircuitBreakerRecord(
      id: 'new',
      target: target,
      state: CircuitState.closed,
      createdAt: DateTime.now(),
      updatedAt: DateTime.now(),
    );

    if (breaker.state == CircuitState.closed && breaker.failureCount >= 4) {
      // 转换到 Open 状态
      _breakers[target] = CircuitBreakerRecord(
        id: breaker.id,
        target: target,
        state: CircuitState.open,
        failureCount: breaker.failureCount + 1,
        lastFailureAt: DateTime.now(),
        coolDownUntil: DateTime.now().add(Duration(seconds: 30)),
        createdAt: breaker.createdAt,
        updatedAt: DateTime.now(),
      );
    } else if (breaker.state == CircuitState.halfOpen) {
      // 半开状态失败直接回到 Open
      _breakers[target] = CircuitBreakerRecord(
        id: breaker.id,
        target: target,
        state: CircuitState.open,
        failureCount: breaker.failureCount + 1,
        lastFailureAt: DateTime.now(),
        coolDownUntil: DateTime.now().add(Duration(seconds: 30)),
        createdAt: breaker.createdAt,
        updatedAt: DateTime.now(),
      );
    } else {
      _breakers[target] = CircuitBreakerRecord(
        id: breaker.id,
        target: target,
        state: breaker.state,
        failureCount: breaker.failureCount + 1,
        lastFailureAt: DateTime.now(),
        createdAt: breaker.createdAt,
        updatedAt: DateTime.now(),
      );
    }
  }
}