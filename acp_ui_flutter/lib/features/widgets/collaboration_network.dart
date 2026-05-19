/// Collaboration Network
/// 协作网络可视化组件

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/stores/collaboration_store.dart';
import '../../data/models/collaboration.dart';

class CollaborationNetwork extends ConsumerStatefulWidget {
  final bool miniMode;

  const CollaborationNetwork({
    super.key,
    this.miniMode = false,
  });

  @override
  ConsumerState<CollaborationNetwork> createState() => _CollaborationNetworkState();
}

class _CollaborationNetworkState extends ConsumerState<CollaborationNetwork> {
  String? _selectedNodeId;
  String? _selectedEdgeId;
  double _scale = 1.0;

  @override
  Widget build(BuildContext context) {
    final collaborationNotifier = ref.read(collaborationProvider.notifier);
    final stats = collaborationNotifier.getStats();
    final state = ref.watch(collaborationProvider);

    // Initialize mock data if empty
    if (state.nodes.isEmpty && !state.isLoading) {
      collaborationNotifier.initializeMockData();
    }

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.grey.withOpacity(0.05),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.grey.withOpacity(0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 标题栏
          _buildHeader(stats),
          const SizedBox(height: 12),
          // 网络图
          Expanded(
            child: _buildNetworkGraph(state),
          ),
          // 底部信息栏
          if (!widget.miniMode) _buildFooter(state),
        ],
      ),
    );
  }

  Widget _buildHeader(CollaborationNetworkStats stats) {
    return Row(
      children: [
        const Icon(Icons.device_hub, size: 20, color: Colors.purple),
        const SizedBox(width: 8),
        const Text(
          '协作网络',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const Spacer(),
        if (!widget.miniMode)
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildMiniStat('节点', '${stats.totalAgents}'),
              const SizedBox(width: 8),
              _buildMiniStat('连接', '${stats.totalTasks}'),
              const SizedBox(width: 8),
              _buildMiniStat('任务', '${stats.runningTasks}'),
            ],
          ),
      ],
    );
  }

  Widget _buildMiniStat(String label, String value) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.blue.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text('$label: $value', style: const TextStyle(fontSize: 12)),
    );
  }

  Widget _buildNetworkGraph(CollaborationState state) {
    if (state.nodes.isEmpty) {
      return _buildEmptyState();
    }

    return GestureDetector(
      onScaleUpdate: (details) {
        if (!widget.miniMode) {
          setState(() => _scale = details.scale.clamp(0.5, 2.0));
        }
      },
      child: InteractiveViewer(
        minScale: 0.5,
        maxScale: 2.0,
        child: Stack(
          children: [
            // 绘制边（连接线）
            ...state.edges.map((edge) => _buildEdge(edge, state.nodes)),
            // 绘制节点
            ...state.nodes.map((node) => _buildNode(node)),
          ],
        ),
      ),
    );
  }

  Widget _buildEmptyState() {
    return const Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.device_hub, size: 48, color: Colors.grey),
          SizedBox(height: 16),
          Text('暂无协作数据', style: TextStyle(color: Colors.grey)),
        ],
      ),
    );
  }

  Widget _buildEdge(CollaborationEdge edge, List<CollaborationNode> nodes) {
    final fromNode = nodes.firstWhere((n) => n.id == edge.sourceAgentId, orElse: () => nodes.first);
    final toNode = nodes.firstWhere((n) => n.id == edge.targetAgentId, orElse: () => nodes.last);

    final isSelected = _selectedEdgeId == edge.id;
    final edgeColor = _getEdgeColor(edge.status);

    return Positioned(
      left: fromNode.position.x * _scale,
      top: fromNode.position.y * _scale,
      child: CustomPaint(
        size: Size(
          (toNode.position.x - fromNode.position.x) * _scale,
          (toNode.position.y - fromNode.position.y) * _scale,
        ),
        painter: EdgePainter(
          color: isSelected ? Colors.blue : edgeColor,
          strokeWidth: isSelected ? 3 : 2,
          status: edge.status,
        ),
      ),
    );
  }

  Color _getEdgeColor(String status) {
    switch (status) {
      case 'flowing':
        return Colors.green;
      case 'pending':
        return Colors.orange;
      case 'completed':
        return Colors.blue;
      default:
        return Colors.grey;
    }
  }

  Widget _buildNode(CollaborationNode node) {
    final isSelected = _selectedNodeId == node.id;

    return Positioned(
      left: node.position.x * _scale,
      top: node.position.y * _scale,
      child: GestureDetector(
        onTap: () => setState(() => _selectedNodeId = node.id),
        child: Container(
          width: (widget.miniMode ? 60 : 80) * _scale,
          height: (widget.miniMode ? 60 : 80) * _scale,
          decoration: BoxDecoration(
            color: _getNodeColor(node.status).withOpacity(0.2),
            shape: BoxShape.circle,
            border: Border.all(
              color: isSelected ? Colors.blue : _getNodeColor(node.status),
              width: isSelected ? 3 : 2,
            ),
            boxShadow: isSelected
                ? [
                    BoxShadow(
                      color: Colors.blue.withOpacity(0.3),
                      blurRadius: 10,
                      spreadRadius: 3,
                    ),
                  ]
                : null,
          ),
          child: Stack(
            alignment: Alignment.center,
            children: [
              // Agent图标
              Text(
                _getNodeIcon(node.agentType),
                style: TextStyle(fontSize: (widget.miniMode ? 24 : 32) * _scale),
              ),
              // 节点名称
              Positioned(
                bottom: 0,
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.white.withOpacity(0.8),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    node.agentName,
                    style: TextStyle(fontSize: (widget.miniMode ? 10 : 12) * _scale),
                    textAlign: TextAlign.center,
                  ),
                ),
              ),
              // 负载指示器
              if (!widget.miniMode)
                Positioned(
                  top: 4,
                  right: 4,
                  child: _buildLoadIndicator(node.currentLoad / node.maxLoad),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Color _getNodeColor(String status) {
    switch (status) {
      case 'active':
        return Colors.green;
      case 'idle':
        return Colors.grey;
      case 'busy':
        return Colors.orange;
      case 'error':
        return Colors.red;
      default:
        return Colors.grey;
    }
  }

  String _getNodeIcon(String agentType) {
    switch (agentType) {
      case 'planner':
        return '📋';
      case 'architect':
        return '🏗️';
      case 'developer':
        return '💻';
      case 'tddGuide':
        return '🧪';
      case 'codeReviewer':
        return '🔍';
      case 'securityReviewer':
        return '🔒';
      case 'performanceReviewer':
        return '⚡';
      case 'docUpdater':
        return '📖';
      default:
        return '🤖';
    }
  }

  Widget _buildLoadIndicator(double load) {
    final color = load > 0.7 ? Colors.red : load > 0.5 ? Colors.orange : Colors.green;

    return Container(
      width: 16,
      height: 16,
      decoration: BoxDecoration(
        color: color,
        shape: BoxShape.circle,
      ),
      child: Center(
        child: Text(
          '${(load * 100).toStringAsFixed(0)}',
          style: const TextStyle(fontSize: 8, color: Colors.white),
        ),
      ),
    );
  }

  Widget _buildFooter(CollaborationState state) {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _buildFooterButton('网络图', Icons.device_hub, true),
          _buildFooterButton('时间线', Icons.timeline, false),
          _buildFooterButton('看板', Icons.view_kanban, false),
          // 缩放控件
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              IconButton(
                icon: const Icon(Icons.zoom_out, size: 20),
                onPressed: () => setState(() => _scale = (_scale - 0.1).clamp(0.5, 2.0)),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(),
              ),
              Text('${(_scale * 100).toStringAsFixed(0)}%', style: const TextStyle(fontSize: 12)),
              IconButton(
                icon: const Icon(Icons.zoom_in, size: 20),
                onPressed: () => setState(() => _scale = (_scale + 0.1).clamp(0.5, 2.0)),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildFooterButton(String label, IconData icon, bool isActive) {
    return InkWell(
      onTap: () {},
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
        decoration: BoxDecoration(
          color: isActive ? Colors.purple : Colors.transparent,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(
            color: isActive ? Colors.purple : Colors.grey.withOpacity(0.3),
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 16, color: isActive ? Colors.white : Colors.grey),
            const SizedBox(width: 4),
            Text(
              label,
              style: TextStyle(
                color: isActive ? Colors.white : Colors.grey,
                fontSize: 12,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// 边绘制器
class EdgePainter extends CustomPainter {
  final Color color;
  final double strokeWidth;
  final String status;

  EdgePainter({
    required this.color,
    required this.strokeWidth,
    required this.status,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();
    paint.color = color;
    paint.strokeWidth = strokeWidth;
    paint.style = PaintingStyle.stroke;

    // 绘制连接线
    final path = Path();
    path.moveTo(0, 0);
    path.lineTo(size.width, size.height);
    canvas.drawPath(path, paint);

    // 绘制箭头
    final arrowPaint = Paint();
    arrowPaint.color = color;
    arrowPaint.style = PaintingStyle.fill;

    final arrowPath = Path();
    final arrowSize = 10.0;
    final endX = size.width;
    final endY = size.height;

    arrowPath.moveTo(endX, endY);
    arrowPath.lineTo(endX - arrowSize, endY - arrowSize);
    arrowPath.lineTo(endX - arrowSize, endY + arrowSize);
    arrowPath.close();

    canvas.drawPath(arrowPath, arrowPaint);

    // 如果是流动状态，添加动态效果指示
    if (status == 'flowing') {
      final dotPaint = Paint();
      dotPaint.color = Colors.white;
      dotPaint.style = PaintingStyle.fill;

      final centerX = size.width / 2;
      final centerY = size.height / 2;
      canvas.drawCircle(Offset(centerX, centerY), 4, dotPaint);
    }
  }

  @override
  bool shouldRepaint(covariant EdgePainter oldDelegate) {
    return color != oldDelegate.color ||
        strokeWidth != oldDelegate.strokeWidth ||
        status != oldDelegate.status;
  }
}