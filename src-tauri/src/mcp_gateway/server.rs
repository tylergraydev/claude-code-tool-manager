//! MCP Gateway Server lifecycle management
//!
//! Handles starting, stopping, and managing the Gateway HTTP server.

use crate::db::Database;
use crate::mcp_gateway::backend::{AvailableMcp, BackendInfo, GatewayBackendManager};
use crate::mcp_gateway::tools::GatewayServer;
use axum::Router;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpService,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};

/// Default port for the MCP Gateway
pub const DEFAULT_GATEWAY_PORT: u16 = 23848;

/// Gateway server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub auto_start: bool,
}

impl Default for GatewayServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: DEFAULT_GATEWAY_PORT,
            auto_start: false,
        }
    }
}

/// Status information about the Gateway server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayServerStatus {
    pub is_running: bool,
    pub port: u16,
    pub url: String,
    pub mcp_endpoint: String,
    /// Available MCPs that can be lazily connected
    pub available_mcps: Vec<AvailableMcp>,
    /// Currently connected backends (lazily loaded)
    pub connected_backends: Vec<BackendInfo>,
    /// Total tools from connected backends
    pub total_tools: usize,
}

/// Gateway server state managed by Tauri
pub struct GatewayServerState {
    pub is_running: Arc<AtomicBool>,
    pub config: Arc<Mutex<GatewayServerConfig>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    port: Arc<Mutex<u16>>,
    pub backend_manager: Arc<tokio::sync::Mutex<GatewayBackendManager>>,
}

impl GatewayServerState {
    pub fn with_config(config: GatewayServerConfig, db: Arc<Mutex<Database>>) -> Self {
        let port = config.port;
        let backend_manager = GatewayBackendManager::new(db.clone());
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(config)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(port)),
            backend_manager: Arc::new(tokio::sync::Mutex::new(backend_manager)),
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

    /// Get current status with backend info (requires async)
    pub async fn get_status(&self) -> GatewayServerStatus {
        let backend_manager = self.backend_manager.lock().await;
        GatewayServerStatus {
            is_running: self.is_running(),
            port: self.get_port(),
            url: self.get_url(),
            mcp_endpoint: self.get_mcp_endpoint(),
            available_mcps: backend_manager.get_available_mcps(),
            connected_backends: backend_manager.get_backends_info(),
            total_tools: backend_manager.tool_count(),
        }
    }

    /// Start the Gateway server
    pub async fn start(&self) -> Result<(), String> {
        if self.is_running() {
            return Err("Gateway server is already running".to_string());
        }

        let port = {
            let config = self.config.lock().map_err(|e| e.to_string())?;
            config.port
        };

        // Load available MCPs (lazy mode - no connections yet)
        {
            let mut backend_manager = self.backend_manager.lock().await;
            backend_manager
                .load_available_mcps()
                .map_err(|e| e.to_string())?;
        }

        // Try to bind to the port
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

        log::info!("[Gateway] Starting MCP Gateway on {}", addr);

        // Create the gateway MCP service
        let backend_manager = self.backend_manager.clone();
        let service = StreamableHttpService::new(
            move || Ok(GatewayServer::new(backend_manager.clone())),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Set up CORS for flexibility
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Build the router
        let router = Router::new().nest_service("/mcp", service).layer(cors);

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
        let backend_manager_shutdown = self.backend_manager.clone();
        tokio::spawn(async move {
            let server = axum::serve(listener, router).with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
                log::info!("[Gateway] Shutdown signal received");
            });

            if let Err(e) = server.await {
                log::error!("[Gateway] Server error: {}", e);
            }

            // Shutdown backend connections
            {
                let mut manager = backend_manager_shutdown.lock().await;
                manager.shutdown();
            }

            is_running.store(false, Ordering::SeqCst);
            log::info!("[Gateway] Server stopped");
        });

        let status = self.get_status().await;
        log::info!(
            "[Gateway] MCP Gateway started in lazy mode on port {} with {} available MCPs",
            port,
            status.available_mcps.len()
        );
        Ok(())
    }

    /// Stop the Gateway server
    pub async fn stop(&self) -> Result<(), String> {
        if !self.is_running() {
            return Err("Gateway server is not running".to_string());
        }

        let shutdown_tx = {
            let mut tx = self.shutdown_tx.lock().map_err(|e| e.to_string())?;
            tx.take()
        };

        if let Some(tx) = shutdown_tx {
            let _ = tx.send(());
            log::info!("[Gateway] Shutdown signal sent");
        }

        Ok(())
    }

    /// Update configuration
    pub fn update_config(&self, new_config: GatewayServerConfig) -> Result<(), String> {
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        *config = new_config;
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> Result<GatewayServerConfig, String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;
        Ok(config.clone())
    }

    /// Restart a specific backend
    pub async fn restart_backend(&self, mcp_id: i64) -> Result<BackendInfo, String> {
        let mut backend_manager = self.backend_manager.lock().await;
        backend_manager
            .restart_backend(mcp_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// Get connection config JSON for users to add to their Claude config
    pub fn get_connection_config(&self) -> serde_json::Value {
        serde_json::json!({
            "mcp-gateway": {
                "type": "sse",
                "url": self.get_mcp_endpoint()
            }
        })
    }
}

/// Generate an MCP entry for the Gateway server to add to the library as a system MCP
pub fn generate_gateway_mcp_entry(port: u16) -> crate::db::models::CreateMcpRequest {
    crate::db::models::CreateMcpRequest {
        name: "MCP Gateway".to_string(),
        description: Some("Lazy-loading MCP gateway. Use list_available_mcps to discover MCPs, load_mcp_tools to connect and get tools, and call_mcp_tool to execute tools.".to_string()),
        mcp_type: "http".to_string(),  // Uses HTTP transport (Streamable HTTP)
        command: None,
        args: None,
        url: Some(format!("http://127.0.0.1:{}/mcp", port)),
        headers: None,
        env: None,
        icon: Some("📡".to_string()),
        tags: Some(vec!["gateway".to_string(), "lazy".to_string(), "meta-tools".to_string()]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GatewayServerConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.port, DEFAULT_GATEWAY_PORT);
        assert!(!config.auto_start);
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = GatewayServerConfig {
            enabled: true,
            port: 9999,
            auto_start: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("autoStart")); // camelCase
        let parsed: GatewayServerConfig = serde_json::from_str(&json).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.port, 9999);
        assert!(parsed.auto_start);
    }

    #[test]
    fn test_generate_gateway_mcp_entry() {
        let entry = generate_gateway_mcp_entry(23848);
        assert_eq!(entry.name, "MCP Gateway");
        assert_eq!(entry.mcp_type, "http");
        assert!(entry.url.is_some());
        assert!(entry.url.unwrap().contains("23848"));
        assert!(entry.description.is_some());
        assert!(entry.tags.is_some());
        assert!(entry.tags.unwrap().contains(&"gateway".to_string()));
    }

    #[test]
    fn test_generate_gateway_mcp_entry_custom_port() {
        let entry = generate_gateway_mcp_entry(12345);
        assert!(entry.url.unwrap().contains("12345"));
    }

    fn make_test_state() -> GatewayServerState {
        let db = Database::in_memory().unwrap();
        let db_arc = Arc::new(Mutex::new(db));
        let config = GatewayServerConfig::default();
        GatewayServerState::with_config(config, db_arc)
    }

    #[test]
    fn test_server_state_initial() {
        let state = make_test_state();
        assert!(!state.is_running());
        assert_eq!(state.get_port(), DEFAULT_GATEWAY_PORT);
    }

    #[test]
    fn test_server_state_get_url() {
        let state = make_test_state();
        let url = state.get_url();
        assert_eq!(url, format!("http://127.0.0.1:{}", DEFAULT_GATEWAY_PORT));
    }

    #[test]
    fn test_server_state_get_mcp_endpoint() {
        let state = make_test_state();
        let endpoint = state.get_mcp_endpoint();
        assert!(endpoint.ends_with("/mcp"));
        assert!(endpoint.starts_with("http://127.0.0.1:"));
    }

    #[test]
    fn test_server_state_update_config() {
        let state = make_test_state();
        let new_config = GatewayServerConfig {
            enabled: true,
            port: 5555,
            auto_start: true,
        };
        state.update_config(new_config).unwrap();

        let config = state.get_config().unwrap();
        assert!(config.enabled);
        assert_eq!(config.port, 5555);
        assert!(config.auto_start);
    }

    #[test]
    fn test_server_state_get_connection_config() {
        let state = make_test_state();
        let config = state.get_connection_config();
        let gateway = config.get("mcp-gateway").unwrap();
        assert_eq!(gateway.get("type").unwrap(), "sse");
        assert!(gateway.get("url").unwrap().as_str().unwrap().contains("/mcp"));
    }

    #[test]
    fn test_server_state_custom_port() {
        let db = Database::in_memory().unwrap();
        let db_arc = Arc::new(Mutex::new(db));
        let config = GatewayServerConfig {
            enabled: true,
            port: 8080,
            auto_start: false,
        };
        let state = GatewayServerState::with_config(config, db_arc);
        assert_eq!(state.get_port(), 8080);
        assert!(state.get_url().contains("8080"));
    }

    #[tokio::test]
    async fn test_server_state_get_status() {
        let state = make_test_state();
        let status = state.get_status().await;
        assert!(!status.is_running);
        assert_eq!(status.port, DEFAULT_GATEWAY_PORT);
        assert!(status.available_mcps.is_empty());
        assert!(status.connected_backends.is_empty());
        assert_eq!(status.total_tools, 0);
    }

    #[tokio::test]
    async fn test_server_state_stop_when_not_running() {
        let state = make_test_state();
        let result = state.stop().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not running"));
    }

    #[test]
    fn test_gateway_server_status_serialize() {
        let status = GatewayServerStatus {
            is_running: true,
            port: 23848,
            url: "http://127.0.0.1:23848".to_string(),
            mcp_endpoint: "http://127.0.0.1:23848/mcp".to_string(),
            available_mcps: vec![],
            connected_backends: vec![],
            total_tools: 5,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("isRunning")); // camelCase
        assert!(json.contains("mcpEndpoint"));
        assert!(json.contains("totalTools"));
    }
}
