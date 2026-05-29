---
name: delegation
category: management
description: Delegate tasks to sub-agents
version: 1.0.0
parameters:
  task:
    type: string
    required: true
    description: Task to delegate
  agent:
    type: string
    required: false
    description: Target agent name
  priority:
    type: enum
    values: ["low", "medium", "high", "critical"]
    required: false
    default: "medium"
---

# Delegation

## Steps
1. Analyze task complexity and requirements
2. Select appropriate agent based on capabilities
3. Package task with context and parameters
4. Send to target agent and monitor progress
5. Collect results and report back

## Tools Used
- delegate_task
