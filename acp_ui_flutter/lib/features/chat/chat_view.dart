import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../data/stores/agent_store.dart';
import '../../data/stores/session_store.dart';
import '../../data/models/session.dart';
import '../../data/models/agent.dart';

/// Chat view - main conversation interface
class ChatView extends ConsumerStatefulWidget {
  const ChatView({super.key});

  @override
  ConsumerState<ChatView> createState() => _ChatViewState();
}

class _ChatViewState extends ConsumerState<ChatView> {
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();

  @override
  void dispose() {
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final agents = ref.watch(agentListProvider);
    final selectedAgent = ref.watch(selectedAgentProvider);
    final currentSession = ref.watch(currentSessionProvider);
    final messages = currentSession != null
        ? ref.watch(sessionMessagesProvider(currentSession.id))
        : <SessionMessage>[];

    return Scaffold(
      body: Column(
        children: [
          // Header
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
                  'Chat',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
                const Spacer(),
                // Agent selector dropdown
                _AgentSelector(
                  agents: agents,
                  selectedAgent: selectedAgent,
                  onSelect: (agent) {
                    ref.read(selectedAgentProvider.notifier).state = agent;
                  },
                ),
              ],
            ),
          ),
          // Messages area
          Expanded(
            child: _MessagesArea(
              scrollController: _scrollController,
              messages: messages,
            ),
          ),
          // Input area
          _ChatInput(
            controller: _messageController,
            onSend: () => _sendMessage(ref, selectedAgent),
            enabled: selectedAgent != null,
          ),
        ],
      ),
    );
  }

  void _sendMessage(WidgetRef ref, Agent? agent) {
    final text = _messageController.text.trim();
    if (text.isEmpty || agent == null) return;

    final sessionManager = ref.read(sessionManagerProvider);
    var session = ref.read(currentSessionProvider);

    // Create session if none exists
    if (session == null) {
      session = sessionManager.createSession(agent.id);
      ref.read(currentSessionProvider.notifier).state = session;
    }

    // Add user message
    final userMessage = SessionMessage(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      role: MessageRole.user,
      content: text,
      timestamp: DateTime.now(),
    );

    sessionManager.addMessage(session.id, userMessage);
    _messageController.clear();

    // Scroll to bottom
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });

    // Simulate agent response for now
    Future.delayed(const Duration(seconds: 1), () {
      if (session != null) {
        final agentMessage = SessionMessage(
          id: DateTime.now().millisecondsSinceEpoch.toString(),
          role: MessageRole.assistant,
          content: 'Agent ${agent.name} received: "$text"',
          timestamp: DateTime.now(),
        );
        sessionManager.addMessage(session.id, agentMessage);
      }
    });
  }
}

class _AgentSelector extends StatelessWidget {
  const _AgentSelector({
    required this.agents,
    required this.selectedAgent,
    required this.onSelect,
  });

  final List<Agent> agents;
  final Agent? selectedAgent;
  final void Function(Agent) onSelect;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
      ),
      child: DropdownButton<Agent>(
        value: selectedAgent,
        hint: const Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.smart_toy_outlined, size: 20),
            SizedBox(width: 8),
            Text('Select Agent'),
          ],
        ),
        underline: const SizedBox(),
        items: agents.map((agent) {
          return DropdownMenuItem(
            value: agent,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(_getAgentIcon(agent.type), size: 20),
                const SizedBox(width: 8),
                Text(agent.name),
              ],
            ),
          );
        }).toList(),
        onChanged: (agent) {
          if (agent != null) onSelect(agent);
        },
      ),
    );
  }

  IconData _getAgentIcon(AgentType type) {
    switch (type) {
      case AgentType.planner:
        return Icons.architecture;
      case AgentType.architect:
        return Icons.domain;
      case AgentType.codeReviewer:
        return Icons.code;
      case AgentType.tddGuide:
        return Icons.science;
      case AgentType.securityReviewer:
        return Icons.security;
      default:
        return Icons.smart_toy;
    }
  }
}

class _MessagesArea extends StatelessWidget {
  const _MessagesArea({
    required this.scrollController,
    required this.messages,
  });

  final ScrollController scrollController;
  final List<SessionMessage> messages;

  @override
  Widget build(BuildContext context) {
    if (messages.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.chat_bubble_outline, size: 64, color: Colors.grey),
            const SizedBox(height: 16),
            Text(
              'Select an agent to start chatting',
              style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      controller: scrollController,
      padding: const EdgeInsets.all(16),
      itemCount: messages.length,
      itemBuilder: (context, index) {
        final message = messages[index];
        return _MessageBubble(
          role: message.role,
          content: message.content,
          timestamp: message.timestamp,
        );
      },
    );
  }
}

class _MessageBubble extends StatelessWidget {
  const _MessageBubble({
    required this.role,
    required this.content,
    required this.timestamp,
  });

  final MessageRole role;
  final String content;
  final DateTime timestamp;

  @override
  Widget build(BuildContext context) {
    final isUser = role == MessageRole.user;

    return Align(
      alignment: isUser ? Alignment.centerRight : Alignment.centerLeft,
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 8),
        padding: const EdgeInsets.all(12),
        constraints: BoxConstraints(
          maxWidth: MediaQuery.of(context).size.width * 0.7,
        ),
        decoration: BoxDecoration(
          color: isUser
              ? Theme.of(context).colorScheme.primary
              : Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              content,
              style: TextStyle(
                color: isUser
                    ? Theme.of(context).colorScheme.onPrimary
                    : Theme.of(context).colorScheme.onSurface,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              _formatTime(timestamp),
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

class _ChatInput extends StatelessWidget {
  const _ChatInput({
    required this.controller,
    required this.onSend,
    required this.enabled,
  });

  final TextEditingController controller;
  final VoidCallback onSend;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    return Container(
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
              controller: controller,
              enabled: enabled,
              decoration: InputDecoration(
                hintText: enabled ? 'Type a message...' : 'Select an agent first',
                suffixIcon: IconButton(
                  icon: Icon(
                    Icons.send,
                    color: enabled
                        ? Theme.of(context).colorScheme.primary
                        : Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
                  onPressed: enabled ? onSend : null,
                ),
              ),
              onSubmitted: enabled ? (_) => onSend() : null,
            ),
          ),
        ],
      ),
    );
  }
}