//! MCP Client for testing MCP server connections
//!
//! Implements the MCP protocol to connect to servers, perform handshake,
//! and retrieve available tools.

use anyhow::{anyhow, Result};
use futures::StreamExt;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::timeout;

// Global request ID counter
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_request_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

// ============================================================================
// MCP Protocol Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Option<Value>,
}

// ============================================================================
// Tool Execution Types
// ============================================================================

/// Content types returned by MCP tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource {
        uri: String,
        #[serde(default)]
        mime_type: Option<String>,
        #[serde(default)]
        text: Option<String>,
    },
}

/// Result of executing a tool via MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub success: bool,
    pub content: Vec<ToolContent>,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTestResult {
    pub success: bool,
    #[serde(default)]
    pub server_info: Option<McpServerInfo>,
    #[serde(default)]
    pub tools: Vec<McpTool>,
    #[serde(default)]
    pub resources_supported: bool,
    #[serde(default)]
    pub prompts_supported: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub response_time_ms: u64,
}

impl McpTestResult {
    pub fn success(
        server_info: McpServerInfo,
        tools: Vec<McpTool>,
        resources_supported: bool,
        prompts_supported: bool,
        response_time_ms: u64,
    ) -> Self {
        Self {
            success: true,
            server_info: Some(server_info),
            tools,
            resources_supported,
            prompts_supported,
            error: None,
            response_time_ms,
        }
    }

    pub fn error(message: String, response_time_ms: u64) -> Self {
        Self {
            success: false,
            server_info: None,
            tools: vec![],
            resources_supported: false,
            prompts_supported: false,
            error: Some(message),
            response_time_ms,
        }
    }
}

// JSON-RPC types
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcNotification {
    jsonrpc: &'static str,
    method: String,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
    #[allow(dead_code)]
    data: Option<Value>,
}

// ============================================================================
// STDIO MCP Client
// ============================================================================

/// Client for communicating with stdio-based MCP servers
pub struct StdioMcpClient {
    child: Child,
    timeout: Duration,
    server_info: Option<McpServerInfo>,
    tools: Vec<McpTool>,
    resources_supported: bool,
    prompts_supported: bool,
}

impl StdioMcpClient {
    fn spawn_process(
        command: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<Self> {
        info!("[MCP Client] Spawning process: {} {:?}", command, args);

        // On Windows, run through cmd.exe to properly resolve PATH
        // On Unix, run through sh -c for the same reason
        #[cfg(windows)]
        let mut cmd = {
            let mut c = Command::new("cmd");
            // Build the full command string
            let full_command = if args.is_empty() {
                command.to_string()
            } else {
                format!("{} {}", command, args.join(" "))
            };
            c.args(["/c", &full_command]);
            c
        };

        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = Command::new("sh");
            let full_command = if args.is_empty() {
                command.to_string()
            } else {
                format!("{} {}", command, args.join(" "))
            };
            c.args(["-c", &full_command]);
            c
        };

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add environment variables
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // On Windows, prevent console window from appearing
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let child = cmd.spawn().map_err(|e| {
            anyhow!(
                "Failed to spawn MCP process '{}': {}. Make sure the command is installed and in PATH.",
                command,
                e
            )
        })?;

        Ok(Self {
            child,
            timeout: Duration::from_secs(timeout_secs),
            server_info: None,
            tools: vec![],
            resources_supported: false,
            prompts_supported: false,
        })
    }

    /// Spawn and initialize an MCP client, returning a fully connected session
    pub fn spawn(
        command: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<Self> {
        let mut client = Self::spawn_process(command, args, env, timeout_secs)?;
        client.initialize()?;
        Ok(client)
    }

    /// Perform MCP protocol handshake
    fn initialize(&mut self) -> Result<()> {
        info!("[MCP Client] Sending initialize request...");
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "claude-code-tool-manager",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let init_result = self.send_request("initialize", Some(init_params))?;

        // Parse server info and capabilities
        self.server_info = if let Some(info) = init_result.get("serverInfo") {
            Some(McpServerInfo {
                name: info
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        } else {
            None
        };

        let capabilities = init_result.get("capabilities");
        self.resources_supported = capabilities.and_then(|c| c.get("resources")).is_some();
        self.prompts_supported = capabilities.and_then(|c| c.get("prompts")).is_some();

        info!("[MCP Client] Server: {:?}", self.server_info);

        // Send initialized notification
        info!("[MCP Client] Sending initialized notification...");
        self.send_notification("initialized")?;

        // List tools
        info!("[MCP Client] Requesting tools list...");
        let tools_result = self.send_request("tools/list", Some(json!({})))?;

        self.tools = if let Some(tools_array) = tools_result.get("tools") {
            serde_json::from_value(tools_array.clone()).unwrap_or_default()
        } else {
            vec![]
        };

        info!("[MCP Client] Found {} tools", self.tools.len());

        Ok(())
    }

    /// Get server info
    pub fn server_info(&self) -> Option<&McpServerInfo> {
        self.server_info.as_ref()
    }

    /// Get available tools
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Check if resources are supported
    pub fn resources_supported(&self) -> bool {
        self.resources_supported
    }

    /// Check if prompts are supported
    pub fn prompts_supported(&self) -> bool {
        self.prompts_supported
    }

    /// Call a tool with the given arguments
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        info!(
            "[MCP Client] Calling tool: {} with args: {}",
            name, arguments
        );

        let params = json!({
            "name": name,
            "arguments": arguments
        });

        let start = Instant::now();
        let result = self.send_request("tools/call", Some(params));
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => Self::parse_tool_result(response, elapsed),
            Err(e) => Ok(ToolCallResult {
                success: false,
                content: vec![],
                is_error: true,
                error: Some(e.to_string()),
                execution_time_ms: elapsed,
            }),
        }
    }

    /// Parse the result of a tool call
    fn parse_tool_result(result: Value, elapsed: u64) -> Result<ToolCallResult> {
        // Parse content array from result
        let content: Vec<ToolContent> = result
            .get("content")
            .and_then(|c| serde_json::from_value(c.clone()).ok())
            .unwrap_or_default();

        let is_error = result
            .get("isError")
            .and_then(|e| e.as_bool())
            .unwrap_or(false);

        Ok(ToolCallResult {
            success: !is_error,
            content,
            is_error,
            error: None,
            execution_time_ms: elapsed,
        })
    }

    fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdin"))?;

        let id = next_request_id();
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };

        let request_str = serde_json::to_string(&request)?;
        info!("[MCP Client] Sending request: {}", request_str);

        writeln!(stdin, "{}", request_str)?;
        stdin.flush()?;

        self.read_response(id)
    }

    fn send_notification(&mut self, method: &str) -> Result<()> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdin"))?;

        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: method.to_string(),
        };

        let notification_str = serde_json::to_string(&notification)?;
        info!("[MCP Client] Sending notification: {}", notification_str);

        writeln!(stdin, "{}", notification_str)?;
        stdin.flush()?;

        Ok(())
    }

    fn read_response(&mut self, expected_id: u64) -> Result<Value> {
        let stdout = self
            .child
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to get stdout"))?;

        let mut reader = BufReader::new(stdout);
        let start = Instant::now();

        loop {
            if start.elapsed() > self.timeout {
                return Err(anyhow!(
                    "Timeout waiting for response ({}s)",
                    self.timeout.as_secs()
                ));
            }

            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    // EOF - check stderr for error message
                    if let Some(stderr) = self.child.stderr.as_mut() {
                        let mut stderr_reader = BufReader::new(stderr);
                        let mut stderr_content = String::new();
                        let _ = stderr_reader.read_line(&mut stderr_content);
                        if !stderr_content.is_empty() {
                            let stderr_msg = stderr_content.trim();
                            // Check for common npm authentication errors
                            if stderr_msg.contains("Access token expired")
                                || stderr_msg.contains("token revoked")
                            {
                                return Err(anyhow!(
                                    "npm authentication error: {}\n\nTry running:\n  npm logout\n  npm login\n\nOr remove the expired token:\n  npm config delete //registry.npmjs.org/:_authToken",
                                    stderr_msg
                                ));
                            }
                            return Err(anyhow!("Process stderr: {}", stderr_msg));
                        }
                    }
                    return Err(anyhow!("Process closed stdout unexpectedly"));
                }
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    info!("[MCP Client] Received: {}", line);

                    // Try to parse as JSON-RPC response
                    match serde_json::from_str::<JsonRpcResponse>(line) {
                        Ok(response) => {
                            // Check if this is a response to our request
                            if let Some(id) = response.id {
                                if id == expected_id {
                                    if let Some(error) = response.error {
                                        return Err(anyhow!("MCP error: {}", error.message));
                                    }
                                    return response
                                        .result
                                        .ok_or_else(|| anyhow!("Empty result in response"));
                                }
                            }
                            // Not our response, might be a notification - continue reading
                        }
                        Err(_) => {
                            // Not valid JSON-RPC, might be debug output - continue
                        }
                    }
                }
                Err(e) => return Err(anyhow!("Failed to read stdout: {}", e)),
            }
        }
    }

    /// Close the client and terminate the process
    pub fn close(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ============================================================================
// HTTP MCP Client (for persistent sessions)
// ============================================================================

/// Client for communicating with HTTP-based MCP servers
pub struct HttpMcpClient {
    client: reqwest::blocking::Client,
    url: String,
    session_id: Option<String>,
    headers: Option<HashMap<String, String>>,
    server_info: Option<McpServerInfo>,
    tools: Vec<McpTool>,
    resources_supported: bool,
    prompts_supported: bool,
}

impl HttpMcpClient {
    /// Connect to an HTTP MCP server and initialize the session
    pub fn connect(
        url: &str,
        headers: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<Self> {
        info!("[HTTP MCP Client] Connecting to: {}", url);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        let mut instance = Self {
            client,
            url: url.to_string(),
            session_id: None,
            headers: headers.cloned(),
            server_info: None,
            tools: vec![],
            resources_supported: false,
            prompts_supported: false,
        };

        instance.initialize()?;
        Ok(instance)
    }

    fn initialize(&mut self) -> Result<()> {
        info!("[HTTP MCP Client] Sending initialize request...");

        let init_request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "claude-code-tool-manager",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        let response = self.send_request(&init_request)?;

        // Extract session ID if provided
        if let Some(sid) = response.headers.get("mcp-session-id") {
            self.session_id = sid.to_str().ok().map(|s| s.to_string());
            info!("[HTTP MCP Client] Got session ID: {:?}", self.session_id);
        }

        // Parse server info
        if let Some(info) = response.body.get("serverInfo") {
            self.server_info = Some(McpServerInfo {
                name: info
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            });
        }

        // Parse capabilities
        if let Some(caps) = response.body.get("capabilities") {
            self.resources_supported = caps.get("resources").is_some();
            self.prompts_supported = caps.get("prompts").is_some();
        }

        info!("[HTTP MCP Client] Server: {:?}", self.server_info);

        // Send initialized notification
        let _ = self.send_notification("initialized");

        // List tools
        info!("[HTTP MCP Client] Requesting tools list...");
        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "tools/list",
            "params": {}
        });

        let tools_response = self.send_request(&tools_request)?;
        if let Some(tools_array) = tools_response.body.get("tools") {
            self.tools = serde_json::from_value(tools_array.clone()).unwrap_or_default();
        }

        info!("[HTTP MCP Client] Found {} tools", self.tools.len());
        Ok(())
    }

    /// Get server info
    pub fn server_info(&self) -> Option<&McpServerInfo> {
        self.server_info.as_ref()
    }

    /// Get available tools
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Check if resources are supported
    pub fn resources_supported(&self) -> bool {
        self.resources_supported
    }

    /// Check if prompts are supported
    pub fn prompts_supported(&self) -> bool {
        self.prompts_supported
    }

    /// Call a tool with the given arguments
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        info!(
            "[HTTP MCP Client] Calling tool: {} with args: {}",
            name, arguments
        );

        let request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let start = Instant::now();
        let result = self.send_request(&request);
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => StdioMcpClient::parse_tool_result(response.body, elapsed),
            Err(e) => Ok(ToolCallResult {
                success: false,
                content: vec![],
                is_error: true,
                error: Some(e.to_string()),
                execution_time_ms: elapsed,
            }),
        }
    }

    fn send_request(&self, request: &Value) -> Result<HttpResponse> {
        let body = serde_json::to_string(request)?;
        info!(
            "[HTTP MCP Client] Sending: {}",
            &body[..body.len().min(200)]
        );

        let mut builder = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(body);

        if let Some(sid) = &self.session_id {
            builder = builder.header("mcp-session-id", sid);
        }

        if let Some(hdrs) = &self.headers {
            for (key, value) in hdrs {
                builder = builder.header(key, value);
            }
        }

        let response = builder
            .send()
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;
        let headers = response.headers().clone();
        let status = response.status();

        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("HTTP error {}: {}", status, body));
        }

        let content_type = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let text = response.text()?;
        info!(
            "[HTTP MCP Client] Response: {}",
            &text[..text.len().min(200)]
        );

        let json_response: JsonRpcResponse = if content_type.contains("text/event-stream") {
            parse_sse_response(&text)?
        } else {
            serde_json::from_str(&text)?
        };

        if let Some(error) = json_response.error {
            return Err(anyhow!("MCP error: {}", error.message));
        }

        Ok(HttpResponse {
            headers,
            body: json_response.result.unwrap_or(Value::Null),
        })
    }

    fn send_notification(&self, method: &str) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method
        });

        let body = serde_json::to_string(&notification)?;

        let mut builder = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .body(body);

        if let Some(sid) = &self.session_id {
            builder = builder.header("mcp-session-id", sid);
        }

        if let Some(hdrs) = &self.headers {
            for (key, value) in hdrs {
                builder = builder.header(key, value);
            }
        }

        let _ = builder.send();
        Ok(())
    }

    /// Close the client (HTTP clients don't need explicit cleanup)
    pub fn close(self) {
        info!("[HTTP MCP Client] Session closed");
    }
}

struct HttpResponse {
    headers: reqwest::header::HeaderMap,
    body: Value,
}

// ============================================================================
// SSE MCP Client (for persistent sessions)
// ============================================================================

/// Client for communicating with SSE-based MCP servers
pub struct SseMcpClient {
    client: reqwest::blocking::Client,
    messages_endpoint: String,
    headers: Option<HashMap<String, String>>,
    server_info: Option<McpServerInfo>,
    tools: Vec<McpTool>,
    resources_supported: bool,
    prompts_supported: bool,
}

impl SseMcpClient {
    /// Connect to an SSE MCP server and initialize the session
    pub fn connect(
        url: &str,
        headers: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<Self> {
        info!("[SSE MCP Client] Connecting to: {}", url);

        // Use a short timeout for the initial connection since we'll read incrementally
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(timeout_secs))
            .build()?;

        // Step 1: Connect via GET to get the messages endpoint
        let mut request_builder = client.get(url).header("Accept", "text/event-stream");

        if let Some(hdrs) = headers {
            for (key, value) in hdrs {
                request_builder = request_builder.header(key, value);
            }
        }

        info!("[SSE MCP Client] Connecting to SSE endpoint via GET...");
        let response = request_builder
            .send()
            .map_err(|e| anyhow!("SSE connection failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(anyhow!("SSE connection failed with status {}", status));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/event-stream") {
            return Err(anyhow!(
                "Server did not return SSE content-type. Got: {}",
                content_type
            ));
        }

        // Read SSE stream incrementally to find the endpoint event
        // SSE connections stay open, so we can't use response.text() which would block forever
        info!("[SSE MCP Client] Reading SSE stream for endpoint event...");

        let mut messages_endpoint: Option<String> = None;
        let mut current_event = SseEvent {
            event_type: None,
            data: None,
        };
        let mut buffer = String::new();
        let start_time = Instant::now();
        let read_timeout = Duration::from_secs(10);

        // Use the response body as a reader
        use std::io::{BufRead, BufReader};
        let mut reader = BufReader::new(response);

        while start_time.elapsed() < read_timeout {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    buffer.push_str(&line);
                    let trimmed = line.trim();
                    info!("[SSE MCP Client] SSE line: {}", trimmed);

                    if let Some(event) = parse_sse_line(trimmed, &mut current_event) {
                        info!("[SSE MCP Client] Parsed event: {:?}", event);
                        if event.event_type.as_deref() == Some("endpoint") {
                            if let Some(data) = &event.data {
                                // Parse the endpoint URL - may be JSON-encoded
                                let endpoint_str = serde_json::from_str::<String>(data)
                                    .unwrap_or_else(|_| data.clone());
                                messages_endpoint = Some(endpoint_str);
                                info!("[SSE MCP Client] Found endpoint: {:?}", messages_endpoint);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    // Check if it's a timeout-like error (WouldBlock)
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    return Err(anyhow!("Error reading SSE stream: {}", e));
                }
            }
        }

        // Check for any pending event data
        if messages_endpoint.is_none() {
            if let Some(event) = parse_sse_line("", &mut current_event) {
                if event.event_type.as_deref() == Some("endpoint") {
                    if let Some(data) = &event.data {
                        let endpoint_str =
                            serde_json::from_str::<String>(data).unwrap_or_else(|_| data.clone());
                        messages_endpoint = Some(endpoint_str);
                    }
                }
            }
        }

        let endpoint_url = messages_endpoint.ok_or_else(|| {
            anyhow!(
                "SSE server did not provide a message endpoint. Buffer: {}",
                buffer
            )
        })?;

        // Build full URL for the messages endpoint
        let full_endpoint_url =
            if endpoint_url.starts_with("http://") || endpoint_url.starts_with("https://") {
                endpoint_url
            } else {
                let base_url = reqwest::Url::parse(url)?;
                base_url.join(&endpoint_url)?.to_string()
            };

        info!(
            "[SSE MCP Client] Using messages endpoint: {}",
            full_endpoint_url
        );

        // Create a new client for the session (the reader consumed the response)
        let session_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()?;

        let mut instance = Self {
            client: session_client,
            messages_endpoint: full_endpoint_url,
            headers: headers.cloned(),
            server_info: None,
            tools: vec![],
            resources_supported: false,
            prompts_supported: false,
        };

        instance.initialize()?;
        Ok(instance)
    }

    fn initialize(&mut self) -> Result<()> {
        info!("[SSE MCP Client] Sending initialize request...");

        let init_request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "claude-code-tool-manager",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        // For SSE, we POST to the messages endpoint and responses come back via SSE
        // In blocking mode, we'll use synchronous POST and read the direct response
        let response = self.send_request(&init_request)?;

        // Parse server info
        if let Some(info) = response.get("serverInfo") {
            self.server_info = Some(McpServerInfo {
                name: info
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            });
        }

        // Parse capabilities
        if let Some(caps) = response.get("capabilities") {
            self.resources_supported = caps.get("resources").is_some();
            self.prompts_supported = caps.get("prompts").is_some();
        }

        info!("[SSE MCP Client] Server: {:?}", self.server_info);

        // Send initialized notification
        let _ = self.send_notification("initialized");

        // List tools
        info!("[SSE MCP Client] Requesting tools list...");
        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "tools/list",
            "params": {}
        });

        let tools_response = self.send_request(&tools_request)?;
        if let Some(tools_array) = tools_response.get("tools") {
            self.tools = serde_json::from_value(tools_array.clone()).unwrap_or_default();
        }

        info!("[SSE MCP Client] Found {} tools", self.tools.len());
        Ok(())
    }

    /// Get server info
    pub fn server_info(&self) -> Option<&McpServerInfo> {
        self.server_info.as_ref()
    }

    /// Get available tools
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Check if resources are supported
    pub fn resources_supported(&self) -> bool {
        self.resources_supported
    }

    /// Check if prompts are supported
    pub fn prompts_supported(&self) -> bool {
        self.prompts_supported
    }

    /// Call a tool with the given arguments
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        info!(
            "[SSE MCP Client] Calling tool: {} with args: {}",
            name, arguments
        );

        let request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let start = Instant::now();
        let result = self.send_request(&request);
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => StdioMcpClient::parse_tool_result(response, elapsed),
            Err(e) => Ok(ToolCallResult {
                success: false,
                content: vec![],
                is_error: true,
                error: Some(e.to_string()),
                execution_time_ms: elapsed,
            }),
        }
    }

    fn send_request(&self, request: &Value) -> Result<Value> {
        let body = serde_json::to_string(request)?;
        info!(
            "[SSE MCP Client] Sending to {}: {}",
            self.messages_endpoint,
            &body[..body.len().min(200)]
        );

        let mut builder = self
            .client
            .post(&self.messages_endpoint)
            .header("Content-Type", "application/json");

        if let Some(hdrs) = &self.headers {
            for (key, value) in hdrs {
                builder = builder.header(key, value);
            }
        }

        let response = builder
            .body(body)
            .send()
            .map_err(|e| anyhow!("SSE POST request failed: {}", e))?;

        let status = response.status();

        // SSE servers may return 202 Accepted for async processing
        // In that case, we need to wait for the response via SSE stream
        // For simplicity in blocking mode, we'll accept both 200 and 202
        if !status.is_success() && status.as_u16() != 202 {
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("SSE POST error {}: {}", status, body));
        }

        // Read the response body
        let text = response.text()?;
        info!(
            "[SSE MCP Client] Response: {}",
            &text[..text.len().min(200)]
        );

        // Check for empty or 202-style responses (notifications)
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }

        // Check if it's just a status acknowledgment (no actual response)
        if let Ok(status_obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if status_obj.get("status").and_then(|s| s.as_str()) == Some("accepted") {
                return Ok(Value::Null);
            }
        }

        // Try to parse as JSON-RPC response
        let json_response: JsonRpcResponse =
            serde_json::from_str(&text).map_err(|e| anyhow!("Invalid JSON response: {}", e))?;

        if let Some(error) = json_response.error {
            return Err(anyhow!("MCP error: {}", error.message));
        }

        Ok(json_response.result.unwrap_or(Value::Null))
    }

    fn send_notification(&self, method: &str) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method
        });

        let body = serde_json::to_string(&notification)?;

        let mut builder = self
            .client
            .post(&self.messages_endpoint)
            .header("Content-Type", "application/json")
            .body(body);

        if let Some(hdrs) = &self.headers {
            for (key, value) in hdrs {
                builder = builder.header(key, value);
            }
        }

        let _ = builder.send();
        Ok(())
    }

    /// Close the client
    pub fn close(self) {
        info!("[SSE MCP Client] Session closed");
    }
}

// ============================================================================
// Streamable HTTP MCP Client (for system MCPs like Tool Manager and Gateway)
// ============================================================================

/// Client for communicating with Streamable HTTP-based MCP servers
/// This is used for rmcp-based servers that use SSE responses
pub struct StreamableHttpMcpClient {
    url: String,
    session_id: Option<String>,
    headers: Option<HashMap<String, String>>,
    server_info: Option<McpServerInfo>,
    tools: Vec<McpTool>,
    resources_supported: bool,
    prompts_supported: bool,
    timeout_secs: u64,
}

impl StreamableHttpMcpClient {
    /// Connect to a Streamable HTTP MCP server and initialize the session
    pub fn connect(
        url: &str,
        headers: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<Self> {
        info!("[Streamable HTTP Client] Connecting to: {}", url);

        let mut instance = Self {
            url: url.to_string(),
            session_id: None,
            headers: headers.cloned(),
            server_info: None,
            tools: vec![],
            resources_supported: false,
            prompts_supported: false,
            timeout_secs,
        };

        // Use tokio runtime to run async initialization
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create async runtime: {}", e))?;

        rt.block_on(instance.initialize_async())?;

        Ok(instance)
    }

    async fn initialize_async(&mut self) -> Result<()> {
        let client = reqwest::Client::builder().build()?;
        let timeout = Duration::from_secs(self.timeout_secs);

        // Step 1: Send initialize request
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "claude-code-tool-manager",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        let mut request_builder = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(serde_json::to_string(&init_request)?);

        if let Some(hdrs) = &self.headers {
            for (key, value) in hdrs {
                request_builder = request_builder.header(key, value);
            }
        }

        info!("[Streamable HTTP Client] Sending initialize request...");
        let response = tokio::time::timeout(timeout, request_builder.send())
            .await
            .map_err(|_| anyhow!("Connection timeout after {}s", self.timeout_secs))?
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("HTTP error {}: {}", status, body));
        }

        // Extract session ID
        self.session_id = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if let Some(ref sid) = self.session_id {
            info!("[Streamable HTTP Client] Got session ID: {}", sid);
        }

        // Read SSE response
        let init_response = Self::read_sse_response_static(response).await?;

        if let Some(error) = init_response.error {
            return Err(anyhow!("MCP initialize error: {}", error.message));
        }

        let init_result = init_response
            .result
            .ok_or_else(|| anyhow!("Empty initialize result"))?;

        // Parse server info
        self.server_info = if let Some(info) = init_result.get("serverInfo") {
            Some(McpServerInfo {
                name: info
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                version: info
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        } else {
            None
        };

        let capabilities = init_result.get("capabilities");
        self.resources_supported = capabilities.and_then(|c| c.get("resources")).is_some();
        self.prompts_supported = capabilities.and_then(|c| c.get("prompts")).is_some();

        // Step 2: Send initialized notification
        let notify_request = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let mut notify_builder = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");

        if let Some(ref sid) = self.session_id {
            notify_builder = notify_builder.header("mcp-session-id", sid);
        }

        notify_builder = notify_builder.body(serde_json::to_string(&notify_request)?);

        if let Err(e) = notify_builder.send().await {
            error!("[Streamable HTTP Client] Failed to send initialized notification: {}", e);
        }

        // Small delay to ensure notification is processed
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Step 3: List tools
        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "tools/list",
            "params": {}
        });

        let mut tools_builder = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(serde_json::to_string(&tools_request)?);

        if let Some(ref sid) = self.session_id {
            tools_builder = tools_builder.header("mcp-session-id", sid);
        }

        info!("[Streamable HTTP Client] Sending tools/list request...");
        let tools_response = tokio::time::timeout(timeout, tools_builder.send())
            .await
            .map_err(|_| anyhow!("Tools request timeout"))?
            .map_err(|e| anyhow!("Tools request failed: {}", e))?;

        if !tools_response.status().is_success() {
            let body = tools_response.text().await.unwrap_or_default();
            return Err(anyhow!("Tools request error: {}", body));
        }

        let tools_json = Self::read_sse_response_static(tools_response).await?;

        if let Some(error) = tools_json.error {
            return Err(anyhow!("MCP tools/list error: {}", error.message));
        }

        let tools_result = tools_json
            .result
            .ok_or_else(|| anyhow!("Empty tools/list result"))?;

        self.tools = if let Some(tools_array) = tools_result.get("tools") {
            serde_json::from_value::<Vec<McpTool>>(tools_array.clone()).unwrap_or_default()
        } else {
            vec![]
        };

        info!("[Streamable HTTP Client] Connected with {} tools", self.tools.len());
        Ok(())
    }

    /// Get server info
    pub fn server_info(&self) -> Option<&McpServerInfo> {
        self.server_info.as_ref()
    }

    /// Get available tools
    pub fn tools(&self) -> &[McpTool] {
        &self.tools
    }

    /// Check if resources are supported
    pub fn resources_supported(&self) -> bool {
        self.resources_supported
    }

    /// Check if prompts are supported
    pub fn prompts_supported(&self) -> bool {
        self.prompts_supported
    }

    /// Call a tool with the given arguments
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        info!(
            "[Streamable HTTP Client] Calling tool: {} with args: {}",
            name, arguments
        );

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow!("Failed to create async runtime: {}", e))?;

        rt.block_on(self.call_tool_async(name, arguments))
    }

    async fn call_tool_async(&self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        let client = reqwest::Client::builder().build()?;
        let timeout = Duration::from_secs(self.timeout_secs);

        let request = json!({
            "jsonrpc": "2.0",
            "id": next_request_id(),
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let start = Instant::now();

        let mut request_builder = client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(serde_json::to_string(&request)?);

        if let Some(ref sid) = self.session_id {
            request_builder = request_builder.header("mcp-session-id", sid);
        }

        if let Some(ref hdrs) = self.headers {
            for (key, value) in hdrs {
                request_builder = request_builder.header(key, value);
            }
        }

        let response = tokio::time::timeout(timeout, request_builder.send())
            .await
            .map_err(|_| anyhow!("Tool call timeout"))?
            .map_err(|e| anyhow!("Tool call failed: {}", e))?;

        let elapsed = start.elapsed().as_millis() as u64;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Ok(ToolCallResult {
                success: false,
                content: vec![],
                is_error: true,
                error: Some(format!("HTTP error: {}", body)),
                execution_time_ms: elapsed,
            });
        }

        let json_response = Self::read_sse_response_static(response).await;

        match json_response {
            Ok(resp) => {
                if let Some(error) = resp.error {
                    return Ok(ToolCallResult {
                        success: false,
                        content: vec![],
                        is_error: true,
                        error: Some(error.message),
                        execution_time_ms: elapsed,
                    });
                }

                let result = resp.result.unwrap_or(Value::Null);
                StdioMcpClient::parse_tool_result(result, elapsed)
            }
            Err(e) => Ok(ToolCallResult {
                success: false,
                content: vec![],
                is_error: true,
                error: Some(e.to_string()),
                execution_time_ms: elapsed,
            }),
        }
    }

    /// Static helper to read SSE response (used in async contexts)
    async fn read_sse_response_static(response: reqwest::Response) -> Result<JsonRpcResponse> {
        let body_text = response.text().await.unwrap_or_else(|e| {
            info!("[Streamable HTTP Client] Failed to read body as text: {}", e);
            String::new()
        });

        info!(
            "[Streamable HTTP Client] Response body ({} bytes): {}",
            body_text.len(),
            &body_text[..body_text.len().min(500)]
        );

        if body_text.is_empty() {
            return Err(anyhow!("Empty response body from server"));
        }

        // Check if this looks like SSE (has data: lines)
        if body_text.contains("data:") {
            for line in body_text.lines() {
                let line = line.trim();
                if line.starts_with("data:") {
                    let json_str = line.strip_prefix("data:").unwrap().trim();
                    if !json_str.is_empty() && json_str != "[DONE]" {
                        info!("[Streamable HTTP Client] Found SSE data: {}", &json_str[..json_str.len().min(200)]);
                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(json_str) {
                            return Ok(response);
                        }
                    }
                }
            }
        }

        // Try parsing the whole body as JSON directly
        serde_json::from_str(&body_text)
            .map_err(|e| anyhow!("Could not parse response: {}. Body: {}", e, &body_text[..body_text.len().min(200)]))
    }

    /// Close the client
    pub fn close(self) {
        info!("[Streamable HTTP Client] Session closed");
    }
}

// ============================================================================
// SSE Response Parsing
// ============================================================================

/// Parse an SSE (Server-Sent Events) response to extract JSON-RPC message
/// SSE format:
/// ```text
/// event: message
/// data: {"jsonrpc":"2.0","id":1,"result":{...}}
/// ```
fn parse_sse_response(sse_text: &str) -> Result<JsonRpcResponse> {
    info!("[MCP Client] Parsing SSE response...");

    // Look for data: lines and extract JSON
    for line in sse_text.lines() {
        let line = line.trim();
        if line.starts_with("data:") {
            let json_str = line.strip_prefix("data:").unwrap().trim();
            if !json_str.is_empty() && json_str != "[DONE]" {
                info!(
                    "[MCP Client] Found SSE data: {}",
                    &json_str[..json_str.len().min(200)]
                );
                match serde_json::from_str::<JsonRpcResponse>(json_str) {
                    Ok(response) => return Ok(response),
                    Err(e) => {
                        info!("[MCP Client] Failed to parse SSE data as JSON-RPC: {}", e);
                        continue;
                    }
                }
            }
        }
    }

    // If no data: line found, try parsing the whole thing as JSON
    // (some servers might not use proper SSE format)
    serde_json::from_str(sse_text)
        .map_err(|e| anyhow!("Could not parse SSE response. No valid JSON-RPC message found in data: lines. Parse error: {}", e))
}

// ============================================================================
// Public API
// ============================================================================

/// Test a stdio-based MCP server
pub fn test_stdio_mcp(
    command: &str,
    args: &[String],
    env: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> McpTestResult {
    let start = Instant::now();

    let result = test_stdio_mcp_internal(command, args, env, timeout_secs);

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((server_info, tools, resources, prompts)) => {
            info!(
                "[MCP Client] Test successful: {} tools found in {}ms",
                tools.len(),
                elapsed_ms
            );
            McpTestResult::success(server_info, tools, resources, prompts, elapsed_ms)
        }
        Err(e) => {
            error!("[MCP Client] Test failed: {}", e);
            McpTestResult::error(e.to_string(), elapsed_ms)
        }
    }
}

fn test_stdio_mcp_internal(
    command: &str,
    args: &[String],
    env: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> Result<(McpServerInfo, Vec<McpTool>, bool, bool)> {
    // Use the new spawn method which handles initialize + tools/list
    let client = StdioMcpClient::spawn(command, args, env, timeout_secs)?;

    let server_info = client
        .server_info()
        .cloned()
        .unwrap_or_else(|| McpServerInfo {
            name: "unknown".to_string(),
            version: None,
        });
    let tools = client.tools().to_vec();
    let resources_supported = client.resources_supported();
    let prompts_supported = client.prompts_supported();

    // Clean up
    client.close();

    Ok((server_info, tools, resources_supported, prompts_supported))
}

/// Test an SSE-based MCP server (async version)
/// SSE transport works differently:
/// 1. Client connects via GET to SSE endpoint
/// 2. Server sends 'endpoint' event with POST URL for messages
/// 3. Client sends JSON-RPC via POST to that endpoint
/// 4. Responses come back via SSE events
pub async fn test_sse_mcp_async(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> McpTestResult {
    let start = Instant::now();

    let result = test_sse_mcp_internal_async(url, headers, timeout_secs).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((server_info, tools, resources, prompts)) => {
            info!(
                "[MCP Client] SSE test successful: {} tools found in {}ms",
                tools.len(),
                elapsed_ms
            );
            McpTestResult::success(server_info, tools, resources, prompts, elapsed_ms)
        }
        Err(e) => {
            error!("[MCP Client] SSE test failed: {}", e);
            McpTestResult::error(e.to_string(), elapsed_ms)
        }
    }
}

/// Synchronous wrapper for async SSE test (for use from sync contexts)
pub fn test_sse_mcp(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> McpTestResult {
    // Create a new tokio runtime for the async SSE test
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            return McpTestResult::error(format!("Failed to create async runtime: {}", e), 0);
        }
    };

    rt.block_on(test_sse_mcp_async(url, headers, timeout_secs))
}

/// SSE Event parsed from the stream
#[derive(Debug)]
struct SseEvent {
    event_type: Option<String>,
    data: Option<String>,
}

/// Parse SSE events from a line buffer
fn parse_sse_line(line: &str, current_event: &mut SseEvent) -> Option<SseEvent> {
    let line = line.trim();

    if line.is_empty() {
        // Empty line means end of event
        if current_event.data.is_some() || current_event.event_type.is_some() {
            let event = SseEvent {
                event_type: current_event.event_type.take(),
                data: current_event.data.take(),
            };
            return Some(event);
        }
        return None;
    }

    if line.starts_with("event:") {
        current_event.event_type = Some(line.strip_prefix("event:").unwrap().trim().to_string());
    } else if line.starts_with("data:") {
        let data = line.strip_prefix("data:").unwrap().trim();
        current_event.data = Some(data.to_string());
    }
    // Ignore other fields like id:, retry:, comments (:)

    None
}

async fn test_sse_mcp_internal_async(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> Result<(McpServerInfo, Vec<McpTool>, bool, bool)> {
    info!("[MCP Client] Testing SSE MCP at: {} (async)", url);

    let client = reqwest::Client::builder().build()?;

    // SSE uses GET to establish connection
    let mut request_builder = client.get(url).header("Accept", "text/event-stream");

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            request_builder = request_builder.header(key, value);
        }
    }

    info!("[MCP Client] Connecting to SSE endpoint via GET...");
    let response = timeout(Duration::from_secs(timeout_secs), request_builder.send())
        .await
        .map_err(|_| anyhow!("Connection timeout after {}s", timeout_secs))?
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("dns error") || err_str.contains("resolve") {
                anyhow!("Cannot resolve host. Check that the URL is correct.")
            } else if err_str.contains("connection refused") {
                anyhow!("Connection refused. The server may not be running.")
            } else {
                anyhow!("SSE connection failed: {}", err_str)
            }
        })?;

    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    info!(
        "[MCP Client] SSE response status: {}, content-type: {}",
        status, content_type
    );

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!(
            "SSE connection failed with status {}: {}",
            status,
            if body.is_empty() {
                status.canonical_reason().unwrap_or("Unknown").to_string()
            } else {
                body[..body.len().min(200)].to_string()
            }
        ));
    }

    if !content_type.contains("text/event-stream") {
        return Err(anyhow!(
            "Server did not return SSE content-type. Got: {}. This endpoint may not support SSE transport.",
            content_type
        ));
    }

    // Stream SSE events to find the endpoint URL
    info!("[MCP Client] SSE connection established, reading events...");

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut current_event = SseEvent {
        event_type: None,
        data: None,
    };
    let mut messages_endpoint: Option<String> = None;
    let mut _session_id: Option<String> = None;

    // Read events until we find the endpoint, with a timeout
    let read_timeout = Duration::from_secs(5);
    let start = Instant::now();

    while start.elapsed() < read_timeout {
        let chunk_result = timeout(Duration::from_millis(500), stream.next()).await;

        match chunk_result {
            Ok(Some(Ok(bytes))) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                // Process complete lines
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if let Some(event) = parse_sse_line(&line, &mut current_event) {
                        info!("[MCP Client] SSE event: {:?}", event);

                        if event.event_type.as_deref() == Some("endpoint") {
                            if let Some(data) = &event.data {
                                // Parse the endpoint URL - it may be JSON-encoded (with quotes)
                                // Try to parse as JSON string first, fall back to raw value
                                let endpoint_str = serde_json::from_str::<String>(data)
                                    .unwrap_or_else(|_| data.clone());

                                messages_endpoint = Some(endpoint_str.clone());
                                if endpoint_str.contains("sessionId=") {
                                    // Extract session ID from query string
                                    if let Some(sid) = endpoint_str.split("sessionId=").nth(1) {
                                        _session_id =
                                            Some(sid.split('&').next().unwrap_or(sid).to_string());
                                    }
                                }
                                info!("[MCP Client] Found messages endpoint: {}", endpoint_str);
                            }
                        }
                    }
                }

                // If we found the endpoint, we can proceed
                if messages_endpoint.is_some() {
                    break;
                }
            }
            Ok(Some(Err(e))) => {
                return Err(anyhow!("Error reading SSE stream: {}", e));
            }
            Ok(None) => {
                // Stream ended
                break;
            }
            Err(_) => {
                // Timeout on this chunk, continue if we haven't found endpoint yet
                if messages_endpoint.is_some() {
                    break;
                }
            }
        }
    }

    let endpoint_url = messages_endpoint.ok_or_else(|| {
        anyhow!("SSE server did not provide a message endpoint. The 'endpoint' event is required.")
    })?;

    // Build full URL for the messages endpoint
    // If endpoint_url is already absolute, use it directly; otherwise join with base
    let full_endpoint_url =
        if endpoint_url.starts_with("http://") || endpoint_url.starts_with("https://") {
            reqwest::Url::parse(&endpoint_url)?
        } else {
            let base_url = reqwest::Url::parse(url)?;
            base_url.join(&endpoint_url)?
        };
    info!(
        "[MCP Client] Using messages endpoint: {}",
        full_endpoint_url
    );

    // Now we need to send initialize via POST and read response from SSE stream
    // Create a channel to receive SSE events
    let (tx, mut rx) = mpsc::channel::<SseEvent>(32);

    // Spawn a task to continue reading SSE events
    let stream_handle = {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut current_event = SseEvent {
                event_type: None,
                data: None,
            };
            let mut buffer = buffer; // Continue with remaining buffer

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            if let Some(event) = parse_sse_line(&line, &mut current_event) {
                                if tx.send(event).await.is_err() {
                                    return; // Receiver dropped
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        })
    };

    // Send initialize request
    let init_id = next_request_id();
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": init_id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "claude-code-tool-manager",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    info!(
        "[MCP Client] Sending initialize via POST to {}",
        full_endpoint_url
    );
    let init_body = serde_json::to_string(&init_request)?;

    let mut post_builder = client
        .post(full_endpoint_url.as_str())
        .header("Content-Type", "application/json")
        .body(init_body);

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            post_builder = post_builder.header(key, value);
        }
    }

    let _init_response = post_builder
        .send()
        .await
        .map_err(|e| anyhow!("Failed to send initialize: {}", e))?;

    // Wait for initialize response from SSE stream
    let mut server_info = McpServerInfo {
        name: "SSE Server".to_string(),
        version: None,
    };
    let mut resources_supported = false;
    let mut prompts_supported = false;

    let init_timeout = Duration::from_secs(10);
    let init_start = Instant::now();

    while init_start.elapsed() < init_timeout {
        match timeout(Duration::from_secs(1), rx.recv()).await {
            Ok(Some(event)) => {
                if event.event_type.as_deref() == Some("message") {
                    if let Some(data) = event.data {
                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&data) {
                            if response.id == Some(init_id) {
                                info!("[MCP Client] Received initialize response");
                                if let Some(error) = response.error {
                                    return Err(anyhow!("Initialize error: {}", error.message));
                                }
                                if let Some(result) = response.result {
                                    if let Some(info) = result.get("serverInfo") {
                                        server_info = McpServerInfo {
                                            name: info
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("SSE Server")
                                                .to_string(),
                                            version: info
                                                .get("version")
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                        };
                                    }
                                    if let Some(caps) = result.get("capabilities") {
                                        resources_supported = caps.get("resources").is_some();
                                        prompts_supported = caps.get("prompts").is_some();
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
            Ok(None) => break,  // Channel closed
            Err(_) => continue, // Timeout, keep waiting
        }
    }

    // Send initialized notification
    let initialized_request = json!({
        "jsonrpc": "2.0",
        "method": "initialized"
    });

    let mut notify_builder = client
        .post(full_endpoint_url.as_str())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&initialized_request)?);

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            notify_builder = notify_builder.header(key, value);
        }
    }

    let _ = notify_builder.send().await;

    // Send tools/list request
    let tools_id = next_request_id();
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": tools_id,
        "method": "tools/list",
        "params": {}
    });

    info!("[MCP Client] Sending tools/list via POST");
    let mut tools_builder = client
        .post(full_endpoint_url.as_str())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&tools_request)?);

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            tools_builder = tools_builder.header(key, value);
        }
    }

    let _ = tools_builder.send().await;

    // Wait for tools/list response
    let mut tools: Vec<McpTool> = vec![];
    let tools_timeout = Duration::from_secs(10);
    let tools_start = Instant::now();

    while tools_start.elapsed() < tools_timeout {
        match timeout(Duration::from_secs(1), rx.recv()).await {
            Ok(Some(event)) => {
                if event.event_type.as_deref() == Some("message") {
                    if let Some(data) = event.data {
                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&data) {
                            if response.id == Some(tools_id) {
                                info!("[MCP Client] Received tools/list response");
                                if let Some(error) = response.error {
                                    return Err(anyhow!("tools/list error: {}", error.message));
                                }
                                if let Some(result) = response.result {
                                    if let Some(tools_array) = result.get("tools") {
                                        tools = serde_json::from_value(tools_array.clone())
                                            .unwrap_or_default();
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    // Clean up
    stream_handle.abort();
    drop(rx);

    info!(
        "[MCP Client] SSE test complete: {} tools found",
        tools.len()
    );

    Ok((server_info, tools, resources_supported, prompts_supported))
}

/// Test an HTTP-based MCP server (Streamable HTTP transport)
pub fn test_http_mcp(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> McpTestResult {
    let start = Instant::now();

    let result = test_http_mcp_internal(url, headers, timeout_secs);

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((server_info, tools, resources, prompts)) => {
            info!(
                "[MCP Client] HTTP test successful: {} tools found in {}ms",
                tools.len(),
                elapsed_ms
            );
            McpTestResult::success(server_info, tools, resources, prompts, elapsed_ms)
        }
        Err(e) => {
            error!("[MCP Client] HTTP test failed: {}", e);
            McpTestResult::error(e.to_string(), elapsed_ms)
        }
    }
}

/// Test a Streamable HTTP MCP server (async version with proper SSE handling)
/// Streamable HTTP uses POST requests with SSE responses
pub async fn test_streamable_http_mcp_async(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> McpTestResult {
    let start = Instant::now();

    let result = test_streamable_http_internal_async(url, headers, timeout_secs).await;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok((server_info, tools, resources, prompts)) => {
            info!(
                "[MCP Client] Streamable HTTP test successful: {} tools found in {}ms",
                tools.len(),
                elapsed_ms
            );
            McpTestResult::success(server_info, tools, resources, prompts, elapsed_ms)
        }
        Err(e) => {
            error!("[MCP Client] Streamable HTTP test failed: {}", e);
            McpTestResult::error(e.to_string(), elapsed_ms)
        }
    }
}

/// Synchronous wrapper for async Streamable HTTP test
pub fn test_streamable_http_mcp(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> McpTestResult {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            return McpTestResult::error(format!("Failed to create async runtime: {}", e), 0);
        }
    };

    rt.block_on(test_streamable_http_mcp_async(url, headers, timeout_secs))
}

/// Internal async implementation for Streamable HTTP
async fn test_streamable_http_internal_async(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> Result<(McpServerInfo, Vec<McpTool>, bool, bool)> {
    info!("[MCP Client] Testing Streamable HTTP MCP at: {}", url);

    let client = reqwest::Client::builder().build()?;

    // Step 1: Send initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": next_request_id(),
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "claude-code-tool-manager",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    let mut request_builder = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(serde_json::to_string(&init_request)?);

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            request_builder = request_builder.header(key, value);
        }
    }

    info!("[MCP Client] Sending Streamable HTTP initialize request...");
    let response = timeout(Duration::from_secs(timeout_secs), request_builder.send())
        .await
        .map_err(|_| anyhow!("Connection timeout after {}s", timeout_secs))?
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow!("HTTP error {}: {}", status, body));
    }

    // Extract session ID from headers
    let session_id: Option<String> = response
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(ref sid) = session_id {
        info!("[MCP Client] Got session ID: {}", sid);
    }

    // Read SSE response for initialize
    let init_response = read_sse_response(response).await?;
    info!("[MCP Client] Initialize response received");

    if let Some(error) = init_response.error {
        return Err(anyhow!("MCP initialize error: {}", error.message));
    }

    let init_result = init_response
        .result
        .ok_or_else(|| anyhow!("Empty initialize result"))?;

    // Parse server info
    let server_info = if let Some(info) = init_result.get("serverInfo") {
        McpServerInfo {
            name: info
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: info
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    } else {
        McpServerInfo {
            name: "unknown".to_string(),
            version: None,
        }
    };

    let capabilities = init_result.get("capabilities");
    let resources_supported = capabilities.and_then(|c| c.get("resources")).is_some();
    let prompts_supported = capabilities.and_then(|c| c.get("prompts")).is_some();

    info!("[MCP Client] Session ID for notifications: {:?}", session_id);

    // Step 2: Send initialized notification
    // Note: rmcp expects "notifications/initialized" with no id (notification, not request)
    let notify_request = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });

    let mut notify_builder = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream");

    if let Some(ref sid) = session_id {
        info!("[MCP Client] Adding session ID to notification: {}", sid);
        notify_builder = notify_builder.header("mcp-session-id", sid);
    } else {
        info!("[MCP Client] WARNING: No session ID for notification!");
    }

    notify_builder = notify_builder.body(serde_json::to_string(&notify_request)?);

    // Send notification and wait for it to complete (don't ignore errors)
    match notify_builder.send().await {
        Ok(resp) => {
            let notify_status = resp.status();
            info!("[MCP Client] Initialized notification response status: {}", notify_status);
            // Read and log the response body
            if let Ok(body) = resp.text().await {
                if !body.is_empty() {
                    info!("[MCP Client] Notification response: {}", &body[..body.len().min(200)]);
                }
            }
        }
        Err(e) => {
            error!("[MCP Client] Failed to send initialized notification: {}", e);
        }
    }

    // Small delay to ensure notification is processed
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Step 3: List tools
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": next_request_id(),
        "method": "tools/list",
        "params": {}
    });

    let mut tools_builder = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(serde_json::to_string(&tools_request)?);

    if let Some(ref sid) = session_id {
        tools_builder = tools_builder.header("mcp-session-id", sid);
    }

    info!("[MCP Client] Sending tools/list request...");
    let tools_response = timeout(Duration::from_secs(timeout_secs), tools_builder.send())
        .await
        .map_err(|_| anyhow!("Tools request timeout"))?
        .map_err(|e| anyhow!("Tools request failed: {}", e))?;

    if !tools_response.status().is_success() {
        let body = tools_response.text().await.unwrap_or_default();
        return Err(anyhow!("Tools request error: {}", body));
    }

    let tools_json = read_sse_response(tools_response).await?;

    if let Some(error) = tools_json.error {
        return Err(anyhow!("MCP tools/list error: {}", error.message));
    }

    let tools_result = tools_json
        .result
        .ok_or_else(|| anyhow!("Empty tools/list result"))?;

    let tools: Vec<McpTool> = if let Some(tools_array) = tools_result.get("tools") {
        serde_json::from_value::<Vec<McpTool>>(tools_array.clone()).unwrap_or_default()
    } else {
        vec![]
    };

    info!(
        "[MCP Client] Streamable HTTP test complete: {} tools found",
        tools.len()
    );

    Ok((server_info, tools, resources_supported, prompts_supported))
}

/// Read SSE response from a Streamable HTTP response
async fn read_sse_response(response: reqwest::Response) -> Result<JsonRpcResponse> {
    let status = response.status();
    let headers = response.headers().clone();

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let content_length = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    info!(
        "[MCP Client] Response - status: {}, content-type: {}, content-length: {}",
        status, content_type, content_length
    );

    // Log all headers for debugging
    for (name, value) in headers.iter() {
        info!("[MCP Client] Header: {} = {:?}", name, value);
    }

    // First, try reading the entire body as text regardless of content-type
    // This helps us see what the server is actually returning
    let body_text = response.text().await.unwrap_or_else(|e| {
        info!("[MCP Client] Failed to read body as text: {}", e);
        String::new()
    });

    info!(
        "[MCP Client] Response body ({} bytes): {}",
        body_text.len(),
        &body_text[..body_text.len().min(1000)]
    );

    if body_text.is_empty() {
        return Err(anyhow!("Empty response body from server"));
    }

    // Check if this looks like SSE (has data: lines)
    if body_text.contains("data:") {
        // Parse SSE data lines
        for line in body_text.lines() {
            let line = line.trim();
            if line.starts_with("data:") {
                let json_str = line.strip_prefix("data:").unwrap().trim();
                if !json_str.is_empty() && json_str != "[DONE]" {
                    info!("[MCP Client] Found SSE data: {}", &json_str[..json_str.len().min(200)]);
                    if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(json_str) {
                        return Ok(response);
                    }
                }
            }
        }
    }

    // Try parsing the whole body as JSON directly
    serde_json::from_str(&body_text)
        .map_err(|e| anyhow!("Could not parse response: {}. Body: {}", e, &body_text[..body_text.len().min(200)]))
}

/// Helper to build an HTTP request with common headers
fn build_http_request(
    client: &reqwest::blocking::Client,
    url: &str,
    body: String,
    session_id: Option<&str>,
    custom_headers: Option<&HashMap<String, String>>,
) -> reqwest::blocking::RequestBuilder {
    let mut builder = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(body);

    // Add session ID if we have one
    if let Some(sid) = session_id {
        builder = builder.header("mcp-session-id", sid);
    }

    // Add custom headers
    if let Some(hdrs) = custom_headers {
        for (key, value) in hdrs {
            builder = builder.header(key, value);
        }
    }

    builder
}

fn test_http_mcp_internal(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> Result<(McpServerInfo, Vec<McpTool>, bool, bool)> {
    info!("[MCP Client] Testing HTTP MCP at: {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    // Step 1: Initialize
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": next_request_id(),
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "claude-code-tool-manager",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    let request_body = serde_json::to_string(&init_request)?;
    info!(
        "[MCP Client] Sending HTTP initialize request: {}",
        request_body
    );

    let response = build_http_request(&client, url, request_body, None, headers)
        .send()
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("dns error")
                || err_str.contains("resolve")
                || err_str.contains("No such host")
            {
                anyhow!(
                    "Cannot resolve host. Check that the URL is correct and the server is online."
                )
            } else if err_str.contains("connection refused") {
                anyhow!(
                    "Connection refused. The server may not be running or the port is incorrect."
                )
            } else if err_str.contains("timed out") || err_str.contains("timeout") {
                anyhow!("Connection timed out. The server may be slow or unreachable.")
            } else if err_str.contains("certificate")
                || err_str.contains("SSL")
                || err_str.contains("TLS")
            {
                anyhow!(
                    "SSL/TLS error: {}. The server may have an invalid certificate.",
                    err_str
                )
            } else {
                anyhow!("HTTP request failed: {}", err_str)
            }
        })?;

    let status = response.status();

    // Extract session ID from response headers BEFORE consuming body
    let session_id: Option<String> = response
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(ref sid) = session_id {
        info!("[MCP Client] Got session ID: {}", sid);
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    info!(
        "[MCP Client] Response status: {}, content-type: {}",
        status, content_type
    );

    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(anyhow!(
            "HTTP error {}: {}",
            status,
            if body.is_empty() {
                status
                    .canonical_reason()
                    .unwrap_or("Unknown error")
                    .to_string()
            } else {
                body
            }
        ));
    }

    let response_text = response
        .text()
        .map_err(|e| anyhow!("Failed to read response: {}", e))?;
    info!(
        "[MCP Client] Response body: {}",
        &response_text[..response_text.len().min(500)]
    );

    // Parse response - handle both JSON and SSE formats
    let init_response: JsonRpcResponse = if content_type.contains("text/event-stream") {
        // Parse SSE format: extract JSON from "data:" lines
        parse_sse_response(&response_text)?
    } else {
        serde_json::from_str(&response_text).map_err(|e| {
            anyhow!(
                "Invalid JSON response: {}. Response was: {}",
                e,
                &response_text[..response_text.len().min(200)]
            )
        })?
    };

    if let Some(error) = init_response.error {
        return Err(anyhow!("MCP initialize error: {}", error.message));
    }

    let init_result = init_response
        .result
        .ok_or_else(|| anyhow!("Empty initialize result"))?;

    // Parse server info
    let server_info = if let Some(info) = init_result.get("serverInfo") {
        McpServerInfo {
            name: info
                .get("name")
                .and_then(|v: &Value| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: info
                .get("version")
                .and_then(|v: &Value| v.as_str())
                .map(|s: &str| s.to_string()),
        }
    } else {
        McpServerInfo {
            name: "unknown".to_string(),
            version: None,
        }
    };

    let capabilities = init_result.get("capabilities");
    let resources_supported = capabilities
        .and_then(|c: &Value| c.get("resources"))
        .is_some();
    let prompts_supported = capabilities
        .and_then(|c: &Value| c.get("prompts"))
        .is_some();

    // Step 2: Send initialized notification (some servers may require this)
    let notify_body = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": "initialized"
    }))?;
    info!(
        "[MCP Client] Sending initialized notification with session: {:?}",
        session_id
    );

    let _ = build_http_request(&client, url, notify_body, session_id.as_deref(), headers).send();

    // Step 3: List tools
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": next_request_id(),
        "method": "tools/list",
        "params": {}
    });
    let tools_body = serde_json::to_string(&tools_request)?;
    info!(
        "[MCP Client] Sending HTTP tools/list request with session: {:?}",
        session_id
    );

    let tools_response =
        build_http_request(&client, url, tools_body, session_id.as_deref(), headers)
            .send()
            .map_err(|e| anyhow!("HTTP tools/list request failed: {}", e))?;

    let tools_content_type = tools_response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let tools_text = tools_response
        .text()
        .map_err(|e| anyhow!("Failed to read tools response: {}", e))?;
    info!(
        "[MCP Client] Tools response: {}",
        &tools_text[..tools_text.len().min(500)]
    );

    let tools_json: JsonRpcResponse = if tools_content_type.contains("text/event-stream") {
        parse_sse_response(&tools_text)?
    } else {
        serde_json::from_str(&tools_text)
            .map_err(|e| anyhow!("Invalid JSON in tools response: {}", e))?
    };

    if let Some(error) = tools_json.error {
        return Err(anyhow!("MCP tools/list error: {}", error.message));
    }

    let tools_result = tools_json
        .result
        .ok_or_else(|| anyhow!("Empty tools/list result"))?;

    let tools: Vec<McpTool> = if let Some(tools_array) = tools_result.get("tools") {
        serde_json::from_value::<Vec<McpTool>>(tools_array.clone()).unwrap_or_default()
    } else {
        vec![]
    };

    Ok((server_info, tools, resources_supported, prompts_supported))
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // McpTestResult tests
    // =========================================================================

    #[test]
    fn test_mcp_test_result_success() {
        let result = McpTestResult::success(
            McpServerInfo {
                name: "test".to_string(),
                version: Some("1.0.0".to_string()),
            },
            vec![],
            true,
            false,
            100,
        );
        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.server_info.is_some());
        assert!(result.resources_supported);
        assert!(!result.prompts_supported);
        assert_eq!(result.response_time_ms, 100);
    }

    #[test]
    fn test_mcp_test_result_success_with_tools() {
        let tools = vec![
            McpTool {
                name: "read_file".to_string(),
                description: Some("Read a file".to_string()),
                input_schema: None,
            },
            McpTool {
                name: "write_file".to_string(),
                description: Some("Write a file".to_string()),
                input_schema: Some(json!({"type": "object"})),
            },
        ];

        let result = McpTestResult::success(
            McpServerInfo {
                name: "filesystem".to_string(),
                version: Some("1.0.0".to_string()),
            },
            tools,
            true,
            true,
            250,
        );

        assert_eq!(result.tools.len(), 2);
        assert_eq!(result.tools[0].name, "read_file");
        assert!(result.resources_supported);
        assert!(result.prompts_supported);
    }

    #[test]
    fn test_mcp_test_result_error() {
        let result = McpTestResult::error("Test error".to_string(), 50);
        assert!(!result.success);
        assert_eq!(result.error, Some("Test error".to_string()));
        assert!(result.server_info.is_none());
        assert!(result.tools.is_empty());
        assert!(!result.resources_supported);
        assert!(!result.prompts_supported);
    }

    #[test]
    fn test_mcp_test_result_error_preserves_time() {
        let result = McpTestResult::error("Timeout".to_string(), 30000);
        assert_eq!(result.response_time_ms, 30000);
    }

    // =========================================================================
    // McpServerInfo tests
    // =========================================================================

    #[test]
    fn test_mcp_server_info_deserialization() {
        let json = r#"{"name": "test-server", "version": "2.0.0"}"#;
        let info: McpServerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "test-server");
        assert_eq!(info.version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_mcp_server_info_deserialization_no_version() {
        let json = r#"{"name": "minimal-server"}"#;
        let info: McpServerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "minimal-server");
        assert!(info.version.is_none());
    }

    // =========================================================================
    // McpTool tests
    // =========================================================================

    #[test]
    fn test_mcp_tool_deserialization_full() {
        let json = r#"{
            "name": "read_file",
            "description": "Read file contents",
            "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}
        }"#;
        let tool: McpTool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "read_file");
        assert_eq!(tool.description, Some("Read file contents".to_string()));
        assert!(tool.input_schema.is_some());
    }

    #[test]
    fn test_mcp_tool_deserialization_minimal() {
        let json = r#"{"name": "simple_tool"}"#;
        let tool: McpTool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "simple_tool");
        assert!(tool.description.is_none());
        assert!(tool.input_schema.is_none());
    }

    #[test]
    fn test_mcp_tool_serialization() {
        let tool = McpTool {
            name: "test".to_string(),
            description: Some("A test tool".to_string()),
            input_schema: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"description\":\"A test tool\""));
    }

    // =========================================================================
    // parse_sse_response tests
    // =========================================================================

    #[test]
    fn test_parse_sse_response_valid() {
        let sse_text = r#"event: message
data: {"jsonrpc":"2.0","id":1,"result":{"success":true}}

"#;
        let result = parse_sse_response(sse_text);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.result.is_some());
    }

    #[test]
    fn test_parse_sse_response_multiple_data_lines() {
        let sse_text = r#"data: invalid json
data: {"jsonrpc":"2.0","id":1,"result":{}}
"#;
        // Should skip invalid and find valid
        let result = parse_sse_response(sse_text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sse_response_data_with_whitespace() {
        let sse_text = "data:   {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}  \n";
        let result = parse_sse_response(sse_text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sse_response_done_marker_ignored() {
        let sse_text = r#"data: [DONE]
data: {"jsonrpc":"2.0","id":1,"result":{}}
"#;
        let result = parse_sse_response(sse_text);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sse_response_fallback_to_raw_json() {
        // No data: prefix, but valid JSON
        let raw_json = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        let result = parse_sse_response(raw_json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sse_response_invalid() {
        let invalid = "not valid at all";
        let result = parse_sse_response(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sse_response_with_error() {
        let sse_text =
            r#"data: {"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid request"}}"#;
        let result = parse_sse_response(sse_text).unwrap();
        assert!(result.error.is_some());
        assert_eq!(result.error.unwrap().message, "Invalid request");
    }

    // =========================================================================
    // next_request_id tests
    // =========================================================================

    #[test]
    fn test_next_request_id_increments() {
        let id1 = next_request_id();
        let id2 = next_request_id();
        let id3 = next_request_id();

        // IDs should be strictly increasing
        assert!(id2 > id1);
        assert!(id3 > id2);
    }

    // =========================================================================
    // JSON-RPC types serialization tests
    // =========================================================================

    #[test]
    fn test_json_rpc_response_with_result() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"tools":[]}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_with_error() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 42,
            method: "initialize".to_string(),
            params: Some(json!({"test": true})),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"method\":\"initialize\""));
        assert!(json.contains("\"params\":{\"test\":true}"));
    }

    #[test]
    fn test_json_rpc_request_without_params() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "test".to_string(),
            params: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        // params should be omitted when None
        assert!(!json.contains("params"));
    }

    #[test]
    fn test_json_rpc_notification_serialization() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: "notifications/initialized".to_string(),
        };
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("\"method\":\"notifications/initialized\""));
        // Notifications don't have an id field
        assert!(!json.contains("\"id\""));
    }

    // =========================================================================
    // McpTestResult serialization tests
    // =========================================================================

    #[test]
    fn test_mcp_test_result_serialization() {
        let result = McpTestResult::success(
            McpServerInfo {
                name: "test".to_string(),
                version: Some("1.0".to_string()),
            },
            vec![McpTool {
                name: "tool1".to_string(),
                description: None,
                input_schema: None,
            }],
            true,
            false,
            150,
        );

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"responseTimeMs\":150"));
        assert!(json.contains("\"resourcesSupported\":true"));
    }

    #[test]
    fn test_mcp_test_result_deserialization() {
        let json = r#"{
            "success": true,
            "serverInfo": {"name": "test", "version": "1.0"},
            "tools": [],
            "resourcesSupported": true,
            "promptsSupported": false,
            "responseTimeMs": 100
        }"#;

        let result: McpTestResult = serde_json::from_str(json).unwrap();
        assert!(result.success);
        assert_eq!(result.server_info.unwrap().name, "test");
        assert!(result.tools.is_empty());
    }

    // =========================================================================
    // Additional edge case tests
    // =========================================================================

    #[test]
    fn test_mcp_tool_with_complex_schema() {
        let json = r#"{
            "name": "complex_tool",
            "description": "A tool with complex schema",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "options": {
                        "type": "object",
                        "properties": {
                            "recursive": {"type": "boolean"},
                            "maxDepth": {"type": "integer"}
                        }
                    }
                },
                "required": ["path"]
            }
        }"#;

        let tool: McpTool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "complex_tool");
        let schema = tool.input_schema.unwrap();
        assert!(schema.get("properties").is_some());
        assert!(schema.get("required").is_some());
    }

    #[test]
    fn test_json_rpc_response_with_null_id() {
        // Notifications may have null or missing id
        let json = r#"{"jsonrpc":"2.0","id":null,"result":{}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.id.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_json_rpc_error_with_data() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"Invalid params","data":{"expected":"string","got":"number"}}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid params");
        assert!(error.data.is_some());
    }

    #[test]
    fn test_mcp_test_result_round_trip() {
        let original = McpTestResult::success(
            McpServerInfo {
                name: "round-trip".to_string(),
                version: Some("2.5.0".to_string()),
            },
            vec![
                McpTool {
                    name: "tool1".to_string(),
                    description: Some("First tool".to_string()),
                    input_schema: Some(json!({"type": "object"})),
                },
                McpTool {
                    name: "tool2".to_string(),
                    description: None,
                    input_schema: None,
                },
            ],
            true,
            true,
            999,
        );

        let json = serde_json::to_string(&original).unwrap();
        let parsed: McpTestResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.success, original.success);
        assert_eq!(parsed.tools.len(), 2);
        assert_eq!(parsed.response_time_ms, 999);
    }

    #[test]
    fn test_parse_sse_response_empty_result() {
        let sse_text = r#"data: {"jsonrpc":"2.0","id":1,"result":null}"#;
        let result = parse_sse_response(sse_text).unwrap();
        assert!(result.result.is_none() || result.result.unwrap().is_null());
    }

    #[test]
    fn test_parse_sse_response_with_event_prefix() {
        let sse_text = r#"event: message
id: 123
data: {"jsonrpc":"2.0","id":1,"result":{"test":true}}

"#;
        let result = parse_sse_response(sse_text).unwrap();
        assert!(result.result.is_some());
    }

    #[test]
    fn test_parse_sse_response_multiline_data() {
        // Some SSE implementations may split across lines
        let sse_text = r#"data: {"jsonrpc":"2.0",
data: "id":1,
data: "result":{}}
"#;
        // Our parser should handle this or fall back to raw JSON parse
        // Since this is malformed, it should still try to find valid JSON
        let result = parse_sse_response(sse_text);
        // The parser may not handle this case, which is expected
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_mcp_server_info_serialization() {
        let info = McpServerInfo {
            name: "test-server".to_string(),
            version: Some("1.2.3".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"test-server\""));
        assert!(json.contains("\"version\":\"1.2.3\""));
    }

    #[test]
    fn test_mcp_server_info_camel_case() {
        // Ensure camelCase is preserved
        let info = McpServerInfo {
            name: "test".to_string(),
            version: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        // name and version should NOT be converted to camelCase in output
        // since they're already lowercase
        assert!(json.contains("\"name\""));
    }

    #[test]
    fn test_mcp_test_result_defaults() {
        // Test that defaults work when deserializing minimal JSON
        let json = r#"{"success": false}"#;
        let result: McpTestResult = serde_json::from_str(json).unwrap();

        assert!(!result.success);
        assert!(result.server_info.is_none());
        assert!(result.tools.is_empty());
        assert!(!result.resources_supported);
        assert!(!result.prompts_supported);
        assert!(result.error.is_none());
        assert_eq!(result.response_time_ms, 0);
    }

    #[test]
    fn test_mcp_tool_empty_description() {
        let json = r#"{"name": "tool", "description": ""}"#;
        let tool: McpTool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.description, Some("".to_string()));
    }

    #[test]
    fn test_json_rpc_request_with_complex_params() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {"listChanged": true},
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("protocolVersion"));
        assert!(json.contains("capabilities"));
        assert!(json.contains("clientInfo"));
    }

    #[test]
    fn test_mcp_tool_array_deserialization() {
        let json = r#"[
            {"name": "tool1", "description": "First"},
            {"name": "tool2"},
            {"name": "tool3", "inputSchema": {"type": "object"}}
        ]"#;

        let tools: Vec<McpTool> = serde_json::from_str(json).unwrap();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "tool1");
        assert_eq!(tools[1].description, None);
        assert!(tools[2].input_schema.is_some());
    }
}
