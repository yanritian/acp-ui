import type { RuntimeOutput, RuntimeEvent } from '../agent-runtime/types'
import type {
  CollaborationNode,
  CollaborationEdge,
  CollaborationEvent,
  AgentCapability,
  AgentType,
} from './types'
import { useCollaborationStore } from '@/stores/collaboration'

/**
 * Agent Event Stream Service
 *
 * Converts runtime events to collaboration events and updates the collaboration network visualization
 */
export class AgentEventStreamService {
  private collaborationStore = useCollaborationStore()
  private agentNodes = new Map<string, CollaborationNode>()
  private taskEdges = new Map<string, CollaborationEdge>()
  private eventQueue: CollaborationEvent[] = []
  private processingInterval: ReturnType<typeof setInterval> | null = null

  constructor() {
    // Start event processing interval
    this.processingInterval = setInterval(() => {
      this.processEventQueue()
    }, 100)
  }

  /**
   * Subscribe to runtime output
   */
  subscribeToRuntimeOutput(output: RuntimeOutput): void {
    // Convert runtime output to collaboration updates
    this.handleRuntimeOutput(output)
  }

  /**
   * Subscribe to runtime events
   */
  subscribeToRuntimeEvents(event: RuntimeEvent): void {
    // Convert runtime event to collaboration event
    this.handleRuntimeEvent(event)
  }

  /**
   * Handle runtime output
   */
  private handleRuntimeOutput(output: RuntimeOutput): void {
    // Update agent node status
    this.updateAgentNode(output.agentName, output.status)

    // Create/update task edge
    this.updateTaskEdge(output)

    // Create collaboration events from messages
    this.createEventsFromMessages(output)
  }

  /**
   * Handle runtime event
   */
  private handleRuntimeEvent(event: RuntimeEvent): void {
    const collaborationEvent = this.convertRuntimeEvent(event)
    this.eventQueue.push(collaborationEvent)
  }

  /**
   * Update agent node
   */
  private updateAgentNode(agentName: string, status: string): void {
    const nodeId = `node-${agentName}`

    if (!this.agentNodes.has(nodeId)) {
      // Create new node
      const node: CollaborationNode = {
        id: nodeId,
        agentId: agentName,
        agentName: agentName,
        agentType: this.getAgentType(agentName),
        position: this.calculateNodePosition(this.agentNodes.size),
        status: this.mapStatus(status),
        capabilities: this.getCapabilitiesForAgent(agentName),
        currentLoad: 0,
        maxLoad: 5,
        description: `${agentName} - Active agent`,
      }

      this.agentNodes.set(nodeId, node)
      this.collaborationStore.addNode(node)
    } else {
      // Update existing node
      const node = this.agentNodes.get(nodeId)!
      node.status = this.mapStatus(status)
      this.collaborationStore.updateNode(nodeId, node)
    }
  }

  /**
   * Update task edge
   */
  private updateTaskEdge(output: RuntimeOutput): void {
    const edgeId = `edge-${output.taskId}`

    // Find or create edge
    // Note: We need to determine source and target agents based on context
    // For now, we'll create a self-loop edge if we don't have enough context
    const nodeId = `node-${output.agentName}`

    if (!this.taskEdges.has(edgeId)) {
      const edge: CollaborationEdge = {
        id: edgeId,
        sourceAgentId: nodeId, // Will be updated when we know the source
        targetAgentId: nodeId,
        taskId: output.taskId,
        taskDescription: output.content.slice(0, 100),
        status: this.mapTaskStatus(output.status),
        timestamp: Date.now(),
        animationProgress: output.status === 'running' ? 0.5 : 1,
        messageType: 'task_assign',
        payloadPreview: output.content.slice(0, 50),
        duration: undefined,
      }

      this.taskEdges.set(edgeId, edge)
      this.collaborationStore.addEdge(edge)
    } else {
      // Update existing edge
      const edge = this.taskEdges.get(edgeId)!
      edge.status = this.mapTaskStatus(output.status)
      edge.payloadPreview = output.content.slice(0, 50)

      if (output.status === 'completed' || output.status === 'failed') {
        edge.animationProgress = 1
        edge.duration = Date.now() - edge.timestamp
      }

      this.collaborationStore.updateEdge(edgeId, edge)
    }
  }

  /**
   * Create events from messages
   */
  private createEventsFromMessages(output: RuntimeOutput): void {
    // Create event for each message
    output.messages.forEach((message: any, index: number) => {
      const event: CollaborationEvent = {
        id: `event-${output.taskId}-${index}`,
        type: message.role === 'user' ? 'message_sent' : 'message_received',
        timestamp: message.timestamp,
        sourceAgentId: message.role === 'user' ? 'user' : `node-${output.agentName}`,
        targetAgentId: message.role === 'user' ? `node-${output.agentName}` : undefined,
        taskId: output.taskId,
        payload: { content: message.content },
        details: {
          summary: message.content.slice(0, 50),
          description: message.content,
        },
        severity: 'info',
      }

      this.eventQueue.push(event)
    })

    // Create events for tool calls
    output.toolCalls.forEach((toolCall: any, index: number) => {
      const event: CollaborationEvent = {
        id: `event-${output.taskId}-tool-${index}`,
        type: 'tool_call',
        timestamp: Date.now(),
        sourceAgentId: `node-${output.agentName}`,
        taskId: output.taskId,
        payload: {
          toolCallId: toolCall.toolCallId,
          title: toolCall.title,
          kind: toolCall.kind,
        },
        details: {
          summary: `Tool call: ${toolCall.title}`,
          description: toolCall.title,
          duration: undefined,
        },
        severity: 'info',
      }

      this.eventQueue.push(event)
    })
  }

  /**
   * Convert runtime event to collaboration event
   */
  private convertRuntimeEvent(event: RuntimeEvent): CollaborationEvent {
    return {
      id: `collab-${event.id}`,
      type: this.mapEventType(event.type),
      timestamp: event.timestamp,
      sourceAgentId: event.sessionId || 'unknown',
      taskId: event.taskId,
      sessionId: event.sessionId,
      payload: event.payload,
      details: {
        summary: event.message,
        description: event.message,
      },
      severity: this.mapSeverity(event.type),
    }
  }

  /**
   * Process event queue
   */
  private processEventQueue(): void {
    if (this.eventQueue.length === 0) return

    // Process events in batches
    const batch = this.eventQueue.splice(0, 10)
    batch.forEach(event => {
      this.collaborationStore.addEvent(event)
    })
  }

  /**
   * Helper: Get agent type
   */
  private getAgentType(agentName: string): AgentType {
    const typeMap: Record<string, AgentType> = {
      planner: 'planner',
      architect: 'architect',
      tddGuide: 'tdd-guide',
      codeReviewer: 'code-reviewer',
      securityReviewer: 'security-reviewer',
      buildErrorResolver: 'build-error-resolver',
      e2eRunner: 'e2e-runner',
      refactorCleaner: 'refactor-cleaner',
      docUpdater: 'doc-updater',
      databaseReviewer: 'database-reviewer',
    }

    return typeMap[agentName] || 'general-purpose'
  }

  /**
   * Helper: Calculate node position
   */
  private calculateNodePosition(index: number): { x: number; y: number } {
    // Circular layout
    const radius = 200
    const angle = (index / 6) * 2 * Math.PI
    return {
      x: 400 + radius * Math.cos(angle),
      y: 300 + radius * Math.sin(angle),
    }
  }

  /**
   * Helper: Map status
   */
  private mapStatus(status: string): 'idle' | 'active' | 'waiting' | 'error' {
    const statusMap: Record<string, 'idle' | 'active' | 'waiting' | 'error'> = {
      idle: 'idle',
      connecting: 'waiting',
      connected: 'idle',
      busy: 'active',
      paused: 'waiting',
      error: 'error',
      disconnected: 'idle',
      running: 'active',
      completed: 'idle',
      failed: 'error',
      cancelled: 'idle',
    }

    return statusMap[status] || 'idle'
  }

  /**
   * Helper: Map task status
   */
  private mapTaskStatus(status: string): 'pending' | 'flowing' | 'completed' | 'failed' {
    const statusMap: Record<string, 'pending' | 'flowing' | 'completed' | 'failed'> = {
      pending: 'pending',
      running: 'flowing',
      completed: 'completed',
      failed: 'failed',
      cancelled: 'failed',
    }

    return statusMap[status] || 'pending'
  }

  /**
   * Helper: Get capabilities for agent
   */
  private getCapabilitiesForAgent(agentName: string): AgentCapability[] {
    const capabilitiesByType: Record<string, AgentCapability[]> = {
      planner: [
        {
          id: 'cap-plan',
          name: 'Task Planning',
          description: 'Break down complex tasks into actionable steps',
          icon: '📋',
          category: 'planning',
          proficiency: 90,
        },
      ],
      architect: [
        {
          id: 'cap-design',
          name: 'System Design',
          description: 'Design scalable system architectures',
          icon: '🏗️',
          category: 'planning',
          proficiency: 95,
        },
      ],
      tddGuide: [
        {
          id: 'cap-test',
          name: 'Test Writing',
          description: 'Write comprehensive test suites',
          icon: '🧪',
          category: 'testing',
          proficiency: 92,
        },
      ],
      codeReviewer: [
        {
          id: 'cap-review',
          name: 'Code Review',
          description: 'Review code quality and best practices',
          icon: '👀',
          category: 'review',
          proficiency: 90,
        },
      ],
      securityReviewer: [
        {
          id: 'cap-security',
          name: 'Security Audit',
          description: 'Identify security vulnerabilities',
          icon: '🔒',
          category: 'review',
          proficiency: 95,
        },
      ],
    }

    const type = this.getAgentType(agentName)
    return capabilitiesByType[type] || [
      {
        id: 'cap-general',
        name: 'General Execution',
        description: 'Execute general tasks',
        icon: '🤖',
        category: 'execution',
        proficiency: 70,
      },
    ]
  }

  /**
   * Helper: Map event type
   */
  private mapEventType(type: string): CollaborationEvent['type'] {
    const typeMap: Record<string, CollaborationEvent['type']> = {
      'session-created': 'status_change',
      'session-loaded': 'status_change',
      'session-closed': 'status_change',
      'task-started': 'task_assign',
      'task-output': 'message_sent',
      'task-completed': 'task_complete',
      'task-failed': 'task_fail',
      'permission-requested': 'status_change',
      'transport-closed': 'status_change',
    }

    return typeMap[type] || 'status_change'
  }

  /**
   * Helper: Map severity
   */
  private mapSeverity(type: string): 'info' | 'warning' | 'error' | 'critical' {
    if (type.includes('fail') || type.includes('error')) return 'error'
    if (type.includes('permission')) return 'warning'
    return 'info'
  }

  /**
   * Cleanup
   */
  destroy(): void {
    if (this.processingInterval) {
      clearInterval(this.processingInterval)
    }
    this.eventQueue = []
  }
}

/**
 * Singleton instance
 */
export const agentEventStreamService = new AgentEventStreamService()