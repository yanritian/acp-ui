# Caching Strategy Guide

## Overview

This guide covers caching strategies for Hermes Game Operator to improve performance and reduce load.

---

## Caching Layers

### 1. Browser Cache

**What to Cache**:
- Static assets (JS, CSS, images)
- API responses with long TTL
- User preferences

**Configuration**:
```http
Cache-Control: public, max-age=31536000
ETag: "abc123"
```

**Implementation**:
```typescript
// Set cache headers
res.setHeader('Cache-Control', 'public, max-age=31536000')
res.setHeader('ETag', etag)
```

---

### 2. Application Cache

**What to Cache**:
- Frequently accessed data
- Computed results
- External API responses

**Implementation**:
```typescript
class ApplicationCache {
  private cache = new Map<string, { value: any; expires: number }>()
  
  set(key: string, value: any, ttl: number = 3600) {
    const expires = Date.now() + ttl * 1000
    this.cache.set(key, { value, expires })
  }
  
  get(key: string): any | null {
    const item = this.cache.get(key)
    if (!item) return null
    
    if (Date.now() > item.expires) {
      this.cache.delete(key)
      return null
    }
    
    return item.value
  }
  
  delete(key: string) {
    this.cache.delete(key)
  }
  
  clear() {
    this.cache.clear()
  }
}

// Usage
const cache = new ApplicationCache()
cache.set('task_123', taskData, 3600) // Cache for 1 hour
```

---

### 3. Database Cache

**What to Cache**:
- Query results
- Aggregations
- Metadata

**Implementation**:
```typescript
class DatabaseCache {
  private cache = new Map<string, any>()
  
  async query(sql: string, params: any[]): Promise<any> {
    const cacheKey = `${sql}:${JSON.stringify(params)}`
    
    // Check cache
    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey)
    }
    
    // Execute query
    const result = await db.execute(sql, params)
    
    // Store in cache
    this.cache.set(cacheKey, result)
    
    return result
  }
  
  invalidate(pattern: string) {
    for (const key of this.cache.keys()) {
      if (key.includes(pattern)) {
        this.cache.delete(key)
      }
    }
  }
}
```

---

### 4. CDN Cache

**What to Cache**:
- Static assets
- Public API responses
- Media files

**Configuration**:
```javascript
// Cloudflare example
{
  "page_rules": [
    {
      "targets": [
        {
          "constraint": {
            "operator": "matches",
            "value": "*example.com/static/*"
          }
        }
      ],
      "actions": [
        {
          "id": "cache_level",
          "value": "cache_everything"
        },
        {
          "id": "edge_cache_ttl",
          "value": 31536000
        }
      ]
    }
  ]
}
```

---

## Cache Invalidation Strategies

### 1. Time-Based Invalidation

```typescript
// Cache for fixed duration
cache.set(key, value, 3600) // 1 hour

// Auto-expire after TTL
setTimeout(() => {
  cache.delete(key)
}, 3600 * 1000)
```

### 2. Event-Based Invalidation

```typescript
// Invalidate on data change
function updateTask(taskId: string, data: any) {
  db.updateTask(taskId, data)
  cache.delete(`task_${taskId}`)
  cache.invalidate('tasks')
}
```

### 3. Version-Based Invalidation

```typescript
// Include version in cache key
const cacheKey = `task_${taskId}_v${version}`
cache.set(cacheKey, data)

// Old versions automatically expire
```

### 4. Manual Invalidation

```typescript
// Clear specific cache
cache.delete('task_123')

// Clear all cache
cache.clear()

// Clear by pattern
cache.invalidate('task_*')
```

---

## Cache Warmup

### Preload Critical Data

```typescript
async function warmupCache() {
  // Load frequently accessed tasks
  const tasks = await db.getRecentTasks(100)
  for (const task of tasks) {
    cache.set(`task_${task.task_id}`, task)
  }
  
  // Load metadata
  const metadata = await db.getMetadata()
  cache.set('metadata', metadata, 86400) // 24 hours
}

// Run on startup
warmupCache()
```

---

## Cache Monitoring

### Metrics

```typescript
class CacheMetrics {
  private hits = 0
  private misses = 0
  
  recordHit() {
    this.hits++
  }
  
  recordMiss() {
    this.misses++
  }
  
  getHitRate(): number {
    const total = this.hits + this.misses
    return total > 0 ? this.hits / total : 0
  }
  
  getStats() {
    return {
      hits: this.hits,
      misses: this.misses,
      hitRate: this.getHitRate()
    }
  }
}

// Usage
const metrics = new CacheMetrics()

function getCachedData(key: string) {
  const data = cache.get(key)
  if (data) {
    metrics.recordHit()
    return data
  }
  metrics.recordMiss()
  return null
}
```

### Alerts

```typescript
// Alert on low hit rate
if (metrics.getHitRate() < 0.5) {
  logger.warn('Cache hit rate below 50%')
  // Send alert
}

// Alert on cache size
if (cache.size > 10000) {
  logger.warn('Cache size exceeds 10000 items')
}
```

---

## Best Practices

### 1. Cache Only What's Necessary

```typescript
// Good: Cache expensive computations
const result = expensiveCalculation(data)
cache.set(key, result)

// Bad: Cache everything
cache.set('every_key', 'every_value')
```

### 2. Use Appropriate TTL

```typescript
// Good: Short TTL for volatile data
cache.set('user_session', session, 300) // 5 minutes

// Good: Long TTL for stable data
cache.set('config', config, 86400) // 24 hours
```

### 3. Handle Cache Misses Gracefully

```typescript
// Good: Fallback to database
async function getData(key: string) {
  const cached = cache.get(key)
  if (cached) return cached
  
  const data = await db.query(key)
  cache.set(key, data)
  return data
}

// Bad: Fail on cache miss
function getData(key: string) {
  return cache.get(key) // May return null
}
```

### 4. Monitor Cache Performance

```typescript
// Track hit rate
setInterval(() => {
  const hitRate = metrics.getHitRate()
  logger.info(`Cache hit rate: ${(hitRate * 100).toFixed(2)}%`)
}, 60000) // Every minute
```

---

## Cache Implementation Examples

### Task Cache

```typescript
class TaskCache {
  private cache = new Map<string, OperatorTask>()
  
  async getTask(taskId: string): Promise<OperatorTask | null> {
    // Check cache
    const cached = this.cache.get(taskId)
    if (cached) {
      metrics.recordHit()
      return cached
    }
    
    // Query database
    metrics.recordMiss()
    const task = await db.getTask(taskId)
    
    // Cache result
    if (task) {
      this.cache.set(taskId, task)
    }
    
    return task
  }
  
  invalidateTask(taskId: string) {
    this.cache.delete(taskId)
  }
  
  clear() {
    this.cache.clear()
  }
}
```

### Event Cache

```typescript
class EventCache {
  private cache = new Map<string, OperatorEvent[]>()
  
  async getEvents(taskId: string, limit: number): Promise<OperatorEvent[]> {
    const cacheKey = `${taskId}_${limit}`
    
    // Check cache
    const cached = this.cache.get(cacheKey)
    if (cached) {
      return cached
    }
    
    // Query database
    const events = await db.getEvents(taskId, limit)
    
    // Cache result
    this.cache.set(cacheKey, events)
    
    return events
  }
  
  invalidateEvents(taskId: string) {
    // Remove all cached events for this task
    for (const key of this.cache.keys()) {
      if (key.startsWith(taskId)) {
        this.cache.delete(key)
      }
    }
  }
}
```

---

## Resources

- [Caching Best Practices](https://aws.amazon.com/caching/best-practices/)
- [HTTP Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)
- [Cache Invalidation Strategies](https://martinfowler.com/bliki/CacheInvalidation.html)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
