# Web Platform Architecture Design

## Overview

Web platform provides browser-based access to Agent Platform, using Vue 3 + TypeScript frontend communicating with Tauri backend via HTTP API or WebSocket.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Web Browser                               │
├─────────────────────────────────────────────────────────────────┤
│  Vue 3 App                                                       │
│  ├── Components                                                  │
│  │   ├── AgentPanel.vue          (Agent selection & status)     │
│  │   ├── OneShotInput.vue        (Single input interface)       │
│  │   ├── CostDashboard.vue       (Cost tracking & budget)       │
│  │   ├── GameAssetManager.vue    (Game asset detection)         │
│  │   ├── PrivacySettings.vue     (Privacy zone configuration)   │
│  │   └── MarketingTools.vue      (Jimeng/Kling/WPS integration) │
│  ├── Stores (Pinia)                                              │
│  │   ├── agentStore.ts           (Agent state management)       │
│  │   ├── costStore.ts            (Cost tracking state)          │
│  │   ├── privacyStore.ts         (Privacy settings)             │
│  │   └── userStore.ts            (User role detection)          │
│  ├── Services                                                    │
│  │   ├── agentService.ts         (Agent API calls)              │
│  │   ├── costService.ts          (Cost API calls)               │
│  │   └── websocketService.ts     (Real-time updates)            │
│  └── Router                                                      │
│      ├── /                        (One-Shot Interface)           │
│      ├── /agents                  (Agent Management)             │
│      ├── /costs                   (Cost Dashboard)               │
│      ├── /game                    (Game Asset Tools)             │
│      └── /marketing               (Marketing Tools)              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP REST API / WebSocket
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Tauri Backend (Rust)                          │
│  ├── HTTP Server (axum or actix-web)                            │
│  │   ├── /api/agents             (Agent CRUD)                   │
│  │   ├── /api/costs              (Cost tracking)                │
│  │   ├── /api/execute            (Execute task)                 │
│  │   ├── /api/oneshot            (One-Shot Interface)           │
│  │   └── /ws/events              (WebSocket events)             │
│  ├── Agent Adapter System                                       │
│  └── Cost Tracker + Privacy Orchestrator                        │
└─────────────────────────────────────────────────────────────────┘
```

## API Endpoints

### Agent Management
```
GET    /api/agents              - List all available agents
GET    /api/agents/:id          - Get agent details
GET    /api/agents/:id/health   - Get agent health metrics
POST   /api/agents/:id/config   - Configure agent
```

### One-Shot Interface
```
POST   /api/oneshot             - Execute one-shot request
  Body: { input, context_hint, preferred_agent, max_cost, timeout_ms }
  Response: { detected_role, detected_scene, selected_agent, result, cost }
```

### Cost Tracking
```
GET    /api/costs/summary       - Get cost summary
GET    /api/costs/history       - Get cost history
POST   /api/costs/budget        - Set budget limits
GET    /api/costs/forecast      - Get cost forecast
```

### Execution
```
POST   /api/execute             - Execute task with specific agent
  Body: { agent_id, task }
  Response: { result, cost, duration_ms }
```

### WebSocket Events
```
/ws/events                      - Real-time event stream
  Events: cost_update, agent_status, task_progress, budget_alert
```

## Frontend Components

### 1. OneShotInput.vue
```vue
<template>
  <div class="one-shot-container">
    <textarea 
      v-model="input" 
      placeholder="输入你的需求..."
      @keyup.ctrl.enter="execute"
    />
    <div class="options">
      <select v-model="preferredAgent">
        <option value="">自动选择</option>
        <option value="claude-code">Claude Code</option>
        <option value="kimi">Kimi</option>
        <option value="jimeng">即梦</option>
        <option value="kling">可灵</option>
      </select>
      <input type="number" v-model="maxCost" placeholder="最大成本 (¥)" />
    </div>
    <button @click="execute">执行</button>
    
    <div v-if="result" class="result">
      <div class="meta">
        检测角色: {{ result.detected_role }}
        选择 Agent: {{ result.selected_agent }}
        原因: {{ result.selection_reason }}
      </div>
      <div class="output">{{ result.result?.content }}</div>
      <div class="cost">成本: ¥{{ result.cost.toFixed(4) }}</div>
    </div>
  </div>
</template>
```

### 2. CostDashboard.vue
```vue
<template>
  <div class="cost-dashboard">
    <div class="summary">
      <div class="card">
        <h3>本月总成本</h3>
        <span class="amount">¥{{ summary.monthly_total.toFixed(2) }}</span>
        <span class="budget">预算 ¥{{ budget.monthly_limit }}</span>
        <progress :value="summary.monthly_total" :max="budget.monthly_limit" />
      </div>
      <div class="card">
        <h3>今日成本</h3>
        <span class="amount">¥{{ summary.daily_total.toFixed(4) }}</span>
      </div>
    </div>
    
    <div class="breakdown">
      <h3>Agent 成本分布</h3>
      <Chart :data="breakdownData" type="pie" />
    </div>
    
    <div class="alerts" v-if="alerts.length">
      <Alert v-for="alert in alerts" :level="alert.level" :message="alert.message" />
    </div>
  </div>
</template>
```

### 3. AgentPanel.vue
```vue
<template>
  <div class="agent-panel">
    <div v-for="agent in agents" :key="agent.id" class="agent-card">
      <div class="header">
        <span class="name">{{ agent.name }}</span>
        <span :class="['status', agent.status]">{{ agent.status }}</span>
      </div>
      <div class="health">
        健康度: {{ agent.health_score.toFixed(0) }}%
        成功率: {{ agent.success_rate.toFixed(2) }}
      </div>
      <div class="capabilities">
        <span v-for="cap in agent.capabilities" class="cap">{{ cap.name }}</span>
      </div>
    </div>
  </div>
</template>
```

## Pinia Stores

### agentStore.ts
```typescript
import { defineStore } from 'pinia';

export const useAgentStore = defineStore('agents', {
  state: () => ({
    agents: [],
    selectedAgent: null,
    healthMetrics: new Map(),
  }),
  
  actions: {
    async fetchAgents() {
      const response = await fetch('/api/agents');
      this.agents = await response.json();
    },
    
    async executeTask(agentId: string, task: AgentTask) {
      const response = await fetch('/api/execute', {
        method: 'POST',
        body: JSON.stringify({ agent_id: agentId, task }),
      });
      return await response.json();
    },
  },
});
```

### costStore.ts
```typescript
export const useCostStore = defineStore('costs', {
  state: () => ({
    summary: { daily_total: 0, monthly_total: 0 },
    budget: { monthly_limit: 100, daily_limit: 10 },
    history: [],
    alerts: [],
  }),
  
  actions: {
    async fetchSummary() {
      const response = await fetch('/api/costs/summary');
      this.summary = await response.json();
    },
    
    async setBudget(budget: CostBudget) {
      await fetch('/api/costs/budget', {
        method: 'POST',
        body: JSON.stringify(budget),
      });
      this.budget = budget;
    },
  },
});
```

## WebSocket Integration

### websocketService.ts
```typescript
class WebSocketService {
  private ws: WebSocket;
  
  connect() {
    this.ws = new WebSocket('ws://localhost:3000/ws/events');
    
    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      this.handleEvent(data);
    };
  }
  
  handleEvent(event: AgentEvent) {
    switch (event.type) {
      case 'cost_update':
        costStore.updateCost(event.payload);
        break;
      case 'agent_status':
        agentStore.updateStatus(event.payload);
        break;
      case 'budget_alert':
        costStore.addAlert(event.payload);
        break;
    }
  }
}
```

## Build & Deployment

### Development
```bash
# Start Tauri backend with HTTP server
cd src-tauri && cargo run --features http-server

# Start Vue frontend
cd src && npm run dev
```

### Production
```bash
# Build Tauri backend
cargo build --release --features http-server

# Build Vue frontend
npm run build

# Deploy: Use nginx to serve Vue static files + reverse proxy to Tauri HTTP server
```

## Security

- CORS configuration for cross-origin requests
- JWT authentication for API access
- Rate limiting to prevent abuse
- Input validation on all endpoints

## Features Mapping

| Desktop (Tauri) | Web Equivalent |
|-----------------|----------------|
| tauri.invoke() | fetch('/api/...') |
| tauri.listen() | WebSocket events |
| Local file access | Server-side file handling |
| System notifications | Browser notifications |
| Native menus | Web navigation |

---

**Status: Design Complete**
**Next: Implement HTTP server in Tauri backend + Vue frontend components**