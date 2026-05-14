import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../data/stores/orchestrator_store.dart';
import '../../data/stores/agent_store.dart';
import '../../data/stores/evolution_store.dart';
import '../../core/orchestrator/orchestrator.dart';
import '../../data/models/agent.dart';

/// Hermes Dashboard - real-time monitoring and log stream
class HermesDashboard extends ConsumerWidget {
  const HermesDashboard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final tasks = ref.watch(tasksProvider);
    final agents = ref.watch(agentListProvider);
    final healingEvents = ref.watch(healingLogProvider);

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
                // Stats badges
                _StatBadge(label: 'Tasks', value: tasks.length.toString()),
                const SizedBox(width: 8),
                _StatBadge(label: 'Agents', value: agents.length.toString()),
                const SizedBox(width: 8),
                _StatBadge(label: 'Events', value: healingEvents.length.toString()),
                const SizedBox(width: 16),
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
                  child: _TaskGraphView(tasks: tasks),
                ),
                const VerticalDivider(),
                // Agent status
                SizedBox(
                  width: 200,
                  child: _AgentStatusPanel(agents: agents),
                ),
                const VerticalDivider(),
                // Log stream
                Expanded(
                  flex: 1,
                  child: _LogStreamView(events: healingEvents),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _StatBadge extends StatelessWidget {
  const _StatBadge({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(label, style: Theme.of(context).textTheme.bodySmall),
          const SizedBox(width: 8),
          Text(value, style: Theme.of(context).textTheme.titleSmall),
        ],
      ),
    );
  }
}

class _TaskGraphView extends StatelessWidget {
  const _TaskGraphView({required this.tasks});

  final List<Task> tasks;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Text(
                'Task Graph',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const Spacer(),
              if (tasks.isNotEmpty)
                TextButton.icon(
                  icon: const Icon(Icons.clear_all),
                  label: const Text('Clear'),
                  onPressed: () {},
                ),
            ],
          ),
        ),
        const Divider(),
        Expanded(
          child: tasks.isEmpty
              ? Center(
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
                      const SizedBox(height: 8),
                      Text(
                        'Create a task in Multi-Agent view',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  itemCount: tasks.length,
                  itemBuilder: (context, index) {
                    final task = tasks[index];
                    return _TaskNode(task: task);
                  },
                ),
        ),
      ],
    );
  }
}

class _TaskNode extends StatelessWidget {
  const _TaskNode({required this.task});

  final Task task;

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.all(8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: _getStatusColor(task.status).withOpacity(0.1),
        border: Border.all(color: _getStatusColor(task.status)),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(_getStatusIcon(task.status), size: 16, color: _getStatusColor(task.status)),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  task.description,
                  style: Theme.of(context).textTheme.titleSmall,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          if (task.assignedAgents.isNotEmpty)
            Text(
              'Agents: ${task.assignedAgents.join(", ")}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          if (task.result != null)
            Text(
              'Result: ${task.result}',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
        ],
      ),
    );
  }

  Color _getStatusColor(TaskStatus status) {
    switch (status) {
      case TaskStatus.pending:
        return Colors.grey;
      case TaskStatus.running:
        return Colors.blue;
      case TaskStatus.completed:
        return Colors.green;
      case TaskStatus.failed:
        return Colors.red;
      case TaskStatus.paused:
        return Colors.orange;
    }
  }

  IconData _getStatusIcon(TaskStatus status) {
    switch (status) {
      case TaskStatus.pending:
        return Icons.pending;
      case TaskStatus.running:
        return Icons.play_arrow;
      case TaskStatus.completed:
        return Icons.check_circle;
      case TaskStatus.failed:
        return Icons.error;
      case TaskStatus.paused:
        return Icons.pause;
    }
  }
}

class _AgentStatusPanel extends StatelessWidget {
  const _AgentStatusPanel({required this.agents});

  final List<Agent> agents;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Agent Status',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        const Divider(),
        Expanded(
          child: ListView.builder(
            itemCount: agents.length,
            itemBuilder: (context, index) {
              final agent = agents[index];
              return ListTile(
                dense: true,
                leading: Container(
                  width: 12,
                  height: 12,
                  decoration: BoxDecoration(
                    color: _getStatusColor(agent.status),
                    shape: BoxShape.circle,
                  ),
                ),
                title: Text(agent.name),
                subtitle: Text(agent.status.name),
              );
            },
          ),
        ),
      ],
    );
  }

  Color _getStatusColor(AgentStatus status) {
    switch (status) {
      case AgentStatus.idle:
        return Colors.green;
      case AgentStatus.busy:
        return Colors.blue;
      case AgentStatus.error:
        return Colors.red;
      case AgentStatus.offline:
        return Colors.grey;
    }
  }
}

class _LogStreamView extends StatelessWidget {
  const _LogStreamView({required this.events});

  final List<dynamic> events;

  @override
  Widget build(BuildContext context) {
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
              IconButton(
                icon: const Icon(Icons.clear_all),
                onPressed: () {},
              ),
            ],
          ),
        ),
        const Divider(),
        Expanded(
          child: events.isEmpty
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.article, size: 64, color: Colors.grey),
                      const SizedBox(height: 16),
                      Text(
                        'No log events',
                        style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                  ),
                )
              : ListView.builder(
                  itemCount: events.length,
                  itemBuilder: (context, index) {
                    final event = events[index];
                    return ListTile(
                      dense: true,
                      leading: Container(
                        width: 8,
                        height: 8,
                        decoration: BoxDecoration(
                          color: event.status == 'success' ? Colors.green : Colors.orange,
                          shape: BoxShape.circle,
                        ),
                      ),
                      title: Text(
                        event.toString(),
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    );
                  },
                ),
        ),
      ],
    );
  }
}