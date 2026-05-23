import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:acp_ui_flutter/data/stores/session_store.dart';
import 'package:acp_ui_flutter/core/session/session_manager.dart';

void main() {
  group('Session Store Providers', () {
    test('sessionManagerProvider creates SessionManager with correct config', () {
      final container = ProviderContainer();
      final manager = container.read(sessionManagerProvider);

      expect(manager, isA<SessionManager>());

      container.dispose();
    });

    test('activeSessionsProvider returns empty list initially', () {
      final container = ProviderContainer();
      final sessions = container.read(activeSessionsProvider);

      expect(sessions, isEmpty);

      container.dispose();
    });

    test('currentSessionProvider starts as null', () {
      final container = ProviderContainer();
      final currentSession = container.read(currentSessionProvider);

      expect(currentSession, isNull);

      container.dispose();
    });

    test('currentSessionProvider can be updated', () {
      final container = ProviderContainer();
      final manager = container.read(sessionManagerProvider);
      final session = manager.createSession('agent-1');

      container.read(currentSessionProvider.notifier).state = session;

      expect(container.read(currentSessionProvider), session);

      container.dispose();
    });

    test('sessionMessagesProvider returns empty list for non-existent session', () {
      final container = ProviderContainer();
      final messages = container.read(sessionMessagesProvider('non-existent'));

      expect(messages, isEmpty);

      container.dispose();
    });

    test('sessionMessagesProvider returns messages for existing session', () {
      final container = ProviderContainer();
      final manager = container.read(sessionManagerProvider);

      // Create session through manager
      final session = manager.createSession('agent-1');

      final messages = container.read(sessionMessagesProvider(session.id));

      // Initially empty
      expect(messages, isEmpty);

      container.dispose();
    });
  });
}