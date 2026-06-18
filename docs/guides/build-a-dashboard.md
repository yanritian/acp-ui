# Building a Custom Dashboard

Use the Event Protocol to build custom real-time dashboards.

## Event Sources

ACP-Swarm pushes events through WebSocket (port 1421) or Tauri events:

```typescript
// Connect to backend WebSocket
const ws = new WebSocket('ws://127.0.0.1:1421')

ws.onmessage = (event) => {
  const data = JSON.parse(event.data)
  handleEvent(data)
}
```

## Event Types

| Event | Description |
|-------|-------------|
| `goal.created` | New goal submitted |
| `goal.active` | Goal started execution |
| `goal.converged` | Goal completed successfully |
| `goal.failed` | Goal failed |
| `worker.registered` | Worker joined swarm |
| `worker.heartbeat` | Worker status update |

## Dashboard Components

### 1. Goal Status Panel

```vue
<template>
  <div class="goal-status">
    <div v-for="goal in goals" :key="goal.id">
      <span :class="statusColor(goal.status)">{{ statusLabel(goal.status) }}</span>
      <span>{{ goal.description }}</span>
    </div>
  </div>
</template>

<script setup>
import { useGoalStore } from '@/stores/goal'
const { goals, statusColor, statusLabel } = useGoalStore()
</script>
```

### 2. Worker Health Panel

```vue
<template>
  <div class="worker-health">
    <div v-for="worker in workers" :key="worker.id">
      <span>{{ worker.id }}</span>
      <span>{{ worker.status }}</span>
      <span>{{ worker.tasksCompleted }}</span>
    </div>
  </div>
</template>

<script setup>
import { useSwarmStore } from '@/stores/swarm'
const { workers } = useSwarmStore()
</script>
```

### 3. Execution Timeline

```vue
<template>
  <div class="timeline">
    <div v-for="event in events" :key="event.id">
      <span>{{ formatTime(event.timestamp) }}</span>
      <span>{{ event.type }}</span>
      <span>{{ event.data }}</span>
    </div>
  </div>
</template>
```

## Real-time Updates

### Option 1: WebSocket (Recommended for Web)

```typescript
import { useACPServer } from '@/lib/acp-protocol/server'

const server = useACPServer()
await server.connectToBackend(1421)

// Events arrive via WebSocket
server.messageHandlers.set('goal.status', async (msg) => {
  updateGoalDisplay(msg.params)
})
```

### Option 2: Tauri Events (Desktop)

```typescript
import { listen } from '@tauri-apps/api/event'

await listen('swarm:goal_update', (event) => {
  updateGoalDisplay(event.payload)
})
```

### Option 3: Polling (Fallback)

```typescript
// Poll every 5 seconds
setInterval(async () => {
  const summary = await goalGetGraphSummary()
  updateSummary(summary)
}, 5000)
```

## Example: Full Dashboard

See `src/views/SwarmDashboard.vue` for a complete implementation:

- Goal submission form
- Real-time status grid
- Worker health cards
- Event timeline
- Performance metrics