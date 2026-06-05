// WebSocket-based command proxy for invoking Tauri backend commands from Web.
//
// When the app runs in a plain browser (no Tauri runtime), direct `invoke()`
// calls fail. This module bridges that gap by forwarding commands over a
// persistent WebSocket connection to the Tauri backend's built-in WebSocket
// server (default: ws://localhost:1421).
//
// Protocol (JSON-RPC-like):
//   Request:  { id: number, method: string, params: Record<string, unknown> }
//   Response: { id: number, result?: unknown, error?: string }

import { isTauriHost } from '../platform';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const DEFAULT_WS_URL = 'ws://localhost:1421';

/** Resolve the WebSocket URL. Honors `VITE_WS_URL` from Vite env if set. */
function resolveWsUrl(): string {
  if (typeof import.meta !== 'undefined') {
    const env = (import.meta as unknown as { env?: Record<string, string | undefined> }).env;
    if (env?.VITE_WS_URL) return env.VITE_WS_URL;
  }
  return DEFAULT_WS_URL;
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface WsRequest {
  id: number;
  method: string;
  params: Record<string, unknown>;
}

interface WsResponse {
  id: number;
  result?: unknown;
  error?: string;
}

interface PendingCall {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

// ---------------------------------------------------------------------------
// Singleton proxy
// ---------------------------------------------------------------------------

class WsCommandProxy {
  private ws: WebSocket | null = null;
  private nextId = 1;
  private pending = new Map<number, PendingCall>();
  private queue: string[] = [];
  private connecting = false;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempts = 0;
  private closed = false;

  /** Default timeout for a single command (ms). */
  private readonly commandTimeout = 30_000;
  /** Base delay between reconnection attempts (ms). Doubles on each retry. */
  private readonly baseReconnectDelay = 1_000;
  /** Maximum reconnect delay cap (ms). */
  private readonly maxReconnectDelay = 30_000;
  /** Maximum number of reconnection attempts before giving up. */
  private readonly maxReconnectAttempts = 10;

  // -----------------------------------------------------------------------
  // Public API
  // -----------------------------------------------------------------------

  /**
   * Send a command to the Tauri backend and return the result.
   * Mirrors the signature of `@tauri-apps/api/core` `invoke()`.
   */
  async invoke<T = unknown>(
    command: string,
    params?: Record<string, unknown>,
  ): Promise<T> {
    if (isTauriHost()) {
      // Should not happen — callers should short-circuit via isTauriHost()
      // before reaching here, but guard defensively.
      throw new Error(
        `wsCommandProxy.invoke() called inside Tauri host for "${command}". ` +
        `Use invokeOrProxy() instead.`,
      );
    }

    await this.ensureConnected();

    const id = this.nextId++;
    const request: WsRequest = {
      id,
      method: command,
      params: params ?? {},
    };

    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(
          new Error(`WebSocket command "${command}" timed out after ${this.commandTimeout}ms`),
        );
      }, this.commandTimeout);

      this.pending.set(id, {
        resolve: resolve as (v: unknown) => void,
        reject,
        timer,
      });

      const payload = JSON.stringify(request);
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(payload);
      } else {
        // Queue while still connecting (ensureConnected resolved but the
        // socket may have dropped immediately).
        this.queue.push(payload);
      }
    });
  }

  /** Close the connection and cancel all pending calls. */
  dispose(): void {
    this.closed = true;
    this.clearReconnectTimer();

    for (const [id, p] of this.pending) {
      clearTimeout(p.timer);
      p.reject(new Error('WsCommandProxy disposed'));
      this.pending.delete(id);
    }

    if (this.ws) {
      this.ws.onopen = null;
      this.ws.onmessage = null;
      this.ws.onerror = null;
      this.ws.onclose = null;
      if (
        this.ws.readyState === WebSocket.OPEN ||
        this.ws.readyState === WebSocket.CONNECTING
      ) {
        this.ws.close();
      }
      this.ws = null;
    }
  }

  // -----------------------------------------------------------------------
  // Connection management
  // -----------------------------------------------------------------------

  private ensureConnected(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) return Promise.resolve();
    if (this.connecting && this._connectPromise) return this._connectPromise;
    return this.connect();
  }

  private _connectPromise: Promise<void> | null = null;

  private connect(): Promise<void> {
    this.connecting = true;
    this.closed = false;

    this._connectPromise = new Promise<void>((resolve, reject) => {
      const url = resolveWsUrl();
      let ws: WebSocket;
      try {
        ws = new WebSocket(url);
      } catch (err) {
        this.connecting = false;
        reject(err);
        return;
      }

      // Connection timeout: reject if not connected within 5 seconds
      const connectTimeout = setTimeout(() => {
        cleanup();
        this.connecting = false;
        try { ws.close(); } catch { /* ignore */ }
        reject(new Error(`WebSocket connection to ${url} timed out after 5000ms`));
      }, 5000);

      const onOpen = () => {
        clearTimeout(connectTimeout);
        cleanup();
        this.ws = ws;
        this.connecting = false;
        this.reconnectAttempts = 0;
        this.flushQueue();
        resolve();
      };

      const onError = (ev: Event) => {
        clearTimeout(connectTimeout);
        cleanup();
        this.connecting = false;
        try { ws.close(); } catch { /* ignore */ }
        reject(new Error(`WebSocket connection to ${url} failed: ${ev.type}`));
      };

      const cleanup = () => {
        ws.removeEventListener('open', onOpen);
        ws.removeEventListener('error', onError);
        // Attach long-lived handlers once connected.
      };

      ws.addEventListener('open', onOpen);
      ws.addEventListener('error', onError);
    });

    // Attach persistent handlers after the promise settles.
    this._connectPromise.then(
      () => this.attachHandlers(),
      () => {},
    );

    return this._connectPromise;
  }

  private attachHandlers(): void {
    if (!this.ws) return;

    this.ws.onmessage = (ev: MessageEvent) => this.handleMessage(ev);
    this.ws.onerror = () => {
      // The 'close' event always fires after 'error', so reconnect there.
    };
    this.ws.onclose = () => {
      this.ws = null;
      this.rejectAllPending('WebSocket connection closed');
      if (!this.closed) {
        this.scheduleReconnect();
      }
    };
  }

  private handleMessage(ev: MessageEvent): void {
    let response: WsResponse;
    try {
      response = JSON.parse(ev.data as string) as WsResponse;
    } catch {
      console.warn('[ws-command-proxy] Received non-JSON message:', ev.data);
      return;
    }

    if (typeof response.id !== 'number') {
      // Could be a server-initiated push notification — ignore for now.
      return;
    }

    const pending = this.pending.get(response.id);
    if (!pending) {
      console.warn('[ws-command-proxy] No pending call for response id:', response.id);
      return;
    }

    this.pending.delete(response.id);
    clearTimeout(pending.timer);

    if (response.error) {
      pending.reject(new Error(response.error));
    } else {
      pending.resolve(response.result);
    }
  }

  // -----------------------------------------------------------------------
  // Reconnection
  // -----------------------------------------------------------------------

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error(
        `[ws-command-proxy] Max reconnection attempts (${this.maxReconnectAttempts}) reached. Giving up.`,
      );
      this.rejectAllPending('WebSocket max reconnect attempts exceeded');
      return;
    }

    const delay = Math.min(
      this.baseReconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.maxReconnectDelay,
    );
    this.reconnectAttempts++;

    console.info(
      `[ws-command-proxy] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})...`,
    );

    this.clearReconnectTimer();
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect().catch(() => {
        // connect() failure triggers onclose → scheduleReconnect() again.
      });
    }, delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  // -----------------------------------------------------------------------
  // Helpers
  // -----------------------------------------------------------------------

  private flushQueue(): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
    while (this.queue.length > 0) {
      const msg = this.queue.shift()!;
      this.ws.send(msg);
    }
  }

  private rejectAllPending(reason: string): void {
    for (const [id, p] of this.pending) {
      clearTimeout(p.timer);
      p.reject(new Error(reason));
      this.pending.delete(id);
    }
  }
}

// ---------------------------------------------------------------------------
// Module-level singleton
// ---------------------------------------------------------------------------

let instance: WsCommandProxy | null = null;

/** Get (or lazily create) the shared WebSocket command proxy. */
export function getWsCommandProxy(): WsCommandProxy {
  if (!instance) {
    instance = new WsCommandProxy();
  }
  return instance;
}

export type { WsCommandProxy };
