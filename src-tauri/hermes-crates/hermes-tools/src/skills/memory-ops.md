---
name: memory-ops
category: management
description: Memory storage and retrieval
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["store", "retrieve", "search", "delete"]
    required: true
  key:
    type: string
    required: false
  content:
    type: string
    required: false
  scope:
    type: string
    required: false
    default: "session"
---

# Memory Operations

## Steps
1. Determine operation type (store/retrieve/search/delete)
2. Apply scope filtering (session/task/global)
3. For store: serialize and persist with metadata
4. For retrieve: query by key or semantic similarity
5. Return results with timestamps

## Tools Used
- memory, session_search
