// Unified Plugin Manifest - Single format for Skills/MCP/Hooks/CLI/Adapters
// All plugins use the same registration format

import { ref, type Ref } from 'vue';
import { invokeOrProxy } from '../host';

export type PluginKind =
  | 'skill'
  | 'mcp'
  | 'hook'
  | 'cli-extension'
  | 'im-channel'
  | 'model-router'
  | 'agent-adapter';

export type PluginEntryType =
  | 'typescript'
  | 'rust'
  | 'wasm'
  | 'mcp-server'
  | 'binary';

export interface PluginManifest {
  id: string;                           // "skill.code-review" | "mcp.github"
  kind: PluginKind;
  name: string;
  version: string;
  description: string;
  capabilities: string[];
  entry: {
    type: PluginEntryType;
    path?: string;                      // For typescript/wasm
    command?: string;                   // For binary/mcp-server
    args?: string[];
  };
  dependencies?: string[];              // Plugin IDs this depends on
  config?: Record<string, ConfigField>;
  enabled?: boolean;
}

export interface ConfigField {
  type: 'string' | 'number' | 'boolean' | 'select' | 'array';
  label: string;
  default?: unknown;
  options?: string[];                   // For select type
  required?: boolean;
  secret?: boolean;                     // Mark as sensitive (don't log)
}

export interface PluginInstance {
  id: string;
  manifest: PluginManifest;
  status: 'installed' | 'enabled' | 'disabled' | 'error' | 'loading';
  loadedAt?: number;
  error?: string;
  configValues?: Record<string, unknown>;
}

export class PluginRegistry {
  private plugins: Ref<Map<string, PluginInstance>> = ref(new Map());
  private loadingPlugins: Ref<string[]> = ref([]);

  // State for UI
  public pluginCount: Ref<number> = ref(0);
  public enabledCount: Ref<number> = ref(0);

  // Register plugin from manifest
  async register(manifest: PluginManifest): Promise<PluginInstance> {
    // Check dependencies
    if (manifest.dependencies) {
      for (const depId of manifest.dependencies) {
        const dep = this.plugins.value.get(depId);
        if (!dep || dep.status !== 'enabled') {
          throw new Error(`Dependency ${depId} not available`);
        }
      }
    }

    // Call backend to register
    await invokeOrProxy('plugin_register', { manifest });

    const instance: PluginInstance = {
      id: manifest.id,
      manifest,
      status: 'installed',
      loadedAt: Date.now(),
      configValues: this.getDefaultConfig(manifest),
    };

    this.plugins.value.set(manifest.id, instance);
    this.updateCounts();

    return instance;
  }

  // Get default config values
  private getDefaultConfig(manifest: PluginManifest): Record<string, unknown> {
    const defaults: Record<string, unknown> = {};
    if (manifest.config) {
      for (const [key, field] of Object.entries(manifest.config)) {
        defaults[key] = field.default;
      }
    }
    return defaults;
  }

  // Enable plugin
  async enable(pluginId: string): Promise<void> {
    const instance = this.plugins.value.get(pluginId);
    if (!instance) {
      throw new Error(`Plugin ${pluginId} not found`);
    }

    instance.status = 'loading';
    this.loadingPlugins.value.push(pluginId);

    try {
      // Load plugin entry
      await this.loadPluginEntry(instance.manifest);

      // Apply config
      if (instance.configValues) {
        await invokeOrProxy('plugin_configure', {
          plugin_id: pluginId,
          config: instance.configValues,
        });
      }

      instance.status = 'enabled';
    } catch (e) {
      instance.status = 'error';
      instance.error = String(e);
    }

    this.loadingPlugins.value = this.loadingPlugins.value.filter(id => id !== pluginId);
    this.updateCounts();
  }

  // Disable plugin
  async disable(pluginId: string): Promise<void> {
    const instance = this.plugins.value.get(pluginId);
    if (!instance) {
      throw new Error(`Plugin ${pluginId} not found`);
    }

    await invokeOrProxy('plugin_disable', { plugin_id: pluginId });
    instance.status = 'disabled';
    this.updateCounts();
  }

  // Unregister plugin
  async unregister(pluginId: string): Promise<void> {
    // Check if other plugins depend on this
    for (const [id, instance] of this.plugins.value) {
      if (instance.manifest.dependencies?.includes(pluginId)) {
        throw new Error(`Plugin ${id} depends on ${pluginId}`);
      }
    }

    await invokeOrProxy('plugin_unregister', { plugin_id: pluginId });
    this.plugins.value.delete(pluginId);
    this.updateCounts();
  }

  // Load plugin entry based on type
  private async loadPluginEntry(manifest: PluginManifest): Promise<void> {
    switch (manifest.entry.type) {
      case 'typescript':
        // Dynamic import
        if (manifest.entry.path) {
          await import(/* @vite-ignore */ manifest.entry.path);
        }
        break;

      case 'mcp-server':
        // Start MCP server process
        await invokeOrProxy('mcp_start_server', {
          command: manifest.entry.command,
          args: manifest.entry.args,
          name: manifest.name,
        });
        break;

      case 'binary':
        // Start binary process
        await invokeOrProxy('plugin_start_binary', {
          command: manifest.entry.command,
          args: manifest.entry.args,
          plugin_id: manifest.id,
        });
        break;

      case 'rust':
      case 'wasm':
        // Already compiled in Tauri
        break;
    }
  }

  // Update config
  async updateConfig(pluginId: string, config: Record<string, unknown>): Promise<void> {
    const instance = this.plugins.value.get(pluginId);
    if (!instance) {
      throw new Error(`Plugin ${pluginId} not found`);
    }

    // Validate config
    this.validateConfig(instance.manifest, config);

    // Apply config
    await invokeOrProxy('plugin_configure', { plugin_id: pluginId, config });

    instance.configValues = config;
  }

  // Validate config against manifest
  private validateConfig(manifest: PluginManifest, config: Record<string, unknown>): void {
    if (!manifest.config) return;

    for (const [key, field] of Object.entries(manifest.config)) {
      if (field.required && config[key] === undefined) {
        throw new Error(`Required config field ${key} missing`);
      }

      if (config[key] !== undefined) {
        // Type validation
        const value = config[key];
        switch (field.type) {
          case 'string':
            if (typeof value !== 'string') {
              throw new Error(`Config ${key} must be string`);
            }
            break;
          case 'number':
            if (typeof value !== 'number') {
              throw new Error(`Config ${key} must be number`);
            }
            break;
          case 'boolean':
            if (typeof value !== 'boolean') {
              throw new Error(`Config ${key} must be boolean`);
            }
            break;
          case 'select':
            if (!field.options?.includes(value as string)) {
              throw new Error(`Config ${key} must be one of: ${field.options?.join(',')}`);
            }
            break;
        }
      }
    }
  }

  // Update counts
  private updateCounts(): void {
    this.pluginCount.value = this.plugins.value.size;
    this.enabledCount.value = Array.from(this.plugins.value.values())
      .filter(p => p.status === 'enabled').length;
  }

  // Get all plugins
  getPlugins(): PluginInstance[] {
    return Array.from(this.plugins.value.values());
  }

  // Get plugin by ID
  getPlugin(pluginId: string): PluginInstance | undefined {
    return this.plugins.value.get(pluginId);
  }

  // Get plugins by kind
  getPluginsByKind(kind: PluginKind): PluginInstance[] {
    return Array.from(this.plugins.value.values())
      .filter(p => p.manifest.kind === kind);
  }

  // Search plugins
  searchPlugins(query: string): PluginInstance[] {
    const lowerQuery = query.toLowerCase();
    return Array.from(this.plugins.value.values())
      .filter(p =>
        p.manifest.name.toLowerCase().includes(lowerQuery) ||
        p.manifest.description.toLowerCase().includes(lowerQuery) ||
        p.manifest.capabilities.some(c => c.toLowerCase().includes(lowerQuery))
      );
  }

  // Export all manifests
  exportManifests(): PluginManifest[] {
    return Array.from(this.plugins.value.values())
      .map(p => p.manifest);
  }
}

// Singleton instance
let pluginRegistryInstance: PluginRegistry | null = null;

export function usePluginRegistry(): PluginRegistry {
  if (!pluginRegistryInstance) {
    pluginRegistryInstance = new PluginRegistry();
  }
  return pluginRegistryInstance;
}

// Default plugin manifests for built-in capabilities
export const DEFAULT_PLUGINS: PluginManifest[] = [
  {
    id: 'skill.code-review',
    kind: 'skill',
    name: 'Code Review',
    version: '1.0.0',
    description: 'Automated code review with security analysis',
    capabilities: ['review', 'security', 'lint'],
    entry: { type: 'typescript', path: '@/lib/skill-system/dev-flow-skills.ts' },
    config: {
      autoReview: { type: 'boolean', label: 'Auto-review on commit', default: false },
      severityLevel: { type: 'select', label: 'Severity threshold', default: 'medium', options: ['low', 'medium', 'high'] },
    },
  },
  {
    id: 'mcp.github',
    kind: 'mcp',
    name: 'GitHub MCP',
    version: '1.0.0',
    description: 'GitHub integration via MCP',
    capabilities: ['github', 'repo', 'issues', 'prs'],
    entry: { type: 'mcp-server', command: 'github-mcp-server' },
    config: {
      token: { type: 'string', label: 'GitHub Token', required: true, secret: true },
    },
  },
  {
    id: 'hook.self-heal',
    kind: 'hook',
    name: 'Self-Healing Hook',
    version: '1.0.0',
    description: 'Auto-recovery on errors',
    capabilities: ['self-healing', 'error-handling'],
    entry: { type: 'rust' },
    config: {
      maxRetries: { type: 'number', label: 'Max retries', default: 3 },
      retryDelayMs: { type: 'number', label: 'Retry delay (ms)', default: 1000 },
    },
  },
  {
    id: 'im.feishu',
    kind: 'im-channel',
    name: 'Feishu Bot',
    version: '1.0.0',
    description: 'Feishu/Lark integration for remote control',
    capabilities: ['feishu', 'remote-approve', 'notifications'],
    entry: { type: 'typescript', path: '@/lib/im-channels/channel-registry.ts' },
    config: {
      appId: { type: 'string', label: 'App ID', required: true },
      appSecret: { type: 'string', label: 'App Secret', required: true, secret: true },
    },
  },
];