//! MCP Gateway Server lifecycle management
//!
//! Handles starting, stopping, and managing the Gateway HTTP server.

use crate::db::Database;
use crate::mcp_gateway::backend::{BackendInfo, GatewayBackendManager};
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
    pub connected_backends: Vec<BackendInfo>,
    pub total_tools: usize,
}

/// Gateway server state managed by Tauri
pub struct GatewayServerState {
    pub is_running: Arc<AtomicBool>,
    pub config: Arc<Mutex<GatewayServerConfig>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    port: Arc<Mutex<u16>>,
    pub backend_manager: Arc<tokio::sync::Mutex<GatewayBackendManager>>,
    db: Arc<Mutex<Database>>,
}

impl GatewayServerState {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        let backend_manager = GatewayBackendManager::new(db.clone());
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(GatewayServerConfig::default())),
            shutdown_tx: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(DEFAULT_GATEWAY_PORT)),
            backend_manager: Arc::new(tokio::sync::Mutex::new(backend_manager)),
            db,
        }
    }

    pub fn with_config(config: GatewayServerConfig, db: Arc<Mutex<Database>>) -> Self {
        let port = config.port;
        let backend_manager = GatewayBackendManager::new(db.clone());
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            config: Arc::new(Mutex::new(config)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(port)),
            backend_manager: Arc::new(tokio::sync::Mutex::new(backend_manager)),
            db,
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

    /// Get current status (synchronous version for basic info)
    pub fn get_status_sync(&self) -> GatewayServerStatus {
        GatewayServerStatus {
            is_running: self.is_running(),
            port: self.get_port(),
            url: self.get_url(),
            mcp_endpoint: self.get_mcp_endpoint(),
            connected_backends: Vec::new(),
            total_tools: 0,
        }
    }

    /// Get current status with backend info (requires async)
    pub async fn get_status(&self) -> GatewayServerStatus {
        let backend_manager = self.backend_manager.lock().await;
        GatewayServerStatus {
            is_running: self.is_running(),
            port: self.get_port(),
            url: self.get_url(),
            mcp_endpoint: self.get_mcp_endpoint(),
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

        // Load and connect to all gateway MCPs
        {
            let mut backend_manager = self.backend_manager.lock().await;
            backend_manager
                .load_and_connect()
                .await
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
            "[Gateway] MCP Gateway started successfully on port {} with {} backends and {} tools",
            port,
            status.connected_backends.len(),
            status.total_tools
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
        description: Some("Aggregates multiple MCP servers into a single endpoint. Tool names are prefixed with their source MCP (e.g., 'filesystem__read_file').".to_string()),
        mcp_type: "http".to_string(),  // Uses HTTP transport (Streamable HTTP)
        command: None,
        args: None,
        url: Some(format!("http://127.0.0.1:{}/mcp", port)),
        headers: None,
        env: None,
        icon: Some("📡".to_string()),
        tags: Some(vec!["gateway".to_string(), "aggregator".to_string(), "proxy".to_string()]),
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
    fn test_get_urls() {
        // Can't easily test without database, but we can test the URL format logic
        assert_eq!(
            format!("http://127.0.0.1:{}", DEFAULT_GATEWAY_PORT),
            format!("http://127.0.0.1:{}", 23848)
        );
    }

    #[test]
    fn test_generate_gateway_mcp_entry() {
        let entry = generate_gateway_mcp_entry(23848);
        assert_eq!(entry.name, "MCP Gateway");
        assert_eq!(entry.mcp_type, "http");
        assert!(entry.url.is_some());
        assert!(entry.url.unwrap().contains("23848"));
    }
}
