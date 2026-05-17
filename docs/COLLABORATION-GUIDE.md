# Agent Collaboration Network Visualization System

## 概述

Agent协作网络可视化系统是一个完整的多Agent协作可视化解决方案，提供：
- 流程图视图（DAG网络）
- 时间线视图
- 看板视图
- Agent能力约定可视化
- 完整的协作细节展示

## 快速开始

### Vue桌面端

#### 1. 访问协作网络
```
路由: /collaboration
功能导航: 🕸️ 协作网络
```

#### 2. 初始化数据
```typescript
import { mockDataGenerator } from '@/lib/collaboration/mock-data-generator'
import { useCollaborationStore } from '@/stores/collaboration'

const store = useCollaborationStore()
const data = mockDataGenerator.generateCompleteDataset()

// Add nodes
data.nodes.forEach(node => store.addNode(node))

// Add edges
data.edges.forEach(edge => store.addEdge(edge))

// Add events
data.events.forEach(event => store.addEvent(event))

// Add protocols
data.protocols.forEach(protocol => store.addProtocol(protocol))
```

#### 3. 视图切换
```typescript
// Network view
store.setViewMode('network')

// Timeline view
store.setViewMode('timeline')

// Kanban view
store.setViewMode('kanban')
```

### Flutter移动端

#### 1. 访问协作网络
```
路由: /collaboration
侧边栏: Collaboration (account_tree图标)
```

#### 2. 使用Riverpod Provider
```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:acp_ui_flutter/data/stores/collaboration_store.dart';

// Watch state
final state = ref.watch(collaborationProvider);

// Watch stats
final stats = ref.watch(collaborationStatsProvider);

// Add node
ref.read(collaborationProvider.notifier).addNode(node);

// Update node
ref.read(collaborationProvider.notifier).updateNode(nodeId, updatedNode);

// Set view mode
ref.read(collaborationProvider.notifier).setViewMode('network');
```

## 功能详解

### 三大视图模式

#### 流程图视图（Network/DAG）

**功能**：
- DAG网络可视化
- Agent节点展示（状态、负载、能力）
- 任务流转动画（流动效果）
- 节点拖拽和缩放
- 点击查看详情
- Background、Controls、MiniMap

**使用**：
```vue
<CollaborationNetworkFlow
  :nodes="store.nodes"
  :edges="store.edges"
  :config="store.config"
  @nodeClick="handleNodeClick"
  @edgeClick="handleEdgeClick"
/>
```

**自定义节点**：
```vue
<AgentNode
  :agentId="node.agentId"
  :agentName="node.agentName"
  :status="node.status"
  :capabilities="node.capabilities"
  :currentLoad="node.currentLoad"
  :maxLoad="node.maxLoad"
/>
```

**自定义边**：
```vue
<TaskEdge
  :taskId="edge.taskId"
  :taskDescription="edge.taskDescription"
  :status="edge.status"
  :animationProgress="edge.animationProgress"
/>
```

#### 时间线视图

**功能**：
- 横向时间轴
- Agent活动条（彩色）
- 里程碑标记（🚀开始、📍检查点、🎉完成、⚠️错误）
- 协作事件节点（12种事件类型）
- 时间范围选择器
- Hover显示详情

**使用**：
```vue
<CollaborationTimeline
  :timelineData="store.timelineData"
  :autoRefresh="true"
  @agentClick="handleAgentClick"
  @eventClick="handleEventClick"
/>
```

#### 看板视图

**功能**：
- 任务状态列（Pending/Running/Completed/Failed）
- Agent任务分配视图（双模式切换）
- 任务卡片详情
- 进度条和优先级标签
- 拖拽任务分配

**使用**：
```vue
<CollaborationKanban
  :kanbanData="store.kanbanData"
  :enableDrag="true"
  @taskDrag="handleTaskDrag"
  @taskClick="handleTaskClick"
/>
```

### Agent能力约定可视化

**功能**：
- 能力卡片展示（能力列表、熟练度、前提条件、输出）
- 协议可视化（输入/输出契约、执行条件、约束规则）
- 参与者列表显示

**使用**：
```vue
<CapabilityProtocolView
  :agentId="agent.agentId"
  :agentName="agent.agentName"
  :capabilities="agent.capabilities"
  :protocols="protocols"
/>
```

### 协作细节展示

**任务流转**：
- 源Agent → 目标Agent
- 状态：pending/flowing/completed/failed
- 时间戳和时长
- 流动动画效果

**消息内容**：
- 消息类型：task_assign/transfer/request/response/notification
- 内容预览和完整展开
- 发送/接收状态

**工具调用**：
- 工具名称、类型、参数
- 执行结果和耗时
- 错误处理

## 状态管理

### Pinia Store（Vue）

**Computed Properties**：
```typescript
// Stats
const stats = store.stats
// {
//   totalAgents: 6,
//   activeAgents: 2,
//   totalTasks: 8,
//   runningTasks: 3,
//   completedTasks: 5,
//   collaborationEfficiency: 87
// }

// Active nodes
const activeNodes = store.activeNodes

// Recent events
const recentEvents = store.recentEvents

// Timeline data
const timelineData = store.timelineData

// Kanban data
const kanbanData = store.kanbanData
```

**Actions**：
```typescript
// Add node
store.addNode(node)

// Update node
store.updateNode(nodeId, { status: 'active' })

// Remove node
store.removeNode(nodeId)

// Add edge
store.addEdge(edge)

// Update edge
store.updateEdge(edgeId, { status: 'flowing' })

// Add event
store.addEvent(event)

// Select node
store.selectNode(nodeId)

// Set view mode
store.setViewMode('network')

// Update config
store.updateConfig({ animationEnabled: false })
```

### Riverpod Store（Flutter）

**Providers**：
```dart
// Main state
final state = ref.watch(collaborationProvider);

// Stats
final stats = ref.watch(collaborationStatsProvider);

// Selected node
final selectedNode = ref.watch(selectedNodeProvider);

// Active nodes
final activeNodes = ref.watch(activeNodesProvider);

// Recent events
final recentEvents = ref.watch(recentEventsProvider);
```

**Notifier Actions**：
```dart
// Add node
ref.read(collaborationProvider.notifier).addNode(node);

// Update node
ref.read(collaborationProvider.notifier).updateNode(nodeId, updatedNode);

// Add event
ref.read(collaborationProvider.notifier).addEvent(event);

// Select node
ref.read(collaborationProvider.notifier).selectNode(nodeId);

// Set view mode
ref.read(collaborationProvider.notifier).setViewMode('timeline');

// Get stats
final stats = ref.read(collaborationProvider.notifier).getStats();
```

## 辅助工具

### Mock数据生成器

**生成完整数据集**：
```typescript
import { mockDataGenerator } from '@/lib/collaboration/mock-data-generator'

const dataset = mockDataGenerator.generateCompleteDataset()
// Returns: { nodes, edges, events, protocols, stats }
```

**单独生成**：
```typescript
// Generate nodes
const nodes = mockDataGenerator.generateNodes(6)

// Generate edges
const edges = mockDataGenerator.generateEdges(nodes, 8)

// Generate events
const events = mockDataGenerator.generateEvents(nodes, 20)

// Generate protocols
const protocols = mockDataGenerator.generateProtocols(nodes, 3)
```

### 配置管理器

**获取配置**：
```typescript
import { collaborationConfigManager } from '@/lib/collaboration/config-manager'

const config = collaborationConfigManager.getConfig()
```

**更新配置**：
```typescript
collaborationConfigManager.updateConfig({
  layoutAlgorithm: 'hierarchical',
  animationSpeed: 1.0,
  showCapabilities: false,
})
```

**布局算法**：
- `force-directed`: Force Directed (nodes push each other away)
- `hierarchical`: Hierarchical (top-down tree layout)
- `circular`: Circular (nodes in a circle)
- `grid`: Grid (nodes in a grid pattern)

### 性能监控器

**获取性能指标**：
```typescript
import { collaborationPerformanceMonitor } from '@/lib/collaboration/performance-monitor'

const metrics = collaborationPerformanceMonitor.getMetrics()
// {
//   renderTime: 12.5,
//   nodeCount: 6,
//   edgeCount: 8,
//   animationFPS: 60,
//   errorRate: 0
// }
```

**性能评分**：
```typescript
const score = collaborationPerformanceMonitor.calculatePerformanceScore()
// 0-100, higher is better

const status = collaborationPerformanceMonitor.getPerformanceStatus()
// 'excellent' | 'good' | 'fair' | 'poor'

const recommendations = collaborationPerformanceMonitor.getRecommendations()
// ['Performance is optimal. No recommendations needed']
```

**监控渲染时间**：
```typescript
const startTime = collaborationPerformanceMonitor.startRenderTimer()
// ... render ...
collaborationPerformanceMonitor.endRenderTimer(startTime)
```

### 键盘快捷键

**默认快捷键**：
- `n`: 切换到流程图视图
- `t`: 切换到时间线视图
- `k`: 切换到看板视图
- `r`: 刷新数据
- `a`: 切换自动刷新
- `c`: 切换能力面板
- `f`: 切换全屏
- `h` 或 `?`: 切换帮助
- `Escape`: 关闭面板
- `Ctrl++`: 放大
- `Ctrl+-`: 缩小
- `Ctrl+0`: 重置缩放
- `Ctrl+s`: 保存配置
- `Ctrl+l`: 加载配置

**使用**：
```vue
<script setup>
import { useCollaborationShortcuts } from '@/lib/collaboration/keyboard-shortcuts'

const shortcuts = useCollaborationShortcuts()

// Register custom handler
shortcuts.registerHandler('custom-action', () => {
  console.log('Custom action executed')
})
</script>
```

## 事件流集成

### RuntimeOutput集成

```typescript
import { agentEventStreamService } from '@/lib/collaboration/agent-event-stream-service'

// Subscribe to runtime output
agentEventStreamService.subscribeToRuntimeOutput(output)

// Subscribe to runtime events
agentEventStreamService.subscribeToRuntimeEvents(event)
```

### 自动转换

RuntimeOutput → CollaborationEvent自动转换：
- `user_message_chunk` → `message_sent`
- `agent_message_chunk` → `message_received`
- `agent_thought_chunk` → `status_change`
- `tool_call` → `tool_call`
- `tool_call_update` → `tool_result`

## 测试

### 单元测试

```bash
npm run test src/stores/__tests__/collaboration.test.ts
```

覆盖内容：
- Node管理测试
- Edge管理测试
- Event管理测试
- Computed属性测试
- View Mode测试
- Selection测试

### E2E测试

```bash
npm run test tests/e2e/collaboration.spec.ts
```

覆盖内容：
- Dashboard加载测试
- 视图切换测试
- 节点交互测试
- 时间线测试
- 看板测试
- 响应式测试

## 性能优化建议

### 大量节点（>50）

建议：
1. 使用Grid布局算法
2. 禁用动画效果
3. 降低刷新频率
4. 使用节点过滤

```typescript
store.updateConfig({
  layoutAlgorithm: 'grid',
  animationEnabled: false,
  refreshInterval: 5000,
})
```

### 大量边（>100）

建议：
1. 使用Orthogonal边样式
2. 简化任务流转
3. 使用时间线视图

```typescript
store.updateConfig({
  edgeStyle: 'orthogonal',
})
```

### 低FPS（<60）

建议：
1. 减少节点数量
2. 禁用动画
3. 使用静态布局

## 最佳实践

### 数据管理

1. **限制事件数量**：最多保留100个事件
2. **批量添加**：批量添加节点和边
3. **及时清理**：定期清理过时数据

```typescript
// Batch add
const nodes = mockDataGenerator.generateNodes(6)
nodes.forEach(n => store.addNode(n))

// Clear old events
store.clearEvents()
```

### UI交互

1. **合理使用视图**：根据场景选择合适视图
   - 流程图：整体协作关系
   - 时间线：Agent活动追踪
   - 看板：任务状态管理

2. **配置优化**：根据设备性能调整配置
   - 高性能设备：启用动画、快速刷新
   - 低性能设备：禁用动画、降低刷新频率

### 性能监控

1. **定期检查**：定期查看性能指标
2. **及时优化**：根据建议及时优化
3. **导出报告**：导出性能报告进行分析

```typescript
// Export metrics
const metricsJson = collaborationPerformanceMonitor.exportMetrics()

// Get summary
const summary = collaborationPerformanceMonitor.getSummary()
```

## 故障排除

### 问题1：节点不显示

检查：
1. 节点位置是否合理（在可视范围内）
2. 节点状态是否正确
3. Vue Flow依赖是否安装

解决：
```typescript
// Check node position
console.log(node.position) // Should be within viewport

// Check dependencies
npm list @vue-flow/core
```

### 问题2：动画卡顿

检查：
1. FPS是否低于60
2. 节点数量是否过大
3. 是否启用了所有动画

解决：
```typescript
// Disable animations
store.updateConfig({ animationEnabled: false })

// Reduce node count
store.removeNode(nodeId)
```

### 问题3：数据不更新

检查：
1. 自动刷新是否启用
2. 刷新频率是否合理
3. 事件流是否连接

解决：
```typescript
// Enable auto refresh
store.updateConfig({
  autoRefresh: true,
  refreshInterval: 2000
})

// Manual refresh
store.setLoading(true)
// ... fetch data ...
store.setLoading(false)
```

## 扩展开发

### 添加新视图模式

1. 创建新组件
2. 添加到EnhancedHermesDashboard
3. 更新类型定义
4. 添加快捷键

### 添加新事件类型

1. 更新types.ts
2. 添加事件处理逻辑
3. 更新mock数据生成器
4. 添加UI展示

### 自定义节点样式

1. 创建新的AgentNode组件
2. 注册到Vue Flow
3. 更新配置管理器

## 资源

### 文档
- 规划文档：`.claude/plans/agent-collaboration-network-plan.md`
- 实施总结：`.claude/plans/agent-collaboration-implementation-summary.md`
- 最终报告：`.claude/plans/agent-collaboration-final-report.md`

### 示例
- Mock数据：`src/lib/collaboration/mock-data-generator.ts`
- 配置管理：`src/lib/collaboration/config-manager.ts`
- 性能监控：`src/lib/collaboration/performance-monitor.ts`

### 测试
- 单元测试：`src/stores/__tests__/collaboration.test.ts`
- E2E测试：`tests/e2e/collaboration.spec.ts`

## 更新日志

### v1.0.0 (2026-05-15)
- ✅ 完成所有核心组件
- ✅ Vue桌面端实现
- ✅ Flutter移动端实现
- ✅ 数据层和状态管理
- ✅ 事件流服务
- ✅ Mock数据生成器
- ✅ 配置管理器
- ✅ 性能监控器
- ✅ 键盘快捷键
- ✅ 单元测试和E2E测试
- ✅ 路由集成

---

**状态**: ✅ 完成
**可用性**: ✅ 即刻可用
**支持**: Vue桌面端 + Flutter移动端