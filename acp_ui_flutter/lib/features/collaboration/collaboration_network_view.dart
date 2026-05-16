import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:acp_ui_flutter/data/stores/collaboration_store.dart';
import 'package:acp_ui_flutter/data/models/collaboration.dart';

/// Collaboration Network View - Main view for agent collaboration visualization
class CollaborationNetworkView extends ConsumerStatefulWidget {
  const CollaborationNetworkView({super.key});

  @override
  ConsumerState<CollaborationNetworkView> createState() =>
    _CollaborationNetworkViewState();
}

class _CollaborationNetworkViewState
  extends ConsumerState<CollaborationNetworkView> {
  String _selectedViewMode = 'network';

  @override
  void initState() {
    super.initState();
    // Initialize mock data on load
    Future.microtask(() {
      ref.read(collaborationProvider.notifier).initializeMockData();
    });
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(collaborationProvider);
    final stats = ref.watch(collaborationStatsProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Agent Collaboration'),
        actions: [
          // View mode selector
          PopupMenuButton<String>(
            initialValue: _selectedViewMode,
            onSelected: (mode) {
              setState(() => _selectedViewMode = mode);
              ref.read(collaborationProvider.notifier).setViewMode(mode);
            },
            itemBuilder: (context) => [
              const PopupMenuItem(value: 'network', child: Text('Network View')),
              const PopupMenuItem(value: 'timeline', child: Text('Timeline View')),
              const PopupMenuItem(value: 'kanban', child: Text('Kanban View')),
            ],
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () => _refreshData(),
          ),
        ],
      ),
      body: Column(
        children: [
          // Stats summary
          _buildStatsSummary(stats),

          // Main content
          Expanded(
            child: _selectedViewMode == 'network'
              ? _NetworkGraphView(nodes: state.nodes, edges: state.edges)
              : _selectedViewMode == 'timeline'
                ? _TimelineView(events: state.events)
                : _KanbanView(nodes: state.nodes, edges: state.edges),
          ),
        ],
      ),
    );
  }

  Widget _buildStatsSummary(CollaborationNetworkStats stats) {
    return Container(
      padding: const EdgeInsets.all(16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _StatBadge(
            icon: Icons.people,
            label: 'Agents',
            value: '${stats.activeAgents}/${stats.totalAgents}',
            color: Colors.blue,
          ),
          _StatBadge(
            icon: Icons.task,
            label: 'Tasks',
            value: '${stats.runningTasks}/${stats.totalTasks}',
            color: Colors.orange,
          ),
          _StatBadge(
            icon: Icons.check_circle,
            label: 'Completed',
            value: '${stats.completedTasks}',
            color: Colors.green,
          ),
          _StatBadge(
            icon: Icons.error,
            label: 'Failed',
            value: '${stats.failedTasks}',
            color: Colors.red,
          ),
          _StatBadge(
            icon: Icons.speed,
            label: 'Efficiency',
            value: '${stats.collaborationEfficiency}%',
            color: Colors.purple,
          ),
        ],
      ),
    );
  }

  void _refreshData() {
    // TODO: Implement data refresh
    ref.read(collaborationProvider.notifier).setLoading(true);
  }
}

/// Stat badge widget
class _StatBadge extends StatelessWidget {
  final IconData icon;
  final String label;
  final String value;
  final Color color;

  const _StatBadge({
    required this.icon,
    required this.label,
    required this.value,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 20, color: color),
          const SizedBox(height: 4),
          Text(
            label,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: color,
              fontWeight: FontWeight.bold,
            ),
          ),
          Text(
            value,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
              color: color,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }
}

/// Network graph view widget
class _NetworkGraphView extends StatelessWidget {
  final List<CollaborationNode> nodes;
  final List<CollaborationEdge> edges;

  const _NetworkGraphView({
    required this.nodes,
    required this.edges,
  });

  @override
  Widget build(BuildContext context) {
    if (nodes.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.device_hub, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text(
              'No collaboration network',
              style: TextStyle(fontSize: 18, color: Colors.grey),
            ),
            SizedBox(height: 8),
            Text(
              'Start agent collaboration to see the network',
              style: TextStyle(fontSize: 14, color: Colors.grey),
            ),
          ],
        ),
      );
    }

    return InteractiveViewer(
      minScale: 0.5,
      maxScale: 2.0,
      child: CustomPaint(
        size: Size.infinite,
        painter: _NetworkGraphPainter(nodes: nodes, edges: edges),
      ),
    );
  }
}

/// Custom painter for network graph
class _NetworkGraphPainter extends CustomPainter {
  final List<CollaborationNode> nodes;
  final List<CollaborationEdge> edges;

  _NetworkGraphPainter({
    required this.nodes,
    required this.edges,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // Draw edges first
    for (final edge in edges) {
      final sourceNode = nodes.where((n) => n.agentId == edge.sourceAgentId).firstOrNull;
      final targetNode = nodes.where((n) => n.agentId == edge.targetAgentId).firstOrNull;

      if (sourceNode != null && targetNode != null) {
        _drawEdge(canvas, edge, sourceNode, targetNode);
      }
    }

    // Draw nodes
    for (final node in nodes) {
      _drawNode(canvas, node, size);
    }
  }

  void _drawEdge(
    Canvas canvas,
    CollaborationEdge edge,
    CollaborationNode source,
    CollaborationNode target,
  ) {
    final paint = Paint()
      ..color = _getEdgeColor(edge.status)
      ..strokeWidth = 2.0
      ..style = PaintingStyle.stroke;

    // Draw curved path
    final path = Path();
    final startX = source.position.x;
    final startY = source.position.y;
    final endX = target.position.x;
    final endY = target.position.y;

    // Calculate control point for curved edge
    final midX = (startX + endX) / 2;
    final midY = (startY + endY) / 2;
    final controlOffset = 50.0;

    path.moveTo(startX, startY);
    path.quadraticBezierTo(
      midX + controlOffset,
      midY - controlOffset,
      endX,
      endY,
    );

    canvas.drawPath(path, paint);

    // Draw arrow at the end
    final arrowPaint = Paint()
      ..color = _getEdgeColor(edge.status)
      ..style = PaintingStyle.fill;

    final arrowSize = 10.0;
    final angle = atan2(endY - midY + controlOffset, endX - midX - controlOffset);

    final arrowPath = Path();
    arrowPath.moveTo(endX, endY);
    arrowPath.lineTo(
      endX - arrowSize * cos(angle - pi / 6),
      endY - arrowSize * sin(angle - pi / 6),
    );
    arrowPath.lineTo(
      endX - arrowSize * cos(angle + pi / 6),
      endY - arrowSize * sin(angle + pi / 6),
    );
    arrowPath.close();

    canvas.drawPath(arrowPath, arrowPaint);
  }

  void _drawNode(Canvas canvas, CollaborationNode node, Size size) {
    final center = Offset(node.position.x, node.position.y);
    final radius = 40.0;

    // Draw outer circle (status indicator)
    final outerPaint = Paint()
      ..color = _getNodeColor(node.status)
      ..style = PaintingStyle.fill;

    canvas.drawCircle(center, radius, outerPaint);

    // Draw inner circle
    final innerPaint = Paint()
      ..color = Colors.white
      ..style = PaintingStyle.fill;

    canvas.drawCircle(center, radius - 5, innerPaint);

    // Draw load indicator
    final loadPercentage = node.currentLoad / node.maxLoad;
    final loadPaint = Paint()
      ..color = _getLoadColor(loadPercentage)
      ..strokeWidth = 4.0
      ..style = PaintingStyle.stroke;

    final loadRect = Rect.fromCircle(center: center, radius: radius - 10);
    canvas.drawArc(
      loadRect,
      -pi / 2,
      2 * pi * loadPercentage,
      false,
      loadPaint,
    );

    // Draw node icon
    final icon = _getNodeIcon(node.agentType);
    final textPainter = TextPainter(
      text: TextSpan(
        text: icon,
        style: const TextStyle(fontSize: 24),
      ),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout();
    textPainter.paint(
      canvas,
      Offset(
        center.dx - textPainter.width / 2,
        center.dy - textPainter.height / 2,
      ),
    );

    // Draw agent name below
    final namePainter = TextPainter(
      text: TextSpan(
        text: node.agentName,
        style: const TextStyle(fontSize: 12, color: Colors.black87),
      ),
      textDirection: TextDirection.ltr,
    );
    namePainter.layout();
    namePainter.paint(
      canvas,
      Offset(
        center.dx - namePainter.width / 2,
        center.dy + radius + 5,
      ),
    );
  }

  Color _getEdgeColor(String status) {
    switch (status) {
      case 'pending':
        return Colors.orange;
      case 'flowing':
        return Colors.blue;
      case 'completed':
        return Colors.green;
      case 'failed':
        return Colors.red;
      default:
        return Colors.grey;
    }
  }

  Color _getNodeColor(String status) {
    switch (status) {
      case 'idle':
        return Colors.grey;
      case 'active':
        return Colors.blue;
      case 'waiting':
        return Colors.orange;
      case 'error':
        return Colors.red;
      default:
        return Colors.grey;
    }
  }

  Color _getLoadColor(double percentage) {
    if (percentage >= 0.8) return Colors.red;
    if (percentage >= 0.5) return Colors.orange;
    return Colors.green;
  }

  String _getNodeIcon(String agentType) {
    switch (agentType) {
      case 'planner':
        return '📋';
      case 'architect':
        return '🏗️';
      case 'tddGuide':
        return '🧪';
      case 'codeReviewer':
        return '👀';
      case 'securityReviewer':
        return '🔒';
      case 'buildErrorResolver':
        return '🔧';
      case 'e2eRunner':
        return '🚀';
      case 'refactorCleaner':
        return '🧹';
      case 'docUpdater':
        return '📝';
      case 'generalPurpose':
        return '🤖';
      case 'explore':
        return '🔍';
      default:
        return '⚙️';
    }
  }

  @override
  bool shouldRepaint(_NetworkGraphPainter oldDelegate) {
    return nodes != oldDelegate.nodes || edges != oldDelegate.edges;
  }
}

/// Timeline view widget
class _TimelineView extends StatelessWidget {
  final List<CollaborationEvent> events;

  const _TimelineView({required this.events});

  @override
  Widget build(BuildContext context) {
    if (events.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.timeline, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text(
              'No collaboration events',
              style: TextStyle(fontSize: 18, color: Colors.grey),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: events.length,
      itemBuilder: (context, index) {
        final event = events[index];
        return _EventCard(event: event);
      },
    );
  }
}

/// Event card widget
class _EventCard extends StatelessWidget {
  final CollaborationEvent event;

  const _EventCard({required this.event});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: ListTile(
        leading: CircleAvatar(
          backgroundColor: _getEventColor(event.type),
          child: Text(_getEventIcon(event.type)),
        ),
        title: Text(event.details.summary),
        subtitle: Text(
          '${_formatTime(event.timestamp)} • ${event.sourceAgentId}',
          style: Theme.of(context).textTheme.bodySmall,
        ),
        trailing: event.details.duration != null
          ? Text('${event.details.duration}ms')
          : null,
        onTap: () => _showEventDetails(context, event),
      ),
    );
  }

  Color _getEventColor(String type) {
    if (type.contains('fail') || type.contains('error')) {
      return Colors.red.withOpacity(0.1);
    }
    if (type.contains('complete')) {
      return Colors.green.withOpacity(0.1);
    }
    if (type.contains('task')) {
      return Colors.blue.withOpacity(0.1);
    }
    return Colors.grey.withOpacity(0.1);
  }

  String _getEventIcon(String type) {
    switch (type) {
      case 'task_assign':
        return '📋';
      case 'task_transfer':
        return '🔄';
      case 'task_complete':
        return '✅';
      case 'task_fail':
        return '❌';
      case 'message_sent':
        return '📤';
      case 'message_received':
        return '📥';
      case 'tool_call':
        return '🔧';
      case 'tool_result':
        return '📊';
      case 'protocol_invoked':
        return '📜';
      default:
        return '⚡';
    }
  }

  String _formatTime(int timestamp) {
    final date = DateTime.fromMillisecondsSinceEpoch(timestamp);
    return '${date.hour}:${date.minute}:${date.second}';
  }

  void _showEventDetails(BuildContext context, CollaborationEvent event) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(event.type),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Summary: ${event.details.summary}'),
            if (event.details.description != null)
              Text('Description: ${event.details.description}'),
            if (event.targetAgentId != null)
              Text('Target: ${event.targetAgentId}'),
            if (event.taskId != null) Text('Task: ${event.taskId}'),
            if (event.details.duration != null)
              Text('Duration: ${event.details.duration}ms'),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }
}

/// Kanban view widget
class _KanbanView extends StatelessWidget {
  final List<CollaborationNode> nodes;
  final List<CollaborationEdge> edges;

  const _KanbanView({
    required this.nodes,
    required this.edges,
  });

  @override
  Widget build(BuildContext context) {
    final pendingTasks = edges.where((e) => e.status == 'pending').toList();
    final runningTasks = edges.where((e) => e.status == 'flowing').toList();
    final completedTasks = edges.where((e) => e.status == 'completed').toList();
    final failedTasks = edges.where((e) => e.status == 'failed').toList();

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        children: [
          _KanbanColumn(title: 'Pending', tasks: pendingTasks, color: Colors.orange),
          _KanbanColumn(title: 'Running', tasks: runningTasks, color: Colors.blue),
          _KanbanColumn(title: 'Completed', tasks: completedTasks, color: Colors.green),
          _KanbanColumn(title: 'Failed', tasks: failedTasks, color: Colors.red),
        ],
      ),
    );
  }
}

/// Kanban column widget
class _KanbanColumn extends StatelessWidget {
  final String title;
  final List<CollaborationEdge> tasks;
  final Color color;

  const _KanbanColumn({
    required this.title,
    required this.tasks,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 280,
      margin: const EdgeInsets.all(8),
      child: Card(
        child: Column(
          children: [
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: color,
                borderRadius: const BorderRadius.vertical(top: Radius.circular(4)),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(
                    title,
                    style: const TextStyle(
                      color: Colors.white,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: Colors.white.withOpacity(0.3),
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: Text(
                      '${tasks.length}',
                      style: const TextStyle(color: Colors.white),
                    ),
                  ),
                ],
              ),
            ),
            Expanded(
              child: ListView.builder(
                itemCount: tasks.length,
                itemBuilder: (context, index) {
                  final task = tasks[index];
                  return _TaskCard(task: task);
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Task card widget
class _TaskCard extends StatelessWidget {
  final CollaborationEdge task;

  const _TaskCard({required this.task});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.all(8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Text(
                    task.taskDescription,
                    style: const TextStyle(fontWeight: FontWeight.bold),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                if (task.duration != null)
                  Text(
                    '${task.duration}ms',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'From: ${task.sourceAgentId}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            Text(
              'To: ${task.targetAgentId}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}