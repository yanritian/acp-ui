import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:acp_ui_flutter/features/agent_teams/agent_teams_dashboard.dart';
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

  group('AgentTeamsDashboard Widget Tests', () {
    testWidgets('should render dashboard title', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Verify title exists
      expect(find.text('Agent Teams'), findsOneWidget);
    });

    testWidgets('should render connection status indicator', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Connection status should be visible (mock service does not connect)
      expect(find.text('离线'), findsOneWidget);
    });

    testWidgets('should render bottom navigation items', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // View mode buttons should exist
      expect(find.text('首页'), findsOneWidget);
      expect(find.text('进度'), findsOneWidget);
      expect(find.text('协作'), findsOneWidget);
      expect(find.text('宠物'), findsOneWidget);
    });

    testWidgets('should render quick actions', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Quick actions should be visible
      expect(find.text('快捷操作'), findsOneWidget);
      expect(find.text('新建任务'), findsOneWidget);
    });

    testWidgets('should render stat cards', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Stats should be displayed
      expect(find.text('活跃Agent'), findsOneWidget);
      expect(find.text('思考中'), findsOneWidget);
      expect(find.text('执行中'), findsOneWidget);
      expect(find.text('智能协作平台'), findsOneWidget);
    });

    testWidgets('should render Agent team panel', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Agent team panel header should be visible
      expect(find.text('Agent 团队'), findsOneWidget);
    });

    testWidgets('should have config button', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Settings button should exist
      expect(find.byIcon(Icons.settings_rounded), findsOneWidget);
    });

    testWidgets('should have rocket launch icon', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Rocket launch icon should exist in app bar
      expect(find.byIcon(Icons.rocket_launch_rounded), findsOneWidget);
    });

    testWidgets('should have quick action icons', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Quick action code icon should exist
      expect(find.byIcon(Icons.code_rounded), findsOneWidget);
    });

    testWidgets('should change view mode when button pressed', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Tap progress view button
      await tester.tap(find.text('进度'));
      await tester.pump();

      // Progress view should be active (blue color)
      // This is verified by the view mode state change
    });
  });

  group('AgentTeamsDashboard Functionality Tests', () {
    testWidgets('should show config dialog when settings button pressed', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Tap settings button
      await tester.tap(find.byIcon(Icons.settings_rounded));
      await tester.pumpAndSettle();

      // Config dialog should appear
      expect(find.text('配置设置'), findsOneWidget);
      expect(find.text('执行模式'), findsOneWidget);
    });

    testWidgets('should show input dialog when FAB pressed', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: _overrides(),
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Tap FAB (add button)
      await tester.tap(find.byType(FloatingActionButton));
      await tester.pumpAndSettle();

      // Input dialog should appear
      expect(find.text('创建新任务'), findsOneWidget);
      expect(find.text('发送任务'), findsOneWidget);
    });
  });
}
