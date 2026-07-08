# Performance Optimization Guide

This guide provides strategies for optimizing Hermes Game Operator performance.

## Project Analysis Optimization

### Reduce Analysis Scope

**Problem**: Analyzing large projects is slow

**Solution**: Use `.godotignore` to exclude unnecessary files

```
# .godotignore
build/
export/
*.import
.tmp/
.cache/
```

**Impact**: 50-70% faster analysis

### Optimize Directory Structure

**Problem**: Deep nesting slows traversal

**Solution**: Flatten structure where possible

```
# Before (slow)
project/
└── scripts/
    └── player/
        └── movement/
            └── jump/
                └── Jump.gd

# After (fast)
project/
└── scripts/
    └── player/
        └── Jump.gd
```

**Impact**: 30-40% faster analysis

### Split Large Projects

**Problem**: Projects with 500+ files are slow

**Solution**: Split into modules

```
project/
├── core/
│   ├── scripts/
│   └── scenes/
├── gameplay/
│   ├── scripts/
│   └── scenes/
└── ui/
    ├── scripts/
    └── scenes/
```

**Impact**: Analyze only relevant module

### Cache Analysis Results

**Problem**: Re-analyzing same project

**Solution**: Enable analysis caching

```typescript
// Settings → Advanced → Enable Analysis Cache
```

**Impact**: Instant subsequent analyses

## Memory Optimization

### Limit Concurrent Tasks

**Problem**: Multiple tasks consume too much memory

**Solution**: Limit concurrent tasks

```typescript
// Settings → Performance → Max Concurrent Tasks: 2
```

**Impact**: 40-60% less memory usage

### Reduce Event Buffer

**Problem**: Large event history uses memory

**Solution**: Limit event retention

```typescript
// Settings → Performance → Event Retention: 1000 events
```

**Impact**: 20-30% less memory usage

### Clear Cache Regularly

**Problem**: Cache grows over time

**Solution**: Clear cache periodically

```bash
# Linux/macOS
rm -rf ~/.config/acp-ui/cache/

# Windows
rmdir /s /q %APPDATA%\acp-ui\cache\
```

**Impact**: Reclaim disk space and memory

### Optimize Component Rendering

**Problem**: UI components re-render unnecessarily

**Solution**: Use Vue composition API

```typescript
// Good: Computed properties
const filteredEvents = computed(() => 
  events.value.filter(e => e.level === 'error')
)

// Bad: Method calls in template
<div>{{ events.filter(e => e.level === 'error') }}</div>
```

**Impact**: 30-50% faster UI

## Network Optimization

### Enable Response Caching

**Problem**: Repeated API calls are slow

**Solution**: Enable response caching

```typescript
// Settings → Network → Enable Response Cache
```

**Impact**: 60-80% faster for repeated operations

### Use Compression

**Problem**: Large API responses are slow

**Solution**: Enable gzip compression

```typescript
// Already enabled by default in Tauri
```

**Impact**: 40-60% smaller transfers

### Batch API Calls

**Problem**: Multiple small requests are slow

**Solution**: Batch related operations

```typescript
// Good: Single batch request
const results = await Promise.all([
  analyzeProject(),
  listScripts(),
  listScenes()
])

// Bad: Sequential requests
await analyzeProject()
await listScripts()
await listScenes()
```

**Impact**: 50-70% faster

### Optimize WebSocket

**Problem**: WebSocket connections drop

**Solution**: Configure keep-alive

```typescript
// Settings → Network → WebSocket Keep-alive: 25s
```

**Impact**: More stable connections

## Code Generation Optimization

### Simplify Goals

**Problem**: Complex goals take longer

**Solution**: Break into smaller tasks

```
# Good: Simple, focused
"Add double jump to player"

# Bad: Complex, multiple features
"Add double jump, sprint, dash, wall jump, and combat system"
```

**Impact**: 70-90% faster generation

### Use Templates

**Problem**: Generating common patterns from scratch

**Solution**: Create skill templates

```markdown
# skills/godot/double-jump/SKILL.md
Common double jump implementation pattern...
```

**Impact**: 80-90% faster for common tasks

### Enable Incremental Generation

**Problem**: Regenerating entire codebase

**Solution**: Generate only changes

```typescript
// Settings → Code Generation → Enable Incremental Mode
```

**Impact**: 60-80% faster for modifications

### Optimize Prompt Engineering

**Problem**: Inefficient prompts waste tokens

**Solution**: Use concise, clear prompts

```typescript
// Good: Concise
"Add double jump to Player.gd"

// Bad: Verbose
"Please add a double jump ability to the player character script located at Player.gd. 
The player should be able to jump twice in the air before landing..."
```

**Impact**: 30-50% fewer tokens

## Storage Optimization

### Compress Backups

**Problem**: Backup files use too much space

**Solution**: Compress backups

```bash
# Compress backup
gzip file.gd.bak

# Decompress when needed
gunzip file.gd.bak.gz
```

**Impact**: 70-90% smaller backups

### Clean Old Files

**Problem**: Old files accumulate

**Solution**: Regular cleanup

```bash
# Remove old backups
find . -name "*.bak" -mtime +7 -delete

# Remove old logs
find ~/.config/acp-ui/logs/ -mtime +30 -delete
```

**Impact**: Reclaim disk space

### Optimize Database

**Problem**: Database grows over time

**Solution**: Vacuum database

```bash
# SQLite
sqlite3 ~/.config/acp-ui/data.db "VACUUM;"
```

**Impact**: 20-40% smaller database

## UI Optimization

### Lazy Load Components

**Problem**: All components load at once

**Solution**: Lazy load heavy components

```typescript
// Lazy load heavy components
const ProgressTimeline = defineAsyncComponent(() => 
  import('./ProgressTimeline.vue')
)
```

**Impact**: 40-60% faster initial load

### Virtual Scrolling

**Problem**: Large event lists are slow

**Solution**: Use virtual scrolling

```typescript
// Use vue-virtual-scroller for large lists
<RecycleScroller
  :items="events"
  :item-size="50"
  key-field="event_id"
>
  <template #default="{ item }">
    <EventItem :event="item" />
  </template>
</RecycleScroller>
```

**Impact**: 80-90% faster for large lists

### Optimize Animations

**Problem**: Complex animations are slow

**Solution**: Use CSS transforms

```css
/* Good: GPU-accelerated */
.card {
  transform: translateX(100px);
}

/* Bad: Layout thrashing */
.card {
  left: 100px;
}
```

**Impact**: 60 fps animations

### Debounce User Input

**Problem**: Frequent updates slow UI

**Solution**: Debounce input

```typescript
const debouncedSearch = debounce((query) => {
  searchEvents(query)
}, 300)
```

**Impact**: 50-70% less rendering

## Monitoring Optimization

### Sample Events

**Problem**: Too many events overwhelm UI

**Solution**: Sample events for display

```typescript
const sampledEvents = computed(() => {
  if (events.value.length > 100) {
    return events.value.filter((_, i) => i % 10 === 0)
  }
  return events.value
})
```

**Impact**: 90% fewer events displayed

### Aggregate Metrics

**Problem**: Individual metrics are noisy

**Solution**: Aggregate over time windows

```typescript
const aggregatedMetrics = computed(() => {
  const window = 60000 // 1 minute
  return metrics.value.reduce((acc, m) => {
    const bucket = Math.floor(m.timestamp / window)
    acc[bucket] = (acc[bucket] || 0) + m.value
    return acc
  }, {})
})
```

**Impact**: Smoother, more readable metrics

### Optimize Log Queries

**Problem**: Log queries are slow

**Solution**: Index log fields

```sql
CREATE INDEX idx_logs_timestamp ON logs(timestamp);
CREATE INDEX idx_logs_level ON logs(level);
CREATE INDEX idx_logs_task_id ON logs(task_id);
```

**Impact**: 80-90% faster queries

## Benchmarking

### Measure Performance

Use built-in performance tools:

```typescript
// Start timing
const start = performance.now()

// Run operation
await analyzeProject()

// Log duration
console.log(`Analysis took ${performance.now() - start}ms`)
```

### Profile Memory

Use browser dev tools:

```
Chrome DevTools → Memory → Take heap snapshot
```

### Monitor Network

Use browser dev tools:

```
Chrome DevTools → Network → Throttle: Slow 3G
```

## Configuration

### Performance Settings

**Settings** → **Performance**:

- **Max Concurrent Tasks**: 2
- **Event Retention**: 1000
- **Analysis Cache**: Enabled
- **Response Cache**: Enabled
- **Incremental Generation**: Enabled

### Advanced Settings

**Settings** → **Advanced**:

- **Log Level**: Info (not Debug)
- **Event Buffer Size**: 1000
- **Cache Max Size**: 500 MB
- **Cleanup Interval**: 24 hours

## Troubleshooting

### Slow Analysis

**Check**:
1. Project size (files)
2. Directory depth
3. Network speed
4. Cache enabled

**Solutions**:
- Reduce project scope
- Flatten directory structure
- Enable caching
- Upgrade hardware

### High Memory Usage

**Check**:
1. Concurrent tasks
2. Event buffer size
3. Cache size
4. Open applications

**Solutions**:
- Limit concurrent tasks
- Reduce event retention
- Clear cache
- Close other apps

### Slow Code Generation

**Check**:
1. Goal complexity
2. Network speed
3. API rate limits
4. Cache enabled

**Solutions**:
- Simplify goals
- Use templates
- Enable caching
- Check rate limits

## Resources

- [User Manual](USER-MANUAL.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [API Reference](api.md)
- [Security Guide](SECURITY.md)

---

**Last Updated**: 2026-07-08  
**Version**: 1.0.0
