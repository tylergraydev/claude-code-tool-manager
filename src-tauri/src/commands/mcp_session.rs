//! MCP Session Commands
//!
//! Tauri commands for managing MCP execution sessions.

use crate::db::Database;
use crate::services::mcp_client::{McpTool, ToolCallResult};
use crate::services::mcp_session::{McpSessionManager, SessionInfo, StartSessionResult};
use log::{error, info};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

/// Start a new MCP execution session
#[tauri::command]
pub fn start_mcp_session(
    db: State<'_, Mutex<Database>>,
    session_manager: State<'_, Mutex<McpSessionManager>>,
    mcp_id: i64,
) -> Result<StartSessionResult, String> {
    info!("[MCP Session] Starting session for MCP id={}", mcp_id);

    // Extract MCP data from database
    let (name, mcp_type, command, args, env, url, headers) = {
        let db = db.lock().map_err(|e| {
            error!("[MCP Session] Failed to acquire database lock: {}", e);
            e.to_string()
        })?;

        let mut stmt = db
            .conn()
            .prepare("SELECT name, type, command, args, env, url, headers FROM mcps WHERE id = ?")
            .map_err(|e| e.to_string())?;

        let mcp_data: (
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = stmt
            .query_row([mcp_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .map_err(|e| {
                error!("[MCP Session] MCP not found: {}", e);
                format!("MCP not found: {}", e)
            })?;

        let (name, mcp_type, command, args_json, env_json, url, headers_json) = mcp_data;

        let args: Vec<String> = args_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let env: Option<HashMap<String, String>> =
            env_json.and_then(|s| serde_json::from_str(&s).ok());

        let headers: Option<HashMap<String, String>> =
            headers_json.and_then(|s| serde_json::from_str(&s).ok());

        (name, mcp_type, command, args, env, url, headers)
    };

    // Start session based on MCP type
    let manager = session_manager.lock().map_err(|e| {
        error!("[MCP Session] Failed to acquire session manager lock: {}", e);
        e.to_string()
    })?;

    match mcp_type.as_str() {
        "stdio" => {
            let cmd = command.ok_or_else(|| "STDIO MCP requires a command".to_string())?;
            manager
                .start_stdio_session(mcp_id, &name, &cmd, &args, env.as_ref(), 60)
                .map_err(|e| e.to_string())
        }
        "http" => {
            let mcp_url = url.ok_or_else(|| "HTTP MCP requires a URL".to_string())?;
            manager
                .start_http_session(mcp_id, &name, &mcp_url, headers.as_ref(), 60)
                .map_err(|e| e.to_string())
        }
        "sse" => {
            let mcp_url = url.ok_or_else(|| "SSE MCP requires a URL".to_string())?;
            manager
                .start_sse_session(mcp_id, &name, &mcp_url, headers.as_ref(), 60)
                .map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown MCP type: {}", mcp_type)),
    }
}

/// Execute a tool in an active session
#[tauri::command]
pub fn execute_tool(
    session_manager: State<'_, Mutex<McpSessionManager>>,
    session_id: String,
    tool_name: String,
    arguments: Value,
) -> Result<ToolCallResult, String> {
    info!(
        "[MCP Session] Executing tool '{}' in session {}",
        tool_name, session_id
    );

    let manager = session_manager.lock().map_err(|e| {
        error!("[MCP Session] Failed to acquire session manager lock: {}", e);
        e.to_string()
    })?;

    manager
        .call_tool(&session_id, &tool_name, arguments)
        .map_err(|e| e.to_string())
}

/// End an MCP session
#[tauri::command]
pub fn end_mcp_session(
    session_manager: State<'_, Mutex<McpSessionManager>>,
    session_id: String,
) -> Result<(), String> {
    info!("[MCP Session] Ending session {}", session_id);

    let manager = session_manager.lock().map_err(|e| {
        error!("[MCP Session] Failed to acquire session manager lock: {}", e);
        e.to_string()
    })?;

    manager.end_session(&session_id).map_err(|e| e.to_string())
}

/// List all active sessions
#[tauri::command]
pub fn list_mcp_sessions(
    session_manager: State<'_, Mutex<McpSessionManager>>,
) -> Result<Vec<SessionInfo>, String> {
    let manager = session_manager.lock().map_err(|e| {
        error!("[MCP Session] Failed to acquire session manager lock: {}", e);
        e.to_string()
    })?;

    Ok(manager.list_sessions())
}

/// Get information about a specific session
#[tauri::command]
pub fn get_mcp_session(
    session_manager: State<'_, Mutex<McpSessionManager>>,
    session_id: String,
) -> Result<Option<SessionInfo>, String> {
    let manager = session_manager.lock().map_err(|e| {
        error!("[MCP Session] Failed to acquire session manager lock: {}", e);
        e.to_string()
    })?;

    Ok(manager.get_session_info(&session_id))
}

/// Get tools available in a session
#[tauri::command]
pub fn get_session_tools(
    session_manager: State<'_, Mutex<McpSessionManager>>,
    session_id: String,
) -> Result<Vec<McpTool>, String> {
    let manager = session_manager.lock().map_err(|e| {
        error!("[MCP Session] Failed to acquire session manager lock: {}", e);
        e.to_string()
    })?;

    manager
        .get_session_tools(&session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))
}

/// Clean up idle sessions
#[tauri::command]
pub fn cleanup_idle_sessions(
    session_manager: State<'_, Mutex<McpSessionManager>>,
    max_idle_secs: Option<u64>,
) -> Result<usize, String> {
    let manager = session_manager.lock().map_err(|e| {
        error!("[MCP Session] Failed to acquire session manager lock: {}", e);
        e.to_string()
    })?;

    let idle_threshold = max_idle_secs.unwrap_or(300); // Default 5 minutes
    Ok(manager.cleanup_idle_sessions(idle_threshold))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_type() {
        // Just verify the types compile correctly
        let info = SessionInfo {
            id: "test".to_string(),
            mcp_id: 1,
            mcp_name: "Test".to_string(),
            mcp_type: "stdio".to_string(),
            server_info: None,
            tool_count: 0,
            created_at: "0s".to_string(),
            last_used_at: "0s".to_string(),
        };
        assert_eq!(info.id, "test");
    }
}
