/**
 * Browser Adapter - controls Chrome browser via DevTools Protocol
 */

interface BrowserConfig {
  port: number;
  host: string;
  autoConnect: boolean;
  defaultTimeout: number;
}

interface CdpCommand {
  id: number;
  method: string;
  params?: Record<string, unknown>;
}

interface CdpResponse {
  id: number;
  result?: unknown;
  error?: { code: number; message: string };
}

interface CdpEvent {
  method: string;
  params?: Record<string, unknown>;
}

interface NavigateResult {
  url: string;
  title: string;
  loadTime: number;
}

interface ClickResult {
  selector: string;
  x: number;
  y: number;
  success: boolean;
}

interface EvaluateResult {
  result: unknown;
  exceptionDetails?: {
    text: string;
    lineNumber: number;
    columnNumber: number;
  };
}

interface CaptureResult {
  data: string; // Base64 encoded image
  format: 'jpeg' | 'png';
  width: number;
  height: number;
}

interface ConsoleMessage {
  type: 'log' | 'warning' | 'error' | 'info' | 'debug';
  text: string;
  url?: string;
  line?: number;
  timestamp: number;
}

interface NetworkRequest {
  requestId: string;
  url: string;
  method: string;
  headers: Record<string, string>;
  postData?: string;
  status?: number;
  responseHeaders?: Record<string, string>;
  responseBody?: string;
  timing?: {
    requestTime: number;
    responseTime: number;
    loadTime: number;
  };
}

class BrowserAdapter {
  private ws: WebSocket | null = null;
  private config: BrowserConfig;
  private commandId = 0;
  private pendingCommands: Map<number, { resolve: Function; reject: Function }> = new Map();
  private consoleMessages: ConsoleMessage[] = [];
  private networkRequests: Map<string, NetworkRequest> = new Map();
  private currentPageUrl = '';
  private isConnected = false;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  constructor(config?: Partial<BrowserConfig>) {
    this.config = {
      port: 9222,
      host: 'localhost',
      autoConnect: true,
      defaultTimeout: 10000,
      ...config,
    };
  }

  /**
   * Connect to Chrome DevTools Protocol
   */
  async connect(): Promise<boolean> {
    if (this.isConnected) {
      return true;
    }

    try {
      // Get DevTools endpoint from Chrome
      const endpoints = await this.getDevToolsEndpoints();

      if (endpoints.length === 0) {
        throw new Error('No Chrome DevTools endpoints found. Ensure Chrome is running with --remote-debugging-port=' + this.config.port);
      }

      // Connect to WebSocket
      const wsUrl = endpoints[0].webSocketDebuggerUrl;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        this.isConnected = true;
        this.reconnectAttempts = 0;
        console.log('BrowserAdapter: Connected to Chrome DevTools');
        this.enableDomains();
      };

      this.ws.onmessage = (event) => {
        this.handleMessage(JSON.parse(event.data));
      };

      this.ws.onerror = (error) => {
        console.error('BrowserAdapter: WebSocket error', error);
      };

      this.ws.onclose = () => {
        this.isConnected = false;
        this.ws = null;
        console.log('BrowserAdapter: Disconnected from Chrome DevTools');

        // Auto reconnect
        if (this.config.autoConnect && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++;
          setTimeout(() => this.connect(), 2000);
        }
      };

      // Wait for connection
      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => {
          reject(new Error('Connection timeout'));
        }, this.config.defaultTimeout);

        this.ws!.onopen = () => {
          clearTimeout(timeout);
          resolve();
        };

        this.ws!.onerror = (err) => {
          clearTimeout(timeout);
          reject(err);
        };
      });

      return true;
    } catch (error) {
      console.error('BrowserAdapter: Connection failed', error);
      return false;
    }
  }

  /**
   * Get DevTools endpoints from Chrome
   */
  private async getDevToolsEndpoints(): Promise<{ webSocketDebuggerUrl: string }[]> {
    try {
      const response = await fetch(`http://${this.config.host}:${this.config.port}/json`);
      const endpoints = await response.json();
      return endpoints;
    } catch (error) {
      console.error('BrowserAdapter: Failed to get DevTools endpoints', error);
      return [];
    }
  }

  /**
   * Enable CDP domains
   */
  private async enableDomains(): Promise<void> {
    await this.sendCommand('Page.enable');
    await this.sendCommand('Runtime.enable');
    await this.sendCommand('Network.enable');
    await this.sendCommand('Console.enable');
    await this.sendCommand('Log.enable');
  }

  /**
   * Handle WebSocket message
   */
  private handleMessage(data: CdpResponse | CdpEvent): void {
    if ('id' in data) {
      // Command response
      const pending = this.pendingCommands.get(data.id);
      if (pending) {
        this.pendingCommands.delete(data.id);
        if (data.error) {
          pending.reject(new Error(data.error.message));
        } else {
          pending.resolve(data.result);
        }
      }
    } else {
      // Event
      this.handleEvent(data);
    }
  }

  /**
   * Handle CDP event
   */
  private handleEvent(event: CdpEvent): void {
    switch (event.method) {
      case 'Page.frameNavigated':
        this.currentPageUrl = (event.params as { frame: { url: string } }).frame.url;
        break;

      case 'Runtime.consoleAPICalled':
        const consoleParams = event.params as {
          type: string;
          args: Array<{ value?: string }>;
          timestamp: number;
        };
        this.consoleMessages.push({
          type: consoleParams.type as ConsoleMessage['type'],
          text: consoleParams.args.map(a => a.value || '').join(' '),
          timestamp: consoleParams.timestamp,
        });
        break;

      case 'Network.requestWillBeSent':
        const reqParams = event.params as {
          requestId: string;
          request: {
            url: string;
            method: string;
            headers: Record<string, string>;
            postData?: string;
          };
        };
        this.networkRequests.set(reqParams.requestId, {
          requestId: reqParams.requestId,
          url: reqParams.request.url,
          method: reqParams.request.method,
          headers: reqParams.request.headers,
          postData: reqParams.request.postData,
        });
        break;

      case 'Network.responseReceived':
        const respParams = event.params as {
          requestId: string;
          response: {
            status: number;
            headers: Record<string, string>;
          };
        };
        const req = this.networkRequests.get(respParams.requestId);
        if (req) {
          req.status = respParams.response.status;
          req.responseHeaders = respParams.response.headers;
        }
        break;

      case 'Network.loadingFinished':
        const finishParams = event.params as { requestId: string; timestamp: number };
        const reqFinish = this.networkRequests.get(finishParams.requestId);
        if (reqFinish) {
          reqFinish.timing = {
            requestTime: 0,
            responseTime: 0,
            loadTime: finishParams.timestamp,
          };
        }
        break;
    }
  }

  /**
   * Send CDP command
   */
  private async sendCommand(method: string, params?: Record<string, unknown>): Promise<unknown> {
    if (!this.ws || !this.isConnected) {
      throw new Error('Not connected to Chrome DevTools');
    }

    const id = ++this.commandId;
    const command: CdpCommand = { id, method, params };

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pendingCommands.delete(id);
        reject(new Error(`Command ${method} timeout`));
      }, this.config.defaultTimeout);

      this.pendingCommands.set(id, {
        resolve: (result: unknown) => {
          clearTimeout(timeout);
          resolve(result);
        },
        reject: (error: Error) => {
          clearTimeout(timeout);
          reject(error);
        },
      });

      this.ws!.send(JSON.stringify(command));
    });
  }

  /**
   * Navigate to URL
   */
  async navigate(url: string): Promise<NavigateResult> {
    const startTime = Date.now();

    await this.sendCommand('Page.navigate', { url });

    // Wait for load
    await this.sendCommand('Page.loadEventFired');

    const loadTime = Date.now() - startTime;

    // Get page info
    const result = await this.sendCommand('Page.getTargetInfo') as {
      targetInfo: { url: string; title: string };
    };

    this.currentPageUrl = result.targetInfo.url;

    return {
      url: this.currentPageUrl,
      title: result.targetInfo.title,
      loadTime,
    };
  }

  /**
   * Get current URL
   */
  getCurrentUrl(): string {
    return this.currentPageUrl;
  }

  /**
   * Click element at coordinates
   */
  async click(x: number, y: number): Promise<ClickResult> {
    // Mouse press
    await this.sendCommand('Input.dispatchMouseEvent', {
      type: 'mousePressed',
      x,
      y,
      button: 'left',
      clickCount: 1,
    });

    // Mouse release
    await this.sendCommand('Input.dispatchMouseEvent', {
      type: 'mouseReleased',
      x,
      y,
      button: 'left',
      clickCount: 1,
    });

    return {
      selector: '',
      x,
      y,
      success: true,
    };
  }

  /**
   * Click element by selector
   */
  async clickSelector(selector: string): Promise<ClickResult> {
    // Get element position
    const result = await this.sendCommand('Runtime.evaluate', {
      expression: `
        const el = document.querySelector('${selector}');
        if (el) {
          const rect = el.getBoundingClientRect();
          JSON.stringify({
            x: rect.left + rect.width / 2,
            y: rect.top + rect.height / 2,
            found: true
          });
        } else {
          JSON.stringify({ found: false });
        }
      `,
    }) as { result: { value: string } };

    const position = JSON.parse(result.result.value);

    if (!position.found) {
      return {
        selector,
        x: 0,
        y: 0,
        success: false,
      };
    }

    return this.click(position.x, position.y);
  }

  /**
   * Type text
   */
  async type(text: string): Promise<void> {
    for (const char of text) {
      await this.sendCommand('Input.dispatchKeyEvent', {
        type: 'keyDown',
        text: char,
      });
      await this.sendCommand('Input.dispatchKeyEvent', {
        type: 'keyUp',
        text: char,
      });
      // Small delay between characters
      await new Promise(resolve => setTimeout(resolve, 50));
    }
  }

  /**
   * Execute JavaScript
   */
  async evaluate(expression: string): Promise<EvaluateResult> {
    const result = await this.sendCommand('Runtime.evaluate', {
      expression,
      returnByValue: true,
    }) as {
      result: { value?: unknown; description?: string };
      exceptionDetails?: EvaluateResult['exceptionDetails'];
    };

    return {
      result: result.result.value || result.result.description,
      exceptionDetails: result.exceptionDetails,
    };
  }

  /**
   * Capture screenshot
   */
  async captureScreenshot(format: 'jpeg' | 'png' = 'png'): Promise<CaptureResult> {
    const result = await this.sendCommand('Page.captureScreenshot', {
      format,
    }) as { data: string };

    // Get viewport size
    const layout = await this.sendCommand('Page.getLayoutMetrics') as {
      contentSize: { width: number; height: number };
    };

    return {
      data: result.data,
      format,
      width: layout.contentSize.width,
      height: layout.contentSize.height,
    };
  }

  /**
   * Scroll page
   */
  async scroll(x: number, y: number): Promise<void> {
    await this.evaluate(`window.scrollTo(${x}, ${y})`);
  }

  /**
   * Scroll to element
   */
  async scrollToSelector(selector: string): Promise<boolean> {
    const result = await this.evaluate(`
      const el = document.querySelector('${selector}');
      if (el) {
        el.scrollIntoView({ behavior: 'instant' });
        true;
      } else {
        false;
      }
    `);

    return result.result === true;
  }

  /**
   * Wait for element
   */
  async waitForSelector(selector: string, timeout: number = 5000): Promise<boolean> {
    const startTime = Date.now();

    while (Date.now() - startTime < timeout) {
      const result = await this.evaluate(`
        document.querySelector('${selector}') !== null
      `);

      if (result.result === true) {
        return true;
      }

      await new Promise(resolve => setTimeout(resolve, 100));
    }

    return false;
  }

  /**
   * Get console messages
   */
  getConsoleMessages(): ConsoleMessage[] {
    return [...this.consoleMessages];
  }

  /**
   * Clear console messages
   */
  clearConsoleMessages(): void {
    this.consoleMessages = [];
  }

  /**
   * Get network requests
   */
  getNetworkRequests(): NetworkRequest[] {
    return Array.from(this.networkRequests.values());
  }

  /**
   * Get network request by ID
   */
  getNetworkRequest(requestId: string): NetworkRequest | undefined {
    return this.networkRequests.get(requestId);
  }

  /**
   * Clear network requests
   */
  clearNetworkRequests(): void {
    this.networkRequests.clear();
  }

  /**
   * Get page content
   */
  async getPageContent(): Promise<string> {
    const result = await this.evaluate('document.documentElement.outerHTML');
    return result.result as string;
  }

  /**
   * Get page title
   */
  async getTitle(): Promise<string> {
    const result = await this.evaluate('document.title');
    return result.result as string;
  }

  /**
   * Refresh page
   */
  async refresh(): Promise<NavigateResult> {
    await this.sendCommand('Page.reload');
    await this.sendCommand('Page.loadEventFired');

    return {
      url: this.currentPageUrl,
      title: await this.getTitle(),
      loadTime: 0,
    };
  }

  /**
   * Go back
   */
  async goBack(): Promise<boolean> {
    try {
      await this.sendCommand('Page.goBack');
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Go forward
   */
  async goForward(): Promise<boolean> {
    try {
      await this.sendCommand('Page.goForward');
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Close browser tab
   */
  async close(): Promise<void> {
    if (this.ws) {
      await this.sendCommand('Page.close');
      this.ws.close();
      this.ws = null;
      this.isConnected = false;
    }
  }

  /**
   * Disconnect from browser
   */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
      this.isConnected = false;
    }
  }

  /**
   * Check connection status
   */
  isConnectedStatus(): boolean {
    return this.isConnected;
  }

  /**
   * Generate adapter report
   */
  generateReport(): string {
    const lines: string[] = [];

    lines.push('# Browser Adapter Report');
    lines.push('\n## Connection Status\n');
    lines.push(`- Connected: ${this.isConnected ? 'Yes' : 'No'}`);
    lines.push(`- Current URL: ${this.currentPageUrl || 'N/A'}`);
    lines.push(`- Port: ${this.config.port}`);

    lines.push('\n## Console Messages\n');
    const recentConsole = this.consoleMessages.slice(-10);
    for (const msg of recentConsole) {
      lines.push(`- [${msg.type}] ${msg.text}`);
    }

    lines.push('\n## Network Requests\n');
    const recentRequests = Array.from(this.networkRequests.values()).slice(-10);
    for (const req of recentRequests) {
      lines.push(`- ${req.method} ${req.url} (${req.status || 'pending'})`);
    }

    return lines.join('\n');
  }
}

export const browserAdapter = new BrowserAdapter();
export type {
  BrowserConfig,
  NavigateResult,
  ClickResult,
  EvaluateResult,
  CaptureResult,
  ConsoleMessage,
  NetworkRequest,
};