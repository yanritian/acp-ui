import 'dart:async';
import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'server_config_service.dart';

/// WebSocket Service Provider - connects to Rust Tauri Backend
/// Uses ServerConfigProvider for dynamic server URL resolution.
final webSocketServiceProvider = Provider<WebSocketService>((ref) {
  final serverConfig = ref.watch(serverConfigProvider);
  return WebSocketService(url: serverConfig.serverUrl, authToken: serverConfig.authToken);
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

  /// Pending request completers for request/response pattern
  final Map<String, Completer<Map<String, dynamic>>> _pendingRequests = {};

  WebSocketService({required String url, String? authToken})
      : _url = url,
        _authToken = authToken;

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
          _failAllPending('WebSocket error: $error');
        },
        onDone: () {
          _isConnected = false;
          _failAllPending('WebSocket connection closed');
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
      _failAllPending('Connection failed: $e');
      throw Exception('WebSocket connection failed: $e');
    }
  }

  /// Handle incoming messages
  void _handleMessage(String data) {
    try {
      final msg = jsonDecode(data) as Map<String, dynamic>;

      // Resolve pending request if the message has a matching 'id'
      final msgId = msg['id'] as String?;
      if (msgId != null && _pendingRequests.containsKey(msgId)) {
        final completer = _pendingRequests.remove(msgId)!;
        completer.complete(msg);
        return;
      }

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

  /// Fail all pending requests (on disconnect / error)
  void _failAllPending(String reason) {
    for (final completer in _pendingRequests.values) {
      if (!completer.isCompleted) {
        completer.completeError(Exception(reason));
      }
    }
    _pendingRequests.clear();
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
    _failAllPending('Disconnected by user');
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

  /// Send a request and wait for the response (matched by request id).
  /// Returns the full response message as a Map.
  Future<Map<String, dynamic>> sendRequest(
    String command,
    Map<String, dynamic> payload,
  ) async {
    final id = _generateRequestId();
    final completer = Completer<Map<String, dynamic>>();
    _pendingRequests[id] = completer;

    await sendJson({
      'id': id,
      'type': 'request',
      'command': command,
      'payload': payload,
    });

    // Timeout after 30 seconds to avoid leaked completers
    return completer.future.timeout(
      const Duration(seconds: 30),
      onTimeout: () {
        _pendingRequests.remove(id);
        throw TimeoutException('Request $command timed out');
      },
    );
  }

  /// Generic proxy command - forwards commands through the WebSocket
  /// to the Tauri backend's pluggable architecture.
  ///
  /// Example:
  ///   await proxyCommand('plugin_list', {});
  ///   await proxyCommand('swarm_status', {'verbose': true});
  Future<Map<String, dynamic>> proxyCommand(
    String command,
    Map<String, dynamic> params,
  ) async {
    return await sendRequest('proxy', {
      'command': command,
      'params': params,
    });
  }

  /// Generate unique request ID
  String _generateRequestId() {
    return 'req-${DateTime.now().millisecondsSinceEpoch}';
  }

  /// Initialize Executive Agent
  Future<void> initExecutiveAgent(String workspace) async {
    await sendJson({
      'id': _generateRequestId(),
      'type': 'request',
      'command': 'init_executive_agent',
      'payload': {'workspace': workspace},
    });
  }

  /// Execute development task
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
