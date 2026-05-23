import 'package:flutter_test/flutter_test.dart';
import 'package:acp_ui_flutter/data/models/session.dart';

void main() {
  group('SessionMessage Model', () {
    test('should create SessionMessage with required values', () {
      final message = SessionMessage(
        id: 'msg-1',
        role: MessageRole.user,
        content: 'Hello',
        timestamp: DateTime(2024, 1, 1),
      );

      expect(message.id, 'msg-1');
      expect(message.role, MessageRole.user);
      expect(message.content, 'Hello');
      expect(message.compressed, false);
    });

    test('should serialize to JSON correctly', () {
      final message = SessionMessage(
        id: 'msg-1',
        role: MessageRole.assistant,
        content: 'Response',
        timestamp: DateTime(2024, 1, 1),
        compressed: true,
      );

      final json = message.toJson();

      expect(json['id'], 'msg-1');
      expect(json['role'], 'assistant');
      expect(json['content'], 'Response');
      expect(json['compressed'], true);
    });

    test('should deserialize from JSON correctly', () {
      final json = {
        'id': 'msg-1',
        'role': 'system',
        'content': 'System message',
        'timestamp': '2024-01-01T00:00:00.000',
        'compressed': false,
      };

      final message = SessionMessage.fromJson(json);

      expect(message.id, 'msg-1');
      expect(message.role, MessageRole.system);
      expect(message.content, 'System message');
    });

    test('all MessageRole values should serialize correctly', () {
      for (final role in MessageRole.values) {
        final message = SessionMessage(
          id: 'msg',
          role: role,
          content: 'test',
          timestamp: DateTime.now(),
        );
        final json = message.toJson();
        final restored = SessionMessage.fromJson(json);
        expect(restored.role, role);
      }
    });
  });

  group('Session Model', () {
    test('should create Session with default values', () {
      final session = Session(
        id: 'session-1',
        agentId: 'agent-1',
        createdAt: DateTime(2024, 1, 1),
        lastActive: DateTime(2024, 1, 1),
      );

      expect(session.id, 'session-1');
      expect(session.agentId, 'agent-1');
      expect(session.messages, isEmpty);
      expect(session.status, SessionStatus.active);
      expect(session.branchLock, isNull);
      expect(session.compactionCount, 0);
    });

    test('should add message to session', () {
      final session = Session(
        id: 'session-1',
        agentId: 'agent-1',
        createdAt: DateTime(2024, 1, 1),
        lastActive: DateTime(2024, 1, 1),
      );

      final message = SessionMessage(
        id: 'msg-1',
        role: MessageRole.user,
        content: 'Hello',
        timestamp: DateTime(2024, 1, 1, 12, 0),
      );

      final updatedSession = session.addMessage(message);

      expect(updatedSession.messages.length, 1);
      expect(updatedSession.messages.first.content, 'Hello');
      expect(updatedSession.lastActive, message.timestamp);
      // Original session should be unchanged (immutability)
      expect(session.messages, isEmpty);
    });

    test('should detect when compaction is needed', () {
      final session = Session(
        id: 'session-1',
        agentId: 'agent-1',
        createdAt: DateTime(2024, 1, 1),
        lastActive: DateTime(2024, 1, 1),
        messages: List.generate(
          60,
          (i) => SessionMessage(
            id: 'msg-$i',
            role: MessageRole.user,
            content: 'Message $i',
            timestamp: DateTime.now(),
          ),
        ),
      );

      expect(session.needsCompaction(50), true);
      expect(session.needsCompaction(100), false);
    });

    test('should copy with new values', () {
      final session = Session(
        id: 'session-1',
        agentId: 'agent-1',
        createdAt: DateTime(2024, 1, 1),
        lastActive: DateTime(2024, 1, 1),
      );

      final updatedSession = session.copyWith(
        status: SessionStatus.completed,
        compactionCount: 2,
      );

      expect(updatedSession.status, SessionStatus.completed);
      expect(updatedSession.compactionCount, 2);
      expect(updatedSession.id, session.id); // unchanged
    });

    test('should serialize to JSON correctly', () {
      final session = Session(
        id: 'session-1',
        agentId: 'agent-1',
        createdAt: DateTime(2024, 1, 1),
        lastActive: DateTime(2024, 1, 1, 12, 0),
        status: SessionStatus.active,
        branchLock: 'feature-branch',
        compactionCount: 1,
      );

      final json = session.toJson();

      expect(json['id'], 'session-1');
      expect(json['agentId'], 'agent-1');
      expect(json['status'], 'active');
      expect(json['branchLock'], 'feature-branch');
      expect(json['compactionCount'], 1);
    });

    test('should deserialize from JSON correctly', () {
      final json = {
        'id': 'session-1',
        'agentId': 'agent-1',
        'createdAt': '2024-01-01T00:00:00.000',
        'lastActive': '2024-01-01T12:00:00.000',
        'messages': [],
        'branchLock': null,
        'status': 'paused',
        'compactionCount': 2,
      };

      final session = Session.fromJson(json);

      expect(session.id, 'session-1');
      expect(session.agentId, 'agent-1');
      expect(session.status, SessionStatus.paused);
      expect(session.compactionCount, 2);
    });

    test('all SessionStatus values should serialize correctly', () {
      for (final status in SessionStatus.values) {
        final session = Session(
          id: 'session',
          agentId: 'agent',
          createdAt: DateTime.now(),
          lastActive: DateTime.now(),
          status: status,
        );
        final json = session.toJson();
        final restored = Session.fromJson(json);
        expect(restored.status, status);
      }
    });
  });
}