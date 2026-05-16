<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import type { TaskDAG } from '@/lib/task-parser';

interface GraphNode {
  id: string;
  name: string;
  x: number;
  y: number;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'blocked';
  dependencies: string[];
  agent?: string;
}

interface GraphEdge {
  from: string;
  to: string;
}

// Props
const props = defineProps<{
  dag?: TaskDAG | null;
  showAgents?: boolean;
  orientation?: 'horizontal' | 'vertical';
}>();

// State
const nodes = ref<GraphNode[]>([]);
const edges = ref<GraphEdge[]>([]);
const selectedNode = ref<string | null>(null);
const hoveredNode = ref<string | null>(null);
const canvasWidth = ref(800);
const canvasHeight = ref(600);

// Node dimensions - fixed constants
const nodeWidth = 160;
const nodeHeight = 50;
const nodeSpacingX = 200;
const nodeSpacingY = 100;
const paddingX = 80;
const paddingY = 60;

// Status colors
const statusColors: Record<string, { bg: string; border: string; text: string }> = {
  pending: { bg: '#F3F4F6', border: '#9CA3AF', text: '#6B7280' },
  running: { bg: '#EFF6FF', border: '#3B82F6', text: '#1D4ED8' },
  completed: { bg: '#ECFDF5', border: '#10B981', text: '#059669' },
  failed: { bg: '#FEF2F2', border: '#EF4444', text: '#DC2626' },
  blocked: { bg: '#FFF7ED', border: '#F97316', text: '#EA580C' },
};

const getStatusStyle = (status: string) => {
  return statusColors[status] || statusColors.pending;
};

const viewBox = computed(() => {
  return `0 0 ${canvasWidth.value} ${canvasHeight.value}`;
});

// Methods
const calculateLayout = () => {
  if (!props.dag || props.dag.nodes.size === 0) {
    nodes.value = [];
    edges.value = [];
    canvasWidth.value = 800;
    canvasHeight.value = 400;
    return;
  }

  const dagNodes = Array.from(props.dag.nodes.values());

  // Group nodes by depth using BFS
  const levels: string[][] = [];
  const visited = new Set<string>();
  const rootIds = props.dag.rootNodes || [];

  // Start with root nodes at level 0
  if (rootIds.length > 0) {
    levels.push(rootIds);
    rootIds.forEach(id => visited.add(id));
  }

  // BFS traversal
  let levelIndex = 0;
  while (visited.size < dagNodes.length && levelIndex < levels.length) {
    const nextLevel: string[] = [];
    const currentLevelIds = levels[levelIndex] || [];

    for (const nodeId of currentLevelIds) {
      // Find nodes that depend on this node (edges from this node)
      const dependents = props.dag.edges.get(nodeId) || [];
      for (const depId of dependents) {
        if (!visited.has(depId)) {
          const depNode = props.dag.nodes.get(depId);
          if (depNode) {
            nextLevel.push(depId);
            visited.add(depId);
          }
        }
      }
    }

    if (nextLevel.length > 0) {
      levels.push(nextLevel);
    }
    levelIndex++;
  }

  // Add any unvisited nodes to last level
  for (const node of dagNodes) {
    if (!visited.has(node.id)) {
      if (levels.length === 0) {
        levels.push([node.id]);
      } else {
        levels[levels.length - 1].push(node.id);
      }
      visited.add(node.id);
    }
  }

  // Calculate positions based on orientation
  const layoutNodes: GraphNode[] = [];
  const totalLevels = levels.length;
  const maxNodesPerLevel = Math.max(...levels.map(l => l.length));

  // Set canvas size
  if (props.orientation === 'vertical') {
    canvasHeight.value = Math.max(400, totalLevels * nodeSpacingY + paddingY * 2);
    canvasWidth.value = Math.max(600, maxNodesPerLevel * nodeSpacingX + paddingX);
  } else {
    canvasWidth.value = Math.max(600, totalLevels * nodeSpacingX + paddingX * 2);
    canvasHeight.value = Math.max(400, maxNodesPerLevel * nodeSpacingY + paddingY);
  }

  for (let levelIdx = 0; levelIdx < levels.length; levelIdx++) {
    const nodeIds = levels[levelIdx];
    const nodesInLevel = nodeIds.length;

    for (let nodeIdx = 0; nodeIdx < nodesInLevel; nodeIdx++) {
      const nodeId = nodeIds[nodeIdx];
      const node = props.dag.nodes.get(nodeId);
      if (!node) continue;

      let x: number, y: number;

      if (props.orientation === 'vertical') {
        // Vertical: levels go down, nodes spread horizontally
        y = paddingY + levelIdx * nodeSpacingY;
        // Center nodes within level
        const levelWidth = nodesInLevel * nodeSpacingX;
        const startX = (canvasWidth.value - levelWidth) / 2 + nodeSpacingX / 2;
        x = startX + nodeIdx * nodeSpacingX;
      } else {
        // Horizontal: levels go right, nodes spread vertically
        x = paddingX + levelIdx * nodeSpacingX;
        const levelHeight = nodesInLevel * nodeSpacingY;
        const startY = (canvasHeight.value - levelHeight) / 2 + nodeSpacingY / 2;
        y = startY + nodeIdx * nodeSpacingY;
      }

      layoutNodes.push({
        id: nodeId,
        name: node.step.name,
        x,
        y,
        status: node.status,
        dependencies: node.step.dependencies || [],
        agent: node.assignedAgent,
      });
    }
  }

  nodes.value = layoutNodes;

  // Build edges from dependencies
  const layoutEdges: GraphEdge[] = [];
  for (const node of layoutNodes) {
    for (const depId of node.dependencies) {
      // Find the source node with matching dependency
      const sourceId = `${props.dag?.id}-${depId}`;
      const sourceNode = layoutNodes.find(n => n.id === sourceId);
      if (sourceNode) {
        layoutEdges.push({
          from: sourceId,
          to: node.id,
        });
      }
    }
  }
  edges.value = layoutEdges;
};

const getEdgePath = (edge: GraphEdge): string => {
  const fromNode = nodes.value.find(n => n.id === edge.from);
  const toNode = nodes.value.find(n => n.id === edge.to);

  if (!fromNode || !toNode) return '';

  // Calculate edge positions based on node centers
  const fromCenterX = fromNode.x;
  const fromCenterY = fromNode.y;
  const toCenterX = toNode.x;
  const toCenterY = toNode.y;

  if (props.orientation === 'vertical') {
    // Vertical: connect bottom of source to top of target
    const startY = fromCenterY + nodeHeight / 2;
    const endY = toCenterY - nodeHeight / 2;
    const midY = (startY + endY) / 2;

    return `M ${fromCenterX} ${startY}
            C ${fromCenterX} ${midY},
              ${toCenterX} ${midY},
              ${toCenterX} ${endY}`;
  } else {
    // Horizontal: connect right of source to left of target
    const startX = fromCenterX + nodeWidth / 2;
    const endX = toCenterX - nodeWidth / 2;
    const midX = (startX + endX) / 2;

    return `M ${startX} ${fromCenterY}
            C ${midX} ${fromCenterY},
              ${midX} ${toCenterY},
              ${endX} ${toCenterY}`;
  }
};

const selectNodeHandler = (nodeId: string) => {
  selectedNode.value = nodeId;
};

const hoveredNodeHandler = (nodeId: string) => {
  hoveredNode.value = nodeId;
};

const unhoveredNodeHandler = () => {
  hoveredNode.value = null;
};

const isEdgeHighlighted = (edge: GraphEdge): boolean => {
  return hoveredNode.value === edge.from || hoveredNode.value === edge.to ||
         selectedNode.value === edge.from || selectedNode.value === edge.to;
};

// Watch for DAG changes
watch(() => props.dag, calculateLayout, { immediate: true, deep: true });

onMounted(() => {
  calculateLayout();
});
</script>

<template>
  <div class="task-graph-container">
    <!-- Header with legend -->
    <div class="graph-header">
      <h3 class="graph-title">Task Graph</h3>
      <div class="legend">
        <span class="legend-item">
          <span class="legend-dot pending"></span>
          <span>Pending</span>
        </span>
        <span class="legend-item">
          <span class="legend-dot running"></span>
          <span>Running</span>
        </span>
        <span class="legend-item">
          <span class="legend-dot completed"></span>
          <span>Completed</span>
        </span>
        <span class="legend-item">
          <span class="legend-dot failed"></span>
          <span>Failed</span>
        </span>
        <span class="legend-item">
          <span class="legend-dot blocked"></span>
          <span>Blocked</span>
        </span>
      </div>
    </div>

    <!-- Graph SVG -->
    <div class="graph-content">
      <div v-if="!dag || nodes.length === 0" class="empty-state">
        <div class="empty-icon">📊</div>
        <div class="empty-text">No task graph to display</div>
        <div class="empty-hint">Start a task to see the DAG visualization</div>
      </div>

      <svg
        v-else
        :viewBox="viewBox"
        class="task-graph-svg"
      >
        <!-- Arrow marker definition -->
        <defs>
          <marker
            id="arrowhead"
            markerWidth="8"
            markerHeight="6"
            refX="7"
            refY="3"
            orient="auto"
          >
            <path d="M0,0 L8,3 L0,6 L2,3 Z" fill="#94A3B8" />
          </marker>
          <marker
            id="arrowhead-highlight"
            markerWidth="8"
            markerHeight="6"
            refX="7"
            refY="3"
            orient="auto"
          >
            <path d="M0,0 L8,3 L0,6 L2,3 Z" fill="#3B82F6" />
          </marker>
        </defs>

        <!-- Edges layer -->
        <g class="edges-layer">
          <path
            v-for="edge in edges"
            :key="`${edge.from}-${edge.to}`"
            :d="getEdgePath(edge)"
            fill="none"
            :stroke="isEdgeHighlighted(edge) ? '#3B82F6' : '#CBD5E1'"
            :stroke-width="isEdgeHighlighted(edge) ? 2.5 : 2"
            :marker-end="isEdgeHighlighted(edge) ? 'url(#arrowhead-highlight)' : 'url(#arrowhead)'"
            class="task-edge"
          />
        </g>

        <!-- Nodes layer -->
        <g class="nodes-layer">
          <g
            v-for="node in nodes"
            :key="node.id"
            :transform="`translate(${node.x - nodeWidth / 2}, ${node.y - nodeHeight / 2})`"
            @click="selectNodeHandler(node.id)"
            @mouseenter="hoveredNodeHandler(node.id)"
            @mouseleave="unhoveredNodeHandler"
            class="task-node-group"
          >
            <!-- Node card background -->
            <rect
              :width="nodeWidth"
              :height="nodeHeight"
              rx="6"
              ry="6"
              :fill="getStatusStyle(node.status).bg"
              :stroke="getStatusStyle(node.status).border"
              :stroke-width="selectedNode === node.id ? 2.5 : hoveredNode === node.id ? 2 : 1.5"
              class="node-card"
            />

            <!-- Status icon -->
            <text
              x="12"
              :y="nodeHeight / 2"
              dominant-baseline="middle"
              class="status-icon"
            >
              {{ node.status === 'completed' ? '✓' : node.status === 'running' ? '⏳' : node.status === 'failed' ? '✗' : node.status === 'blocked' ? '⚠' : '○' }}
            </text>

            <!-- Node name -->
            <text
              x="28"
              :y="nodeHeight / 2 - 8"
              dominant-baseline="middle"
              class="node-name"
              :fill="getStatusStyle(node.status).text"
            >
              {{ node.name }}
            </text>

            <!-- Agent badge -->
            <text
              v-if="showAgents && node.agent"
              :x="nodeWidth - 8"
              :y="nodeHeight / 2 + 8"
              text-anchor="end"
              dominant-baseline="middle"
              class="agent-badge"
            >
              {{ node.agent.split('-')[0] }}
            </text>

            <!-- Progress bar for running tasks -->
            <g v-if="node.status === 'running'">
              <rect
                x="4"
                :y="nodeHeight - 8"
                :width="nodeWidth - 8"
                height="4"
                rx="2"
                fill="#DBEAFE"
              />
              <rect
                x="4"
                :y="nodeHeight - 8"
                :width="(nodeWidth - 8) * 0.6"
                height="4"
                rx="2"
                fill="#3B82F6"
              >
                <animate
                  attributeName="width"
                  values="8;(nodeWidth - 8);(nodeWidth - 8) * 0.6;8"
                  dur="3s"
                  repeatCount="indefinite"
                />
              </rect>
            </g>
          </g>
        </g>
      </svg>
    </div>

    <!-- Node detail panel -->
    <div
      v-if="selectedNode"
      class="detail-panel-overlay"
      @click.self="selectedNode = null"
    >
      <div class="detail-panel">
        <div class="detail-header">
          <h4>Task Details</h4>
          <button @click="selectedNode = null" class="close-btn">×</button>
        </div>
        <div class="detail-body">
          <template v-for="node in nodes.filter(n => n.id === selectedNode)" :key="node.id">
            <div class="detail-row">
              <label>Name</label>
              <span class="detail-value">{{ node.name }}</span>
            </div>
            <div class="detail-row">
              <label>Status</label>
              <span :class="['status-tag', node.status]">{{ node.status }}</span>
            </div>
            <div class="detail-row">
              <label>Dependencies</label>
              <span class="detail-value">{{ node.dependencies.length > 0 ? node.dependencies.join(', ') : 'None' }}</span>
            </div>
            <div v-if="node.agent" class="detail-row">
              <label>Agent</label>
              <span class="detail-value">{{ node.agent }}</span>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-graph-container {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #F8FAFC 0%, #F1F5F9 100%);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  font-family: system-ui, -apple-system, sans-serif;
}

.graph-header {
  padding: 16px 24px;
  background: white;
  border-bottom: 1px solid #E2E8F0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.graph-title {
  font-size: 18px;
  font-weight: 600;
  color: #1E293B;
}

.legend {
  display: flex;
  gap: 12px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #64748B;
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.legend-dot.pending { background: #94A3B8; }
.legend-dot.running { background: #3B82F6; }
.legend-dot.completed { background: #22C55E; }
.legend-dot.failed { background: #EF4444; }
.legend-dot.blocked { background: #F97316; }

.graph-content {
  flex: 1;
  overflow: auto;
  padding: 20px;
  min-height: 300px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #94A3B8;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.empty-text {
  font-size: 16px;
  color: #64748B;
}

.empty-hint {
  font-size: 13px;
  color: #94A3B8;
  margin-top: 4px;
}

.task-graph-svg {
  width: 100%;
  height: 100%;
  min-height: 400px;
}

.task-edge {
  transition: stroke 0.2s, stroke-width 0.2s;
}

.task-node-group {
  cursor: pointer;
  transition: transform 0.2s;
}

.task-node-group:hover {
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.1));
}

.node-card {
  transition: all 0.2s;
}

.status-icon {
  font-size: 14px;
  fill: currentColor;
}

.node-name {
  font-size: 13px;
  font-weight: 500;
  max-width: 120px;
}

.agent-badge {
  font-size: 11px;
  fill: #94A3B8;
}

.detail-panel-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.detail-panel {
  background: white;
  border-radius: 12px;
  width: 360px;
  max-width: 90vw;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}

.detail-header {
  padding: 16px 20px;
  border-bottom: 1px solid #E2E8F0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.detail-header h4 {
  font-size: 16px;
  font-weight: 600;
  color: #1E293B;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: #94A3B8;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.close-btn:hover {
  color: #64748B;
}

.detail-body {
  padding: 16px 20px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.detail-row label {
  font-size: 13px;
  color: #64748B;
}

.detail-value {
  font-size: 13px;
  color: #1E293B;
  font-weight: 500;
}

.status-tag {
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  text-transform: capitalize;
}

.status-tag.pending {
  background: #F1F5F9;
  color: #64748B;
}

.status-tag.running {
  background: #DBEAFE;
  color: #1D4ED8;
}

.status-tag.completed {
  background: #DCFCE7;
  color: #166534;
}

.status-tag.failed {
  background: #FEE2E2;
  color: #991B1B;
}

.status-tag.blocked {
  background: #FED7AA;
  color: #9A3412;
}
</style>