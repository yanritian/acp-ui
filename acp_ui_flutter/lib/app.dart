import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'shared/theme/app_theme.dart';
import 'shared/widgets/sidebar.dart';
import 'features/chat/chat_view.dart';
import 'features/multi_agent/multi_agent_view.dart';
import 'features/history/history_view.dart';
import 'features/settings/settings_view.dart';
import 'features/evolution/evolution_dashboard.dart';
import 'features/hermes/hermes_dashboard.dart';
import 'features/collaboration/collaboration_network_view.dart';
import 'features/agent_teams/agent_teams_dashboard.dart';

// Router configuration with ShellRoute for persistent sidebar
final goRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    routes: [
      ShellRoute(
        builder: (context, state, child) => MainLayout(child: child),
        routes: [
          GoRoute(
            path: '/',
            builder: (context, state) => const ChatView(),
          ),
          GoRoute(
            path: '/multi-agent',
            builder: (context, state) => const MultiAgentView(),
          ),
          GoRoute(
            path: '/history',
            builder: (context, state) => const HistoryView(),
          ),
          GoRoute(
            path: '/settings',
            builder: (context, state) => const SettingsView(),
          ),
          GoRoute(
            path: '/evolution',
            builder: (context, state) => const EvolutionDashboard(),
          ),
          GoRoute(
            path: '/hermes',
            builder: (context, state) => const HermesDashboard(),
          ),
          GoRoute(
            path: '/collaboration',
            builder: (context, state) => const CollaborationNetworkView(),
          ),
          GoRoute(
            path: '/agent-teams',
            builder: (context, state) => const AgentTeamsDashboard(),
          ),
        ],
      ),
    ],
  );
});

class AcpUiApp extends ConsumerWidget {
  const AcpUiApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(goRouterProvider);

    return MaterialApp.router(
      title: 'ACP-UI',
      theme: AppTheme.light,
      darkTheme: AppTheme.dark,
      themeMode: ThemeMode.system,
      routerConfig: router,
      debugShowCheckedModeBanner: false,
    );
  }
}

// Main layout with sidebar
class MainLayout extends ConsumerWidget {
  const MainLayout({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      body: Row(
        children: [
          // Sidebar (fixed width)
          const Sidebar(),
          // Main content area
          Expanded(
            child: child,
          ),
        ],
      ),
    );
  }
}