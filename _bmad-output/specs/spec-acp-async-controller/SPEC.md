---
title: "ACP-UI Async Agent Controller"
status: "draft"
created: "2026-06-07"
updated: "2026-06-07"
companions:
  - "architecture-diagrams.md"
  - "failure-modes.md"
  - "conventions.md"
  - "../planning-artifacts/acp-ui-architecture-refactor-plan.md"
sources:
  - "../planning-artifacts/agent-teams-platform-prd.md"
open_questions:
  - "How to handle Codex CLI authentication across platforms"
  - "Whether mobile push notifications require a relay server"
  - "WeChat/WeCom official API access vs enterprise workaround"
assumptions:
  - "Claude Code CLI is installed and available on the user's machine"
  - "Sync Engine runs only on trusted local devices"
  - "Bot gateway tokens are managed by the user, not stored by ACP-UI"
---

# SPEC.md — ACP-UI Async Agent Controller

## Why

Developers are locked into sitting at their computers to use AI coding agents (Claude Code, Codex, Cursor). ACP-UI is an **async AI Agent controller** — the user sends a task, agents execute locally, results arrive via any channel (desktop app, Flutter mobile, Feishu, Telegram, Discord, WeChat, web). The user can approve, reject, or give feedback from anywhere, at any time.

This is not another AI coding tool. ACP-UI does not build professional agents. It **orchestrates** them.

## Capabilities

### CAP-1: Cross-Platform Async Control

**Intent:** Users can initiate, monitor, and control agent tasks from any platform (Tauri desktop, Flutter mobile, Feishu, Telegram, Discord, WeChat, WeCom, web) with results and progress synchronized across all channels.

**Success:** A user sends a task via Feishu on their phone, watches progress on their desktop app, approves a change from Telegram on the train, and receives final results via Discord — all from the same session.

### CAP-2: Agent Orchestration (驾驭层)

**Intent:** ACP-UI maintains a registry of available agent CLIs (Claude Code, Codex, Cursor, etc.), routes tasks to the best-fit agent based on capability matching, spawns CLI processes with timeout handling, and captures structured results.

**Success:** Given a task description like "refactor this module," the system selects Claude Code (best for code understanding), spawns it, captures output, reports progress in real-time, and returns results.

### CAP-3: Human-in-the-Loop Approval

**Intent:** When agent execution requires human judgment (code review failures, architectural decisions, security checks), the system creates an approval request that can be reviewed and decided (approve/reject/feedback) from any connected channel.

**Success:** An agent encounters a merge conflict during refactoring, sends an approval card with options to all channels, the user approves from their phone, and the agent continues execution.

### CAP-4: Bot Gateway (Multi-Channel Integration)

**Intent:** ACP-UI exposes a unified RichResponse API that adapts messages to the capabilities of each platform — interactive cards on Feishu/Discord, inline buttons on Telegram, text fallback on WeChat, push notifications on mobile.

**Success:** A single execution result renders as an interactive card with approve/reject buttons on Feishu, as an embed with action buttons on Discord, and as formatted text with numbered options on WeChat.

### CAP-5: Local-First Data Persistence

**Intent:** All data (sync entities, approval requests, task history, agent registry) persists in local SQLite databases. No cloud dependency. Multi-device sync uses peer-to-peer or relayed connections with encrypted transport.

**Success:** The user can close and reopen the desktop app, and all tasks, approvals, and sync state are restored. A mobile device connecting to the same local network receives synchronized data without cloud storage.

### CAP-6: Workflow Definition DSL

**Intent:** Users define multi-stage workflows declaratively (JSON) with stage dependencies, agent assignments, approval gates, and retry policies — instead of hardcoding execution flows.

**Success:** A developer writes a 5-stage workflow JSON (plan → code → test → review → deploy), assigns different agents to each stage, and the orchestrator executes it with dependency resolution and failure handling.

### CAP-7: Push Notifications

**Intent:** The system sends real-time notifications for task completion, approval requests, errors, and progress milestones via Web Push, APNs, FCM, or Tauri desktop notifications.

**Success:** When an agent completes a refactoring task, the user receives a notification on their phone with a preview card showing "3 files changed, +128/-45 lines" and [View] [Approve] [Rollback] buttons.

### CAP-8: ACP Protocol

**Intent:** A standardized message format (ACPMessage) with 11 message types and optional AES-256-GCM encryption governs all communication between agents, the orchestrator, and client channels.

**Success:** Any component can send/receive ACP messages, encrypted messages decrypt correctly on the receiving end, and the message type determines routing and handling logic.

## Constraints

1. **Local-first architecture** — All data stays on the user's machine. No cloud storage for code, tasks, or agent outputs. This rules out centralized sync services and requires local SQLite persistence.
2. **CLI-based agent integration** — Agents are invoked via their native CLIs (`claude`, `codex`, `cursor`) with argument-safe invocation (no shell metacharacter injection). Unknown CLIs are rejected, not executed via shell.
3. **Cross-platform sync without cloud** — Multi-device synchronization must work over local network (WiFi/Bluetooth) or user-provided relay, not built-in cloud services.
4. **Open source** — The entire platform is open source with no paid features at launch. Community contributions drive feature velocity.
5. **Process lifecycle management** — Agent CLI processes are tracked by real PID, cancellable via OS-level process termination. Cancellation must kill the actual child process, not set a flag.

## Non-goals

1. **Building professional AI agents** — ACP-UI does not implement code understanding, code generation, or editor intelligence. It integrates existing tools.
2. **Cloud-hosted AI execution** — This is not a cloud IDE or hosted agent platform. Everything runs locally.
3. **Replacing existing workflows** — ACP-UI augments, not replaces, existing developer toolchains. Claude Code still works standalone.
4. **Enterprise SSO/SAML** — Authentication is local. Enterprise identity integration is out of scope.
5. **Real-time collaborative editing** — Multiple users editing the same codebase simultaneously is not supported.

## Success Signal

1. A user can send a task from their phone via Feishu, watch agents execute on their desktop, approve a code review from Telegram, and receive final results — all within 5 minutes, with zero cloud dependency.
2. All 236 existing tests pass with the new architecture.
3. Rust compilation produces zero errors.
4. Flutter mobile app renders three tabs (Progress, Approvals, Instructions) with live data.
5. A new developer can clone the repo, `npm run dev`, and connect Claude Code to run their first task within 10 minutes.
