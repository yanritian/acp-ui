<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { VueFlow, useVueFlow, type Node, type Edge } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import type {
  CollaborationNode as CollabNode,
  CollaborationEdge as CollabEdge,
  CollaborationNetworkConfig,
} from '@/lib/collaboration/types'
import AgentNode from './AgentNode.vue'
import TaskEdge from './TaskEdge.vue'

// Props
interface Props {
  nodes: CollabNode[]
  edges: CollabEdge[]
  config?: CollaborationNetworkConfig
  selectedNodeId?: string
}

const props = withDefaults(defineProps<Props>(), {
  config: () => ({
    layoutAlgorithm: 'force-directed',
    animationEnabled: true,
    animationSpeed: 0.5,
    nodeSize: 'medium',
    edgeStyle: 'curved',
    showCapabilities: true,
    showProtocols: true,
    showMetrics: true,
    autoRefresh: true,
    refreshInterval: 2000,
  }),
})

// Emits
const emit = defineEmits<{
  nodeClick: [nodeId: string]
  edgeClick: [edgeId: string]
  nodeDrag: [nodeId: string, position: { x: number; y: number }]
  selectionChange: [nodeIds: string[], edgeIds: string[]]
}>()

// Vue Flow setup
const { onNodeClick, onEdgeClick, onNodeDragStop, fitView } = useVueFlow()

// Use reactive refs for Vue Flow (required for v-model)
const flowNodes = ref<Node[]>([])
const flowEdges = ref<Edge[]>([])

// Convert collaboration nodes/edges to Vue Flow format and update refs
function updateFlowData() {
  if (!props.nodes || props.nodes.length === 0) {
    flowNodes.value = []
    flowEdges.value = []
    return
  }

  // Calculate better positions if nodes overlap
  const nodeSpacing = 200
  const startY = 100
  const centerX = 400

  flowNodes.value = props.nodes.map((node, index) => {
    // Better layout: spread nodes horizontally
    const row = Math.floor(index / 3)
    const col = index % 3
    const rowWidth = Math.min(props.nodes.length - row * 3, 3)
    const offsetX = (rowWidth - 1) * nodeSpacing / 2

    return {
      id: node.id,
      type: 'agent',
      position: {
        x: centerX - offsetX + col * nodeSpacing,
        y: startY + row * 150
      },
      data: {
        agentId: node.agentId,
        agentName: node.agentName,
        agentType: node.agentType,
        status: node.status,
        capabilities: node.capabilities || [],
        currentLoad: node.currentLoad,
        maxLoad: node.maxLoad,
        description: node.description,
        avatarUrl: node.avatarUrl,
        config: props.config,
      },
      class: `agent-node-${node.status}`,
    }
  })

  flowEdges.value = props.edges.map(edge => {
    const isAnimated = edge.status === 'flowing' && props.config.animationEnabled

    return {
      id: edge.id,
      source: edge.sourceAgentId,
      target: edge.targetAgentId,
      type: 'task',
      animated: isAnimated,
      data: {
        taskId: edge.taskId,
        taskDescription: edge.taskDescription,
        status: edge.status,
        timestamp: edge.timestamp,
        messageType: edge.messageType,
        payloadPreview: edge.payloadPreview,
        duration: edge.duration,
        animationProgress: edge.animationProgress,
      },
      class: `task-edge-${edge.status}`,
      style: {
        stroke: getEdgeColor(edge.status),
        strokeWidth: getEdgeWidth(edge.status),
      },
    }
  })
}

function getEdgeColor(status: string): string {
  const colors: Record<string, string> = {
    pending: '#FFA500',
    flowing: '#3B82F6',
    completed: '#10B981',
    failed: '#EF4444',
  }
  return colors[status] || '#94A3B8'
}

function getEdgeWidth(status: string): number {
  const widths: Record<string, number> = {
    pending: 2,
    flowing: 3,
    completed: 2,
    failed: 2,
  }
  return widths[status] || 2
}

interface AgentNodeData {
  agentId: string
  agentName: string
  status: string
  capabilities: any[]
  currentLoad: number
  maxLoad: number
}

function getNodeColor(node: { data: AgentNodeData }): string {
  const status = node?.data?.status || 'idle'
  const colors: Record<string, string> = {
    idle: '#94A3B8',
    active: '#3B82F6',
    waiting: '#F59E0B',
    error: '#EF4444',
  }
  return colors[status] || '#94A3B8'
}

// Watch props and update flow data
watch([() => props.nodes, () => props.edges], updateFlowData, { immediate: true, deep: true })

// Event handlers
onNodeClick((event) => {
  emit('nodeClick', event.node.id)
})

onEdgeClick((event) => {
  emit('edgeClick', event.edge.id)
})

onNodeDragStop((event) => {
  emit('nodeDrag', event.node.id, event.node.position)
})

// Auto fit view on mount and after data changes
onMounted(() => {
  updateFlowData()
  setTimeout(() => {
    if (flowNodes.value.length > 0) {
      fitView({ padding: 0.3, duration: 200 })
    }
  }, 300)
})

// Custom node types
const nodeTypes = { agent: AgentNode }
// Custom edge types
const edgeTypes = { task: TaskEdge }
</script>

<template>
  <div class="network-flow-container">
    <VueFlow
      v-model:nodes="flowNodes"
      v-model:edges="flowEdges"
      :node-types="nodeTypes"
      :edge-types="edgeTypes"
      :default-edge-options="{ type: 'task', animated: config.animationEnabled }"
      :fit-view-on-init="true"
      :snap-to-grid="false"
      :nodes-draggable="true"
      :nodes-connectable="true"
      :elevate-nodes-on-select="true"
      class="vue-flow-instance"
    >
      <Background pattern-color="#E2E8F0" :gap="24" />
      <Controls position="bottom-left" />
      <MiniMap position="bottom-right" :node-color="getNodeColor" pannable zoomable />
    </VueFlow>
  </div>
</template>

<style>
/* Vue Flow base styles - MUST be unscoped */
@import '@vue-flow/core/dist/style.css';
@import '@vue-flow/core/dist/theme-default.css';
@import '@vue-flow/controls/dist/style.css';
@import '@vue-flow/minimap/dist/style.css';
</style>

<style scoped>
.network-flow-container {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #F8FAFC 0%, #F1F5F9 100%);
  border-radius: 8px;
}

.vue-flow-instance {
  width: 100%;
  height: 100%;
}
</style>