import 'dart:async';
import 'package:uuid/uuid.dart';

import '../../data/models/session.dart';

/// Session Manager (Claw Code inspired)
class SessionManager {
  final Map<String, Session> _sessions = {};
  final Map<String, String> _branchLocks = {}; // branch -> session_id
  final int _maxMessages;
  final int _compactionThreshold;
  final StreamController<Session> _sessionController = StreamController<Session>.broadcast();
  final Uuid _uuid = const Uuid();

  SessionManager({
    int maxMessages = 100,
  })  : _maxMessages = maxMessages,
        _compactionThreshold = maxMessages ~/ 2;

  /// Create a new session for an agent
  Session createSession(String agentId, {String? branch}) {
    // Check for branch lock collision
    if (branch != null && _branchLocks.containsKey(branch)) {
      throw Exception('Branch $branch is locked by session ${_branchLocks[branch]}');
    }

    final session = Session(
      id: _uuid.v4(),
      agentId: agentId,
      createdAt: DateTime.now(),
      lastActive: DateTime.now(),
      messages: [],
      branchLock: branch,
      status: SessionStatus.active,
    );

    // Register branch lock
    if (branch != null) {
      _branchLocks[branch] = session.id;
    }

    _sessions[session.id] = session;
    _sessionController.add(session);
    return session;
  }

  /// Add a message to a session
  Session addMessage(String sessionId, SessionMessage message) {
    final session = _sessions[sessionId];
    if (session == null) {
      throw Exception('Session $sessionId not found');
    }

    final updatedSession = session.addMessage(message);

    // Compact if needed
    if (updatedSession.needsCompaction(_compactionThreshold)) {
      final compactedSession = _compactSession(updatedSession);
      _sessions[sessionId] = compactedSession;
      _sessionController.add(compactedSession);
      return compactedSession;
    }

    _sessions[sessionId] = updatedSession;
    _sessionController.add(updatedSession);
    return updatedSession;
  }

  /// Compact a session
  Session _compactSession(Session session) {
    final total = session.messages.length;
    final keepCount = _compactionThreshold ~/ 2;

    // Mark older messages as compressed
    final compressedMessages = session.messages.map((m) {
      final index = session.messages.indexOf(m);
      if (index < total - keepCount) {
        return SessionMessage(
          id: m.id,
          role: m.role,
          content: '[Compressed]',
          timestamp: m.timestamp,
          compressed: true,
        );
      }
      return m;
    }).toList();

    return Session(
      id: session.id,
      agentId: session.agentId,
      createdAt: session.createdAt,
      lastActive: DateTime.now(),
      messages: compressedMessages,
      branchLock: session.branchLock,
      status: SessionStatus.compacted,
      compactionCount: session.compactionCount + 1,
    );
  }

  /// End a session and release branch lock
  void endSession(String sessionId) {
    final session = _sessions[sessionId];
    if (session == null) return;

    // Release branch lock
    if (session.branchLock != null) {
      _branchLocks.remove(session.branchLock!);
    }

    _sessions[sessionId] = Session(
      id: session.id,
      agentId: session.agentId,
      createdAt: session.createdAt,
      lastActive: DateTime.now(),
      messages: session.messages,
      branchLock: session.branchLock,
      status: SessionStatus.completed,
    );
  }

  /// Get session by ID
  Session? getSession(String sessionId) => _sessions[sessionId];

  /// Get all sessions for an agent
  List<Session> getAgentSessions(String agentId) {
    return _sessions.values
        .where((s) => s.agentId == agentId && s.status == SessionStatus.active)
        .toList();
  }

  /// Check if branch has collision
  bool hasCollision(String branch) => _branchLocks.containsKey(branch);

  /// Get session stream
  Stream<Session> get sessionStream => _sessionController.stream;

  /// List all sessions
  List<Session> listSessions() => _sessions.values.toList();
}