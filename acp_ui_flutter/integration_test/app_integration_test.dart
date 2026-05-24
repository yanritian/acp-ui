import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:acp_ui_flutter/app.dart';
import 'package:acp_ui_flutter/data/models/agent.dart';
import 'package:acp_ui_flutter/data/stores/agent_store.dart';

/// Helper to pump with timeout to handle infinite animations (like CircularProgressIndicator)
Future<void> pumpWithTimeout(WidgetTester tester, {Duration duration = const Duration(seconds: 5)}) async {
  await tester.pumpWidget(const ProviderScope(child: AcpUiApp()));
  // Pump for a fixed duration instead of waiting for settle
  // This handles cases where CircularProgressIndicator or other infinite animations exist
  await tester.pump(duration);
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  group('App Integration Tests', () {
    testWidgets('Full app navigation flow', (WidgetTester tester) async {
      await tester.pumpWidget(const ProviderScope(child: AcpUiApp()));

      // Pump for initial render (use pump instead of pumpAndSettle to handle infinite animations)
      await tester.pump(const Duration(seconds: 2));

      // Verify initial state (Chat view) - ACP-UI is in sidebar header
      expect(find.text('ACP-UI'), findsOneWidget);

      // Navigate to Multi-Agent
      await tester.tap(find.text('Multi-Agent'));
      await tester.pump(const Duration(seconds: 1));

      // Navigate to History
      await tester.tap(find.text('History'));
      await tester.pump(const Duration(seconds: 1));

      // Navigate to Settings
      await tester.tap(find.text('Settings'));
      await tester.pump(const Duration(seconds: 1));

      // Navigate back to Chat
      await tester.tap(find.text('Chat'));
      await tester.pump(const Duration(seconds: 1));
    });

    testWidgets('Agent status can be updated through provider', (WidgetTester tester) async {
      await tester.pumpWidget(const ProviderScope(child: AcpUiApp()));
      await tester.pump(const Duration(seconds: 1));

      // This tests the provider state management integration
      final container = ProviderContainer();
      final notifier = container.read(agentStatusProvider.notifier);

      // Update status
      notifier.setBusy('planner-1');
      expect(container.read(agentStatusProvider)['planner-1'], AgentStatus.busy);

      container.dispose();
    });

    testWidgets('Sidebar navigation items are interactive', (WidgetTester tester) async {
      await tester.pumpWidget(const ProviderScope(child: AcpUiApp()));
      await tester.pump(const Duration(seconds: 1));

      // Count navigation items
      final navItems = find.byType(ListTile);
      expect(navItems, findsWidgets);

      // Each item should be tappable
      for (final item in tester.widgetList<ListTile>(navItems)) {
        expect(item.enabled, true);
      }
    });
  });
}