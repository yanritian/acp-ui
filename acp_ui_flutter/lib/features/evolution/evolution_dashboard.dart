import 'package:flutter/material.dart';

/// Evolution Dashboard - self-healing and evolution monitoring
class EvolutionDashboard extends StatelessWidget {
  const EvolutionDashboard({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          // Left panel - healing log
          SizedBox(
            width: 300,
            child: _HealingLogPanel(),
          ),
          const VerticalDivider(),
          // Center - pattern view
          Expanded(
            child: _PatternViewPanel(),
          ),
          const VerticalDivider(),
          // Right - statistics
          SizedBox(
            width: 250,
            child: _StatisticsPanel(),
          ),
        ],
      ),
    );
  }
}

class _HealingLogPanel extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final healingEvents = [
      {'type': 'Error Recovery', 'status': 'success', 'time': '19:45'},
      {'type': 'Memory Compaction', 'status': 'success', 'time': '19:30'},
      {'type': 'Agent Restart', 'status': 'running', 'time': '19:15'},
    ];

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Healing Log',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        const Divider(),
        Expanded(
          child: ListView.builder(
            itemCount: healingEvents.length,
            itemBuilder: (context, index) {
              final event = healingEvents[index];
              return ListTile(
                leading: Icon(
                  event['status'] == 'success' ? Icons.check_circle : Icons.pending,
                  color: event['status'] == 'success' ? Colors.green : Colors.orange,
                ),
                title: Text(event['type'] as String),
                subtitle: Text(event['time'] as String),
              );
            },
          ),
        ),
      ],
    );
  }
}

class _PatternViewPanel extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Text(
                'Evolution Patterns',
                style: Theme.of(context).textTheme.titleMedium,
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
        Expanded(
          child: Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.auto_fix_high, size: 64, color: Colors.grey),
                const SizedBox(height: 16),
                Text(
                  'No patterns detected',
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

class _StatisticsPanel extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Statistics',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        const Divider(),
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            children: [
              _StatItem(label: 'Healing Events', value: '12'),
              _StatItem(label: 'Success Rate', value: '94%'),
              _StatItem(label: 'Patterns Learned', value: '5'),
              _StatItem(label: 'Evolution Score', value: '8.5'),
            ],
          ),
        ),
      ],
    );
  }
}

class _StatItem extends StatelessWidget {
  const _StatItem({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          Text(label),
          const Spacer(),
          Text(
            value,
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ],
      ),
    );
  }
}