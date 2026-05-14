import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:acp_ui_flutter/app.dart';

void main() {
  testWidgets('App renders correctly', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: AcpUiApp(),
      ),
    );

    // Verify sidebar is present
    expect(find.text('ACP-UI'), findsOneWidget);

    // Chat appears in sidebar and header (at least one)
    expect(find.text('Chat'), findsWidgets);
  });

  testWidgets('Sidebar navigation works', (WidgetTester tester) async {
    await tester.pumpWidget(
      const ProviderScope(
        child: AcpUiApp(),
      ),
    );

    // Verify navigation items exist
    expect(find.text('Multi-Agent'), findsOneWidget);
    expect(find.text('History'), findsOneWidget);
    expect(find.text('Settings'), findsOneWidget);
    expect(find.text('Evolution'), findsOneWidget);
    expect(find.text('Hermes'), findsOneWidget);
  });
}