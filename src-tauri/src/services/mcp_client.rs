//! MCP Client for testing MCP server connections
//!
//! Implements the MCP protocol to connect to servers, perform handshake,
//! and retrieve available tools.

use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

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

struct StdioMcpClient {
    child: Child,
    timeout: Duration,
}

impl StdioMcpClient {
    fn spawn(
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
                            if stderr_msg.contains("Access token expired") || stderr_msg.contains("token revoked") {
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

    fn close(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ============================================================================
// SSE Response Parsing
// ============================================================================

/// Parse an SSE (Server-Sent Events) response to extract JSON-RPC message
/// SSE format:
/// ```
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
                info!("[MCP Client] Found SSE data: {}", &json_str[..json_str.len().min(200)]);
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
    let mut client = StdioMcpClient::spawn(command, args, env, timeout_secs)?;

    // Step 1: Send initialize request
    info!("[MCP Client] Sending initialize request...");
    let init_params = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "claude-code-tool-manager",
            "version": env!("CARGO_PKG_VERSION")
        }
    });

    let init_result = client.send_request("initialize", Some(init_params))?;

    // Parse server info and capabilities
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
    let resources_supported = capabilities
        .and_then(|c| c.get("resources"))
        .is_some();
    let prompts_supported = capabilities
        .and_then(|c| c.get("prompts"))
        .is_some();

    info!(
        "[MCP Client] Server: {} v{:?}",
        server_info.name, server_info.version
    );

    // Step 2: Send initialized notification
    info!("[MCP Client] Sending initialized notification...");
    client.send_notification("initialized")?;

    // Step 3: List tools
    info!("[MCP Client] Requesting tools list...");
    let tools_result = client.send_request("tools/list", Some(json!({})))?;

    let tools: Vec<McpTool> = if let Some(tools_array) = tools_result.get("tools") {
        serde_json::from_value(tools_array.clone()).unwrap_or_default()
    } else {
        vec![]
    };

    info!("[MCP Client] Found {} tools", tools.len());

    // Clean up
    client.close();

    Ok((server_info, tools, resources_supported, prompts_supported))
}

/// Test an SSE-based MCP server
/// SSE transport works differently:
/// 1. Client connects via GET to SSE endpoint
/// 2. Server sends 'endpoint' event with POST URL for messages
/// 3. Client sends JSON-RPC via POST to that endpoint
/// 4. Responses come back via SSE events
pub fn test_sse_mcp(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    timeout_secs: u64,
) -> McpTestResult {
    let start = Instant::now();

    let result = test_sse_mcp_internal(url, headers, timeout_secs);

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

fn test_sse_mcp_internal(
    url: &str,
    headers: Option<&HashMap<String, String>>,
    _timeout_secs: u64,
) -> Result<(McpServerInfo, Vec<McpTool>, bool, bool)> {
    info!("[MCP Client] Testing SSE MCP at: {}", url);

    // Use a short timeout for SSE since it's a streaming connection
    // We just want to verify connectivity and get initial events
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    // SSE uses GET to establish connection
    let mut request_builder = client.get(url)
        .header("Accept", "text/event-stream");

    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            request_builder = request_builder.header(key, value);
        }
    }

    info!("[MCP Client] Connecting to SSE endpoint via GET...");
    let response = request_builder.send().map_err(|e| {
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

    info!("[MCP Client] SSE response status: {}, content-type: {}", status, content_type);

    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(anyhow!(
            "SSE connection failed with status {}: {}",
            status,
            if body.is_empty() { status.canonical_reason().unwrap_or("Unknown").to_string() } else { body[..body.len().min(200)].to_string() }
        ));
    }

    if !content_type.contains("text/event-stream") {
        return Err(anyhow!(
            "Server did not return SSE content-type. Got: {}. This endpoint may not support SSE transport.",
            content_type
        ));
    }

    // SSE connection verified!
    // We can't read the streaming body with blocking reqwest - it would hang forever.
    // The successful 200 + text/event-stream content-type confirms SSE is working.
    info!("[MCP Client] SSE connection verified (status 200, content-type: text/event-stream)");

    // Drop the response to close the connection cleanly
    drop(response);

    Ok((
        McpServerInfo {
            name: "SSE Server".to_string(),
            version: Some("connected".to_string()),
        },
        vec![],  // Can't list tools without async SSE handling
        false,
        false,
    ))
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

/// Helper to build an HTTP request with common headers
fn build_http_request(
    client: &reqwest::blocking::Client,
    url: &str,
    body: String,
    session_id: Option<&str>,
    custom_headers: Option<&HashMap<String, String>>,
) -> reqwest::blocking::RequestBuilder {
    let mut builder = client.post(url)
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
    info!("[MCP Client] Sending HTTP initialize request: {}", request_body);

    let response = build_http_request(&client, url, request_body, None, headers)
        .send()
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("dns error") || err_str.contains("resolve") || err_str.contains("No such host") {
                anyhow!("Cannot resolve host. Check that the URL is correct and the server is online.")
            } else if err_str.contains("connection refused") {
                anyhow!("Connection refused. The server may not be running or the port is incorrect.")
            } else if err_str.contains("timed out") || err_str.contains("timeout") {
                anyhow!("Connection timed out. The server may be slow or unreachable.")
            } else if err_str.contains("certificate") || err_str.contains("SSL") || err_str.contains("TLS") {
                anyhow!("SSL/TLS error: {}. The server may have an invalid certificate.", err_str)
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

    info!("[MCP Client] Response status: {}, content-type: {}", status, content_type);

    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(anyhow!(
            "HTTP error {}: {}",
            status,
            if body.is_empty() { status.canonical_reason().unwrap_or("Unknown error").to_string() } else { body }
        ));
    }

    let response_text = response.text().map_err(|e| anyhow!("Failed to read response: {}", e))?;
    info!("[MCP Client] Response body: {}", &response_text[..response_text.len().min(500)]);

    // Parse response - handle both JSON and SSE formats
    let init_response: JsonRpcResponse = if content_type.contains("text/event-stream") {
        // Parse SSE format: extract JSON from "data:" lines
        parse_sse_response(&response_text)?
    } else {
        serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Invalid JSON response: {}. Response was: {}", e, &response_text[..response_text.len().min(200)]))?
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
    let resources_supported = capabilities.and_then(|c: &Value| c.get("resources")).is_some();
    let prompts_supported = capabilities.and_then(|c: &Value| c.get("prompts")).is_some();

    // Step 2: Send initialized notification (some servers may require this)
    let notify_body = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "method": "initialized"
    }))?;
    info!("[MCP Client] Sending initialized notification with session: {:?}", session_id);

    let _ = build_http_request(&client, url, notify_body, session_id.as_deref(), headers).send();

    // Step 3: List tools
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": next_request_id(),
        "method": "tools/list",
        "params": {}
    });
    let tools_body = serde_json::to_string(&tools_request)?;
    info!("[MCP Client] Sending HTTP tools/list request with session: {:?}", session_id);

    let tools_response = build_http_request(&client, url, tools_body, session_id.as_deref(), headers)
        .send()
        .map_err(|e| anyhow!("HTTP tools/list request failed: {}", e))?;

    let tools_content_type = tools_response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let tools_text = tools_response.text().map_err(|e| anyhow!("Failed to read tools response: {}", e))?;
    info!("[MCP Client] Tools response: {}", &tools_text[..tools_text.len().min(500)]);

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
    }

    #[test]
    fn test_mcp_test_result_error() {
        let result = McpTestResult::error("Test error".to_string(), 50);
        assert!(!result.success);
        assert_eq!(result.error, Some("Test error".to_string()));
    }
}
