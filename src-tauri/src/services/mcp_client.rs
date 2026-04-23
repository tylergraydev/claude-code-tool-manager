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
        self.server_info = init_result.get("serverInfo").map(|info| McpServerInfo {
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
        self.server_info = init_result.get("serverInfo").map(|info| McpServerInfo {
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
            error!(
                "[Streamable HTTP Client] Failed to send initialized notification: {}",
                e
            );
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

        info!(
            "[Streamable HTTP Client] Connected with {} tools",
            self.tools.len()
        );
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
            info!(
                "[Streamable HTTP Client] Failed to read body as text: {}",
                e
            );
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
                        info!(
                            "[Streamable HTTP Client] Found SSE data: {}",
                            &json_str[..json_str.len().min(200)]
                        );
                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(json_str) {
                            return Ok(response);
                        }
                    }
                }
            }
        }

        // Try parsing the whole body as JSON directly
        serde_json::from_str(&body_text).map_err(|e| {
            anyhow!(
                "Could not parse response: {}. Body: {}",
                e,
                &body_text[..body_text.len().min(200)]
            )
        })
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

    info!(
        "[MCP Client] Session ID for notifications: {:?}",
        session_id
    );

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
            info!(
                "[MCP Client] Initialized notification response status: {}",
                notify_status
            );
            // Read and log the response body
            if let Ok(body) = resp.text().await {
                if !body.is_empty() {
                    info!(
                        "[MCP Client] Notification response: {}",
                        &body[..body.len().min(200)]
                    );
                }
            }
        }
        Err(e) => {
            error!(
                "[MCP Client] Failed to send initialized notification: {}",
                e
            );
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
                    info!(
                        "[MCP Client] Found SSE data: {}",
                        &json_str[..json_str.len().min(200)]
                    );
                    if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(json_str) {
                        return Ok(response);
                    }
                }
            }
        }
    }

    // Try parsing the whole body as JSON directly
    serde_json::from_str(&body_text).map_err(|e| {
        anyhow!(
            "Could not parse response: {}. Body: {}",
            e,
            &body_text[..body_text.len().min(200)]
        )
    })
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

    // =========================================================================
    // ToolContent serialization/deserialization tests
    // =========================================================================

    #[test]
    fn test_tool_content_text_serialize() {
        let content = ToolContent::Text {
            text: "hello world".to_string(),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"text\":\"hello world\""));
    }

    #[test]
    fn test_tool_content_text_deserialize() {
        let json = r#"{"type":"text","text":"hello"}"#;
        let content: ToolContent = serde_json::from_str(json).unwrap();
        match content {
            ToolContent::Text { text } => assert_eq!(text, "hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_tool_content_image_roundtrip() {
        let content = ToolContent::Image {
            data: "base64data==".to_string(),
            mime_type: "image/png".to_string(),
        };
        let json = serde_json::to_string(&content).unwrap();
        let parsed: ToolContent = serde_json::from_str(&json).unwrap();
        match parsed {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "base64data==");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_tool_content_resource_roundtrip() {
        let content = ToolContent::Resource {
            uri: "file:///tmp/test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("file contents".to_string()),
        };
        let json = serde_json::to_string(&content).unwrap();
        let parsed: ToolContent = serde_json::from_str(&json).unwrap();
        match parsed {
            ToolContent::Resource {
                uri,
                mime_type,
                text,
            } => {
                assert_eq!(uri, "file:///tmp/test.txt");
                assert_eq!(mime_type, Some("text/plain".to_string()));
                assert_eq!(text, Some("file contents".to_string()));
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    #[test]
    fn test_tool_content_resource_minimal() {
        let json = r#"{"type":"resource","uri":"file:///test"}"#;
        let content: ToolContent = serde_json::from_str(json).unwrap();
        match content {
            ToolContent::Resource {
                uri,
                mime_type,
                text,
            } => {
                assert_eq!(uri, "file:///test");
                assert!(mime_type.is_none());
                assert!(text.is_none());
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    // =========================================================================
    // ToolCallResult tests
    // =========================================================================

    #[test]
    fn test_tool_call_result_serialize() {
        let result = ToolCallResult {
            success: true,
            content: vec![ToolContent::Text {
                text: "done".to_string(),
            }],
            is_error: false,
            error: None,
            execution_time_ms: 42,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"executionTimeMs\":42"));
        assert!(json.contains("\"isError\":false"));
    }

    #[test]
    fn test_tool_call_result_deserialize() {
        let json = r#"{
            "success": false,
            "content": [],
            "isError": true,
            "error": "something broke",
            "executionTimeMs": 500
        }"#;
        let result: ToolCallResult = serde_json::from_str(json).unwrap();
        assert!(!result.success);
        assert!(result.is_error);
        assert_eq!(result.error, Some("something broke".to_string()));
        assert_eq!(result.execution_time_ms, 500);
    }

    #[test]
    fn test_tool_call_result_with_content() {
        let json = r#"{
            "success": true,
            "content": [
                {"type": "text", "text": "line 1"},
                {"type": "text", "text": "line 2"}
            ],
            "isError": false,
            "executionTimeMs": 10
        }"#;
        let result: ToolCallResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.content.len(), 2);
    }

    // =========================================================================
    // parse_tool_result tests
    // =========================================================================

    #[test]
    fn test_parse_tool_result_success() {
        let result_json = json!({
            "content": [
                {"type": "text", "text": "result data"}
            ]
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 100).unwrap();
        assert!(result.success);
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.execution_time_ms, 100);
    }

    #[test]
    fn test_parse_tool_result_with_error_flag() {
        let result_json = json!({
            "content": [{"type": "text", "text": "error message"}],
            "isError": true
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 50).unwrap();
        assert!(!result.success);
        assert!(result.is_error);
    }

    #[test]
    fn test_parse_tool_result_empty_content() {
        let result_json = json!({});
        let result = StdioMcpClient::parse_tool_result(result_json, 5).unwrap();
        assert!(result.success);
        assert!(result.content.is_empty());
    }

    #[test]
    fn test_parse_tool_result_invalid_content() {
        let result_json = json!({
            "content": "not an array"
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 5).unwrap();
        assert!(result.content.is_empty()); // Gracefully handles invalid content
    }

    // =========================================================================
    // parse_sse_line tests
    // =========================================================================

    #[test]
    fn test_parse_sse_line_event_type() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        let result = parse_sse_line("event: endpoint", &mut current);
        assert!(result.is_none()); // No event emitted yet (waiting for empty line)
        assert_eq!(current.event_type, Some("endpoint".to_string()));
    }

    #[test]
    fn test_parse_sse_line_data() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        let result = parse_sse_line("data: /messages?sid=123", &mut current);
        assert!(result.is_none());
        assert_eq!(current.data, Some("/messages?sid=123".to_string()));
    }

    #[test]
    fn test_parse_sse_line_complete_event() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        parse_sse_line("event: endpoint", &mut current);
        parse_sse_line("data: /messages", &mut current);
        let event = parse_sse_line("", &mut current);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.event_type, Some("endpoint".to_string()));
        assert_eq!(event.data, Some("/messages".to_string()));

        // Current event should be cleared
        assert!(current.event_type.is_none());
        assert!(current.data.is_none());
    }

    #[test]
    fn test_parse_sse_line_empty_line_no_event() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        let result = parse_sse_line("", &mut current);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_line_ignores_comments_and_unknown() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        let result = parse_sse_line(": this is a comment", &mut current);
        assert!(result.is_none());
        assert!(current.event_type.is_none());
        assert!(current.data.is_none());

        let result = parse_sse_line("retry: 3000", &mut current);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_line_data_with_whitespace() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        parse_sse_line("data:   trimmed value  ", &mut current);
        assert_eq!(current.data, Some("trimmed value".to_string()));
    }

    // =========================================================================
    // McpTestResult additional tests
    // =========================================================================

    #[test]
    fn test_mcp_test_result_error_has_no_tools() {
        let result = McpTestResult::error("fail".to_string(), 100);
        assert!(result.tools.is_empty());
        assert!(result.server_info.is_none());
        assert!(!result.resources_supported);
        assert!(!result.prompts_supported);
    }

    #[test]
    fn test_mcp_test_result_success_all_features() {
        let result = McpTestResult::success(
            McpServerInfo {
                name: "full".to_string(),
                version: Some("3.0".to_string()),
            },
            vec![McpTool {
                name: "t1".to_string(),
                description: None,
                input_schema: None,
            }],
            true,
            true,
            0,
        );
        assert!(result.success);
        assert!(result.resources_supported);
        assert!(result.prompts_supported);
        assert_eq!(result.tools.len(), 1);
        assert_eq!(result.response_time_ms, 0);
    }

    // =========================================================================
    // McpTool additional serialization tests
    // =========================================================================

    #[test]
    fn test_mcp_tool_full_round_trip() {
        let original = McpTool {
            name: "search".to_string(),
            description: Some("Search for things".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "limit": {"type": "integer", "default": 10}
                },
                "required": ["query"]
            })),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: McpTool = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "search");
        assert_eq!(parsed.description, Some("Search for things".to_string()));
        let schema = parsed.input_schema.unwrap();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"][0], "query");
    }

    #[test]
    fn test_mcp_tool_camel_case_input_schema() {
        // Verify inputSchema is used in JSON (camelCase), not input_schema
        let tool = McpTool {
            name: "test".to_string(),
            description: None,
            input_schema: Some(json!({"type": "object"})),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("\"inputSchema\""));
        assert!(!json.contains("\"input_schema\""));
    }

    #[test]
    fn test_mcp_tool_null_vs_missing_fields() {
        // Explicit null values for optional fields
        let json = r#"{"name": "t", "description": null, "inputSchema": null}"#;
        let tool: McpTool = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "t");
        assert!(tool.description.is_none());
        assert!(tool.input_schema.is_none());
    }

    #[test]
    fn test_mcp_tool_unicode_name() {
        let tool = McpTool {
            name: "search_\u{1F50D}".to_string(),
            description: Some("Buscar \u{00E9}l\u{00E9}ments".to_string()),
            input_schema: None,
        };
        let json = serde_json::to_string(&tool).unwrap();
        let parsed: McpTool = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, tool.name);
        assert_eq!(parsed.description, tool.description);
    }

    // =========================================================================
    // ToolContent additional tests
    // =========================================================================

    #[test]
    fn test_tool_content_image_serialize_type_tag() {
        let content = ToolContent::Image {
            data: "abc123".to_string(),
            mime_type: "image/jpeg".to_string(),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"type\":\"image\""));
        assert!(json.contains("\"data\":\"abc123\""));
        assert!(json.contains("\"mime_type\":\"image/jpeg\""));
    }

    #[test]
    fn test_tool_content_resource_serialize_type_tag() {
        let content = ToolContent::Resource {
            uri: "file:///x".to_string(),
            mime_type: None,
            text: None,
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"type\":\"resource\""));
        assert!(json.contains("\"uri\":\"file:///x\""));
    }

    #[test]
    fn test_tool_content_text_empty_string() {
        let content = ToolContent::Text {
            text: "".to_string(),
        };
        let json = serde_json::to_string(&content).unwrap();
        let parsed: ToolContent = serde_json::from_str(&json).unwrap();
        match parsed {
            ToolContent::Text { text } => assert_eq!(text, ""),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_tool_content_image_deserialize() {
        let json = r#"{"type":"image","data":"iVBOR...","mime_type":"image/png"}"#;
        let content: ToolContent = serde_json::from_str(json).unwrap();
        match content {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "iVBOR...");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_tool_content_mixed_array() {
        let json = r#"[
            {"type":"text","text":"hello"},
            {"type":"image","data":"abc","mime_type":"image/png"},
            {"type":"resource","uri":"file:///tmp","mime_type":"text/plain","text":"content"}
        ]"#;
        let contents: Vec<ToolContent> = serde_json::from_str(json).unwrap();
        assert_eq!(contents.len(), 3);
        matches!(&contents[0], ToolContent::Text { .. });
        matches!(&contents[1], ToolContent::Image { .. });
        matches!(&contents[2], ToolContent::Resource { .. });
    }

    #[test]
    fn test_tool_content_resource_with_only_text() {
        let json = r#"{"type":"resource","uri":"mem://buf","text":"inline data"}"#;
        let content: ToolContent = serde_json::from_str(json).unwrap();
        match content {
            ToolContent::Resource {
                uri,
                mime_type,
                text,
            } => {
                assert_eq!(uri, "mem://buf");
                assert!(mime_type.is_none());
                assert_eq!(text, Some("inline data".to_string()));
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    // =========================================================================
    // ToolCallResult additional tests
    // =========================================================================

    #[test]
    fn test_tool_call_result_round_trip() {
        let original = ToolCallResult {
            success: true,
            content: vec![
                ToolContent::Text {
                    text: "output".to_string(),
                },
                ToolContent::Image {
                    data: "base64==".to_string(),
                    mime_type: "image/png".to_string(),
                },
            ],
            is_error: false,
            error: None,
            execution_time_ms: 123,
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ToolCallResult = serde_json::from_str(&json).unwrap();

        assert!(parsed.success);
        assert_eq!(parsed.content.len(), 2);
        assert!(!parsed.is_error);
        assert!(parsed.error.is_none());
        assert_eq!(parsed.execution_time_ms, 123);
    }

    #[test]
    fn test_tool_call_result_defaults_on_deserialize() {
        // isError and error have serde(default), verify they work
        let json = r#"{"success":true,"content":[],"executionTimeMs":0}"#;
        let result: ToolCallResult = serde_json::from_str(json).unwrap();
        assert!(!result.is_error);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_call_result_error_with_content() {
        // An error result can still carry content (e.g. partial output)
        let result = ToolCallResult {
            success: false,
            content: vec![ToolContent::Text {
                text: "partial output before crash".to_string(),
            }],
            is_error: true,
            error: Some("Process exited with code 1".to_string()),
            execution_time_ms: 5000,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ToolCallResult = serde_json::from_str(&json).unwrap();
        assert!(!parsed.success);
        assert!(parsed.is_error);
        assert_eq!(parsed.content.len(), 1);
        assert_eq!(parsed.error, Some("Process exited with code 1".to_string()));
    }

    // =========================================================================
    // McpServerInfo additional tests
    // =========================================================================

    #[test]
    fn test_mcp_server_info_round_trip_with_version() {
        let original = McpServerInfo {
            name: "my-mcp-server".to_string(),
            version: Some("0.1.0-beta.3".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: McpServerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "my-mcp-server");
        assert_eq!(parsed.version, Some("0.1.0-beta.3".to_string()));
    }

    #[test]
    fn test_mcp_server_info_round_trip_no_version() {
        let original = McpServerInfo {
            name: "bare".to_string(),
            version: None,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: McpServerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "bare");
        assert!(parsed.version.is_none());
    }

    #[test]
    fn test_mcp_server_info_extra_fields_ignored() {
        // serde should ignore unknown fields by default
        let json = r#"{"name":"s","version":"1","extraField":"ignored"}"#;
        let info: McpServerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "s");
    }

    // =========================================================================
    // next_request_id additional tests
    // =========================================================================

    #[test]
    fn test_next_request_id_always_positive() {
        for _ in 0..10 {
            let id = next_request_id();
            assert!(id > 0);
        }
    }

    #[test]
    fn test_next_request_id_no_duplicates() {
        let ids: Vec<u64> = (0..100).map(|_| next_request_id()).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(ids.len(), unique.len(), "All IDs should be unique");
    }

    // =========================================================================
    // JSON-RPC types additional tests
    // =========================================================================

    #[test]
    fn test_json_rpc_response_no_id_no_result() {
        // A notification-like response
        let json = r#"{"jsonrpc":"2.0"}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.id.is_none());
        assert!(response.result.is_none());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_request_tools_call_format() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 5,
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "read_file",
                "arguments": {"path": "/tmp/test.txt"}
            })),
        };
        let json = serde_json::to_string(&request).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 5);
        assert_eq!(parsed["method"], "tools/call");
        assert_eq!(parsed["params"]["name"], "read_file");
        assert_eq!(parsed["params"]["arguments"]["path"], "/tmp/test.txt");
    }

    #[test]
    fn test_json_rpc_error_code_values() {
        // Test standard JSON-RPC error codes
        let codes = vec![
            (-32700, "Parse error"),
            (-32600, "Invalid Request"),
            (-32601, "Method not found"),
            (-32602, "Invalid params"),
            (-32603, "Internal error"),
        ];

        for (code, message) in codes {
            let json = format!(
                r#"{{"jsonrpc":"2.0","id":1,"error":{{"code":{},"message":"{}"}}}}"#,
                code, message
            );
            let response: JsonRpcResponse = serde_json::from_str(&json).unwrap();
            let error = response.error.unwrap();
            assert_eq!(error.code, code);
            assert_eq!(error.message, message);
        }
    }

    #[test]
    fn test_json_rpc_notification_no_id_no_params() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: "initialized".to_string(),
        };
        let json = serde_json::to_string(&notification).unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], "initialized");
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("params").is_none());
    }

    #[test]
    fn test_json_rpc_response_with_large_result() {
        let big_result = json!({
            "tools": (0..50).map(|i| json!({
                "name": format!("tool_{}", i),
                "description": format!("Tool number {}", i),
                "inputSchema": {"type": "object"}
            })).collect::<Vec<_>>()
        });
        let json_str = format!(
            r#"{{"jsonrpc":"2.0","id":1,"result":{}}}"#,
            serde_json::to_string(&big_result).unwrap()
        );
        let response: JsonRpcResponse = serde_json::from_str(&json_str).unwrap();
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 50);
    }

    // =========================================================================
    // parse_tool_result additional tests
    // =========================================================================

    #[test]
    fn test_parse_tool_result_mixed_content_types() {
        let result_json = json!({
            "content": [
                {"type": "text", "text": "Description of image:"},
                {"type": "image", "data": "iVBOR...", "mime_type": "image/png"},
                {"type": "resource", "uri": "file:///tmp/log.txt", "text": "log data"}
            ]
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 200).unwrap();
        assert!(result.success);
        assert_eq!(result.content.len(), 3);

        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Description of image:"),
            _ => panic!("Expected Text"),
        }
        match &result.content[1] {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "iVBOR...");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected Image"),
        }
        match &result.content[2] {
            ToolContent::Resource { uri, text, .. } => {
                assert_eq!(uri, "file:///tmp/log.txt");
                assert_eq!(text, &Some("log data".to_string()));
            }
            _ => panic!("Expected Resource"),
        }
    }

    #[test]
    fn test_parse_tool_result_null_content() {
        let result_json = json!({"content": null});
        let result = StdioMcpClient::parse_tool_result(result_json, 1).unwrap();
        assert!(result.content.is_empty());
    }

    #[test]
    fn test_parse_tool_result_is_error_false_explicitly() {
        let result_json = json!({
            "content": [{"type": "text", "text": "ok"}],
            "isError": false
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 10).unwrap();
        assert!(result.success);
        assert!(!result.is_error);
    }

    // =========================================================================
    // parse_sse_line additional tests
    // =========================================================================

    #[test]
    fn test_parse_sse_line_message_event_with_json_data() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        parse_sse_line("event: message", &mut current);
        parse_sse_line(
            r#"data: {"jsonrpc":"2.0","id":1,"result":{}}"#,
            &mut current,
        );
        let event = parse_sse_line("", &mut current).unwrap();
        assert_eq!(event.event_type, Some("message".to_string()));
        let data = event.data.unwrap();
        let parsed: JsonRpcResponse = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.id, Some(1));
    }

    #[test]
    fn test_parse_sse_line_consecutive_events() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };

        // First event
        parse_sse_line("event: endpoint", &mut current);
        parse_sse_line("data: /messages", &mut current);
        let ev1 = parse_sse_line("", &mut current).unwrap();
        assert_eq!(ev1.event_type, Some("endpoint".to_string()));

        // Second event
        parse_sse_line("event: message", &mut current);
        parse_sse_line("data: {}", &mut current);
        let ev2 = parse_sse_line("", &mut current).unwrap();
        assert_eq!(ev2.event_type, Some("message".to_string()));
    }

    #[test]
    fn test_parse_sse_line_data_only_event() {
        // An event without an explicit event: field
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        parse_sse_line("data: some payload", &mut current);
        let event = parse_sse_line("", &mut current).unwrap();
        assert!(event.event_type.is_none());
        assert_eq!(event.data, Some("some payload".to_string()));
    }

    // =========================================================================
    // parse_sse_response additional tests
    // =========================================================================

    #[test]
    fn test_parse_sse_response_empty_data_lines_skipped() {
        let sse_text = "data: \ndata: \ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n";
        let result = parse_sse_response(sse_text).unwrap();
        assert!(result.result.is_some());
    }

    #[test]
    fn test_parse_sse_response_with_mixed_event_lines() {
        let sse_text = "event: message\nid: 42\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"tools\":[]}}\n\n";
        let result = parse_sse_response(sse_text).unwrap();
        let result_val = result.result.unwrap();
        let tools = result_val["tools"].as_array().unwrap();
        assert!(tools.is_empty());
    }

    #[test]
    fn test_parse_sse_response_completely_empty() {
        let result = parse_sse_response("");
        assert!(result.is_err());
    }

    // =========================================================================
    // McpTestResult deserialization edge cases
    // =========================================================================

    #[test]
    fn test_mcp_test_result_deserialize_with_all_defaults() {
        // Only the required field
        let json = r#"{"success": true}"#;
        let result: McpTestResult = serde_json::from_str(json).unwrap();
        assert!(result.success);
        assert!(result.server_info.is_none());
        assert!(result.tools.is_empty());
        assert!(!result.resources_supported);
        assert!(!result.prompts_supported);
        assert!(result.error.is_none());
        assert_eq!(result.response_time_ms, 0);
    }

    #[test]
    fn test_mcp_test_result_error_round_trip() {
        let original = McpTestResult::error("connection refused".to_string(), 42);
        let json = serde_json::to_string(&original).unwrap();
        let parsed: McpTestResult = serde_json::from_str(&json).unwrap();
        assert!(!parsed.success);
        assert_eq!(parsed.error, Some("connection refused".to_string()));
        assert_eq!(parsed.response_time_ms, 42);
    }

    // =========================================================================
    // McpTool serialization: verify serde rename_all camelCase
    // =========================================================================

    #[test]
    fn test_mcp_tool_serialization_uses_camel_case() {
        let tool = McpTool {
            name: "x".to_string(),
            description: Some("d".to_string()),
            input_schema: Some(json!({})),
        };
        let json = serde_json::to_value(&tool).unwrap();
        // Should have camelCase keys
        assert!(json.get("inputSchema").is_some());
        assert!(json.get("name").is_some());
        assert!(json.get("description").is_some());
        // Should NOT have snake_case
        assert!(json.get("input_schema").is_none());
    }

    // =========================================================================
    // ToolCallResult camelCase verification
    // =========================================================================

    #[test]
    fn test_tool_call_result_camel_case_keys() {
        let result = ToolCallResult {
            success: true,
            content: vec![],
            is_error: false,
            error: None,
            execution_time_ms: 1,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("isError").is_some());
        assert!(json.get("executionTimeMs").is_some());
        assert!(json.get("is_error").is_none());
        assert!(json.get("execution_time_ms").is_none());
    }

    // =========================================================================
    // McpTestResult camelCase verification
    // =========================================================================

    #[test]
    fn test_mcp_test_result_camel_case_keys() {
        let result = McpTestResult::success(
            McpServerInfo {
                name: "s".to_string(),
                version: None,
            },
            vec![],
            true,
            false,
            10,
        );
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("serverInfo").is_some());
        assert!(json.get("resourcesSupported").is_some());
        assert!(json.get("promptsSupported").is_some());
        assert!(json.get("responseTimeMs").is_some());
        // Verify no snake_case keys
        assert!(json.get("server_info").is_none());
        assert!(json.get("resources_supported").is_none());
        assert!(json.get("prompts_supported").is_none());
        assert!(json.get("response_time_ms").is_none());
    }

    // =========================================================================
    // build_http_request tests
    // =========================================================================

    #[test]
    fn test_build_http_request_basic() {
        let client = reqwest::blocking::Client::new();
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#.to_string();
        let builder = build_http_request(
            &client,
            "http://localhost:8080/mcp",
            body.clone(),
            None,
            None,
        );
        let req = builder.build().unwrap();
        assert_eq!(req.method(), "POST");
        assert_eq!(req.url().as_str(), "http://localhost:8080/mcp");
        // Check Content-Type and Accept headers
        let ct = req.headers().get("content-type").unwrap().to_str().unwrap();
        assert_eq!(ct, "application/json");
        let accept = req.headers().get("accept").unwrap().to_str().unwrap();
        assert!(accept.contains("application/json"));
        assert!(accept.contains("text/event-stream"));
        // Verify body
        let req_body = req.body().unwrap().as_bytes().unwrap();
        assert_eq!(req_body, body.as_bytes());
    }

    #[test]
    fn test_build_http_request_with_session_id() {
        let client = reqwest::blocking::Client::new();
        let body = "{}".to_string();
        let builder = build_http_request(
            &client,
            "http://localhost:8080/mcp",
            body,
            Some("session-abc-123"),
            None,
        );
        let req = builder.build().unwrap();
        let sid = req
            .headers()
            .get("mcp-session-id")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(sid, "session-abc-123");
    }

    #[test]
    fn test_build_http_request_with_custom_headers() {
        let client = reqwest::blocking::Client::new();
        let body = "{}".to_string();
        let mut custom_headers = HashMap::new();
        custom_headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        custom_headers.insert("X-Custom".to_string(), "value".to_string());
        let builder = build_http_request(
            &client,
            "http://localhost:8080/mcp",
            body,
            None,
            Some(&custom_headers),
        );
        let req = builder.build().unwrap();
        let auth = req
            .headers()
            .get("Authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(auth, "Bearer token123");
        let custom = req.headers().get("X-Custom").unwrap().to_str().unwrap();
        assert_eq!(custom, "value");
    }

    #[test]
    fn test_build_http_request_with_session_and_custom_headers() {
        let client = reqwest::blocking::Client::new();
        let body = r#"{"test": true}"#.to_string();
        let mut custom_headers = HashMap::new();
        custom_headers.insert("X-Api-Key".to_string(), "key123".to_string());
        let builder = build_http_request(
            &client,
            "http://example.com/mcp",
            body,
            Some("sess-xyz"),
            Some(&custom_headers),
        );
        let req = builder.build().unwrap();
        assert_eq!(
            req.headers()
                .get("mcp-session-id")
                .unwrap()
                .to_str()
                .unwrap(),
            "sess-xyz"
        );
        assert_eq!(
            req.headers().get("X-Api-Key").unwrap().to_str().unwrap(),
            "key123"
        );
    }

    // =========================================================================
    // URL construction for SSE endpoint resolution
    // =========================================================================

    #[test]
    fn test_sse_endpoint_url_absolute_http() {
        let endpoint_url = "http://other-server.com/messages";
        // Absolute URL should be used as-is
        let result = if endpoint_url.starts_with("http://") || endpoint_url.starts_with("https://")
        {
            endpoint_url.to_string()
        } else {
            let base_url = reqwest::Url::parse("http://localhost:8080/sse").unwrap();
            base_url.join(endpoint_url).unwrap().to_string()
        };
        assert_eq!(result, "http://other-server.com/messages");
    }

    #[test]
    fn test_sse_endpoint_url_absolute_https() {
        let endpoint_url = "https://secure.example.com/msg";
        let result = if endpoint_url.starts_with("http://") || endpoint_url.starts_with("https://")
        {
            endpoint_url.to_string()
        } else {
            let base_url = reqwest::Url::parse("http://localhost/sse").unwrap();
            base_url.join(endpoint_url).unwrap().to_string()
        };
        assert_eq!(result, "https://secure.example.com/msg");
    }

    #[test]
    fn test_sse_endpoint_url_relative_path() {
        let endpoint_url = "/messages?sessionId=abc";
        let result = if endpoint_url.starts_with("http://") || endpoint_url.starts_with("https://")
        {
            endpoint_url.to_string()
        } else {
            let base_url = reqwest::Url::parse("http://localhost:3000/sse").unwrap();
            base_url.join(endpoint_url).unwrap().to_string()
        };
        assert_eq!(result, "http://localhost:3000/messages?sessionId=abc");
    }

    #[test]
    fn test_sse_endpoint_url_relative_no_leading_slash() {
        let endpoint_url = "messages";
        let result = if endpoint_url.starts_with("http://") || endpoint_url.starts_with("https://")
        {
            endpoint_url.to_string()
        } else {
            let base_url = reqwest::Url::parse("http://localhost:3000/api/sse").unwrap();
            base_url.join(endpoint_url).unwrap().to_string()
        };
        assert_eq!(result, "http://localhost:3000/api/messages");
    }

    // =========================================================================
    // Session ID extraction from endpoint URL
    // =========================================================================

    #[test]
    fn test_session_id_extraction_from_endpoint_url() {
        let endpoint_str = "/messages?sessionId=abc-123-def";
        let mut session_id: Option<String> = None;
        if endpoint_str.contains("sessionId=") {
            if let Some(sid) = endpoint_str.split("sessionId=").nth(1) {
                session_id = Some(sid.split('&').next().unwrap_or(sid).to_string());
            }
        }
        assert_eq!(session_id, Some("abc-123-def".to_string()));
    }

    #[test]
    fn test_session_id_extraction_with_extra_params() {
        let endpoint_str = "/messages?sessionId=xyz&other=val";
        let mut session_id: Option<String> = None;
        if endpoint_str.contains("sessionId=") {
            if let Some(sid) = endpoint_str.split("sessionId=").nth(1) {
                session_id = Some(sid.split('&').next().unwrap_or(sid).to_string());
            }
        }
        assert_eq!(session_id, Some("xyz".to_string()));
    }

    #[test]
    fn test_session_id_extraction_no_session_id() {
        let endpoint_str = "/messages?other=val";
        let mut session_id: Option<String> = None;
        if endpoint_str.contains("sessionId=") {
            if let Some(sid) = endpoint_str.split("sessionId=").nth(1) {
                session_id = Some(sid.split('&').next().unwrap_or(sid).to_string());
            }
        }
        assert!(session_id.is_none());
    }

    // =========================================================================
    // JSON endpoint parsing (JSON-encoded string vs raw)
    // =========================================================================

    #[test]
    fn test_endpoint_json_encoded_string() {
        let data = r#""/messages?sid=123""#;
        let endpoint_str =
            serde_json::from_str::<String>(data).unwrap_or_else(|_| data.to_string());
        assert_eq!(endpoint_str, "/messages?sid=123");
    }

    #[test]
    fn test_endpoint_raw_string() {
        let data = "/messages?sid=456";
        let endpoint_str =
            serde_json::from_str::<String>(data).unwrap_or_else(|_| data.to_string());
        assert_eq!(endpoint_str, "/messages?sid=456");
    }

    // =========================================================================
    // parse_sse_response extended tests
    // =========================================================================

    #[test]
    fn test_parse_sse_response_only_done_markers() {
        let sse_text = "data: [DONE]\ndata: [DONE]\n";
        let result = parse_sse_response(sse_text);
        // No valid JSON-RPC data, and raw parse should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sse_response_data_with_colon_in_value() {
        // The data after "data:" may itself contain colons
        let sse_text =
            r#"data: {"jsonrpc":"2.0","id":1,"result":{"url":"http://example.com:8080"}}"#;
        let result = parse_sse_response(sse_text).unwrap();
        let res = result.result.unwrap();
        assert_eq!(res["url"], "http://example.com:8080");
    }

    #[test]
    fn test_parse_sse_response_with_comments_interspersed() {
        let sse_text =
            ": keep-alive\n: heartbeat\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{}}\n";
        let result = parse_sse_response(sse_text).unwrap();
        assert!(result.result.is_some());
    }

    #[test]
    fn test_parse_sse_response_error_message_content() {
        let result = parse_sse_response("totally not json or sse");
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("Could not parse SSE response"));
        assert!(err_str.contains("No valid JSON-RPC message found"));
    }

    #[test]
    fn test_parse_sse_response_whitespace_only() {
        let result = parse_sse_response("   \n\n  \n");
        assert!(result.is_err());
    }

    // =========================================================================
    // JsonRpcRequest init params format verification
    // =========================================================================

    #[test]
    fn test_initialize_request_format() {
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "claude-code-tool-manager",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "initialize".to_string(),
            params: Some(init_params.clone()),
        };
        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(serialized["jsonrpc"], "2.0");
        assert_eq!(serialized["method"], "initialize");
        assert_eq!(serialized["params"]["protocolVersion"], "2024-11-05");
        assert_eq!(
            serialized["params"]["clientInfo"]["name"],
            "claude-code-tool-manager"
        );
        assert!(serialized["params"]["capabilities"].is_object());
    }

    #[test]
    fn test_tools_list_request_format() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 2,
            method: "tools/list".to_string(),
            params: Some(json!({})),
        };
        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(serialized["method"], "tools/list");
        assert!(serialized["params"].is_object());
    }

    #[test]
    fn test_tools_call_request_format() {
        let params = json!({
            "name": "read_file",
            "arguments": {"path": "/tmp/test.txt"}
        });
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 3,
            method: "tools/call".to_string(),
            params: Some(params),
        };
        let serialized = serde_json::to_value(&request).unwrap();
        assert_eq!(serialized["method"], "tools/call");
        assert_eq!(serialized["params"]["name"], "read_file");
        assert_eq!(serialized["params"]["arguments"]["path"], "/tmp/test.txt");
    }

    // =========================================================================
    // JsonRpcNotification format tests
    // =========================================================================

    #[test]
    fn test_notification_initialized_format() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: "initialized".to_string(),
        };
        let json = serde_json::to_value(&notification).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "initialized");
        assert!(json.get("id").is_none());
        assert!(json.get("params").is_none());
    }

    #[test]
    fn test_notification_notifications_initialized_format() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: "notifications/initialized".to_string(),
        };
        let json = serde_json::to_value(&notification).unwrap();
        assert_eq!(json["method"], "notifications/initialized");
    }

    // =========================================================================
    // JsonRpcResponse deserialization edge cases
    // =========================================================================

    #[test]
    fn test_json_rpc_response_missing_id_field() {
        let json = r#"{"jsonrpc":"2.0","result":{"tools":[]}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.id.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_json_rpc_response_both_result_and_error_prefers_parse() {
        // Per JSON-RPC spec this shouldn't happen, but test that parsing still works
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{},"error":{"code":-1,"message":"weird"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_some());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_json_rpc_response_with_nested_result() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"serverInfo":{"name":"test","version":"1.0"},"capabilities":{"resources":{},"prompts":{}}}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let result = response.result.unwrap();
        assert!(result.get("serverInfo").is_some());
        assert!(result.get("capabilities").is_some());
        let caps = result.get("capabilities").unwrap();
        assert!(caps.get("resources").is_some());
        assert!(caps.get("prompts").is_some());
    }

    #[test]
    fn test_json_rpc_error_without_data_field() {
        let json =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert_eq!(error.message, "Method not found");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_json_rpc_error_with_complex_data() {
        let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"Server error","data":{"details":"stack trace here","retryable":true}}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let error = response.error.unwrap();
        let data = error.data.unwrap();
        assert_eq!(data["details"], "stack trace here");
        assert_eq!(data["retryable"], true);
    }

    // =========================================================================
    // parse_tool_result additional edge cases
    // =========================================================================

    #[test]
    fn test_parse_tool_result_with_multiple_text_contents() {
        let result_json = json!({
            "content": [
                {"type": "text", "text": "line 1"},
                {"type": "text", "text": "line 2"},
                {"type": "text", "text": "line 3"}
            ]
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 42).unwrap();
        assert!(result.success);
        assert_eq!(result.content.len(), 3);
        assert_eq!(result.execution_time_ms, 42);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_parse_tool_result_with_is_error_missing() {
        // isError field missing entirely - should default to false
        let result_json = json!({
            "content": [{"type": "text", "text": "ok"}]
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 0).unwrap();
        assert!(result.success);
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_tool_result_with_zero_elapsed() {
        let result_json = json!({});
        let result = StdioMcpClient::parse_tool_result(result_json, 0).unwrap();
        assert_eq!(result.execution_time_ms, 0);
    }

    #[test]
    fn test_parse_tool_result_with_large_elapsed() {
        let result_json = json!({"content": []});
        let result = StdioMcpClient::parse_tool_result(result_json, u64::MAX).unwrap();
        assert_eq!(result.execution_time_ms, u64::MAX);
    }

    #[test]
    fn test_parse_tool_result_with_image_content() {
        let result_json = json!({
            "content": [
                {"type": "image", "data": "iVBORw0KGgo...", "mime_type": "image/png"}
            ]
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 50).unwrap();
        assert_eq!(result.content.len(), 1);
        match &result.content[0] {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "iVBORw0KGgo...");
                assert_eq!(mime_type, "image/png");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_parse_tool_result_with_resource_content() {
        let result_json = json!({
            "content": [
                {"type": "resource", "uri": "file:///tmp/out.txt", "mime_type": "text/plain", "text": "file data"}
            ]
        });
        let result = StdioMcpClient::parse_tool_result(result_json, 30).unwrap();
        assert_eq!(result.content.len(), 1);
        match &result.content[0] {
            ToolContent::Resource {
                uri,
                mime_type,
                text,
            } => {
                assert_eq!(uri, "file:///tmp/out.txt");
                assert_eq!(mime_type, &Some("text/plain".to_string()));
                assert_eq!(text, &Some("file data".to_string()));
            }
            _ => panic!("Expected Resource variant"),
        }
    }

    // =========================================================================
    // ToolCallResult construction patterns (matching error paths in call_tool)
    // =========================================================================

    #[test]
    fn test_tool_call_result_error_construction() {
        // Matches the pattern used in StdioMcpClient::call_tool error branch
        let error_msg = "Connection refused".to_string();
        let elapsed = 150u64;
        let result = ToolCallResult {
            success: false,
            content: vec![],
            is_error: true,
            error: Some(error_msg.clone()),
            execution_time_ms: elapsed,
        };
        assert!(!result.success);
        assert!(result.is_error);
        assert!(result.content.is_empty());
        assert_eq!(result.error, Some("Connection refused".to_string()));
        assert_eq!(result.execution_time_ms, 150);
    }

    #[test]
    fn test_tool_call_result_http_error_construction() {
        // Matches the pattern in StreamableHttpMcpClient::call_tool_async HTTP error path
        let body = "Internal Server Error";
        let elapsed = 200u64;
        let result = ToolCallResult {
            success: false,
            content: vec![],
            is_error: true,
            error: Some(format!("HTTP error: {}", body)),
            execution_time_ms: elapsed,
        };
        assert_eq!(
            result.error,
            Some("HTTP error: Internal Server Error".to_string())
        );
    }

    // =========================================================================
    // McpTestResult construction for various error scenarios
    // =========================================================================

    #[test]
    fn test_mcp_test_result_error_runtime_failure() {
        let result = McpTestResult::error(
            format!("Failed to create async runtime: {}", "test error"),
            0,
        );
        assert!(!result.success);
        assert!(result
            .error
            .unwrap()
            .contains("Failed to create async runtime"));
        assert_eq!(result.response_time_ms, 0);
    }

    #[test]
    fn test_mcp_test_result_error_dns() {
        let result = McpTestResult::error(
            "Cannot resolve host. Check that the URL is correct.".to_string(),
            5000,
        );
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Cannot resolve host"));
    }

    #[test]
    fn test_mcp_test_result_error_connection_refused() {
        let result = McpTestResult::error(
            "Connection refused. The server may not be running.".to_string(),
            100,
        );
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Connection refused"));
    }

    #[test]
    fn test_mcp_test_result_error_ssl() {
        let result = McpTestResult::error(
            "SSL/TLS error: certificate verify failed. The server may have an invalid certificate."
                .to_string(),
            300,
        );
        assert!(!result.success);
        let err = result.error.unwrap();
        assert!(err.contains("SSL/TLS error"));
        assert!(err.contains("invalid certificate"));
    }

    #[test]
    fn test_mcp_test_result_success_no_version() {
        let result = McpTestResult::success(
            McpServerInfo {
                name: "unknown".to_string(),
                version: None,
            },
            vec![],
            false,
            false,
            0,
        );
        assert!(result.success);
        let info = result.server_info.unwrap();
        assert_eq!(info.name, "unknown");
        assert!(info.version.is_none());
        assert!(!result.resources_supported);
        assert!(!result.prompts_supported);
    }

    // =========================================================================
    // Server info parsing patterns (matching initialize response parsing)
    // =========================================================================

    #[test]
    fn test_server_info_parsing_from_init_result_with_info() {
        let init_result = json!({
            "serverInfo": {
                "name": "my-mcp-server",
                "version": "2.0.0"
            },
            "capabilities": {
                "resources": {},
                "prompts": {}
            }
        });

        let server_info = init_result.get("serverInfo").map(|info| McpServerInfo {
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
        assert!(server_info.is_some());
        let si = server_info.unwrap();
        assert_eq!(si.name, "my-mcp-server");
        assert_eq!(si.version, Some("2.0.0".to_string()));

        let capabilities = init_result.get("capabilities");
        let resources_supported = capabilities.and_then(|c| c.get("resources")).is_some();
        let prompts_supported = capabilities.and_then(|c| c.get("prompts")).is_some();
        assert!(resources_supported);
        assert!(prompts_supported);
    }

    #[test]
    fn test_server_info_parsing_from_init_result_no_info() {
        let init_result = json!({
            "capabilities": {}
        });

        let server_info = init_result.get("serverInfo").map(|info| McpServerInfo {
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
        assert!(server_info.is_none());
    }

    #[test]
    fn test_server_info_parsing_name_fallback() {
        // serverInfo present but name missing - falls back to "unknown"
        let init_result = json!({
            "serverInfo": {}
        });

        let info = init_result.get("serverInfo").unwrap();
        let name = info
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let version = info
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        assert_eq!(name, "unknown");
        assert!(version.is_none());
    }

    #[test]
    fn test_capabilities_parsing_no_resources_no_prompts() {
        let init_result = json!({
            "capabilities": {
                "logging": {}
            }
        });
        let capabilities = init_result.get("capabilities");
        let resources_supported = capabilities.and_then(|c| c.get("resources")).is_some();
        let prompts_supported = capabilities.and_then(|c| c.get("prompts")).is_some();
        assert!(!resources_supported);
        assert!(!prompts_supported);
    }

    #[test]
    fn test_capabilities_parsing_missing_entirely() {
        let init_result = json!({
            "serverInfo": {"name": "test"}
        });
        let capabilities = init_result.get("capabilities");
        assert!(capabilities.is_none());
        let resources_supported = capabilities.and_then(|c| c.get("resources")).is_some();
        let prompts_supported = capabilities.and_then(|c| c.get("prompts")).is_some();
        assert!(!resources_supported);
        assert!(!prompts_supported);
    }

    // =========================================================================
    // Tools parsing from response
    // =========================================================================

    #[test]
    fn test_tools_parsing_from_tools_list_response() {
        let tools_result = json!({
            "tools": [
                {"name": "read_file", "description": "Read a file", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}},
                {"name": "write_file", "description": "Write a file"},
                {"name": "list_dir"}
            ]
        });

        let tools: Vec<McpTool> = if let Some(tools_array) = tools_result.get("tools") {
            serde_json::from_value(tools_array.clone()).unwrap_or_default()
        } else {
            vec![]
        };

        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name, "read_file");
        assert!(tools[0].input_schema.is_some());
        assert_eq!(tools[1].name, "write_file");
        assert!(tools[1].input_schema.is_none());
        assert_eq!(tools[2].name, "list_dir");
        assert!(tools[2].description.is_none());
    }

    #[test]
    fn test_tools_parsing_no_tools_field() {
        let tools_result = json!({});
        let tools: Vec<McpTool> = if let Some(tools_array) = tools_result.get("tools") {
            serde_json::from_value(tools_array.clone()).unwrap_or_default()
        } else {
            vec![]
        };
        assert!(tools.is_empty());
    }

    #[test]
    fn test_tools_parsing_invalid_tools_value() {
        let tools_result = json!({"tools": "not an array"});
        let tools: Vec<McpTool> = if let Some(tools_array) = tools_result.get("tools") {
            serde_json::from_value(tools_array.clone()).unwrap_or_default()
        } else {
            vec![]
        };
        assert!(tools.is_empty());
    }

    // =========================================================================
    // parse_sse_line more patterns
    // =========================================================================

    #[test]
    fn test_parse_sse_line_id_field_ignored() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        let result = parse_sse_line("id: 12345", &mut current);
        assert!(result.is_none());
        // id field doesn't set event_type or data
        assert!(current.event_type.is_none());
        assert!(current.data.is_none());
    }

    #[test]
    fn test_parse_sse_line_retry_field_ignored() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        let result = parse_sse_line("retry: 5000", &mut current);
        assert!(result.is_none());
        assert!(current.event_type.is_none());
        assert!(current.data.is_none());
    }

    #[test]
    fn test_parse_sse_line_event_with_extra_whitespace() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        parse_sse_line("  event:   message  ", &mut current);
        assert_eq!(current.event_type, Some("message".to_string()));
    }

    #[test]
    fn test_parse_sse_line_data_json_rpc_response() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        let json_data = r#"{"jsonrpc":"2.0","id":42,"result":{"serverInfo":{"name":"test"}}}"#;
        parse_sse_line(&format!("data: {}", json_data), &mut current);
        assert_eq!(current.data, Some(json_data.to_string()));
        // Verify the data is valid JSON-RPC
        let parsed: JsonRpcResponse = serde_json::from_str(current.data.as_ref().unwrap()).unwrap();
        assert_eq!(parsed.id, Some(42));
    }

    #[test]
    fn test_parse_sse_line_multiple_empty_lines() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        // Multiple empty lines without any data should not produce events
        assert!(parse_sse_line("", &mut current).is_none());
        assert!(parse_sse_line("", &mut current).is_none());
        assert!(parse_sse_line("", &mut current).is_none());
    }

    #[test]
    fn test_parse_sse_line_event_only_no_data() {
        let mut current = SseEvent {
            event_type: None,
            data: None,
        };
        parse_sse_line("event: ping", &mut current);
        let event = parse_sse_line("", &mut current);
        // Event with type but no data is still emitted
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.event_type, Some("ping".to_string()));
        assert!(ev.data.is_none());
    }

    // =========================================================================
    // ToolContent: verify all variants in array deserialization
    // =========================================================================

    #[test]
    fn test_tool_content_array_with_all_types() {
        let json = r#"[
            {"type":"text","text":"output"},
            {"type":"image","data":"AAAA","mime_type":"image/gif"},
            {"type":"resource","uri":"https://example.com","mime_type":"text/html","text":"<html></html>"}
        ]"#;
        let contents: Vec<ToolContent> = serde_json::from_str(json).unwrap();
        assert_eq!(contents.len(), 3);

        match &contents[0] {
            ToolContent::Text { text } => assert_eq!(text, "output"),
            _ => panic!("Expected Text"),
        }
        match &contents[1] {
            ToolContent::Image { data, mime_type } => {
                assert_eq!(data, "AAAA");
                assert_eq!(mime_type, "image/gif");
            }
            _ => panic!("Expected Image"),
        }
        match &contents[2] {
            ToolContent::Resource {
                uri,
                mime_type,
                text,
            } => {
                assert_eq!(uri, "https://example.com");
                assert_eq!(mime_type, &Some("text/html".to_string()));
                assert_eq!(text, &Some("<html></html>".to_string()));
            }
            _ => panic!("Expected Resource"),
        }
    }

    #[test]
    fn test_tool_content_empty_array() {
        let json = "[]";
        let contents: Vec<ToolContent> = serde_json::from_str(json).unwrap();
        assert!(contents.is_empty());
    }

    // =========================================================================
    // Error message formatting patterns
    // =========================================================================

    #[test]
    fn test_npm_auth_error_detection() {
        // Test the patterns that would be detected in stderr parsing
        let stderr_msg = "Access token expired for @company/mcp-server";
        let contains_auth_error =
            stderr_msg.contains("Access token expired") || stderr_msg.contains("token revoked");
        assert!(contains_auth_error);

        let stderr_msg2 = "npm ERR! token revoked";
        let contains_auth_error2 =
            stderr_msg2.contains("Access token expired") || stderr_msg2.contains("token revoked");
        assert!(contains_auth_error2);

        let stderr_msg3 = "some other error";
        let contains_auth_error3 =
            stderr_msg3.contains("Access token expired") || stderr_msg3.contains("token revoked");
        assert!(!contains_auth_error3);
    }

    #[test]
    fn test_http_error_categorization_dns() {
        let err_str = "dns error: failed to lookup hostname";
        let is_dns = err_str.contains("dns error")
            || err_str.contains("resolve")
            || err_str.contains("No such host");
        assert!(is_dns);
    }

    #[test]
    fn test_http_error_categorization_connection_refused() {
        let err_str = "connection refused on localhost:8080";
        let is_refused = err_str.contains("connection refused");
        assert!(is_refused);
    }

    #[test]
    fn test_http_error_categorization_timeout() {
        let err_str = "operation timed out after 30s";
        let is_timeout = err_str.contains("timed out") || err_str.contains("timeout");
        assert!(is_timeout);

        let err_str2 = "request timeout exceeded";
        let is_timeout2 = err_str2.contains("timed out") || err_str2.contains("timeout");
        assert!(is_timeout2);
    }

    #[test]
    fn test_http_error_categorization_ssl() {
        let err_str = "SSL certificate problem: unable to get local issuer certificate";
        let is_ssl =
            err_str.contains("certificate") || err_str.contains("SSL") || err_str.contains("TLS");
        assert!(is_ssl);

        let err_str2 = "TLS handshake failed";
        let is_ssl2 = err_str2.contains("certificate")
            || err_str2.contains("SSL")
            || err_str2.contains("TLS");
        assert!(is_ssl2);
    }

    #[test]
    fn test_http_error_categorization_generic() {
        let err_str = "unexpected error happened";
        let is_dns = err_str.contains("dns error")
            || err_str.contains("resolve")
            || err_str.contains("No such host");
        let is_refused = err_str.contains("connection refused");
        let is_timeout = err_str.contains("timed out") || err_str.contains("timeout");
        let is_ssl =
            err_str.contains("certificate") || err_str.contains("SSL") || err_str.contains("TLS");
        assert!(!is_dns && !is_refused && !is_timeout && !is_ssl);
    }

    // =========================================================================
    // McpTestResult with many tools
    // =========================================================================

    #[test]
    fn test_mcp_test_result_with_many_tools() {
        let tools: Vec<McpTool> = (0..100)
            .map(|i| McpTool {
                name: format!("tool_{}", i),
                description: Some(format!("Tool number {}", i)),
                input_schema: Some(
                    json!({"type": "object", "properties": {"arg": {"type": "string"}}}),
                ),
            })
            .collect();

        let result = McpTestResult::success(
            McpServerInfo {
                name: "mega-server".to_string(),
                version: Some("5.0.0".to_string()),
            },
            tools,
            true,
            true,
            5000,
        );
        assert_eq!(result.tools.len(), 100);
        assert_eq!(result.tools[0].name, "tool_0");
        assert_eq!(result.tools[99].name, "tool_99");

        // Verify round-trip works with many tools
        let json = serde_json::to_string(&result).unwrap();
        let parsed: McpTestResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tools.len(), 100);
    }

    // =========================================================================
    // Full JSON-RPC response + server info + tools parsing integration
    // =========================================================================

    #[test]
    fn test_full_initialize_response_parsing() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","serverInfo":{"name":"my-mcp","version":"1.2.3"},"capabilities":{"resources":{"subscribe":true},"prompts":{"listChanged":true}}}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();
        assert!(response.error.is_none());
        let result = response.result.unwrap();

        // Parse server info
        let info = result.get("serverInfo").unwrap();
        let server_info = McpServerInfo {
            name: info
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            version: info
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };
        assert_eq!(server_info.name, "my-mcp");
        assert_eq!(server_info.version, Some("1.2.3".to_string()));

        // Parse capabilities
        let caps = result.get("capabilities").unwrap();
        assert!(caps.get("resources").is_some());
        assert!(caps.get("prompts").is_some());
    }

    #[test]
    fn test_full_tools_list_response_parsing() {
        let json_str = r#"{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"read","description":"Read file","inputSchema":{"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}},{"name":"write","description":"Write file","inputSchema":{"type":"object"}}]}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();
        let result = response.result.unwrap();
        let tools: Vec<McpTool> = serde_json::from_value(result["tools"].clone()).unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "read");
        assert_eq!(tools[0].description, Some("Read file".to_string()));
        let schema = tools[0].input_schema.as_ref().unwrap();
        assert_eq!(schema["required"][0], "path");
    }

    #[test]
    fn test_full_tool_call_response_parsing() {
        let json_str = r#"{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"File contents here"}],"isError":false}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();
        let result = response.result.unwrap();
        let tool_result = StdioMcpClient::parse_tool_result(result, 75).unwrap();
        assert!(tool_result.success);
        assert_eq!(tool_result.content.len(), 1);
        match &tool_result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "File contents here"),
            _ => panic!("Expected Text"),
        }
    }

    #[test]
    fn test_full_tool_call_error_response_parsing() {
        let json_str = r#"{"jsonrpc":"2.0","id":4,"result":{"content":[{"type":"text","text":"Error: file not found"}],"isError":true}}"#;
        let response: JsonRpcResponse = serde_json::from_str(json_str).unwrap();
        let result = response.result.unwrap();
        let tool_result = StdioMcpClient::parse_tool_result(result, 25).unwrap();
        assert!(!tool_result.success);
        assert!(tool_result.is_error);
    }

    // =========================================================================
    // SSE response parsing with realistic server output
    // =========================================================================

    #[test]
    fn test_parse_sse_response_realistic_initialize() {
        let sse_text = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"protocolVersion\":\"2024-11-05\",\"serverInfo\":{\"name\":\"test-mcp\",\"version\":\"0.1.0\"},\"capabilities\":{\"resources\":{},\"prompts\":{}}}}\n\n";
        let result = parse_sse_response(sse_text).unwrap();
        let init_result = result.result.unwrap();
        assert_eq!(init_result["serverInfo"]["name"], "test-mcp");
        assert_eq!(init_result["protocolVersion"], "2024-11-05");
    }

    #[test]
    fn test_parse_sse_response_realistic_tools_list() {
        let sse_text = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"tools\":[{\"name\":\"search\",\"description\":\"Search things\",\"inputSchema\":{\"type\":\"object\",\"properties\":{\"query\":{\"type\":\"string\"}}}}]}}\n\n";
        let result = parse_sse_response(sse_text).unwrap();
        let tools_result = result.result.unwrap();
        let tools: Vec<McpTool> = serde_json::from_value(tools_result["tools"].clone()).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "search");
    }

    // =========================================================================
    // McpServerInfo with special characters
    // =========================================================================

    #[test]
    fn test_mcp_server_info_special_characters() {
        let info = McpServerInfo {
            name: "server/with-special_chars.v2".to_string(),
            version: Some("1.0.0-rc.1+build.123".to_string()),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: McpServerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "server/with-special_chars.v2");
        assert_eq!(parsed.version, Some("1.0.0-rc.1+build.123".to_string()));
    }

    // =========================================================================
    // Content-type SSE detection pattern
    // =========================================================================

    #[test]
    fn test_content_type_sse_detection() {
        let ct1 = "text/event-stream";
        assert!(ct1.contains("text/event-stream"));

        let ct2 = "text/event-stream; charset=utf-8";
        assert!(ct2.contains("text/event-stream"));

        let ct3 = "application/json";
        assert!(!ct3.contains("text/event-stream"));

        let ct4 = "";
        assert!(!ct4.contains("text/event-stream"));
    }

    // =========================================================================
    // Accepted status pattern (SSE client)
    // =========================================================================

    #[test]
    fn test_accepted_status_response_parsing() {
        // Pattern from SseMcpClient::send_request for 202 responses
        let text = "";
        let is_empty = text.trim().is_empty();
        assert!(is_empty);

        // Check for status acknowledgment
        let ack_text = r#"{"status":"accepted"}"#;
        let status_obj: serde_json::Value = serde_json::from_str(ack_text).unwrap();
        assert_eq!(
            status_obj.get("status").and_then(|s| s.as_str()),
            Some("accepted")
        );

        // Non-acknowledgment
        let non_ack = r#"{"status":"processing"}"#;
        let status_obj2: serde_json::Value = serde_json::from_str(non_ack).unwrap();
        assert_ne!(
            status_obj2.get("status").and_then(|s| s.as_str()),
            Some("accepted")
        );
    }
}
