import 'dart:async';
import 'dart:convert';
import 'dart:io' if (dart.library.html) 'dart:html';

import '../../data/models/agent.dart';
import '../transport/websocket_transport.dart';
import 'agent_bridge.dart';

/// Agent Pool - manages multiple agent connections
class AgentPool {
  final Map<String, AgentBridge> _bridges = {};
  final String baseUrl;
  final int _maxAgents;

  AgentPool({
    required this.baseUrl,
    int maxAgents = 10,
  }) : _maxAgents = maxAgents;

  /// Create and register a new agent
  Future<AgentBridge> createAgent(Agent agent) async {
    if (_bridges.length >= _maxAgents) {
      throw Exception('Maximum agent limit reached: $_maxAgents');
    }

    if (_bridges.containsKey(agent.id)) {
      return _bridges[agent.id]!;
    }

    // Create transport based on agent configuration
    final transport = WebSocketTransport(
      url: '$baseUrl/agent/${agent.id}',
    );

    final bridge = AgentBridge(
      agent: agent,
      transport: transport,
    );

    await bridge.connect();
    bridge.startListening();

    _bridges[agent.id] = bridge;
    return bridge;
  }

  /// Get existing agent bridge
  AgentBridge? getAgent(String agentId) {
    return _bridges[agentId];
  }

  /// Remove agent from pool
  Future<void> removeAgent(String agentId) async {
    final bridge = _bridges[agentId];
    if (bridge != null) {
      await bridge.disconnect();
      _bridges.remove(agentId);
    }
  }

  /// Get all active agents
  List<AgentBridge> getActiveAgents() {
    return _bridges.values.where((b) => b.isConnected()).toList();
  }

  /// Get pool statistics
  Map<String, dynamic> getStats() {
    return {
      'total_agents': _bridges.length,
      'active_agents': getActiveAgents().length,
      'max_agents': _maxAgents,
    };
  }

  /// Shutdown all agents
  Future<void> shutdown() async {
    for (final bridge in _bridges.values) {
      await bridge.disconnect();
    }
    _bridges.clear();
  }
}

/// Process Manager - manages local agent processes (desktop only)
class ProcessManager {
  final Map<String, Process> _processes = {};

  /// Start a local agent process
  Future<Process> startProcess(String agentId, String command, List<String> args) async {
    // Only available on desktop platforms
    // Mobile platforms use remote agents only
    try {
      final process = await Process.start(command, args);
      _processes[agentId] = process;
      return process;
    } catch (e) {
      throw Exception('Failed to start process: $e');
    }
  }

  /// Stop a process
  Future<void> stopProcess(String agentId) async {
    final process = _processes[agentId];
    if (process != null) {
      process.kill();
      _processes.remove(agentId);
    }
  }

  /// Get process output stream
  Stream<String>? getOutput(String agentId) {
    final process = _processes[agentId];
    if (process != null) {
      return process.stdout.transform(utf8.decoder);
    }
    return null;
  }

  /// Get process error stream
  Stream<String>? getError(String agentId) {
    final process = _processes[agentId];
    if (process != null) {
      return process.stderr.transform(utf8.decoder);
    }
    return null;
  }

  /// Check if process is running
  bool isRunning(String agentId) {
    return _processes.containsKey(agentId);
  }
}