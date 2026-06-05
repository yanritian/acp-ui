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
import 'features/plugins/plugin_manager_screen.dart';
import 'features/swarm/swarm_dashboard_screen.dart';
import 'features/server_config/server_config_screen.dart';

// Router configuration with ShellRoute for persistent sidebar
final goRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    routes: [
      ShellRoute(
        builder: (context, state, child) => MainLayout(child: child),
        routes: [
          GoRoute(
            path: '/',
            builder: (context, state) => const AgentTeamsDashboard(),
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
          GoRoute(
            path: '/plugins',
            builder: (context, state) => const PluginManagerScreen(),
          ),
          GoRoute(
            path: '/swarm',
            builder: (context, state) => const SwarmDashboardScreen(),
          ),
          GoRoute(
            path: '/server-config',
            builder: (context, state) => const ServerConfigScreen(),
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
    final screenWidth = MediaQuery.of(context).size.width;
    final isMobile = screenWidth < 600;

    if (isMobile) {
      // On mobile, just show the child directly (full screen)
      return child;
    }

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