/**
 * HBuilderX Adapter - controls HBuilderX for miniprogram development
 */

interface HBuilderXConfig {
  cliPath: string;
  projectPath: string;
  outputPath: string;
  timeout: number;
}

interface CompileResult {
  success: boolean;
  outputPath: string;
  duration: number;
  errors: CompileError[];
  warnings: CompileWarning[];
}

interface CompileError {
  file: string;
  line: number;
  column: number;
  message: string;
  code: string;
}

interface CompileWarning {
  file: string;
  line: number;
  message: string;
}

interface RunResult {
  success: boolean;
  platform: 'h5' | 'mp-weixin' | 'mp-alipay' | 'app-android' | 'app-ios';
  deviceId?: string;
  deviceName?: string;
  devServerUrl?: string;
  logs: string[];
}

interface ProjectInfo {
  name: string;
  type: 'uniapp' | 'vue' | 'nvue';
  version: string;
  dependencies: Record<string, string>;
}

interface DeviceInfo {
  id: string;
  name: string;
  type: 'android' | 'ios' | 'web' | 'devtools';
  connected: boolean;
}

class HBuilderXAdapter {
  private config: HBuilderXConfig;
  private runningProcess: number | null = null;
  private logBuffer: string[] = [];
  private devices: Map<string, DeviceInfo> = new Map();
  private projectInfo: ProjectInfo | null = null;

  constructor(config?: Partial<HBuilderXConfig>) {
    this.config = {
      cliPath: 'hbuilderx-cli',
      projectPath: './',
      outputPath: './unpackage',
      timeout: 120000,
      ...config,
    };
  }

  /**
   * Initialize HBuilderX CLI
   */
  async init(): Promise<boolean> {
    // Check if CLI is available
    try {
      const version = await this.executeCommand('--version');
      console.log('HBuilderX CLI version:', version);
      return true;
    } catch (error) {
      console.error('HBuilderX CLI not available:', error);
      return false;
    }
  }

  /**
   * Open project in HBuilderX
   */
  async openProject(projectPath: string): Promise<boolean> {
    try {
      await this.executeCommand('open', projectPath);
      this.config.projectPath = projectPath;
      return true;
    } catch (error) {
      console.error('Failed to open project:', error);
      return false;
    }
  }

  /**
   * Get project info
   */
  async getProjectInfo(): Promise<ProjectInfo | null> {
    try {
      // Parse manifest.json or package.json — simplified
      await this.executeCommand('project-info', this.config.projectPath);

      // Parse result — simplified
      const info: ProjectInfo = {
        name: 'unknown',
        type: 'uniapp',
        version: '1.0.0',
        dependencies: {},
      };

      this.projectInfo = info;
      return info;
    } catch (error) {
      console.error('Failed to get project info:', error);
      return null;
    }
  }

  /**
   * Compile project
   */
  async compile(platform: 'h5' | 'mp-weixin' | 'mp-alipay' | 'app'): Promise<CompileResult> {
    const startTime = Date.now();
    const errors: CompileError[] = [];
    const warnings: CompileWarning[] = [];

    try {
      const output = await this.executeCommand(
        'compile',
        '--platform', platform,
        '--project', this.config.projectPath,
        '--output', this.config.outputPath,
        '--no-cache'
      );

      // Parse compile output
      const parsed = this.parseCompileOutput(output);
      errors.push(...parsed.errors);
      warnings.push(...parsed.warnings);

      const duration = Date.now() - startTime;

      return {
        success: errors.length === 0,
        outputPath: this.config.outputPath,
        duration,
        errors,
        warnings,
      };
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      errors.push({
        file: '',
        line: 0,
        column: 0,
        message: errorMsg,
        code: 'COMPILE_ERROR',
      });

      return {
        success: false,
        outputPath: '',
        duration: Date.now() - startTime,
        errors,
        warnings,
      };
    }
  }

  /**
   * Parse compile output
   */
  private parseCompileOutput(output: string): { errors: CompileError[]; warnings: CompileWarning[] } {
    const errors: CompileError[] = [];
    const warnings: CompileWarning[] = [];
    const lines = output.split('\n');

    for (const line of lines) {
      // HBuilderX error format: ERROR in file:line:column - message
      const errorMatch = line.match(/ERROR\s+in\s+(.+?):(\d+):(\d+)\s+-\s+(.+)$/);
      if (errorMatch) {
        errors.push({
          file: errorMatch[1],
          line: parseInt(errorMatch[2]),
          column: parseInt(errorMatch[3]),
          message: errorMatch[4],
          code: 'HBUILDER_ERROR',
        });
        continue;
      }

      // Alternative error format
      const altErrorMatch = line.match(/(.+?):(\d+):\s+error:\s+(.+)$/);
      if (altErrorMatch) {
        errors.push({
          file: altErrorMatch[1],
          line: parseInt(altErrorMatch[2]),
          column: 0,
          message: altErrorMatch[3],
          code: 'HBUILDER_ERROR',
        });
        continue;
      }

      // Warning format
      const warningMatch = line.match(/WARNING\s+in\s+(.+?):(\d+)\s+-\s+(.+)$/);
      if (warningMatch) {
        warnings.push({
          file: warningMatch[1],
          line: parseInt(warningMatch[2]),
          message: warningMatch[3],
        });
      }
    }

    return { errors, warnings };
  }

  /**
   * Run project in development mode
   */
  async run(platform: 'h5' | 'mp-weixin' | 'app-android' | 'app-ios', deviceId?: string): Promise<RunResult> {
    this.logBuffer = [];

    try {
      const args = ['run', '--platform', platform, '--project', this.config.projectPath];

      if (deviceId) {
        args.push('--device', deviceId);
      }

      // Start dev server (background process)
      const output = await this.executeCommand(...args);

      // Parse dev server info
      const devServerMatch = output.match(/Dev server running at (http:\/\/[\d.]+:\d+)/);
      const devServerUrl = devServerMatch ? devServerMatch[1] : undefined;

      // Parse device info
      const deviceMatch = output.match(/Running on (.+) \((.+)\)/);

      return {
        success: true,
        platform: platform as RunResult['platform'],
        deviceId: deviceId || deviceMatch?.[2],
        deviceName: deviceMatch?.[1],
        devServerUrl,
        logs: this.logBuffer,
      };
    } catch (error) {
      return {
        success: false,
        platform: platform as RunResult['platform'],
        logs: [error instanceof Error ? error.message : String(error)],
      };
    }
  }

  /**
   * Stop running process
   */
  async stop(): Promise<boolean> {
    if (this.runningProcess) {
      try {
        await this.executeCommand('stop', '--process', String(this.runningProcess));
        this.runningProcess = null;
        return true;
      } catch {
        return false;
      }
    }
    return true;
  }

  /**
   * Get available devices
   */
  async getDevices(): Promise<DeviceInfo[]> {
    try {
      const output = await this.executeCommand('devices');

      // Parse device list
      const lines = output.split('\n');
      const devices: DeviceInfo[] = [];

      for (const line of lines) {
        const match = line.match(/(.+?)\s+\((.+?)\)\s+(connected|disconnected)/);
        if (match) {
          const device: DeviceInfo = {
            id: match[2],
            name: match[1],
            type: this.getDeviceType(match[2]),
            connected: match[3] === 'connected',
          };
          devices.push(device);
          this.devices.set(device.id, device);
        }
      }

      return devices;
    } catch {
      return [];
    }
  }

  /**
   * Get device type from ID
   */
  private getDeviceType(id: string): DeviceInfo['type'] {
    if (id.includes('android')) return 'android';
    if (id.includes('ios') || id.includes('iphone')) return 'ios';
    if (id.includes('devtools')) return 'devtools';
    return 'web';
  }

  /**
   * Get build logs
   */
  getLogs(): string[] {
    return [...this.logBuffer];
  }

  /**
   * Clear logs
   */
  clearLogs(): void {
    this.logBuffer = [];
  }

  /**
   * Build for production
   */
  async build(platform: 'h5' | 'mp-weixin' | 'app-android' | 'app-ios'): Promise<CompileResult> {
    const startTime = Date.now();

    try {
      const output = await this.executeCommand(
        'build',
        '--platform', platform,
        '--project', this.config.projectPath,
        '--output', this.config.outputPath,
        '--production'
      );

      const parsed = this.parseCompileOutput(output);

      return {
        success: parsed.errors.length === 0,
        outputPath: `${this.config.outputPath}/${platform}`,
        duration: Date.now() - startTime,
        errors: parsed.errors,
        warnings: parsed.warnings,
      };
    } catch (error) {
      return {
        success: false,
        outputPath: '',
        duration: Date.now() - startTime,
        errors: [{
          file: '',
          line: 0,
          column: 0,
          message: error instanceof Error ? error.message : String(error),
          code: 'BUILD_ERROR',
        }],
        warnings: [],
      };
    }
  }

  /**
   * Package APK/IPA
   */
  async package(platform: 'app-android' | 'app-ios', options?: {
    appName?: string;
    appId?: string;
    version?: string;
  }): Promise<{ success: boolean; outputPath: string; error?: string }> {
    try {
      const args = [
        'package',
        '--platform', platform.replace('app-', ''),
        '--project', this.config.projectPath,
        '--output', this.config.outputPath,
      ];

      if (options?.appName) args.push('--name', options.appName);
      if (options?.appId) args.push('--appid', options.appId);
      if (options?.version) args.push('--version', options.version);

      await this.executeCommand(...args);

      const outputPath = platform === 'app-android'
        ? `${this.config.outputPath}/android/release.apk`
        : `${this.config.outputPath}/ios/release.ipa`;

      return { success: true, outputPath };
    } catch (error) {
      return {
        success: false,
        outputPath: '',
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * Execute CLI command — TODO(Phase 4): use Tauri shell
   */
  private async executeCommand(...args: string[]): Promise<string> {
    // Simplified implementation — TODO(Phase 4): full integration
    // Real implementation would use Tauri's shell plugin
    console.log('HBuilderX CLI command:', this.config.cliPath, args.join(' '));

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

    lines.push('# HBuilderX Adapter Report');
    lines.push('\n## Project Info\n');
    if (this.projectInfo) {
      lines.push(`- Name: ${this.projectInfo.name}`);
      lines.push(`- Type: ${this.projectInfo.type}`);
      lines.push(`- Version: ${this.projectInfo.version}`);
    } else {
      lines.push('- No project loaded');
    }

    lines.push('\n## Devices\n');
    for (const device of this.devices.values()) {
      lines.push(`- ${device.name} (${device.id}): ${device.connected ? 'Connected' : 'Disconnected'}`);
    }

    lines.push('\n## Recent Logs\n');
    const recentLogs = this.logBuffer.slice(-20);
    for (const log of recentLogs) {
      lines.push(`- ${log}`);
    }

    return lines.join('\n');
  }
}

export const hbuilderXAdapter = new HBuilderXAdapter();
export type {
  HBuilderXConfig,
  CompileResult,
  CompileError,
  CompileWarning,
  RunResult,
  ProjectInfo,
  DeviceInfo,
};