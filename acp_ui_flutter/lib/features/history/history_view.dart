import 'package:flutter/material.dart';

/// History view - task history and session records
class HistoryView extends StatelessWidget {
  const HistoryView({super.key});

  @override
  Widget build(BuildContext context) {
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
                IconButton(
                  icon: const Icon(Icons.search),
                  onPressed: () {
                    // TODO: Search history
                  },
                ),
              ],
            ),
          ),
          const Divider(),
          Expanded(
            child: _HistoryList(),
          ),
        ],
      ),
    );
  }
}

class _HistoryList extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final sessions = [
      {'title': 'Feature: User authentication', 'date': '2026-05-14', 'status': 'completed'},
      {'title': 'Bug fix: Login timeout', 'date': '2026-05-13', 'status': 'completed'},
      {'title': 'Refactor: Database layer', 'date': '2026-05-12', 'status': 'paused'},
    ];

    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: sessions.length,
      itemBuilder: (context, index) {
        final session = sessions[index];
        return Card(
          child: ListTile(
            leading: const Icon(Icons.history),
            title: Text(session['title'] as String),
            subtitle: Text(session['date'] as String),
            trailing: _StatusBadge(status: session['status'] as String),
            onTap: () {
              // TODO: Open session detail
            },
          ),
        );
      },
    );
  }
}

class _StatusBadge extends StatelessWidget {
  const _StatusBadge({required this.status});

  final String status;

  @override
  Widget build(BuildContext context) {
    Color color;
    switch (status) {
      case 'completed':
        color = Colors.green;
        break;
      case 'paused':
        color = Colors.orange;
        break;
      case 'active':
        color = Colors.blue;
        break;
      default:
        color = Colors.grey;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.2),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        status,
        style: TextStyle(color: color, fontSize: 12),
      ),
    );
  }
}