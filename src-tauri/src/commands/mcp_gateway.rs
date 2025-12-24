//! Tauri commands for MCP Gateway control
//!
//! These commands allow the frontend to start/stop and configure the MCP Gateway.

use crate::db::models::GatewayMcp;
use crate::db::Database;
use crate::mcp_gateway::backend::BackendInfo;
use crate::mcp_gateway::server::{GatewayServerConfig, GatewayServerStatus};
use crate::mcp_gateway::GatewayServerState;
use log::info;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Get the current Gateway server status
#[tauri::command]
pub async fn get_gateway_status(
    gateway_state: State<'_, Arc<GatewayServerState>>,
) -> Result<GatewayServerStatus, String> {
    info!("[GatewayCmd] Getting gateway status");
    Ok(gateway_state.get_status().await)
}

/// Get the current Gateway server configuration
#[tauri::command]
pub fn get_gateway_config(
    gateway_state: State<'_, Arc<GatewayServerState>>,
) -> Result<GatewayServerConfig, String> {
    info!("[GatewayCmd] Getting gateway config");
    gateway_state.get_config()
}

/// Update the Gateway server configuration
#[tauri::command]
pub fn update_gateway_config(
    gateway_state: State<'_, Arc<GatewayServerState>>,
    db: State<'_, Arc<Mutex<Database>>>,
    config: GatewayServerConfig,
) -> Result<(), String> {
    info!("[GatewayCmd] Updating gateway config: {:?}", config);

    // Update in-memory config
    gateway_state.update_config(config.clone())?;

    // Persist to database
    let db = db.lock().map_err(|e| e.to_string())?;
    db.set_setting("gateway_enabled", &config.enabled.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("gateway_port", &config.port.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("gateway_auto_start", &config.auto_start.to_string())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Start the Gateway server
#[tauri::command]
pub async fn start_gateway(
    gateway_state: State<'_, Arc<GatewayServerState>>,
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<GatewayServerStatus, String> {
    info!("[GatewayCmd] Starting Gateway server");

    let gateway = gateway_state.inner().clone();
    gateway.start().await?;

    // Add/update the Gateway MCP in the library as a system MCP
    let port = gateway.get_port();
    {
        let db = db.lock().map_err(|e| e.to_string())?;
        let entry = crate::mcp_gateway::server::generate_gateway_mcp_entry(port);
        match db.get_mcp_by_name(&entry.name) {
            Ok(Some(existing)) => {
                let mut updated = existing.clone();
                updated.url = entry.url;
                updated.mcp_type = entry.mcp_type;
                updated.source = "system".to_string();
                if let Err(e) = db.update_mcp(&updated) {
                    info!("[GatewayCmd] Failed to update Gateway MCP: {}", e);
                }
            }
            Ok(None) => {
                if let Err(e) = db.create_system_mcp(&entry) {
                    info!("[GatewayCmd] Failed to add Gateway MCP: {}", e);
                } else {
                    info!("[GatewayCmd] MCP Gateway added to library");
                }
            }
            Err(e) => {
                info!("[GatewayCmd] Failed to check for Gateway MCP: {}", e);
            }
        }
    } // db lock is released here before the await

    Ok(gateway.get_status().await)
}

/// Stop the Gateway server
#[tauri::command]
pub async fn stop_gateway(
    gateway_state: State<'_, Arc<GatewayServerState>>,
) -> Result<GatewayServerStatus, String> {
    info!("[GatewayCmd] Stopping Gateway server");

    let gateway = gateway_state.inner().clone();
    gateway.stop().await?;

    Ok(gateway.get_status().await)
}

/// Get the connection config JSON for adding to Claude config
#[tauri::command]
pub fn get_gateway_connection_config(
    gateway_state: State<'_, Arc<GatewayServerState>>,
) -> Result<Value, String> {
    info!("[GatewayCmd] Getting gateway connection config");
    Ok(gateway_state.get_connection_config())
}

/// Get all MCPs configured for the gateway
#[tauri::command]
pub fn get_gateway_mcps(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<GatewayMcp>, String> {
    info!("[GatewayCmd] Getting gateway MCPs");
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_gateway_mcps().map_err(|e| e.to_string())
}

/// Add an MCP to the gateway
#[tauri::command]
pub fn add_mcp_to_gateway(db: State<'_, Arc<Mutex<Database>>>, mcp_id: i64) -> Result<(), String> {
    info!("[GatewayCmd] Adding MCP {} to gateway", mcp_id);
    let db = db.lock().map_err(|e| e.to_string())?;
    db.add_gateway_mcp(mcp_id).map_err(|e| e.to_string())
}

/// Remove an MCP from the gateway
#[tauri::command]
pub fn remove_mcp_from_gateway(
    db: State<'_, Arc<Mutex<Database>>>,
    mcp_id: i64,
) -> Result<(), String> {
    info!("[GatewayCmd] Removing MCP {} from gateway", mcp_id);
    let db = db.lock().map_err(|e| e.to_string())?;
    db.remove_gateway_mcp(mcp_id).map_err(|e| e.to_string())
}

/// Toggle an MCP's enabled status in the gateway
#[tauri::command]
pub fn toggle_gateway_mcp(
    db: State<'_, Arc<Mutex<Database>>>,
    mcp_id: i64,
    enabled: bool,
) -> Result<(), String> {
    info!(
        "[GatewayCmd] Toggling gateway MCP {} to {}",
        mcp_id, enabled
    );
    let db = db.lock().map_err(|e| e.to_string())?;
    db.toggle_gateway_mcp(mcp_id, enabled)
        .map_err(|e| e.to_string())
}

/// Set auto-restart for a gateway MCP
#[tauri::command]
pub fn set_gateway_mcp_auto_restart(
    db: State<'_, Arc<Mutex<Database>>>,
    mcp_id: i64,
    auto_restart: bool,
) -> Result<(), String> {
    info!(
        "[GatewayCmd] Setting gateway MCP {} auto_restart to {}",
        mcp_id, auto_restart
    );
    let db = db.lock().map_err(|e| e.to_string())?;
    db.set_gateway_mcp_auto_restart(mcp_id, auto_restart)
        .map_err(|e| e.to_string())
}

/// Check if an MCP is in the gateway
#[tauri::command]
pub fn is_mcp_in_gateway(db: State<'_, Arc<Mutex<Database>>>, mcp_id: i64) -> Result<bool, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.is_mcp_in_gateway(mcp_id).map_err(|e| e.to_string())
}

/// Get backend connection info
#[tauri::command]
pub async fn get_gateway_backends(
    gateway_state: State<'_, Arc<GatewayServerState>>,
) -> Result<Vec<BackendInfo>, String> {
    info!("[GatewayCmd] Getting gateway backends");
    let backend_manager = gateway_state.backend_manager.lock().await;
    Ok(backend_manager.get_backends_info())
}

/// Restart a specific backend connection
#[tauri::command]
pub async fn restart_gateway_backend(
    gateway_state: State<'_, Arc<GatewayServerState>>,
    mcp_id: i64,
) -> Result<BackendInfo, String> {
    info!("[GatewayCmd] Restarting gateway backend for MCP {}", mcp_id);
    gateway_state.restart_backend(mcp_id).await
}
