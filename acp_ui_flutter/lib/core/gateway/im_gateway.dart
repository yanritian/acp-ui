import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

/// IM Gateway Interface
abstract class ImGateway {
  /// Send text message
  Future<String> sendText(String receiveId, String text);

  /// Send image message
  Future<String> sendImage(String receiveId, List<int> imageData);

  /// Send card message
  Future<String> sendCard(String receiveId, Map<String, dynamic> card);

  /// Get gateway type
  String gatewayType();
}

/// Feishu Gateway
class FeishuGateway implements ImGateway {
  final String appId;
  final String appSecret;
  final String baseUrl = 'https://open.feishu.cn/open-apis';

  String? _tenantToken;
  DateTime? _tokenExpiry;

  FeishuGateway({
    required this.appId,
    required this.appSecret,
  });

  /// Get tenant access token (auto-refresh)
  Future<String> _getToken() async {
    if (_tenantToken != null && _tokenExpiry != null) {
      if (DateTime.now().isBefore(_tokenExpiry!)) {
        return _tenantToken!;
      }
    }

    // Refresh token
    final response = await http.post(
      Uri.parse('$baseUrl/auth/v3/tenant_access_token/internal'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'app_id': appId,
        'app_secret': appSecret,
      }),
    );

    final json = jsonDecode(response.body);

    if (json['code'] != 0) {
      throw Exception('Failed to get token: ${json['msg']}');
    }

    _tenantToken = json['tenant_access_token'] as String;
    final expireSeconds = json['expire'] as int;
    _tokenExpiry = DateTime.now().add(Duration(seconds: expireSeconds - 300));

    return _tenantToken!;
  }

  @override
  Future<String> sendText(String receiveId, String text) async {
    final token = await _getToken();

    final response = await http.post(
      Uri.parse('$baseUrl/im/v1/messages?receive_id_type=open_id'),
      headers: {
        'Authorization': 'Bearer $token',
        'Content-Type': 'application/json',
      },
      body: jsonEncode({
        'receive_id': receiveId,
        'msg_type': 'text',
        'content': jsonEncode({'text': text}),
      }),
    );

    final json = jsonDecode(response.body);

    if (json['code'] != 0) {
      throw Exception('Failed to send message: ${json['msg']}');
    }

    return json['data']['message_id'] as String;
  }

  @override
  Future<String> sendImage(String receiveId, List<int> imageData) async {
    final token = await _getToken();

    // Upload image first
    final uploadResponse = await http.post(
      Uri.parse('$baseUrl/im/v1/images'),
      headers: {'Authorization': 'Bearer $token'},
      body: imageData,
    );

    final uploadJson = jsonDecode(uploadResponse.body);
    if (uploadJson['code'] != 0) {
      throw Exception('Failed to upload image: ${uploadJson['msg']}');
    }

    final imageKey = uploadJson['data']['image_key'] as String;

    // Send image message
    return sendText(receiveId, '[Image: $imageKey]');
  }

  @override
  Future<String> sendCard(String receiveId, Map<String, dynamic> card) async {
    final token = await _getToken();

    final response = await http.post(
      Uri.parse('$baseUrl/im/v1/messages?receive_id_type=open_id'),
      headers: {
        'Authorization': 'Bearer $token',
        'Content-Type': 'application/json',
      },
      body: jsonEncode({
        'receive_id': receiveId,
        'msg_type': 'interactive',
        'content': jsonEncode({
          'type': 'card',
          'data': card,
        }),
      }),
    );

    final json = jsonDecode(response.body);

    if (json['code'] != 0) {
      throw Exception('Failed to send card: ${json['msg']}');
    }

    return json['data']['message_id'] as String;
  }

  @override
  String gatewayType() => 'Feishu';
}

/// Telegram Gateway (via Bot API)
class TelegramGateway implements ImGateway {
  final String botToken;
  final String baseUrl = 'https://api.telegram.org';

  TelegramGateway({required this.botToken});

  @override
  Future<String> sendText(String receiveId, String text) async {
    final response = await http.post(
      Uri.parse('$baseUrl/bot$botToken/sendMessage'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'chat_id': receiveId,
        'text': text,
      }),
    );

    final json = jsonDecode(response.body);

    if (!json['ok']) {
      throw Exception('Failed to send message: ${json['description']}');
    }

    return json['result']['message_id'].toString();
  }

  @override
  Future<String> sendImage(String receiveId, List<int> imageData) async {
    // Telegram requires photo URL, not direct upload
    throw UnimplementedError('Telegram requires photo URL');
  }

  @override
  Future<String> sendCard(String receiveId, Map<String, dynamic> card) async {
    // Telegram doesn't support cards directly
    // Convert to formatted text
    final text = _formatCardAsText(card);
    return sendText(receiveId, text);
  }

  String _formatCardAsText(Map<String, dynamic> card) {
    final buffer = StringBuffer();
    buffer.writeln('📊 ${card['title'] ?? 'Card'}');
    if (card['content'] != null) {
      buffer.writeln(card['content']);
    }
    return buffer.toString();
  }

  @override
  String gatewayType() => 'Telegram';
}

/// IM Gateway Manager
class ImGatewayManager {
  final Map<String, ImGateway> _gateways = {};
  final StreamController<ImMessage> _messageController = StreamController<ImMessage>.broadcast();

  /// Register a gateway
  void registerGateway(String name, ImGateway gateway) {
    _gateways[name] = gateway;
  }

  /// Get gateway by name
  ImGateway? getGateway(String name) => _gateways[name];

  /// Send message through gateway
  Future<String> sendMessage(String gatewayName, String receiveId, String text) async {
    final gateway = _gateways[gatewayName];
    if (gateway == null) {
      throw Exception('Gateway $gatewayName not found');
    }

    final messageId = await gateway.sendText(receiveId, text);

    // Record message event
    _messageController.add(ImMessage(
      gatewayType: gateway.gatewayType(),
      receiveId: receiveId,
      content: text,
      messageId: messageId,
      timestamp: DateTime.now(),
    ));

    return messageId;
  }

  /// Broadcast message to all gateways
  Future<void> broadcastMessage(String text, List<String> receiveIds) async {
    for (final gateway in _gateways.values) {
      for (final receiveId in receiveIds) {
        await gateway.sendText(receiveId, text);
      }
    }
  }

  /// Get registered gateway types
  List<String> getGatewayTypes() {
    return _gateways.values.map((g) => g.gatewayType()).toList();
  }

  /// Get message stream
  Stream<ImMessage> get messageStream => _messageController.stream;
}

/// IM Message model
class ImMessage {
  final String gatewayType;
  final String receiveId;
  final String content;
  final String messageId;
  final DateTime timestamp;

  ImMessage({
    required this.gatewayType,
    required this.receiveId,
    required this.content,
    required this.messageId,
    required this.timestamp,
  });
}