import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

/// Sidebar navigation widget
class Sidebar extends ConsumerWidget {
  const Sidebar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final currentPath = GoRouterState.of(context).path;

    return Container(
      width: 240,
      color: Theme.of(context).colorScheme.surface,
      child: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Icon(
                  Icons.smart_toy_outlined,
                  color: Theme.of(context).colorScheme.primary,
                  size: 32,
                ),
                const SizedBox(width: 12),
                Text(
                  'ACP-UI',
                  style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
          ),
          const Divider(),
          // Navigation items
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(vertical: 8),
              children: [
                _NavItem(
                  icon: Icons.chat_outlined,
                  label: 'Chat',
                  path: '/',
                  isSelected: currentPath == '/',
                ),
                _NavItem(
                  icon: Icons.people_outline,
                  label: 'Multi-Agent',
                  path: '/multi-agent',
                  isSelected: currentPath == '/multi-agent',
                ),
                _NavItem(
                  icon: Icons.history_outlined,
                  label: 'History',
                  path: '/history',
                  isSelected: currentPath == '/history',
                ),
                _NavItem(
                  icon: Icons.auto_fix_high_outlined,
                  label: 'Evolution',
                  path: '/evolution',
                  isSelected: currentPath == '/evolution',
                ),
                _NavItem(
                  icon: Icons.monitor_outlined,
                  label: 'Hermes',
                  path: '/hermes',
                  isSelected: currentPath == '/hermes',
                ),
                const Divider(),
                _NavItem(
                  icon: Icons.settings_outlined,
                  label: 'Settings',
                  path: '/settings',
                  isSelected: currentPath == '/settings',
                ),
              ],
            ),
          ),
          // Status indicator
          Container(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                const Icon(Icons.circle, color: Colors.green, size: 12),
                const SizedBox(width: 8),
                Text(
                  'Connected',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _NavItem extends StatelessWidget {
  const _NavItem({
    required this.icon,
    required this.label,
    required this.path,
    required this.isSelected,
  });

  final IconData icon;
  final String label;
  final String path;
  final bool isSelected;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(
        icon,
        color: isSelected
          ? Theme.of(context).colorScheme.primary
          : Theme.of(context).colorScheme.onSurfaceVariant,
      ),
      title: Text(
        label,
        style: TextStyle(
          color: isSelected
            ? Theme.of(context).colorScheme.primary
            : Theme.of(context).colorScheme.onSurface,
          fontWeight: isSelected ? FontWeight.w600 : FontWeight.normal,
        ),
      ),
      selected: isSelected,
      selectedTileColor: Theme.of(context).colorScheme.primaryContainer.withOpacity(0.3),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8),
      ),
      onTap: () => context.go(path),
    );
  }
}