import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/session/session_manager.dart';
import '../models/session.dart';

/// Session Manager Provider
final sessionManagerProvider = Provider<SessionManager>((ref) {
  return SessionManager(maxMessages: 100);
});

/// Active Sessions Provider
final activeSessionsProvider = Provider<List<Session>>((ref) {
  final manager = ref.watch(sessionManagerProvider);
  return manager.listSessions().where((s) => s.status == SessionStatus.active).toList();
});

/// Current Session Provider
final currentSessionProvider = StateProvider<Session?>((ref) {
  return null;
});

/// Session Messages Provider
final sessionMessagesProvider = Provider.family<List<SessionMessage>, String>((ref, sessionId) {
  final session = ref.watch(sessionManagerProvider).getSession(sessionId);
  return session?.messages ?? [];
});

/// Session Stream Provider
final sessionStreamProvider = StreamProvider<Session>((ref) {
  final manager = ref.watch(sessionManagerProvider);
  return manager.sessionStream;
});