---
name: planning
category: development
description: Project planning and task breakdown
version: 1.0.0
parameters:
  goal:
    type: string
    required: true
    description: High-level goal to plan for
  depth:
    type: enum
    values: ["high-level", "detailed", "step-by-step"]
    required: false
    default: "detailed"
  constraints:
    type: array
    required: false
    description: Known constraints
---

# Planning

## Steps
1. Understand the goal and constraints
2. Break down into phases/milestones
3. Identify dependencies between tasks
4. Estimate effort for each task
5. Output structured plan with task tree

## Tools Used
- todo, delegate_task
