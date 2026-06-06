import 'package:flutter/material.dart';

/// Approval action result
enum ApprovalAction { approve, reject, feedback }

/// Approval request model
class ApprovalRequest {
  final String id;
  final String title;
  final String description;
  final DateTime deadline;
  final String requestor;
  final List<String> options;
  final bool expanded;

  const ApprovalRequest({
    required this.id,
    required this.title,
    required this.description,
    required this.deadline,
    this.requestor = 'System',
    this.options = const ['Approve', 'Reject'],
    this.expanded = false,
  });

  ApprovalRequest copyWith({
    String? id,
    String? title,
    String? description,
    DateTime? deadline,
    String? requestor,
    List<String>? options,
    bool? expanded,
  }) {
    return ApprovalRequest(
      id: id ?? this.id,
      title: title ?? this.title,
      description: description ?? this.description,
      deadline: deadline ?? this.deadline,
      requestor: requestor ?? this.requestor,
      options: options ?? this.options,
      expanded: expanded ?? this.expanded,
    );
  }

  bool get isOverdue => DateTime.now().isAfter(deadline);
  String get deadlineLabel {
    final now = DateTime.now();
    final diff = deadline.difference(now);
    if (diff.isNegative) return 'Overdue';
    if (diff.inHours < 1) return '${diff.inMinutes}m left';
    if (diff.inHours < 24) return '${diff.inHours}h left';
    return '${diff.inDays}d left';
  }
}

/// Reusable approval card with expand/collapse, swipe actions, and quick response.
class ApprovalCard extends StatefulWidget {
  final ApprovalRequest request;
  final Function(ApprovalRequest, ApprovalAction) onAction;

  const ApprovalCard({
    super.key,
    required this.request,
    required this.onAction,
  });

  @override
  State<ApprovalCard> createState() => _ApprovalCardState();
}

class _ApprovalCardState extends State<ApprovalCard>
    with SingleTickerProviderStateMixin {
  late bool _expanded;
  final TextEditingController _feedbackController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _expanded = widget.request.expanded;
  }

  @override
  void dispose() {
    _feedbackController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final isOverdue = widget.request.isOverdue;

    return Dismissible(
      key: Key(widget.request.id),
      direction: DismissDirection.horizontal,
      confirmDismiss: (direction) async {
        if (direction == DismissDirection.startToEnd) {
          widget.onAction(widget.request, ApprovalAction.approve);
          return false;
        } else {
          widget.onAction(widget.request, ApprovalAction.reject);
          return false;
        }
      },
      background: _dismissBackground(colorScheme.primary, Icons.check, Alignment.centerLeft),
      secondaryBackground: _dismissBackground(colorScheme.error, Icons.close, Alignment.centerRight),
      child: Card(
        margin: const EdgeInsets.only(bottom: 8),
        child: Column(
          children: [
            ListTile(
              onTap: () => setState(() => _expanded = !_expanded),
              leading: CircleAvatar(
                backgroundColor: isOverdue
                    ? colorScheme.errorContainer
                    : colorScheme.primaryContainer,
                child: Icon(
                  isOverdue ? Icons.warning_amber : Icons.pending_actions,
                  color: isOverdue
                      ? colorScheme.onErrorContainer
                      : colorScheme.onPrimaryContainer,
                  size: 20,
                ),
              ),
              title: Text(
                widget.request.title,
                style: theme.textTheme.titleSmall?.copyWith(
                  fontWeight: FontWeight.w600,
                  color: isOverdue ? colorScheme.error : null,
                ),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              subtitle: Row(
                children: [
                  Icon(Icons.person_outline, size: 12, color: colorScheme.onSurfaceVariant),
                  const SizedBox(width: 4),
                  Text(widget.request.requestor, style: theme.textTheme.bodySmall),
                  const SizedBox(width: 8),
                  Icon(Icons.access_time, size: 12, color: isOverdue ? colorScheme.error : colorScheme.onSurfaceVariant),
                  const SizedBox(width: 4),
                  Text(
                    widget.request.deadlineLabel,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: isOverdue ? colorScheme.error : colorScheme.onSurfaceVariant,
                      fontWeight: isOverdue ? FontWeight.w600 : null,
                    ),
                  ),
                ],
              ),
              trailing: Icon(
                _expanded ? Icons.expand_less : Icons.expand_more,
                color: colorScheme.onSurfaceVariant,
              ),
            ),
            if (_expanded)
              AnimatedSwitcher(
                duration: const Duration(milliseconds: 200),
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        widget.request.description,
                        style: theme.textTheme.bodyMedium,
                      ),
                      const SizedBox(height: 16),
                      // Action buttons
                      Row(
                        children: [
                          Expanded(
                            child: OutlinedButton.icon(
                              onPressed: () =>
                                  widget.onAction(widget.request, ApprovalAction.reject),
                              icon: const Icon(Icons.close, size: 16),
                              label: const Text('Reject'),
                              style: OutlinedButton.styleFrom(
                                foregroundColor: colorScheme.error,
                                side: BorderSide(color: colorScheme.error),
                              ),
                            ),
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: FilledButton.icon(
                              onPressed: () =>
                                  widget.onAction(widget.request, ApprovalAction.approve),
                              icon: const Icon(Icons.check, size: 16),
                              label: const Text('Approve'),
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      // Feedback input
                      TextField(
                        controller: _feedbackController,
                        decoration: InputDecoration(
                          hintText: 'Add feedback (optional)',
                          prefixIcon: const Icon(Icons.chat_bubble_outline, size: 20),
                          suffixIcon: TextButton(
                            onPressed: _feedbackController.text.trim().isNotEmpty
                                ? () {
                                    widget.onAction(
                                      widget.request,
                                      ApprovalAction.feedback,
                                    );
                                  }
                                : null,
                            child: const Text('Send'),
                          ),
                        ),
                        maxLines: 2,
                        textInputAction: TextInputAction.send,
                        onSubmitted: (_) {
                          if (_feedbackController.text.trim().isNotEmpty) {
                            widget.onAction(
                              widget.request,
                              ApprovalAction.feedback,
                            );
                          }
                        },
                      ),
                    ],
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _dismissBackground(Color color, IconData icon, Alignment alignment) {
    return Container(
      color: color,
      alignment: alignment,
      padding: const EdgeInsets.symmetric(horizontal: 24),
      child: Icon(icon, color: Colors.white, size: 28),
    );
  }
}
