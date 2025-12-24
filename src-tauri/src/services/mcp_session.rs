//! MCP Session Manager
//!
//! Manages persistent MCP connections for interactive tool execution.
//! Sessions are kept alive to allow multiple tool calls without re-initializing.

use anyhow::{anyhow, Result};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

use super::mcp_client::{
    HttpMcpClient, McpServerInfo, McpTool, SseMcpClient, StdioMcpClient, StreamableHttpMcpClient,
    ToolCallResult,
};

// ============================================================================
// Session Types
// ============================================================================

/// Information about an active session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: String,
    pub mcp_id: i64,
    pub mcp_name: String,
    pub mcp_type: String,
    pub server_info: Option<McpServerInfo>,
    pub tool_count: usize,
    pub created_at: String,
    pub last_used_at: String,
}

/// Result of starting a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResult {
    pub session_id: String,
    pub server_info: Option<McpServerInfo>,
    pub tools: Vec<McpTool>,
    pub resources_supported: bool,
    pub prompts_supported: bool,
}

// ============================================================================
// Session Wrapper
// ============================================================================

/// Wrapper for different MCP session types
enum McpSession {
    Stdio(StdioSession),
    Http(HttpSession),
    Sse(SseSession),
    StreamableHttp(StreamableHttpSession),
}

struct StdioSession {
    mcp_id: i64,
    mcp_name: String,
    client: StdioMcpClient,
    created_at: Instant,
    last_used_at: Instant,
}

struct HttpSession {
    mcp_id: i64,
    mcp_name: String,
    client: HttpMcpClient,
    created_at: Instant,
    last_used_at: Instant,
}

struct SseSession {
    mcp_id: i64,
    mcp_name: String,
    client: SseMcpClient,
    created_at: Instant,
    last_used_at: Instant,
}

struct StreamableHttpSession {
    mcp_id: i64,
    mcp_name: String,
    client: StreamableHttpMcpClient,
    created_at: Instant,
    last_used_at: Instant,
}

impl McpSession {
    fn mcp_id(&self) -> i64 {
        match self {
            McpSession::Stdio(s) => s.mcp_id,
            McpSession::Http(s) => s.mcp_id,
            McpSession::Sse(s) => s.mcp_id,
            McpSession::StreamableHttp(s) => s.mcp_id,
        }
    }

    fn mcp_name(&self) -> &str {
        match self {
            McpSession::Stdio(s) => &s.mcp_name,
            McpSession::Http(s) => &s.mcp_name,
            McpSession::Sse(s) => &s.mcp_name,
            McpSession::StreamableHttp(s) => &s.mcp_name,
        }
    }

    fn mcp_type(&self) -> &str {
        match self {
            McpSession::Stdio(_) => "stdio",
            McpSession::Http(_) => "http",
            McpSession::Sse(_) => "sse",
            McpSession::StreamableHttp(_) => "streamable_http",
        }
    }

    fn server_info(&self) -> Option<&McpServerInfo> {
        match self {
            McpSession::Stdio(s) => s.client.server_info(),
            McpSession::Http(s) => s.client.server_info(),
            McpSession::Sse(s) => s.client.server_info(),
            McpSession::StreamableHttp(s) => s.client.server_info(),
        }
    }

    fn tools(&self) -> &[McpTool] {
        match self {
            McpSession::Stdio(s) => s.client.tools(),
            McpSession::Http(s) => s.client.tools(),
            McpSession::Sse(s) => s.client.tools(),
            McpSession::StreamableHttp(s) => s.client.tools(),
        }
    }

    fn resources_supported(&self) -> bool {
        match self {
            McpSession::Stdio(s) => s.client.resources_supported(),
            McpSession::Http(s) => s.client.resources_supported(),
            McpSession::Sse(s) => s.client.resources_supported(),
            McpSession::StreamableHttp(s) => s.client.resources_supported(),
        }
    }

    fn prompts_supported(&self) -> bool {
        match self {
            McpSession::Stdio(s) => s.client.prompts_supported(),
            McpSession::Http(s) => s.client.prompts_supported(),
            McpSession::Sse(s) => s.client.prompts_supported(),
            McpSession::StreamableHttp(s) => s.client.prompts_supported(),
        }
    }

    fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        match self {
            McpSession::Stdio(s) => {
                s.last_used_at = Instant::now();
                s.client.call_tool(name, arguments)
            }
            McpSession::Http(s) => {
                s.last_used_at = Instant::now();
                s.client.call_tool(name, arguments)
            }
            McpSession::Sse(s) => {
                s.last_used_at = Instant::now();
                s.client.call_tool(name, arguments)
            }
            McpSession::StreamableHttp(s) => {
                s.last_used_at = Instant::now();
                s.client.call_tool(name, arguments)
            }
        }
    }

    fn created_at(&self) -> Instant {
        match self {
            McpSession::Stdio(s) => s.created_at,
            McpSession::Http(s) => s.created_at,
            McpSession::Sse(s) => s.created_at,
            McpSession::StreamableHttp(s) => s.created_at,
        }
    }

    fn last_used_at(&self) -> Instant {
        match self {
            McpSession::Stdio(s) => s.last_used_at,
            McpSession::Http(s) => s.last_used_at,
            McpSession::Sse(s) => s.last_used_at,
            McpSession::StreamableHttp(s) => s.last_used_at,
        }
    }

    fn close(self) {
        match self {
            McpSession::Stdio(s) => s.client.close(),
            McpSession::Http(s) => s.client.close(),
            McpSession::Sse(s) => s.client.close(),
            McpSession::StreamableHttp(s) => s.client.close(),
        }
    }
}

// ============================================================================
// Session Manager
// ============================================================================

/// Manages multiple active MCP sessions
pub struct McpSessionManager {
    sessions: Arc<Mutex<HashMap<String, McpSession>>>,
}

impl McpSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a new stdio-based MCP session
    pub fn start_stdio_session(
        &self,
        mcp_id: i64,
        mcp_name: &str,
        command: &str,
        args: &[String],
        env: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<StartSessionResult> {
        info!(
            "[Session Manager] Starting stdio session for MCP {} ({})",
            mcp_id, mcp_name
        );

        // Spawn and initialize the client
        let client = StdioMcpClient::spawn(command, args, env, timeout_secs)?;

        let session_id = Uuid::new_v4().to_string();
        let now = Instant::now();

        let result = StartSessionResult {
            session_id: session_id.clone(),
            server_info: client.server_info().cloned(),
            tools: client.tools().to_vec(),
            resources_supported: client.resources_supported(),
            prompts_supported: client.prompts_supported(),
        };

        let session = McpSession::Stdio(StdioSession {
            mcp_id,
            mcp_name: mcp_name.to_string(),
            client,
            created_at: now,
            last_used_at: now,
        });

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        info!(
            "[Session Manager] Session {} started with {} tools",
            session_id,
            result.tools.len()
        );

        Ok(result)
    }

    /// Start a new HTTP-based MCP session
    pub fn start_http_session(
        &self,
        mcp_id: i64,
        mcp_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<StartSessionResult> {
        info!(
            "[Session Manager] Starting HTTP session for MCP {} ({})",
            mcp_id, mcp_name
        );

        // Connect and initialize the client
        let client = HttpMcpClient::connect(url, headers, timeout_secs)?;

        let session_id = Uuid::new_v4().to_string();
        let now = Instant::now();

        let result = StartSessionResult {
            session_id: session_id.clone(),
            server_info: client.server_info().cloned(),
            tools: client.tools().to_vec(),
            resources_supported: client.resources_supported(),
            prompts_supported: client.prompts_supported(),
        };

        let session = McpSession::Http(HttpSession {
            mcp_id,
            mcp_name: mcp_name.to_string(),
            client,
            created_at: now,
            last_used_at: now,
        });

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        info!(
            "[Session Manager] HTTP session {} started with {} tools",
            session_id,
            result.tools.len()
        );

        Ok(result)
    }

    /// Start a new SSE-based MCP session
    pub fn start_sse_session(
        &self,
        mcp_id: i64,
        mcp_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<StartSessionResult> {
        info!(
            "[Session Manager] Starting SSE session for MCP {} ({})",
            mcp_id, mcp_name
        );

        // Connect and initialize the client
        let client = SseMcpClient::connect(url, headers, timeout_secs)?;

        let session_id = Uuid::new_v4().to_string();
        let now = Instant::now();

        let result = StartSessionResult {
            session_id: session_id.clone(),
            server_info: client.server_info().cloned(),
            tools: client.tools().to_vec(),
            resources_supported: client.resources_supported(),
            prompts_supported: client.prompts_supported(),
        };

        let session = McpSession::Sse(SseSession {
            mcp_id,
            mcp_name: mcp_name.to_string(),
            client,
            created_at: now,
            last_used_at: now,
        });

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        info!(
            "[Session Manager] SSE session {} started with {} tools",
            session_id,
            result.tools.len()
        );

        Ok(result)
    }

    /// Start a new Streamable HTTP-based MCP session (for system MCPs like Tool Manager and Gateway)
    pub fn start_streamable_http_session(
        &self,
        mcp_id: i64,
        mcp_name: &str,
        url: &str,
        headers: Option<&HashMap<String, String>>,
        timeout_secs: u64,
    ) -> Result<StartSessionResult> {
        info!(
            "[Session Manager] Starting Streamable HTTP session for MCP {} ({})",
            mcp_id, mcp_name
        );

        // Connect and initialize the client
        let client = StreamableHttpMcpClient::connect(url, headers, timeout_secs)?;

        let session_id = Uuid::new_v4().to_string();
        let now = Instant::now();

        let result = StartSessionResult {
            session_id: session_id.clone(),
            server_info: client.server_info().cloned(),
            tools: client.tools().to_vec(),
            resources_supported: client.resources_supported(),
            prompts_supported: client.prompts_supported(),
        };

        let session = McpSession::StreamableHttp(StreamableHttpSession {
            mcp_id,
            mcp_name: mcp_name.to_string(),
            client,
            created_at: now,
            last_used_at: now,
        });

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        info!(
            "[Session Manager] Streamable HTTP session {} started with {} tools",
            session_id,
            result.tools.len()
        );

        Ok(result)
    }

    /// Execute a tool in an existing session
    pub fn call_tool(
        &self,
        session_id: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<ToolCallResult> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        info!(
            "[Session Manager] Calling tool '{}' in session {}",
            tool_name, session_id
        );

        session.call_tool(tool_name, arguments)
    }

    /// Get information about a specific session
    pub fn get_session_info(&self, session_id: &str) -> Option<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .get(session_id)
            .map(|s| self.session_to_info(session_id, s))
    }

    /// List all active sessions
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .iter()
            .map(|(id, session)| self.session_to_info(id, session))
            .collect()
    }

    /// End a session and clean up resources
    pub fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.remove(session_id) {
            info!("[Session Manager] Ending session {}", session_id);
            session.close();
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Clean up idle sessions (older than the specified duration in seconds)
    pub fn cleanup_idle_sessions(&self, max_idle_secs: u64) -> usize {
        let mut sessions = self.sessions.lock().unwrap();
        let now = Instant::now();
        let threshold = std::time::Duration::from_secs(max_idle_secs);

        let to_remove: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| now.duration_since(session.last_used_at()) > threshold)
            .map(|(id, _)| id.clone())
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            if let Some(session) = sessions.remove(&id) {
                info!("[Session Manager] Cleaning up idle session {}", id);
                session.close();
            }
        }

        count
    }

    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    /// Check if a session exists
    pub fn has_session(&self, session_id: &str) -> bool {
        self.sessions.lock().unwrap().contains_key(session_id)
    }

    /// Get tools for a session
    pub fn get_session_tools(&self, session_id: &str) -> Option<Vec<McpTool>> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| s.tools().to_vec())
    }

    fn session_to_info(&self, id: &str, session: &McpSession) -> SessionInfo {
        let created = session.created_at();
        let last_used = session.last_used_at();

        SessionInfo {
            id: id.to_string(),
            mcp_id: session.mcp_id(),
            mcp_name: session.mcp_name().to_string(),
            mcp_type: session.mcp_type().to_string(),
            server_info: session.server_info().cloned(),
            tool_count: session.tools().len(),
            created_at: format!("{:?}", created.elapsed()),
            last_used_at: format!("{:?}", last_used.elapsed()),
        }
    }
}

impl Default for McpSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_new() {
        let manager = McpSessionManager::new();
        assert_eq!(manager.session_count(), 0);
        assert!(manager.list_sessions().is_empty());
    }

    #[test]
    fn test_session_manager_has_session() {
        let manager = McpSessionManager::new();
        assert!(!manager.has_session("nonexistent"));
    }

    #[test]
    fn test_session_manager_get_nonexistent() {
        let manager = McpSessionManager::new();
        assert!(manager.get_session_info("nonexistent").is_none());
        assert!(manager.get_session_tools("nonexistent").is_none());
    }

    #[test]
    fn test_session_manager_end_nonexistent() {
        let manager = McpSessionManager::new();
        let result = manager.end_session("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_manager_call_tool_nonexistent() {
        let manager = McpSessionManager::new();
        let result = manager.call_tool("nonexistent", "test", serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: "test-id".to_string(),
            mcp_id: 1,
            mcp_name: "Test MCP".to_string(),
            mcp_type: "stdio".to_string(),
            server_info: Some(McpServerInfo {
                name: "Test Server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            tool_count: 5,
            created_at: "0s".to_string(),
            last_used_at: "0s".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"mcpId\":1"));
        assert!(json.contains("\"toolCount\":5"));
    }

    #[test]
    fn test_start_session_result_serialization() {
        let result = StartSessionResult {
            session_id: "session-123".to_string(),
            server_info: None,
            tools: vec![],
            resources_supported: true,
            prompts_supported: false,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"sessionId\":\"session-123\""));
        assert!(json.contains("\"resourcesSupported\":true"));
        assert!(json.contains("\"promptsSupported\":false"));
    }
}
