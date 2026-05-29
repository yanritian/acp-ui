---
name: task-management
category: management
description: Todo lists, planning, and task tracking
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["add", "list", "complete", "delete", "plan"]
    required: true
  task:
    type: string
    required: false
  priority:
    type: integer
    required: false
    default: 3
---

# Task Management

## Steps
1. Parse action and validate parameters
2. For add: create task with priority and metadata
3. For list: filter by status/priority/date
4. For complete: update status and record completion time
5. For plan: break down into subtasks with dependencies

## Tools Used
- todo, cronjob
