import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:acp_ui_flutter/app.dart';
import 'package:acp_ui_flutter/data/services/server_config_service.dart';
import 'package:acp_ui_flutter/data/services/websocket_service.dart';

/// Mock WebSocket service that avoids real network connections in tests.
class _MockWebSocketService extends WebSocketService {
  _MockWebSocketService() : super(url: 'ws://0.0.0.0:1');

  @override
  Future<void> connect() async {
    throw Exception('Mock: no connection');
  }
}

void main() {
  late SharedPreferences prefs;

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    prefs = await SharedPreferences.getInstance();
  });

  List<Override> _overrides() => [
    sharedPreferencesProvider.overrideWithValue(prefs),
    webSocketServiceProvider.overrideWithValue(_MockWebSocketService()),
  ];

  testWidgets('App renders correctly', (WidgetTester tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: _overrides(),
        child: const AcpUiApp(),
      ),
    );

    await tester.pump();

    // Verify sidebar is present
    expect(find.text('ACP-UI'), findsOneWidget);

    // Default route now renders AgentTeamsDashboard
    expect(find.text('Agent Teams'), findsWidgets);
  });

  testWidgets('Sidebar navigation works', (WidgetTester tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: _overrides(),
        child: const AcpUiApp(),
      ),
    );

    await tester.pump();

    // Verify navigation items exist
    expect(find.text('Multi-Agent'), findsOneWidget);
    expect(find.text('History'), findsOneWidget);
    expect(find.text('Chat'), findsOneWidget);
    expect(find.text('Evolution'), findsOneWidget);
    expect(find.text('Hermes'), findsOneWidget);
    expect(find.text('Collaboration'), findsOneWidget);
    expect(find.text('Agent Teams'), findsWidgets);
    expect(find.text('Plugins'), findsOneWidget);

    // Scroll sidebar to reveal items below the fold
    await tester.drag(find.byType(ListView).first, const Offset(0, -200));
    await tester.pump();

    expect(find.text('Swarm'), findsOneWidget);
    expect(find.text('Server'), findsOneWidget);
    expect(find.text('Settings'), findsOneWidget);
  });
}
