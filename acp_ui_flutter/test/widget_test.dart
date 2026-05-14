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
    expect(find.text('Chat'), findsOneWidget);
  });
}