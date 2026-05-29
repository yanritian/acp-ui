---
name: communication
category: management
description: Send messages and request clarification
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["send_message", "clarify", "notify"]
    required: true
  recipient:
    type: string
    required: false
  message:
    type: string
    required: true
  channel:
    type: string
    required: false
    default: "default"
---

# Communication

## Steps
1. Identify communication type (message, clarification, notification)
2. Route to appropriate channel (user, agent, external)
3. Format message with context
4. Deliver and confirm receipt
5. Log communication for audit

## Tools Used
- send_message, clarify
