//! Tauri commands for MCP server control
//!
//! These commands allow the frontend to start/stop and configure the MCP server.

use crate::db::Database;
use crate::mcp_server::server::{generate_self_mcp_entry, McpServerConfig, McpServerStatus};
use crate::mcp_server::McpServerState;
use log::info;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Get the current MCP server status
#[tauri::command]
pub fn get_mcp_server_status(
    server_state: State<'_, Arc<McpServerState>>,
) -> Result<McpServerStatus, String> {
    info!("[McpServerCmd] Getting server status");
    Ok(server_state.get_status())
}

/// Get the current MCP server configuration
#[tauri::command]
pub fn get_mcp_server_config(
    server_state: State<'_, Arc<McpServerState>>,
) -> Result<McpServerConfig, String> {
    info!("[McpServerCmd] Getting server config");
    server_state.get_config()
}

/// Update the MCP server configuration
#[tauri::command]
pub fn update_mcp_server_config(
    server_state: State<'_, Arc<McpServerState>>,
    db: State<'_, Arc<Mutex<Database>>>,
    config: McpServerConfig,
) -> Result<(), String> {
    info!("[McpServerCmd] Updating server config: {:?}", config);

    // Update in-memory config
    server_state.update_config(config.clone())?;

    // Persist to database
    let db = db.lock().map_err(|e| e.to_string())?;
    db.set_setting("mcp_server_enabled", &config.enabled.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("mcp_server_port", &config.port.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("mcp_server_auto_start", &config.auto_start.to_string())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Start the MCP server
#[tauri::command]
pub async fn start_mcp_server(
    server_state: State<'_, Arc<McpServerState>>,
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<McpServerStatus, String> {
    info!("[McpServerCmd] Starting MCP server");

    // Clone the Arcs to use in the async context
    let server = server_state.inner().clone();
    let db_arc = db.inner().clone();

    server.start(db_arc).await?;

    Ok(server.get_status())
}

/// Stop the MCP server
#[tauri::command]
pub fn stop_mcp_server(
    server_state: State<'_, Arc<McpServerState>>,
) -> Result<McpServerStatus, String> {
    info!("[McpServerCmd] Stopping MCP server");

    server_state.stop()?;

    Ok(server_state.get_status())
}

/// Get the connection config JSON for adding to Claude config
#[tauri::command]
pub fn get_mcp_server_connection_config(
    server_state: State<'_, Arc<McpServerState>>,
) -> Result<Value, String> {
    info!("[McpServerCmd] Getting connection config");
    Ok(server_state.get_connection_config())
}

/// Add the Tool Manager MCP to the library
#[tauri::command]
pub fn add_self_mcp_to_library(
    server_state: State<'_, Arc<McpServerState>>,
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<crate::db::models::Mcp, String> {
    info!("[McpServerCmd] Adding Tool Manager MCP to library");

    let port = server_state.get_port();
    let entry = generate_self_mcp_entry(port);

    let db = db.lock().map_err(|e| e.to_string())?;

    // Check if it already exists
    if let Ok(Some(existing)) = db.get_mcp_by_name(&entry.name) {
        info!("[McpServerCmd] Tool Manager MCP already exists, updating");
        // Update URL, type, and source to ensure consistency
        let mut updated = existing.clone();
        updated.url = entry.url;
        updated.mcp_type = entry.mcp_type;
        updated.source = "system".to_string();
        return db.update_mcp(&updated).map_err(|e| e.to_string());
    }

    // Create new as system MCP (readonly)
    db.create_system_mcp(&entry).map_err(|e| e.to_string())
}

/// Remove the Tool Manager MCP from the library
#[tauri::command]
pub fn remove_self_mcp_from_library(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    info!("[McpServerCmd] Removing Tool Manager MCP from library");

    let db = db.lock().map_err(|e| e.to_string())?;

    if let Ok(Some(existing)) = db.get_mcp_by_name("Tool Manager MCP") {
        db.delete_mcp(existing.id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Check if the Tool Manager MCP is in the library
#[tauri::command]
pub fn is_self_mcp_in_library(db: State<'_, Arc<Mutex<Database>>>) -> Result<bool, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let exists = db
        .get_mcp_by_name("Tool Manager MCP")
        .map(|opt| opt.is_some())
        .unwrap_or(false);
    Ok(exists)
}
