// Push Notification System - async approval alerts and cross-platform push.

export interface PushNotification {
  id: string;
  type: 'task_complete' | 'approval_needed' | 'error' | 'progress';
  title: string;
  message: string;
  timestamp: number;
  data?: Record<string, unknown>;
}

export interface NotificationSubscription {
  platform: 'web_push' | 'apns' | 'fcm' | 'websocket';
  endpoint: string;
  keys?: { p256dh: string; auth: string };
}

type NotificationListener = (notification: PushNotification) => void;

/**
 * In-memory push notification service.
 *
 * On Tauri desktop builds, `showDesktopNotification` delegates to the native
 * notification plugin via `@tauri-apps/plugin-notification`.
 * On web builds it falls back to the standard Notification API.
 */
export class PushService {
  private subscriptions: Map<string, NotificationSubscription> = new Map();
  private pending: Map<string, PushNotification> = new Map();
  private listeners: Set<NotificationListener> = new Set();

  // ----- Subscription management -----

  subscribe(platform: string, subscription: NotificationSubscription): void {
    this.subscriptions.set(platform, subscription);
  }

  unsubscribe(platform: string): void {
    this.subscriptions.delete(platform);
  }

  // ----- Notification lifecycle -----

  send(notification: PushNotification): void {
    const enriched: PushNotification = {
      ...notification,
      timestamp: notification.timestamp || Date.now(),
      id: notification.id || crypto.randomUUID(),
    };
    this.pending.set(enriched.id, enriched);
    this.notifyListeners(enriched);
  }

  getPending(): PushNotification[] {
    return Array.from(this.pending.values()).sort(
      (a, b) => b.timestamp - a.timestamp
    );
  }

  dismiss(id: string): void {
    this.pending.delete(id);
  }

  clearAll(): void {
    this.pending.clear();
  }

  // ----- Listener API -----

  onNotification(listener: NotificationListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notifyListeners(notification: PushNotification): void {
    for (const listener of this.listeners) {
      try {
        listener(notification);
      } catch {
        // Listener threw — log to telemetry in production, skip in dev
      }
    }
  }

  // ----- Web Push -----

  async requestWebPushPermission(): Promise<boolean> {
    if (typeof Notification === 'undefined') {
      return false;
    }

    if (Notification.permission === 'granted') {
      return true;
    }

    if (Notification.permission === 'denied') {
      return false;
    }

    const permission = await Notification.requestPermission();
    return permission === 'granted';
  }

  // ----- Desktop notification (Tauri integration) -----

  async showDesktopNotification(notification: PushNotification): Promise<void> {
    // Try Tauri notification API first (desktop builds only).
    // We use the global __TAURI__ object rather than a static import so that
    // this module compiles even when the plugin package is not installed.
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const tauri = (globalThis as any).__TAURI__;
      if (tauri?.notification) {
        const notifMod = tauri.notification;
        // Check permission
        if (typeof notifMod.isPermissionGranted === 'function') {
          const granted = await notifMod.isPermissionGranted();
          if (!granted && typeof notifMod.requestPermission === 'function') {
            const result = await notifMod.requestPermission();
            if (result === 'denied') {
              return;
            }
          }
        }
        if (typeof notifMod.sendNotification === 'function') {
          notifMod.sendNotification({
            title: notification.title,
            body: notification.message,
          });
          return;
        }
      }
    } catch {
      // Tauri notification not available -- fall through.
    }

    // Fallback: browser Notification API.
    if (typeof Notification !== 'undefined' && Notification.permission === 'granted') {
      new Notification(notification.title, {
        body: notification.message,
        icon: this.selectIcon(notification.type),
      });
    }
  }

  private selectIcon(type: PushNotification['type']): string | undefined {
    const iconMap: Record<PushNotification['type'], string> = {
      task_complete: '/icons/task-complete.png',
      approval_needed: '/icons/approval-needed.png',
      error: '/icons/error.png',
      progress: '/icons/progress.png',
    };
    return iconMap[type];
  }
}

// Singleton instance for application-wide use.
export const pushService = new PushService();
