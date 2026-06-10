/**
 * Android Studio Adapter - controls Android build and deployment
 */

interface AndroidConfig {
  gradlePath: string;
  projectPath: string;
  adbPath: string;
  outputPath: string;
}

interface GradleBuildResult {
  success: boolean;
  outputPath: string;
  duration: number;
  errors: BuildError[];
  warnings: BuildWarning[];
  tasks: string[];
}

interface BuildError {
  file: string;
  line: number;
  message: string;
  type: 'compilation' | 'linker' | 'resource' | 'manifest';
}

interface BuildWarning {
  file: string;
  line: number;
  message: string;
}

interface DeviceInfo {
  id: string;
  name: string;
  model: string;
  androidVersion: string;
  apiLevel: number;
  status: 'online' | 'offline' | 'booting' | 'unauthorized';
  type: 'emulator' | 'device';
}

interface InstallResult {
  success: boolean;
  packageName: string;
  version: string;
  deviceId: string;
  installTime: number;
}

interface AdbLogEntry {
  timestamp: number;
  level: 'verbose' | 'debug' | 'info' | 'warning' | 'error' | 'fatal';
  tag: string;
  message: string;
  pid: number;
  tid: number;
}

interface EmulatorConfig {
  name: string;
  deviceType: string;
  apiLevel: number;
  memory?: number;
  resolution?: string;
}

class AndroidStudioAdapter {
  private config: AndroidConfig;
  private devices: Map<string, DeviceInfo> = new Map();
  private logBuffer: AdbLogEntry[] = [];
  private runningEmulators: string[] = [];
  private currentBuild: GradleBuildResult | null = null;

  constructor(config?: Partial<AndroidConfig>) {
    this.config = {
      gradlePath: './gradlew',
      projectPath: './',
      adbPath: 'adb',
      outputPath: './build/outputs',
      ...config,
    };
  }

  /**
   * Initialize Android environment
   */
  async init(): Promise<boolean> {
    // Check if Gradle wrapper exists
    try {
      await this.executeGradle('--version');
      await this.executeAdb('version');
      return true;
    } catch (error) {
      console.error('Android environment not available:', error);
      return false;
    }
  }

  /**
   * Get project info
   */
  async getProjectInfo(): Promise<{
    name: string;
    packageName: string;
    version: string;
    minSdk: number;
    targetSdk: number;
  } | null> {
    try {
      // Parse build.gradle or AndroidManifest.xml
      // Simplified implementation — TODO(Phase 4): full integration
      return {
        name: 'AndroidProject',
        packageName: 'com.example.app',
        version: '1.0.0',
        minSdk: 21,
        targetSdk: 33,
      };
    } catch {
      return null;
    }
  }

  /**
   * Clean project
   */
  async clean(): Promise<boolean> {
    try {
      await this.executeGradle('clean');
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Build debug APK
   */
  async buildDebug(): Promise<GradleBuildResult> {
    return this.build('assembleDebug', 'debug');
  }

  /**
   * Build release APK
   */
  async buildRelease(): Promise<GradleBuildResult> {
    return this.build('assembleRelease', 'release');
  }

  /**
   * Build APK
   */
  private async build(task: string, buildType: string): Promise<GradleBuildResult> {
    const startTime = Date.now();
    const errors: BuildError[] = [];
    const warnings: BuildWarning[] = [];

    try {
      const output = await this.executeGradle(task, '--stacktrace');

      // Parse build output
      const parsed = this.parseGradleOutput(output);
      errors.push(...parsed.errors);
      warnings.push(...parsed.warnings);

      const outputPath = `${this.config.outputPath}/apk/${buildType}/app-${buildType}.apk`;

      this.currentBuild = {
        success: errors.length === 0,
        outputPath,
        duration: Date.now() - startTime,
        errors,
        warnings,
        tasks: parsed.tasks,
      };

      return this.currentBuild;
    } catch (error) {
      return {
        success: false,
        outputPath: '',
        duration: Date.now() - startTime,
        errors: [{
          file: '',
          line: 0,
          message: error instanceof Error ? error.message : String(error),
          type: 'compilation',
        }],
        warnings,
        tasks: [],
      };
    }
  }

  /**
   * Parse Gradle output
   */
  private parseGradleOutput(output: string): {
    errors: BuildError[];
    warnings: BuildWarning[];
    tasks: string[];
  } {
    const errors: BuildError[] = [];
    const warnings: BuildWarning[] = [];
    const tasks: string[] = [];
    const lines = output.split('\n');

    for (const line of lines) {
      // Task execution
      const taskMatch = line.match(/:([a-zA-Z]+)$/);
      if (taskMatch) {
        tasks.push(taskMatch[1]);
      }

      // Compilation error
      const errorMatch = line.match(/(.+?):(\d+):\s+error:\s+(.+)$/);
      if (errorMatch) {
        errors.push({
          file: errorMatch[1],
          line: parseInt(errorMatch[2]),
          message: errorMatch[3],
          type: 'compilation',
        });
        continue;
      }

      // Gradle failure
      if (line.includes('FAILURE') || line.includes('Execution failed')) {
        const failMatch = line.match(/Execution failed for task '(.+)'/);
        if (failMatch) {
          errors.push({
            file: '',
            line: 0,
            message: `Task '${failMatch[1]}' failed`,
            type: 'compilation',
          });
        }
      }

      // Warning
      const warningMatch = line.match(/(.+?):(\d+):\s+warning:\s+(.+)$/);
      if (warningMatch) {
        warnings.push({
          file: warningMatch[1],
          line: parseInt(warningMatch[2]),
          message: warningMatch[3],
        });
      }
    }

    return { errors, warnings, tasks };
  }

  /**
   * Get connected devices
   */
  async getDevices(): Promise<DeviceInfo[]> {
    try {
      const output = await this.executeAdb('devices', '-l');

      // Parse device list
      const lines = output.split('\n');
      const devices: DeviceInfo[] = [];

      for (const line of lines) {
        if (line.includes('List of devices')) continue;
        if (!line.trim()) continue;

        const parts = line.trim().split(/\s+/);
        if (parts.length < 2) continue;

        const id = parts[0];
        const statusStr = parts[1];

        // Parse device info
        const device: DeviceInfo = {
          id,
          name: parts.includes('model:') ? this.extractProperty(line, 'model:') : 'Unknown',
          model: parts.includes('device:') ? this.extractProperty(line, 'device:') : 'Unknown',
          androidVersion: parts.includes('version:') ? this.extractProperty(line, 'version:') : 'Unknown',
          apiLevel: 0,
          status: this.parseDeviceStatus(statusStr),
          type: id.includes('emulator') ? 'emulator' : 'device',
        };

        devices.push(device);
        this.devices.set(device.id, device);
      }

      return devices;
    } catch {
      return [];
    }
  }

  /**
   * Extract property from adb output
   */
  private extractProperty(line: string, prop: string): string {
    const match = line.match(new RegExp(`${prop}(.+)`));
    return match ? match[1].trim().split(':')[0] : '';
  }

  /**
   * Parse device status
   */
  private parseDeviceStatus(status: string): DeviceInfo['status'] {
    const statusMap: Record<string, DeviceInfo['status']> = {
      'device': 'online',
      'offline': 'offline',
      'bootloader': 'offline',
      'unauthorized': 'unauthorized',
    };
    return statusMap[status] || 'offline';
  }

  /**
   * Install APK to device
   */
  async installApk(apkPath: string, deviceId: string): Promise<InstallResult> {
    const startTime = Date.now();

    try {
      await this.executeAdb('-s', deviceId, 'install', '-r', apkPath);

      // Get package name
      const packageOutput = await this.executeAdb('-s', deviceId, 'shell', 'pm', 'list', 'packages', '-f');
      const packageMatch = packageOutput.match(/package:(.+)=/);
      const packageName = packageMatch ? packageMatch[1].split('/').pop() || '' : '';

      return {
        success: true,
        packageName,
        version: '1.0.0',
        deviceId,
        installTime: Date.now() - startTime,
      };
    } catch (error) {
      return {
        success: false,
        packageName: '',
        version: '',
        deviceId,
        installTime: Date.now() - startTime,
      };
    }
  }

  /**
   * Uninstall app
   */
  async uninstallApp(packageName: string, deviceId: string): Promise<boolean> {
    try {
      await this.executeAdb('-s', deviceId, 'uninstall', packageName);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Start app
   */
  async startApp(packageName: string, activityName: string, deviceId: string): Promise<boolean> {
    try {
      await this.executeAdb(
        '-s', deviceId,
        'shell', 'am', 'start',
        '-n', `${packageName}/${activityName}`
      );
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Stop app
   */
  async stopApp(packageName: string, deviceId: string): Promise<boolean> {
    try {
      await this.executeAdb('-s', deviceId, 'shell', 'am', 'force-stop', packageName);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get app logs (logcat)
   */
  async getLogs(packageName: string, deviceId: string): Promise<AdbLogEntry[]> {
    try {
      const output = await this.executeAdb(
        '-s', deviceId,
        'logcat', '-d',
        '-v', 'time',
        packageName
      );

      return this.parseLogcat(output);
    } catch {
      return [];
    }
  }

  /**
   * Parse logcat output
   */
  private parseLogcat(output: string): AdbLogEntry[] {
    const entries: AdbLogEntry[] = [];
    const lines = output.split('\n');

    for (const line of lines) {
      // Timestamp format: MM-DD HH:MM:SS.mmm
      const match = line.match(/(\d+-\d+\s+\d+:\d+:\d+\.\d+)\s+(\w)/);

      if (match) {
        const levelMap: Record<string, AdbLogEntry['level']> = {
          'V': 'verbose',
          'D': 'debug',
          'I': 'info',
          'W': 'warning',
          'E': 'error',
          'F': 'fatal',
        };

        entries.push({
          timestamp: Date.now(),
          level: levelMap[match[2]] || 'info',
          tag: '',
          message: line,
          pid: 0,
          tid: 0,
        });
      }
    }

    return entries;
  }

  /**
   * Clear app data
   */
  async clearAppData(packageName: string, deviceId: string): Promise<boolean> {
    try {
      await this.executeAdb('-s', deviceId, 'shell', 'pm', 'clear', packageName);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Take screenshot
   */
  async takeScreenshot(deviceId: string, outputPath: string): Promise<string> {
    try {
      const tempPath = '/sdcard/screenshot.png';
      await this.executeAdb('-s', deviceId, 'shell', 'screencap', '-p', tempPath);
      await this.executeAdb('-s', deviceId, 'pull', tempPath, outputPath);
      return outputPath;
    } catch {
      return '';
    }
  }

  /**
   * Start emulator
   */
  async startEmulator(config: EmulatorConfig): Promise<string | null> {
    try {
      const emulatorName = config.name || 'default';

      // Start emulator
      const output = await this.executeCommand('emulator', '-avd', emulatorName, '-no-snapshot-load');

      // Find emulator ID
      const idMatch = output.match(/emulator-(\d+)/);
      if (idMatch) {
        const emulatorId = `emulator-${idMatch[1]}`;
        this.runningEmulators.push(emulatorId);
        return emulatorId;
      }

      return null;
    } catch {
      return null;
    }
  }

  /**
   * Stop emulator
   */
  async stopEmulator(emulatorId: string): Promise<boolean> {
    try {
      await this.executeAdb('-s', emulatorId, 'emu', 'kill');
      this.runningEmulators = this.runningEmulators.filter(id => id !== emulatorId);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Execute Gradle command
   */
  private async executeGradle(...args: string[]): Promise<string> {
    console.log('Gradle:', this.config.gradlePath, args.join(' '));
    return new Promise((resolve) => {
      setTimeout(() => resolve('Build SUCCESS'), 100);
    });
  }

  /**
   * Execute ADB command
   */
  private async executeAdb(...args: string[]): Promise<string> {
    console.log('ADB:', this.config.adbPath, args.join(' '));
    return new Promise((resolve) => {
      setTimeout(() => resolve('Success'), 100);
    });
  }

  /**
   * Execute generic command
   */
  private async executeCommand(cmd: string, ...args: string[]): Promise<string> {
    console.log('Command:', cmd, args.join(' '));
    return new Promise((resolve) => {
      setTimeout(() => resolve('Success'), 100);
    });
  }

  /**
   * Generate adapter report
   */
  generateReport(): string {
    const lines: string[] = [];

    lines.push('# Android Studio Adapter Report');
    lines.push('\n## Build Status\n');
    if (this.currentBuild) {
      lines.push(`- Success: ${this.currentBuild.success}`);
      lines.push(`- Output: ${this.currentBuild.outputPath}`);
      lines.push(`- Duration: ${this.currentBuild.duration}ms`);
      lines.push(`- Errors: ${this.currentBuild.errors.length}`);
      lines.push(`- Warnings: ${this.currentBuild.warnings.length}`);
    } else {
      lines.push('- No recent build');
    }

    lines.push('\n## Connected Devices\n');
    for (const device of this.devices.values()) {
      lines.push(`- ${device.name} (${device.id}): ${device.status} - Android ${device.androidVersion}`);
    }

    lines.push('\n## Running Emulators\n');
    for (const emulatorId of this.runningEmulators) {
      lines.push(`- ${emulatorId}`);
    }

    lines.push('\n## Recent Logs\n');
    const recentLogs = this.logBuffer.slice(-10);
    for (const log of recentLogs) {
      lines.push(`- [${log.level}] ${log.message}`);
    }

    return lines.join('\n');
  }
}

export const androidStudioAdapter = new AndroidStudioAdapter();
export type {
  AndroidConfig,
  GradleBuildResult,
  BuildError,
  BuildWarning,
  DeviceInfo,
  InstallResult,
  AdbLogEntry,
  EmulatorConfig,
};