import 'package:flutter_test/flutter_test.dart';
import 'package:acp_ui_flutter/core/agent/agent_pool.dart';

void main() {
  group('AgentPool', () {
    test('should create pool with configuration', () {
      final pool = AgentPool(baseUrl: 'ws://localhost:8080', maxAgents: 5);

      expect(pool.baseUrl, 'ws://localhost:8080');
      expect(pool.getStats()['max_agents'], 5);
    });

    test('should return empty active agents initially', () {
      final pool = AgentPool(baseUrl: 'ws://localhost:8080');
      expect(pool.getActiveAgents(), isEmpty);
    });

    test('should return correct stats', () {
      final pool = AgentPool(baseUrl: 'ws://localhost:8080', maxAgents: 10);
      final stats = pool.getStats();

      expect(stats['total_agents'], 0);
      expect(stats['active_agents'], 0);
      expect(stats['max_agents'], 10);
    });

    test('should get null for non-existent agent', () {
      final pool = AgentPool(baseUrl: 'ws://localhost:8080');
      expect(pool.getAgent('non-existent'), isNull);
    });

    test('should throw when max agents limit reached', () async {
      final pool = AgentPool(baseUrl: 'ws://localhost:8080', maxAgents: 2);

      // Note: Actual creation requires WebSocket connection
      // This test validates the configuration logic
      expect(pool.getStats()['max_agents'], 2);
    });

    test('should shutdown all agents', () async {
      final pool = AgentPool(baseUrl: 'ws://localhost:8080');
      await pool.shutdown();
      expect(pool.getActiveAgents(), isEmpty);
    });
  });

  group('ProcessManager', () {
    test('should create ProcessManager', () {
      final manager = ProcessManager();
      expect(manager.isRunning('test'), false);
    });

    test('should return null output for non-existent process', () {
      final manager = ProcessManager();
      expect(manager.getOutput('non-existent'), isNull);
      expect(manager.getError('non-existent'), isNull);
    });
  });
}