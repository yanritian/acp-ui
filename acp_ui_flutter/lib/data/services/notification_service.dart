import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Notification Service - manages in-app notifications
class NotificationService {
  final List<AppNotification> _notifications = [];
  final StreamController<AppNotification> _notificationController = StreamController<AppNotification>.broadcast();

  /// Show notification
  void show(String title, String message, NotificationType type) {
    final notification = AppNotification(
      id: DateTime.now().millisecondsSinceEpoch.toString(),
      title: title,
      message: message,
      type: type,
      timestamp: DateTime.now(),
    );

    _notifications.add(notification);
    _notificationController.add(notification);
  }

  /// Show info notification
  void info(String title, String message) => show(title, message, NotificationType.info);

  /// Show success notification
  void success(String title, String message) => show(title, message, NotificationType.success);

  /// Show warning notification
  void warning(String title, String message) => show(title, message, NotificationType.warning);

  /// Show error notification
  void error(String title, String message) => show(title, message, NotificationType.error);

  /// Get all notifications
  List<AppNotification> getNotifications() => List.unmodifiable(_notifications);

  /// Clear notifications
  void clear() {
    _notifications.clear();
  }

  /// Get notification stream
  Stream<AppNotification> get stream => _notificationController.stream;
}

/// Notification Type
enum NotificationType {
  info,
  success,
  warning,
  error,
}

/// App Notification model
class AppNotification {
  final String id;
  final String title;
  final String message;
  final NotificationType type;
  final DateTime timestamp;

  AppNotification({
    required this.id,
    required this.title,
    required this.message,
    required this.type,
    required this.timestamp,
  });
}

/// Notification Service Provider
final notificationServiceProvider = Provider<NotificationService>((ref) {
  return NotificationService();
});

/// Notifications List Provider
final notificationsProvider = Provider<List<AppNotification>>((ref) {
  final service = ref.watch(notificationServiceProvider);
  return service.getNotifications();
});

/// Notification Stream Provider
final notificationStreamProvider = StreamProvider<AppNotification>((ref) {
  final service = ref.watch(notificationServiceProvider);
  return service.stream;
});