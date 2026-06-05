import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/services/websocket_service.dart';

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

/// Represents a swarm agent returned by the backend.
class SwarmAgent {
  final String id;
  final String name;
  final String role;
  final String status; // 'idle' | 'busy' | 'error' | 'offline'
  final String? currentTask;
  final int tasksCompleted;

  const SwarmAgent({
    required this.id,
    required this.name,
    required this.role,
    required this.status,
    this.currentTask,
    this.tasksCompleted = 0,
  });

  factory SwarmAgent.fromJson(Map<String, dynamic> json) {
    return SwarmAgent(
      id: json['id'] as String? ?? '',
      name: json['name'] as String? ?? 'Unknown',
      role: json['role'] as String? ?? json['type'] as String? ?? 'worker',
      status: json['status'] as String? ?? 'unknown',
      currentTask: json['current_task'] as String?,
      tasksCompleted: json['tasks_completed'] as int? ?? 0,
    );
  }
}

/// Represents a swarm task.
class SwarmTask {
  final String id;
  final String name;
  final String status; // 'pending' | 'running' | 'completed' | 'failed'
  final String? assignedAgent;
  final DateTime? createdAt;
  final double progress;

  const SwarmTask({
    required this.id,
    required this.name,
    required this.status,
    this.assignedAgent,
    this.createdAt,
    this.progress = 0.0,
  });

  factory SwarmTask.fromJson(Map<String, dynamic> json) {
    return SwarmTask(
      id: json['id'] as String? ?? '',
      name: json['name'] as String? ?? json['description'] as String? ?? 'Unnamed task',
      status: json['status'] as String? ?? 'pending',
      assignedAgent: json['assigned_agent'] as String?,
      createdAt: json['created_at'] != null
          ? DateTime.tryParse(json['created_at'] as String)
          : null,
      progress: (json['progress'] as num?)?.toDouble() ?? 0.0,
    );
  }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

class SwarmState {
  final List<SwarmAgent> agents;
  final List<SwarmTask> tasks;
  final bool isLoading;
  final String? error;

  const SwarmState({
    this.agents = const [],
    this.tasks = const [],
    this.isLoading = false,
    this.error,
  });

  SwarmState copyWith({
    List<SwarmAgent>? agents,
    List<SwarmTask>? tasks,
    bool? isLoading,
    String? error,
  }) {
    return SwarmState(
      agents: agents ?? this.agents,
      tasks: tasks ?? this.tasks,
      isLoading: isLoading ?? this.isLoading,
      error: error,
    );
  }
}

class SwarmNotifier extends StateNotifier<SwarmState> {
  final WebSocketService _ws;

  SwarmNotifier(this._ws) : super(const SwarmState());

  /// Load swarm agents from backend.
  Future<void> loadAgents() async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final response = await _ws.proxyCommand('swarm_agents', {});
      final data = response['data'] as Map<String, dynamic>? ?? response;
      final list = (data['agents'] as List?) ?? [];
      final agents =
          list.map((e) => SwarmAgent.fromJson(e as Map<String, dynamic>)).toList();
      state = state.copyWith(agents: agents, isLoading: false);
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  /// Load swarm tasks from backend.
  Future<void> loadTasks() async {
    state = state.copyWith(isLoading: true, error: null);
    try {
      final response = await _ws.proxyCommand('swarm_tasks', {});
      final data = response['data'] as Map<String, dynamic>? ?? response;
      final list = (data['tasks'] as List?) ?? [];
      final tasks =
          list.map((e) => SwarmTask.fromJson(e as Map<String, dynamic>)).toList();
      state = state.copyWith(tasks: tasks, isLoading: false);
    } catch (e) {
      state = state.copyWith(isLoading: false, error: e.toString());
    }
  }

  /// Load both agents and tasks.
  Future<void> loadAll() async {
    state = state.copyWith(isLoading: true, error: null);
    await Future.wait([loadAgents(), loadTasks()]);
    state = state.copyWith(isLoading: false);
  }

  /// Create a new swarm task.
  Future<bool> createTask({
    required String name,
    required String description,
    String? assignToAgent,
  }) async {
    try {
      await _ws.proxyCommand('swarm_create_task', {
        'name': name,
        'description': description,
        if (assignToAgent != null) 'assigned_agent': assignToAgent,
      });
      // Reload tasks after creation
      await loadTasks();
      return true;
    } catch (_) {
      return false;
    }
  }
}

final swarmProvider =
    StateNotifierProvider<SwarmNotifier, SwarmState>((ref) {
  final ws = ref.watch(webSocketServiceProvider);
  final notifier = SwarmNotifier(ws);
  Future.microtask(() => notifier.loadAll());
  return notifier;
});

// ---------------------------------------------------------------------------
// Screen
// ---------------------------------------------------------------------------

class SwarmDashboardScreen extends ConsumerStatefulWidget {
  const SwarmDashboardScreen({super.key});

  @override
  ConsumerState<SwarmDashboardScreen> createState() =>
      _SwarmDashboardScreenState();
}

class _SwarmDashboardScreenState extends ConsumerState<SwarmDashboardScreen>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  // ---------- helpers ----------

  Color _statusColor(String status) {
    switch (status) {
      case 'idle':
        return Colors.blueGrey;
      case 'busy':
      case 'running':
        return Colors.orange;
      case 'completed':
        return Colors.green;
      case 'failed':
      case 'error':
        return Colors.red;
      case 'offline':
        return Colors.grey;
      case 'pending':
        return Colors.blue;
      default:
        return Colors.grey;
    }
  }

  IconData _statusIcon(String status) {
    switch (status) {
      case 'idle':
        return Icons.pause_circle_outline_rounded;
      case 'busy':
      case 'running':
        return Icons.play_circle_outline_rounded;
      case 'completed':
        return Icons.check_circle_outline_rounded;
      case 'failed':
      case 'error':
        return Icons.error_outline_rounded;
      case 'offline':
        return Icons.remove_circle_outline_rounded;
      case 'pending':
        return Icons.hourglass_empty_rounded;
      default:
        return Icons.help_outline_rounded;
    }
  }

  Color _roleColor(String role, ThemeData theme) {
    switch (role.toLowerCase()) {
      case 'planner':
        return Colors.indigo;
      case 'architect':
        return Colors.deepPurple;
      case 'developer':
      case 'coder':
        return Colors.teal;
      case 'reviewer':
        return Colors.orange;
      case 'tester':
        return Colors.green;
      case 'coordinator':
      case 'leader':
        return theme.colorScheme.primary;
      default:
        return theme.colorScheme.secondary;
    }
  }

  IconData _roleIcon(String role) {
    switch (role.toLowerCase()) {
      case 'planner':
        return Icons.format_list_bulleted_rounded;
      case 'architect':
        return Icons.account_tree_rounded;
      case 'developer':
      case 'coder':
        return Icons.code_rounded;
      case 'reviewer':
        return Icons.rate_review_rounded;
      case 'tester':
        return Icons.bug_report_rounded;
      case 'coordinator':
      case 'leader':
        return Icons.hub_rounded;
      default:
        return Icons.smart_toy_rounded;
    }
  }

  // ---------- create task dialog ----------

  void _showCreateTaskDialog() {
    final nameController = TextEditingController();
    final descController = TextEditingController();
    String? selectedAgent;
    final agents = ref.read(swarmProvider).agents;

    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (ctx, setDialogState) => AlertDialog(
          title: Row(
            children: [
              Icon(Icons.add_task_rounded, color: Theme.of(context).colorScheme.primary),
              const SizedBox(width: 8),
              const Text('Create Swarm Task'),
            ],
          ),
          content: SizedBox(
            width: 400,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: nameController,
                  decoration: InputDecoration(
                    labelText: 'Task Name',
                    hintText: 'e.g. Refactor auth module',
                    prefixIcon: const Icon(Icons.label_outline_rounded),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: descController,
                  maxLines: 3,
                  decoration: InputDecoration(
                    labelText: 'Description',
                    hintText: 'Describe the task...',
                    prefixIcon: const Icon(Icons.description_outlined),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                DropdownButtonFormField<String>(
                  initialValue: selectedAgent,
                  decoration: InputDecoration(
                    labelText: 'Assign to Agent (optional)',
                    prefixIcon: const Icon(Icons.person_outline_rounded),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                  items: [
                    const DropdownMenuItem(value: null, child: Text('Auto-assign')),
                    ...agents.map((a) => DropdownMenuItem(
                          value: a.id,
                          child: Text('${a.name} (${a.role})'),
                        )),
                  ],
                  onChanged: (v) => setDialogState(() => selectedAgent = v),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel'),
            ),
            ElevatedButton.icon(
              icon: const Icon(Icons.send_rounded),
              label: const Text('Create'),
              onPressed: () async {
                final name = nameController.text.trim();
                if (name.isEmpty) return;
                final ok =
                    await ref.read(swarmProvider.notifier).createTask(
                          name: name,
                          description: descController.text.trim(),
                          assignToAgent: selectedAgent,
                        );
                if (ctx.mounted) Navigator.of(ctx).pop();
                if (mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text(ok ? 'Task created' : 'Failed to create task'),
                      backgroundColor: ok ? null : Theme.of(context).colorScheme.error,
                    ),
                  );
                }
              },
            ),
          ],
        ),
      ),
    );
  }

  // ---------- build ----------

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(swarmProvider);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Swarm Dashboard'),
        bottom: TabBar(
          controller: _tabController,
          tabs: [
            Tab(
              icon: const Icon(Icons.people_outline_rounded),
              text: 'Agents (${state.agents.length})',
            ),
            Tab(
              icon: const Icon(Icons.task_alt_rounded),
              text: 'Tasks (${state.tasks.length})',
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh_rounded),
            tooltip: 'Refresh',
            onPressed: () => ref.read(swarmProvider.notifier).loadAll(),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _showCreateTaskDialog,
        icon: const Icon(Icons.add_rounded),
        label: const Text('New Task'),
      ),
      body: state.isLoading && state.agents.isEmpty && state.tasks.isEmpty
          ? const Center(child: CircularProgressIndicator())
          : state.error != null && state.agents.isEmpty && state.tasks.isEmpty
              ? _buildGlobalError(state.error!, theme)
              : TabBarView(
                  controller: _tabController,
                  children: [
                    _buildAgentsTab(state, theme),
                    _buildTasksTab(state, theme),
                  ],
                ),
    );
  }

  Widget _buildGlobalError(String error, ThemeData theme) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.cloud_off_rounded, size: 56, color: theme.colorScheme.error),
            const SizedBox(height: 16),
            Text('Unable to load swarm data', style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            Text(error,
                textAlign: TextAlign.center,
                style: theme.textTheme.bodySmall
                    ?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              icon: const Icon(Icons.refresh_rounded),
              label: const Text('Retry'),
              onPressed: () => ref.read(swarmProvider.notifier).loadAll(),
            ),
          ],
        ),
      ),
    );
  }

  // ---------- Agents tab ----------

  Widget _buildAgentsTab(SwarmState state, ThemeData theme) {
    final agents = state.agents;

    if (agents.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.people_outline_rounded, size: 56, color: theme.colorScheme.outline),
            const SizedBox(height: 12),
            Text('No agents in swarm', style: theme.textTheme.bodyLarge),
            const SizedBox(height: 8),
            Text(
              'Agents will appear here once the swarm is initialized',
              style: theme.textTheme.bodySmall
                  ?.copyWith(color: theme.colorScheme.onSurfaceVariant),
            ),
          ],
        ),
      );
    }

    final screenWidth = MediaQuery.of(context).size.width;
    final isDesktop = screenWidth > 900;

    if (isDesktop) {
      return GridView.builder(
        padding: const EdgeInsets.all(16),
        gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
          maxCrossAxisExtent: 400,
          mainAxisExtent: 160,
          crossAxisSpacing: 12,
          mainAxisSpacing: 12,
        ),
        itemCount: agents.length,
        itemBuilder: (ctx, i) => _buildAgentCard(agents[i], theme),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: agents.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (ctx, i) => _buildAgentCard(agents[i], theme),
    );
  }

  Widget _buildAgentCard(SwarmAgent agent, ThemeData theme) {
    final statusColor = _statusColor(agent.status);
    final roleColor = _roleColor(agent.role, theme);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                // Role icon
                Container(
                  width: 44,
                  height: 44,
                  decoration: BoxDecoration(
                    color: roleColor.withValues(alpha: 0.12),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Icon(_roleIcon(agent.role), color: roleColor, size: 22),
                ),
                const SizedBox(width: 12),
                // Name + role badge
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        agent.name,
                        style: theme.textTheme.titleSmall
                            ?.copyWith(fontWeight: FontWeight.w600),
                      ),
                      const SizedBox(height: 4),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                        decoration: BoxDecoration(
                          color: roleColor.withValues(alpha: 0.12),
                          borderRadius: BorderRadius.circular(6),
                        ),
                        child: Text(
                          agent.role,
                          style: TextStyle(
                            fontSize: 11,
                            fontWeight: FontWeight.w500,
                            color: roleColor,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                // Status indicator
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: statusColor.withValues(alpha: 0.12),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(_statusIcon(agent.status), color: statusColor, size: 16),
                      const SizedBox(width: 4),
                      Text(
                        agent.status,
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w500,
                          color: statusColor,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Stats row
            Row(
              children: [
                Icon(Icons.task_alt_rounded, size: 14, color: theme.colorScheme.onSurfaceVariant),
                const SizedBox(width: 4),
                Text(
                  '${agent.tasksCompleted} tasks completed',
                  style: theme.textTheme.bodySmall
                      ?.copyWith(color: theme.colorScheme.onSurfaceVariant),
                ),
                const Spacer(),
                if (agent.currentTask != null) ...[
                  Icon(Icons.play_arrow_rounded, size: 14, color: Colors.orange),
                  const SizedBox(width: 4),
                  Flexible(
                    child: Text(
                      agent.currentTask!,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: Colors.orange.shade700,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ],
            ),
          ],
        ),
      ),
    );
  }

  // ---------- Tasks tab ----------

  Widget _buildTasksTab(SwarmState state, ThemeData theme) {
    final tasks = state.tasks;

    if (tasks.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.task_alt_rounded, size: 56, color: theme.colorScheme.outline),
            const SizedBox(height: 12),
            Text('No tasks yet', style: theme.textTheme.bodyLarge),
            const SizedBox(height: 8),
            Text(
              'Tap the button below to create a new swarm task',
              style: theme.textTheme.bodySmall
                  ?.copyWith(color: theme.colorScheme.onSurfaceVariant),
            ),
          ],
        ),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 80),
      itemCount: tasks.length,
      separatorBuilder: (_, __) => const SizedBox(height: 8),
      itemBuilder: (ctx, i) => _buildTaskCard(tasks[i], theme),
    );
  }

  Widget _buildTaskCard(SwarmTask task, ThemeData theme) {
    final statusColor = _statusColor(task.status);
    final isRunning = task.status == 'running';

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                // Status icon
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: statusColor.withValues(alpha: 0.12),
                    shape: BoxShape.circle,
                  ),
                  child: Icon(_statusIcon(task.status), color: statusColor, size: 20),
                ),
                const SizedBox(width: 12),
                // Name
                Expanded(
                  child: Text(
                    task.name,
                    style: theme.textTheme.titleSmall
                        ?.copyWith(fontWeight: FontWeight.w600),
                  ),
                ),
                // Status badge
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: statusColor.withValues(alpha: 0.12),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Text(
                    task.status,
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                      color: statusColor,
                    ),
                  ),
                ),
              ],
            ),
            // Progress bar for running tasks
            if (isRunning || task.progress > 0) ...[
              const SizedBox(height: 12),
              ClipRRect(
                borderRadius: BorderRadius.circular(4),
                child: LinearProgressIndicator(
                  value: task.progress > 0 ? task.progress / 100 : null,
                  minHeight: 6,
                  backgroundColor: theme.colorScheme.surfaceContainerHighest,
                  valueColor: AlwaysStoppedAnimation(statusColor),
                ),
              ),
              if (task.progress > 0) ...[
                const SizedBox(height: 4),
                Align(
                  alignment: Alignment.centerRight,
                  child: Text(
                    '${task.progress.toStringAsFixed(0)}%',
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                      fontSize: 11,
                    ),
                  ),
                ),
              ],
            ],
            // Footer: assigned agent + created date
            const SizedBox(height: 8),
            Row(
              children: [
                if (task.assignedAgent != null) ...[
                  Icon(Icons.person_outline_rounded,
                      size: 14, color: theme.colorScheme.onSurfaceVariant),
                  const SizedBox(width: 4),
                  Text(
                    task.assignedAgent!,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                  ),
                ],
                const Spacer(),
                if (task.createdAt != null)
                  Text(
                    _formatDate(task.createdAt!),
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurfaceVariant,
                      fontSize: 11,
                    ),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  String _formatDate(DateTime dt) {
    final now = DateTime.now();
    final diff = now.difference(dt);
    if (diff.inMinutes < 1) return 'just now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    return '${diff.inDays}d ago';
  }
}
