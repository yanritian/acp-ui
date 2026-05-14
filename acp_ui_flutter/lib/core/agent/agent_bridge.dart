import 'dart:async';
import 'dart:convert';

import '../transport/acp_transport.dart';
import '../../data/models/agent.dart';

/// Agent Bridge - manages communication with a single agent
class AgentBridge {
  final Agent agent;
  final AcpTransport transport;
  final StreamController<AgentMessage> _messageController = StreamController<AgentMessage>.broadcast();

  AgentBridge({
    required this.agent,
    required this.transport,
  });

  /// Connect to agent
  Future<void> connect() async {
    await transport.connect();
  }

  /// Disconnect from agent
  Future<void> disconnect() async {
    await transport.disconnect();
    await _messageController.close();
  }

  /// Send message to agent
  Future<void> sendMessage(String content) async {
    final message = AgentMessage(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      role: MessageRole.user,
      content: content,
      timestamp: DateTime.now(),
    );

    final payload = {
      'type': 'message',
      'agent_id': agent.id,
      'message': message.toJson(),
    };

    await transport.send(jsonEncode(payload));
  }

  /// Receive messages from agent
  Stream<AgentMessage> receiveMessages() {
    return _messageController.stream;
  }

  /// Handle incoming data from transport
  void handleIncomingData(String data) {
    try {
      final json = jsonDecode(data);

      if (json['type'] == 'message') {
        final message = AgentMessage.fromJson(json['message']);
        _messageController.add(message);
      } else if (json['type'] == 'tool_use') {
        // Handle tool use event
        final event = AgentEvent.fromJson(json);
        _messageController.add(AgentMessage(
          id: event.id,
          role: MessageRole.tool,
          content: jsonEncode(json),
          timestamp: DateTime.now(),
        ));
      }
    } catch (e) {
      // Handle parse error
      _messageController.addError(Exception('Failed to parse message: $e'));
    }
  }

  /// Check if agent is connected
  bool isConnected() => transport.isConnected();

  /// Start listening to transport
  void startListening() {
    transport.receive().listen(handleIncomingData);
  }
}

/// Agent message model
class AgentMessage {
  final String id;
  final MessageRole role;
  final String content;
  final DateTime timestamp;

  const AgentMessage({
    required this.id,
    required this.role,
    required this.content,
    required this.timestamp,
  });

  Map<String, dynamic> toJson() => {
    'id': id,
    'role': role.name,
    'content': content,
    'timestamp': timestamp.toIso8601String(),
  };

  factory AgentMessage.fromJson(Map<String, dynamic> json) => AgentMessage(
    id: json['id'] as String,
    role: MessageRole.values.firstWhere((e) => e.name == json['role']),
    content: json['content'] as String,
    timestamp: DateTime.parse(json['timestamp'] as String),
  );
}

/// Agent event model
class AgentEvent {
  final String id;
  final String type;
  final String agentId;
  final DateTime timestamp;

  const AgentEvent({
    required this.id,
    required this.type,
    required this.agentId,
    required this.timestamp,
  });

  factory AgentEvent.fromJson(Map<String, dynamic> json) => AgentEvent(
    id: json['id'] as String,
    type: json['type'] as String,
    agentId: json['agent_id'] as String,
    timestamp: DateTime.now(),
  );
}