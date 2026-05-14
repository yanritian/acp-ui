/**
 * Offline Cache System - stores data for offline access on mobile
 */

interface CacheEntry<T> {
  key: string;
  data: T;
  timestamp: number;
  expiresAt: number;
  size: number;
}

interface CacheConfig {
  maxSize: number; // in bytes
  defaultTTL: number; // in milliseconds
  maxEntries: number;
}

interface LogCacheEntry {
  id: string;
  type: string;
  message: string;
  timestamp: number;
  agentId: string;
}

interface ApiCallCacheEntry {
  id: string;
  url: string;
  method: string;
  status: number;
  duration: number;
  timestamp: number;
}

interface StateCacheEntry {
  agentId: string;
  status: string;
  lastUpdate: number;
  tasks: string[];
}

class OfflineCache {
  private cache: Map<string, CacheEntry<unknown>> = new Map();
  private config: CacheConfig;
  private currentSize = 0;

  // Specialized caches
  private logCache: LogCacheEntry[] = [];
  private apiCallCache: ApiCallCacheEntry[] = [];
  private stateCache: Map<string, StateCacheEntry> = new Map();

  constructor(config?: Partial<CacheConfig>) {
    this.config = {
      maxSize: 10 * 1024 * 1024, // 10MB
      defaultTTL: 30 * 60 * 1000, // 30 minutes
      maxEntries: 1000,
      ...config,
    };
  }

  /**
   * Set cache entry
   */
  set<T>(key: string, data: T, ttl?: number): boolean {
    const expiresAt = Date.now() + (ttl || this.config.defaultTTL);
    const size = this.estimateSize(data);

    // Check if we need to evict
    if (this.currentSize + size > this.config.maxSize) {
      this.evictOldest();
    }

    // Check max entries
    if (this.cache.size >= this.config.maxEntries) {
      this.evictOldest();
    }

    // Remove old entry if exists
    const existing = this.cache.get(key);
    if (existing) {
      this.currentSize -= existing.size;
    }

    // Add new entry
    const entry: CacheEntry<T> = {
      key,
      data,
      timestamp: Date.now(),
      expiresAt,
      size,
    };

    this.cache.set(key, entry);
    this.currentSize += size;

    return true;
  }

  /**
   * Get cache entry
   */
  get<T>(key: string): T | null {
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    // Check expiration
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      this.currentSize -= entry.size;
      return null;
    }

    return entry.data as T;
  }

  /**
   * Check if entry exists and is valid
   */
  has(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;
    return Date.now() <= entry.expiresAt;
  }

  /**
   * Delete entry
   */
  delete(key: string): boolean {
    const entry = this.cache.get(key);
    if (!entry) return false;

    this.cache.delete(key);
    this.currentSize -= entry.size;
    return true;
  }

  /**
   * Clear all cache
   */
  clear(): void {
    this.cache.clear();
    this.logCache = [];
    this.apiCallCache = [];
    this.stateCache.clear();
    this.currentSize = 0;
  }

  /**
   * Evict oldest entries
   */
  private evictOldest(): void {
    const entries = Array.from(this.cache.entries())
      .sort((a, b) => a[1].timestamp - b[1].timestamp);

    // Evict 10% of entries or until we have space
    const toEvict = Math.ceil(entries.length * 0.1);

    for (let i = 0; i < toEvict && this.cache.size > 0; i++) {
      const [key, entry] = entries[i];
      this.cache.delete(key);
      this.currentSize -= entry.size;
    }
  }

  /**
   * Estimate size of data
   */
  private estimateSize(data: unknown): number {
    if (typeof data === 'string') {
      return data.length * 2; // UTF-16
    }

    if (data instanceof ArrayBuffer) {
      return data.byteLength;
    }

    // Rough estimate for objects
    return JSON.stringify(data).length * 2;
  }

  /**
   * Get cache stats
   */
  getStats(): {
    entries: number;
    size: number;
    maxSize: number;
    hitRate: number;
  } {
    return {
      entries: this.cache.size,
      size: this.currentSize,
      maxSize: this.config.maxSize,
      hitRate: 0, // Would need to track hits/misses
    };
  }

  // === Specialized Cache Methods ===

  /**
   * Add log to cache (limit to 100 entries)
   */
  addLog(log: LogCacheEntry): void {
    this.logCache.push(log);

    // Limit cache size
    if (this.logCache.length > 100) {
      this.logCache.shift();
    }
  }

  /**
   * Get cached logs
   */
  getLogs(limit?: number): LogCacheEntry[] {
    return limit ? this.logCache.slice(-limit) : [...this.logCache];
  }

  /**
   * Clear log cache
   */
  clearLogs(): void {
    this.logCache = [];
  }

  /**
   * Add API call to cache (limit to 50 entries)
   */
  addApiCall(call: ApiCallCacheEntry): void {
    this.apiCallCache.push(call);

    if (this.apiCallCache.length > 50) {
      this.apiCallCache.shift();
    }
  }

  /**
   * Get cached API calls
   */
  getApiCalls(limit?: number): ApiCallCacheEntry[] {
    return limit ? this.apiCallCache.slice(-limit) : [...this.apiCallCache];
  }

  /**
   * Clear API call cache
   */
  clearApiCalls(): void {
    this.apiCallCache = [];
  }

  /**
   * Update agent state cache
   */
  updateAgentState(state: StateCacheEntry): void {
    this.stateCache.set(state.agentId, state);
  }

  /**
   * Get agent state
   */
  getAgentState(agentId: string): StateCacheEntry | undefined {
    return this.stateCache.get(agentId);
  }

  /**
   * Get all agent states
   */
  getAllAgentStates(): StateCacheEntry[] {
    return Array.from(this.stateCache.values());
  }

  /**
   * Clear state cache
   */
  clearStateCache(): void {
    this.stateCache.clear();
  }

  /**
   * Sync with server (placeholder)
   */
  async syncWithServer(): Promise<{
    synced: number;
    failed: number;
  }> {
    // Placeholder - would implement actual sync logic
    // This would push local changes and pull server updates
    return { synced: 0, failed: 0 };
  }

  /**
   * Export cache data (for backup)
   */
  export(): {
    cache: Array<[string, CacheEntry<unknown>]>;
    logs: LogCacheEntry[];
    apiCalls: ApiCallCacheEntry[];
    states: Array<[string, StateCacheEntry]>;
  } {
    return {
      cache: Array.from(this.cache.entries()),
      logs: [...this.logCache],
      apiCalls: [...this.apiCallCache],
      states: Array.from(this.stateCache.entries()),
    };
  }

  /**
   * Import cache data (for restore)
   */
  import(data: ReturnType<typeof this.export>): void {
    this.cache.clear();
    for (const [key, entry] of data.cache) {
      this.cache.set(key, entry);
    }

    this.logCache = data.logs;
    this.apiCallCache = data.apiCalls;
    this.stateCache.clear();
    for (const [key, entry] of data.states) {
      this.stateCache.set(key, entry);
    }
  }

  /**
   * Generate cache report
   */
  generateReport(): string {
    const lines: string[] = [];

    lines.push('# Offline Cache Report');
    lines.push('\n## General Cache\n');
    const stats = this.getStats();
    lines.push(`- Entries: ${stats.entries}`);
    lines.push(`- Size: ${(stats.size / 1024).toFixed(2)}KB / ${(stats.maxSize / 1024 / 1024).toFixed(2)}MB`);

    lines.push('\n## Specialized Caches\n');
    lines.push(`- Log Cache: ${this.logCache.length} entries`);
    lines.push(`- API Call Cache: ${this.apiCallCache.length} entries`);
    lines.push(`- State Cache: ${this.stateCache.size} agents`);

    lines.push('\n## Cached Logs (last 10)\n');
    const recentLogs = this.logCache.slice(-10);
    for (const log of recentLogs) {
      lines.push(`- [${log.type}] ${log.message}`);
    }

    lines.push('\n## Cached API Calls (last 10)\n');
    const recentCalls = this.apiCallCache.slice(-10);
    for (const call of recentCalls) {
      lines.push(`- ${call.method} ${call.url} (${call.status}) - ${call.duration}ms`);
    }

    return lines.join('\n');
  }
}

export const offlineCache = new OfflineCache();
export type {
  CacheEntry,
  CacheConfig,
  LogCacheEntry,
  ApiCallCacheEntry,
  StateCacheEntry,
};