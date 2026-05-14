import 'dart:async';
import 'package:web_socket_channel/web_socket_channel.dart';

/// WebSocket Service - manages WebSocket connections
class WebSocketService {
  WebSocketChannel? _channel;
  final String url;
  final StreamController<String> _messageController = StreamController<String>.broadcast();
  bool _isConnected = false;

  WebSocketService({required this.url});

  /// Connect to WebSocket server
  Future<void> connect() async {
    if (_isConnected) return;

    try {
      _channel = WebSocketChannel.connect(Uri.parse(url));
      _isConnected = true;

      _channel!.stream.listen(
        (data) {
          _messageController.add(data.toString());
        },
        onError: (error) {
          _messageController.addError(error);
          _isConnected = false;
        },
        onDone: () {
          _isConnected = false;
        },
      );
    } catch (e) {
      throw Exception('WebSocket connection failed: $e');
    }
  }

  /// Disconnect from WebSocket server
  Future<void> disconnect() async {
    if (!_isConnected) return;

    await _channel?.sink.close();
    _isConnected = false;
  }

  /// Send message
  void send(String message) {
    if (!_isConnected || _channel == null) {
      throw Exception('WebSocket not connected');
    }
    _channel!.sink.add(message);
  }

  /// Receive messages stream
  Stream<String> get messages => _messageController.stream;

  /// Check if connected
  bool get isConnected => _isConnected;

  /// Get connection status
  String get status => _isConnected ? 'connected' : 'disconnected';
}