//! MCP JSON-RPC Client Module
//!
//! Implements the actual MCP protocol communication layer.
//! Works alongside `mcp_manager.rs` which handles process lifecycle -
//! this module handles the JSON-RPC 2.0 communication with MCP servers.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use tauri::State;

use crate::AppState;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 Protocol Types
// ---------------------------------------------------------------------------

/// JSON-RPC 2.0 Request
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String, // Always "2.0"
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC error {}: {}", self.code, self.message)
    }
}

// ---------------------------------------------------------------------------
// MCP Protocol Types
// ---------------------------------------------------------------------------

/// Discovered MCP Tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub server_name: String,
}

/// MCP Tool Call Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub tool_name: String,
    pub success: bool,
    pub content: Vec<McpContent>,
    pub is_error: bool,
    pub duration_ms: u64,
}

/// MCP Content types (text, image, resource)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String, text: Option<String> },
}

/// MCP Initialize Result (from server)
#[derive(Debug, Deserialize)]
struct InitializeResult {
    #[serde(default)]
    capabilities: Value,
    #[serde(default, rename = "serverInfo")]
    server_info: Option<ServerInfo>,
}

#[derive(Debug, Deserialize)]
struct ServerInfo {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
}

/// Tool list response from server
#[derive(Debug, Deserialize)]
struct ToolsListResult {
    tools: Vec<RawTool>,
}

#[derive(Debug, Deserialize)]
struct RawTool {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    input_schema: Option<Value>,
}

// ---------------------------------------------------------------------------
// MCP Connection
// ---------------------------------------------------------------------------

/// Connected MCP server with its tools
#[derive(Debug)]
pub struct McpConnection {
    pub server_name: String,
    pub process: Option<Child>,
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<BufReader<ChildStdout>>,
    pub tools: Vec<McpTool>,
    pub next_id: u64,
    pub connected: bool,
}

impl McpConnection {
    /// Create a new connection from a spawned process
    fn new(server_name: &str, mut process: Child) -> Result<Self, String> {
        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| "Failed to capture stdin from MCP process".to_string())?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout from MCP process".to_string())?;

        Ok(Self {
            server_name: server_name.to_string(),
            process: Some(process),
            stdin: Some(stdin),
            stdout: Some(BufReader::new(stdout)),
            tools: Vec::new(),
            next_id: 1,
            connected: false,
        })
    }

    /// Get the next request ID and increment
    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

// ---------------------------------------------------------------------------
// MCP Client
// ---------------------------------------------------------------------------

/// MCP Client that communicates with MCP servers via JSON-RPC 2.0
pub struct McpClient {
    connections: HashMap<String, McpConnection>,
    tool_cache: HashMap<String, McpTool>, // tool_name -> tool (for fast lookup)
    total_requests: u64,
    total_errors: u64,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            tool_cache: HashMap::new(),
            total_requests: 0,
            total_errors: 0,
        }
    }

    /// Initialize connection to an MCP server (send initialize request)
    pub fn connect(
        &mut self,
        server_name: &str,
        process: Child,
    ) -> Result<Vec<McpTool>, String> {
        if self.connections.contains_key(server_name) {
            return Err(format!("Already connected to MCP server '{}'", server_name));
        }

        let mut conn = McpConnection::new(server_name, process)?;

        // Step 1: Send "initialize" request
        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "acp-ui",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let init_response = Self::send_request_internal(&mut conn, "initialize", Some(init_params))?;

        if let Some(err) = init_response.error {
            return Err(format!("Initialize failed: {}", err));
        }

        // Parse initialize result to verify capabilities
        if let Some(result) = &init_response.result {
            let _init_result: InitializeResult = serde_json::from_value(result.clone())
                .map_err(|e| format!("Failed to parse initialize result: {}", e))?;
        }

        // Step 2: Send "initialized" notification (no id, no response expected)
        Self::send_notification_internal(&mut conn, "notifications/initialized", None)?;

        // Step 3: Discover tools via "tools/list"
        let tools_response = Self::send_request_internal(&mut conn, "tools/list", None)?;

        if let Some(err) = tools_response.error {
            return Err(format!("Tools list failed: {}", err));
        }

        let mut discovered_tools = Vec::new();

        if let Some(result) = tools_response.result {
            let tools_list: ToolsListResult = serde_json::from_value(result)
                .map_err(|e| format!("Failed to parse tools list: {}", e))?;

            for raw_tool in tools_list.tools {
                let tool = McpTool {
                    name: raw_tool.name.clone(),
                    description: raw_tool.description.unwrap_or_default(),
                    input_schema: raw_tool
                        .input_schema
                        .unwrap_or_else(|| serde_json::json!({"type": "object"})),
                    server_name: server_name.to_string(),
                };

                // Cache tool for fast lookup
                self.tool_cache.insert(tool.name.clone(), tool.clone());
                discovered_tools.push(tool);
            }
        }

        conn.tools = discovered_tools.clone();
        conn.connected = true;
        self.connections.insert(server_name.to_string(), conn);

        println!(
            "MCP Client: Connected to '{}' with {} tools",
            server_name,
            discovered_tools.len()
        );

        Ok(discovered_tools)
    }

    /// Disconnect from an MCP server
    pub fn disconnect(&mut self, server_name: &str) -> Result<(), String> {
        if let Some(mut conn) = self.connections.remove(server_name) {
            // Remove tools from cache
            for tool in &conn.tools {
                self.tool_cache.remove(&tool.name);
            }

            // Close stdin to signal shutdown
            conn.stdin.take();

            // Kill process if still running
            if let Some(mut process) = conn.process.take() {
                let _ = process.kill();
                let _ = process.wait();
            }

            conn.connected = false;
            println!("MCP Client: Disconnected from '{}'", server_name);
        }

        Ok(())
    }

    /// List all discovered tools across all connected servers
    pub fn list_tools(&self) -> Vec<&McpTool> {
        self.tool_cache.values().collect()
    }

    /// List tools for a specific server
    pub fn list_server_tools(&self, server_name: &str) -> Vec<&McpTool> {
        match self.connections.get(server_name) {
            Some(conn) => conn.tools.iter().collect(),
            None => Vec::new(),
        }
    }

    /// Call a tool on its owning MCP server
    pub fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<McpToolResult, String> {
        let start_time = std::time::Instant::now();

        // Find which server has this tool
        let server_name = self
            .tool_cache
            .get(tool_name)
            .map(|t| t.server_name.clone())
            .ok_or_else(|| format!("Tool '{}' not found in any connected server", tool_name))?;

        // Get mutable connection
        let conn = self
            .connections
            .get_mut(&server_name)
            .ok_or_else(|| format!("Server '{}' not connected", server_name))?;

        if !conn.connected {
            return Err(format!("Server '{}' is not connected", server_name));
        }

        self.total_requests += 1;

        // Send "tools/call" request
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        });

        let response = match Self::send_request_internal(conn, "tools/call", Some(params)) {
            Ok(resp) => resp,
            Err(e) => {
                self.total_errors += 1;
                return Err(e);
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Check for JSON-RPC error
        if let Some(err) = response.error {
            self.total_errors += 1;
            return Ok(McpToolResult {
                tool_name: tool_name.to_string(),
                success: false,
                content: vec![McpContent::Text {
                    text: format!("Error {}: {}", err.code, err.message),
                }],
                is_error: true,
                duration_ms,
            });
        }

        // Parse result
        let (content, is_error) = if let Some(result) = response.result {
            Self::parse_tool_result(result)
        } else {
            (Vec::new(), false)
        };

        Ok(McpToolResult {
            tool_name: tool_name.to_string(),
            success: !is_error,
            content,
            is_error,
            duration_ms,
        })
    }

    /// Parse MCP tool result into content array
    fn parse_tool_result(result: Value) -> (Vec<McpContent>, bool) {
        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let content_array = match result.get("content").and_then(|v| v.as_array()) {
            Some(arr) => arr.clone(),
            None => return (Vec::new(), is_error),
        };

        let mut contents = Vec::new();

        for item in content_array {
            let content_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("text");

            match content_type {
                "text" => {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        contents.push(McpContent::Text {
                            text: text.to_string(),
                        });
                    }
                }
                "image" => {
                    let data = item
                        .get("data")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let mime_type = item
                        .get("mimeType")
                        .and_then(|v| v.as_str())
                        .unwrap_or("image/png")
                        .to_string();
                    contents.push(McpContent::Image { data, mime_type });
                }
                "resource" => {
                    if let Some(resource) = item.get("resource") {
                        let uri = resource
                            .get("uri")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let text = resource.get("text").and_then(|v| v.as_str()).map(String::from);
                        contents.push(McpContent::Resource { uri, text });
                    }
                }
                _ => {
                    // Unknown type, treat as text
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        contents.push(McpContent::Text {
                            text: text.to_string(),
                        });
                    }
                }
            }
        }

        (contents, is_error)
    }

    /// Send raw JSON-RPC request and read response (internal, takes mutable connection)
    fn send_request_internal(
        conn: &mut McpConnection,
        method: &str,
        params: Option<Value>,
    ) -> Result<JsonRpcResponse, String> {
        let id = conn.next_request_id();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let request_json =
            serde_json::to_string(&request).map_err(|e| format!("Serialize error: {}", e))?;

        // Write JSON line to stdin
        let stdin = conn
            .stdin
            .as_mut()
            .ok_or_else(|| "No stdin available".to_string())?;

        writeln!(stdin, "{}", request_json)
            .map_err(|e| format!("Failed to write to MCP stdin: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush MCP stdin: {}", e))?;

        // Read JSON line from stdout
        let stdout = conn
            .stdout
            .as_mut()
            .ok_or_else(|| "No stdout available".to_string())?;

        let mut response_line = String::new();
        stdout
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read from MCP stdout: {}", e))?;

        if response_line.trim().is_empty() {
            return Err("Empty response from MCP server".to_string());
        }

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Failed to parse JSON-RPC response: {} - line: {}", e, response_line.trim()))?;

        // Verify response ID matches
        if response.id != id {
            return Err(format!(
                "Response ID mismatch: expected {}, got {}",
                id, response.id
            ));
        }

        Ok(response)
    }

    /// Send JSON-RPC notification (no response expected)
    fn send_notification_internal(
        conn: &mut McpConnection,
        method: &str,
        params: Option<Value>,
    ) -> Result<(), String> {
        // Notifications don't have an ID
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or(Value::Null)
        });

        let notification_json = serde_json::to_string(&notification)
            .map_err(|e| format!("Serialize error: {}", e))?;

        let stdin = conn
            .stdin
            .as_mut()
            .ok_or_else(|| "No stdin available".to_string())?;

        writeln!(stdin, "{}", notification_json)
            .map_err(|e| format!("Failed to write notification to MCP stdin: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush MCP stdin: {}", e))?;

        Ok(())
    }

    /// Check if a server is connected
    pub fn is_connected(&self, server_name: &str) -> bool {
        self.connections
            .get(server_name)
            .map(|c| c.connected)
            .unwrap_or(false)
    }

    /// Get all connected server names
    pub fn connected_servers(&self) -> Vec<&str> {
        self.connections
            .iter()
            .filter(|(_, c)| c.connected)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get statistics about the client
    pub fn get_stats(&self) -> McpClientStats {
        let total_tools = self.tool_cache.len();
        let connected = self.connections.values().filter(|c| c.connected).count();

        McpClientStats {
            connected_servers: connected as u32,
            total_tools: total_tools as u32,
            total_requests: self.total_requests,
            total_errors: self.total_errors,
        }
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP Client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientStats {
    pub connected_servers: u32,
    pub total_tools: u32,
    pub total_requests: u64,
    pub total_errors: u64,
}

// ---------------------------------------------------------------------------
// Tauri Commands
// ---------------------------------------------------------------------------

/// List all MCP tools from all connected servers
#[tauri::command]
pub fn mcp_list_tools(state: State<'_, AppState>) -> Result<Vec<McpTool>, String> {
    let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
    Ok(client.list_tools().into_iter().cloned().collect())
}

/// Call an MCP tool by name
#[tauri::command]
pub fn mcp_call_tool(
    state: State<'_, AppState>,
    tool_name: String,
    arguments: Value,
) -> Result<McpToolResult, String> {
    let mut client = state.mcp_client.lock().map_err(|e| e.to_string())?;
    client.call_tool(&tool_name, arguments)
}

/// Get list of connected MCP server names
#[tauri::command]
pub fn mcp_connected_servers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
    Ok(client
        .connected_servers()
        .into_iter()
        .map(|s| s.to_string())
        .collect())
}

/// Connect to an MCP server by spawning it as a child process and performing JSON-RPC handshake.
/// Accepts the command and optional args to spawn the MCP server process.
#[tauri::command]
pub fn mcp_connect(
    state: State<'_, AppState>,
    server_name: String,
    command: String,
    args: Option<Vec<String>>,
    env_vars: Option<HashMap<String, String>>,
) -> Result<Vec<McpTool>, String> {
    // Spawn the MCP server process with stdin/stdout piped for JSON-RPC communication
    #[cfg(desktop)]
    {
        let mut cmd = Command::new(&command);
        if let Some(ref a) = args {
            cmd.args(a);
        }
        if let Some(ref envs) = env_vars {
            for (k, v) in envs {
                cmd.env(k, v);
            }
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn MCP server '{}': {}", server_name, e))?;

        let mut client = state.mcp_client.lock().map_err(|e| e.to_string())?;
        client.connect(&server_name, child)
    }

    #[cfg(not(desktop))]
    {
        let _ = (state, server_name, command, args, env_vars);
        Err("MCP connections are only supported on desktop platforms".to_string())
    }
}

/// Disconnect from an MCP server
#[tauri::command]
pub fn mcp_disconnect(state: State<'_, AppState>, server_name: String) -> Result<(), String> {
    let mut client = state.mcp_client.lock().map_err(|e| e.to_string())?;
    client.disconnect(&server_name)
}

/// Get MCP client statistics
#[tauri::command]
pub fn mcp_get_stats(state: State<'_, AppState>) -> Result<McpClientStats, String> {
    let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
    Ok(client.get_stats())
}

/// List tools for a specific MCP server
#[tauri::command]
pub fn mcp_list_server_tools(
    state: State<'_, AppState>,
    server_name: String,
) -> Result<Vec<McpTool>, String> {
    let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
    Ok(client
        .list_server_tools(&server_name)
        .into_iter()
        .cloned()
        .collect())
}

/// Check if a specific MCP server is connected
#[tauri::command]
pub fn mcp_is_connected(state: State<'_, AppState>, server_name: String) -> Result<bool, String> {
    let client = state.mcp_client.lock().map_err(|e| e.to_string())?;
    Ok(client.is_connected(&server_name))
}
