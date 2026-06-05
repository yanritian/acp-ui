import 'dart:io' show Platform;
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Server configuration model
class ServerConfig {
  final String serverUrl;
  final String? authToken;
  final bool autoConnect;
  final int reconnectAttempts;

  const ServerConfig({
    required this.serverUrl,
    this.authToken,
    this.autoConnect = true,
    this.reconnectAttempts = 3,
  });

  /// Default URL based on current platform
  static String get defaultUrl {
    if (kIsWeb) return 'ws://localhost:1421';
    try {
      if (Platform.isAndroid) return 'ws://10.0.2.2:1421';
      if (Platform.isIOS) return 'ws://localhost:1421';
      // Desktop (Windows, macOS, Linux)
      return 'ws://localhost:1421';
    } catch (_) {
      return 'ws://localhost:1421';
    }
  }

  ServerConfig copyWith({
    String? serverUrl,
    String? authToken,
    bool? autoConnect,
    int? reconnectAttempts,
  }) {
    return ServerConfig(
      serverUrl: serverUrl ?? this.serverUrl,
      authToken: authToken ?? this.authToken,
      autoConnect: autoConnect ?? this.autoConnect,
      reconnectAttempts: reconnectAttempts ?? this.reconnectAttempts,
    );
  }
}

/// State notifier for server configuration
class ServerConfigNotifier extends StateNotifier<ServerConfig> {
  final SharedPreferences _prefs;

  static const _keyServerUrl = 'server_url';
  static const _keyAuthToken = 'server_auth_token';
  static const _keyAutoConnect = 'server_auto_connect';
  static const _keyReconnectAttempts = 'server_reconnect_attempts';

  ServerConfigNotifier(this._prefs)
      : super(ServerConfig(
          serverUrl: _prefs.getString(_keyServerUrl) ?? ServerConfig.defaultUrl,
          authToken: _prefs.getString(_keyAuthToken),
          autoConnect: _prefs.getBool(_keyAutoConnect) ?? true,
          reconnectAttempts: _prefs.getInt(_keyReconnectAttempts) ?? 3,
        ));

  /// Update server URL and persist
  Future<void> setServerUrl(String url) async {
    await _prefs.setString(_keyServerUrl, url);
    state = state.copyWith(serverUrl: url);
  }

  /// Update auth token and persist
  Future<void> setAuthToken(String? token) async {
    if (token == null) {
      await _prefs.remove(_keyAuthToken);
    } else {
      await _prefs.setString(_keyAuthToken, token);
    }
    state = state.copyWith(authToken: token);
  }

  /// Update auto-connect setting
  Future<void> setAutoConnect(bool value) async {
    await _prefs.setBool(_keyAutoConnect, value);
    state = state.copyWith(autoConnect: value);
  }

  /// Update reconnect attempts
  Future<void> setReconnectAttempts(int value) async {
    await _prefs.setInt(_keyReconnectAttempts, value);
    state = state.copyWith(reconnectAttempts: value);
  }

  /// Reset to defaults
  Future<void> resetToDefaults() async {
    await _prefs.remove(_keyServerUrl);
    await _prefs.remove(_keyAuthToken);
    await _prefs.remove(_keyAutoConnect);
    await _prefs.remove(_keyReconnectAttempts);
    state = ServerConfig(serverUrl: ServerConfig.defaultUrl);
  }
}

/// Provider for SharedPreferences (shared across the app)
final sharedPreferencesProvider = Provider<SharedPreferences>((ref) {
  throw UnimplementedError('Must be overridden in ProviderScope');
});

/// Provider for the server configuration notifier
final serverConfigProvider =
    StateNotifierProvider<ServerConfigNotifier, ServerConfig>((ref) {
  final prefs = ref.watch(sharedPreferencesProvider);
  return ServerConfigNotifier(prefs);
});
