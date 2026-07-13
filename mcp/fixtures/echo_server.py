import json
import sys


def reply(request_id, result=None, error=None):
    response = {"jsonrpc": "2.0", "id": request_id}
    if error is not None:
        response["error"] = error
    else:
        response["result"] = result
    sys.stdout.write(json.dumps(response, ensure_ascii=False) + "\n")
    sys.stdout.flush()


for line in sys.stdin:
    if not line.strip():
        continue
    request = json.loads(line)
    method = request.get("method")
    request_id = request.get("id")
    if request_id is None:
        continue
    if method == "initialize":
        reply(
            request_id,
            {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "acp-ui-echo", "version": "1.0.0"},
            },
        )
    elif method == "tools/list":
        reply(
            request_id,
            {
                "tools": [
                    {
                        "name": "echo",
                        "description": "Return the supplied text without side effects",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"text": {"type": "string"}},
                            "required": ["text"],
                        },
                    }
                ]
            },
        )
    elif method == "tools/call":
        arguments = request.get("params", {}).get("arguments", {})
        reply(
            request_id,
            {
                "content": [{"type": "text", "text": arguments.get("text", "")}],
                "isError": False,
            },
        )
    else:
        reply(request_id, error={"code": -32601, "message": "Method not found"})
