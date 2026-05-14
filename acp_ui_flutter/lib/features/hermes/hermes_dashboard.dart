import 'package:flutter/material.dart';

/// Hermes Dashboard - real-time monitoring and log stream
class HermesDashboard extends StatelessWidget {
  const HermesDashboard({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Text(
                  'Hermes Monitoring',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.refresh),
                  onPressed: () {},
                ),
              ],
            ),
          ),
          const Divider(),
          // Main content
          Expanded(
            child: Row(
              children: [
                // Task graph
                Expanded(
                  flex: 2,
                  child: _TaskGraphView(),
                ),
                const VerticalDivider(),
                // Log stream
                Expanded(
                  flex: 1,
                  child: _LogStreamView(),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _TaskGraphView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Task Graph',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        Expanded(
          child: Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.account_tree, size: 64, color: Colors.grey),
                const SizedBox(height: 16),
                Text(
                  'No active tasks',
                  style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

class _LogStreamView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final logs = [
      {'level': 'info', 'message': 'Agent started', 'time': '19:45'},
      {'level': 'debug', 'message': 'Processing request', 'time': '19:44'},
      {'level': 'warn', 'message': 'Context near limit', 'time': '19:43'},
      {'level': 'info', 'message': 'Task completed', 'time': '19:42'},
    ];

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Text(
                'Log Stream',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const Spacer(),
              IconButton(
                icon: const Icon(Icons.filter_list),
                onPressed: () {},
              ),
            ],
          ),
        ),
        Expanded(
          child: ListView.builder(
            itemCount: logs.length,
            itemBuilder: (context, index) {
              final log = logs[index];
              Color logColor;
              switch (log['level']) {
                case 'error':
                  logColor = Colors.red;
                  break;
                case 'warn':
                  logColor = Colors.orange;
                  break;
                case 'debug':
                  logColor = Colors.grey;
                  break;
                default:
                  logColor = Colors.blue;
              }

              return ListTile(
                dense: true,
                leading: Container(
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    color: logColor,
                    shape: BoxShape.circle,
                  ),
                ),
                title: Text(
                  log['message'] as String,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
                subtitle: Text(
                  log['time'] as String,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}