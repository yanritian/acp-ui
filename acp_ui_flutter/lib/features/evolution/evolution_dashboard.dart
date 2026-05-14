import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../data/stores/evolution_store.dart';
import '../../core/self_improvement/self_healing.dart';

/// Evolution Dashboard - self-healing and evolution monitoring
class EvolutionDashboard extends ConsumerWidget {
  const EvolutionDashboard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
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

class _HealingLogPanel extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final healingLog = ref.watch(healingLogProvider);

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Text(
                'Healing Log',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const Spacer(),
              IconButton(
                icon: const Icon(Icons.add),
                onPressed: () => _triggerTestHealing(ref),
              ),
            ],
          ),
        ),
        const Divider(),
        Expanded(
          child: healingLog.isEmpty
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.history, size: 64, color: Colors.grey),
                      const SizedBox(height: 16),
                      Text(
                        'No healing events',
                        style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  itemCount: healingLog.length,
                  itemBuilder: (context, index) {
                    final event = healingLog[index];
                    return ListTile(
                      leading: Icon(
                        event.status == 'success' ? Icons.check_circle : Icons.pending,
                        color: event.status == 'success' ? Colors.green : Colors.orange,
                      ),
                      title: Text(_getEventTypeName(event.type)),
                      subtitle: Text(_formatTime(event.timestamp)),
                    );
                  },
                ),
        ),
      ],
    );
  }

  void _triggerTestHealing(WidgetRef ref) {
    final system = ref.read(selfHealingProvider);
    system.recordEvent(HealingEventType.errorRecovery, 'success', message: 'Test healing triggered');
  }

  String _getEventTypeName(HealingEventType type) {
    switch (type) {
      case HealingEventType.errorRecovery:
        return 'Error Recovery';
      case HealingEventType.memoryCompaction:
        return 'Memory Compaction';
      case HealingEventType.agentRestart:
        return 'Agent Restart';
      case HealingEventType.contextReset:
        return 'Context Reset';
      case HealingEventType.cacheClear:
        return 'Cache Clear';
    }
  }

  String _formatTime(DateTime time) {
    return '${time.hour}:${time.minute.toString().padLeft(2, '0')}';
  }
}

class _PatternViewPanel extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final patterns = ref.watch(evolutionPatternsProvider);

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
              IconButton(
                icon: const Icon(Icons.lightbulb_outline),
                onPressed: () => _learnTestPattern(ref),
              ),
            ],
          ),
        ),
        const Divider(),
        Expanded(
          child: patterns.isEmpty
              ? Center(
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
                      const SizedBox(height: 8),
                      TextButton(
                        onPressed: () => _learnTestPattern(ref),
                        child: const Text('Learn test pattern'),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  itemCount: patterns.length,
                  itemBuilder: (context, index) {
                    final pattern = patterns[index];
                    return ListTile(
                      leading: const Icon(Icons.pattern),
                      title: Text(pattern.name),
                      subtitle: Text(pattern.description),
                      trailing: Text(
                        '${pattern.frequency}x',
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    );
                  },
                ),
        ),
      ],
    );
  }

  void _learnTestPattern(WidgetRef ref) {
    final engine = ref.read(evolutionEngineProvider);
    engine.learnPattern(
      'Error Recovery Pattern',
      'Automatic recovery from common errors',
    );
  }
}

class _StatisticsPanel extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final stats = ref.watch(healingStatsProvider);
    final evolutionScore = ref.watch(evolutionScoreProvider);

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
              _StatItem(
                label: 'Healing Events',
                value: '${stats['totalEvents'] ?? 0}',
              ),
              _StatItem(
                label: 'Success Rate',
                value: '${(stats['successRate'] ?? 0).toStringAsFixed(1)}%',
              ),
              _StatItem(
                label: 'Evolution Score',
                value: evolutionScore.toStringAsFixed(1),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        // Suggestion section
        Container(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Suggestions',
                style: Theme.of(context).textTheme.titleSmall,
              ),
              const SizedBox(height: 8),
              _buildSuggestion(context, ref),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildSuggestion(BuildContext context, WidgetRef ref) {
    final engine = ref.read(evolutionEngineProvider);
    final suggestion = engine.suggestImprovement();

    if (suggestion == null) {
      return Text(
        'No suggestions yet',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(
          color: Theme.of(context).colorScheme.onSurfaceVariant,
        ),
      );
    }

    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.primaryContainer.withOpacity(0.3),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(Icons.lightbulb, size: 16, color: Theme.of(context).colorScheme.primary),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              suggestion,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        ],
      ),
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