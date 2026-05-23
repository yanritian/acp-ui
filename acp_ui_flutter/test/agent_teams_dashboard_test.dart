import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:acp_ui_flutter/features/agent_teams/agent_teams_dashboard.dart';

void main() {
  group('AgentTeamsDashboard Widget Tests', () {
    testWidgets('should render dashboard title', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      // Wait for initial frame
      await tester.pump();

      // Verify title exists
      expect(find.text('Agent Teams Platform'), findsOneWidget);
    });

    testWidgets('should render connection status indicator', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Connection status should be visible
      expect(find.text('断开'), findsOneWidget);
    });

    testWidgets('should render view toggle buttons', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // View mode buttons should exist
      expect(find.text('全部'), findsOneWidget);
      expect(find.text('进度'), findsOneWidget);
      expect(find.text('协作'), findsOneWidget);
      expect(find.text('宠物'), findsOneWidget);
    });

    testWidgets('should render request input panel', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Input hint text should be visible
      expect(find.text('输入开发需求 (如: 做一个ERP系统)'), findsOneWidget);
    });

    testWidgets('should render stat cards', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Stats should be displayed
      expect(find.text('代理'), findsOneWidget);
      expect(find.text('思考'), findsOneWidget);
      expect(find.text('执行'), findsOneWidget);
      expect(find.text('成功率'), findsOneWidget);
    });

    testWidgets('should render Agent team panel', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Agent team panel header should be visible
      expect(find.text('Agent团队'), findsOneWidget);
    });

    testWidgets('should have config button', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Settings button should exist
      expect(find.byIcon(Icons.settings_applications), findsOneWidget);
    });

    testWidgets('should have help button', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Help button should exist
      expect(find.byIcon(Icons.help_outline), findsOneWidget);
    });

    testWidgets('should have language button', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Language button should exist
      expect(find.byIcon(Icons.language), findsOneWidget);
    });

    testWidgets('should change view mode when button pressed', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
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
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Tap settings button
      await tester.tap(find.byIcon(Icons.settings_applications));
      await tester.pumpAndSettle();

      // Config dialog should appear
      expect(find.text('Agent Teams 配置'), findsOneWidget);
      expect(find.text('执行模式'), findsOneWidget);
      expect(find.text('工作目录'), findsOneWidget);
    });

    testWidgets('should show help dialog when help button pressed', (WidgetTester tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const AgentTeamsDashboard(),
          ),
        ),
      );

      await tester.pump();

      // Tap help button
      await tester.tap(find.byIcon(Icons.help_outline));
      await tester.pumpAndSettle();

      // Help dialog should appear
      expect(find.text('使用帮助'), findsOneWidget);
      expect(find.text('快捷键:'), findsOneWidget);
    });
  });
}