import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../widgets/approval_card.dart';

/// Approval state for demo purposes
class ApprovalState {
  final List<ApprovalRequest> pending;
  final List<ApprovalRequest> processed;

  const ApprovalState({
    this.pending = const [],
    this.processed = const [],
  });

  ApprovalState copyWith({
    List<ApprovalRequest>? pending,
    List<ApprovalRequest>? processed,
  }) {
    return ApprovalState(
      pending: pending ?? this.pending,
      processed: processed ?? this.processed,
    );
  }
}

final _approvalStateProvider =
    StateNotifierProvider<ApprovalNotifier, ApprovalState>((ref) {
  return ApprovalNotifier();
});

class ApprovalNotifier extends StateNotifier<ApprovalState> {
  ApprovalNotifier() : super(_initialState()) {
    // Seed demo data
    state = state.copyWith(
      pending: <ApprovalRequest>[
        ApprovalRequest(
          id: 'req-1',
          title: 'Deploy agent-team-alpha to staging',
          description: 'Deploy the latest agent team configuration to the staging environment for validation. Includes 3 new agents and updated collaboration rules.',
          deadline: DateTime.now().add(const Duration(hours: 4)),
          requestor: 'Agent Orchestrator',
          options: ['Approve', 'Reject'],
        ),
        ApprovalRequest(
          id: 'req-2',
          title: 'Merge feature/refactor-auth',
          description: 'Pull request contains authentication module refactor with breaking API changes. Requires manual review before merge.',
          deadline: DateTime.now().add(const Duration(hours: 2)),
          requestor: 'Code Reviewer',
          options: ['Approve', 'Reject'],
        ),
        ApprovalRequest(
          id: 'req-3',
          title: 'Increase memory limit for planner agent',
          description: 'Planner agent hitting context limits. Request to increase maxContextTokens from 200000 to 400000.',
          deadline: DateTime.now().subtract(const Duration(hours: 1)),
          requestor: 'Self-Healing System',
          options: ['Approve', 'Reject'],
        ),
      ],
    );
    ApprovalStateNotifier.updateCount(state.pending.length);
  }

  static ApprovalState _initialState() => const ApprovalState();

  void handleAction(ApprovalRequest request, ApprovalAction action) {
    final updatedPending = state.pending.where((r) => r.id != request.id).toList();
    final processed = [
      ...state.processed,
      request.copyWith(expanded: false),
    ];
    state = state.copyWith(
      pending: updatedPending,
      processed: processed,
    );
    ApprovalStateNotifier.updateCount(updatedPending.length);
  }
}

/// Global accessor for main screen badge
class ApprovalStateNotifier {
  static int get pendingApprovals => _currentCount;
  static int _currentCount = 0;
  static final List<VoidCallback> _listeners = [];

  static void addListener(VoidCallback cb) => _listeners.add(cb);
  static void removeListener(VoidCallback cb) => _listeners.remove(cb);
  static void notify() {
    for (final cb in _listeners) cb();
  }

  static void updateCount(int count) {
    _currentCount = count;
    notify();
  }
}

/// Approval workflow — the "做审批" (Make Approval) screen.
/// Lists pending approvals as cards with quick actions.
class ApprovalScreen extends ConsumerWidget {
  const ApprovalScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final approvalState = ref.watch(_approvalStateProvider);

    return Scaffold(
      body: CustomScrollView(
        slivers: [
          _buildAppBar(context, approvalState.pending.length),
          if (approvalState.pending.isEmpty && approvalState.processed.isEmpty)
            SliverFillRemaining(
              hasScrollBody: false,
              child: Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      Icons.checklist_rtl_outlined,
                      size: 64,
                      color: Theme.of(context).colorScheme.outlineVariant,
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'All caught up',
                      style: Theme.of(context).textTheme.titleMedium?.copyWith(
                            color: Theme.of(context).colorScheme.outline,
                          ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'No pending approvals',
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Theme.of(context).colorScheme.outlineVariant,
                          ),
                    ),
                  ],
                ),
              ),
            )
          else ...[
            if (approvalState.pending.isNotEmpty) ...[
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                  child: Text(
                    'Pending (${approvalState.pending.length})',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.w700,
                        ),
                  ),
                ),
              ),
              SliverPadding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                sliver: SliverList(
                  delegate: SliverChildBuilderDelegate(
                    (context, index) {
                      final request = approvalState.pending[index];
                      return ApprovalCard(
                        key: Key('pending-${request.id}'),
                        request: request,
                        onAction: (req, action) =>
                            ref.read(_approvalStateProvider.notifier).handleAction(req, action),
                      );
                    },
                    childCount: approvalState.pending.length,
                  ),
                ),
              ),
            ],
            if (approvalState.processed.isNotEmpty) ...[
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.fromLTRB(16, 24, 16, 8),
                  child: Text(
                    'Processed',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.w700,
                          color: Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                  ),
                ),
              ),
              SliverOpacity(
                opacity: 0.6,
                sliver: SliverPadding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  sliver: SliverList(
                    delegate: SliverChildBuilderDelegate(
                      (context, index) {
                        final request = approvalState.processed[index];
                        return Card(
                          margin: const EdgeInsets.only(bottom: 8),
                          child: ListTile(
                            leading: const Icon(Icons.check_circle_outline, size: 20),
                            title: Text(
                              request.title,
                              style: const TextStyle(decoration: TextDecoration.lineThrough),
                            ),
                            subtitle: Text('Processed — ${request.requestor}'),
                            trailing: const Icon(Icons.check, size: 18),
                          ),
                        );
                      },
                      childCount: approvalState.processed.length,
                    ),
                  ),
                ),
              ),
            ],
          ],
          const SliverToBoxAdapter(child: SizedBox(height: 24)),
        ],
      ),
    );
  }

  Widget _buildAppBar(BuildContext context, int pendingCount) {
    return SliverAppBar(
      floating: true,
      snap: true,
      title: const Text('Approvals'),
      actions: [
        if (pendingCount > 0)
          Padding(
            padding: const EdgeInsets.only(right: 16),
            child: Badge(
              label: Text('$pendingCount'),
              child: Icon(
                Icons.notifications_none,
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ),
      ],
    );
  }
}
