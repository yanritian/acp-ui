import 'package:flutter/material.dart';

/// Settings view - agent and gateway configuration
class SettingsView extends StatelessWidget {
  const SettingsView({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SingleChildScrollView(
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
                _SettingsItem(
                  title: 'Default Agent',
                  subtitle: 'Select the default agent for new conversations',
                  trailing: DropdownButton<String>(
                    value: 'general-purpose',
                    items: const [
                      DropdownMenuItem(value: 'general-purpose', child: Text('General Purpose')),
                      DropdownMenuItem(value: 'planner', child: Text('Planner')),
                      DropdownMenuItem(value: 'architect', child: Text('Architect')),
                      DropdownMenuItem(value: 'code-reviewer', child: Text('Code Reviewer')),
                    ],
                    onChanged: (_) {},
                  ),
                ),
                _SettingsItem(
                  title: 'Max Context Tokens',
                  subtitle: '200000 tokens',
                  trailing: Slider(
                    value: 200000,
                    min: 50000,
                    max: 500000,
                    onChanged: (_) {},
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            _SettingsSection(
              title: 'Gateway Configuration',
              children: [
                _SettingsItem(
                  title: 'Feishu Gateway',
                  subtitle: 'Connected',
                  trailing: Switch(
                    value: true,
                    onChanged: (_) {},
                  ),
                ),
                _SettingsItem(
                  title: 'Telegram Gateway',
                  subtitle: 'Disconnected',
                  trailing: Switch(
                    value: false,
                    onChanged: (_) {},
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            _SettingsSection(
              title: 'Permission Mode',
              children: [
                _SettingsItem(
                  title: 'Default Permission',
                  subtitle: 'Auto-allow with rule-based checks',
                  trailing: DropdownButton<String>(
                    value: 'Allow',
                    items: const [
                      DropdownMenuItem(value: 'ReadOnly', child: Text('ReadOnly')),
                      DropdownMenuItem(value: 'WorkspaceWrite', child: Text('WorkspaceWrite')),
                      DropdownMenuItem(value: 'Allow', child: Text('Allow')),
                      DropdownMenuItem(value: 'Prompt', child: Text('Prompt')),
                      DropdownMenuItem(value: 'DangerFullAccess', child: Text('DangerFullAccess')),
                    ],
                    onChanged: (_) {},
                  ),
                ),
              ],
            ),
          ],
        ),
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

class _SettingsItem extends StatelessWidget {
  const _SettingsItem({
    required this.title,
    required this.subtitle,
    required this.trailing,
  });

  final String title;
  final String subtitle;
  final Widget trailing;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      title: Text(title),
      subtitle: Text(subtitle),
      trailing: trailing,
    );
  }
}