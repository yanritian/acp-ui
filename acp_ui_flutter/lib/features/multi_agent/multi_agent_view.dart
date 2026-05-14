import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Multi-Agent view - parallel agent orchestration
class MultiAgentView extends ConsumerWidget {
  const MultiAgentView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      body: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Text(
                  'Multi-Agent Orchestration',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                const Spacer(),
                ElevatedButton.icon(
                  icon: const Icon(Icons.add),
                  label: const Text('New Task'),
                  onPressed: () {
                    // TODO: Create new task
                  },
                ),
              ],
            ),
          ),
          const Divider(),
          // Agent status panels
          Expanded(
            child: Row(
              children: [
                // Agent list
                SizedBox(
                  width: 200,
                  child: _AgentListPanel(),
                ),
                const VerticalDivider(),
                // Task progress
                Expanded(
                  child: _TaskProgressPanel(),
                ),
                const VerticalDivider(),
                // Output stream
                SizedBox(
                  width: 300,
                  child: _OutputStreamPanel(),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _AgentListPanel extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final agents = [
      {'name': 'planner', 'status': 'idle', 'icon': Icons.architecture},
      {'name': 'architect', 'status': 'idle', 'icon': Icons.domain},
      {'name': 'code-reviewer', 'status': 'idle', 'icon': Icons.code},
      {'name': 'tdd-guide', 'status': 'idle', 'icon': Icons.science},
      {'name': 'security-reviewer', 'status': 'idle', 'icon': Icons.security},
    ];

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: agents.length,
      itemBuilder: (context, index) {
        final agent = agents[index];
        return ListTile(
          leading: Icon(
            agent['icon'] as IconData,
            color: Theme.of(context).colorScheme.primary,
          ),
          title: Text(agent['name'] as String),
          subtitle: Text(agent['status'] as String),
          trailing: Container(
            width: 12,
            height: 12,
            decoration: BoxDecoration(
              color: agent['status'] == 'idle' ? Colors.green : Colors.orange,
              shape: BoxShape.circle,
            ),
          ),
        );
      },
    );
  }
}

class _TaskProgressPanel extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Task Progress',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        Expanded(
          child: Center(
            child: Text(
              'No active tasks',
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _OutputStreamPanel extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Output Stream',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        Expanded(
          child: Container(
            padding: const EdgeInsets.all(8),
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: const Center(
              child: Text('Waiting for output...'),
            ),
          ),
        ),
      ],
    );
  }
}