import type {
  CollaborationNode,
  CollaborationEdge,
  CollaborationEvent,
  AgentCapability,
  CollaborationProtocol,
  ContractDefinition,
  AgentType,
} from '@/lib/collaboration/types'

/**
 * Mock Data Generator for Collaboration Network
 *
 * Generates realistic mock data for testing and demonstration
 */
export class CollaborationMockDataGenerator {
  private agentTypes: AgentType[] = [
    'planner',
    'architect',
    'tdd-guide',
    'code-reviewer',
    'security-reviewer',
    'build-error-resolver',
    'e2e-runner',
    'refactor-cleaner',
    'doc-updater',
    'general-purpose',
  ]

  private agentNames = [
    'Planner Agent',
    'Architect Agent',
    'TDD Guide Agent',
    'Code Reviewer Agent',
    'Security Reviewer Agent',
    'Build Fixer Agent',
    'E2E Runner Agent',
    'Refactor Agent',
    'Doc Writer Agent',
    'General Agent',
  ]

  private taskTemplates = [
    {
      description: 'Analyze project requirements',
      capabilities: ['planning', 'analysis'],
    },
    {
      description: 'Design system architecture',
      capabilities: ['architecture', 'design'],
    },
    {
      description: 'Implement TDD workflow',
      capabilities: ['testing', 'implementation'],
    },
    {
      description: 'Review code quality',
      capabilities: ['review', 'quality'],
    },
    {
      description: 'Perform security audit',
      capabilities: ['security', 'audit'],
    },
    {
      description: 'Fix build errors',
      capabilities: ['build', 'debugging'],
    },
    {
      description: 'Run E2E tests',
      capabilities: ['testing', 'automation'],
    },
    {
      description: 'Refactor legacy code',
      capabilities: ['refactoring', 'optimization'],
    },
    {
      description: 'Update documentation',
      capabilities: ['documentation', 'writing'],
    },
    {
      description: 'Execute general task',
      capabilities: ['execution', 'general'],
    },
  ]

  /**
   * Generate mock collaboration nodes
   */
  generateNodes(count: number = 6): CollaborationNode[] {
    const nodes: CollaborationNode[] = []

    // Circular layout
    const centerX = 400
    const centerY = 300
    const radius = 200

    for (let i = 0; i < count; i++) {
      const angle = (i / count) * 2 * Math.PI
      const x = centerX + radius * Math.cos(angle)
      const y = centerY + radius * Math.sin(angle)

      const agentType = this.agentTypes[i % this.agentTypes.length]
      const agentName = this.agentNames[i % this.agentNames.length]
      const agentId = `${agentType}-${i.toString().padStart(3, '0')}`

      const statuses: ('idle' | 'active' | 'waiting' | 'error')[] = ['idle', 'active', 'waiting', 'error']
      const status = statuses[Math.floor(Math.random() * statuses.length)]

      const maxLoad = Math.floor(Math.random() * 5) + 3
      const currentLoad = status === 'active' ? Math.floor(Math.random() * maxLoad) : 0

      nodes.push({
        id: `node-${agentId}`,
        agentId: agentId,
        agentName: agentName,
        agentType: agentType,
        position: { x, y },
        status: status,
        capabilities: this.generateCapabilitiesForAgent(agentType),
        currentLoad: currentLoad,
        maxLoad: maxLoad,
        description: `${agentName} - Specialized ${agentType} agent`,
        avatarUrl: undefined,
      })
    }

    return nodes
  }

  /**
   * Generate mock collaboration edges
   */
  generateEdges(nodes: CollaborationNode[], count: number = 8): CollaborationEdge[] {
    const edges: CollaborationEdge[] = []

    for (let i = 0; i < count; i++) {
      const sourceIndex = Math.floor(Math.random() * nodes.length)
      let targetIndex = Math.floor(Math.random() * nodes.length)

      // Avoid self-loops
      while (targetIndex === sourceIndex) {
        targetIndex = Math.floor(Math.random() * nodes.length)
      }

      const sourceNode = nodes[sourceIndex]
      const targetNode = nodes[targetIndex]
      const taskTemplate = this.taskTemplates[i % this.taskTemplates.length]

      const statuses: ('pending' | 'flowing' | 'completed' | 'failed')[] = [
        'pending',
        'flowing',
        'completed',
        'failed',
      ]
      const status = statuses[Math.floor(Math.random() * statuses.length)]

      const timestamp = Date.now() - Math.floor(Math.random() * 30000)
      const duration = status === 'completed' || status === 'failed'
        ? Math.floor(Math.random() * 5000) + 100
        : undefined

      edges.push({
        id: `edge-${i}`,
        sourceAgentId: sourceNode.id,
        targetAgentId: targetNode.id,
        taskId: `task-${i}`,
        taskDescription: taskTemplate.description,
        status: status,
        timestamp: timestamp,
        animationProgress: status === 'flowing' ? Math.random() : status === 'completed' ? 1 : 0,
        messageType: this.getRandomMessageType(),
        payloadPreview: `${taskTemplate.description} - Preview`,
        duration: duration,
      })
    }

    return edges
  }

  /**
   * Generate mock collaboration events
   */
  generateEvents(nodes: CollaborationNode[], count: number = 20): CollaborationEvent[] {
    const events: CollaborationEvent[] = []

    const eventTypes = [
      'task_assign',
      'task_transfer',
      'task_complete',
      'task_fail',
      'message_sent',
      'message_received',
      'tool_call',
      'tool_result',
      'status_change',
      'protocol_invoked',
      'capability_match',
      'load_balance',
    ] as const

    const severities: ('info' | 'warning' | 'error' | 'critical')[] = [
      'info',
      'warning',
      'error',
      'critical',
    ]

    for (let i = 0; i < count; i++) {
      const sourceNode = nodes[Math.floor(Math.random() * nodes.length)]
      const targetNode = nodes[Math.floor(Math.random() * nodes.length)]
      const eventType = eventTypes[Math.floor(Math.random() * eventTypes.length)]

      events.push({
        id: `event-${i}`,
        type: eventType,
        timestamp: Date.now() - i * 1000,
        sourceAgentId: sourceNode.agentId,
        targetAgentId: targetNode.agentId,
        taskId: `task-${Math.floor(Math.random() * 8)}`,
        protocolId: Math.random() > 0.5 ? `protocol-${Math.floor(Math.random() * 3)}` : undefined,
        capabilityId: Math.random() > 0.5 ? `cap-${Math.floor(Math.random() * 5)}` : undefined,
        payload: {
          details: this.generateEventPayload(eventType),
        },
        details: {
          summary: this.generateEventSummary(eventType, sourceNode.agentName, targetNode.agentName),
          description: `${eventType} event from ${sourceNode.agentName} to ${targetNode.agentName}`,
          duration: eventType.includes('tool') ? Math.floor(Math.random() * 1000) : undefined,
          success: eventType.includes('complete') ? true : eventType.includes('fail') ? false : undefined,
          error: eventType === 'task_fail' ? 'Task execution failed' : undefined,
          metrics: {
            executionTime: Math.floor(Math.random() * 500),
            resourceUsage: Math.floor(Math.random() * 100),
            messageCount: Math.floor(Math.random() * 10),
            toolCallCount: Math.floor(Math.random() * 5),
            retryCount: Math.floor(Math.random() * 3),
          },
        },
        severity: eventType.includes('fail') ? 'error' : severities[Math.floor(Math.random() * severities.length)],
      })
    }

    return events
  }

  /**
   * Generate mock protocols
   */
  generateProtocols(nodes: CollaborationNode[], count: number = 3): CollaborationProtocol[] {
    const protocols: CollaborationProtocol[] = []

    const protocolNames = [
      'Code Review Protocol',
      'TDD Workflow Protocol',
      'Security Audit Protocol',
    ]

    const protocolDescriptions = [
      'Multi-agent code review workflow',
      'Test-driven development workflow',
      'Security audit and validation workflow',
    ]

    for (let i = 0; i < count; i++) {
      const participants = nodes
        .slice(0, Math.floor(Math.random() * 3) + 2)
        .map(n => n.agentId)

      protocols.push({
        id: `protocol-${i}`,
        name: protocolNames[i],
        version: `1.${i}.0`,
        participants: participants,
        inputContract: this.generateContract('input'),
        outputContract: this.generateContract('output'),
        executionConditions: this.generateConditions(2),
        constraints: this.generateConstraints(3),
        examples: this.generateProtocolExamples(i, 2),
        metadata: {
          author: 'System',
          createdAt: Date.now() - i * 10000,
          updatedAt: Date.now(),
          tags: ['workflow', 'collaboration', protocolNames[i].toLowerCase()],
        },
      })
    }

    return protocols
  }

  /**
   * Generate capabilities for an agent
   */
  private generateCapabilitiesForAgent(agentType: string): AgentCapability[] {
    const capabilityMap: Record<string, AgentCapability[]> = {
      planner: [
        {
          id: 'cap-plan',
          name: 'Task Planning',
          description: 'Break down complex tasks into actionable steps',
          icon: '📋',
          category: 'planning',
          proficiency: 90,
          prerequisites: ['domain-knowledge'],
          outputs: ['task-list', 'execution-plan'],
        },
        {
          id: 'cap-allocate',
          name: 'Resource Allocation',
          description: 'Assign agents to tasks based on capabilities',
          icon: '🎭',
          category: 'orchestration',
          proficiency: 85,
          outputs: ['allocation-map'],
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
          prerequisites: ['architecture-patterns'],
          outputs: ['architecture-doc', 'design-diagrams'],
        },
        {
          id: 'cap-review',
          name: 'Architecture Review',
          description: 'Review and validate architecture decisions',
          icon: '👀',
          category: 'review',
          proficiency: 88,
          outputs: ['review-report'],
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
          prerequisites: ['testing-frameworks'],
          outputs: ['test-files', 'coverage-report'],
        },
        {
          id: 'cap-implement',
          name: 'Implementation',
          description: 'Implement features following TDD workflow',
          icon: '⚡',
          category: 'execution',
          proficiency: 85,
          prerequisites: ['test-coverage'],
          outputs: ['source-code'],
        },
      ],
      codeReviewer: [
        {
          id: 'cap-analyze',
          name: 'Code Analysis',
          description: 'Analyze code quality and patterns',
          icon: '🔍',
          category: 'review',
          proficiency: 90,
          outputs: ['analysis-report', 'quality-metrics'],
        },
        {
          id: 'cap-best-practices',
          name: 'Best Practices',
          description: 'Enforce coding standards and best practices',
          icon: '✓',
          category: 'review',
          proficiency: 85,
          outputs: ['standards-checklist'],
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
          prerequisites: ['security-knowledge'],
          outputs: ['security-report', 'vulnerability-list'],
        },
        {
          id: 'cap-compliance',
          name: 'Compliance Check',
          description: 'Verify compliance requirements',
          icon: '📋',
          category: 'review',
          proficiency: 90,
          outputs: ['compliance-report'],
        },
      ],
    }

    const defaultCapabilities: AgentCapability[] = [
      {
        id: 'cap-general',
        name: 'General Execution',
        description: 'Execute general tasks',
        icon: '🤖',
        category: 'execution',
        proficiency: 70,
        outputs: ['task-result'],
      },
    ]

    return capabilityMap[agentType] || defaultCapabilities
  }

  /**
   * Generate contract definition
   */
  private generateContract(type: 'input' | 'output'): ContractDefinition {
    const contractTypes = ['data', 'message', 'task', 'resource'] as const

    return {
      type: contractTypes[Math.floor(Math.random() * contractTypes.length)],
      schema: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          content: { type: 'string' },
          timestamp: { type: 'number' },
        },
        required: ['id', 'content'],
      },
      required: type === 'input',
      description: `${type} contract for collaboration protocol`,
      validationRules: [
        {
          id: `rule-${type}`,
          type: 'type_check',
          rule: 'typeof content === "string"',
          errorMessage: `${type} must be a string`,
        },
      ],
    }
  }

  /**
   * Generate execution conditions
   */
  private generateConditions(count: number): any[] {
    const conditionTypes = ['precondition', 'postcondition', 'invariant']

    const conditions = []
    for (let i = 0; i < count; i++) {
      conditions.push({
        id: `cond-${i}`,
        type: conditionTypes[i % conditionTypes.length],
        expression: `checkCondition${i}()`,
        description: `${conditionTypes[i % conditionTypes.length]} for protocol execution`,
        priority: i + 1,
      })
    }

    return conditions
  }

  /**
   * Generate constraints
   */
  private generateConstraints(count: number): any[] {
    const constraintTypes = ['timeout', 'resource_limit', 'quality_threshold', 'dependency']

    const constraints = []
    for (let i = 0; i < count; i++) {
      const type = constraintTypes[i % constraintTypes.length]
      const value = type === 'timeout'
        ? Math.floor(Math.random() * 30000) + 5000
        : Math.floor(Math.random() * 100)

      constraints.push({
        id: `constraint-${i}`,
        type: type,
        value: value,
        unit: type === 'timeout' ? 'ms' : '%',
        description: `${type} constraint`,
        enforcement: Math.random() > 0.5 ? 'strict' : 'flexible',
      })
    }

    return constraints
  }

  /**
   * Generate protocol examples
   */
  private generateProtocolExamples(protocolIndex: number, count: number): any[] {
    const examples = []
    for (let i = 0; i < count; i++) {
      examples.push({
        id: `example-${protocolIndex}-${i}`,
        name: `Example ${i + 1}`,
        scenario: `Scenario for protocol ${protocolIndex + 1}`,
        input: {
          taskId: `task-${i}`,
          requirements: ['requirement1', 'requirement2'],
        },
        output: {
          result: 'success',
          artifacts: ['artifact1', 'artifact2'],
        },
        notes: ['Note 1', 'Note 2'],
      })
    }

    return examples
  }

  /**
   * Get random message type
   */
  private getRandomMessageType(): 'task_assign' | 'task_transfer' | 'request' | 'response' | 'notification' {
    const types: Array<'task_assign' | 'task_transfer' | 'request' | 'response' | 'notification'> = ['task_assign', 'task_transfer', 'request', 'response', 'notification']
    return types[Math.floor(Math.random() * types.length)]
  }

  /**
   * Generate event payload
   */
  private generateEventPayload(eventType: string): any {
    switch (eventType) {
      case 'tool_call':
        return {
          toolCallId: `tool-${Math.random()}`,
          toolName: 'FileReader',
          parameters: { path: '/src/app.ts' },
        }
      case 'message_sent':
        return {
          messageId: `msg-${Math.random()}`,
          content: 'Example message content',
        }
      case 'task_assign':
        return {
          taskId: `task-${Math.floor(Math.random() * 10)}`,
          requirements: ['req1', 'req2'],
        }
      default:
        return { data: 'payload data' }
    }
  }

  /**
   * Generate event summary
   */
  private generateEventSummary(eventType: string, sourceName: string, targetName: string): string {
    switch (eventType) {
      case 'task_assign':
        return `Task assigned from ${sourceName} to ${targetName}`
      case 'task_transfer':
        return `Task transferred from ${sourceName} to ${targetName}`
      case 'task_complete':
        return `Task completed by ${sourceName}`
      case 'task_fail':
        return `Task failed by ${sourceName}`
      case 'message_sent':
        return `Message sent from ${sourceName} to ${targetName}`
      case 'message_received':
        return `Message received by ${targetName} from ${sourceName}`
      case 'tool_call':
        return `Tool called by ${sourceName}`
      case 'tool_result':
        return `Tool result received by ${sourceName}`
      case 'status_change':
        return `Status changed for ${sourceName}`
      case 'protocol_invoked':
        return `Protocol invoked by ${sourceName}`
      case 'capability_match':
        return `Capability matched for ${sourceName}`
      case 'load_balance':
        return `Load balanced from ${sourceName} to ${targetName}`
      default:
        return `${eventType} event by ${sourceName}`
    }
  }

  /**
   * Generate complete mock dataset
   */
  generateCompleteDataset() {
    const nodes = this.generateNodes(6)
    const edges = this.generateEdges(nodes, 8)
    const events = this.generateEvents(nodes, 20)
    const protocols = this.generateProtocols(nodes, 3)

    return {
      nodes,
      edges,
      events,
      protocols,
      stats: {
        totalAgents: nodes.length,
        activeAgents: nodes.filter(n => n.status === 'active').length,
        totalTasks: edges.length,
        runningTasks: edges.filter(e => e.status === 'flowing').length,
        completedTasks: edges.filter(e => e.status === 'completed').length,
        failedTasks: edges.filter(e => e.status === 'failed').length,
        averageTaskDuration: Math.floor(Math.random() * 3000) + 500,
        collaborationEfficiency: Math.floor(Math.random() * 30) + 70,
        messageCount: events.filter(e => e.type.includes('message')).length,
        toolCallCount: events.filter(e => e.type.includes('tool')).length,
        protocolInvocations: events.filter(e => e.type === 'protocol_invoked').length,
      },
    }
  }
}

/**
 * Singleton instance
 */
export const mockDataGenerator = new CollaborationMockDataGenerator()