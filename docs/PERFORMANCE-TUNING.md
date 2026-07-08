# Performance Tuning Guide

## Overview

This guide covers performance optimization techniques for Hermes Game Operator.

---

## Performance Metrics

### Key Metrics

| Metric | Target | Tool |
|--------|--------|------|
| Response Time | < 2s | Chrome DevTools |
| Bundle Size | < 3MB | Webpack Analyzer |
| Memory Usage | < 500MB | Node.js Inspector |
| CPU Usage | < 80% | System Monitor |
| Database Query | < 100ms | SQLite Explain |

---

## Frontend Optimization

### Code Splitting

```typescript
// Lazy load components
const GameOperatorView = lazy(() => import('./views/GameOperatorView.vue'))
const SettingsView = lazy(() => import('./views/SettingsView.vue'))

// Dynamic imports
async function loadModule() {
  const module = await import('./heavy-module')
  return module.default
}
```

### Tree Shaking

```typescript
// Good: Import specific functions
import { formatDate } from 'date-fns'

// Bad: Import entire library
import * as dateFns from 'date-fns'
```

### Memoization

```typescript
// React
const MemoizedComponent = React.memo(({ data }) => {
  return <div>{data.value}</div>
})

// Vue
const computedValue = computed(() => {
  return expensiveCalculation(data.value)
})
```

### Virtual Scrolling

```typescript
// For large lists
import { FixedSizeList } from 'react-window'

<FixedSizeList
  height={600}
  itemCount={10000}
  itemSize={50}
  width="100%"
>
  {Row}
</FixedSizeList>
```

---

## Backend Optimization

### Database Queries

#### Indexing

```sql
-- Create indexes
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);
CREATE INDEX idx_events_task_id ON events(task_id);

-- Composite index
CREATE INDEX idx_events_type_timestamp ON events(type, timestamp);
```

#### Query Optimization

```sql
-- Bad: SELECT *
SELECT * FROM tasks WHERE status = 'running';

-- Good: Select specific columns
SELECT task_id, status, created_at FROM tasks WHERE status = 'running';

-- Bad: No WHERE clause
SELECT COUNT(*) FROM events;

-- Good: With WHERE clause
SELECT COUNT(*) FROM events WHERE timestamp > datetime('now', '-1 day');
```

#### Connection Pooling

```typescript
// Configure connection pool
const pool = new sqlite3.Database({
  filename: 'data.db',
  pool: {
    min: 2,
    max: 10
  }
})
```

### Caching

#### Application Cache

```typescript
// In-memory cache
const cache = new Map()

function getCached(key: string, fetcher: () => Promise<any>) {
  if (cache.has(key)) {
    return cache.get(key)
  }
  
  const value = await fetcher()
  cache.set(key, value)
  return value
}
```

#### HTTP Cache

```typescript
// Set cache headers
res.setHeader('Cache-Control', 'public, max-age=3600')

// ETag
res.setHeader('ETag', etag)
```

#### Response Cache

```typescript
// Cache API responses
const responseCache = new Map()

async function cachedApiCall(endpoint: string) {
  if (responseCache.has(endpoint)) {
    return responseCache.get(endpoint)
  }
  
  const response = await fetch(endpoint)
  responseCache.set(endpoint, response)
  return response
}
```

### Async Processing

#### Worker Threads

```typescript
// Use worker threads for CPU-intensive tasks
const { Worker } = require('worker_threads')

const worker = new Worker('./worker.js')
worker.postMessage({ data: heavyData })

worker.on('message', (result) => {
  console.log('Result:', result)
})
```

#### Queue Processing

```typescript
// Use queue for background tasks
const queue = new Queue('task-processing')

await queue.add('process-task', {
  taskId: 'task_123'
})

queue.process(async (job) => {
  await processTask(job.data.taskId)
})
```

---

## Network Optimization

### Request Batching

```typescript
// Batch multiple requests
async function batchRequests(requests: Request[]) {
  const responses = await Promise.all(
    requests.map(req => fetch(req.url, req.options))
  )
  return responses
}

// Usage
const results = await batchRequests([
  { url: '/api/tasks', options: { method: 'GET' } },
  { url: '/api/events', options: { method: 'GET' } }
])
```

### Compression

```typescript
// Enable gzip compression
const compression = require('compression')
app.use(compression())
```

### HTTP/2

```typescript
// Use HTTP/2 for multiplexing
const http2 = require('http2')

const server = http2.createServer()
server.on('stream', (stream, headers) => {
  stream.respond({
    'content-type': 'application/json',
    ':status': 200
  })
  stream.end(JSON.stringify({ message: 'Hello' }))
})
```

---

## Memory Optimization

### Garbage Collection

```typescript
// Force garbage collection (Node.js)
if (global.gc) {
  global.gc()
}
```

### Memory Leaks

```typescript
// Avoid memory leaks
// Bad: Event listeners not removed
element.addEventListener('click', handler)

// Good: Remove event listeners
element.removeEventListener('click', handler)

// Bad: Closures holding references
function createHandler() {
  const largeObject = { /* ... */ }
  return () => console.log(largeObject)
}

// Good: Release references
function createHandler() {
  let largeObject = { /* ... */ }
  return () => {
    console.log(largeObject)
    largeObject = null // Release reference
  }
}
```

### Memory Monitoring

```bash
# Monitor memory usage
node --inspect app.js

# Take heap snapshot
# Chrome DevTools → Memory → Take heap snapshot
```

---

## CPU Optimization

### Debouncing

```typescript
// Debounce expensive operations
function debounce(fn: Function, delay: number) {
  let timeoutId: NodeJS.Timeout
  
  return function(...args: any[]) {
    clearTimeout(timeoutId)
    timeoutId = setTimeout(() => fn(...args), delay)
  }
}

// Usage
const debouncedSearch = debounce(search, 300)
input.addEventListener('input', debouncedSearch)
```

### Throttling

```typescript
// Throttle frequent operations
function throttle(fn: Function, limit: number) {
  let inThrottle: boolean
  
  return function(...args: any[]) {
    if (!inThrottle) {
      fn(...args)
      inThrottle = true
      setTimeout(() => inThrottle = false, limit)
    }
  }
}

// Usage
const throttledScroll = throttle(handleScroll, 100)
window.addEventListener('scroll', throttledScroll)
```

### Web Workers

```typescript
// Offload CPU-intensive work
const worker = new Worker('./worker.js')

worker.postMessage({ data: heavyData })

worker.onmessage = (event) => {
  console.log('Result:', event.data)
}
```

---

## Profiling Tools

### Chrome DevTools

```bash
# Open DevTools
# Performance tab → Record → Analyze
```

### Node.js Profiler

```bash
# Profile application
node --prof app.js

# Process profile
node --prof-process isolate-*.log > profile.txt
```

### SQLite Profiler

```sql
-- Explain query plan
EXPLAIN QUERY PLAN SELECT * FROM tasks WHERE status = 'running';

-- Analyze database
ANALYZE;
```

---

## Performance Monitoring

### Application Performance Monitoring (APM)

```typescript
// Sentry
import * as Sentry from '@sentry/node'

Sentry.init({
  dsn: 'YOUR_DSN',
  tracesSampleRate: 1.0
})

// Custom metrics
Sentry.metrics.increment('task.completed')
Sentry.metrics.timing('task.duration', duration)
```

### Custom Metrics

```typescript
// Track performance metrics
class Metrics {
  private metrics = new Map<string, number[]>()
  
  record(name: string, value: number) {
    if (!this.metrics.has(name)) {
      this.metrics.set(name, [])
    }
    this.metrics.get(name)!.push(value)
  }
  
  getAverage(name: string): number {
    const values = this.metrics.get(name) || []
    return values.reduce((a, b) => a + b, 0) / values.length
  }
  
  getPercentile(name: string, percentile: number): number {
    const values = this.metrics.get(name) || []
    const sorted = [...values].sort((a, b) => a - b)
    const index = Math.ceil(percentile / 100 * sorted.length) - 1
    return sorted[index]
  }
}
```

---

## Optimization Checklist

### Frontend
- [ ] Code splitting implemented
- [ ] Tree shaking enabled
- [ ] Lazy loading for images
- [ ] Memoization for expensive computations
- [ ] Virtual scrolling for large lists
- [ ] Compression enabled
- [ ] Caching headers set
- [ ] Bundle size < 3MB

### Backend
- [ ] Database indexes created
- [ ] Query optimization applied
- [ ] Connection pooling configured
- [ ] Caching implemented
- [ ] Async processing for heavy tasks
- [ ] Compression enabled
- [ ] Rate limiting configured

### Infrastructure
- [ ] CDN configured
- [ ] Load balancing set up
- [ ] Auto-scaling enabled
- [ ] Monitoring configured
- [ ] Alerting set up
- [ ] Backup strategy in place

---

## Best Practices

### 1. Measure Before Optimizing

```typescript
// Profile first
const start = performance.now()
await expensiveOperation()
const duration = performance.now() - start

console.log(`Operation took ${duration}ms`)
```

### 2. Optimize Critical Path

```typescript
// Focus on critical path first
// 1. Identify bottlenecks
// 2. Measure impact
// 3. Optimize highest impact
```

### 3. Avoid Premature Optimization

```typescript
// Don't optimize prematurely
// 1. Write clean code first
// 2. Profile to find bottlenecks
// 3. Optimize only what's needed
```

### 4. Test Performance Regularly

```bash
# Run performance tests
npm run test:performance

# Compare with baseline
npm run test:performance:compare
```

---

## Resources

- [Web Performance](https://web.dev/performance/)
- [Node.js Performance](https://nodejs.org/en/docs/guides/simple-profiling/)
- [SQLite Optimization](https://www.sqlite.org/optoverview.html)
- [Chrome DevTools](https://developer.chrome.com/docs/devtools/)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
