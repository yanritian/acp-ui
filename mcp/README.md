# MCP 能力目录

MCP server 必须由 Operator 运行时登记，并且命令路径必须是 D 盘上的绝对可执行文件。服务通过 JSON-RPC stdio 暴露工具，工具调用结果会进入任务事件和审计链。

## 本地 fixture

`fixtures/echo_server.py` 是一个无网络、无文件写入的 MCP server，用于验证 initialize、tools/list 和 tools/call。运行它时使用 D 盘 Python：

```powershell
D:\Python313\python.exe D:\dingsun\acp-ui\mcp\fixtures\echo_server.py
```

它只用于协议测试，不应作为生产工具注册。
