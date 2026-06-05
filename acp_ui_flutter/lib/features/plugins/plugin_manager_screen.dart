import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/services/websocket_service.dart';

// ---------------------------------------------------------------------------
// Plugin model
// ---------------------------------------------------------------------------

/// Represents a single plugin returned by the backend.
class PluginInfo {
  final String id;
  final String name;
  final String kind;
  final bool enabled;
  final String health; // 'healthy' | 'degraded' | 'error' | 'unknown'
  final String? version;
  final String? description;

  const PluginInfo({
    required this.id,
    required this.name,
    required this.kind,
    required this.enabled,
    required this.health,
    this.version,
    this.description,
  });

  factory PluginInfo.fromJson(Map<String, dynamic> json) {
    return PluginInfo(
      id: json['id'] as String? ?? json['name'] as String? ?? '',
      name: json['name'] as String? ?? json['id'] as String? ?? 'Unknown',
      kind: json['kind'] as String? ?? json['type'] as String? ?? 'unknown',
      enabled: json['enabled'] as bool? ?? false,
      health: json['health'] as String? ?? json['status'] as String? ?? 'unknown',
      version: json['version'] as String?,
      description: json['description'] as String?,
    );
  }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

class PluginManagerState {
  final List<PluginInfo> plugins;
  final bool isLoading;
  final String? error;

  const PluginManagerState({
    this.plugins = const [],
    this.isLoading = false,
    this.error,
  });

  PluginManagerState copyWith({
    List<PluginInfo>? plugins,
    bool? isLoading,
    String? error,
  }) {
    return PluginManagerState(
      plugins: plugins ?? this.plugins,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }
}

class PluginManagerNotifier extends StateNotifier<PluginManagerState> {
  final WebSocketService _ws;

  PluginManagerNotifier(this._ws) : super(const PluginManagerState());

  /// Fetch plugin list from backend via the proxy command.
  Future<void> loadPlugins() async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final response = await _ws.proxyCommand('plugin_list', {});
      final data = response['data'] as Map<String, dynamic>? ?? response;
      final list = (data['plugins'] as List?) ?? [];
      final plugins =
          list.map((e) => PluginInfo.fromJson(e as Map<String, dynamic>)).toList();
      state = state.copyWith(plugins: plugins, isLoading: false);
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  /// Toggle plugin enabled state via backend.
  Future<void> togglePlugin(String pluginId, bool enabled) async {
    try {
      await _ws.proxyCommand('plugin_toggle', {
        'plugin_id': pluginId,
        'enabled': enabled,
      });
      // Optimistic local update
      state = state.copyWith(
        plugins: state.plugins.map((p) {
          if (p.id == pluginId) {
            return PluginInfo(
              id: p.id,
              name: p.name,
              kind: p.kind,
              enabled: enabled,
              health: p.health,
              version: p.version,
              description: p.description,
            );
          }
          return p;
        }).toList(),
      );
    } catch (e) {
      // Revert on failure by reloading
      await loadPlugins();
    }
  }
}

final pluginManagerProvider =
    StateNotifierProvider<PluginManagerNotifier, PluginManagerState>((ref) {
  final ws = ref.watch(webSocketServiceProvider);
  final notifier = PluginManagerNotifier(ws);
  // Auto-load when provider is first created
  Future.microtask(() => notifier.loadPlugins());
  return notifier;
});

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

class PluginManagerScreen extends ConsumerStatefulWidget {
  const PluginManagerScreen({super.key});

  @override
  ConsumerState<PluginManagerScreen> createState() => _PluginManagerScreenState();
}

class _PluginManagerScreenState extends ConsumerState<PluginManagerScreen> {
  final _searchController = TextEditingController();
  String _searchQuery = '';

  @override
  void initState() {
    super.initState();
    _searchController.addListener(() {
      setState(() => _searchQuery = _searchController.text.toLowerCase());
    });
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  List<PluginInfo> _filteredPlugins(List<PluginInfo> plugins) {
    if (_searchQuery.isEmpty) return plugins;
    return plugins.where((p) {
      return p.name.toLowerCase().contains(_searchQuery) ||
          p.kind.toLowerCase().contains(_searchQuery) ||
          p.id.toLowerCase().contains(_searchQuery);
    }).toList();
  }

  Color _healthColor(String health) {
    switch (health) {
      case 'healthy':
        return Colors.green;
      case 'degraded':
        return Colors.orange;
      case 'error':
        return Colors.red;
      default:
        return Colors.grey;
    }
  }

  IconData _kindIcon(String kind) {
    switch (kind.toLowerCase()) {
      case 'llm':
        return Icons.psychology_rounded;
      case 'tool':
        return Icons.build_rounded;
      case 'memory':
        return Icons.memory_rounded;
      case 'gateway':
        return Icons.router_rounded;
      case 'adapter':
        return Icons.swap_horiz_rounded;
      default:
        return Icons.extension_rounded;
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(pluginManagerProvider);
    final theme = Theme.of(context);
    final screenWidth = MediaQuery.of(context).size.width;
    final isDesktop = screenWidth > 900;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Plugin Manager'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh_rounded),
            tooltip: 'Refresh',
            onPressed: () => ref.read(pluginManagerProvider.notifier).loadPlugins(),
          ),
        ],
      ),
      body: Column(
        children: [
          // Search bar
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
            child: TextField(
              controller: _searchController,
              decoration: InputDecoration(
                hintText: 'Search plugins...',
                prefixIcon: const Icon(Icons.search_rounded),
                suffixIcon: _searchQuery.isNotEmpty
                    ? IconButton(
                        icon: const Icon(Icons.clear_rounded),
                        onPressed: () {
                          _searchController.clear();
                        },
                      )
                    : null,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                  borderSide: BorderSide.none,
                ),
                filled: true,
              ),
            ),
          ),
          // Content
          Expanded(
            child: state.isLoading
                ? const Center(child: CircularProgressIndicator())
                : state.error != null
                    ? _buildErrorState(state.error!, theme)
                    : _buildPluginList(state, theme, isDesktop),
          ),
        ],
      ),
    );
  }

  Widget _buildErrorState(String error, ThemeData theme) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.error_outline_rounded,
                size: 56, color: theme.colorScheme.error),
            const SizedBox(height: 16),
            Text(
              'Failed to load plugins',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              error,
              textAlign: TextAlign.center,
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              icon: const Icon(Icons.refresh_rounded),
              label: const Text('Retry'),
              onPressed: () =>
                  ref.read(pluginManagerProvider.notifier).loadPlugins(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPluginList(
      PluginManagerState state, ThemeData theme, bool isDesktop) {
    final plugins = _filteredPlugins(state.plugins);

    if (plugins.isEmpty && _searchQuery.isNotEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.search_off_rounded,
                size: 56, color: theme.colorScheme.outline),
            const SizedBox(height: 12),
            Text('No plugins match "$_searchQuery"',
                style: theme.textTheme.bodyLarge),
          ],
        ),
      );
    }

    if (plugins.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.extension_outlined,
                size: 56, color: theme.colorScheme.outline),
            const SizedBox(height: 12),
            Text('No plugins available',
                style: theme.textTheme.bodyLarge),
            const SizedBox(height: 8),
            Text(
              'Plugins extend the capabilities of ACP-UI',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }

    // Use a grid on wider screens
    if (isDesktop) {
      return GridView.builder(
        padding: const EdgeInsets.all(16),
        gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
          maxCrossAxisExtent: 420,
          mainAxisExtent: 140,
          crossAxisSpacing: 12,
          mainAxisSpacing: 12,
        ),
        itemCount: plugins.length,
        itemBuilder: (context, index) =>
            _buildPluginCard(plugins[index], theme),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: plugins.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (context, index) =>
          _buildPluginCard(plugins[index], theme),
    );
  }

  Widget _buildPluginCard(PluginInfo plugin, ThemeData theme) {
    final healthColor = _healthColor(plugin.health);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            // Kind icon
            Container(
              width: 48,
              height: 48,
              decoration: BoxDecoration(
                color: theme.colorScheme.primaryContainer.withValues(alpha: 0.5),
                borderRadius: BorderRadius.circular(12),
              ),
              child: Icon(
                _kindIcon(plugin.kind),
                color: theme.colorScheme.primary,
                size: 24,
              ),
            ),
            const SizedBox(width: 14),
            // Info
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Row(
                    children: [
                      Flexible(
                        child: Text(
                          plugin.name,
                          style: theme.textTheme.titleSmall?.copyWith(
                            fontWeight: FontWeight.w600,
                          ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      if (plugin.version != null) ...[
                        const SizedBox(width: 6),
                        Container(
                          padding: const EdgeInsets.symmetric(
                              horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color:
                                theme.colorScheme.surfaceContainerHighest,
                            borderRadius: BorderRadius.circular(6),
                          ),
                          child: Text(
                            'v${plugin.version}',
                            style: theme.textTheme.bodySmall?.copyWith(
                              fontSize: 10,
                              color: theme.colorScheme.onSurfaceVariant,
                            ),
                          ),
                        ),
                      ],
                    ],
                  ),
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      // Kind badge
                      Container(
                        padding: const EdgeInsets.symmetric(
                            horizontal: 8, vertical: 2),
                        decoration: BoxDecoration(
                          color: theme.colorScheme.secondaryContainer
                              .withValues(alpha: 0.6),
                          borderRadius: BorderRadius.circular(6),
                        ),
                        child: Text(
                          plugin.kind,
                          style: theme.textTheme.bodySmall?.copyWith(
                            fontSize: 11,
                            color: theme.colorScheme.onSecondaryContainer,
                            fontWeight: FontWeight.w500,
                          ),
                        ),
                      ),
                      const SizedBox(width: 8),
                      // Health badge
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Container(
                            width: 8,
                            height: 8,
                            decoration: BoxDecoration(
                              color: healthColor,
                              shape: BoxShape.circle,
                            ),
                          ),
                          const SizedBox(width: 4),
                          Text(
                            plugin.health,
                            style: theme.textTheme.bodySmall?.copyWith(
                              fontSize: 11,
                              color: healthColor,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                  if (plugin.description != null) ...[
                    const SizedBox(height: 4),
                    Text(
                      plugin.description!,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ],
              ),
            ),
            const SizedBox(width: 8),
            // Enable/disable toggle
            Switch.adaptive(
              value: plugin.enabled,
              onChanged: (v) => ref
                  .read(pluginManagerProvider.notifier)
                  .togglePlugin(plugin.id, v),
              activeThumbColor: theme.colorScheme.primary,
            ),
          ],
        ),
      ),
    );
  }
}
