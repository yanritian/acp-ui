# API Versioning Guide

## Overview

This guide covers API versioning strategy for Hermes Game Operator.

---

## Versioning Strategy

### Semantic Versioning

We follow [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### API Version Format

```
/v1/endpoint
/v2/endpoint
```

**Example**:
```
GET /v1/tasks
GET /v2/tasks
```

---

## Version Lifecycle

### Stage 1: Development

- **Status**: Beta
- **Support**: No guarantees
- **Changes**: Frequent breaking changes
- **Example**: `/v0/tasks`

### Stage 2: Stable

- **Status**: GA (General Availability)
- **Support**: Full support
- **Changes**: Backward compatible only
- **Example**: `/v1/tasks`

### Stage 3: Deprecated

- **Status**: Deprecated
- **Support**: Security fixes only
- **Changes**: No new features
- **Example**: `/v1/tasks` (deprecated)

### Stage 4: Retired

- **Status**: Retired
- **Support**: None
- **Changes**: Endpoint removed
- **Example**: `/v0/tasks` (removed)

---

## Deprecation Policy

### Timeline

| Stage | Duration | Notice |
|-------|----------|--------|
| Deprecated | 6 months | Email + Documentation |
| Retired | Immediate | Email + Migration Guide |

### Deprecation Process

1. **Announce** (Month 1)
   - Email notification
   - Documentation update
   - Deprecation header

2. **Migrate** (Month 2-5)
   - Migration guide
   - Code samples
   - Support

3. **Retire** (Month 6)
   - Final reminder
   - Remove endpoint
   - Archive documentation

---

## Version Headers

### Request Header

```http
Accept: application/vnd.hermes.v1+json
```

### Response Header

```http
X-API-Version: v1
X-API-Deprecated: false
X-API-Sunset: 2027-01-01
```

---

## Migration Guide

### v1 to v2 Migration

#### Changes

1. **Endpoint Path**
   ```
   Old: /api/tasks
   New: /v2/tasks
   ```

2. **Request Format**
   ```json
   // Old (v1)
   {
     "goal": "Add feature"
   }
   
   // New (v2)
   {
     "domain": "game.godot",
     "project_path": "/path/to/project",
     "goal": "Add feature",
     "mode": "propose_then_apply"
   }
   ```

3. **Response Format**
   ```json
   // Old (v1)
   {
     "id": "task_123",
     "status": "running"
   }
   
   // New (v2)
   {
     "task_id": "task_123",
     "status": "running",
     "created_at": "2026-07-08T10:00:00Z",
     "updated_at": "2026-07-08T10:05:00Z"
   }
   ```

#### Migration Steps

1. **Update Base URL**
   ```typescript
   // Old
   const baseURL = 'http://localhost:1420/api'
   
   // New
   const baseURL = 'http://localhost:1420/v2'
   ```

2. **Update Request Format**
   ```typescript
   // Old
   const response = await fetch('/api/tasks', {
     method: 'POST',
     body: JSON.stringify({ goal: 'Add feature' })
   })
   
   // New
   const response = await fetch('/v2/tasks', {
     method: 'POST',
     body: JSON.stringify({
       domain: 'game.godot',
       project_path: '/path/to/project',
       goal: 'Add feature',
       mode: 'propose_then_apply'
     })
   })
   ```

3. **Update Response Handling**
   ```typescript
   // Old
   const task = await response.json()
   console.log(task.id)
   
   // New
   const task = await response.json()
   console.log(task.task_id)
   console.log(task.created_at)
   ```

---

## Breaking Changes Policy

### What Constitutes a Breaking Change

✅ **Breaking**:
- Removing an endpoint
- Changing request/response format
- Changing endpoint path
- Removing a field
- Changing field type

❌ **Not Breaking**:
- Adding a new endpoint
- Adding an optional field
- Adding a new header
- Deprecating (not removing) a field

---

## Version Support Matrix

| Version | Status | Support End | Notes |
|---------|--------|-------------|-------|
| v0 | Retired | 2026-01-01 | Beta version |
| v1 | Stable | 2027-07-08 | Current stable |
| v2 | Development | N/A | Next version |

---

## Best Practices

### 1. Always Specify Version

```typescript
// Good
const response = await fetch('/v1/tasks')

// Bad
const response = await fetch('/tasks') // May change
```

### 2. Check Deprecation Headers

```typescript
const response = await fetch('/v1/tasks')

if (response.headers.get('X-API-Deprecated') === 'true') {
  const sunset = response.headers.get('X-API-Sunset')
  console.warn(`API deprecated, sunset: ${sunset}`)
}
```

### 3. Handle Version Errors

```typescript
try {
  const response = await fetch('/v1/tasks')
  
  if (response.status === 410) {
    // Version retired
    console.error('API version retired')
    // Redirect to migration guide
  }
} catch (error) {
  console.error('API error:', error)
}
```

### 4. Test Against Multiple Versions

```typescript
// Test against v1
await testAgainstVersion('v1')

// Test against v2
await testAgainstVersion('v2')
```

---

## Monitoring

### Version Usage Metrics

```sql
-- Track API version usage
SELECT 
  version,
  COUNT(*) as request_count,
  AVG(response_time) as avg_response_time
FROM api_logs
WHERE timestamp > NOW() - INTERVAL 7 DAY
GROUP BY version
ORDER BY request_count DESC;
```

### Deprecation Warnings

```typescript
// Log deprecation warnings
if (isDeprecated(version)) {
  logger.warn({
    version,
    endpoint,
    sunset: getSunsetDate(version)
  }, 'Deprecated API version used')
}
```

---

## Tools

### Version Detector

```typescript
function detectVersion(request: Request): string {
  // Check URL path
  const match = request.url.match(/\/v(\d+)\//)
  if (match) {
    return `v${match[1]}`
  }
  
  // Check Accept header
  const accept = request.headers.get('Accept')
  if (accept?.includes('vnd.hermes.v')) {
    const versionMatch = accept.match(/vnd\.hermes\.v(\d+)/)
    if (versionMatch) {
      return `v${versionMatch[1]}`
    }
  }
  
  // Default to latest
  return 'v2'
}
```

### Version Router

```typescript
function routeRequest(version: string, endpoint: string): string {
  const routes = {
    'v1': {
      '/tasks': '/v1/tasks',
      '/events': '/v1/events'
    },
    'v2': {
      '/tasks': '/v2/tasks',
      '/events': '/v2/events'
    }
  }
  
  return routes[version]?.[endpoint] || '/v2' + endpoint
}
```

---

## Documentation

### Version-Specific Docs

```markdown
# API v1 Documentation

**Status**: Stable  
**Support End**: 2027-07-08

## Endpoints

### GET /v1/tasks

Returns list of tasks.

**Response**:
```json
{
  "id": "task_123",
  "status": "running"
}
```
```

---

## Resources

- [Semantic Versioning](https://semver.org/)
- [API Versioning Best Practices](https://restfulapi.net/versioning/)
- [Migration Guide](MIGRATION-GUIDE.md)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
