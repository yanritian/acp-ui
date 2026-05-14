import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Config Service - manages application configuration
class ConfigService {
  final SharedPreferences _prefs;

  ConfigService(this._prefs);

  // Keys
  static const String _keyBaseUrl = 'base_url';
  static const String _keyDefaultAgent = 'default_agent';
  static const String _keyPermissionMode = 'permission_mode';
  static const String _keyMaxContextTokens = 'max_context_tokens';
  static const String _keyThemeMode = 'theme_mode';

  /// Get base URL for WebSocket connection
  String get baseUrl => _prefs.getString(_keyBaseUrl) ?? 'ws://localhost:8080';

  /// Set base URL
  Future<void> setBaseUrl(String url) async {
    await _prefs.setString(_keyBaseUrl, url);
  }

  /// Get default agent ID
  String get defaultAgent => _prefs.getString(_keyDefaultAgent) ?? 'general-purpose';

  /// Set default agent
  Future<void> setDefaultAgent(String agentId) async {
    await _prefs.setString(_keyDefaultAgent, agentId);
  }

  /// Get permission mode
  String get permissionMode => _prefs.getString(_keyPermissionMode) ?? 'Allow';

  /// Set permission mode
  Future<void> setPermissionMode(String mode) async {
    await _prefs.setString(_keyPermissionMode, mode);
  }

  /// Get max context tokens
  int get maxContextTokens => _prefs.getInt(_keyMaxContextTokens) ?? 200000;

  /// Set max context tokens
  Future<void> setMaxContextTokens(int tokens) async {
    await _prefs.setInt(_keyMaxContextTokens, tokens);
  }

  /// Get theme mode (light/dark/system)
  String get themeMode => _prefs.getString(_keyThemeMode) ?? 'system';

  /// Set theme mode
  Future<void> setThemeMode(String mode) async {
    await _prefs.setString(_keyThemeMode, mode);
  }

  /// Clear all preferences
  Future<void> clear() async {
    await _prefs.clear();
  }
}

/// Config Service Provider
final configServiceProvider = FutureProvider<ConfigService>((ref) async {
  final prefs = await SharedPreferences.getInstance();
  return ConfigService(prefs);
});