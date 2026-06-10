// IM Channel Registry - Unified IM integration for DingTalk/Feishu/WeChat
// Allows remote control and monitoring from mobile

import { ref, computed, type Ref } from 'vue';
import { invokeOrProxy } from '../host';

export type IMChannelType = 'feishu' | 'dingtalk' | 'wechat' | 'wechat-work' | 'telegram';

export interface IMMessage {
  text: string;
  card?: IMCard;               // Rich text card
  buttons?: IMButton[];        // Interactive buttons
  attachments?: IMAttachment[];
}

export interface IMCard {
  title: string;
  content: string;
  color?: 'blue' | 'green' | 'red' | 'yellow';
}

export interface IMButton {
  id: string;
  text: string;
  action: 'approve' | 'reject' | 'view' | 'command';
  payload?: Record<string, unknown>;
}

export interface IMAttachment {
  type: 'image' | 'file' | 'link';
  url: string;
  name?: string;
}

export interface IMChannel {
  id: IMChannelType;
  name: string;
  connected: boolean;
  chatId?: string;             // Target chat/group ID
  supportsCards: boolean;
  supportsButtons: boolean;
  lastMessageTime?: number;
}

export interface IMNotification {
  id: string;
  type: 'agent_complete' | 'approval_needed' | 'error' | 'status';
  message: IMMessage;
  channels: IMChannelType[];
  sentAt?: number;
}

export class IMChannelRegistry {
  private channels: Ref<Map<IMChannelType, IMChannel>> = ref(new Map());
  private messageHandlers: Map<IMChannelType, (msg: IMMessage) => Promise<void>> = new Map();
  private pendingNotifications: Ref<IMNotification[]> = ref([]);

  // State for UI
  public isAnyConnected: Ref<boolean> = ref(false);
  public connectedCount: Ref<number> = ref(0);

  // Register IM channel
  async registerChannel(type: IMChannelType, config: {
    chatId: string;
    token?: string;
    webhookUrl?: string;
  }): Promise<IMChannel> {
    // Call backend to initialize
    await invokeOrProxy('im_channel_register', {
      type,
      chat_id: config.chatId,
      token: config.token,
      webhook_url: config.webhookUrl,
    });

    const channel: IMChannel = {
      id: type,
      name: this.getChannelName(type),
      connected: true,
      chatId: config.chatId,
      supportsCards: this.supportsCards(type),
      supportsButtons: this.supportsButtons(type),
      lastMessageTime: Date.now(),
    };

    this.channels.value.set(type, channel);
    this.updateConnectionStatus();

    // Set up message handler
    this.messageHandlers.set(type, this.handleIncomingMessage(type));

    return channel;
  }

  // Get channel display name
  private getChannelName(type: IMChannelType): string {
    const names: Record<IMChannelType, string> = {
      feishu: '飞书',
      dingtalk: '钉钉',
      wechat: '微信',
      'wechat-work': '企业微信',
      telegram: 'Telegram',
    };
    return names[type];
  }

  // Check if channel supports rich cards
  private supportsCards(type: IMChannelType): boolean {
    return ['feishu', 'dingtalk'].includes(type);
  }

  // Check if channel supports interactive buttons
  private supportsButtons(type: IMChannelType): boolean {
    return ['feishu', 'dingtalk', 'telegram'].includes(type);
  }

  // Handle incoming message from IM
  private handleIncomingMessage(type: IMChannelType): (msg: IMMessage) => Promise<void> {
    return async (msg: IMMessage) => {
      // Parse command from message
      const command = this.parseCommand(msg.text);

      if (command) {
        await this.executeCommand(command, msg);
      }
    };
  }

  // Parse command from IM message
  private parseCommand(text: string): {
    type: 'status' | 'approve' | 'reject' | 'start' | 'stop' | 'list';
    payload?: Record<string, unknown>;
  } | null {
    const patterns: { pattern: RegExp; type: 'status' | 'approve' | 'reject' | 'start' | 'stop' | 'list' }[] = [
      { pattern: /^status|状态$/i, type: 'status' },
      { pattern: /^approve|批准\s+(\S+)/i, type: 'approve' },
      { pattern: /^reject|拒绝\s+(\S+)/i, type: 'reject' },
      { pattern: /^start|启动\s+(\S+)/i, type: 'start' },
      { pattern: /^stop|停止\s+(\S+)/i, type: 'stop' },
      { pattern: /^list|列表$/i, type: 'list' },
    ];

    for (const { pattern, type } of patterns) {
      const match = text.match(pattern);
      if (match) {
        return {
          type,
          payload: match[1] ? { id: match[1] } : undefined,
        };
      }
    }

    return null;
  }

  // Execute command from IM
  private async executeCommand(command: { type: string; payload?: Record<string, unknown> }, msg: IMMessage): Promise<void> {
    switch (command.type) {
      case 'status':
        await this.sendStatusReport();
        break;
      case 'approve': {
        if (command.payload?.id) {
          await invokeOrProxy('approval_decide', {
            request_id: command.payload.id,
            decision: 'approved',
          });
          // Reply on the originating channel, not hardcoded to feishu
          const approveChannel = (command.payload as Record<string, unknown>).channel as IMChannelType || 'feishu';
          await this.sendMessage(approveChannel, {
            text: `已批准请求 ${command.payload.id}`,
          });
        }
        break;
      }
      case 'reject': {
        if (command.payload?.id) {
          await invokeOrProxy('approval_decide', {
            request_id: command.payload.id,
            decision: 'rejected',
          });
          // Reply on the originating channel, not hardcoded to feishu
          const rejectChannel = (command.payload as Record<string, unknown>).channel as IMChannelType || 'feishu';
          await this.sendMessage(rejectChannel, {
            text: `已拒绝请求 ${command.payload.id}`,
          });
        }
        break;
      }
      case 'list':
        await this.sendTaskList();
        break;
    }
  }

  // Send message to specific channel
  async sendMessage(type: IMChannelType, message: IMMessage): Promise<void> {
    const channel = this.channels.value.get(type);
    if (!channel || !channel.connected) {
      throw new Error(`Channel ${type} not connected`);
    }

    await invokeOrProxy('im_channel_send', {
      type,
      chat_id: channel.chatId,
      message,
    });

    channel.lastMessageTime = Date.now();
  }

  // Broadcast to all connected channels
  async broadcast(message: IMMessage): Promise<void> {
    for (const [type, channel] of this.channels.value) {
      if (channel.connected) {
        try {
          await this.sendMessage(type, message);
        } catch (e) {
          console.warn(`[IMChannel] Failed to send to ${type}:`, e);
        }
      }
    }
  }

  // Notify agent completion
  async notifyAgentComplete(agentId: string, result: {
    taskDescription: string;
    status: 'completed' | 'failed';
    summary: string;
  }): Promise<void> {
    const message: IMMessage = {
      text: `🤖 Agent完成通知`,
      card: {
        title: `Agent ${agentId} 任务完成`,
        content: `${result.taskDescription}\n状态: ${result.status}\n摘要: ${result.summary}`,
        color: result.status === 'completed' ? 'green' : 'red',
      },
      buttons: [
        { id: 'view', text: '查看详情', action: 'view', payload: { agentId } },
      ],
    };

    await this.broadcast(message);
  }

  // Push approval request
  async pushApprovalRequest(request: {
    id: string;
    type: string;
    description: string;
  }): Promise<void> {
    const message: IMMessage = {
      text: `🔔 审批请求`,
      card: {
        title: `需要审批: ${request.type}`,
        content: request.description,
        color: 'yellow',
      },
      buttons: [
        { id: request.id, text: '批准', action: 'approve', payload: { requestId: request.id } },
        { id: request.id, text: '拒绝', action: 'reject', payload: { requestId: request.id } },
      ],
    };

    await this.broadcast(message);
  }

  // Send status report
  private async sendStatusReport(): Promise<void> {
    const status = await invokeOrProxy('orchestration_get_status') as {
      activeTasks?: number;
      pendingApprovals?: number;
      totalAgents?: number;
    };
    const message: IMMessage = {
      text: `📊 ACP-UI 状态报告\n活跃任务: ${status.activeTasks ?? 0}\n待审批: ${status.pendingApprovals ?? 0}\n注册Agent: ${status.totalAgents ?? 0}`,
    };

    await this.broadcast(message);
  }

  // Send task list
  private async sendTaskList(): Promise<void> {
    const tasks = await invokeOrProxy('orchestration_list_active_tasks');
    const taskList = (tasks as Array<{ id: string; description: string; status: string }>)
      .map(t => `- ${t.id}: ${t.description} (${t.status})`)
      .join('\n');

    const message: IMMessage = {
      text: `📋 任务列表\n${taskList || '暂无活跃任务'}`,
    };

    await this.broadcast(message);
  }

  // Update connection status
  private updateConnectionStatus(): void {
    const connected = Array.from(this.channels.value.values()).filter(c => c.connected);
    this.isAnyConnected.value = connected.length > 0;
    this.connectedCount.value = connected.length;
  }

  // Get connected channels
  getConnectedChannels(): IMChannel[] {
    return Array.from(this.channels.value.values()).filter(c => c.connected);
  }

  // Disconnect channel
  async disconnectChannel(type: IMChannelType): Promise<void> {
    await invokeOrProxy('im_channel_disconnect', { type });

    const channel = this.channels.value.get(type);
    if (channel) {
      channel.connected = false;
    }

    this.updateConnectionStatus();
  }
}

// Singleton instance
let imChannelRegistryInstance: IMChannelRegistry | null = null;

export function useIMChannelRegistry(): IMChannelRegistry {
  if (!imChannelRegistryInstance) {
    imChannelRegistryInstance = new IMChannelRegistry();
  }
  return imChannelRegistryInstance;
}