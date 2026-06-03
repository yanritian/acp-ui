# MES Agent 执行测试报告

**测试日期**: 2026-06-03
**测试时间**: 17:08:07 - 17:08:26
**任务**: 做一个MES系统（制造执行系统），包含：生产计划管理、工单管理、物料管理、质量管理模块

## 测试结果

### 1. WebSocket 连接 ✅
- 服务器：ws://localhost:8080/claude-code
- subprotocol：acp.v1
- 状态：ESTABLISHED (2条连接)

### 2. ACP 协议握手 ✅
| 时间 | 方向 | 方法 | 状态 |
|------|------|------|------|
| 17:08:07.377 | → | initialize | ✅ 发送 |
| 17:08:07.379 | ← | initialize (response) | ✅ 成功 |
| 17:08:07.379 | → | session/new | ✅ 发送 |
| 17:08:07.380 | ← | session/new (response) | ✅ 成功 |
| 17:08:26.050 | → | session/prompt | ✅ 发送 |
| 17:08:26.080 | ← | session/update (notification) | ✅ 收到 |

### 3. UI 状态显示 ✅
- 用户消息正常显示
- 助手消息 "[Planning] 正在分析需求..." 正常显示
- "思考中..." 状态短暂显示后消失
- 输入框和发送按钮在执行时正确禁用

### 4. 多 Agent 工作流 ⚠️ 部分
- Planning Agent 启动成功
- API 调用正常（Alibaba Bailian glm-5/qwen3.6-plus）
- 但后续 Agent 步骤结果未显示在 UI

## 发现的问题

### 问题1: session/update 消息数量不足

**现象**: 只收到一条 session/update 消息（17:08:26.080）
**原因**: 服务器 `real_agent_server.py` 的工作流执行函数使用的是自定义格式，而非 ACP-compliant 格式

**服务器发送的自定义格式**:
```json
{
  "type": "agent_message",
  "agentId": "planner",
  "agentName": "Planning Agent",
  "content": "...",
  "taskId": "...",
  "timestamp": "..."
}
```

**ACP 要求的格式**:
```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "sessionId": "...",
    "update": {
      "sessionUpdate": "agent_message_chunk",
      "content": {"type": "text", "text": "..."}
    }
  }
}
```

### 问题2: session/prompt 响应格式

服务器在 `execute_real_workflow` 中发送了自定义格式消息，但客户端 `output-buffer.ts` 只处理 ACP 格式。

虽然 `output-buffer.ts` 已添加了对 alternative format 的处理（status/output 格式），但服务器发送的是 `type: agent_message` 格式。

## 截图记录

| 文件 | 内容 |
|------|------|
| `mes-agent-execution-2026-06-03.png` | Agent 执行状态 |
| `mes-agent-final-2026-06-03.png` | 最终状态 |
| `mes-acp-traffic-2026-06-03.png` | ACP Traffic Monitor |

## 修复建议

### 服务器端 (real_agent_server.py)

修改 `execute_real_workflow` 函数，使用 ACP-compliant 格式发送进度消息：

```python
async def execute_real_workflow(self, ws, task_id, request):
    for step in workflow_steps:
        # 使用 ACP 格式发送进度
        await ws.send(json.dumps({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": session_id,
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": {"type": "text", "text": f"[{phase}] 正在分析..."}
                }
            }
        }))
        
        # 调用 API
        api_response = await self.call_alibaba_api(agent_id, prompt, context)
        
        # 使用 ACP 格式发送结果
        await ws.send(json.dumps({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": session_id,
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": {"type": "text", "text": api_response}
                }
            }
        }))
```

### 客户端端 (output-buffer.ts)

添加对 `agent_message` 自定义格式的处理：

```typescript
if (!update) {
  const altNotification = notification as Record<string, unknown>
  // 添加对 agent_message 格式的处理
  if (altNotification.type === 'agent_message') {
    this.content += altNotification.content as string
    this.appendMessage('assistant', altNotification.content as string)
    return this.snapshot()
  }
  // ... 其他格式处理
}
```

## 测试结论

**ACP 协议基础通信**: ✅ 成功
**UI 状态显示**: ✅ 成功
**多 Agent 工作流进度更新**: ⚠️ 需修复服务器消息格式

测试证明了 ACP-UI 与 WebSocket Agent Server 的基础通信正常，需要统一消息格式以实现完整的多 Agent 工作流显示。