---
title: "ACP-UI Experience Design"
status: "final"
created: "2026-06-06"
updated: "2026-06-06"
form_factor: "web-desktop"
---

# EXPERIENCE.md — ACP-UI How It Works

## Foundation

**Form factor**: Web desktop application (Vue 3 + Vite)
**UI system**: Custom CSS variables with automatic dark mode (`prefers-color-scheme: dark`)
**Design reference**: See {DESIGN.md} for visual identity

### Interaction Principles
- All async operations show skeleton loading states
- Page transitions use `fadeInUp` animation (0.3s)
- Card hover lifts element 2px with shadow increase
- Buttons provide tactile feedback via `scale(0.98)` on active

## Information Architecture

```
/
├── / (Dashboard) — Agent status, quick task input, recent tasks
├── /chat — Single Agent conversation session
├── /agent-config — Agent management (add/edit/connect/disconnect)
├── /agent-teams — Multi-agent dashboard with pets and real-time progress
├── /history — Task execution history
├── /workflow — Workflow management and execution
└── /orchestration — Task graph and execution plans
```

## Voice and Tone

**Developer-first**: Technical but accessible language
**Action-oriented**: Buttons use verbs (Execute, Connect, Manage)
**Status clarity**: Running/Stopped — not ambiguous states
**Error messages**: Specific and actionable (not "Something went wrong")

## Component Patterns

### Agent Status Card
- Shows list of configured agents with connection status
- Running agents show green badge with pulse animation
- Clicking an agent selects it for task execution
- Empty state shows onboarding prompt

### Quick Task Input
- Text area with Ctrl+Enter submit shortcut
- Agent selector dropdown for multi-agent setups
- On submit, navigates to chat with task pre-filled

### Agent Config View
- Grid of agent cards with connection status
- Connected cards highlighted with green border
- Modal for adding/editing agents
- Template panel for quick agent setup

## State Patterns

### Loading States
- Skeleton placeholders during async content load
- Agent status cards show skeleton avatar + skeleton text
- Recent tasks list shows skeleton cards during fetch

### Empty States
- No agents configured → Onboarding flow with wizard
- No recent tasks → "Create your first task" prompt
- No agent teams → "Get started" CTA

### Error States
- Connection failure → Red status badge + error toast
- Task submission failure → Inline error message + retry button
- Agent disconnect → Status changes to "Stopped" automatically

## Interaction Primitives

| Pattern | Trigger | Behavior |
|---------|---------|----------|
| Agent selection | Click card | Select agent, highlight border |
| Task submission | Ctrl+Enter | Navigate to chat with task |
| Agent connect | Click connect | Status changes to running (green pulse) |
| Modal open | Click add/edit | Overlay with form |
| Toast dismiss | Auto (5s) or click X | Slide-out animation |

## Accessibility Floor

- All interactive elements have visible focus states
- Color contrast meets WCAG AA (text on colored backgrounds)
- Keyboard navigation supported (tab order follows visual flow)
- Status changes communicated visually AND via text labels
- Skeleton loaders provide loading indication for screen readers

## Key Flows

### Flow 1: First-time setup (Mary, new developer)
1. Mary opens ACP-UI for the first time
2. No agents configured → onboarding flow auto-shows
3. Mary selects "Quick Start" → guided wizard opens
4. Mary configures first agent (Claude Code via websocket)
5. Agent connects → green pulse confirms success
6. Dashboard now shows agent status + quick task input
7. Mary submits first task → navigates to chat

### Flow 2: Task execution (Dev, daily workflow)
1. Dev opens dashboard → sees all agents with status
2. Selects primary agent → card highlights with blue border
3. Types task description → Ctrl+Enter submits
4. Navigates to chat → task pre-filled
5. Watches real-time output → progress updates
6. Task completes → toast notification confirms success

### Flow 3: Agent management (Admin, maintenance)
1. Admin navigates to Agent Config
2. Reviews agent grid → one showing red "Stopped" status
3. Clicks connect → agent reconnects, status changes to green
4. Adds new agent → modal opens, fills form, saves
5. New agent appears in grid with connected border highlight
