import 'dart:async';
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:acp_ui_flutter/core/agent/agent_bridge.dart';
import 'package:acp_ui_flutter/core/transport/acp_transport.dart';
import 'package:acp_ui_flutter/data/models/agent.dart';

/// Mock Transport for testing
class MockTransport implements AcpTransport {
  bool _connected = false;
  final List<String> _sentMessages = [];
  final StreamController<String> _receiveController = StreamController<String>.broadcast();

  List<String> get sentMessages => _sentMessages;

  void simulateReceive(String data) {
    _receiveController.add(data);
  }

  @override
  Future<void> connect() async {
    _connected = true;
  }

  @override
  Future<void> disconnect() async {
    _connected = false;
    await _receiveController.close();
  }

  @override
  Future<void> send(String message) async {
    if (!_connected) throw Exception('Not connected');
    _sentMessages.add(message);
  }

  @override
  Stream<String> receive() => _receiveController.stream;

  @override
  bool isConnected() => _connected;

  @override
  String transportType() => 'Mock';
}

void main() {
  group('AgentBridge', () {
    test('should create bridge with agent and transport', () {
      final agent = Agent(
        id: 'test-agent',
        name: 'Test',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );
      final transport = MockTransport();
      final bridge = AgentBridge(agent: agent, transport: transport);

      expect(bridge.agent.id, 'test-agent');
      expect(bridge.isConnected(), false);
    });

    test('should connect through transport', () async {
      final agent = Agent(
        id: 'test-agent',
        name: 'Test',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );
      final transport = MockTransport();
      final bridge = AgentBridge(agent: agent, transport: transport);

      await bridge.connect();
      expect(bridge.isConnected(), true);
    });

    test('should disconnect and close message stream', () async {
      final agent = Agent(
        id: 'test-agent',
        name: 'Test',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );
      final transport = MockTransport();
      final bridge = AgentBridge(agent: agent, transport: transport);

      await bridge.connect();
      await bridge.disconnect();
      expect(bridge.isConnected(), false);
    });

    test('should send message through transport', () async {
      final agent = Agent(
        id: 'test-agent',
        name: 'Test',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );
      final transport = MockTransport();
      final bridge = AgentBridge(agent: agent, transport: transport);

      await bridge.connect();
      await bridge.sendMessage('Hello Agent');

      expect(transport.sentMessages.length, 1);
      final sentData = transport.sentMessages.first;
      expect(sentData.contains('Hello Agent'), true);
      expect(sentData.contains('test-agent'), true);
    });

    test('should handle incoming message data', () async {
      final agent = Agent(
        id: 'test-agent',
        name: 'Test',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );
      final transport = MockTransport();
      final bridge = AgentBridge(agent: agent, transport: transport);

      await bridge.connect();

      final messageFuture = bridge.receiveMessages().first;

      // Simulate receiving a message
      bridge.handleIncomingData(jsonEncode({
        'type': 'message',
        'message': {
          'id': 'msg-1',
          'role': 'assistant',
          'content': 'Hello User',
          'timestamp': DateTime.now().toIso8601String(),
        },
      }));

      final message = await messageFuture;
      expect(message.content, 'Hello User');
      expect(message.role, MessageRole.assistant);
    });

    test('should handle tool_use event', () async {
      final agent = Agent(
        id: 'test-agent',
        name: 'Test',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );
      final transport = MockTransport();
      final bridge = AgentBridge(agent: agent, transport: transport);

      await bridge.connect();

      final messageFuture = bridge.receiveMessages().first;

      bridge.handleIncomingData(jsonEncode({
        'type': 'tool_use',
        'id': 'event-1',
        'agent_id': 'test-agent',
      }));

      final message = await messageFuture;
      expect(message.role, MessageRole.tool);
    });

    test('should handle parse error', () async {
      final agent = Agent(
        id: 'test-agent',
        name: 'Test',
        type: AgentType.planner,
        createdAt: DateTime.now(),
      );
      final transport = MockTransport();
      final bridge = AgentBridge(agent: agent, transport: transport);

      await bridge.connect();

      final errorFuture = bridge.receiveMessages().first;

      // Send invalid JSON
      bridge.handleIncomingData('invalid json');

      try {
        await errorFuture;
      } catch (e) {
        expect(e, isA<Exception>());
        expect(e.toString().contains('Failed to parse'), true);
      }
    });
  });

  group('AgentMessage', () {
    test('should serialize to JSON', () {
      final message = AgentMessage(
        id: 'msg-1',
        role: MessageRole.user,
        content: 'Test content',
        timestamp: DateTime(2024, 1, 1),
      );

      final json = message.toJson();
      expect(json['id'], 'msg-1');
      expect(json['role'], 'user');
      expect(json['content'], 'Test content');
    });

    test('should deserialize from JSON', () {
      final json = {
        'id': 'msg-1',
        'role': 'assistant',
        'content': 'Response',
        'timestamp': '2024-01-01T00:00:00',
      };

      final message = AgentMessage.fromJson(json);
      expect(message.id, 'msg-1');
      expect(message.role, MessageRole.assistant);
      expect(message.content, 'Response');
    });
  });

  group('AgentEvent', () {
    test('should parse from JSON', () {
      final json = {
        'id': 'event-1',
        'type': 'tool_use',
        'agent_id': 'agent-1',
      };

      final event = AgentEvent.fromJson(json);
      expect(event.id, 'event-1');
      expect(event.type, 'tool_use');
      expect(event.agentId, 'agent-1');
    });
  });
}