//! MCP Testing Commands
//!
//! Tauri commands for testing MCP server connections.

use crate::db::Database;
use crate::services::mcp_client::{self, McpTestResult};
use log::{error, info};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

/// Test an MCP by its database ID
#[tauri::command]
pub fn test_mcp(db: State<'_, Mutex<Database>>, mcp_id: i64) -> Result<McpTestResult, String> {
    info!("[MCP Test] Testing MCP id={}", mcp_id);

    // Extract MCP data from database in a separate scope to release the lock
    let (mcp_type, command, args, headers, env, url) = {
        let db = db.lock().map_err(|e| {
            error!("[MCP Test] Failed to acquire database lock: {}", e);
            e.to_string()
        })?;

        let mut stmt = db
            .conn()
            .prepare("SELECT type, command, args, url, headers, env FROM mcps WHERE id = ?")
            .map_err(|e| e.to_string())?;

        let mcp_data: (String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) = stmt
            .query_row([mcp_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|e| {
                error!("[MCP Test] MCP not found: {}", e);
                format!("MCP not found: {}", e)
            })?;

        let (mcp_type, command, args_json, url, headers_json, env_json) = mcp_data;

        // Parse JSON fields
        let args: Vec<String> = args_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let headers: Option<HashMap<String, String>> = headers_json
            .and_then(|s| serde_json::from_str(&s).ok());

        let env: Option<HashMap<String, String>> = env_json
            .and_then(|s| serde_json::from_str(&s).ok());

        (mcp_type, command, args, headers, env, url)
    };

    // Now the database lock is released, perform the test
    let result = match mcp_type.as_str() {
        "stdio" => {
            let cmd = command.ok_or_else(|| "STDIO MCP requires a command".to_string())?;
            info!("[MCP Test] Testing STDIO MCP: {} {:?}", cmd, args);
            mcp_client::test_stdio_mcp(&cmd, &args, env.as_ref(), 30)
        }
        "http" => {
            let mcp_url = url.ok_or_else(|| "HTTP MCP requires a URL".to_string())?;
            info!("[MCP Test] Testing HTTP MCP: {}", mcp_url);
            mcp_client::test_http_mcp(&mcp_url, headers.as_ref(), 30)
        }
        "sse" => {
            let mcp_url = url.ok_or_else(|| "SSE MCP requires a URL".to_string())?;
            info!("[MCP Test] Testing SSE MCP: {}", mcp_url);
            mcp_client::test_sse_mcp(&mcp_url, headers.as_ref(), 30)
        }
        _ => {
            return Err(format!("Unknown MCP type: {}", mcp_type));
        }
    };

    Ok(result)
}

/// Test an MCP configuration directly (for testing before saving)
#[tauri::command]
pub fn test_mcp_config(
    mcp_type: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    url: Option<String>,
    headers: Option<HashMap<String, String>>,
    env: Option<HashMap<String, String>>,
) -> Result<McpTestResult, String> {
    info!("[MCP Test] Testing MCP config: type={}", mcp_type);

    let args_vec = args.unwrap_or_default();

    let result = match mcp_type.as_str() {
        "stdio" => {
            let cmd = command.ok_or_else(|| "STDIO MCP requires a command".to_string())?;
            info!("[MCP Test] Testing STDIO config: {} {:?}", cmd, args_vec);
            mcp_client::test_stdio_mcp(&cmd, &args_vec, env.as_ref(), 30)
        }
        "http" => {
            let mcp_url = url.ok_or_else(|| "HTTP MCP requires a URL".to_string())?;
            info!("[MCP Test] Testing HTTP config: {}", mcp_url);
            mcp_client::test_http_mcp(&mcp_url, headers.as_ref(), 30)
        }
        "sse" => {
            let mcp_url = url.ok_or_else(|| "SSE MCP requires a URL".to_string())?;
            info!("[MCP Test] Testing SSE config: {}", mcp_url);
            mcp_client::test_sse_mcp(&mcp_url, headers.as_ref(), 30)
        }
        _ => {
            return Err(format!("Unknown MCP type: {}", mcp_type));
        }
    };

    Ok(result)
}
