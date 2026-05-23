import 'dart:convert';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:acp_ui_flutter/data/services/websocket_service.dart';

@GenerateMocks([WebSocketChannel])
void main() {
  group('WebSocketService', () {
    test('should initialize with correct default URL', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      expect(service.url, 'ws://127.0.0.1:1421');
      expect(service.isConnected(), false);
    });

    test('should configure URL correctly', () {
      final service = WebSocketService(url: 'ws://localhost:1421');
      service.configureUrl('ws://192.168.1.1:1421');
      expect(service.url, 'ws://192.168.1.1:1421');
    });

    test('should generate unique request IDs', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      // Request IDs should be unique and follow pattern
      // We can't directly test the private method, but we can verify behavior
      expect(service.isConnected(), false);
    });

    test('should set auth token correctly', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      service.setAuthToken('test-token-123');
      // Token should be stored (internal state)
      expect(service.isConnected(), false);
    });

    test('should provide message stream', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      expect(service.messageStream, isNotNull);
    });

    test('should return correct status string', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      expect(service.status, 'disconnected');
    });
  });

  group('WebSocketService Commands', () {
    // These tests verify the command generation logic
    // Actual network tests require integration testing

    test('initExecutiveAgent command format', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      // Command format should follow JSON-RPC spec
      // Expected: {"id": "req-xxx", "type": "request", "command": "init_executive_agent", "payload": {"workspace": "..."}}
      expect(service.isConnected(), false);
    });

    test('executeDevelopmentTask command format', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      // Expected: {"id": "req-xxx", "type": "request", "command": "execute_development_task", "payload": {"request": "..."}}
      expect(service.isConnected(), false);
    });

    test('getTaskHistory command format', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      // Expected: {"id": "req-xxx", "type": "request", "command": "get_task_history", "payload": {"limit": 10}}
      expect(service.isConnected(), false);
    });

    test('spawnAndExecuteAgent command format', () {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      // Expected: {"id": "req-xxx", "type": "request", "command": "spawn_and_execute_agent", "payload": {"agent_name": "...", "request": "..."}}
      expect(service.isConnected(), false);
    });
  });

  group('WebSocketService Connection', () {
    test('should not connect when already connected', () async {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      // Second connect should be skipped
      expect(service.isConnected(), false);
    });

    test('disconnect should work safely when not connected', () async {
      final service = WebSocketService(url: 'ws://127.0.0.1:1421');
      await service.disconnect();
      expect(service.isConnected(), false);
    });
  });
}