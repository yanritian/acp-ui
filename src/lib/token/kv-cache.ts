// KV Cache Manager - Token optimization through prompt caching
// Based on OpenClacky's 90.6% cache hit rate methodology

import { ref, computed, type Ref } from 'vue';

export interface CacheEntry {
  hash: string;              // FNV-1a 64-bit hash of prompt
  promptPreview: string;     // First 100 chars for debugging
  response: string;          // Cached response
  timestamp: number;         // When cached
  hitCount: number;          // How many times reused
  tokenCount: number;        // Tokens saved
  metadata: {
    model: string;
    sessionId: string;
    agentId?: string;
  };
}

export interface CacheStats {
  hitRate: number;           // Percentage of requests that hit cache
  totalHits: number;
  totalMisses: number;
  tokensSaved: number;       // Total tokens saved through caching
  cacheSize: number;         // Number of entries
  maxCacheSize: number;      // Maximum entries
}

export interface CacheHit {
  entry: CacheEntry;
  similarity: number;        // 0-1, how similar the prompt is
  isExactMatch: boolean;
}

export class KVCacheManager {
  private cache = new Map<string, CacheEntry>();
  private maxCacheSize = 1000;  // Maximum cache entries
  private maxAgeMs = 3600000;   // 1 hour TTL
  private minHitCountForKeep = 2;  // Keep entries with >= 2 hits

  // State for UI binding
  public stats: Ref<CacheStats> = ref({
    hitRate: 0,
    totalHits: 0,
    totalMisses: 0,
    tokensSaved: 0,
    cacheSize: 0,
    maxCacheSize: this.maxCacheSize,
  });

  // FNV-1a 64-bit hash using BigInt for true 64-bit precision
  private hashPrompt(prompt: string): string {
    const FNV_OFFSET_BASIS = 0xcbf29ce484222325n;
    const FNV_PRIME = 0x100000001b3n;
    const MASK_64 = 0xffffffffffffffffn;
    let hash = FNV_OFFSET_BASIS;

    for (let i = 0; i < prompt.length; i++) {
      hash ^= BigInt(prompt.charCodeAt(i));
      hash = (hash * FNV_PRIME) & MASK_64;
    }

    return hash.toString(16).padStart(16, '0');
  }

  // Normalize prompt for hashing (remove variable parts)
  private normalizePrompt(prompt: string): string {
    // Remove timestamps, session IDs, random UUIDs
    let normalized = prompt;

    // Remove common variable patterns
    normalized = normalized.replace(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/g, '<TIMESTAMP>');
    normalized = normalized.replace(/[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}/gi, '<UUID>');
    normalized = normalized.replace(/\d{10,13}/g, '<UNIX_TIME>');

    return normalized;
  }

  // Look up in cache
  lookup(prompt: string, context?: string): CacheHit | null {
    const normalized = this.normalizePrompt(prompt);
    const hash = this.hashPrompt(normalized);
    const entry = this.cache.get(hash);

    if (entry) {
      // Check if entry is still valid
      if (Date.now() - entry.timestamp > this.maxAgeMs) {
        this.cache.delete(hash);
        this.updateStats();
        return null;
      }

      // Update hit count
      entry.hitCount++;
      entry.timestamp = Date.now();

      this.stats.value.totalHits++;
      this.stats.value.tokensSaved += entry.tokenCount;
      this.updateStats();

      return {
        entry,
        similarity: 1.0,
        isExactMatch: true,
      };
    }

    // Try prefix matching for partial cache hits
    const prefixHash = this.hashPrompt(normalized.slice(0, 200));
    const prefixEntry = this.cache.get(prefixHash);

    if (prefixEntry) {
      // Partial match - may be useful for continuation
      return {
        entry: prefixEntry,
        similarity: 0.5,
        isExactMatch: false,
      };
    }

    this.stats.value.totalMisses++;
    this.updateStats();
    return null;
  }

  // Store in cache
  store(prompt: string, response: string, tokenCount: number, metadata: CacheEntry['metadata']): void {
    const normalized = this.normalizePrompt(prompt);
    const hash = this.hashPrompt(normalized);

    // Check cache size limit
    if (this.cache.size >= this.maxCacheSize) {
      this.evictOldest();
    }

    const entry: CacheEntry = {
      hash,
      promptPreview: prompt.slice(0, 100),
      response,
      timestamp: Date.now(),
      hitCount: 0,
      tokenCount,
      metadata,
    };

    this.cache.set(hash, entry);
    this.updateStats();
  }

  // Evict oldest entries (LRU)
  private evictOldest(): void {
    const entries = Array.from(this.cache.entries());

    // Sort by hit count (descending) and timestamp (ascending)
    entries.sort((a, b) => {
      if (a[1].hitCount < this.minHitCountForKeep && b[1].hitCount >= this.minHitCountForKeep) {
        return -1;  // a should be evicted first
      }
      if (b[1].hitCount < this.minHitCountForKeep && a[1].hitCount >= this.minHitCountForKeep) {
        return 1;   // b should be evicted first
      }
      return a[1].timestamp - b[1].timestamp;  // Older entries first
    });

    // Evict 10% of cache
    const evictCount = Math.ceil(this.maxCacheSize * 0.1);
    for (let i = 0; i < evictCount && i < entries.length; i++) {
      this.cache.delete(entries[i][0]);
    }
  }

  // Update stats
  private updateStats(): void {
    const total = this.stats.value.totalHits + this.stats.value.totalMisses;
    this.stats.value.hitRate = total > 0 ? (this.stats.value.totalHits / total) * 100 : 0;
    this.stats.value.cacheSize = this.cache.size;
  }

  // Clear cache
  clear(): void {
    this.cache.clear();
    this.stats.value = {
      hitRate: 0,
      totalHits: 0,
      totalMisses: 0,
      tokensSaved: 0,
      cacheSize: 0,
      maxCacheSize: this.maxCacheSize,
    };
  }

  // Get all entries (for debugging/visualization)
  getEntries(): CacheEntry[] {
    return Array.from(this.cache.values());
  }

  // Set max cache size
  setMaxCacheSize(size: number): void {
    this.maxCacheSize = size;
    this.stats.value.maxCacheSize = size;

    // Evict if over limit
    while (this.cache.size > this.maxCacheSize) {
      this.evictOldest();
    }
  }

  // Export cache for persistence
  export(): string {
    return JSON.stringify(Array.from(this.cache.entries()));
  }

  // Import cache
  import(data: string): void {
    try {
      const entries = JSON.parse(data) as [string, CacheEntry][];
      for (const [hash, entry] of entries) {
        this.cache.set(hash, entry);
      }
      this.updateStats();
    } catch (e) {
      console.error('[KVCache] Import failed:', e);
    }
  }
}

// Singleton instance
let kvCacheInstance: KVCacheManager | null = null;

export function useKVCache(): KVCacheManager {
  if (!kvCacheInstance) {
    kvCacheInstance = new KVCacheManager();
  }
  return kvCacheInstance;
}