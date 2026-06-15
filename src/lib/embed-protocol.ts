/**
 * ACP Embed Protocol
 *
 * 用于将 ACP-UI 嵌入第三方应用（iframe + postMessage）
 *
 * 功能：
 * - 双向消息通信
 * - 状态同步
 * - 任务提交和监控
 * - 安全验证
 */

// ============================================================================
// Types
// ============================================================================

export interface EmbedMessage {
  type: EmbedMessageType;
  payload: unknown;
  timestamp: number;
  source: 'host' | 'embed';
}

export type EmbedMessageType =
  | 'init'
  | 'ready'
  | 'state_update'
  | 'task_submit'
  | 'task_status'
  | 'goal_list'
  | 'worker_list'
  | 'action_request'
  | 'action_response';

export interface EmbedConfig {
  origin: string;
  allowedOrigins: string[];
  authToken?: string;
}

export interface EmbedState {
  goals: unknown[];
  workers: unknown[];
  tasks: unknown[];
  queen: unknown;
}

// ============================================================================
// Embed Host (第三方应用使用)
// ============================================================================

export class EmbedHost {
  private iframe: HTMLIFrameElement | null = null;
  private messageHandler: ((msg: EmbedMessage) => void) | null = null;
  private ready = false;

  /**
   * 创建并嵌入 ACP-UI
   */
  embed(containerId: string, acpUrl: string, config?: EmbedConfig): void {
    const container = document.getElementById(containerId);
    if (!container) {
      throw new Error(`Container #${containerId} not found`);
    }

    this.iframe = document.createElement('iframe');
    this.iframe.src = acpUrl;
    this.iframe.style.width = '100%';
    this.iframe.style.height = '100%';
    this.iframe.style.border = 'none';

    // 监听消息
    window.addEventListener('message', (event) => {
      this.handleMessage(event, config);
    });

    container.appendChild(this.iframe);
  }

  /**
   * 设置消息处理回调
   */
  onMessage(handler: (msg: EmbedMessage) => void): void {
    this.messageHandler = handler;
  }

  /**
   * 发送消息到 Embed
   */
  send(type: EmbedMessageType, payload: unknown): void {
    if (!this.iframe || !this.iframe.contentWindow) {
      throw new Error('iframe not ready');
    }

    const msg: EmbedMessage = {
      type,
      payload,
      timestamp: Date.now(),
      source: 'host',
    };

    this.iframe.contentWindow.postMessage(msg, '*');
  }

  /**
   * 提交任务
   */
  submitTask(task: { prompt: string; workingDir?: string }): Promise<string> {
    return new Promise((resolve) => {
      const taskId = `task-${Date.now()}`;

      this.send('task_submit', { taskId, ...task });

      // 等待确认
      const handler = (msg: EmbedMessage) => {
        if (msg.type === 'task_status' && (msg.payload as any).taskId === taskId) {
          resolve(taskId);
        }
      };

      this.onMessage(handler);
    });
  }

  /**
   * 获取状态
   */
  getState(): Promise<EmbedState> {
    return new Promise((resolve) => {
      this.send('goal_list', {});
      this.send('worker_list', {});

      // 收集响应
      const state: EmbedState = { goals: [], workers: [], tasks: [], queen: null };
      const handler = (msg: EmbedMessage) => {
        if (msg.type === 'state_update') {
          Object.assign(state, msg.payload);
          if (state.goals.length > 0 && state.workers.length > 0) {
            resolve(state);
          }
        }
      };

      this.onMessage(handler);
    });
  }

  /**
   * 处理接收的消息
   */
  private handleMessage(event: MessageEvent, config?: EmbedConfig): void {
    // 验证来源
    if (config?.allowedOrigins && config.allowedOrigins.length > 0) {
      if (!config.allowedOrigins.includes(event.origin)) {
        return;
      }
    }

    const msg = event.data as EmbedMessage;
    if (msg.source !== 'embed') return;

    if (msg.type === 'ready') {
      this.ready = true;
      // 发送初始化
      this.send('init', { authToken: config?.authToken });
    }

    if (this.messageHandler) {
      this.messageHandler(msg);
    }
  }

  /**
   * 移除 Embed
   */
  remove(): void {
    if (this.iframe) {
      this.iframe.remove();
      this.iframe = null;
    }
  }
}

// ============================================================================
// Embed Client (ACP-UI 内部使用)
// ============================================================================

export class EmbedClient {
  private hostOrigin: string | null = null;
  private messageHandler: ((msg: EmbedMessage) => void) | null = null;
  private initialized = false;

  constructor() {
    window.addEventListener('message', (event) => {
      this.handleMessage(event);
    });
  }

  /**
   * 设置消息处理回调
   */
  onMessage(handler: (msg: EmbedMessage) => void): void {
    this.messageHandler = handler;
  }

  /**
   * 发送消息到 Host
   */
  send(type: EmbedMessageType, payload: unknown): void {
    if (!this.hostOrigin) {
      console.warn('[EmbedClient] host origin not set, cannot send');
      return;
    }

    const msg: EmbedMessage = {
      type,
      payload,
      timestamp: Date.now(),
      source: 'embed',
    };

    window.parent.postMessage(msg, this.hostOrigin);
  }

  /**
   * 发送就绪信号
   */
  sendReady(): void {
    // 发送到父窗口（假设是 host）
    const msg: EmbedMessage = {
      type: 'ready',
      payload: {},
      timestamp: Date.now(),
      source: 'embed',
    };

    window.parent.postMessage(msg, '*');
  }

  /**
   * 发送状态更新
   */
  sendStateUpdate(state: EmbedState): void {
    this.send('state_update', state);
  }

  /**
   * 发送任务状态
   */
  sendTaskStatus(taskId: string, status: string): void {
    this.send('task_status', { taskId, status });
  }

  /**
   * 处理接收的消息
   */
  private handleMessage(event: MessageEvent): void {
    // 忽略非父窗口消息
    if (event.source !== window.parent) return;

    const msg = event.data as EmbedMessage;
    if (msg.source !== 'host') return;

    // 初始化
    if (msg.type === 'init') {
      this.hostOrigin = event.origin;
      this.initialized = true;
      console.log('[EmbedClient] initialized with host:', this.hostOrigin);
    }

    if (this.messageHandler) {
      this.messageHandler(msg);
    }
  }

  /**
   * 检查是否嵌入模式
   */
  isEmbedded(): boolean {
    return window.parent !== window;
  }
}

// ============================================================================
// Export
// ============================================================================

export const embedClient = new EmbedClient();

export function isEmbedded(): boolean {
  return window.parent !== window;
}

export default {
  EmbedHost,
  EmbedClient,
  embedClient,
  isEmbedded,
};