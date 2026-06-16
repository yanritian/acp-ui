# ACP-UI Documentation Index

> Last updated: 2026-06-16

## Quick Start

- [Getting Started](getting-started.md) — 快速上手指南
- [Quick Start Guide](QUICK-START.md) — 详细启动步骤

## Architecture

- [System Architecture](system-architecture.md) — 系统整体架构
- [Multi-Agent Architecture](multi-agent-architecture-full.md) — 多 Agent 架构设计
- [Implementation Plan](implementation-plan-phased.md) — 分阶段实施计划
- [Project Completion Plan](project-completion-plan.md) — 项目完成总计划

## Protocols

- [Worker Protocol](protocols/worker-protocol.md) — Worker 通信协议
- [Goal Protocol](protocols/goal-protocol.md) — Goal 定义协议
- [Event Protocol](protocols/event-protocol.md) — 事件推送协议

## Guides

- [Build a Worker](guides/build-a-worker.md) — Worker 开发指南
- [Agent Teams User Guide](AGENT-TEAMS-USER-GUIDE.md) — Agent Teams 使用指南
- [Collaboration Guide](COLLABORATION-GUIDE.md) — 协作开发指南

## Testing

- [Test Plan](TEST-PLAN.md) — 测试计划
- [Test Report](TEST-REPORT.md) — 测试报告

## Status & Issues

- [Current Issues](current-issues.md) — 当前问题追踪
- [Execution Log](execution-log.md) — 执行日志
- [Implementation Progress](implementation-progress.md) — 实施进度

## RFC & Specifications

- [MVP Spec](mvp-spec.md) — MVP 规范
- [PRD v2](prd-v2-self-evolving-platform.md) — 产品需求文档

## Analysis Reports

- [Project Analysis](project-analysis-report-2026-06-05.md) — 项目分析
- [Refactoring Report](refactoring-report-2026-06-04.md) — 重构报告
- [Pluggable Architecture](pluggable-architecture-design.md) — 可插拔架构设计

## Question Tracking

- [待解决问题](question/01-待解决问题.md)
- [功能优化](question/02-功能优化.md)
- [架构重构](question/03-架构重构.md)
- [执行计划](question/04-execution-plan.md)

### Enhanced Plans

- [ACP-Swarm 总纲](question/enhanced/20260609-1100-acp-swarm总纲.md) — ACP-Swarm 总体规划
- [RFC-001 Goal驱动架构](question/enhanced/20260609-1000-RFC-001-Goal驱动异构蜂群编排.md)
- [项目缺失项清单](question/enhanced/20260615-1430-项目缺失项与问题清单.md)

## Test Reports

- [Bot Adapters Test](test-reports/bot-adapters-test-report.md)
- [MES Agent Test](test-reports/mes-agent-test-report.md)
- [Desktop Touch MCP Test](test-reports/desktop-touch-mcp-test-report.md)
- [ERP Finance Report](test-reports/erp-finance-development-report.md)

## Skill System

- [Skill Meta Tool Architecture](skill-meta-tool-architecture-plan.md)
- [Skill System Plan](skill-system-current-plan.md)
- [Skill System Execution](skill-system-execution-plan.md)

## Flutter

- [Flutter Migration Plan](flutter-migration-plan.md)
- [Flutter Windows Plan](flutter-windows-plan.md)

---

## Crate Documentation

| Crate | Purpose | Key Files |
|-------|---------|-----------|
| acp-core | Protocol definitions | message.rs, worker_protocol.rs, goal_protocol.rs, event_protocol.rs |
| swarm-engine | Goal execution engine | goal.rs, goal_graph.rs, reconcile.rs, topology.rs |
| goal-parser | Natural language parsing | parser.rs, dependency_detector.rs, condition_inference.rs |
| tool-sandbox | Sandbox security | config.rs, patterns.rs |
| hook-runtime | Hook execution | registry.rs, executor.rs |
| workflow-engine | Workflow orchestration | workflow.rs, dag.rs, checkpoint.rs |
| acp-transport | Communication adapters | stdio.rs, websocket.rs, http.rs |
| acp-cli | CLI inspector tool | lib.rs, main.rs |

## Example Files

| File | Description |
|------|-------------|
| examples/hello-swarm.goal.yaml | Basic goal chain example |
| examples/star-topology.goal.yaml | Parallel execution example |
| examples/queen-judgment.goal.yaml | Queen evaluation example |
| examples/compound-condition.goal.yaml | All/Any conditions example |
| examples/http-health.goal.yaml | HTTP health check example |
| examples/output-pattern.goal.yaml | Output matching example |