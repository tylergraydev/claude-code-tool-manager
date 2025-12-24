//! MCP Server lifecycle management
//!
//! Handles starting, stopping, and managing the HTTP MCP server.

use crate::db::Database;
use crate::mcp_server::tools::ToolManagerServer;
use axum::Router;
use rmcp::transport::streamable_http_server::{session::local::LocalSessionManager, StreamableHttpService};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};

/// Default port for the MCP server
pub const DEFAULT_MCP_SERVER_PORT: u16 = 23847;

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub auto_start: bool,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: DEFAULT_MCP_SERVER_PORT,
            auto_start: true,
        }
    }
}

/// Status information about the MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerStatus {
    pub is_running: bool,
    pub port: u16,
    pub url: String,
    pub mcp_endpoint: String,
}

/// Handle to control a running MCP server
pub struct McpServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    port: u16,
}

impl McpServerHandle {
    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// MCP server state managed by Tauri
pub struct McpServerState {
    pub is_running: Arc<AtomicBool>,
    pub config: Arc<Mutex<McpServerConfig>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    port: Arc<Mutex<u16>>,
}

impl Default for McpServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServerState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(McpServerConfig::default())),
            shutdown_tx: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(DEFAULT_MCP_SERVER_PORT)),
        }
    }

    pub fn with_config(config: McpServerConfig) -> Self {
        let port = config.port;
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(config)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(port)),
        }
    }

    /// Check if the server is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Get the current port
    pub fn get_port(&self) -> u16 {
        *self.port.lock().unwrap()
    }

    /// Get the server URL
    pub fn get_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.get_port())
    }

    /// Get the MCP endpoint URL
    pub fn get_mcp_endpoint(&self) -> String {
        format!("{}/mcp", self.get_url())
    }

    /// Get current status
    pub fn get_status(&self) -> McpServerStatus {
        McpServerStatus {
            is_running: self.is_running(),
            port: self.get_port(),
            url: self.get_url(),
            mcp_endpoint: self.get_mcp_endpoint(),
        }
    }

    /// Start the MCP server
    pub async fn start(&self, db: Arc<Mutex<Database>>) -> Result<(), String> {
        if self.is_running() {
            return Err("MCP server is already running".to_string());
        }

        let port = {
            let config = self.config.lock().map_err(|e| e.to_string())?;
            config.port
        };

        // Try to bind to the port
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

        log::info!("[McpServer] Starting MCP server on {}", addr);

        // Create the MCP service
        let service = StreamableHttpService::new(
            move || Ok(ToolManagerServer::new(db.clone())),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Set up CORS for flexibility
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Build the router
        let router = Router::new()
            .nest_service("/mcp", service)
            .layer(cors);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        // Store shutdown sender
        {
            let mut tx = self.shutdown_tx.lock().map_err(|e| e.to_string())?;
            *tx = Some(shutdown_tx);
        }

        // Update state
        {
            let mut p = self.port.lock().map_err(|e| e.to_string())?;
            *p = port;
        }
        self.is_running.store(true, Ordering::SeqCst);

        // Spawn the server
        let is_running = self.is_running.clone();
        tokio::spawn(async move {
            let server = axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                    log::info!("[McpServer] Shutdown signal received");
                });

            if let Err(e) = server.await {
                log::error!("[McpServer] Server error: {}", e);
            }

            is_running.store(false, Ordering::SeqCst);
            log::info!("[McpServer] Server stopped");
        });

        log::info!("[McpServer] MCP server started successfully on port {}", port);
        Ok(())
    }

    /// Stop the MCP server
    pub fn stop(&self) -> Result<(), String> {
        if !self.is_running() {
            return Err("MCP server is not running".to_string());
        }

        let shutdown_tx = {
            let mut tx = self.shutdown_tx.lock().map_err(|e| e.to_string())?;
            tx.take()
        };

        if let Some(tx) = shutdown_tx {
            let _ = tx.send(());
            log::info!("[McpServer] Shutdown signal sent");
        }

        Ok(())
    }

    /// Update configuration
    pub fn update_config(&self, new_config: McpServerConfig) -> Result<(), String> {
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        *config = new_config;
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> Result<McpServerConfig, String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;
        Ok(config.clone())
    }

    /// Get connection config JSON for users to add to their Claude config
    pub fn get_connection_config(&self) -> serde_json::Value {
        serde_json::json!({
            "tool-manager": {
                "type": "sse",
                "url": self.get_mcp_endpoint()
            }
        })
    }
}

/// Generate the MCP entry for auto-adding to the library
pub fn generate_self_mcp_entry(port: u16) -> crate::db::models::CreateMcpRequest {
    crate::db::models::CreateMcpRequest {
        name: "Tool Manager MCP".to_string(),
        description: Some("MCP server exposed by Claude Code Tool Manager for programmatic management of MCPs, Skills, Sub-Agents, Hooks, and Projects.".to_string()),
        mcp_type: "http".to_string(),  // Uses HTTP transport (Streamable HTTP)
        command: None,
        args: None,
        url: Some(format!("http://127.0.0.1:{}/mcp", port)),
        headers: None,
        env: None,
        icon: Some("🔧".to_string()),
        tags: Some(vec!["tool-manager".to_string(), "self".to_string(), "management".to_string()]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = McpServerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, DEFAULT_MCP_SERVER_PORT);
        assert!(config.auto_start);
    }

    #[test]
    fn test_server_state_new() {
        let state = McpServerState::new();
        assert!(!state.is_running());
        assert_eq!(state.get_port(), DEFAULT_MCP_SERVER_PORT);
    }

    #[test]
    fn test_get_urls() {
        let state = McpServerState::new();
        assert_eq!(state.get_url(), format!("http://127.0.0.1:{}", DEFAULT_MCP_SERVER_PORT));
        assert_eq!(state.get_mcp_endpoint(), format!("http://127.0.0.1:{}/mcp", DEFAULT_MCP_SERVER_PORT));
    }

    #[test]
    fn test_connection_config() {
        let state = McpServerState::new();
        let config = state.get_connection_config();
        assert!(config.get("tool-manager").is_some());
        assert_eq!(config["tool-manager"]["type"], "sse");
    }

    #[test]
    fn test_generate_self_mcp_entry() {
        let entry = generate_self_mcp_entry(23847);
        assert_eq!(entry.name, "Tool Manager MCP");
        assert_eq!(entry.mcp_type, "http");
        assert!(entry.url.is_some());
        assert!(entry.url.unwrap().contains("23847"));
    }
}
