# Agent Teams Platform Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把当前 ACP UI 中已经暴露的多 Agent、工作流、总会话、编排监控、Bot 配置、远程控制、记忆能力，从“页面壳和模拟状态”改造成可运行、可验证、可回滚的 Agent Teams Platform。

**Architecture:** 先建立统一的 Agent Runtime 和任务生命周期模型，所有高级页面都只消费同一套运行态、历史态和事件流。前端负责交互、视图和本地 Web 回退，Tauri/Rust 负责桌面本地进程、SQLite 持久化、远程控制服务、Bot 适配器和安全边界。任何功能没有真实后端闭环前，不允许在 UI 中显示成“已运行成功”。

**Tech Stack:** Vue 3、Pinia、TypeScript、Vite、Tauri 2、Rust、rusqlite、tokio、tokio-tungstenite、reqwest、Vitest、Playwright。

---

## 执行原则

这份计划面向 Claude Code 执行，默认工作目录是 `D:\dingsun\acp-ui`。

必须遵守：

- 不删除用户现有未提交改动，先用 `git status --short` 确认工作区。
- 不继续增加仅展示用的假数据、假响应、假成功提示。
- 每个阶段结束都运行指定命令，并把真实输出摘要写入最终交付说明。
- 前端新增业务逻辑必须有 Vitest 覆盖；关键用户路径必须有 Playwright 覆盖。
- Rust 侧所有 SQL 查询必须使用参数绑定，不允许拼接用户输入。
- 每个任务独立提交；提交前必须只 stage 当前任务相关文件。

全局验证命令：

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

新增测试后还要运行：

```powershell
npm run test -- --run
npx playwright test
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 当前问题基线

这些是本计划要消灭的具体问题：

- `src/components/MultiAgentChat.vue` 现在只显示“正在处理”，没有真实调用多 Agent 服务。
- `src/lib/multi-agent/bridge.ts` 的 `TaskResult.output` 总是空字符串，且没有从 `session/update` 汇总输出。
- `src/components/WorkflowView.vue` 加载的是固定示例数据，不是用户创建和执行的真实工作流。
- `src/lib/core/agent-teams-service.ts` 的 agent 配置加载返回空对象，服务启动后不会真正连接配置中的 Agent。
- `src/components/SessionTabs.vue` 新建总会话默认用第一个 Agent 和 `.` 工作目录，和主会话“必须绝对路径”的规则冲突。
- `src-tauri/src/lib.rs` 中 Gateway 配置没有持久化，保存只打印日志。
- `src-tauri/src/lib.rs` 中内存查询存在 SQL 字符串拼接，必须改成参数绑定。
- `src-tauri/src/websocket.rs` 只接收远程命令并转事件，没有认证、响应、权限边界和真实命令回执。
- `src-tauri/src/bot.rs` 可以解析命令，但很多动作只发事件，没有形成统一的任务创建、执行、状态返回链路。
- UI 入口同时暴露太多能力，但成熟度不一致，用户无法判断哪些能用、哪些失败、哪些需要配置。

---

## 目标产品形态

实现完成后，用户应该看到的是一个“Agent Teams 工作台”：

- 左侧仍保留基础 Agent 选择、工作目录、会话列表。
- 主区域有一个统一的任务/会话工作台，能在单会话、多会话、多 Agent、工作流之间切换。
- 所有高级页面共享同一套 Agent 连接池、任务队列、执行计划、历史记录和记忆库。
- 多 Agent 聊天能真实选择多个 Agent、广播或路由任务，并显示每个 Agent 的输出。
- 总会话支持多个 Agent 会话并行存在，不能再默认使用无效 cwd。
- 工作流支持创建、保存、运行、暂停、取消、查看节点输出。
- 编排监控显示真实执行计划，而不是空面板或单独事件孤岛。
- Bot 配置能保存、校验、启动、停止；所有 Bot 指令都走统一任务入口。
- 远程控制至少支持带 token 的 WebSocket App 通道，能返回命令回执和状态快照。
- 记忆支持全局、Agent、会话、任务四种作用域，并能在发送任务前显式注入相关记忆。

---

## 文件结构总览

### 新增前端文件

- `src/lib/agent-runtime/types.ts`  
  定义统一运行时类型：Agent 连接、会话、输出块、任务、计划、事件、错误。

- `src/lib/agent-runtime/acp-session-runner.ts`  
  封装 ACP 初始化、new/load session、prompt、cancel、权限、输出汇总。

- `src/lib/agent-runtime/output-buffer.ts`  
  把 ACP 增量通知归并成可展示、可持久化的消息和任务输出。

- `src/lib/agent-runtime/runtime-errors.ts`  
  标准化错误码和用户可读错误文案。

- `src/lib/agent-runtime/__tests__/output-buffer.test.ts`  
  覆盖 assistant chunk、thought chunk、tool call、tool update、final output 汇总。

- `src/lib/agent-runtime/__tests__/acp-session-runner.test.ts`  
  用 fake transport 验证 new session、load session、prompt、cancel 和断线处理。

- `src/lib/team-service/types.ts`  
  定义 TeamService、Workflow、ExecutionPlan、Gateway、Bot 的前后端共享类型。

- `src/lib/team-service/agent-teams-service.ts`  
  替代当前 `src/lib/core/agent-teams-service.ts` 的核心服务实现。

- `src/stores/team-runtime.ts`  
  统一 Pinia store，作为多 Agent、工作流、编排监控、状态面板的数据源。

- `src/stores/workflows.ts`  
  管理工作流定义、创建、运行、暂停、取消、删除。

- `src/stores/gateway.ts`  
  管理远程控制和 Bot 配置，包含保存、校验、启动状态、连接状态。

- `src/components/workflows/WorkflowEditor.vue`  
  工作流创建和编辑表单。

- `src/components/workflows/WorkflowRunPanel.vue`  
  工作流运行详情和节点输出。

- `src/components/team/AgentPicker.vue`  
  多 Agent 选择器，支持能力、状态、传输类型展示。

- `src/components/team/TaskComposer.vue`  
  多 Agent 和工作流共用的任务输入组件。

- `src/components/team/ExecutionTimeline.vue`  
  统一展示任务事件流。

- `tests/e2e/agent-teams.spec.ts`  
  覆盖高级功能入口、无配置状态、创建多会话、创建工作流、远程配置保存。

### 修改前端文件

- `package.json`  
  增加测试脚本和测试依赖。

- `vite.config.ts`  
  增加 Vitest 配置。

- `src/App.vue`  
  导航改为基于功能注册表渲染；高级页面使用统一 store。

- `src/components/MultiAgentChat.vue`  
  从假响应改为调用 `team-runtime`。

- `src/components/MultiSessionChat.vue`  
  使用 `acp-session-runner`，修正状态、错误和恢复。

- `src/components/SessionTabs.vue`  
  新建会话弹出 Agent 和 cwd 选择，不再默认 `.`。

- `src/components/WorkflowView.vue`  
  删除固定示例数据，改为读取 `workflows` store。

- `src/components/TeamOrchestrationView.vue`  
  改为消费真实计划和任务事件。

- `src/components/PlanVisualization.vue`  
  改为纯展示组件，props 输入，不直接拉取全局状态。

- `src/components/GatewaySettings.vue`  
  改为使用 `gateway` store，保存、校验、启动、停止都显示真实结果。

- `src/components/BotSettings.vue`  
  改为配置和测试 Bot 指令，不再只监听事件。

- `src/components/MemoryView.vue`  
  增加 scope 过滤和任务注入说明。

- `src/stores/session.ts`  
  迁移到 `AcpSessionRunner`，保留单会话 API。

- `src/stores/multi-session.ts`  
  迁移到统一运行时，修复 cwd、输出、恢复和持久化。

- `src/stores/agent-pool.ts`  
  改成 `team-runtime` 的兼容层或删除直接状态复制。

- `src/lib/multi-agent/bridge.ts`  
  迁移到 `team-service`，输出不再为空。

- `src/lib/core/agent-teams-service.ts`  
  删除或改成 re-export，避免两套服务。

- `src/lib/workflow/skill-engine.ts`  
  执行步骤必须真实调用 TeamService，不再模拟成功。

### 新增 Rust 文件

- `src-tauri/src/gateway_config.rs`  
  Gateway 配置持久化、校验、敏感字段遮蔽。

- `src-tauri/src/gateway_server.rs`  
  远程控制 WebSocket 服务，包含 token 认证、请求响应、状态快照。

- `src-tauri/src/bot_adapters/mod.rs`  
  Bot 适配器 trait 和生命周期管理。

- `src-tauri/src/bot_adapters/app_ws.rs`  
  App WebSocket 通道适配器。

- `src-tauri/src/bot_adapters/telegram.rs`  
  Telegram long polling 适配器。

- `src-tauri/src/security.rs`  
  token 生成、token hash、配置脱敏、输入校验。

### 修改 Rust 文件

- `src-tauri/Cargo.toml`  
  如需 HTTP webhook，再增加 `axum`；先不要加用不上的库。

- `src-tauri/src/lib.rs`  
  拆分超大命令实现，只保留 command 注册和 app state。

- `src-tauri/src/database.rs`  
  增加 workflow、plan、event、gateway_config、memory scope 表；修复 SQL 拼接。

- `src-tauri/src/teams.rs`  
  执行计划、任务状态、Agent 输出、完成态必须落库。

- `src-tauri/src/websocket.rs`  
  合并或迁移到 `gateway_server.rs`。

- `src-tauri/src/bot.rs`  
  Bot 命令统一调用 TeamOrchestrator，而不是只发事件。

---

## Task 0: 建立安全执行基线

**Files:**

- Modify: `package.json`
- Modify: `vite.config.ts`
- Create: `src/test/setup.ts`
- Create: `tests/e2e/agent-teams.spec.ts`

- [ ] **Step 1: 确认工作区状态**

Run:

```powershell
git status --short
```

Expected:

- 输出可能包含已有修改。
- 记录哪些文件是用户现有改动。
- 不运行 reset、checkout、clean。

- [ ] **Step 2: 创建实施分支**

Run:

```powershell
git switch -c codex/agent-teams-platform-completion
```

Expected:

- 成功切到新分支。
- 如果分支已存在，使用 `git switch codex/agent-teams-platform-completion`。

- [ ] **Step 3: 增加测试依赖和脚本**

Modify `package.json`:

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest",
    "test:e2e": "playwright test"
  },
  "devDependencies": {
    "@playwright/test": "^1.52.0",
    "@vue/test-utils": "^2.4.6",
    "jsdom": "^26.1.0",
    "vitest": "^3.1.0"
  }
}
```

保留现有脚本，只追加缺失项。

- [ ] **Step 4: 配置 Vitest**

Modify `vite.config.ts`，在 `defineConfig` 返回对象里追加：

```ts
test: {
  environment: "jsdom",
  setupFiles: ["src/test/setup.ts"],
  globals: true,
},
```

如果 TypeScript 提示 `test` 不属于 Vite 类型，在文件顶部增加：

```ts
/// <reference types="vitest" />
```

- [ ] **Step 5: 创建测试 setup**

Create `src/test/setup.ts`:

```ts
import { vi } from "vitest";

Object.defineProperty(globalThis, "crypto", {
  value: {
    randomUUID: () => "test-uuid",
  },
  configurable: true,
});

vi.stubGlobal("__TAURI_INTERNALS__", undefined);
```

- [ ] **Step 6: 创建 E2E 骨架**

Create `tests/e2e/agent-teams.spec.ts`:

```ts
import { test, expect } from "@playwright/test";

test("advanced feature navigation is visible without fake success states", async ({ page }) => {
  await page.goto("http://127.0.0.1:5173");
  await expect(page.getByRole("heading", { name: "ACP UI" })).toBeVisible();
  await expect(page.getByRole("button", { name: /多Agent/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /工作流/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /总会话/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /编排监控/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /远程控制/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /记忆/ })).toBeVisible();
});
```

- [ ] **Step 7: 验证基线**

Run:

```powershell
npm install
npm run build
npm run test -- --run
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected:

- `npm run build` PASS。
- `npm run test -- --run` PASS。
- `cargo check` PASS；如果仍有 warning，记录到交付说明。

- [ ] **Step 8: Commit**

```powershell
git add package.json package-lock.json vite.config.ts src/test/setup.ts tests/e2e/agent-teams.spec.ts
git commit -m "test: add agent teams verification baseline"
```

---

## Task 1: 定义统一运行时契约

**Files:**

- Create: `src/lib/agent-runtime/types.ts`
- Create: `src/lib/agent-runtime/runtime-errors.ts`
- Create: `src/lib/agent-runtime/output-buffer.ts`
- Create: `src/lib/agent-runtime/__tests__/output-buffer.test.ts`

- [ ] **Step 1: 创建运行时类型**

Create `src/lib/agent-runtime/types.ts`:

```ts
import type { AgentConfig, ChatMessage, PermissionRequest, ToolCallInfo } from "../types";

export type RuntimeTransportKind = "stdio" | "websocket" | "http";
export type RuntimeConnectionStatus = "idle" | "connecting" | "connected" | "busy" | "paused" | "error" | "disconnected";
export type RuntimeTaskStatus = "pending" | "running" | "completed" | "failed" | "cancelled";

export interface RuntimeAgentConfig {
  name: string;
  config: AgentConfig;
  cwd: string;
}

export interface RuntimeSession {
  id: string;
  agentName: string;
  acpSessionId: string;
  cwd: string;
  title: string;
  supportsLoadSession: boolean;
  status: RuntimeConnectionStatus;
  createdAt: number;
  lastUpdated: number;
}

export interface RuntimeTask {
  id: string;
  title: string;
  prompt: string;
  source: "single-session" | "multi-session" | "multi-agent" | "workflow" | "bot" | "remote";
  status: RuntimeTaskStatus;
  targetSessionIds: string[];
  createdAt: number;
  startedAt?: number;
  completedAt?: number;
  error?: string;
}

export interface RuntimeOutput {
  taskId: string;
  sessionId: string;
  agentName: string;
  content: string;
  thought: string;
  messages: ChatMessage[];
  toolCalls: ToolCallInfo[];
  status: RuntimeTaskStatus;
  error?: string;
}

export interface RuntimeEvent {
  id: string;
  taskId?: string;
  sessionId?: string;
  type:
    | "session-created"
    | "session-loaded"
    | "session-closed"
    | "task-started"
    | "task-output"
    | "task-completed"
    | "task-failed"
    | "permission-requested"
    | "transport-closed";
  message: string;
  timestamp: number;
  payload?: unknown;
}

export interface RuntimePromptOptions {
  taskId: string;
  prompt: string;
  source: RuntimeTask["source"];
  memories?: string[];
}

export interface RuntimePermissionBridge {
  getPendingPermission(): PermissionRequest | null;
  resolvePermission(optionId: string): void;
  cancelPermission(): void;
}
```

- [ ] **Step 2: 创建标准错误**

Create `src/lib/agent-runtime/runtime-errors.ts`:

```ts
export class RuntimeError extends Error {
  constructor(
    public readonly code:
      | "agent-not-found"
      | "cwd-required"
      | "cwd-not-absolute"
      | "session-not-found"
      | "transport-closed"
      | "auth-cancelled"
      | "task-failed",
    message: string,
    public readonly cause?: unknown,
  ) {
    super(message);
    this.name = "RuntimeError";
  }
}

export function assertAbsoluteCwd(cwd: string): void {
  const value = cwd.trim();
  if (!value) {
    throw new RuntimeError("cwd-required", "请输入 Agent 所在机器上的绝对工作目录。");
  }
  const isAbsolute = value.startsWith("/") || /^[A-Za-z]:[\\/]/.test(value);
  if (!isAbsolute) {
    throw new RuntimeError("cwd-not-absolute", `工作目录必须是绝对路径，当前值: ${cwd}`);
  }
}

export function toRuntimeError(error: unknown, fallbackCode: RuntimeError["code"]): RuntimeError {
  if (error instanceof RuntimeError) return error;
  const message = error instanceof Error ? error.message : String(error);
  return new RuntimeError(fallbackCode, message, error);
}
```

- [ ] **Step 3: 创建输出缓冲器**

Create `src/lib/agent-runtime/output-buffer.ts`:

```ts
import type { SessionNotification } from "@agentclientprotocol/sdk";
import type { ChatMessage, ToolCallInfo } from "../types";
import type { RuntimeOutput, RuntimeTaskStatus } from "./types";

export class OutputBuffer {
  private messages: ChatMessage[] = [];
  private toolCalls = new Map<string, ToolCallInfo>();
  private content = "";
  private thought = "";
  private status: RuntimeTaskStatus = "running";
  private error: string | undefined;

  constructor(
    private readonly taskId: string,
    private readonly sessionId: string,
    private readonly agentName: string,
  ) {}

  apply(notification: SessionNotification): RuntimeOutput {
    const update = notification.update;

    if (update.sessionUpdate === "user_message_chunk" && update.content.type === "text") {
      this.appendMessage("user", update.content.text);
    }

    if (update.sessionUpdate === "agent_message_chunk" && update.content.type === "text") {
      this.content += update.content.text;
      this.appendMessage("assistant", update.content.text);
    }

    if (update.sessionUpdate === "agent_thought_chunk" && update.content.type === "text") {
      this.thought += update.content.text;
      const last = this.messages[this.messages.length - 1];
      if (last && last.role === "assistant") {
        last.thought = (last.thought ?? "") + update.content.text;
      } else {
        this.messages.push({
          id: crypto.randomUUID(),
          role: "assistant",
          content: "",
          thought: update.content.text,
          timestamp: Date.now(),
          toolCalls: [],
        });
      }
    }

    if (update.sessionUpdate === "tool_call") {
      const toolCall: ToolCallInfo = {
        toolCallId: update.toolCallId,
        title: update.title,
        kind: update.kind || "other",
        status: update.status || "pending",
        locations: update.locations,
      };
      this.toolCalls.set(update.toolCallId, toolCall);
      const lastAssistant = [...this.messages].reverse().find((m) => m.role === "assistant");
      if (lastAssistant) {
        lastAssistant.toolCalls = lastAssistant.toolCalls ?? [];
        lastAssistant.toolCalls.push(toolCall);
      }
    }

    if (update.sessionUpdate === "tool_call_update") {
      const existing = this.toolCalls.get(update.toolCallId);
      if (existing) {
        if (update.title) existing.title = update.title;
        if (update.status) existing.status = update.status;
      }
    }

    return this.snapshot();
  }

  complete(): RuntimeOutput {
    this.status = "completed";
    return this.snapshot();
  }

  fail(error: string): RuntimeOutput {
    this.status = "failed";
    this.error = error;
    return this.snapshot();
  }

  snapshot(): RuntimeOutput {
    return {
      taskId: this.taskId,
      sessionId: this.sessionId,
      agentName: this.agentName,
      content: this.content,
      thought: this.thought,
      messages: this.messages,
      toolCalls: Array.from(this.toolCalls.values()),
      status: this.status,
      error: this.error,
    };
  }

  private appendMessage(role: "user" | "assistant", text: string): void {
    const last = this.messages[this.messages.length - 1];
    if (last && last.role === role) {
      last.content += text;
      return;
    }

    this.messages.push({
      id: crypto.randomUUID(),
      role,
      content: text,
      timestamp: Date.now(),
      toolCalls: role === "assistant" ? [] : undefined,
    });
  }
}
```

- [ ] **Step 4: 覆盖输出缓冲器测试**

Create `src/lib/agent-runtime/__tests__/output-buffer.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { OutputBuffer } from "../output-buffer";

function textUpdate(sessionUpdate: "user_message_chunk" | "agent_message_chunk" | "agent_thought_chunk", text: string) {
  return {
    update: {
      sessionUpdate,
      content: { type: "text", text },
    },
  } as any;
}

describe("OutputBuffer", () => {
  it("merges assistant chunks into one output", () => {
    const buffer = new OutputBuffer("task-1", "session-1", "codex");
    buffer.apply(textUpdate("agent_message_chunk", "hello "));
    const output = buffer.apply(textUpdate("agent_message_chunk", "world"));
    expect(output.content).toBe("hello world");
    expect(output.messages).toHaveLength(1);
    expect(output.messages[0].content).toBe("hello world");
  });

  it("tracks thought and tool call updates", () => {
    const buffer = new OutputBuffer("task-1", "session-1", "codex");
    buffer.apply(textUpdate("agent_message_chunk", "answer"));
    buffer.apply(textUpdate("agent_thought_chunk", "reasoning"));
    const output = buffer.apply({
      update: {
        sessionUpdate: "tool_call",
        toolCallId: "tool-1",
        title: "Read file",
        kind: "read",
        status: "pending",
      },
    } as any);
    expect(output.thought).toBe("reasoning");
    expect(output.toolCalls[0].toolCallId).toBe("tool-1");
  });
});
```

- [ ] **Step 5: 验证**

Run:

```powershell
npm run test -- --run src/lib/agent-runtime/__tests__/output-buffer.test.ts
npm run build
```

Expected:

- OutputBuffer tests PASS。
- Build PASS。

- [ ] **Step 6: Commit**

```powershell
git add src/lib/agent-runtime package.json package-lock.json vite.config.ts
git commit -m "feat: define unified agent runtime contracts"
```

---

## Task 2: 封装真实 ACP Session Runner

**Files:**

- Create: `src/lib/agent-runtime/acp-session-runner.ts`
- Create: `src/lib/agent-runtime/__tests__/acp-session-runner.test.ts`
- Modify: `src/stores/session.ts`

- [ ] **Step 1: 实现 AcpSessionRunner**

Create `src/lib/agent-runtime/acp-session-runner.ts`:

```ts
import type { AuthMethod, InitializeResponse } from "@agentclientprotocol/sdk";
import { AcpClientBridge, createAcpClient } from "../acp-bridge";
import type { AgentConfig, SavedSession } from "../types";
import { OutputBuffer } from "./output-buffer";
import { RuntimeError, assertAbsoluteCwd, toRuntimeError } from "./runtime-errors";
import type { RuntimeOutput, RuntimePromptOptions, RuntimeSession } from "./types";

export interface AcpSessionRunnerOptions {
  agentName: string;
  agentConfig: AgentConfig;
  cwd: string;
  appVersion: string;
  promptForAuthMethod?: (authMethods: AuthMethod[], agentName: string) => Promise<string | null>;
  onOutput?: (output: RuntimeOutput) => void;
  onTransportClose?: (reason?: string) => void;
}

export class AcpSessionRunner {
  private client: AcpClientBridge | null = null;
  private runtimeSession: RuntimeSession | null = null;

  constructor(private readonly options: AcpSessionRunnerOptions) {}

  get session(): RuntimeSession | null {
    return this.runtimeSession;
  }

  get acpClient(): AcpClientBridge | null {
    return this.client;
  }

  async create(): Promise<RuntimeSession> {
    assertAbsoluteCwd(this.options.cwd);
    const initResponse = await this.connectAndInitialize();
    const sessionResponse = await this.client!.newSession({
      cwd: this.options.cwd,
      mcpServers: [],
    });

    this.runtimeSession = {
      id: crypto.randomUUID(),
      agentName: this.options.agentName,
      acpSessionId: sessionResponse.sessionId,
      cwd: this.options.cwd,
      title: `Session ${new Date().toLocaleString()}`,
      supportsLoadSession: initResponse.agentCapabilities?.loadSession ?? false,
      status: "connected",
      createdAt: Date.now(),
      lastUpdated: Date.now(),
    };

    return this.runtimeSession;
  }

  async load(saved: SavedSession): Promise<RuntimeSession> {
    assertAbsoluteCwd(saved.cwd);
    const initResponse = await this.connectAndInitialize();

    try {
      await this.client!.loadSession({
        sessionId: saved.sessionId,
        cwd: saved.cwd,
        mcpServers: [],
      });
    } catch (error) {
      await this.disconnect();
      throw toRuntimeError(error, "session-not-found");
    }

    this.runtimeSession = {
      id: saved.id,
      agentName: saved.agentName,
      acpSessionId: saved.sessionId,
      cwd: saved.cwd,
      title: saved.title,
      supportsLoadSession: saved.supportsLoadSession ?? initResponse.agentCapabilities?.loadSession ?? false,
      status: "connected",
      createdAt: saved.lastUpdated,
      lastUpdated: Date.now(),
    };

    return this.runtimeSession;
  }

  async prompt(options: RuntimePromptOptions): Promise<RuntimeOutput> {
    if (!this.client || !this.runtimeSession) {
      throw new RuntimeError("session-not-found", "没有可用会话。");
    }

    const promptText = options.memories?.length
      ? [`以下是相关记忆：`, ...options.memories.map((m) => `- ${m}`), "", options.prompt].join("\n")
      : options.prompt;

    const buffer = new OutputBuffer(options.taskId, this.runtimeSession.id, this.runtimeSession.agentName);
    const previousHandler = this.client.onSessionUpdate;
    this.client.onSessionUpdate = (notification) => {
      const output = buffer.apply(notification);
      this.options.onOutput?.(output);
      previousHandler?.(notification);
    };

    try {
      await this.client.prompt({
        sessionId: this.runtimeSession.acpSessionId,
        prompt: [{ type: "text", text: promptText }],
      });
      const output = buffer.complete();
      this.options.onOutput?.(output);
      return output;
    } catch (error) {
      const runtimeError = toRuntimeError(error, "task-failed");
      const output = buffer.fail(runtimeError.message);
      this.options.onOutput?.(output);
      throw runtimeError;
    } finally {
      this.client.onSessionUpdate = previousHandler;
      this.runtimeSession.lastUpdated = Date.now();
    }
  }

  async cancel(): Promise<void> {
    if (!this.client || !this.runtimeSession) return;
    await this.client.cancel({ sessionId: this.runtimeSession.acpSessionId });
  }

  async disconnect(): Promise<void> {
    if (this.client) {
      await this.client.disconnect();
      this.client = null;
    }
    if (this.runtimeSession) {
      this.runtimeSession.status = "disconnected";
    }
  }

  private async connectAndInitialize(): Promise<InitializeResponse> {
    this.client = await createAcpClient({
      name: this.options.agentName,
      config: this.options.agentConfig,
    });

    this.client.onTransportClose = (reason) => {
      if (this.runtimeSession) this.runtimeSession.status = "disconnected";
      this.options.onTransportClose?.(reason);
    };

    return await this.client.initialize({
      protocolVersion: 1,
      clientCapabilities: {
        fs: {
          readTextFile: true,
          writeTextFile: true,
        },
      },
      clientInfo: {
        name: "acp-ui",
        title: "ACP UI",
        version: this.options.appVersion,
      },
    });
  }
}
```

- [ ] **Step 2: 把 `src/stores/session.ts` 迁移为调用 runner**

要求：

- 保留现有组件依赖的 store 字段名，避免一次性改爆 UI。
- `createSession` 创建 `AcpSessionRunner` 后保存到闭包变量。
- `sendPrompt` 调 `runner.prompt(...)`，不再在 store 内重复写一套 `handleSessionUpdate`。
- 权限和 auth 弹窗先保持现有交互，runner 的 auth hook 接入 `promptForAuthMethod`。

- [ ] **Step 3: 验证单会话行为**

Run:

```powershell
npm run build
npm run test -- --run
```

Expected:

- 构建通过。
- 单会话 store 类型无错误。
- 页面仍能进入 Welcome 状态。

- [ ] **Step 4: Commit**

```powershell
git add src/lib/agent-runtime src/stores/session.ts
git commit -m "feat: route single sessions through acp runner"
```

---

## Task 3: 修复总会话，让多会话成为真实能力

**Files:**

- Modify: `src/stores/multi-session.ts`
- Modify: `src/components/SessionTabs.vue`
- Modify: `src/components/MultiSessionChat.vue`
- Create: `src/components/multi-session/NewSessionDialog.vue`
- Create: `src/stores/__tests__/multi-session.test.ts`

- [ ] **Step 1: 新建总会话弹窗**

Create `src/components/multi-session/NewSessionDialog.vue`，必须包含：

- Agent 下拉。
- 绝对 cwd 输入框。
- 取消按钮。
- 创建按钮。
- cwd 非绝对路径时显示错误并禁止提交。

关键校验逻辑：

```ts
const isAbsoluteCwd = computed(() => {
  const cwd = selectedCwd.value.trim();
  return cwd.startsWith("/") || /^[A-Za-z]:[\\/]/.test(cwd);
});
```

- [ ] **Step 2: 改 SessionTabs**

Modify `src/components/SessionTabs.vue`:

- 点击 `+` 打开 `NewSessionDialog`。
- 删除 `agentEntries[0]` 和 `agentCfg?.env?.PWD ?? "."` 默认行为。
- 创建成功后关闭弹窗。

- [ ] **Step 3: 改 multi-session store**

Modify `src/stores/multi-session.ts`:

- 使用 `AcpSessionRunner` 管理每个 session。
- `clientInstances` 改成 `runnerInstances`。
- `sendPrompt` 先本地追加用户消息，再调用 runner，并把 output 同步到当前 session。
- `isLoading` 在 prompt 期间置 true。
- 断线时只标记当前 session，不清空其他 session。

- [ ] **Step 4: 增加测试**

Create `src/stores/__tests__/multi-session.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { assertAbsoluteCwd } from "../../lib/agent-runtime/runtime-errors";

describe("multi-session cwd validation", () => {
  it("rejects relative cwd", () => {
    expect(() => assertAbsoluteCwd(".")).toThrow(/绝对路径/);
  });

  it("accepts Windows absolute cwd", () => {
    expect(() => assertAbsoluteCwd("D:\\\\work\\\\repo")).not.toThrow();
  });

  it("accepts Unix absolute cwd", () => {
    expect(() => assertAbsoluteCwd("/work/repo")).not.toThrow();
  });
});
```

- [ ] **Step 5: 验证**

Run:

```powershell
npm run test -- --run src/stores/__tests__/multi-session.test.ts
npm run build
```

Expected:

- relative cwd 测试失败场景被覆盖。
- build PASS。

- [ ] **Step 6: Commit**

```powershell
git add src/stores/multi-session.ts src/components/SessionTabs.vue src/components/MultiSessionChat.vue src/components/multi-session src/stores/__tests__
git commit -m "feat: make multi-session creation explicit and valid"
```

---

## Task 4: 建立真实 Team Runtime 和多 Agent 输出闭环

**Files:**

- Create: `src/lib/team-service/types.ts`
- Create: `src/lib/team-service/agent-teams-service.ts`
- Create: `src/stores/team-runtime.ts`
- Modify: `src/stores/agent-pool.ts`
- Modify: `src/components/MultiAgentChat.vue`
- Modify: `src/components/AgentStatusPanel.vue`

- [ ] **Step 1: 定义 Team Service 类型**

Create `src/lib/team-service/types.ts`:

```ts
import type { RuntimeOutput, RuntimeTask, RuntimeTaskStatus } from "../agent-runtime/types";

export type TeamRoutingMode = "single" | "broadcast" | "round-robin" | "load-balanced";

export interface TeamAgentSelection {
  agentName: string;
  cwd: string;
}

export interface TeamRunRequest {
  title: string;
  prompt: string;
  source: RuntimeTask["source"];
  routing: TeamRoutingMode;
  agents: TeamAgentSelection[];
  memoryScope?: "none" | "global" | "agent" | "session";
}

export interface TeamRunResult {
  task: RuntimeTask;
  outputs: RuntimeOutput[];
  status: RuntimeTaskStatus;
  error?: string;
}
```

- [ ] **Step 2: 实现 AgentTeamsService**

Create `src/lib/team-service/agent-teams-service.ts`:

核心要求：

- 从 `useConfigStore()` 读取真实 Agent 配置，不再返回空对象。
- 为每个选中的 Agent 创建或复用 `AcpSessionRunner`。
- broadcast 模式并行执行。
- single 模式只发给第一个空闲 Agent。
- 输出必须保存到 `outputs`。
- 任何 Agent 失败时，任务状态为 `failed`，但已成功输出仍展示。

- [ ] **Step 3: 新建 team-runtime store**

Create `src/stores/team-runtime.ts`:

必须暴露：

```ts
const agents = ref<Map<string, RuntimeSession>>(new Map());
const tasks = ref<Map<string, RuntimeTask>>(new Map());
const outputs = ref<Map<string, RuntimeOutput[]>>(new Map());
const events = ref<RuntimeEvent[]>([]);

async function runTeamTask(request: TeamRunRequest): Promise<TeamRunResult>;
async function cancelTask(taskId: string): Promise<void>;
function getTaskOutputs(taskId: string): RuntimeOutput[];
function clearError(): void;
```

- [ ] **Step 4: 改 MultiAgentChat**

Modify `src/components/MultiAgentChat.vue`:

- 删除“正在处理”的假响应。
- 增加 Agent 多选。
- 增加 routing 选择：单 Agent、广播、负载均衡。
- 点击发送后调用 `teamRuntime.runTeamTask(...)`。
- 每个 Agent 输出单独卡片展示。
- 如果没有 Agent 配置，显示“请先到 Settings 添加 Agent”，输入框禁用。

- [ ] **Step 5: 改 AgentStatusPanel**

Modify `src/components/AgentStatusPanel.vue`:

- 数据源改为 `team-runtime`。
- 展示连接状态、当前任务、最后活动时间。
- 不再依赖旧 `agent-pool` 中可能不同步的 Map。

- [ ] **Step 6: 验证**

Run:

```powershell
npm run build
npm run test -- --run
```

Manual:

- 打开 `http://127.0.0.1:5173`。
- 点击“多Agent”。
- 无 Agent 时应禁用输入并提示配置。
- 配置 Agent 后，发送任务应显示真实输出或真实错误。

- [ ] **Step 7: Commit**

```powershell
git add src/lib/team-service src/stores/team-runtime.ts src/stores/agent-pool.ts src/components/MultiAgentChat.vue src/components/AgentStatusPanel.vue
git commit -m "feat: connect multi-agent chat to real team runtime"
```

---

## Task 5: 工作流从模拟列表改为真实定义和执行

**Files:**

- Create: `src/stores/workflows.ts`
- Create: `src/components/workflows/WorkflowEditor.vue`
- Create: `src/components/workflows/WorkflowRunPanel.vue`
- Modify: `src/components/WorkflowView.vue`
- Modify: `src/lib/workflow/skill-engine.ts`

- [ ] **Step 1: 定义工作流 store**

Create `src/stores/workflows.ts`:

必须支持：

- `createWorkflow`
- `updateWorkflow`
- `deleteWorkflow`
- `runWorkflow`
- `pauseWorkflow`
- `cancelWorkflow`
- `loadWorkflows`

工作流定义：

```ts
export interface WorkflowDefinition {
  id: string;
  name: string;
  description: string;
  steps: WorkflowStep[];
  createdAt: number;
  updatedAt: number;
}

export interface WorkflowStep {
  id: string;
  name: string;
  prompt: string;
  agentName?: string;
  dependsOn: string[];
}
```

Web 环境先用 localStorage；Tauri 环境在 Task 7 接入 SQLite。

- [ ] **Step 2: 替换 WorkflowView 的固定示例数据**

Modify `src/components/WorkflowView.vue`:

- 删除固定示例数组。
- 页面加载时调用 `workflowStore.loadWorkflows()`。
- 无工作流时展示空状态和“新建工作流”。
- 工作流执行必须调用 `workflowStore.runWorkflow(id)`。

- [ ] **Step 3: 实现工作流执行**

Modify `src/lib/workflow/skill-engine.ts`:

- `execute` 必须调用 `teamRuntime.runTeamTask(...)`。
- 每个 step 根据 `depends_on` 排序执行。
- step 输出作为后续 step 的上下文。
- 失败 step 后，依赖它的 step 标记为 skipped。

- [ ] **Step 4: 增加工作流编辑器**

Create `src/components/workflows/WorkflowEditor.vue`:

- 支持添加 step。
- 支持选择 Agent。
- 支持设置依赖。
- 保存时校验：名称非空、至少一个 step、每个 step prompt 非空、依赖不能指向自己。

- [ ] **Step 5: 验证**

Run:

```powershell
npm run build
npm run test -- --run
```

Manual:

- 进入“工作流”。
- 不应再看到固定的 Feature Implementation、Bug Fix Workflow、Code Review。
- 新建一个两步工作流。
- 运行后应产生真实任务记录或真实错误。

- [ ] **Step 6: Commit**

```powershell
git add src/stores/workflows.ts src/components/WorkflowView.vue src/components/workflows src/lib/workflow/skill-engine.ts
git commit -m "feat: replace workflow demos with executable workflows"
```

---

## Task 6: 编排监控接入真实执行计划

**Files:**

- Modify: `src/lib/orchestration/types.ts`
- Modify: `src/lib/orchestration/orchestrator.ts`
- Modify: `src/components/TeamOrchestrationView.vue`
- Modify: `src/components/PlanVisualization.vue`
- Modify: `src/stores/team-runtime.ts`

- [ ] **Step 1: 统一执行计划类型**

Modify `src/lib/orchestration/types.ts`:

- 保留 `ExecutionPlan`。
- 增加运行时字段：`status`、`startedAt`、`completedAt`、`events`。
- `PlannedTask` 增加 `runtimeTaskId?: string`、`output?: string`、`error?: string`。

- [ ] **Step 2: 修正并行执行逻辑**

Modify `src/lib/orchestration/orchestrator.ts`:

当前 `findParallelTasks` 会在循环中重复执行同级任务。改为：

- 维护 `pending`、`running`、`completed`、`failed` 集合。
- 每轮找出依赖已完成的 pending task。
- 并行执行这一批。
- 执行结束后推进下一轮。

- [ ] **Step 3: PlanVisualization 改为受控组件**

Modify `src/components/PlanVisualization.vue`:

- 移除组件内部 `invoke('list_running_plans')`。
- props 接收 `plans`、`selectedPlanId`。
- emits：`select-plan`、`pause-node`、`resume-node`。
- 空状态文案改为“暂无执行计划”。

- [ ] **Step 4: TeamOrchestrationView 使用 team-runtime**

Modify `src/components/TeamOrchestrationView.vue`:

- 运行中的 Agent 来自 `teamRuntime.agents`。
- 计划来自 `teamRuntime.plans`。
- 节点暂停/恢复调用 `teamRuntime.pausePlanNode` / `resumePlanNode`。
- 每个事件写入 `ExecutionTimeline`。

- [ ] **Step 5: 验证**

Run:

```powershell
npm run build
npm run test -- --run
```

Manual:

- 发起多 Agent 广播任务。
- 打开“编排监控”。
- 能看到对应计划、节点状态、输出或错误。
- 取消任务后节点状态变为 cancelled 或 failed，不再停留 running。

- [ ] **Step 6: Commit**

```powershell
git add src/lib/orchestration src/components/TeamOrchestrationView.vue src/components/PlanVisualization.vue src/stores/team-runtime.ts
git commit -m "feat: show real orchestration plans"
```

---

## Task 7: Rust 持久化和 SQL 安全修复

**Files:**

- Modify: `src-tauri/src/database.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/gateway_config.rs`
- Create: `src-tauri/src/security.rs`

- [ ] **Step 1: 修复 memories 查询参数绑定**

Modify `src-tauri/src/lib.rs` 中 `search_memories` 和 `get_agent_memories`：

- 删除 `format!("WHERE ... '{}'", agent_id)`。
- 使用 `rusqlite::params!` 或 `params_from_iter`。
- limit 只允许数字，不能从字符串拼 SQL。

验收查询：

```powershell
Select-String -Path src-tauri\src\lib.rs,src-tauri\src\database.rs -Pattern "agent_id = '\{|source = '\{|WHERE .*'\{"
```

Expected:

- 不应再搜到拼接用户输入的 SQL。

- [ ] **Step 2: 扩展 SQLite 表**

Modify `src-tauri/src/database.rs` 的 `init_tables`，增加：

```sql
CREATE TABLE IF NOT EXISTS workflows (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  definition_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS execution_plans (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  plan_json TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS runtime_events (
  id TEXT PRIMARY KEY,
  task_id TEXT,
  session_id TEXT,
  event_type TEXT NOT NULL,
  message TEXT NOT NULL,
  payload_json TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS gateway_config (
  id TEXT PRIMARY KEY,
  config_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

- [ ] **Step 3: 实现 gateway config 模块**

Create `src-tauri/src/gateway_config.rs`:

- `load_gateway_config(db: &DatabaseManager) -> Result<GatewayConfig, String>`
- `save_gateway_config(db: &DatabaseManager, config: &GatewayConfig) -> Result<(), String>`
- 敏感字段保存前不打印。
- 返回给 UI 时允许显示已配置状态，但不要回显完整 token。

- [ ] **Step 4: 接入 lib.rs commands**

Modify `src-tauri/src/lib.rs`:

- `get_gateway_config` 从数据库读取。
- `save_gateway_config` 写入数据库。
- 删除保存配置时的明文打印。

- [ ] **Step 5: 验证**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

- cargo check PASS。
- 无 SQL 拼接安全回归。

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/database.rs src-tauri/src/lib.rs src-tauri/src/gateway_config.rs src-tauri/src/security.rs
git commit -m "fix: persist gateway config and secure sqlite queries"
```

---

## Task 8: 远程控制 Gateway 做成真实服务

**Files:**

- Create: `src-tauri/src/gateway_server.rs`
- Modify: `src-tauri/src/websocket.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/stores/gateway.ts`
- Modify: `src/components/GatewaySettings.vue`

- [ ] **Step 1: 定义远程协议**

新增协议结构：

```json
{
  "id": "request-id",
  "type": "command",
  "token": "one-time-or-saved-token",
  "command": "get_status",
  "payload": {}
}
```

响应结构：

```json
{
  "id": "request-id",
  "ok": true,
  "data": {},
  "error": null
}
```

必须支持命令：

- `get_status`
- `list_agents`
- `list_plans`
- `pause_agent`
- `resume_agent`
- `cancel_agent`
- `inject_message`
- `pause_plan_node`
- `resume_plan_node`

- [ ] **Step 2: 实现 token 认证**

Create `src-tauri/src/security.rs`:

- 生成随机 token。
- 保存 token hash。
- WebSocket 请求只比较 hash。
- QR URL 包含 token。
- token 可在 UI 重新生成。

- [ ] **Step 3: 改 GatewaySettings**

Modify `src/components/GatewaySettings.vue`:

- 使用 `useGatewayStore()`。
- 保存配置后重新加载脱敏配置。
- “启动服务”必须调用 `start_ws_server` 或新的 `start_gateway_server`。
- 显示真实绑定地址。
- 生成二维码时显示实际 URL；当前二维码图可以先显示 URL，但不能写“扫描连接”假装有二维码图。若没有二维码库，显示“复制连接 URL”。

- [ ] **Step 4: 命令回执**

Modify `src-tauri/src/gateway_server.rs`:

- 每个远程命令执行后必须向 WebSocket 客户端返回响应。
- 执行失败返回 `ok=false` 和错误文本。
- `get_status` 返回当前 agents、plans、running task 数。

- [ ] **Step 5: 验证**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
```

Manual:

- 打开“远程控制”。
- 保存配置。
- 启动服务。
- 复制 WebSocket URL。
- 用一个简单 WebSocket 客户端发送 `get_status`，应收到 JSON 响应。

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/gateway_server.rs src-tauri/src/websocket.rs src-tauri/src/lib.rs src-tauri/src/security.rs src/stores/gateway.ts src/components/GatewaySettings.vue
git commit -m "feat: add authenticated remote control gateway"
```

---

## Task 9: Bot 配置和命令入口接入统一任务系统

**Files:**

- Modify: `src-tauri/src/bot.rs`
- Create: `src-tauri/src/bot_adapters/mod.rs`
- Create: `src-tauri/src/bot_adapters/app_ws.rs`
- Create: `src-tauri/src/bot_adapters/telegram.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/components/BotSettings.vue`
- Modify: `src/stores/gateway.ts`

- [ ] **Step 1: BotManager 改成真实执行**

Modify `src-tauri/src/bot.rs`:

- `/agent` 创建单 Agent task 并调用 orchestrator 执行。
- `/team` 创建多 Agent task 并调用 orchestrator 执行。
- `/status` 返回真实 running plans。
- `/pause`、`/resume`、`/cancel` 调用对应 Rust command 或 orchestrator 方法。
- `/history` 从数据库读取历史。

- [ ] **Step 2: App WebSocket Bot 适配器**

Create `src-tauri/src/bot_adapters/app_ws.rs`:

- 复用 Gateway WebSocket 连接。
- 收到文本后走 `parse_command`。
- 返回 `BotResponse`。

- [ ] **Step 3: Telegram 适配器**

Create `src-tauri/src/bot_adapters/telegram.rs`:

- 使用 `reqwest` long polling 调 Telegram `getUpdates`。
- 只处理配置中允许的 chat id；如果未配置，首次收到消息时返回“请在桌面端绑定此 chat id”。
- 发送消息调用 `sendMessage`。
- token 只保存在数据库，不打印。

- [ ] **Step 4: BotSettings 显示真实状态**

Modify `src/components/BotSettings.vue`:

- 显示各平台状态：未配置、已保存、运行中、错误。
- 提供“测试指令”输入框，调用 `parse_bot_text` 和 `handle_bot_command`。
- 显示返回的 `BotResponse`。
- 对暂未启动的平台，按钮禁用并显示原因。

- [ ] **Step 5: 验证**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
```

Manual:

- Bot 配置页输入 `/status` 测试。
- 应返回真实 running task 数。
- 输入 `/team test codex,claude-code 分析项目`，没有对应 Agent 时返回明确错误，不创建假成功任务。

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/bot.rs src-tauri/src/bot_adapters src-tauri/src/lib.rs src/components/BotSettings.vue src/stores/gateway.ts
git commit -m "feat: route bot commands into team execution"
```

---

## Task 10: 记忆系统接入任务上下文

**Files:**

- Modify: `src-tauri/src/database.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/stores/memory.ts`
- Modify: `src/components/MemoryView.vue`
- Modify: `src/stores/team-runtime.ts`
- Modify: `src/lib/agent-runtime/acp-session-runner.ts`

- [ ] **Step 1: 扩展记忆 scope**

Modify `src-tauri/src/database.rs` 的 `memories` 表，新增兼容字段：

```sql
ALTER TABLE memories ADD COLUMN scope TEXT DEFAULT 'global';
ALTER TABLE memories ADD COLUMN task_id TEXT;
```

迁移必须容错：如果列已存在，不报错中断。

- [ ] **Step 2: 改 memory store**

Modify `src/stores/memory.ts`:

- `saveMemory(content, tags, scope, agentId, sessionId, taskId)`
- `searchMemories(keyword, scope, agentId, sessionId)`
- `loadRelevantMemories({ prompt, agentName, sessionId, limit })`

Web localStorage 数据结构也同步支持 scope。

- [ ] **Step 3: 任务发送前注入记忆**

Modify `src/stores/team-runtime.ts`:

- 如果 `request.memoryScope !== "none"`，调用 `memoryStore.loadRelevantMemories(...)`。
- 把命中的记忆传给 runner 的 `prompt({ memories })`。
- 事件流里记录注入了几条记忆，但不要把敏感内容写进普通日志。

- [ ] **Step 4: MemoryView 增强**

Modify `src/components/MemoryView.vue`:

- 增加 scope 筛选：全局、Agent、会话、任务。
- 新增记忆时可以选择 scope。
- 展示关联 Agent、Session、Task。

- [ ] **Step 5: 验证**

Run:

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

Manual:

- 添加一条全局记忆。
- 多 Agent 任务选择注入全局记忆。
- Agent 收到的 prompt 前缀包含“以下是相关记忆”。

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/database.rs src-tauri/src/lib.rs src/stores/memory.ts src/components/MemoryView.vue src/stores/team-runtime.ts src/lib/agent-runtime/acp-session-runner.ts
git commit -m "feat: inject scoped memories into team tasks"
```

---

## Task 11: UI 收敛，保留全部能力但消除廉价感

**Files:**

- Create: `src/lib/feature-registry.ts`
- Modify: `src/App.vue`
- Modify: `src/assets/modern.css`
- Modify: all advanced feature components touched above

- [ ] **Step 1: 功能注册表**

Create `src/lib/feature-registry.ts`:

```ts
export interface FeatureEntry {
  id: "chat" | "multi-agent" | "multi-session" | "status" | "monitor" | "history" | "workflow" | "gateway" | "orchestration" | "bot" | "memory";
  label: string;
  icon: string;
  requiresAgent?: boolean;
  description: string;
}

export const FEATURES: FeatureEntry[] = [
  { id: "chat", label: "对话", icon: "💬", requiresAgent: true, description: "单 Agent 会话" },
  { id: "multi-agent", label: "多 Agent", icon: "🤖", requiresAgent: true, description: "广播、路由和对比多个 Agent 输出" },
  { id: "multi-session", label: "总会话", icon: "📋", requiresAgent: true, description: "并行管理多个 Agent 会话" },
  { id: "workflow", label: "工作流", icon: "⚡", requiresAgent: true, description: "保存并执行多步骤任务" },
  { id: "orchestration", label: "编排监控", icon: "🎬", requiresAgent: false, description: "查看执行计划、节点和输出" },
  { id: "bot", label: "Bot 配置", icon: "🤖", requiresAgent: false, description: "配置远程指令入口" },
  { id: "gateway", label: "远程控制", icon: "🌐", requiresAgent: false, description: "WebSocket App 控制通道" },
  { id: "memory", label: "记忆", icon: "💡", requiresAgent: false, description: "管理任务上下文记忆" },
  { id: "status", label: "状态", icon: "📊", requiresAgent: false, description: "查看 Agent 连接池" },
  { id: "monitor", label: "监控", icon: "📡", requiresAgent: false, description: "查看实时事件" },
  { id: "history", label: "历史", icon: "📚", requiresAgent: false, description: "查询任务历史" },
];
```

- [ ] **Step 2: App.vue 使用注册表**

Modify `src/App.vue`:

- 删除硬编码导航按钮。
- 用 `FEATURES` 渲染。
- 没有 Agent 配置时，依赖 Agent 的页面显示明确空状态和 Settings 入口。
- 不显示“请先连接代理以使用此功能”这种泛泛提示，改成针对当前功能的配置建议。

- [ ] **Step 3: 统一视觉密度**

Modify `src/assets/modern.css`:

- 保持工具型应用风格。
- 减少渐变、emoji 堆叠和大卡片。
- 表单、列表、状态、时间线使用一致间距。
- 所有按钮有 disabled、loading、error 状态。

- [ ] **Step 4: 验证 UI**

Run:

```powershell
npm run build
npm run dev:web -- --host 127.0.0.1
```

Manual:

- 打开 `http://127.0.0.1:5173`。
- 逐个点击高级功能。
- 不能出现固定示例数据。
- 不能出现假成功。
- 不能出现无意义的大面积空白。
- 无配置时每页都能告诉用户下一步该做什么。

- [ ] **Step 5: Commit**

```powershell
git add src/lib/feature-registry.ts src/App.vue src/assets/modern.css src/components
git commit -m "feat: unify agent teams navigation and empty states"
```

---

## Task 12: 端到端验收和修复回归

**Files:**

- Modify: `tests/e2e/agent-teams.spec.ts`
- Create: `docs/implementation-artifacts/agent-teams-verification.md`

- [ ] **Step 1: 扩展 Playwright 测试**

Modify `tests/e2e/agent-teams.spec.ts` 覆盖：

- 首屏无配置。
- Settings 添加 remote websocket agent 表单校验。
- 总会话点击 `+` 后必须要求 cwd。
- 工作流页面无固定示例。
- 远程控制配置保存后刷新仍存在已配置状态。
- 记忆新增后可搜索。

- [ ] **Step 2: 运行完整验证**

Run:

```powershell
npm run build
npm run test -- --run
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run dev:web -- --host 127.0.0.1
npx playwright test
```

Expected:

- 所有命令 PASS。
- 如果 Tauri runtime 相关 E2E 在 web 模式无法覆盖，必须在验证文档里标明“未覆盖原因”和“人工验证步骤”。

- [ ] **Step 3: 写验证文档**

Create `docs/implementation-artifacts/agent-teams-verification.md`:

必须包含：

- 改动摘要。
- 每个高级功能的当前状态。
- 每条命令的运行结果。
- 手动验证截图路径。
- 已知限制。
- 回滚方式。

- [ ] **Step 4: 最终工作区检查**

Run:

```powershell
git status --short
git log --oneline -12
```

Expected:

- 只剩计划内改动。
- 每个任务至少一个 commit。

- [ ] **Step 5: Commit**

```powershell
git add tests/e2e/agent-teams.spec.ts docs/implementation-artifacts/agent-teams-verification.md
git commit -m "test: verify agent teams platform completion"
```

---

## 验收标准

Claude Code 完成后，必须逐条回答：

- 多 Agent：是否真实调用多个 Agent？是否能展示每个 Agent 的输出、错误和耗时？
- 工作流：是否可以新建、保存、运行、取消？是否没有固定示例数据？
- 总会话：是否可以显式选择 Agent 和绝对 cwd？是否不会默认 `.`？
- 编排监控：是否显示真实计划和节点状态？是否能暂停/恢复节点？
- Bot 配置：配置是否持久化？测试指令是否返回真实状态？
- 远程控制：WebSocket 是否带 token？远程命令是否有回执？
- 记忆：是否支持 scope？任务发送前是否能注入相关记忆？
- 安全：SQL 是否全部参数绑定？token 是否不明文打印？
- 工程：`npm run build`、`npm run test -- --run`、`cargo check` 是否通过？

任何一项未完成，不允许说“已完成平台”。必须标成“已实现 / 部分实现 / 未实现 / 未验证”。

---

## 回滚方案

如果实施过程中出现主会话不可用：

1. 保留 `src/lib/agent-runtime`，但把 `src/stores/session.ts` 回退到实施前版本。
2. 高级功能入口保留，但禁用运行按钮并显示真实错误。
3. 运行：

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

4. 提交回滚：

```powershell
git add src/stores/session.ts src/App.vue
git commit -m "revert: restore stable single session path"
```

---

## Claude Code 最终输出格式

执行完本计划后，请按下面格式回复：

```markdown
## 完成情况
- 多 Agent: 已实现/部分实现/未实现
- 工作流: 已实现/部分实现/未实现
- 总会话: 已实现/部分实现/未实现
- 编排监控: 已实现/部分实现/未实现
- Bot 配置: 已实现/部分实现/未实现
- 远程控制: 已实现/部分实现/未实现
- 记忆: 已实现/部分实现/未实现

## 关键改动
- ...

## 验证结果
- npm run build: PASS/FAIL
- npm run test -- --run: PASS/FAIL
- cargo check --manifest-path src-tauri/Cargo.toml: PASS/FAIL
- cargo test --manifest-path src-tauri/Cargo.toml: PASS/FAIL
- npx playwright test: PASS/FAIL

## 未完成或未验证
- ...

## 交付文件
- ...
```
