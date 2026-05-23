import 'dart:async';
import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

/// WebSocket Service Provider - connects to Rust Tauri Backend
final webSocketServiceProvider = Provider<WebSocketService>((ref) {
  // Default to WebSocket server port (1421)
  // Use 10.0.2.2 for Android emulator to access host machine
  return WebSocketService(url: 'ws://10.0.2.2:1421');
});

/// Connection Status Provider
final connectionStatusProvider = StateProvider<bool>((ref) => false);

/// Agent Messages Stream Provider
final agentMessagesStreamProvider = StreamProvider<Map<String, dynamic>>((ref) {
  final service = ref.watch(webSocketServiceProvider);
  return service.messageStream.map((data) {
    try {
      return jsonDecode(data) as Map<String, dynamic>;
    } catch (e) {
      return {'type': 'error', 'message': 'Parse error'};
    }
  });
});

/// WebSocket Service - manages WebSocket connections to Rust backend
class WebSocketService {
  WebSocketChannel? _channel;
  final StreamController<String> _messageController = StreamController<String>.broadcast();
  String _url;
  bool _isConnected = false;
  String? _authToken;
  String? _clientId;

  WebSocketService({required String url}) : _url = url;

  Stream<String> get messageStream => _messageController.stream;
  bool isConnected() => _isConnected;
  bool get isConnectedGetter => _isConnected;
  String? get clientId => _clientId;
  String get url => _url;

  /// Set authentication token (from QR code)
  void setAuthToken(String token) {
    _authToken = token;
  }

  /// Configure server URL
  void configureUrl(String newUrl) {
    _url = newUrl;
  }

  /// Connect to WebSocket server
  Future<void> connect() async {
    if (_isConnected) return;

    try {
      // Add token to URL if available
      var connectUrl = url;
      if (_authToken != null) {
        connectUrl = '$url?token=$_authToken';
      }

      _channel = WebSocketChannel.connect(Uri.parse(connectUrl));
      _isConnected = true;

      _channel!.stream.listen(
        (data) {
          _messageController.add(data.toString());
          _handleMessage(data.toString());
        },
        onError: (error) {
          print('[WS] Error: $error');
          _isConnected = false;
        },
        onDone: () {
          _isConnected = false;
          print('[WS] Connection closed');
        },
      );

      print('[WS] Connected to $connectUrl');

      // Send auth if token is available
      if (_authToken != null) {
        await authenticate(_authToken!);
      }
    } catch (e) {
      _isConnected = false;
      throw Exception('WebSocket connection failed: $e');
    }
  }

  /// Handle incoming messages
  void _handleMessage(String data) {
    try {
      final msg = jsonDecode(data) as Map<String, dynamic>;

      // Handle auth response
      if (msg['ok'] == true && msg['data'] != null) {
        final data = msg['data'] as Map<String, dynamic>;
        if (data['client_id'] != null) {
          _clientId = data['client_id'] as String;
          print('[WS] Authenticated as client: $_clientId');
        }
      }
    } catch (e) {
      print('[WS] Parse error: $e');
    }
  }

  /// Authenticate with token
  Future<void> authenticate(String token) async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'auth',
      'token': token,
    });
  }

  /// Disconnect from WebSocket server
  Future<void> disconnect() async {
    if (!_isConnected) return;
    await _channel?.sink.close();
    _isConnected = false;
  }

  /// Send raw message
  void send(String message) {
    if (!_isConnected || _channel == null) {
      throw Exception('WebSocket not connected');
    }
    _channel!.sink.add(message);
  }

  /// Send JSON message (JSON-RPC format)
  Future<void> sendJson(Map<String, dynamic> message) async {
    send(jsonEncode(message));
  }

  /// Generate unique request ID
  String _generateRequestId() {
    return 'req-${DateTime.now().millisecondsSinceEpoch}';
  }

  /// Initialize Executive Agent (新功能)
  Future<void> initExecutiveAgent(String workspace) async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'init_executive_agent',
      'payload': {'workspace': workspace},
    });
  }

  /// Execute development task (新功能)
  Future<void> executeDevelopmentTask(String request) async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'execute_development_task',
      'payload': {'request': request},
    });
  }

  /// Get executive agent status
  Future<void> getExecutiveAgentStatus() async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'get_executive_agent_status',
    });
  }

  /// Get generated files
  Future<void> getGeneratedFiles() async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'get_generated_files',
    });
  }

  /// Clear executive agent
  Future<void> clearExecutiveAgent() async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'clear_executive_agent',
    });
  }

  /// Spawn and execute agent from config (Claude Code, Gemini CLI, etc.)
  Future<void> spawnAndExecuteAgent(String agentName, String request, String? workspace) async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'spawn_and_execute_agent',
      'payload': {
        'agent_name': agentName,
        'request': request,
        if (workspace != null) 'workspace': workspace,
      },
    });
  }

  /// Get task history from database
  Future<void> getTaskHistory(int limit, List<String>? status, String? source) async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'get_task_history',
      'payload': {
        'limit': limit,
        if (status != null) 'status': status,
        if (source != null) 'source': source,
      },
    });
  }

  /// Get task statistics
  Future<void> getTaskStatistics() async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'get_task_statistics',
    });
  }

  /// Send user request to agents (legacy format)
  Future<void> sendUserRequest(String content) async {
    await sendJson({
      'type': 'user_request',
      'content': content,
      'timestamp': DateTime.now().toIso8601String(),
    });
  }

  /// Request agent list
  Future<void> requestAgentList() async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'list_agents',
    });
  }

  /// Request agent status
  Future<void> requestAgentStatus() async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'get_status',
    });
  }

  /// Receive messages stream
  Stream<String> get messages => _messageController.stream;

  /// Get connection status
  String get status => _isConnected ? 'connected' : 'disconnected';
}