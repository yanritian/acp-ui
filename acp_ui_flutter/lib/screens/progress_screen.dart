import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/orchestrator/orchestrator.dart';
import '../../data/stores/orchestrator_store.dart';
import '../../widgets/task_card.dart';

/// Progress dashboard — the "看进度" (View Progress) screen.
/// Shows active tasks, completed tasks, and a quick summary of pending approvals.
class ProgressScreen extends ConsumerStatefulWidget {
  const ProgressScreen({super.key});

  @override
  ConsumerState<ProgressScreen> createState() => _ProgressScreenState();
}

class _ProgressScreenState extends ConsumerState<ProgressScreen> {
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  void _onRefresh() {
    ref.invalidate(tasksProvider);
  }

  @override
  Widget build(BuildContext context) {
    final tasks = ref.watch(tasksProvider);
    final activeTasks = tasks.where((t) => t.status == TaskStatus.running).toList();
    final pendingTasks = tasks.where((t) => t.status == TaskStatus.pending).toList();
    final completedTasks = tasks.where((t) => t.status == TaskStatus.completed).toList();

    return Scaffold(
      body: RefreshIndicator(
        onRefresh: () async => _onRefresh(),
        child: CustomScrollView(
          controller: _scrollController,
          slivers: [
            _buildAppBar(context),
            SliverToBoxAdapter(
              child: _buildSummaryCounts(
                context,
                total: tasks.length,
                active: activeTasks.length,
                done: completedTasks.length,
                failed: tasks.where((t) => t.status == TaskStatus.failed).length,
              ),
            ),
            if (activeTasks.isNotEmpty) ...[
              SliverToBoxAdapter(
                child: _sectionHeader(context, 'Active Tasks', activeTasks.length),
              ),
              SliverList(
                delegate: SliverChildBuilderDelegate(
                  (context, index) => Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: TaskCard(task: activeTasks[index]),
                  ),
                  childCount: activeTasks.length,
                ),
              ),
            ],
            if (pendingTasks.isNotEmpty) ...[
              SliverToBoxAdapter(
                child: _sectionHeader(context, 'Pending', pendingTasks.length),
              ),
              SliverList(
                delegate: SliverChildBuilderDelegate(
                  (context, index) => Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: TaskCard(task: pendingTasks[index]),
                  ),
                  childCount: pendingTasks.length,
                ),
              ),
            ],
            if (completedTasks.isNotEmpty) ...[
              SliverToBoxAdapter(
                child: _sectionHeader(context, 'Completed', completedTasks.length),
              ),
              SliverList(
                delegate: SliverChildBuilderDelegate(
                  (context, index) => Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: TaskCard(task: completedTasks[index]),
                  ),
                  childCount: completedTasks.length.clamp(0, 5),
                ),
              ),
            ],
            if (tasks.isEmpty)
              SliverFillRemaining(
                hasScrollBody: false,
                child: Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(
                        Icons.inbox_outlined,
                        size: 64,
                        color: Theme.of(context).colorScheme.outlineVariant,
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'No tasks yet',
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(
                              color: Theme.of(context).colorScheme.outline,
                            ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        'Tasks will appear here when agents start working',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: Theme.of(context).colorScheme.outlineVariant,
                            ),
                      ),
                    ],
                  ),
                ),
              ),
            const SliverToBoxAdapter(child: SizedBox(height: 24)),
          ],
        ),
      ),
    );
  }

  Widget _buildAppBar(BuildContext context) {
    return SliverAppBar(
      floating: true,
      snap: true,
      title: const Text('Progress'),
      actions: [
        IconButton(
          icon: const Icon(Icons.refresh),
          onPressed: _onRefresh,
          tooltip: 'Refresh',
        ),
      ],
    );
  }

  Widget _buildSummaryCounts(
    BuildContext context, {
    required int total,
    required int active,
    required int done,
    required int failed,
  }) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
      child: Row(
        children: [
          _SummaryChip('Total', '$total', Icons.list_alt),
          const SizedBox(width: 8),
          _SummaryChip('Active', '$active', Icons.play_arrow,
              color: Theme.of(context).colorScheme.primary),
          const SizedBox(width: 8),
          _SummaryChip('Done', '$done', Icons.check_circle,
              color: Theme.of(context).colorScheme.tertiary),
          const SizedBox(width: 8),
          _SummaryChip('Failed', '$failed', Icons.error_outline,
              color: Theme.of(context).colorScheme.error),
        ],
      ),
    );
  }

  // Kept for backward compatibility if needed elsewhere
  Widget _buildSummary(BuildContext context, List<Task> tasks) {
    return _buildSummaryCounts(
      context,
      total: tasks.length,
      active: tasks.where((t) => t.status == TaskStatus.running).length,
      done: tasks.where((t) => t.status == TaskStatus.completed).length,
      failed: tasks.where((t) => t.status == TaskStatus.failed).length,
    );
  }

  Widget _sectionHeader(BuildContext context, String title, int count) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 20, 16, 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            title,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.w700,
                ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primaryContainer,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Text(
              '$count',
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: Theme.of(context).colorScheme.onPrimaryContainer,
                    fontWeight: FontWeight.w600,
                  ),
            ),
          ),
        ],
      ),
    );
  }
}

class _SummaryChip extends StatelessWidget {
  final String label;
  final String value;
  final IconData icon;
  final Color? color;

  const _SummaryChip(this.label, this.value, this.icon, {this.color});

  @override
  Widget build(BuildContext context) {
    final c = color ?? Theme.of(context).colorScheme.onSurfaceVariant;
    return Expanded(
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 8),
        decoration: BoxDecoration(
          color: c.withOpacity(0.08),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          children: [
            Icon(icon, size: 18, color: c),
            const SizedBox(height: 4),
            Text(
              value,
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.w700,
                    color: c,
                  ),
            ),
            Text(
              label,
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: c.withOpacity(0.7),
                  ),
            ),
          ],
        ),
      ),
    );
  }
}
