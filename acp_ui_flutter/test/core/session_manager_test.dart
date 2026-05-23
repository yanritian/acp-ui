import 'package:flutter_test/flutter_test.dart';
import 'package:acp_ui_flutter/core/session/session_manager.dart';
import 'package:acp_ui_flutter/data/models/session.dart';

void main() {
  test('SessionManager - create session', () {
    final manager = SessionManager(maxMessages: 100);
    final session = manager.createSession('agent-1');

    expect(session.id, isNotEmpty);
    expect(session.agentId, 'agent-1');
    expect(session.status, SessionStatus.active);
    expect(session.messages, isEmpty);
  });

  test('SessionManager - create session with branch lock', () {
    final manager = SessionManager(maxMessages: 100);
    final session = manager.createSession('agent-1', branch: 'feature-branch');

    expect(session.branchLock, 'feature-branch');
    expect(manager.hasCollision('feature-branch'), true);
  });

  test('SessionManager - branch lock collision', () {
    final manager = SessionManager(maxMessages: 100);
    manager.createSession('agent-1', branch: 'feature-branch');

    expect(
      () => manager.createSession('agent-2', branch: 'feature-branch'),
      throwsException,
    );
  });

  test('SessionManager - add message', () {
    final manager = SessionManager(maxMessages: 100);
    final session = manager.createSession('agent-1');
    final message = SessionMessage(
      id: 'msg-1',
      role: MessageRole.user,
      content: 'Hello',
      timestamp: DateTime.now(),
    );

    final updated = manager.addMessage(session.id, message);
    expect(updated.messages.length, 1);
    expect(updated.messages.first.content, 'Hello');
  });

  test('SessionManager - add message to non-existent session fails', () {
    final manager = SessionManager(maxMessages: 100);
    final message = SessionMessage(
      id: 'msg-1',
      role: MessageRole.user,
      content: 'Hello',
      timestamp: DateTime.now(),
    );

    expect(() => manager.addMessage('non-existent', message), throwsException);
  });

  test('SessionManager - get session by ID', () {
    final manager = SessionManager(maxMessages: 100);
    final session = manager.createSession('agent-1');

    final retrieved = manager.getSession(session.id);
    expect(retrieved, isNotNull);
    expect(retrieved!.id, session.id);
  });

  test('SessionManager - get non-existent session returns null', () {
    final manager = SessionManager(maxMessages: 100);
    expect(manager.getSession('non-existent'), isNull);
  });

  test('SessionManager - get agent sessions', () {
    final manager = SessionManager(maxMessages: 100);
    manager.createSession('agent-x');
    manager.createSession('agent-x');
    manager.createSession('agent-y');

    final agentXSessions = manager.getAgentSessions('agent-x');
    expect(agentXSessions.length, 2);
  });

  test('SessionManager - list all sessions', () {
    final manager = SessionManager(maxMessages: 100);
    manager.createSession('agent-1');
    manager.createSession('agent-2');

    expect(manager.listSessions().length, 2);
  });

  test('SessionManager - end session', () {
    final manager = SessionManager(maxMessages: 100);
    final session = manager.createSession('agent-1', branch: 'feature-branch');
    expect(manager.hasCollision('feature-branch'), true);

    manager.endSession(session.id);

    final ended = manager.getSession(session.id);
    expect(ended!.status, SessionStatus.completed);
    expect(manager.hasCollision('feature-branch'), false);
  });

  test('SessionManager - ended sessions not in agent list', () {
    final manager = SessionManager(maxMessages: 100);
    final s1 = manager.createSession('agent-1');
    final s2 = manager.createSession('agent-1');

    manager.endSession(s2.id);

    final active = manager.getAgentSessions('agent-1');
    expect(active.length, 1);
    expect(active.first.id, s1.id);
  });

  test('SessionManager - compaction', () {
    final manager = SessionManager(maxMessages: 10);
    final session = manager.createSession('agent-1');

    for (int i = 0; i < 8; i++) {
      manager.addMessage(
        session.id,
        SessionMessage(
          id: 'msg-$i',
          role: MessageRole.user,
          content: 'Message $i',
          timestamp: DateTime.now(),
        ),
      );
    }

    final updated = manager.getSession(session.id);
    expect(updated!.status, SessionStatus.compacted);
  });

  test('SessionManager - end non-existent session is safe', () {
    final manager = SessionManager(maxMessages: 100);
    manager.endSession('non-existent'); // Should not throw
  });
}