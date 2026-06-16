/**
 * Mock Task DAG for demo visualization.
 * Creates a sample DAG showing a feature development workflow.
 *
 * @deprecated (M-2) DEMO DATA - This file is for demonstration only.
 *             Do not use in production. Real DAG should come from task-parser
 *             parsing actual user requests.
 *
 * Used by:
 * - router.ts /task-graph demo page
 * - Any visualization component that needs sample data
 */
import type { TaskDAG } from './task-parser'

export function createMockTaskDag(): TaskDAG {
  // Create a sample DAG for visualization
  const nodes = new Map<string, any>()
  const edges = new Map<string, string[]>()

  const steps = [
    { id: 'step-1', name: 'Parse Request', status: 'completed', agent: 'planner-001', deps: [] },
    { id: 'step-2', name: 'Design Architecture', status: 'running', agent: 'architect-001', deps: ['step-1'] },
    { id: 'step-3', name: 'Write Tests', status: 'pending', agent: 'tddGuide-001', deps: ['step-2'] },
    { id: 'step-4', name: 'Implement Code', status: 'pending', agent: 'codeReviewer-001', deps: ['step-2'] },
    { id: 'step-5', name: 'Security Audit', status: 'pending', agent: 'securityReviewer-001', deps: ['step-3', 'step-4'] },
    { id: 'step-6', name: 'Build & Deploy', status: 'pending', agent: 'build-001', deps: ['step-5'] },
  ]

  for (const step of steps) {
    // Use dag.id prefix for node IDs to match edge building logic
    const nodeId = `demo-dag-001-${step.id}`
    nodes.set(nodeId, {
      id: nodeId,
      step: {
        id: step.id,
        name: step.name,
        action: 'task',
        agentType: 'general',
        // Dependencies use short IDs - TaskGraphView will add dag.id prefix
        dependencies: step.deps
      },
      status: step.status as 'pending' | 'running' | 'completed' | 'failed' | 'blocked',
      assignedAgent: step.agent,
    })

    // Build reverse edges: dependency -> dependent
    for (const dep of step.deps) {
      const depNodeId = `demo-dag-001-${dep}`
      if (!edges.has(depNodeId)) {
        edges.set(depNodeId, [])
      }
      edges.get(depNodeId)!.push(nodeId)
    }
  }

  return {
    id: 'demo-dag-001',
    name: 'Feature Development Workflow',
    nodes,
    edges,
    rootNodes: ['demo-dag-001-step-1'],
  }
}
