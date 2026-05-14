import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../data/stores/agent_store.dart';
import '../../data/stores/orchestrator_store.dart';
import '../../core/orchestrator/orchestrator.dart';
import '../../data/models/agent.dart';

/// Multi-Agent view - parallel agent orchestration
class MultiAgentView extends ConsumerWidget {
  const MultiAgentView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final tasks = ref.watch(tasksProvider);
    final activeTasks = ref.watch(activeTasksProvider);

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
                  onPressed: () => _createTask(ref, context),
                ),
              ],
            ),
          ),
          const Divider(),
          // Stats bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Row(
              children: [
                _StatBadge(label: 'Total Tasks', value: tasks.length.toString()),
                const SizedBox(width: 16),
                _StatBadge(label: 'Active', value: activeTasks.length.toString()),
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
                  child: _TaskProgressPanel(tasks: activeTasks),
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

  void _createTask(WidgetRef ref, BuildContext context) {
    final orchestrator = ref.read(orchestratorProvider);
    final task = orchestrator.createTask('New multi-agent task');

    // Show dialog to configure task
    showDialog(
      context: context,
      builder: (context) => _TaskDialog(
        task: task,
        onSubmit: (description) {
          final updatedTask = orchestrator.assignAgents(task.id, ['planner-1']);
          orchestrator.executeTask(updatedTask.id);
        },
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

class _AgentListPanel extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final agents = ref.watch(agentListProvider);
    final statusMap = ref.watch(agentStatusProvider);

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Agents',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        const Divider(),
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.all(8),
            itemCount: agents.length,
            itemBuilder: (context, index) {
              final agent = agents[index];
              final status = statusMap[agent.id] ?? agent.status;
              return ListTile(
                dense: true,
                leading: Icon(
                  _getAgentIcon(agent.type),
                  color: Theme.of(context).colorScheme.primary,
                ),
                title: Text(agent.name),
                subtitle: Text(_getStatusName(status)),
                trailing: Container(
                  width: 12,
                  height: 12,
                  decoration: BoxDecoration(
                    color: _getStatusColor(status),
                    shape: BoxShape.circle,
                  ),
                ),
              );
            },
          ),
        ),
      ],
    );
  }

  IconData _getAgentIcon(AgentType type) {
    switch (type) {
      case AgentType.planner:
        return Icons.architecture;
      case AgentType.architect:
        return Icons.domain;
      case AgentType.codeReviewer:
        return Icons.code;
      case AgentType.tddGuide:
        return Icons.science;
      case AgentType.securityReviewer:
        return Icons.security;
      default:
        return Icons.smart_toy;
    }
  }

  String _getStatusName(AgentStatus status) {
    switch (status) {
      case AgentStatus.idle:
        return 'idle';
      case AgentStatus.busy:
        return 'busy';
      case AgentStatus.error:
        return 'error';
      case AgentStatus.offline:
        return 'offline';
    }
  }

  Color _getStatusColor(AgentStatus status) {
    switch (status) {
      case AgentStatus.idle:
        return Colors.green;
      case AgentStatus.busy:
        return Colors.orange;
      case AgentStatus.error:
        return Colors.red;
      case AgentStatus.offline:
        return Colors.grey;
    }
  }
}

class _TaskProgressPanel extends StatelessWidget {
  const _TaskProgressPanel({required this.tasks});

  final List<Task> tasks;

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
        const Divider(),
        Expanded(
          child: tasks.isEmpty
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(Icons.task_alt, size: 64, color: Colors.grey),
                      const SizedBox(height: 16),
                      Text(
                        'No active tasks',
                        style: Theme.of(context).textTheme.bodyLarge?.copyWith(
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
                    return _TaskCard(task: task);
                  },
                ),
        ),
      ],
    );
  }
}

class _TaskCard extends StatelessWidget {
  const _TaskCard({required this.task});

  final Task task;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.all(8),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(_getStatusIcon(task.status), size: 20),
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
            Text(
              'Agents: ${task.assignedAgents.join(", ")}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
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

class _OutputStreamPanel extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // Watch task stream for updates
    final taskAsync = ref.watch(taskStreamProvider);

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Text(
                'Output Stream',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const Spacer(),
              IconButton(
                icon: const Icon(Icons.clear_all),
                onPressed: () {},
              ),
            ],
          ),
        ),
        const Divider(),
        Expanded(
          child: Container(
            padding: const EdgeInsets.all(8),
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: taskAsync.when(
              data: (task) => Text('Task ${task.id}: ${task.status.name}'),
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (err, _) => Center(child: Text('Error: $err')),
            ),
          ),
        ),
      ],
    );
  }
}

class _TaskDialog extends StatelessWidget {
  const _TaskDialog({
    required this.task,
    required this.onSubmit,
  });

  final Task task;
  final void Function(String) onSubmit;

  @override
  Widget build(BuildContext context) {
    final controller = TextEditingController(text: task.description);

    return AlertDialog(
      title: const Text('Create Task'),
      content: TextField(
        controller: controller,
        decoration: const InputDecoration(
          labelText: 'Task Description',
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        ElevatedButton(
          onPressed: () {
            Navigator.pop(context);
            onSubmit(controller.text);
          },
          child: const Text('Create'),
        ),
      ],
    );
  }
}