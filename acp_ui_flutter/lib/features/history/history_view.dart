import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../data/stores/session_store.dart';
import '../../data/models/session.dart';

/// History view - task history and session records
class HistoryView extends ConsumerWidget {
  const HistoryView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final sessions = ref.watch(activeSessionsProvider);
    final searchQuery = ref.watch(historySearchProvider);

    return Scaffold(
      body: Column(
        children: [
          Container(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Text(
                  'History',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                const Spacer(),
                // Search field
                Expanded(
                  child: TextField(
                    decoration: InputDecoration(
                      hintText: 'Search sessions...',
                      prefixIcon: const Icon(Icons.search),
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(8),
                      ),
                    ),
                    onChanged: (value) {
                      ref.read(historySearchProvider.notifier).state = value;
                    },
                  ),
                ),
                const SizedBox(width: 16),
                // Filter dropdown
                IconButton(
                  icon: const Icon(Icons.filter_list),
                  onPressed: () => _showFilterDialog(context, ref),
                ),
              ],
            ),
          ),
          const Divider(),
          Expanded(
            child: _HistoryList(sessions: sessions),
          ),
        ],
      ),
    );
  }

  void _showFilterDialog(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Filter Sessions'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              title: const Text('All'),
              onTap: () {
                ref.read(historyFilterProvider.notifier).state = SessionFilter.all;
                Navigator.pop(context);
              },
            ),
            ListTile(
              title: const Text('Active'),
              onTap: () {
                ref.read(historyFilterProvider.notifier).state = SessionFilter.active;
                Navigator.pop(context);
              },
            ),
            ListTile(
              title: const Text('Completed'),
              onTap: () {
                ref.read(historyFilterProvider.notifier).state = SessionFilter.completed;
                Navigator.pop(context);
              },
            ),
          ],
        ),
      ),
    );
  }
}

/// History search provider
final historySearchProvider = StateProvider<String>((ref) => '');

/// History filter provider
final historyFilterProvider = StateProvider<SessionFilter>((ref) => SessionFilter.all);

enum SessionFilter {
  all,
  active,
  completed,
}

class _HistoryList extends StatelessWidget {
  const _HistoryList({required this.sessions});

  final List<Session> sessions;

  @override
  Widget build(BuildContext context) {
    if (sessions.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.history, size: 64, color: Colors.grey),
            const SizedBox(height: 16),
            Text(
              'No session history',
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: sessions.length,
      itemBuilder: (context, index) {
        final session = sessions[index];
        return _SessionCard(session: session);
      },
    );
  }
}

class _SessionCard extends StatelessWidget {
  const _SessionCard({required this.session});

  final Session session;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: InkWell(
        onTap: () => _showSessionDetail(context),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(_getStatusIcon(session.status), size: 20),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Session ${session.id}',
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                  ),
                  _StatusBadge(status: session.status),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                'Agent: ${session.agentId}',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              Text(
                'Messages: ${session.messages.length}',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              Text(
                'Created: ${_formatDate(session.createdAt)}',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _showSessionDetail(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Session ${session.id}'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Agent: ${session.agentId}'),
            Text('Status: ${session.status.name}'),
            Text('Messages: ${session.messages.length}'),
            if (session.branchLock != null)
              Text('Branch: ${session.branchLock}'),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  IconData _getStatusIcon(SessionStatus status) {
    switch (status) {
      case SessionStatus.active:
        return Icons.play_arrow;
      case SessionStatus.paused:
        return Icons.pause;
      case SessionStatus.completed:
        return Icons.check_circle;
      case SessionStatus.compacted:
        return Icons.compress;
    }
  }

  String _formatDate(DateTime date) {
    return '${date.year}-${date.month.toString().padLeft(2, '0')}-${date.day.toString().padLeft(2, '0')}';
  }
}

class _StatusBadge extends StatelessWidget {
  const _StatusBadge({required this.status});

  final SessionStatus status;

  @override
  Widget build(BuildContext context) {
    Color color;
    switch (status) {
      case SessionStatus.active:
        color = Colors.blue;
      case SessionStatus.completed:
        color = Colors.green;
      case SessionStatus.paused:
        color = Colors.orange;
      case SessionStatus.compacted:
        color = Colors.purple;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.2),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        status.name,
        style: TextStyle(color: color, fontSize: 12),
      ),
    );
  }
}