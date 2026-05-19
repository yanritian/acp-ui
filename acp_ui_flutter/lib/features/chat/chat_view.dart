import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../data/stores/agent_store.dart';
import '../../data/services/websocket_service.dart';
import '../../data/models/agent.dart';

/// Chat view - main conversation interface with WebSocket integration
class ChatView extends ConsumerStatefulWidget {
  const ChatView({super.key});

  @override
  ConsumerState<ChatView> createState() => _ChatViewState();
}

class _ChatViewState extends ConsumerState<ChatView> {
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();
  final List<ChatMessage> _messages = [];
  final List<GeneratedFileInfo> _generatedFiles = [];
  StreamSubscription<String>? _messageSubscription;
  bool _isExecutiveAgentInitialized = false;

  @override
  void initState() {
    super.initState();
    _connectWebSocket();
  }

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    _messageSubscription?.cancel();
    super.dispose();
  }

  Future<void> _connectWebSocket() async {
    final wsService = ref.read(webSocketServiceProvider);
    try {
      await wsService.connect();
      ref.read(connectionStatusProvider.notifier).state = true;

      // Listen to incoming messages
      _messageSubscription = wsService.messageStream.listen((data) {
        _handleIncomingMessage(data);
      });

      // Initialize executive agent
      await _initExecutiveAgent();
    } catch (e) {
      print('[Chat] WebSocket connection failed: $e');
      ref.read(connectionStatusProvider.notifier).state = false;
    }
  }

  Future<void> _initExecutiveAgent() async {
    final wsService = ref.read(webSocketServiceProvider);
    if (!wsService.isConnected()) return;

    // Initialize with default workspace
    await wsService.initExecutiveAgent('D:/dingsun/acp-ui/erp_system');
    _isExecutiveAgentInitialized = true;

    setState(() {
      _messages.add(ChatMessage(
        agentName: 'System',
        content: '✅ Executive Agent 已初始化\n工作目录: D:/dingsun/acp-ui/erp_system',
        isAgent: true,
        timestamp: DateTime.now(),
      ));
    });
  }

  void _handleIncomingMessage(String data) {
    try {
      final msg = jsonDecode(data) as Map<String, dynamic>;
      final msgType = msg['type'] as String?;

      if (msgType == 'agent_message') {
        final content = msg['content'] as String?;
        final agentName = msg['agentName'] as String?;
        if (content != null) {
          setState(() {
            _messages.add(ChatMessage(
              agentName: agentName ?? 'Agent',
              content: content,
              isAgent: true,
              timestamp: DateTime.now(),
            ));
          });
          _scrollToBottom();
        }
      } else if (msgType == 'agent_status') {
        final status = msg['status'] as Map<String, dynamic>?;
        if (status != null) {
          // Update agent status in provider
          final notifier = ref.read(agentStatusProvider.notifier);
          for (final entry in status.entries) {
            final agentId = entry.key;
            final statusStr = entry.value as String;
            final agentStatus = _parseStatus(statusStr);
            notifier.updateStatus(agentId, agentStatus);
          }
        }
      } else if (msgType == 'file-created') {
        final path = msg['path'] as String?;
        final lines = msg['lines'] as int?;
        if (path != null) {
          setState(() {
            _generatedFiles.add(GeneratedFileInfo(
              path: path,
              lines: lines ?? 0,
              timestamp: DateTime.now(),
            ));
            _messages.add(ChatMessage(
              agentName: 'System',
              content: '📁 创建文件: $path (${lines ?? 0} 行)',
              isAgent: true,
              timestamp: DateTime.now(),
            ));
          });
          _scrollToBottom();
        }
      } else if (msgType == 'files-created') {
        final files = msg['files'] as List?;
        if (files != null && files.isNotEmpty) {
          setState(() {
            _messages.add(ChatMessage(
              agentName: 'System',
              content: '📁 批量创建 ${files.length} 个文件:\n${files.map((f) => '  - $f').join('\n')}',
              isAgent: true,
              timestamp: DateTime.now(),
            ));
          });
          _scrollToBottom();
        }
      } else if (msgType == 'task-completed') {
        final summary = msg['summary'] as String?;
        final files = msg['files'] as List?;
        final workspace = msg['workspace'] as String?;

        setState(() {
          _messages.add(ChatMessage(
            agentName: 'System',
            content: '🎉 任务完成!\n\n生成文件: ${files?.length ?? 0} 个\n工作目录: $workspace\n\n$summary',
            isAgent: true,
            timestamp: DateTime.now(),
          ));
        });
        _scrollToBottom();

        // Refresh generated files list
        final wsService = ref.read(webSocketServiceProvider);
        wsService.getGeneratedFiles();
      } else if (msgType == 'agent-error') {
        final error = msg['error'] as String?;
        if (error != null) {
          setState(() {
            _messages.add(ChatMessage(
              agentName: 'Error',
              content: '❌ 错误: $error',
              isAgent: true,
              isError: true,
              timestamp: DateTime.now(),
            ));
          });
          _scrollToBottom();
        }
      } else if (msg['ok'] == true && msg['data'] != null) {
        // Handle JSON-RPC response
        final responseData = msg['data'] as Map<String, dynamic>;
        if (responseData['initialized'] == true) {
          _isExecutiveAgentInitialized = true;
        } else if (responseData['files'] != null) {
          final filesList = responseData['files'] as List;
          setState(() {
            _generatedFiles.clear();
            for (final f in filesList) {
              final fileMap = f as Map<String, dynamic>;
              _generatedFiles.add(GeneratedFileInfo(
                path: fileMap['relativePath'] as String? ?? '',
                lines: fileMap['lines'] as int? ?? 0,
                timestamp: DateTime.parse(fileMap['createdAt'] as String? ?? DateTime.now().toIso8601String()),
              ));
            }
          });
        }
      }
    } catch (e) {
      print('[Chat] Parse error: $e');
    }
  }

  AgentStatus _parseStatus(String status) {
    switch (status) {
      case 'busy': return AgentStatus.busy;
      case 'error': return AgentStatus.error;
      case 'offline': return AgentStatus.offline;
      default: return AgentStatus.idle;
    }
  }

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _sendMessage() async {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;

    final wsService = ref.read(webSocketServiceProvider);
    if (!wsService.isConnected()) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Not connected to agent server')),
      );
      return;
    }

    // Add user message to list
    setState(() {
      _messages.add(ChatMessage(
        agentName: 'You',
        content: text,
        isAgent: false,
        timestamp: DateTime.now(),
      ));
    });

    _messageController.clear();
    _scrollToBottom();

    // Execute development task (新功能 - 实际执行)
    if (_isExecutiveAgentInitialized) {
      await wsService.executeDevelopmentTask(text);
    } else {
      // Legacy: just send user request
      await wsService.sendUserRequest(text);
    }
  }

  @override
  Widget build(BuildContext context) {
    final agents = ref.watch(agentListProvider);
    final statusMap = ref.watch(agentStatusProvider);
    final isConnected = ref.watch(connectionStatusProvider);

    return Scaffold(
      body: Column(
        children: [
          // Header with connection status
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.05),
                  blurRadius: 4,
                  offset: const Offset(0, 2),
                ),
              ],
            ),
            child: Row(
              children: [
                Text(
                  'Executive Agent Chat',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                const Spacer(),
                // Connection status indicator
                Row(
                  children: [
                    Icon(
                      Icons.circle,
                      color: isConnected ? Colors.green : Colors.red,
                      size: 12,
                    ),
                    const SizedBox(width: 8),
                    Text(
                      isConnected ? 'Connected' : 'Disconnected',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
                const SizedBox(width: 16),
                // Executive Agent status
                if (_isExecutiveAgentInitialized)
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: Colors.green.withOpacity(0.2),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Text('✅ Ready', style: TextStyle(color: Colors.green)),
                  ),
              ],
            ),
          ),
          // Generated files indicator
          if (_generatedFiles.isNotEmpty)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              color: Theme.of(context).colorScheme.primaryContainer.withOpacity(0.3),
              child: Row(
                children: [
                  Icon(Icons.folder, size: 16, color: Theme.of(context).colorScheme.primary),
                  const SizedBox(width: 8),
                  Text(
                    '已生成 ${_generatedFiles.length} 个文件',
                    style: TextStyle(color: Theme.of(context).colorScheme.primary),
                  ),
                  const Spacer(),
                  TextButton(
                    onPressed: () => _showGeneratedFilesDialog(),
                    child: Text('查看'),
                  ),
                ],
              ),
            ),
          // Agent status bar
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            color: Theme.of(context).colorScheme.surfaceContainerHighest.withOpacity(0.5),
            child: Row(
              children: [
                Text('Agents:', style: Theme.of(context).textTheme.bodySmall),
                const SizedBox(width: 8),
                Expanded(
                  child: Wrap(
                    spacing: 12,
                    children: [
                      _AgentStatusBadge(name: 'Planner', status: statusMap['planner'] ?? AgentStatus.idle),
                      _AgentStatusBadge(name: 'Architect', status: statusMap['architect'] ?? AgentStatus.idle),
                      _AgentStatusBadge(name: 'Coder', status: statusMap['coder'] ?? AgentStatus.idle),
                      _AgentStatusBadge(name: 'Reviewer', status: statusMap['codeReviewer'] ?? AgentStatus.idle),
                      _AgentStatusBadge(name: 'Tester', status: statusMap['tester'] ?? AgentStatus.idle),
                      _AgentStatusBadge(name: 'Security', status: statusMap['securityReviewer'] ?? AgentStatus.idle),
                    ],
                  ),
                ),
              ],
            ),
          ),
          // Messages area
          Expanded(
            child: _messages.isEmpty
                ? Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(Icons.code, size: 64, color: Colors.grey),
                        const SizedBox(height: 16),
                        Text(
                          '发送需求，Agent 将实际编写代码',
                          style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                            color: Theme.of(context).colorScheme.onSurfaceVariant,
                          ),
                        ),
                        const SizedBox(height: 8),
                        Text(
                          '示例: "做一个ERP系统"',
                          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Theme.of(context).colorScheme.onSurfaceVariant,
                          ),
                        ),
                      ],
                    ),
                  )
                : ListView.builder(
                    controller: _scrollController,
                    padding: const EdgeInsets.all(16),
                    itemCount: _messages.length,
                    itemBuilder: (context, index) {
                      final message = _messages[index];
                      return _MessageBubble(message: message);
                    },
                  ),
          ),
          // Input area
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surface,
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withOpacity(0.05),
                  blurRadius: 4,
                  offset: const Offset(0, -2),
                ),
              ],
            ),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _messageController,
                    decoration: InputDecoration(
                      hintText: '输入开发需求 (Agent 将实际编写代码)...',
                      suffixIcon: IconButton(
                        icon: Icon(
                          Icons.send,
                          color: isConnected
                              ? Theme.of(context).colorScheme.primary
                              : Theme.of(context).colorScheme.onSurfaceVariant,
                        ),
                        onPressed: isConnected ? _sendMessage : null,
                      ),
                    ),
                    onSubmitted: (_) => _sendMessage(),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _showGeneratedFilesDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('生成的文件 (${_generatedFiles.length})'),
        content: SizedBox(
          width: double.maxFinite,
          height: 300,
          child: ListView.builder(
            itemCount: _generatedFiles.length,
            itemBuilder: (context, index) {
              final file = _generatedFiles[index];
              return ListTile(
                leading: Icon(Icons.code, color: Theme.of(context).colorScheme.primary),
                title: Text(file.path),
                subtitle: Text('${file.lines} 行 · ${_formatTime(file.timestamp)}'),
              );
            },
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: Text('关闭'),
          ),
        ],
      ),
    );
  }

  String _formatTime(DateTime time) {
    return '${time.hour}:${time.minute.toString().padLeft(2, '0')}';
  }
}

class ChatMessage {
  final String agentName;
  final String content;
  final bool isAgent;
  final bool isError;
  final DateTime timestamp;

  ChatMessage({
    required this.agentName,
    required this.content,
    required this.isAgent,
    this.isError = false,
    required this.timestamp,
  });
}

class GeneratedFileInfo {
  final String path;
  final int lines;
  final DateTime timestamp;

  GeneratedFileInfo({
    required this.path,
    required this.lines,
    required this.timestamp,
  });
}

class _AgentStatusBadge extends StatelessWidget {
  const _AgentStatusBadge({
    required this.name,
    required this.status,
  });

  final String name;
  final AgentStatus status;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: _getStatusColor(status),
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 4),
        Text(name, style: Theme.of(context).textTheme.bodySmall),
      ],
    );
  }

  Color _getStatusColor(AgentStatus status) {
    switch (status) {
      case AgentStatus.idle: return Colors.green;
      case AgentStatus.busy: return Colors.orange;
      case AgentStatus.error: return Colors.red;
      case AgentStatus.offline: return Colors.grey;
    }
  }
}

class _MessageBubble extends StatelessWidget {
  const _MessageBubble({required this.message});

  final ChatMessage message;

  @override
  Widget build(BuildContext context) {
    final isUser = !message.isAgent;

    return Align(
      alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 8),
        padding: const EdgeInsets.all(12),
        constraints: BoxConstraints(
          maxWidth: MediaQuery.of(context).size.width * 0.85,
        ),
        decoration: BoxDecoration(
          color: message.isError
              ? Colors.red.withOpacity(0.1)
              : isUser
                  ? Theme.of(context).colorScheme.primary
                  : Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (message.isAgent)
              Padding(
                padding: const EdgeInsets.only(bottom: 4),
                child: Row(
                  children: [
                    if (message.isError)
                      Icon(Icons.error_outline, size: 14, color: Colors.red)
                    else if (message.agentName == 'System')
                      Icon(Icons.info_outline, size: 14, color: Theme.of(context).colorScheme.primary)
                    else
                      Icon(Icons.smart_toy, size: 14, color: Theme.of(context).colorScheme.primary),
                    const SizedBox(width: 4),
                    Text(
                      message.agentName,
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: message.isError
                            ? Colors.red
                            : Theme.of(context).colorScheme.primary,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                  ],
                ),
              ),
            SelectableText(
              message.content,
              style: TextStyle(
                color: message.isError
                    ? Colors.red
                    : isUser
                        ? Theme.of(context).colorScheme.onPrimary
                        : Theme.of(context).colorScheme.onSurface,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              _formatTime(message.timestamp),
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: isUser
                    ? Theme.of(context).colorScheme.onPrimary.withOpacity(0.7)
                    : Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      ),
    );
  }

  String _formatTime(DateTime time) {
    return '${time.hour}:${time.minute.toString().padLeft(2, '0')}';
  }
}