import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useCollaborationStore } from '@/stores/collaboration'
import type {
  CollaborationNode,
  CollaborationEdge,
  CollaborationEvent,
  AgentCapability,
} from '@/lib/collaboration/types'

describe('Collaboration Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  describe('Node Management', () => {
    it('should add a node', () => {
      const store = useCollaborationStore()

      const node: CollaborationNode = {
        id: 'node-1',
        agentId: 'planner-001',
        agentName: 'Planner',
        agentType: 'planner',
        position: { x: 100, y: 100 },
        status: 'idle',
        capabilities: [],
        currentLoad: 0,
        maxLoad: 5,
      }

      store.addNode(node)

      expect(store.nodes).toHaveLength(1)
      expect(store.nodes[0].agentName).toBe('Planner')
    })

    it('should update a node', () => {
      const store = useCollaborationStore()

      const node: CollaborationNode = {
        id: 'node-1',
        agentId: 'planner-001',
        agentName: 'Planner',
        agentType: 'planner',
        position: { x: 100, y: 100 },
        status: 'idle',
        capabilities: [],
        currentLoad: 0,
        maxLoad: 5,
      }

      store.addNode(node)
      store.updateNode('node-1', { status: 'active', currentLoad: 3 })

      expect(store.nodes[0].status).toBe('active')
      expect(store.nodes[0].currentLoad).toBe(3)
    })

    it('should remove a node', () => {
      const store = useCollaborationStore()

      const node: CollaborationNode = {
        id: 'node-1',
        agentId: 'planner-001',
        agentName: 'Planner',
        agentType: 'planner',
        position: { x: 100, y: 100 },
        status: 'idle',
        capabilities: [],
        currentLoad: 0,
        maxLoad: 5,
      }

      store.addNode(node)
      store.removeNode('node-1')

      expect(store.nodes).toHaveLength(0)
    })
  })

  describe('Edge Management', () => {
    it('should add an edge', () => {
      const store = useCollaborationStore()

      const edge: CollaborationEdge = {
        id: 'edge-1',
        sourceAgentId: 'node-1',
        targetAgentId: 'node-2',
        taskId: 'task-1',
        taskDescription: 'Test task',
        status: 'pending',
        timestamp: Date.now(),
        animationProgress: 0,
      }

      store.addEdge(edge)

      expect(store.edges).toHaveLength(1)
      expect(store.edges[0].taskDescription).toBe('Test task')
    })

    it('should update an edge', () => {
      const store = useCollaborationStore()

      const edge: CollaborationEdge = {
        id: 'edge-1',
        sourceAgentId: 'node-1',
        targetAgentId: 'node-2',
        taskId: 'task-1',
        taskDescription: 'Test task',
        status: 'pending',
        timestamp: Date.now(),
        animationProgress: 0,
      }

      store.addEdge(edge)
      store.updateEdge('edge-1', { status: 'flowing', animationProgress: 0.5 })

      expect(store.edges[0].status).toBe('flowing')
      expect(store.edges[0].animationProgress).toBe(0.5)
    })

    it('should remove an edge', () => {
      const store = useCollaborationStore()

      const edge: CollaborationEdge = {
        id: 'edge-1',
        sourceAgentId: 'node-1',
        targetAgentId: 'node-2',
        taskId: 'task-1',
        taskDescription: 'Test task',
        status: 'pending',
        timestamp: Date.now(),
        animationProgress: 0,
      }

      store.addEdge(edge)
      store.removeEdge('edge-1')

      expect(store.edges).toHaveLength(0)
    })
  })

  describe('Event Management', () => {
    it('should add an event', () => {
      const store = useCollaborationStore()

      const event: CollaborationEvent = {
        id: 'event-1',
        type: 'task_assign',
        timestamp: Date.now(),
        sourceAgentId: 'node-1',
        payload: {},
        details: {
          summary: 'Task assigned',
        },
        severity: 'info',
      }

      store.addEvent(event)

      expect(store.events).toHaveLength(1)
      expect(store.events[0].type).toBe('task_assign')
    })

    it('should limit events to 100', () => {
      const store = useCollaborationStore()

      // Add 120 events
      for (let i = 0; i < 120; i++) {
        const event: CollaborationEvent = {
          id: `event-${i}`,
          type: 'task_assign',
          timestamp: Date.now(),
          sourceAgentId: 'node-1',
          payload: {},
          details: {
            summary: 'Task assigned',
          },
          severity: 'info',
        }
        store.addEvent(event)
      }

      expect(store.events).toHaveLength(100)
    })

    it('should clear events', () => {
      const store = useCollaborationStore()

      const event: CollaborationEvent = {
        id: 'event-1',
        type: 'task_assign',
        timestamp: Date.now(),
        sourceAgentId: 'node-1',
        payload: {},
        details: {
          summary: 'Task assigned',
        },
        severity: 'info',
      }

      store.addEvent(event)
      store.clearEvents()

      expect(store.events).toHaveLength(0)
    })
  })

  describe('Computed Properties', () => {
    it('should calculate active nodes', () => {
      const store = useCollaborationStore()

      const node1: CollaborationNode = {
        id: 'node-1',
        agentId: 'planner-001',
        agentName: 'Planner',
        agentType: 'planner',
        position: { x: 100, y: 100 },
        status: 'active',
        capabilities: [],
        currentLoad: 3,
        maxLoad: 5,
      }

      const node2: CollaborationNode = {
        id: 'node-2',
        agentId: 'architect-001',
        agentName: 'Architect',
        agentType: 'architect',
        position: { x: 200, y: 200 },
        status: 'idle',
        capabilities: [],
        currentLoad: 0,
        maxLoad: 5,
      }

      store.addNode(node1)
      store.addNode(node2)

      expect(store.activeNodes).toHaveLength(1)
      expect(store.activeNodes[0].agentName).toBe('Planner')
    })

    it('should calculate flowing edges', () => {
      const store = useCollaborationStore()

      const edge1: CollaborationEdge = {
        id: 'edge-1',
        sourceAgentId: 'node-1',
        targetAgentId: 'node-2',
        taskId: 'task-1',
        taskDescription: 'Test task 1',
        status: 'flowing',
        timestamp: Date.now(),
        animationProgress: 0.5,
      }

      const edge2: CollaborationEdge = {
        id: 'edge-2',
        sourceAgentId: 'node-2',
        targetAgentId: 'node-3',
        taskId: 'task-2',
        taskDescription: 'Test task 2',
        status: 'pending',
        timestamp: Date.now(),
        animationProgress: 0,
      }

      store.addEdge(edge1)
      store.addEdge(edge2)

      expect(store.flowingEdges).toHaveLength(1)
      expect(store.flowingEdges[0].taskDescription).toBe('Test task 1')
    })

    it('should calculate stats', () => {
      const store = useCollaborationStore()

      // Add nodes
      const node1: CollaborationNode = {
        id: 'node-1',
        agentId: 'planner-001',
        agentName: 'Planner',
        agentType: 'planner',
        position: { x: 100, y: 100 },
        status: 'active',
        capabilities: [],
        currentLoad: 3,
        maxLoad: 5,
      }

      const node2: CollaborationNode = {
        id: 'node-2',
        agentId: 'architect-001',
        agentName: 'Architect',
        agentType: 'architect',
        position: { x: 200, y: 200 },
        status: 'idle',
        capabilities: [],
        currentLoad: 0,
        maxLoad: 5,
      }

      store.addNode(node1)
      store.addNode(node2)

      // Add edges
      const edge1: CollaborationEdge = {
        id: 'edge-1',
        sourceAgentId: 'node-1',
        targetAgentId: 'node-2',
        taskId: 'task-1',
        taskDescription: 'Test task 1',
        status: 'completed',
        timestamp: Date.now() - 5000,
        animationProgress: 1,
        duration: 5000,
      }

      const edge2: CollaborationEdge = {
        id: 'edge-2',
        sourceAgentId: 'node-2',
        targetAgentId: 'node-3',
        taskId: 'task-2',
        taskDescription: 'Test task 2',
        status: 'flowing',
        timestamp: Date.now(),
        animationProgress: 0.5,
      }

      store.addEdge(edge1)
      store.addEdge(edge2)

      const stats = store.stats

      expect(stats.totalAgents).toBe(2)
      expect(stats.activeAgents).toBe(1)
      expect(stats.totalTasks).toBe(2)
      expect(stats.runningTasks).toBe(1)
      expect(stats.completedTasks).toBe(1)
      expect(stats.collaborationEfficiency).toBe(50) // 1 completed out of 2 total
    })
  })

  describe('View Mode', () => {
    it('should set view mode', () => {
      const store = useCollaborationStore()

      store.setViewMode('timeline')

      expect(store.viewMode).toBe('timeline')
    })

    it('should default to network view', () => {
      const store = useCollaborationStore()

      expect(store.viewMode).toBe('network')
    })
  })

  describe('Selection', () => {
    it('should select a node', () => {
      const store = useCollaborationStore()

      store.selectNode('node-1')

      expect(store.selectedNodeId).toBe('node-1')
    })

    it('should select an edge', () => {
      const store = useCollaborationStore()

      store.selectEdge('edge-1')

      expect(store.selectedEdgeId).toBe('edge-1')
    })

    it('should clear selection', () => {
      const store = useCollaborationStore()

      store.selectNode('node-1')
      store.selectNode(null)

      expect(store.selectedNodeId).toBeNull()
    })
  })

  describe('Clear All', () => {
    it('should clear all data', () => {
      const store = useCollaborationStore()

      // Add some data
      const node: CollaborationNode = {
        id: 'node-1',
        agentId: 'planner-001',
        agentName: 'Planner',
        agentType: 'planner',
        position: { x: 100, y: 100 },
        status: 'idle',
        capabilities: [],
        currentLoad: 0,
        maxLoad: 5,
      }

      store.addNode(node)
      store.selectNode('node-1')

      // Clear all
      store.clearAll()

      expect(store.nodes).toHaveLength(0)
      expect(store.events).toHaveLength(0)
      expect(store.selectedNodeId).toBeNull()
      expect(store.error).toBeNull()
    })
  })
})