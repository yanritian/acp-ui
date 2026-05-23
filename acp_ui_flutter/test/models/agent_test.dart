import 'package:flutter_test/flutter_test.dart';
import 'package:acp_ui_flutter/data/models/agent.dart';

void main() {
  group('Agent Model', () {
    test('should create Agent with default values', () {
      final agent = Agent(
        id: 'test-1',
        name: 'Test Agent',
        type: AgentType.planner,
        createdAt: DateTime(2024, 1, 1),
      );

      expect(agent.id, 'test-1');
      expect(agent.name, 'Test Agent');
      expect(agent.type, AgentType.planner);
      expect(agent.status, AgentStatus.idle);
      expect(agent.capabilities, isEmpty);
      expect(agent.maxContextTokens, 200000);
      expect(agent.usedContextTokens, 0);
    });

    test('should calculate context usage percentage correctly', () {
      final agent = Agent(
        id: 'test-1',
        name: 'Test Agent',
        type: AgentType.planner,
        maxContextTokens: 1000,
        usedContextTokens: 500,
        createdAt: DateTime(2024, 1, 1),
      );

      expect(agent.contextUsagePercent(), 50.0);
    });

    test('should detect when context is near limit', () {
      final agentNearLimit = Agent(
        id: 'test-1',
        name: 'Test Agent',
        type: AgentType.planner,
        maxContextTokens: 1000,
        usedContextTokens: 800,
        createdAt: DateTime(2024, 1, 1),
      );

      final agentBelowLimit = Agent(
        id: 'test-2',
        name: 'Test Agent',
        type: AgentType.planner,
        maxContextTokens: 1000,
        usedContextTokens: 799,
        createdAt: DateTime(2024, 1, 1),
      );

      expect(agentNearLimit.isContextNearLimit(), true);
      expect(agentBelowLimit.isContextNearLimit(), false);
    });

    test('should handle zero max context tokens', () {
      final agent = Agent(
        id: 'test-1',
        name: 'Test Agent',
        type: AgentType.planner,
        maxContextTokens: 0,
        usedContextTokens: 0,
        createdAt: DateTime(2024, 1, 1),
      );

      expect(agent.contextUsagePercent(), 0);
      expect(agent.isContextNearLimit(), false);
    });

    test('should copy with new values', () {
      final agent = Agent(
        id: 'test-1',
        name: 'Test Agent',
        type: AgentType.planner,
        createdAt: DateTime(2024, 1, 1),
      );

      final updatedAgent = agent.copyWith(
        status: AgentStatus.busy,
        usedContextTokens: 500,
      );

      expect(updatedAgent.status, AgentStatus.busy);
      expect(updatedAgent.usedContextTokens, 500);
      expect(updatedAgent.id, agent.id); // unchanged
    });

    test('should serialize to JSON correctly', () {
      final agent = Agent(
        id: 'test-1',
        name: 'Test Agent',
        type: AgentType.planner,
        status: AgentStatus.busy,
        capabilities: ['planning', 'tasks'],
        createdAt: DateTime(2024, 1, 1),
      );

      final json = agent.toJson();

      expect(json['id'], 'test-1');
      expect(json['name'], 'Test Agent');
      expect(json['type'], 'planner');
      expect(json['status'], 'busy');
      expect(json['capabilities'], ['planning', 'tasks']);
    });

    test('should deserialize from JSON correctly', () {
      final json = {
        'id': 'test-1',
        'name': 'Test Agent',
        'type': 'architect',
        'status': 'idle',
        'capabilities': ['design'],
        'cwd': null,
        'maxContextTokens': 150000,
        'usedContextTokens': 1000,
        'createdAt': '2024-01-01T00:00:00.000',
        'lastActiveAt': null,
      };

      final agent = Agent.fromJson(json);

      expect(agent.id, 'test-1');
      expect(agent.type, AgentType.architect);
      expect(agent.maxContextTokens, 150000);
    });

    test('all AgentType values should serialize correctly', () {
      for (final type in AgentType.values) {
        final agent = Agent(
          id: 'test',
          name: 'Test',
          type: type,
          createdAt: DateTime.now(),
        );
        final json = agent.toJson();
        final restored = Agent.fromJson(json);
        expect(restored.type, type);
      }
    });

    test('all AgentStatus values should serialize correctly', () {
      for (final status in AgentStatus.values) {
        final agent = Agent(
          id: 'test',
          name: 'Test',
          type: AgentType.planner,
          status: status,
          createdAt: DateTime.now(),
        );
        final json = agent.toJson();
        final restored = Agent.fromJson(json);
        expect(restored.status, status);
      }
    });
  });
}