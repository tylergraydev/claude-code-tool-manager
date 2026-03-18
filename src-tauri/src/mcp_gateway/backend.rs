//! Gateway Backend Manager
//!
//! Manages connections to backend MCP servers and aggregates their tools.

use crate::db::models::{GatewayMcp, Mcp};
use crate::db::Database;
use crate::services::mcp_client::{McpServerInfo, McpTool, StdioMcpClient, ToolCallResult};
use anyhow::{anyhow, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Status of a backend MCP connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackendStatus {
    Connecting,
    Connected,
    Disconnected,
    Failed(String),
    Restarting,
}

/// Information about a connected backend for status reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendInfo {
    pub mcp_id: i64,
    pub mcp_name: String,
    pub mcp_type: String,
    pub status: BackendStatus,
    pub tool_count: usize,
    pub server_info: Option<McpServerInfo>,
    pub error_message: Option<String>,
    pub restart_count: u32,
}

/// Metadata about an available MCP (for lazy loading - no connection required)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableMcp {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub mcp_type: String,
    pub status: BackendStatus,
}

/// Mapping from namespaced tool name to original tool info
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolMapping {
    pub mcp_id: i64,
    pub mcp_name: String,
    pub original_name: String,
    pub tool: McpTool,
}

/// Backend connection wrapping an MCP client
pub struct BackendConnection {
    pub mcp: Mcp,
    pub status: BackendStatus,
    pub client: Option<StdioMcpClient>,
    pub tools: Vec<McpTool>,
    pub server_info: Option<McpServerInfo>,
    pub restart_count: u32,
}

impl BackendConnection {
    pub fn new(mcp: Mcp) -> Self {
        Self {
            mcp,
            status: BackendStatus::Disconnected,
            client: None,
            tools: Vec::new(),
            server_info: None,
            restart_count: 0,
        }
    }

    pub fn to_info(&self) -> BackendInfo {
        BackendInfo {
            mcp_id: self.mcp.id,
            mcp_name: self.mcp.name.clone(),
            mcp_type: self.mcp.mcp_type.clone(),
            status: self.status.clone(),
            tool_count: self.tools.len(),
            server_info: self.server_info.clone(),
            error_message: match &self.status {
                BackendStatus::Failed(msg) => Some(msg.clone()),
                _ => None,
            },
            restart_count: self.restart_count,
        }
    }
}

/// Gateway Backend Manager
///
/// Manages all backend MCP connections with lazy loading.
/// MCPs are only connected when explicitly requested via load_mcp_tools or call_mcp_tool.
pub struct GatewayBackendManager {
    /// Available MCPs loaded from database (not connected)
    available_mcps: Vec<AvailableMcp>,
    /// Active backend connections (lazily created)
    backends: HashMap<i64, BackendConnection>,
    /// Tool index for connected backends
    tool_index: HashMap<String, ToolMapping>,
    db: Arc<Mutex<Database>>,
}

impl GatewayBackendManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            available_mcps: Vec::new(),
            backends: HashMap::new(),
            tool_index: HashMap::new(),
            db,
        }
    }

    /// Create a namespaced tool name from MCP name and original tool name
    pub fn namespace_tool(mcp_name: &str, tool_name: &str) -> String {
        // Sanitize MCP name: replace non-alphanumeric with underscore
        let safe_mcp_name: String = mcp_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        format!("{}__{}", safe_mcp_name, tool_name)
    }

    /// Load available MCPs from database (no connections made - lazy loading)
    pub fn load_available_mcps(&mut self) -> Result<()> {
        let gateway_mcps = {
            let db = self
                .db
                .lock()
                .map_err(|e| anyhow!("Failed to lock database: {}", e))?;
            db.get_enabled_gateway_mcps()?
        };

        info!(
            "[Gateway] Loaded {} available MCPs (lazy mode - no connections yet)",
            gateway_mcps.len()
        );

        self.available_mcps = gateway_mcps
            .into_iter()
            .map(|gm| AvailableMcp {
                id: gm.mcp.id,
                name: gm.mcp.name,
                description: gm.mcp.description,
                mcp_type: gm.mcp.mcp_type,
                status: BackendStatus::Disconnected,
            })
            .collect();

        Ok(())
    }

    /// Get list of available MCPs (for list_available_mcps meta-tool)
    pub fn get_available_mcps(&self) -> Vec<AvailableMcp> {
        self.available_mcps
            .iter()
            .map(|mcp| {
                // Update status based on backend connection state
                let status = self
                    .backends
                    .values()
                    .find(|b| b.mcp.id == mcp.id)
                    .map(|b| b.status.clone())
                    .unwrap_or(BackendStatus::Disconnected);

                AvailableMcp {
                    status,
                    ..mcp.clone()
                }
            })
            .collect()
    }

    /// Connect to an MCP lazily by name (for load_mcp_tools meta-tool)
    pub async fn connect_backend_lazy(&mut self, mcp_name: &str) -> Result<Vec<McpTool>> {
        // Find the MCP in available_mcps
        let mcp_meta = self
            .available_mcps
            .iter()
            .find(|m| m.name == mcp_name)
            .ok_or_else(|| anyhow!("MCP '{}' not found in gateway", mcp_name))?
            .clone();

        // Check if already connected
        if let Some(backend) = self.backends.get(&mcp_meta.id) {
            if matches!(backend.status, BackendStatus::Connected) {
                info!(
                    "[Gateway] MCP '{}' already connected, returning cached tools",
                    mcp_name
                );
                return Ok(backend.tools.clone());
            }
        }

        // Load full MCP config from database
        let gateway_mcp = {
            let db = self
                .db
                .lock()
                .map_err(|e| anyhow!("Failed to lock database: {}", e))?;
            db.get_gateway_mcps()?
                .into_iter()
                .find(|gm| gm.mcp.id == mcp_meta.id)
                .ok_or_else(|| anyhow!("MCP '{}' not found in gateway database", mcp_name))?
        };

        info!("[Gateway] Lazy-connecting to MCP '{}'", mcp_name);

        // Connect to the backend
        self.add_backend(gateway_mcp).await;
        self.build_tool_index();

        // Return the tools
        self.backends
            .get(&mcp_meta.id)
            .map(|b| b.tools.clone())
            .ok_or_else(|| anyhow!("Failed to connect to MCP '{}'", mcp_name))
    }

    /// Get tools for a specific MCP (returns None if not connected)
    #[allow(dead_code)]
    pub fn get_backend_tools(&self, mcp_name: &str) -> Option<Vec<McpTool>> {
        self.backends
            .values()
            .find(|b| b.mcp.name == mcp_name && matches!(b.status, BackendStatus::Connected))
            .map(|b| b.tools.clone())
    }

    /// Add a backend connection for an MCP
    async fn add_backend(&mut self, gateway_mcp: GatewayMcp) {
        let mcp_id = gateway_mcp.mcp.id;
        let mcp_name = gateway_mcp.mcp.name.clone();
        let mcp_type = gateway_mcp.mcp.mcp_type.clone();

        info!("[Gateway] Adding backend: {} ({})", mcp_name, mcp_type);

        let mut backend = BackendConnection::new(gateway_mcp.mcp.clone());

        // Only support stdio MCPs for now (HTTP/SSE would need different client handling)
        if mcp_type == "stdio" {
            backend.status = BackendStatus::Connecting;

            match self.connect_stdio_backend(&gateway_mcp.mcp).await {
                Ok((client, server_info, tools)) => {
                    info!(
                        "[Gateway] Connected to {} with {} tools",
                        mcp_name,
                        tools.len()
                    );
                    backend.client = Some(client);
                    backend.server_info = Some(server_info);
                    backend.tools = tools;
                    backend.status = BackendStatus::Connected;
                }
                Err(e) => {
                    error!("[Gateway] Failed to connect to {}: {}", mcp_name, e);
                    backend.status = BackendStatus::Failed(e.to_string());
                }
            }
        } else {
            // HTTP/SSE MCPs are not supported for gateway proxying yet
            warn!(
                "[Gateway] Skipping {} - only stdio MCPs are supported for gateway",
                mcp_name
            );
            backend.status = BackendStatus::Failed(
                "Only stdio MCPs are supported for gateway proxying".to_string(),
            );
        }

        self.backends.insert(mcp_id, backend);
    }

    /// Connect to a stdio-based MCP
    async fn connect_stdio_backend(
        &self,
        mcp: &Mcp,
    ) -> Result<(StdioMcpClient, McpServerInfo, Vec<McpTool>)> {
        let command = mcp
            .command
            .as_ref()
            .ok_or_else(|| anyhow!("STDIO MCP requires a command"))?;

        let args: Vec<String> = mcp.args.clone().unwrap_or_default();
        let env = mcp.env.clone();

        info!("[Gateway] Starting stdio MCP: {} {:?}", command, args);

        // Spawn and initialize the client (spawn calls initialize internally)
        let client = StdioMcpClient::spawn(command, &args, env.as_ref(), 30)?;

        // Get server info and tools from the initialized client
        let server_info = client
            .server_info()
            .cloned()
            .unwrap_or_else(|| McpServerInfo {
                name: mcp.name.clone(),
                version: None,
            });
        let tools = client.tools().to_vec();

        Ok((client, server_info, tools))
    }

    /// Build the aggregated tool index from all connected backends
    fn build_tool_index(&mut self) {
        self.tool_index.clear();

        for (mcp_id, backend) in &self.backends {
            if matches!(backend.status, BackendStatus::Connected) {
                for tool in &backend.tools {
                    let namespaced = Self::namespace_tool(&backend.mcp.name, &tool.name);
                    self.tool_index.insert(
                        namespaced,
                        ToolMapping {
                            mcp_id: *mcp_id,
                            mcp_name: backend.mcp.name.clone(),
                            original_name: tool.name.clone(),
                            tool: tool.clone(),
                        },
                    );
                }
            }
        }
    }

    /// Get all aggregated tools with namespaced names
    #[allow(dead_code)]
    pub fn get_tools(&self) -> Vec<McpTool> {
        self.tool_index
            .values()
            .map(|mapping| {
                let mut tool = mapping.tool.clone();
                // Update the name to the namespaced version
                tool.name = Self::namespace_tool(&mapping.mcp_name, &mapping.original_name);
                // Prepend MCP name to description
                if let Some(desc) = &tool.description {
                    tool.description = Some(format!("[{}] {}", mapping.mcp_name, desc));
                } else {
                    tool.description = Some(format!("[{}]", mapping.mcp_name));
                }
                tool
            })
            .collect()
    }

    /// Call a tool on a specific MCP by name (for call_mcp_tool meta-tool)
    /// This is the primary method for lazy-loading mode
    pub fn call_tool_on_mcp(
        &mut self,
        mcp_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolCallResult> {
        // Find the backend by MCP name
        let backend = self
            .backends
            .values_mut()
            .find(|b| b.mcp.name == mcp_name)
            .ok_or_else(|| {
                anyhow!(
                    "MCP '{}' is not connected. Call load_mcp_tools first to connect.",
                    mcp_name
                )
            })?;

        if !matches!(backend.status, BackendStatus::Connected) {
            return Err(anyhow!(
                "MCP '{}' is not connected (status: {:?}). Call load_mcp_tools first.",
                mcp_name,
                backend.status
            ));
        }

        let client = backend
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("MCP '{}' has no active client", mcp_name))?;

        info!(
            "[Gateway] Calling tool '{}' on MCP '{}'",
            tool_name, mcp_name
        );

        client.call_tool(tool_name, arguments)
    }

    /// Call a tool on the appropriate backend (legacy method for namespaced tools)
    #[allow(dead_code)]
    pub fn call_tool(
        &mut self,
        namespaced_name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolCallResult> {
        let mapping = self
            .tool_index
            .get(namespaced_name)
            .ok_or_else(|| anyhow!("Unknown tool: {}", namespaced_name))?
            .clone();

        let backend = self
            .backends
            .get_mut(&mapping.mcp_id)
            .ok_or_else(|| anyhow!("Backend not found for MCP {}", mapping.mcp_name))?;

        if !matches!(backend.status, BackendStatus::Connected) {
            return Err(anyhow!(
                "Backend {} is not connected (status: {:?})",
                mapping.mcp_name,
                backend.status
            ));
        }

        let client = backend
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("Backend {} has no active client", mapping.mcp_name))?;

        info!(
            "[Gateway] Calling tool {} on backend {}",
            mapping.original_name, mapping.mcp_name
        );

        client.call_tool(&mapping.original_name, arguments)
    }

    /// Get status of all backends
    pub fn get_backends_info(&self) -> Vec<BackendInfo> {
        self.backends.values().map(|b| b.to_info()).collect()
    }

    /// Get the total number of aggregated tools
    pub fn tool_count(&self) -> usize {
        self.tool_index.len()
    }

    /// Shutdown all backend connections
    pub fn shutdown(&mut self) {
        info!("[Gateway] Shutting down all backend connections");
        for (_mcp_id, backend) in self.backends.iter_mut() {
            if let Some(client) = backend.client.take() {
                info!("[Gateway] Closing connection to MCP {}", backend.mcp.name);
                drop(client);
            }
            backend.status = BackendStatus::Disconnected;
        }
        self.tool_index.clear();
    }

    /// Restart a specific backend
    pub async fn restart_backend(&mut self, mcp_id: i64) -> Result<BackendInfo> {
        let gateway_mcp = {
            let db = self
                .db
                .lock()
                .map_err(|e| anyhow!("Failed to lock database: {}", e))?;
            db.get_gateway_mcps()?
                .into_iter()
                .find(|gm| gm.mcp_id == mcp_id)
                .ok_or_else(|| anyhow!("MCP {} not found in gateway", mcp_id))?
        };

        // Remove old backend if exists
        if let Some(mut backend) = self.backends.remove(&mcp_id) {
            if let Some(client) = backend.client.take() {
                drop(client);
            }
        }

        // Re-add the backend
        self.add_backend(gateway_mcp).await;
        self.build_tool_index();

        self.backends
            .get(&mcp_id)
            .map(|b| b.to_info())
            .ok_or_else(|| anyhow!("Failed to restart backend"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mcp_client::McpServerInfo;

    /// Helper to create a test Mcp struct
    fn make_test_mcp(id: i64, name: &str, mcp_type: &str) -> Mcp {
        Mcp {
            id,
            name: name.to_string(),
            description: Some(format!("Test MCP {}", name)),
            mcp_type: mcp_type.to_string(),
            command: Some("echo".to_string()),
            args: Some(vec!["hello".to_string()]),
            url: None,
            headers: None,
            env: None,
            icon: None,
            tags: None,
            source: "library".to_string(),
            source_path: None,
            is_enabled_global: true,
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    /// Helper to create an McpTool
    fn make_test_tool(name: &str, desc: Option<&str>) -> McpTool {
        McpTool {
            name: name.to_string(),
            description: desc.map(|s| s.to_string()),
            input_schema: None,
        }
    }

    // ===== BackendConnection tests =====

    #[test]
    fn test_backend_connection_new() {
        let mcp = make_test_mcp(1, "test-server", "stdio");
        let conn = BackendConnection::new(mcp.clone());

        assert_eq!(conn.mcp.id, 1);
        assert_eq!(conn.mcp.name, "test-server");
        assert_eq!(conn.status, BackendStatus::Disconnected);
        assert!(conn.client.is_none());
        assert!(conn.tools.is_empty());
        assert!(conn.server_info.is_none());
        assert_eq!(conn.restart_count, 0);
    }

    #[test]
    fn test_backend_connection_to_info_disconnected() {
        let mcp = make_test_mcp(42, "my-mcp", "stdio");
        let conn = BackendConnection::new(mcp);
        let info = conn.to_info();

        assert_eq!(info.mcp_id, 42);
        assert_eq!(info.mcp_name, "my-mcp");
        assert_eq!(info.mcp_type, "stdio");
        assert_eq!(info.status, BackendStatus::Disconnected);
        assert_eq!(info.tool_count, 0);
        assert!(info.server_info.is_none());
        assert!(info.error_message.is_none());
        assert_eq!(info.restart_count, 0);
    }

    #[test]
    fn test_backend_connection_to_info_failed() {
        let mcp = make_test_mcp(1, "broken", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Failed("connection refused".to_string());
        conn.restart_count = 3;

        let info = conn.to_info();
        assert_eq!(info.status, BackendStatus::Failed("connection refused".to_string()));
        assert_eq!(info.error_message, Some("connection refused".to_string()));
        assert_eq!(info.restart_count, 3);
    }

    #[test]
    fn test_backend_connection_to_info_connected_with_tools() {
        let mcp = make_test_mcp(5, "toolbox", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Connected;
        conn.tools = vec![
            make_test_tool("read", Some("Read a file")),
            make_test_tool("write", Some("Write a file")),
        ];
        conn.server_info = Some(McpServerInfo {
            name: "toolbox".to_string(),
            version: Some("1.0.0".to_string()),
        });

        let info = conn.to_info();
        assert_eq!(info.tool_count, 2);
        assert!(info.server_info.is_some());
        assert_eq!(info.server_info.unwrap().version, Some("1.0.0".to_string()));
        assert!(info.error_message.is_none());
    }

    #[test]
    fn test_backend_connection_to_info_non_failed_statuses_have_no_error() {
        for status in [
            BackendStatus::Connecting,
            BackendStatus::Connected,
            BackendStatus::Disconnected,
            BackendStatus::Restarting,
        ] {
            let mcp = make_test_mcp(1, "test", "stdio");
            let mut conn = BackendConnection::new(mcp);
            conn.status = status;
            let info = conn.to_info();
            assert!(info.error_message.is_none(), "Status {:?} should not have error_message", info.status);
        }
    }

    // ===== BackendStatus tests =====

    #[test]
    fn test_backend_status_serialize_deserialize() {
        let cases = vec![
            (BackendStatus::Connecting, r#""connecting"#),
            (BackendStatus::Connected, r#""connected"#),
            (BackendStatus::Disconnected, r#""disconnected"#),
            (BackendStatus::Restarting, r#""restarting"#),
        ];

        for (status, expected_prefix) in cases {
            let json = serde_json::to_string(&status).unwrap();
            assert!(json.starts_with(expected_prefix), "Serialized {:?} = {}", status, json);
            let round_tripped: BackendStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(round_tripped, status);
        }
    }

    #[test]
    fn test_backend_status_failed_serialize_roundtrip() {
        let status = BackendStatus::Failed("timeout".to_string());
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("timeout"));
        let round_tripped: BackendStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(round_tripped, BackendStatus::Failed("timeout".to_string()));
    }

    #[test]
    fn test_backend_status_equality() {
        assert_eq!(BackendStatus::Connected, BackendStatus::Connected);
        assert_ne!(BackendStatus::Connected, BackendStatus::Disconnected);
        assert_ne!(
            BackendStatus::Failed("a".to_string()),
            BackendStatus::Failed("b".to_string())
        );
    }

    // ===== AvailableMcp tests =====

    #[test]
    fn test_available_mcp_serialize_deserialize() {
        let mcp = AvailableMcp {
            id: 10,
            name: "test-mcp".to_string(),
            description: Some("A test MCP".to_string()),
            mcp_type: "stdio".to_string(),
            status: BackendStatus::Disconnected,
        };

        let json = serde_json::to_string(&mcp).unwrap();
        assert!(json.contains("test-mcp"));
        assert!(json.contains("mcpType")); // camelCase

        let parsed: AvailableMcp = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, 10);
        assert_eq!(parsed.name, "test-mcp");
        assert_eq!(parsed.description, Some("A test MCP".to_string()));
    }

    #[test]
    fn test_available_mcp_with_none_description() {
        let mcp = AvailableMcp {
            id: 1,
            name: "minimal".to_string(),
            description: None,
            mcp_type: "sse".to_string(),
            status: BackendStatus::Connected,
        };
        let json = serde_json::to_string(&mcp).unwrap();
        let parsed: AvailableMcp = serde_json::from_str(&json).unwrap();
        assert!(parsed.description.is_none());
    }

    // ===== BackendInfo tests =====

    #[test]
    fn test_backend_info_serialize_deserialize() {
        let info = BackendInfo {
            mcp_id: 1,
            mcp_name: "server".to_string(),
            mcp_type: "stdio".to_string(),
            status: BackendStatus::Connected,
            tool_count: 5,
            server_info: Some(McpServerInfo {
                name: "server".to_string(),
                version: Some("2.0".to_string()),
            }),
            error_message: None,
            restart_count: 0,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("mcpId")); // camelCase
        assert!(json.contains("toolCount"));

        let parsed: BackendInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mcp_id, 1);
        assert_eq!(parsed.tool_count, 5);
    }

    // ===== namespace_tool tests =====

    #[test]
    fn test_namespace_tool() {
        assert_eq!(
            GatewayBackendManager::namespace_tool("filesystem", "read_file"),
            "filesystem__read_file"
        );
        assert_eq!(
            GatewayBackendManager::namespace_tool("my-mcp", "get_data"),
            "my-mcp__get_data"
        );
        assert_eq!(
            GatewayBackendManager::namespace_tool("MCP with spaces", "tool"),
            "MCP_with_spaces__tool"
        );
    }

    #[test]
    fn test_namespace_tool_special_chars() {
        // Dots, slashes, etc. become underscores
        assert_eq!(
            GatewayBackendManager::namespace_tool("my.mcp/v2", "run"),
            "my_mcp_v2__run"
        );
        // Underscores and hyphens are preserved
        assert_eq!(
            GatewayBackendManager::namespace_tool("my_mcp-name", "tool"),
            "my_mcp-name__tool"
        );
    }

    #[test]
    fn test_namespace_tool_empty_names() {
        assert_eq!(
            GatewayBackendManager::namespace_tool("", "tool"),
            "__tool"
        );
        assert_eq!(
            GatewayBackendManager::namespace_tool("mcp", ""),
            "mcp__"
        );
    }

    // ===== GatewayBackendManager tests =====

    fn make_test_manager() -> GatewayBackendManager {
        let db = Database::in_memory().unwrap();
        let db_arc = Arc::new(Mutex::new(db));
        GatewayBackendManager::new(db_arc)
    }

    #[test]
    fn test_manager_new_is_empty() {
        let manager = make_test_manager();
        assert!(manager.get_available_mcps().is_empty());
        assert!(manager.get_backends_info().is_empty());
        assert_eq!(manager.tool_count(), 0);
        assert!(manager.get_tools().is_empty());
    }

    #[test]
    fn test_manager_get_available_mcps_empty() {
        let manager = make_test_manager();
        let mcps = manager.get_available_mcps();
        assert!(mcps.is_empty());
    }

    #[test]
    fn test_manager_get_available_mcps_reflects_backend_status() {
        let mut manager = make_test_manager();

        // Add an available MCP
        manager.available_mcps.push(AvailableMcp {
            id: 1,
            name: "test-mcp".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            status: BackendStatus::Disconnected,
        });

        // Without a backend, status should be Disconnected
        let mcps = manager.get_available_mcps();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].status, BackendStatus::Disconnected);

        // Add a connected backend for this MCP
        let mcp = make_test_mcp(1, "test-mcp", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Connected;
        manager.backends.insert(1, conn);

        let mcps = manager.get_available_mcps();
        assert_eq!(mcps[0].status, BackendStatus::Connected);
    }

    #[test]
    fn test_manager_get_backends_info() {
        let mut manager = make_test_manager();

        let mcp1 = make_test_mcp(1, "server-a", "stdio");
        let mut conn1 = BackendConnection::new(mcp1);
        conn1.status = BackendStatus::Connected;
        conn1.tools = vec![make_test_tool("tool1", None)];

        let mcp2 = make_test_mcp(2, "server-b", "stdio");
        let mut conn2 = BackendConnection::new(mcp2);
        conn2.status = BackendStatus::Failed("err".to_string());

        manager.backends.insert(1, conn1);
        manager.backends.insert(2, conn2);

        let infos = manager.get_backends_info();
        assert_eq!(infos.len(), 2);
    }

    #[test]
    fn test_manager_tool_count_empty() {
        let manager = make_test_manager();
        assert_eq!(manager.tool_count(), 0);
    }

    #[test]
    fn test_manager_build_tool_index_and_get_tools() {
        let mut manager = make_test_manager();

        // Add a connected backend with tools
        let mcp = make_test_mcp(1, "fs", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Connected;
        conn.tools = vec![
            make_test_tool("read_file", Some("Read a file")),
            make_test_tool("write_file", None),
        ];
        manager.backends.insert(1, conn);

        // Build the tool index
        manager.build_tool_index();

        assert_eq!(manager.tool_count(), 2);

        let tools = manager.get_tools();
        assert_eq!(tools.len(), 2);

        // Check namespaced names
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"fs__read_file"));
        assert!(names.contains(&"fs__write_file"));

        // Check description prepend
        let read_tool = tools.iter().find(|t| t.name == "fs__read_file").unwrap();
        assert_eq!(read_tool.description, Some("[fs] Read a file".to_string()));

        let write_tool = tools.iter().find(|t| t.name == "fs__write_file").unwrap();
        assert_eq!(write_tool.description, Some("[fs]".to_string()));
    }

    #[test]
    fn test_manager_build_tool_index_skips_disconnected() {
        let mut manager = make_test_manager();

        let mcp = make_test_mcp(1, "offline", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Disconnected;
        conn.tools = vec![make_test_tool("tool1", None)];
        manager.backends.insert(1, conn);

        manager.build_tool_index();
        assert_eq!(manager.tool_count(), 0);
    }

    #[test]
    fn test_manager_build_tool_index_skips_failed() {
        let mut manager = make_test_manager();

        let mcp = make_test_mcp(1, "broken", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Failed("error".to_string());
        conn.tools = vec![make_test_tool("tool1", None)];
        manager.backends.insert(1, conn);

        manager.build_tool_index();
        assert_eq!(manager.tool_count(), 0);
    }

    #[test]
    fn test_manager_build_tool_index_multiple_backends() {
        let mut manager = make_test_manager();

        // Backend 1: connected with 2 tools
        let mcp1 = make_test_mcp(1, "fs", "stdio");
        let mut conn1 = BackendConnection::new(mcp1);
        conn1.status = BackendStatus::Connected;
        conn1.tools = vec![
            make_test_tool("read", None),
            make_test_tool("write", None),
        ];
        manager.backends.insert(1, conn1);

        // Backend 2: connected with 1 tool
        let mcp2 = make_test_mcp(2, "db", "stdio");
        let mut conn2 = BackendConnection::new(mcp2);
        conn2.status = BackendStatus::Connected;
        conn2.tools = vec![make_test_tool("query", Some("Run a query"))];
        manager.backends.insert(2, conn2);

        manager.build_tool_index();
        assert_eq!(manager.tool_count(), 3);

        let tools = manager.get_tools();
        let names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"fs__read".to_string()));
        assert!(names.contains(&"fs__write".to_string()));
        assert!(names.contains(&"db__query".to_string()));
    }

    #[test]
    fn test_manager_get_backend_tools_connected() {
        let mut manager = make_test_manager();

        let mcp = make_test_mcp(1, "test-server", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Connected;
        conn.tools = vec![make_test_tool("tool_a", None)];
        manager.backends.insert(1, conn);

        let tools = manager.get_backend_tools("test-server");
        assert!(tools.is_some());
        assert_eq!(tools.unwrap().len(), 1);
    }

    #[test]
    fn test_manager_get_backend_tools_disconnected() {
        let mut manager = make_test_manager();

        let mcp = make_test_mcp(1, "test-server", "stdio");
        let conn = BackendConnection::new(mcp);
        // status is Disconnected by default
        manager.backends.insert(1, conn);

        let tools = manager.get_backend_tools("test-server");
        assert!(tools.is_none());
    }

    #[test]
    fn test_manager_get_backend_tools_not_found() {
        let manager = make_test_manager();
        let tools = manager.get_backend_tools("nonexistent");
        assert!(tools.is_none());
    }

    #[test]
    fn test_manager_shutdown() {
        let mut manager = make_test_manager();

        // Add a connected backend with tools in the index
        let mcp = make_test_mcp(1, "server", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Connected;
        conn.tools = vec![make_test_tool("tool1", None)];
        manager.backends.insert(1, conn);
        manager.build_tool_index();

        assert_eq!(manager.tool_count(), 1);

        manager.shutdown();

        // After shutdown, tool_index should be empty and backends disconnected
        assert_eq!(manager.tool_count(), 0);
        for backend in manager.backends.values() {
            assert_eq!(backend.status, BackendStatus::Disconnected);
            assert!(backend.client.is_none());
        }
    }

    #[test]
    fn test_manager_shutdown_empty() {
        // Shutting down with no backends should not panic
        let mut manager = make_test_manager();
        manager.shutdown();
        assert_eq!(manager.tool_count(), 0);
    }

    #[test]
    fn test_manager_build_tool_index_clears_old() {
        let mut manager = make_test_manager();

        // Add connected backend
        let mcp = make_test_mcp(1, "a", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Connected;
        conn.tools = vec![make_test_tool("t1", None), make_test_tool("t2", None)];
        manager.backends.insert(1, conn);
        manager.build_tool_index();
        assert_eq!(manager.tool_count(), 2);

        // Now disconnect and rebuild
        manager.backends.get_mut(&1).unwrap().status = BackendStatus::Disconnected;
        manager.build_tool_index();
        assert_eq!(manager.tool_count(), 0);
    }

    #[test]
    fn test_manager_call_tool_on_mcp_not_connected() {
        let mut manager = make_test_manager();

        let result = manager.call_tool_on_mcp("nonexistent", "tool", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not connected"));
    }

    #[test]
    fn test_manager_call_tool_on_mcp_disconnected_status() {
        let mut manager = make_test_manager();

        let mcp = make_test_mcp(1, "my-server", "stdio");
        let conn = BackendConnection::new(mcp);
        manager.backends.insert(1, conn);

        let result = manager.call_tool_on_mcp("my-server", "tool", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not connected"));
    }

    #[test]
    fn test_manager_call_tool_on_mcp_connected_no_client() {
        let mut manager = make_test_manager();

        let mcp = make_test_mcp(1, "my-server", "stdio");
        let mut conn = BackendConnection::new(mcp);
        conn.status = BackendStatus::Connected;
        // client is None
        manager.backends.insert(1, conn);

        let result = manager.call_tool_on_mcp("my-server", "tool", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no active client"));
    }

    #[test]
    fn test_manager_call_tool_unknown_namespaced() {
        let mut manager = make_test_manager();

        let result = manager.call_tool("nonexistent__tool", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown tool"));
    }

    #[test]
    fn test_manager_load_available_mcps_empty_db() {
        let mut manager = make_test_manager();
        let result = manager.load_available_mcps();
        assert!(result.is_ok());
        assert!(manager.get_available_mcps().is_empty());
    }
}
