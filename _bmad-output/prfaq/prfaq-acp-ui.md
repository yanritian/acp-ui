---
title: "PRFAQ: ACP-UI Async Agent Controller"
status: "draft"
created: "2026-06-08"
updated: "2026-06-08"
stage: "1"
inputs:
  - "_bmad-output/planning-artifacts/prds/prd-acp-ui-2026-05-24/prd.md"
  - "_bmad-output/specs/spec-acp-async-controller/SPEC.md"
  - "_bmad-output/planning-artifacts/acp-ui-architecture-refactor-plan.md"
concept_type: "open-source project"
---

# PRFAQ: ACP-UI

## Press Release — Draft

### How It Works

### Getting Started

---

## Customer FAQ

### Q: 

A: 

---

## Internal FAQ

### Q: 

A: 

---

## The Verdict

### Stage 1 Notes

**Concept type**: Open-source project (Apache 2.0)

**User**: Individual developers / small teams who use AI coding agents (Claude Code, Codex, Cursor)

**Problem**: AI agents can only be used sitting at a computer. No async control. No mobile access. No approval from other channels.

**Solution**: Cross-platform async AI agent controller — send tasks from any channel (Tauri Desktop, Flutter Mobile, Feishu, Telegram, Discord), watch progress, approve/reject from anywhere.

**Why now**: AI agents are getting more capable every week. But developers can't be 24/7 in front of screens. The async gap between agent capability and human availability is growing.

**Competitive landscape (June 2026)**: Claude Code, Codex, Cursor all do "sit at computer AI assistant". Nobody does "async control layer". Trigger.dev does cloud-based async AI workflows, but not local-first. This is a blank market.

**Key assumptions to stress-test**:
1. Developers actually want to control agents from their phones (vs. just checking status)
2. Feishu/Telegram/Discord integration is worth the maintenance burden
3. Local-first (no cloud) is a feature, not a limitation

<!-- coaching-notes-stage-1 -->
- **Concept type rationale**: Open-source project, Apache 2.0, fully free with optional convenience features
- **Initial assumptions challenged**: "中国工具链" was the original differentiator — replaced with async control as core value prop
- **Subagent findings**: 2026 AI coding agent market (Claude Code, Codex, Cursor, Devin) all solve "sit at computer" — no async controller exists
- **Key context**: User wants fully open source, community-driven, local-first architecture
