/**
 * Command Queue System - queues commands when offline and sends when reconnected
 */

interface QueuedCommand {
  id: string;
  type: string;
  payload: unknown;
  timestamp: number;
  retryCount: number;
  maxRetries: number;
  priority: 'low' | 'normal' | 'high' | 'critical';
  status: 'pending' | 'sent' | 'success' | 'failed';
  response?: unknown;
  error?: string;
}

interface CommandQueueConfig {
  maxSize: number;
  maxRetries: number;
  retryDelay: number; // milliseconds
  persist: boolean;
}

type CommandHandler = (command: QueuedCommand) => Promise<unknown>;

class CommandQueue {
  private queue: QueuedCommand[] = [];
  private config: CommandQueueConfig;
  private handlers: Map<string, CommandHandler> = new Map();
  private isConnected = true;
  private isProcessing = false;
  private commandCounter = 0;
  private sentCommands: Map<string, QueuedCommand> = new Map(); // Track sent commands for response

  constructor(config?: Partial<CommandQueueConfig>) {
    this.config = {
      maxSize: 100,
      maxRetries: 3,
      retryDelay: 5000,
      persist: true,
      ...config,
    };
  }

  /**
   * Register command handler
   */
  registerHandler(commandType: string, handler: CommandHandler): void {
    this.handlers.set(commandType, handler);
  }

  /**
   * Unregister command handler
   */
  unregisterHandler(commandType: string): void {
    this.handlers.delete(commandType);
  }

  /**
   * Add command to queue
   */
  enqueue(type: string, payload: unknown, priority: QueuedCommand['priority'] = 'normal'): string {
    const id = `cmd-${Date.now()}-${this.commandCounter++}`;

    const command: QueuedCommand = {
      id,
      type,
      payload,
      timestamp: Date.now(),
      retryCount: 0,
      maxRetries: this.config.maxRetries,
      priority,
      status: 'pending',
    };

    // Check queue size
    if (this.queue.length >= this.config.maxSize) {
      // Remove oldest low priority commands
      this.evictLowPriority();
    }

    // Insert by priority (critical first)
    const insertIndex = this.findInsertIndex(priority);
    this.queue.splice(insertIndex, 0, command);

    // Process immediately if connected
    if (this.isConnected && !this.isProcessing) {
      this.processQueue();
    }

    return id;
  }

  /**
   * Find insert index based on priority
   */
  private findInsertIndex(priority: QueuedCommand['priority']): number {
    const priorityOrder = { critical: 0, high: 1, normal: 2, low: 3 };

    for (let i = 0; i < this.queue.length; i++) {
      if (priorityOrder[this.queue[i].priority] > priorityOrder[priority]) {
        return i;
      }
    }

    return this.queue.length;
  }

  /**
   * Evict low priority commands
   */
  private evictLowPriority(): void {
    // Remove oldest low priority commands
    const lowPriority = this.queue.filter(c => c.priority === 'low');

    if (lowPriority.length > 0) {
      const oldest = lowPriority[0];
      const index = this.queue.indexOf(oldest);
      if (index !== -1) {
        this.queue.splice(index, 1);
      }
    }
  }

  /**
   * Process queue
   */
  private async processQueue(): Promise<void> {
    if (this.isProcessing || !this.isConnected || this.queue.length === 0) {
      return;
    }

    this.isProcessing = true;

    while (this.queue.length > 0 && this.isConnected) {
      const command = this.queue[0];

      // Skip if no handler
      const handler = this.handlers.get(command.type);
      if (!handler) {
        console.warn(`No handler for command type: ${command.type}`);
        this.queue.shift();
        continue;
      }

      try {
        command.status = 'sent';
        this.sentCommands.set(command.id, command);

        const response = await handler(command);
        command.status = 'success';
        command.response = response;

        // Remove from queue
        this.queue.shift();
        this.sentCommands.delete(command.id);

      } catch (error) {
        command.retryCount++;
        command.error = error instanceof Error ? error.message : String(error);

        if (command.retryCount >= command.maxRetries) {
          command.status = 'failed';
          this.queue.shift();
          this.sentCommands.delete(command.id);
        } else {
          command.status = 'pending';
          // Move to end of queue for retry
          this.queue.shift();
          setTimeout(() => {
            this.queue.push(command);
          }, this.config.retryDelay);
        }
      }
    }

    this.isProcessing = false;
  }

  /**
   * Set connection status
   */
  setConnected(connected: boolean): void {
    this.isConnected = connected;

    if (connected) {
      // Resume processing
      this.processQueue();
    }
  }

  /**
   * Get connection status
   */
  isConnectedStatus(): boolean {
    return this.isConnected;
  }

  /**
   * Get queue status
   */
  getStatus(): {
    pending: number;
    sent: number;
    success: number;
    failed: number;
    total: number;
  } {
    const pending = this.queue.filter(c => c.status === 'pending').length;
    const sent = this.queue.filter(c => c.status === 'sent').length;
    const success = this.queue.filter(c => c.status === 'success').length;
    const failed = this.queue.filter(c => c.status === 'failed').length;

    return {
      pending,
      sent,
      success,
      failed,
      total: this.queue.length,
    };
  }

  /**
   * Get command by ID
   */
  getCommand(id: string): QueuedCommand | undefined {
    return this.queue.find(c => c.id === id) || this.sentCommands.get(id);
  }

  /**
   * Get pending commands
   */
  getPendingCommands(): QueuedCommand[] {
    return this.queue.filter(c => c.status === 'pending');
  }

  /**
   * Get failed commands
   */
  getFailedCommands(): QueuedCommand[] {
    return this.queue.filter(c => c.status === 'failed');
  }

  /**
   * Clear failed commands
   */
  clearFailedCommands(): void {
    this.queue = this.queue.filter(c => c.status !== 'failed');
  }

  /**
   * Retry failed command
   */
  retryCommand(id: string): boolean {
    const command = this.queue.find(c => c.id === id && c.status === 'failed');
    if (!command) return false;

    command.status = 'pending';
    command.retryCount = 0;
    command.error = undefined;

    return true;
  }

  /**
   * Cancel command
   */
  cancelCommand(id: string): boolean {
    const index = this.queue.findIndex(c => c.id === id);
    if (index === -1) return false;

    this.queue.splice(index, 1);
    return true;
  }

  /**
   * Clear all commands
   */
  clear(): void {
    this.queue = [];
    this.sentCommands.clear();
  }

  /**
   * Export queue (for persistence)
   */
  export(): QueuedCommand[] {
    return [...this.queue];
  }

  /**
   * Import queue (for restore)
   */
  import(commands: QueuedCommand[]): void {
    this.queue = commands.filter(c => c.status === 'pending');
    this.sentCommands.clear();
  }

  /**
   * Batch send all pending commands
   */
  async flush(): Promise<{
    sent: number;
    failed: number;
  }> {
    const pending = this.getPendingCommands();
    let sent = 0;
    let failed = 0;

    for (const command of pending) {
      const handler = this.handlers.get(command.type);
      if (!handler) {
        failed++;
        continue;
      }

      try {
        command.status = 'sent';
        await handler(command);
        command.status = 'success';
        sent++;
      } catch {
        command.status = 'failed';
        failed++;
      }
    }

    // Remove sent/failed from queue
    this.queue = this.queue.filter(c => c.status === 'pending');

    return { sent, failed };
  }

  /**
   * Check if command exists
   */
  hasCommand(id: string): boolean {
    return this.queue.some(c => c.id === id) || this.sentCommands.has(id);
  }

  /**
   * Get command response
   */
  getResponse(id: string): unknown | undefined {
    const command = this.sentCommands.get(id);
    return command?.response;
  }

  /**
   * Generate queue report
   */
  generateReport(): string {
    const lines: string[] = [];
    const status = this.getStatus();

    lines.push('# Command Queue Report');
    lines.push('\n## Queue Status\n');
    lines.push(`- Connection: ${this.isConnected ? 'Connected' : 'Disconnected'}`);
    lines.push(`- Processing: ${this.isProcessing ? 'Yes' : 'No'}`);
    lines.push(`- Pending: ${status.pending}`);
    lines.push(`- Sent: ${status.sent}`);
    lines.push(`- Success: ${status.success}`);
    lines.push(`- Failed: ${status.failed}`);
    lines.push(`- Total: ${status.total}`);

    lines.push('\n## Pending Commands\n');
    const pending = this.getPendingCommands();
    for (const cmd of pending.slice(0, 10)) {
      lines.push(`- [${cmd.priority}] ${cmd.type} (${cmd.id})`);
    }

    lines.push('\n## Failed Commands\n');
    const failed = this.getFailedCommands();
    for (const cmd of failed.slice(0, 10)) {
      lines.push(`- ${cmd.type} (${cmd.id}): ${cmd.error}`);
    }

    return lines.join('\n');
  }
}

export const commandQueue = new CommandQueue();
export type { QueuedCommand, CommandQueueConfig, CommandHandler };