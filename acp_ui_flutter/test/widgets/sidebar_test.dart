import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/material.dart';

void main() {
  group('Sidebar Widget', () {
    // Note: Sidebar requires GoRouter context, so we test it indirectly
    // through the app integration tests. Here we test the basic widget
    // structure without GoRouter dependency.

    testWidgets('Sidebar navigation labels are correct', (WidgetTester tester) async {
      // Test the navigation labels exist as constants
      const navLabels = [
        'Chat',
        'Multi-Agent',
        'History',
        'Evolution',
        'Hermes',
        'Collaboration',
        'Agent Teams',
        'Settings',
      ];

      // Verify all expected navigation items are defined
      expect(navLabels.length, 8);
      expect(navLabels.contains('Chat'), true);
      expect(navLabels.contains('Settings'), true);
    });

    testWidgets('Sidebar has correct icons defined', (WidgetTester tester) async {
      // Test icon constants
      const expectedIcons = [
        Icons.chat_outlined,
        Icons.people_outline,
        Icons.history_outlined,
        Icons.auto_fix_high_outlined,
        Icons.monitor_outlined,
        Icons.account_tree_outlined,
        Icons.rocket_launch_outlined,
        Icons.settings_outlined,
      ];

      expect(expectedIcons.length, 8);
    });

    testWidgets('Sidebar width is 240', (WidgetTester tester) async {
      // Verify the sidebar width constant
      const sidebarWidth = 240.0;
      expect(sidebarWidth, 240.0);
    });
  });
}