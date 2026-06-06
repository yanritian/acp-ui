import 'package:flutter/material.dart';
import '../../core/orchestrator/orchestrator.dart';

/// Reusable task card for the progress dashboard.
/// Shows task description, status, progress, assigned agents, and timestamps.
class TaskCard extends StatelessWidget {
  final Task task;
  final double? progressOverride;
  final VoidCallback? onTap;

  const TaskCard({
    super.key,
    required this.task,
    this.progressOverride,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final statusConfig = _statusConfig(task.status, colorScheme);

    final progress = progressOverride ?? _defaultProgress(task.status);

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Header: status badge + description
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _StatusBadge(
                    label: statusConfig.label,
                    color: statusConfig.color,
                    icon: statusConfig.icon,
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          task.description,
                          style: theme.textTheme.titleSmall?.copyWith(
                            fontWeight: FontWeight.w600,
                          ),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                        const SizedBox(height: 4),
                        Text(
                          _formatTime(task.createdAt),
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: colorScheme.onSurfaceVariant,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              // Progress bar
              LinearProgressIndicator(
                value: progress,
                minHeight: 6,
                backgroundColor: colorScheme.surfaceContainerHighest,
                valueColor: AlwaysStoppedAnimation<Color>(statusConfig.color),
                borderRadius: BorderRadius.circular(3),
              ),
              const SizedBox(height: 8),
              // Footer: agent count + progress %
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  if (task.assignedAgents.isNotEmpty)
                    Row(
                      children: [
                        Icon(
                          Icons.smart_toy_outlined,
                          size: 14,
                          color: colorScheme.onSurfaceVariant,
                        ),
                        const SizedBox(width: 4),
                        Text(
                          '${task.assignedAgents.length} agent${task.assignedAgents.length > 1 ? 's' : ''}',
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: colorScheme.onSurfaceVariant,
                          ),
                        ),
                      ],
                    )
                  else
                    const SizedBox.shrink(),
                  Text(
                    '${(progress * 100).toInt()}%',
                    style: theme.textTheme.bodySmall?.copyWith(
                      fontWeight: FontWeight.w600,
                      color: statusConfig.color,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  double _defaultProgress(TaskStatus status) {
    switch (status) {
      case TaskStatus.pending:
        return 0.0;
      case TaskStatus.running:
        return 0.45;
      case TaskStatus.completed:
        return 1.0;
      case TaskStatus.failed:
        return 1.0;
      case TaskStatus.paused:
        return 0.3;
    }
  }

  String _formatTime(DateTime time) {
    final now = DateTime.now();
    final diff = now.difference(time);
    if (diff.inMinutes < 1) return 'Just now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    return '${diff.inDays}d ago';
  }

  _StatusConfig _statusConfig(TaskStatus status, ColorScheme colors) {
    switch (status) {
      case TaskStatus.pending:
        return _StatusConfig(
          label: 'Pending',
          color: colors.secondary,
          icon: Icons.schedule,
        );
      case TaskStatus.running:
        return _StatusConfig(
          label: 'Running',
          color: colors.primary,
          icon: Icons.play_arrow,
        );
      case TaskStatus.completed:
        return _StatusConfig(
          label: 'Done',
          color: colors.tertiary,
          icon: Icons.check_circle,
        );
      case TaskStatus.failed:
        return _StatusConfig(
          label: 'Failed',
          color: colors.error,
          icon: Icons.error_outline,
        );
      case TaskStatus.paused:
        return _StatusConfig(
          label: 'Paused',
          color: colors.outline,
          icon: Icons.pause_circle,
        );
    }
  }
}

class _StatusConfig {
  final String label;
  final Color color;
  final IconData icon;

  const _StatusConfig({
    required this.label,
    required this.color,
    required this.icon,
  });
}

class _StatusBadge extends StatelessWidget {
  final String label;
  final Color color;
  final IconData icon;

  const _StatusBadge({
    required this.label,
    required this.color,
    required this.icon,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: color.withOpacity(0.12),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: color),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: 11,
              fontWeight: FontWeight.w600,
              color: color,
            ),
          ),
        ],
      ),
    );
  }
}
