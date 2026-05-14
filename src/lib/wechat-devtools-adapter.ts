/**
 * WeChat DevTools Adapter - controls WeChat miniprogram development tools
 */

interface WeChatDevToolsConfig {
  cliPath: string;
  projectPath: string;
  port: number;
}

interface WeChatProjectInfo {
  appid: string;
  projectname: string;
  description: string;
  setting: {
    es6: boolean;
    postcss: boolean;
    minified: boolean;
  };
}

interface WeChatCompileResult {
  success: boolean;
  outputPath: string;
  errors: string[];
  warnings: string[];
}

interface WeChatPreviewResult {
  success: boolean;
  qrCodeUrl: string;
  previewId: string;
}

interface WeChatUploadResult {
  success: boolean;
  version: string;
  desc: string;
  uploadTime: Date;
}

interface ApiCallLog {
  id: string;
  timestamp: number;
  url: string;
  method: string;
  headers: Record<string, string>;
  requestBody?: unknown;
  responseBody?: unknown;
  status: number;
  duration: number;
}

class WeChatDevToolsAdapter {
  private config: WeChatDevToolsConfig;
  private projectInfo: WeChatProjectInfo | null = null;
  private apiCallLogs: ApiCallLog[] = [];
  private isConnected = false;

  constructor(config?: Partial<WeChatDevToolsConfig>) {
    this.config = {
      cliPath: 'cli',
      projectPath: './',
      port: 9420,
      ...config,
    };
  }

  /**
   * Initialize WeChat DevTools CLI
   */
  async init(): Promise<boolean> {
    try {
      // Check CLI availability
      await this.executeCommand('--version');
      return true;
    } catch (error) {
      console.error('WeChat DevTools CLI not available:', error);
      return false;
    }
  }

  /**
   * Open project
   */
  async openProject(projectPath: string): Promise<boolean> {
    try {
      await this.executeCommand('open', '--project', projectPath);
      this.config.projectPath = projectPath;
      this.isConnected = true;
      return true;
    } catch (error) {
      console.error('Failed to open project:', error);
      return false;
    }
  }

  /**
   * Close project
   */
  async closeProject(): Promise<boolean> {
    try {
      await this.executeCommand('close', '--project', this.config.projectPath);
      this.isConnected = false;
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get project info
   */
  async getProjectInfo(): Promise<WeChatProjectInfo | null> {
    try {
      // Project info from CLI - placeholder implementation
      await this.executeCommand('project-info', '--project', this.config.projectPath);

      // Parse project.json content
      const info: WeChatProjectInfo = {
        appid: '',
        projectname: '',
        description: '',
        setting: {
          es6: true,
          postcss: true,
          minified: false,
        },
      };

      this.projectInfo = info;
      return info;
    } catch {
      return null;
    }
  }

  /**
   * Build miniprogram
   */
  async build(): Promise<WeChatCompileResult> {
    try {
      const output = await this.executeCommand(
        'build',
        '--project', this.config.projectPath,
        '--compile-condition', '{}'
      );

      const errors: string[] = [];
      const warnings: string[] = [];

      // Parse build output
      const lines = output.split('\n');
      for (const line of lines) {
        if (line.includes('ERROR') || line.includes('error')) {
          errors.push(line);
        } else if (line.includes('WARNING') || line.includes('warning')) {
          warnings.push(line);
        }
      }

      return {
        success: errors.length === 0,
        outputPath: `${this.config.projectPath}/__dist__/`,
        errors,
        warnings,
      };
    } catch (error) {
      return {
        success: false,
        outputPath: '',
        errors: [error instanceof Error ? error.message : String(error)],
        warnings: [],
      };
    }
  }

  /**
   * Preview miniprogram (generate QR code)
   */
  async preview(): Promise<WeChatPreviewResult> {
    try {
      const output = await this.executeCommand(
        'preview',
        '--project', this.config.projectPath,
        '--compile-condition', '{}',
        '--qr-output', 'terminal'
      );

      // Extract QR code URL
      const qrMatch = output.match(/QR Code: (.+)/);
      const previewMatch = output.match(/Preview ID: (.+)/);

      return {
        success: true,
        qrCodeUrl: qrMatch?.[1] || '',
        previewId: previewMatch?.[1] || '',
      };
    } catch (error) {
      return {
        success: false,
        qrCodeUrl: '',
        previewId: '',
      };
    }
  }

  /**
   * Upload miniprogram
   */
  async upload(version: string, desc: string): Promise<WeChatUploadResult> {
    try {
      await this.executeCommand(
        'upload',
        '--project', this.config.projectPath,
        '--version', version,
        '--desc', desc
      );

      return {
        success: true,
        version,
        desc,
        uploadTime: new Date(),
      };
    } catch (error) {
      return {
        success: false,
        version,
        desc,
        uploadTime: new Date(),
      };
    }
  }

  /**
   * Set compile condition (for preview testing)
   */
  async setCompileCondition(condition: {
    path?: string;
    query?: Record<string, string>;
  }): Promise<void> {
    const conditionJson = JSON.stringify(condition);
    await this.executeCommand(
      'set-compile-condition',
      '--project', this.config.projectPath,
      '--condition', conditionJson
    );
  }

  /**
   * Start debugging
   */
  async startDebug(): Promise<boolean> {
    try {
      await this.executeCommand('debug', '--project', this.config.projectPath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Stop debugging
   */
  async stopDebug(): Promise<boolean> {
    try {
      await this.executeCommand('stop-debug', '--project', this.config.projectPath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Capture API calls (requires debug mode)
   */
  captureApiCall(call: ApiCallLog): void {
    this.apiCallLogs.push(call);
  }

  /**
   * Get API call logs
   */
  getApiCallLogs(): ApiCallLog[] {
    return [...this.apiCallLogs];
  }

  /**
   * Get API call by URL pattern
   */
  getApiCallsByUrl(pattern: string): ApiCallLog[] {
    return this.apiCallLogs.filter(log => log.url.includes(pattern));
  }

  /**
   * Get API call by method
   */
  getApiCallsByMethod(method: string): ApiCallLog[] {
    return this.apiCallLogs.filter(log => log.method.toUpperCase() === method.toUpperCase());
  }

  /**
   * Clear API call logs
   */
  clearApiCallLogs(): void {
    this.apiCallLogs = [];
  }

  /**
   * Analyze API performance
   */
  analyzeApiPerformance(): {
    totalCalls: number;
    avgDuration: number;
    slowCalls: ApiCallLog[];
    errorCalls: ApiCallLog[];
  } {
    const totalCalls = this.apiCallLogs.length;

    if (totalCalls === 0) {
      return {
        totalCalls: 0,
        avgDuration: 0,
        slowCalls: [],
        errorCalls: [],
      };
    }

    const avgDuration = this.apiCallLogs.reduce((sum, log) => sum + log.duration, 0) / totalCalls;
    const slowCalls = this.apiCallLogs.filter(log => log.duration > avgDuration * 2);
    const errorCalls = this.apiCallLogs.filter(log => log.status >= 400);

    return {
      totalCalls,
      avgDuration,
      slowCalls,
      errorCalls,
    };
  }

  /**
   * Get miniprogram context (getCurrentPages)
   */
  async getCurrentPages(): Promise<string[]> {
    // Would need to be implemented via DevTools protocol
    return [];
  }

  /**
   * Get app data
   */
  async getAppData(): Promise<Record<string, unknown>> {
    // Would need to be implemented via DevTools protocol
    return {};
  }

  /**
   * Execute script in miniprogram context
   */
  async executeScript(script: string): Promise<unknown> {
    // Would need to be implemented via DevTools protocol
    console.log('Execute script:', script);
    return undefined;
  }

  /**
   * Get storage data
   */
  async getStorageData(): Promise<Record<string, unknown>> {
    // Would need to be implemented via DevTools protocol
    return {};
  }

  /**
   * Clear storage data
   */
  async clearStorageData(): Promise<void> {
    // Would need to be implemented via DevTools protocol
  }

  /**
   * Get console logs
   */
  async getConsoleLogs(): Promise<string[]> {
    // Would need to be implemented via DevTools protocol
    return [];
  }

  /**
   * Execute CLI command (placeholder)
   */
  private async executeCommand(...args: string[]): Promise<string> {
    // Placeholder - would use Tauri shell
    console.log('WeChat DevTools CLI:', this.config.cliPath, args.join(' '));

    return new Promise((resolve) => {
      setTimeout(() => {
        resolve('Success');
      }, 100);
    });
  }

  /**
   * Generate adapter report
   */
  generateReport(): string {
    const lines: string[] = [];

    lines.push('# WeChat DevTools Adapter Report');
    lines.push('\n## Connection Status\n');
    lines.push(`- Connected: ${this.isConnected ? 'Yes' : 'No'}`);

    lines.push('\n## Project Info\n');
    if (this.projectInfo) {
      lines.push(`- AppID: ${this.projectInfo.appid}`);
      lines.push(`- Name: ${this.projectInfo.projectname}`);
      lines.push(`- Description: ${this.projectInfo.description}`);
    } else {
      lines.push('- No project loaded');
    }

    lines.push('\n## API Calls Analysis\n');
    const analysis = this.analyzeApiPerformance();
    lines.push(`- Total Calls: ${analysis.totalCalls}`);
    lines.push(`- Average Duration: ${analysis.avgDuration.toFixed(2)}ms`);
    lines.push(`- Slow Calls: ${analysis.slowCalls.length}`);
    lines.push(`- Error Calls: ${analysis.errorCalls.length}`);

    lines.push('\n## Recent API Calls\n');
    const recentCalls = this.apiCallLogs.slice(-10);
    for (const call of recentCalls) {
      lines.push(`- ${call.method} ${call.url} (${call.status}) - ${call.duration}ms`);
    }

    return lines.join('\n');
  }
}

export const weChatDevToolsAdapter = new WeChatDevToolsAdapter();
export type {
  WeChatDevToolsConfig,
  WeChatProjectInfo,
  WeChatCompileResult,
  WeChatPreviewResult,
  WeChatUploadResult,
  ApiCallLog,
};