import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Instruction status
enum InstructionStatus { sending, sent, delivered, executing, completed, failed }

/// Instruction message model
class Instruction {
  final String id;
  final String content;
  final DateTime timestamp;
  final InstructionStatus status;
  final String? response;
  final bool isVoice;

  const Instruction({
    required this.id,
    required this.content,
    required this.timestamp,
    this.status = InstructionStatus.sent,
    this.response,
    this.isVoice = false,
  });

  Instruction copyWith({
    String? id,
    String? content,
    DateTime? timestamp,
    InstructionStatus? status,
    String? response,
    bool? isVoice,
  }) {
    return Instruction(
      id: id ?? this.id,
      content: content ?? this.content,
      timestamp: timestamp ?? this.timestamp,
      status: status ?? this.status,
      response: response ?? this.response,
      isVoice: isVoice ?? this.isVoice,
    );
  }
}

/// Instruction state
class InstructionState {
  final List<Instruction> messages;
  final bool isRecording;

  const InstructionState({
    this.messages = const [],
    this.isRecording = false,
  });

  InstructionState copyWith({
    List<Instruction>? messages,
    bool? isRecording,
  }) {
    return InstructionState(
      messages: messages ?? this.messages,
      isRecording: isRecording ?? this.isRecording,
    );
  }
}

final _instructionStateProvider =
    StateNotifierProvider<InstructionNotifier, InstructionState>((ref) {
  return InstructionNotifier();
});

class InstructionNotifier extends StateNotifier<InstructionState> {
  InstructionNotifier()
      : super(InstructionState(
          messages: [
            Instruction(
              id: 'demo-1',
              content: 'Check deployment status for agent-team-alpha',
              timestamp: DateTime(2026, 6, 6, 10, 30),
              status: InstructionStatus.completed,
              response: 'Deployment completed successfully at 10:32 AM. All 3 agents running.',
            ),
            Instruction(
              id: 'demo-2',
              content: 'Pause all non-critical agents',
              timestamp: DateTime(2026, 6, 6, 9, 15),
              status: InstructionStatus.completed,
              response: 'Paused 5 agents. 3 critical agents remain active.',
            ),
          ],
        ));

  void sendMessage(String content, {bool isVoice = false}) {
    if (content.trim().isEmpty) return;

    final instruction = Instruction(
      id: 'inst-${DateTime.now().millisecondsSinceEpoch}',
      content: content.trim(),
      timestamp: DateTime.now(),
      status: InstructionStatus.sending,
      isVoice: isVoice,
    );

    state = state.copyWith(
      messages: [...state.messages, instruction],
    );

    // Simulate sending → delivered → executing → completed
    _simulateProgress(instruction.id);
  }

  Future<void> _simulateProgress(String id) async {
    await Future.delayed(const Duration(milliseconds: 800));
    _updateStatus(id, InstructionStatus.sent);

    await Future.delayed(const Duration(milliseconds: 1200));
    _updateStatus(id, InstructionStatus.delivered);

    await Future.delayed(const Duration(milliseconds: 2000));
    _updateStatus(id, InstructionStatus.executing);

    await Future.delayed(const Duration(milliseconds: 3000));
    _updateStatus(id, InstructionStatus.completed,
        response: 'Instruction received and queued for execution.');
  }

  void _updateStatus(String id, InstructionStatus status, {String? response}) {
    final messages = state.messages.map((m) {
      if (m.id == id) {
        return m.copyWith(status: status, response: response);
      }
      return m;
    }).toList();
    state = state.copyWith(messages: messages);
  }

  void toggleRecording() {
    state = state.copyWith(isRecording: !state.isRecording);
  }
}

/// Send commands — the "发指令" (Send Instructions) screen.
/// Chat-like interface for sending voice/text commands to agents.
class InstructionScreen extends ConsumerWidget {
  const InstructionScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(_instructionStateProvider);

    return Scaffold(
      body: Column(
        children: [
          _buildAppBar(context),
          Expanded(
            child: state.messages.isEmpty
                ? _buildEmptyState(context)
                : _buildMessageList(context, state.messages),
          ),
          _buildInputBar(context, ref),
        ],
      ),
    );
  }

  Widget _buildAppBar(BuildContext context) {
    return Container(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).colorScheme.outlineVariant.withOpacity(0.3),
          ),
        ),
      ),
      child: Row(
        children: [
          Text(
            'Instructions',
            style: Theme.of(context).textTheme.titleLarge?.copyWith(
                  fontWeight: FontWeight.w700,
                ),
          ),
          const Spacer(),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primaryContainer,
              borderRadius: BorderRadius.circular(12),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.circle,
                  size: 8,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const SizedBox(width: 6),
                Text(
                  'Agents online',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: Theme.of(context).colorScheme.onPrimaryContainer,
                      ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            Icons.smart_toy_outlined,
            size: 64,
            color: Theme.of(context).colorScheme.outlineVariant,
          ),
          const SizedBox(height: 16),
          Text(
            'Send an instruction',
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  color: Theme.of(context).colorScheme.outline,
                ),
          ),
          const SizedBox(height: 8),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Text(
              'Use natural language to direct agents. Try: "Check task status" or "Deploy to staging"',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Theme.of(context).colorScheme.outlineVariant,
                  ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMessageList(BuildContext context, List<Instruction> messages) {
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      reverse: false,
      itemCount: messages.length,
      itemBuilder: (context, index) {
        return _InstructionBubble(instruction: messages[index]);
      },
    );
  }

  Widget _buildInputBar(BuildContext context, WidgetRef ref) {
    return Container(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 20),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(
          top: BorderSide(
            color: Theme.of(context).colorScheme.outlineVariant.withOpacity(0.3),
          ),
        ),
      ),
      child: SafeArea(
        top: false,
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            // Voice button (placeholder)
            Consumer(
              builder: (context, ref, child) {
                final isRecording = ref.watch(_instructionStateProvider).isRecording;
                return IconButton(
                  onPressed: () =>
                      ref.read(_instructionStateProvider.notifier).toggleRecording(),
                  icon: Icon(
                    isRecording ? Icons.stop_circle : Icons.mic_none_rounded,
                    color: isRecording
                        ? Theme.of(context).colorScheme.error
                        : Theme.of(context).colorScheme.primary,
                    size: 28,
                  ),
                  tooltip: isRecording ? 'Stop recording' : 'Voice input',
                );
              },
            ),
            const SizedBox(width: 8),
            // Text input
            Expanded(
              child: _InstructionTextField(
                onSend: (text, isVoice) =>
                    ref.read(_instructionStateProvider.notifier).sendMessage(text, isVoice: isVoice),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _InstructionTextField extends StatefulWidget {
  final Function(String text, bool isVoice) onSend;

  const _InstructionTextField({required this.onSend});

  @override
  State<_InstructionTextField> createState() => _InstructionTextFieldState();
}

class _InstructionTextFieldState extends State<_InstructionTextField> {
  final TextEditingController _controller = TextEditingController();

  @override
  void initState() {
    super.initState();
    _controller.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _controller.removeListener(() => setState(() {}));
    _controller.dispose();
    super.dispose();
  }

  @override
  void initState() {
    super.initState();
    _controller.addListener(() => setState(() {}));
  }

  @override
  void dispose() {
    _controller.removeListener(() => setState(() {}));
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: _controller,
      decoration: InputDecoration(
        hintText: 'Type an instruction...',
        suffixIcon: _controller.text.trim().isNotEmpty
            ? IconButton(
                icon: const Icon(Icons.send, size: 20),
                onPressed: () {
                  widget.onSend(_controller.text, false);
                  _controller.clear();
                },
              )
            : null,
      ),
      textInputAction: TextInputAction.send,
      maxLines: null,
      minLines: 1,
      onSubmitted: (value) {
        if (value.trim().isNotEmpty) {
          widget.onSend(value, false);
          _controller.clear();
        }
      },
    );
  }
}

class _InstructionBubble extends StatelessWidget {
  final Instruction instruction;

  const _InstructionBubble({required this.instruction});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          // User message (right-aligned)
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              Flexible(
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 14,
                    vertical: 10,
                  ),
                  decoration: BoxDecoration(
                    color: theme.colorScheme.primary,
                    borderRadius: const BorderRadius.only(
                      topLeft: Radius.circular(16),
                      topRight: Radius.circular(4),
                      bottomLeft: Radius.circular(16),
                      bottomRight: Radius.circular(16),
                    ),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (instruction.isVoice)
                        Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Icon(Icons.mic, size: 14, color: theme.colorScheme.onPrimary.withOpacity(0.7)),
                            const SizedBox(width: 4),
                          ],
                        ),
                      Text(
                        instruction.content,
                        style: theme.textTheme.bodyMedium?.copyWith(
                          color: theme.colorScheme.onPrimary,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 8),
              // Status icon
              _statusIcon(instruction.status, theme),
            ],
          ),
          // Timestamp
          Padding(
            padding: const EdgeInsets.only(top: 4, right: 8),
            child: Text(
              _formatTime(instruction.timestamp),
              style: theme.textTheme.labelSmall?.copyWith(
                color: theme.colorScheme.outlineVariant,
              ),
            ),
          ),
          // System response
          if (instruction.response != null) ...[
            const SizedBox(height: 8),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  padding: const EdgeInsets.all(8),
                  margin: const EdgeInsets.only(right: 8),
                  decoration: BoxDecoration(
                    color: theme.colorScheme.tertiaryContainer,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Icon(
                    Icons.smart_toy_outlined,
                    size: 16,
                    color: theme.colorScheme.onTertiaryContainer,
                  ),
                ),
                Expanded(
                  child: Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 12,
                      vertical: 10,
                    ),
                    decoration: BoxDecoration(
                      color: theme.colorScheme.surfaceContainerHighest,
                      borderRadius: const BorderRadius.only(
                        topLeft: Radius.circular(4),
                        topRight: Radius.circular(16),
                        bottomLeft: Radius.circular(16),
                        bottomRight: Radius.circular(16),
                      ),
                    ),
                    child: Text(
                      instruction.response!,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ),
                ),
              ],
            ),
            Padding(
              padding: const EdgeInsets.only(top: 4, left: 8),
              child: Text(
                'Agent response',
                style: theme.textTheme.labelSmall?.copyWith(
                  color: theme.colorScheme.outlineVariant,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _statusIcon(InstructionStatus status, ThemeData theme) {
    switch (status) {
      case InstructionStatus.sending:
        return const SizedBox(
          width: 16,
          height: 16,
          child: CircularProgressIndicator(strokeWidth: 2),
        );
      case InstructionStatus.sent:
        return Icon(Icons.check, size: 16, color: theme.colorScheme.outline);
      case InstructionStatus.delivered:
        return Icon(Icons.done_all, size: 16, color: theme.colorScheme.outline);
      case InstructionStatus.executing:
        return SizedBox(
          width: 16,
          height: 16,
          child: CircularProgressIndicator(
            strokeWidth: 2,
            valueColor: AlwaysStoppedAnimation(theme.colorScheme.primary),
          ),
        );
      case InstructionStatus.completed:
        return Icon(Icons.done_all, size: 16, color: theme.colorScheme.tertiary);
      case InstructionStatus.failed:
        return Icon(Icons.error_outline, size: 16, color: theme.colorScheme.error);
    }
  }

  String _formatTime(DateTime time) {
    final hour = time.hour.toString().padLeft(2, '0');
    final minute = time.minute.toString().padLeft(2, '0');
    return '$hour:$minute';
  }
}
