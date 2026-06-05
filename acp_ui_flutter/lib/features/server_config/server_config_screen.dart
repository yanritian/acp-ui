import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/services/server_config_service.dart';
import '../../data/services/websocket_service.dart';

/// Server connection settings screen.
/// Allows users to configure the WebSocket server URL,
/// view connection status, and manage authentication.
class ServerConfigScreen extends ConsumerStatefulWidget {
  const ServerConfigScreen({super.key});

  @override
  ConsumerState<ServerConfigScreen> createState() => _ServerConfigScreenState();
}

class _ServerConfigScreenState extends ConsumerState<ServerConfigScreen> {
  late TextEditingController _urlController;
  late TextEditingController _tokenController;
  bool _isTesting = false;
  String? _testResult;
  bool _testSuccess = false;

  @override
  void initState() {
    super.initState();
    final config = ref.read(serverConfigProvider);
    _urlController = TextEditingController(text: config.serverUrl);
    _tokenController = TextEditingController(text: config.authToken ?? '');
  }

  @override
  void dispose() {
    _urlController.dispose();
    _tokenController.dispose();
    super.dispose();
  }

  Future<void> _testConnection() async {
    setState(() {
      _isTesting = true;
      _testResult = null;
    });

    try {
      final url = _urlController.text.trim();
      if (url.isEmpty) {
        setState(() {
          _isTesting = false;
          _testResult = 'URL cannot be empty';
          _testSuccess = false;
        });
        return;
      }

      // Create a temporary WebSocket channel to test connectivity
      final testService = WebSocketService(
        url: url,
        authToken: _tokenController.text.trim().isNotEmpty
            ? _tokenController.text.trim()
            : null,
      );
      await testService.connect();

      // Give it a brief moment to establish the connection
      await Future.delayed(const Duration(milliseconds: 800));

      if (testService.isConnected()) {
        await testService.disconnect();
        setState(() {
          _isTesting = false;
          _testResult = 'Connection successful';
          _testSuccess = true;
        });
      } else {
        setState(() {
          _isTesting = false;
          _testResult = 'Could not establish connection';
          _testSuccess = false;
        });
      }
    } catch (e) {
      setState(() {
        _isTesting = false;
        _testResult = 'Connection failed: $e';
        _testSuccess = false;
      });
    }
  }

  Future<void> _saveAndConnect() async {
    final url = _urlController.text.trim();
    if (url.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Server URL cannot be empty')),
      );
      return;
    }

    final configNotifier = ref.read(serverConfigProvider.notifier);
    await configNotifier.setServerUrl(url);

    final token = _tokenController.text.trim();
    await configNotifier.setAuthToken(token.isNotEmpty ? token : null);

    // Reconnect with new URL
    final wsService = ref.read(webSocketServiceProvider);
    await wsService.disconnect();
    wsService.configureUrl(url);
    if (token.isNotEmpty) wsService.setAuthToken(token);
    try {
      await wsService.connect();
      ref.read(connectionStatusProvider.notifier).state = true;
    } catch (_) {
      ref.read(connectionStatusProvider.notifier).state = false;
    }

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Saved and connecting to $url'),
          backgroundColor: Theme.of(context).colorScheme.primary,
        ),
      );
      Navigator.of(context).pop();
    }
  }

  Future<void> _resetToDefault() async {
    final configNotifier = ref.read(serverConfigProvider.notifier);
    await configNotifier.resetToDefaults();
    final defaultUrl = ServerConfig.defaultUrl;
    _urlController.text = defaultUrl;
    _tokenController.clear();
    setState(() {
      _testResult = null;
    });
  }

  void _showQrScannerScaffold() {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Scan QR Code'),
        content: SizedBox(
          height: 260,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.qr_code_scanner_rounded,
                size: 100,
                color: Theme.of(context).colorScheme.outline,
              ),
              const SizedBox(height: 16),
              Text(
                'Camera-based QR scanning\nwill be available in a future release.',
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
              ),
              const SizedBox(height: 16),
              // Manual entry fallback inside the dialog
              TextField(
                decoration: InputDecoration(
                  hintText: 'Or paste server URL here...',
                  prefixIcon: const Icon(Icons.link_rounded),
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(12),
                  ),
                ),
                onSubmitted: (value) {
                  if (value.trim().isNotEmpty) {
                    _urlController.text = value.trim();
                    Navigator.of(ctx).pop();
                  }
                },
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(ctx).pop(),
            child: const Text('Cancel'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final config = ref.watch(serverConfigProvider);
    final isConnected = ref.watch(connectionStatusProvider);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Server Connection'),
        actions: [
          IconButton(
            icon: const Icon(Icons.restore_rounded),
            tooltip: 'Reset to defaults',
            onPressed: _resetToDefault,
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Connection status banner
            _buildStatusBanner(isConnected, theme),
            const SizedBox(height: 24),

            // Server URL section
            Text(
              'Server Address',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Enter the WebSocket server URL. The default depends on your platform.',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _urlController,
              decoration: InputDecoration(
                labelText: 'Server URL',
                hintText: ServerConfig.defaultUrl,
                prefixIcon: const Icon(Icons.dns_rounded),
                suffixIcon: IconButton(
                  icon: const Icon(Icons.qr_code_scanner_rounded),
                  tooltip: 'Scan QR code',
                  onPressed: _showQrScannerScaffold,
                ),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
              ),
            ),
            const SizedBox(height: 8),
            // Platform hint
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.5),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  Icon(Icons.info_outline_rounded,
                      size: 16, color: theme.colorScheme.onSurfaceVariant),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Default: ${ServerConfig.defaultUrl}  '
                      '(10.0.2.2 for Android emulator, localhost for desktop/web)',
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),

            // Auth token section
            Text(
              'Authentication',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            TextField(
              controller: _tokenController,
              decoration: InputDecoration(
                labelText: 'Auth Token (optional)',
                hintText: 'Paste your access token',
                prefixIcon: const Icon(Icons.key_rounded),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
              ),
              obscureText: true,
            ),
            const SizedBox(height: 24),

            // Advanced settings
            Text(
              'Advanced',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            Card(
              child: Column(
                children: [
                  SwitchListTile(
                    title: const Text('Auto-connect on launch'),
                    subtitle: const Text('Automatically connect when the app starts'),
                    value: config.autoConnect,
                    onChanged: (v) =>
                        ref.read(serverConfigProvider.notifier).setAutoConnect(v),
                  ),
                  const Divider(height: 1),
                  ListTile(
                    title: const Text('Reconnect attempts'),
                    subtitle: Text('${config.reconnectAttempts} attempts'),
                    trailing: DropdownButton<int>(
                      value: config.reconnectAttempts,
                      underline: const SizedBox(),
                      items: [1, 3, 5, 10]
                          .map((n) =>
                              DropdownMenuItem(value: n, child: Text('$n')))
                          .toList(),
                      onChanged: (v) {
                        if (v != null) {
                          ref
                              .read(serverConfigProvider.notifier)
                              .setReconnectAttempts(v);
                        }
                      },
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),

            // Test connection result
            if (_testResult != null)
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(12),
                margin: const EdgeInsets.only(bottom: 16),
                decoration: BoxDecoration(
                  color: _testSuccess
                      ? Colors.green.withValues(alpha: 0.1)
                      : Colors.red.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(12),
                  border: Border.all(
                    color: _testSuccess
                        ? Colors.green.withValues(alpha: 0.4)
                        : Colors.red.withValues(alpha: 0.4),
                  ),
                ),
                child: Row(
                  children: [
                    Icon(
                      _testSuccess
                          ? Icons.check_circle_rounded
                          : Icons.error_rounded,
                      color: _testSuccess ? Colors.green : Colors.red,
                      size: 20,
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        _testResult!,
                        style: TextStyle(
                          color: _testSuccess ? Colors.green.shade800 : Colors.red.shade800,
                          fontSize: 13,
                        ),
                      ),
                    ),
                  ],
                ),
              ),

            // Action buttons
            Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    icon: _isTesting
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.wifi_find_rounded),
                    label: Text(_isTesting ? 'Testing...' : 'Test Connection'),
                    onPressed: _isTesting ? null : _testConnection,
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(vertical: 14),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: ElevatedButton.icon(
                    icon: const Icon(Icons.save_rounded),
                    label: const Text('Save & Connect'),
                    onPressed: _saveAndConnect,
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(vertical: 14),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(12),
                      ),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 32),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusBanner(bool isConnected, ThemeData theme) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: isConnected
            ? Colors.green.withValues(alpha: 0.1)
            : Colors.orange.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(
          color: isConnected
              ? Colors.green.withValues(alpha: 0.3)
              : Colors.orange.withValues(alpha: 0.3),
        ),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: isConnected
                  ? Colors.green.withValues(alpha: 0.2)
                  : Colors.orange.withValues(alpha: 0.2),
              shape: BoxShape.circle,
            ),
            child: Icon(
              isConnected ? Icons.cloud_done_rounded : Icons.cloud_off_rounded,
              color: isConnected ? Colors.green.shade700 : Colors.orange.shade700,
              size: 28,
            ),
          ),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  isConnected ? 'Connected' : 'Disconnected',
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w600,
                    color: isConnected
                        ? Colors.green.shade800
                        : Colors.orange.shade800,
                  ),
                ),
                Text(
                  _urlController.text.trim().isNotEmpty
                      ? _urlController.text.trim()
                      : 'No server configured',
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
