/// ACP Transport Interface
///
/// Abstract interface for agent communication.
/// Supports WebSocket, HTTP, and StdIO transports.

abstract class AcpTransport {
  /// Connect to agent
  Future<void> connect();

  /// Disconnect from agent
  Future<void> disconnect();

  /// Send message to agent
  Future<void> send(String message);

  /// Receive message from agent (stream)
  Stream<String> receive();

  /// Check if connected
  bool isConnected();

  /// Get transport type
  String transportType();
}