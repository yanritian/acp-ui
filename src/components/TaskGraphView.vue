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

// Node dimensions
const nodeWidth = 120;
const nodeHeight = 60;
const nodeSpacingX = 150;
const nodeSpacingY = 80;

// Computed
const statusColors: Record<string, string> = {
  pending: '#9CA3AF',
  running: '#3B82F6',
  completed: '#10B981',
  failed: '#EF4444',
  blocked: '#F97316',
};

const getNodeColor = (status: string): string => {
  return statusColors[status] || '#9CA3AF';
};

const viewBox = computed(() => {
  return `0 0 ${canvasWidth.value} ${canvasHeight.value}`;
});

// Methods
const calculateLayout = () => {
  if (!props.dag) {
    nodes.value = [];
    edges.value = [];
    return;
  }

  const dagNodes = Array.from(props.dag.nodes.values());

  // Group nodes by depth (level in DAG)
  const levels = new Map<number, string[]>();

  // Find root nodes (level 0)
  const rootIds = props.dag.rootNodes;
  levels.set(0, rootIds);

  // Calculate levels for each node
  const visited = new Set<string>(rootIds);
  let currentLevel = 0;

  while (visited.size < dagNodes.length) {
    const nextLevel: string[] = [];
    const currentLevelNodes = levels.get(currentLevel) || [];

    for (const nodeId of currentLevelNodes) {
      const node = props.dag.nodes.get(nodeId);
      if (!node) continue;

      // Find nodes that depend on this node
      const dependents = props.dag.edges.get(nodeId) || [];
      for (const depId of dependents) {
        if (!visited.has(depId)) {
          nextLevel.push(depId);
          visited.add(depId);
        }
      }
    }

    if (nextLevel.length > 0) {
      currentLevel++;
      levels.set(currentLevel, nextLevel);
    } else {
      // Handle remaining nodes (might be disconnected or error)
      for (const node of dagNodes) {
        if (!visited.has(node.id)) {
          visited.add(node.id);
          if (!levels.has(currentLevel + 1)) {
            levels.set(currentLevel + 1, []);
          }
          levels.get(currentLevel + 1)?.push(node.id);
        }
      }
    }
  }

  // Calculate positions
  const layoutNodes: GraphNode[] = [];
  const totalLevels = levels.size;

  for (const [level, nodeIds] of levels.entries()) {
    const y = props.orientation === 'vertical'
      ? level * nodeSpacingY + 50
      : canvasHeight.value / 2;

    for (let i = 0; i < nodeIds.length; i++) {
      const nodeId = nodeIds[i];
      const node = props.dag.nodes.get(nodeId);
      if (!node) continue;

      const x = props.orientation === 'vertical'
        ? (canvasWidth.value - nodeIds.length * nodeSpacingX) / 2 + i * nodeSpacingX + nodeWidth / 2
        : level * nodeSpacingX + 50;

      layoutNodes.push({
        id: nodeId,
        name: node.step.name,
        x,
        y,
        status: node.status,
        dependencies: node.step.dependencies,
        agent: node.assignedAgent,
      });
    }
  }

  // Update canvas size
  if (props.orientation === 'vertical') {
    canvasHeight.value = Math.max(600, totalLevels * nodeSpacingY + 100);
    canvasWidth.value = Math.max(800, Math.max(...Array.from(levels.values()).map(l => l.length)) * nodeSpacingX + 100);
  } else {
    canvasWidth.value = Math.max(800, totalLevels * nodeSpacingX + 100);
  }

  nodes.value = layoutNodes;

  // Build edges
  const layoutEdges: GraphEdge[] = [];
  for (const node of layoutNodes) {
    for (const depId of node.dependencies) {
      const fullDepId = `${props.dag?.id}-${depId}`;
      layoutEdges.push({
        from: fullDepId,
        to: node.id,
      });
    }
  }
  edges.value = layoutEdges;
};

const getEdgePath = (edge: GraphEdge): string => {
  const fromNode = nodes.value.find(n => n.id === edge.from);
  const toNode = nodes.value.find(n => n.id === edge.to);

  if (!fromNode || !toNode) return '';

  if (props.orientation === 'vertical') {
    // Vertical layout: connect bottom of from to top of to
    const startX = fromNode.x;
    const startY = fromNode.y + nodeHeight / 2;
    const endX = toNode.x;
    const endY = toNode.y - nodeHeight / 2;

    // Bezier curve
    const midY = (startY + endY) / 2;
    return `M ${startX} ${startY} C ${startX} ${midY}, ${endX} ${midY}, ${endX} ${endY}`;
  } else {
    // Horizontal layout: connect right of from to left of to
    const startX = fromNode.x + nodeWidth / 2;
    const startY = fromNode.y;
    const endX = toNode.x - nodeWidth / 2;
    const endY = toNode.y;

    const midX = (startX + endX) / 2;
    return `M ${startX} ${startY} C ${midX} ${startY}, ${midX} ${endY}, ${endX} ${endY}`;
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
watch(() => props.dag, calculateLayout, { immediate: true });

onMounted(() => {
  calculateLayout();
});
</script>

<template>
  <div class="task-graph-view bg-white rounded-lg shadow">
    <div class="p-4 border-b flex items-center justify-between">
      <h3 class="text-lg font-semibold">Task Graph</h3>
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-500">Status Legend:</span>
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-full bg-gray-400"></span>
          <span class="text-xs">Pending</span>
        </span>
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-full bg-blue-500"></span>
          <span class="text-xs">Running</span>
        </span>
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-full bg-green-500"></span>
          <span class="text-xs">Completed</span>
        </span>
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-full bg-red-500"></span>
          <span class="text-xs">Failed</span>
        </span>
        <span class="flex items-center gap-1">
          <span class="w-3 h-3 rounded-full bg-orange-500"></span>
          <span class="text-xs">Blocked</span>
        </span>
      </div>
    </div>

    <div class="p-4 overflow-auto">
      <div v-if="!dag" class="text-center text-gray-400 py-12">
        No task graph to display. Start a task to see the DAG visualization.
      </div>

      <svg
        v-else
        :viewBox="viewBox"
        class="w-full h-full"
        style="min-height: 400px;"
      >
        <!-- Edges -->
        <g class="edges">
          <path
            v-for="edge in edges"
            :key="`${edge.from}-${edge.to}`"
            :d="getEdgePath(edge)"
            fill="none"
            :stroke="isEdgeHighlighted(edge) ? '#3B82F6' : '#D1D5DB'"
            :stroke-width="isEdgeHighlighted(edge) ? 2 : 1"
            :stroke-dasharray="nodes.find(n => n.id === edge.from)?.status === 'failed' ? '4 2' : 'none'"
            marker-end="url(#arrowhead)"
          />
        </g>

        <!-- Arrow marker -->
        <defs>
          <marker
            id="arrowhead"
            markerWidth="10"
            markerHeight="7"
            refX="9"
            refY="3.5"
            orient="auto"
          >
            <polygon points="0 0, 10 3.5, 0 7" fill="#D1D5DB" />
          </marker>
        </defs>

        <!-- Nodes -->
        <g class="nodes">
          <g
            v-for="node in nodes"
            :key="node.id"
            :transform="`translate(${node.x - nodeWidth / 2}, ${node.y - nodeHeight / 2})`"
            @click="selectNodeHandler(node.id)"
            @mouseenter="hoveredNodeHandler(node.id)"
            @mouseleave="unhoveredNodeHandler"
            class="cursor-pointer"
          >
            <!-- Node background -->
            <rect
              :width="nodeWidth"
              :height="nodeHeight"
              rx="8"
              :fill="selectedNode === node.id ? '#EFF6FF' : '#F9FAFB'"
              :stroke="getNodeColor(node.status)"
              :stroke-width="selectedNode === node.id || hoveredNode === node.id ? 2 : 1"
            />

            <!-- Status indicator -->
            <circle
              cx="10"
              cy="nodeHeight / 2"
              r="6"
              :fill="getNodeColor(node.status)"
            />

            <!-- Node name -->
            <text
              x="25"
              y="nodeHeight / 2"
              text-anchor="start"
              dominant-baseline="middle"
              class="text-sm font-medium fill-gray-900"
            >
              {{ node.name }}
            </text>

            <!-- Agent badge -->
            <g v-if="showAgents && node.agent" :transform="`translate(${nodeWidth - 50}, 10)`">
              <rect
                width="40"
                height="16"
                rx="4"
                fill="#E5E7EB"
              />
              <text
                x="20"
                y="8"
                text-anchor="middle"
                dominant-baseline="middle"
                class="text-xs fill-gray-600"
              >
                {{ node.agent?.split('-')[0] }}
              </text>
            </g>

            <!-- Progress indicator for running nodes -->
            <g v-if="node.status === 'running'">
              <rect
                x="0"
                :y="nodeHeight - 4"
                :width="nodeWidth"
                height="4"
                fill="#E5E7EB"
              />
              <rect
                x="0"
                :y="nodeHeight - 4"
                :width="nodeWidth * 0.5"
                height="4"
                fill="#3B82F6"
              >
                <animate
                  attributeName="width"
                  from="0"
                    to="nodeWidth"
                  dur="2s"
                  repeatCount="indefinite"
                />
              </rect>
            </g>
          </g>
        </g>
      </svg>
    </div>

    <!-- Node detail tooltip -->
    <div
      v-if="selectedNode"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="selectedNode = null"
    >
      <div class="bg-white rounded-lg shadow-xl w-full max-w-md p-4">
        <div class="flex items-center justify-between mb-3">
          <h4 class="font-semibold">Task Details</h4>
          <button @click="selectedNode = null" class="text-gray-400 hover:text-gray-600">
            ✕
          </button>
        </div>
        <div class="space-y-2">
          <div v-for="node in nodes.filter(n => n.id === selectedNode)" :key="node.id">
            <div class="text-sm">
              <span class="text-gray-500">Name:</span>
              <span class="ml-2 font-medium">{{ node.name }}</span>
            </div>
            <div class="text-sm">
              <span class="text-gray-500">Status:</span>
              <span
                class="ml-2 px-2 py-0.5 rounded text-xs"
                :style="{ backgroundColor: getNodeColor(node.status) + '20', color: getNodeColor(node.status) }"
              >
                {{ node.status }}
              </span>
            </div>
            <div class="text-sm">
              <span class="text-gray-500">Dependencies:</span>
              <span class="ml-2">{{ node.dependencies.length > 0 ? node.dependencies.join(', ') : 'None' }}</span>
            </div>
            <div v-if="node.agent" class="text-sm">
              <span class="text-gray-500">Assigned Agent:</span>
              <span class="ml-2">{{ node.agent }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-graph-view {
  font-family: system-ui, -apple-system, sans-serif;
}

svg text {
  font-family: system-ui, -apple-system, sans-serif;
}
</style>