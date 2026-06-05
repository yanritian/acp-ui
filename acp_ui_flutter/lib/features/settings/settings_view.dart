import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../data/services/config_service.dart';

/// Settings view - agent and gateway configuration
class SettingsView extends ConsumerWidget {
  const SettingsView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final configAsync = ref.watch(configServiceProvider);

    return Scaffold(
      body: configAsync.when(
        data: (config) => _buildSettings(context, ref, config),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, _) => Center(child: Text('Error: $err')),
      ),
    );
  }

  Widget _buildSettings(BuildContext context, WidgetRef ref, ConfigService config) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Settings',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 24),
          _SettingsSection(
            title: 'Agent Configuration',
            children: [
              _SettingsDropdown(
                title: 'Default Agent',
                subtitle: 'Select the default agent for new conversations',
                value: config.defaultAgent,
                items: ['general-purpose', 'planner', 'architect', 'code-reviewer', 'tdd-guide', 'security-reviewer'],
                onChanged: (value) => config.setDefaultAgent(value),
              ),
              _SettingsSlider(
                title: 'Max Context Tokens',
                subtitle: '${config.maxContextTokens} tokens',
                value: config.maxContextTokens.toDouble(),
                min: 50000,
                max: 500000,
                onChanged: (value) => config.setMaxContextTokens(value.round()),
              ),
            ],
          ),
          const SizedBox(height: 16),
          _SettingsSection(
            title: 'Gateway Configuration',
            children: [
              _GatewayItem(
                title: 'Feishu Gateway',
                subtitle: 'Connected',
                connected: true,
                onToggle: () => _configureFeishu(context),
              ),
              _GatewayItem(
                title: 'Telegram Gateway',
                subtitle: 'Disconnected',
                connected: false,
                onToggle: () => _configureTelegram(context),
              ),
            ],
          ),
          const SizedBox(height: 16),
          _SettingsSection(
            title: 'Permission Mode',
            children: [
              _SettingsDropdown(
                title: 'Default Permission',
                subtitle: _getPermissionDescription(config.permissionMode),
                value: config.permissionMode,
                items: ['ReadOnly', 'WorkspaceWrite', 'Allow', 'Prompt', 'DangerFullAccess'],
                onChanged: (value) => config.setPermissionMode(value),
              ),
            ],
          ),
          const SizedBox(height: 16),
          _SettingsSection(
            title: 'Appearance',
            children: [
              _SettingsDropdown(
                title: 'Theme Mode',
                subtitle: 'Light, Dark, or System default',
                value: config.themeMode,
                items: ['light', 'dark', 'system'],
                onChanged: (value) => config.setThemeMode(value),
              ),
            ],
          ),
          const SizedBox(height: 16),
          _SettingsSection(
            title: 'Connection',
            children: [
              _SettingsInput(
                title: 'Base URL',
                subtitle: 'WebSocket server address',
                value: config.baseUrl,
                onChanged: (value) => config.setBaseUrl(value),
              ),
            ],
          ),
          const SizedBox(height: 24),
          // Reset button
          Center(
            child: ElevatedButton.icon(
              icon: const Icon(Icons.refresh),
              label: const Text('Reset to Defaults'),
              onPressed: () => _resetDefaults(context, ref, config),
            ),
          ),
        ],
      ),
    );
  }

  String _getPermissionDescription(String mode) {
    switch (mode) {
      case 'ReadOnly':
        return 'Only read operations allowed';
      case 'WorkspaceWrite':
        return 'Read and write within workspace';
      case 'Allow':
        return 'Auto-allow with rule-based checks';
      case 'Prompt':
        return 'Ask user for each operation';
      case 'DangerFullAccess':
        return 'Full access without checks (dangerous)';
      default:
        return 'Unknown mode';
    }
  }

  void _configureFeishu(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Configure Feishu Gateway'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              decoration: const InputDecoration(labelText: 'App ID'),
            ),
            TextField(
              decoration: const InputDecoration(labelText: 'App Secret'),
              obscureText: true,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }

  void _configureTelegram(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Configure Telegram Gateway'),
        content: TextField(
          decoration: const InputDecoration(labelText: 'Bot Token'),
          obscureText: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }

  void _resetDefaults(BuildContext context, WidgetRef ref, ConfigService config) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Reset Settings'),
        content: const Text('Are you sure you want to reset all settings to defaults?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          ElevatedButton(
            onPressed: () {
              config.clear();
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Settings reset to defaults')),
              );
            },
            child: const Text('Reset'),
          ),
        ],
      ),
    );
  }
}

class _SettingsSection extends StatelessWidget {
  const _SettingsSection({
    required this.title,
    required this.children,
  });

  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              title,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 16),
            ...children,
          ],
        ),
      ),
    );
  }
}

class _SettingsDropdown extends StatelessWidget {
  const _SettingsDropdown({
    required this.title,
    required this.subtitle,
    required this.value,
    required this.items,
    required this.onChanged,
  });

  final String title;
  final String subtitle;
  final String value;
  final List<String> items;
  final void Function(String) onChanged;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(title),
      subtitle: Text(subtitle),
      trailing: DropdownButton<String>(
        value: value,
        underline: const SizedBox(),
        items: items.map((item) {
          return DropdownMenuItem(
            value: item,
            child: Text(item),
          );
        }).toList(),
        onChanged: (newValue) {
          if (newValue != null) onChanged(newValue);
        },
      ),
    );
  }
}

class _SettingsSlider extends StatelessWidget {
  const _SettingsSlider({
    required this.title,
    required this.subtitle,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
  });

  final String title;
  final String subtitle;
  final double value;
  final double min;
  final double max;
  final void Function(double) onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        ListTile(
          title: Text(title),
          subtitle: Text(subtitle),
        ),
        Slider(
          value: value,
          min: min,
          max: max,
          divisions: 10,
          onChanged: onChanged,
        ),
      ],
    );
  }
}

class _SettingsInput extends StatelessWidget {
  const _SettingsInput({
    required this.title,
    required this.subtitle,
    required this.value,
    required this.onChanged,
  });

  final String title;
  final String subtitle;
  final String value;
  final void Function(String) onChanged;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(title),
      subtitle: Text(subtitle),
      trailing: SizedBox(
        width: 200,
        child: TextField(
          controller: TextEditingController(text: value),
          decoration: const InputDecoration(
            border: OutlineInputBorder(),
          ),
          onSubmitted: onChanged,
        ),
      ),
    );
  }
}

class _GatewayItem extends StatelessWidget {
  const _GatewayItem({
    required this.title,
    required this.subtitle,
    required this.connected,
    required this.onToggle,
  });

  final String title;
  final String subtitle;
  final bool connected;
  final VoidCallback onToggle;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(
        connected ? Icons.link : Icons.link_off,
        color: connected ? Colors.green : Colors.grey,
      ),
      title: Text(title),
      subtitle: Text(subtitle),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Switch(
            value: connected,
            onChanged: (_) => onToggle(),
          ),
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: onToggle,
          ),
        ],
      ),
    );
  }
}