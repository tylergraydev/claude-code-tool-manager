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

/// Mapping from namespaced tool name to original tool info
#[derive(Debug, Clone)]
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
/// Manages all backend MCP connections and provides a unified tool registry.
pub struct GatewayBackendManager {
    backends: HashMap<i64, BackendConnection>,
    tool_index: HashMap<String, ToolMapping>,
    db: Arc<Mutex<Database>>,
}

impl GatewayBackendManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
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

    /// Load gateway MCPs from database and connect to them
    pub async fn load_and_connect(&mut self) -> Result<()> {
        let gateway_mcps = {
            let db = self
                .db
                .lock()
                .map_err(|e| anyhow!("Failed to lock database: {}", e))?;
            db.get_enabled_gateway_mcps()?
        };

        info!("[Gateway] Loading {} enabled MCPs", gateway_mcps.len());

        for gateway_mcp in gateway_mcps {
            self.add_backend(gateway_mcp).await;
        }

        self.build_tool_index();

        info!(
            "[Gateway] Loaded {} tools from {} backends",
            self.tool_index.len(),
            self.backends
                .values()
                .filter(|b| matches!(b.status, BackendStatus::Connected))
                .count()
        );

        Ok(())
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

    /// Call a tool on the appropriate backend
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
}
