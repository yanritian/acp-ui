import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import 'acp_transport.dart';

/// WebSocket Transport for remote agent communication
class WebSocketTransport implements AcpTransport {
  final String url;
  WebSocketChannel? _channel;
  final StreamController<String> _receiveController = StreamController<String>.broadcast();
  bool _isConnected = false;

  WebSocketTransport({required this.url});

  @override
  Future<void> connect() async {
    if (_isConnected) return;

    try {
      _channel = WebSocketChannel.connect(Uri.parse(url));
      _isConnected = true;

      // Listen to incoming messages
      _channel!.stream.listen(
        (data) {
          _receiveController.add(data.toString());
        },
        onError: (error) {
          _receiveController.addError(error);
          _isConnected = false;
        },
        onDone: () {
          _isConnected = false;
        },
      );
    } catch (e) {
      _isConnected = false;
      throw Exception('Failed to connect to WebSocket: $e');
    }
  }

  @override
  Future<void> disconnect() async {
    if (!_isConnected) return;

    await _channel?.sink.close();
    _isConnected = false;
  }

  @override
  Future<void> send(String message) async {
    if (!_isConnected || _channel == null) {
      throw Exception('WebSocket not connected');
    }

    _channel!.sink.add(message);
  }

  @override
  Stream<String> receive() {
    return _receiveController.stream;
  }

  @override
  bool isConnected() => _isConnected;

  @override
  String transportType() => 'WebSocket';

  /// Send JSON-RPC request
  Future<Map<String, dynamic>> sendRpc(String method, Map<String, dynamic> params, {String? id}) async {
    final requestId = id ?? DateTime.now().millisecondsSinceEpoch.toString();

    final request = {
      'jsonrpc': '2.0',
      'method': method,
      'params': params,
      'id': requestId,
    };

    await send(jsonEncode(request));

    // Wait for response with matching id
    final response = await receive()
        .where((msg) {
          try {
            final json = jsonDecode(msg);
            return json['id'] == requestId;
          } catch (_) {
            return false;
          }
        })
        .first;

    return jsonDecode(response) as Map<String, dynamic>;
  }
}