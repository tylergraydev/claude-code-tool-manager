//! MCP Session Commands
//!
//! Tauri commands for managing MCP execution sessions.

use crate::db::Database;
use crate::services::mcp_client::{McpTool, ToolCallResult};
use crate::services::mcp_session::{McpSessionManager, SessionInfo, StartSessionResult};
use log::{error, info};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// MCP session data extracted from the database
#[derive(Debug)]
pub struct McpSessionData {
    pub name: String,
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub source: String,
}

/// Extract MCP session data from the database (no Tauri State dependency)
#[allow(clippy::type_complexity)]
pub(crate) fn get_mcp_session_data_from_db(
    db: &Database,
    mcp_id: i64,
) -> Result<McpSessionData, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT name, type, command, args, env, url, headers, source FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    let mcp_data: (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
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
                row.get(7)?,
            ))
        })
        .map_err(|e| format!("MCP not found: {}", e))?;

    let (name, mcp_type, command, args_json, env_json, url, headers_json, source) = mcp_data;

    let args: Vec<String> = args_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let env: Option<HashMap<String, String>> = env_json.and_then(|s| serde_json::from_str(&s).ok());

    let headers: Option<HashMap<String, String>> =
        headers_json.and_then(|s| serde_json::from_str(&s).ok());

    Ok(McpSessionData {
        name,
        mcp_type,
        command,
        args,
        env,
        url,
        headers,
        source,
    })
}

/// Validate MCP session data before starting a session
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn validate_mcp_session_data(data: &McpSessionData) -> Result<(), String> {
    if data.source == "system" {
        if data.url.is_none() {
            return Err("System MCP requires a URL".to_string());
        }
        return Ok(());
    }
    match data.mcp_type.as_str() {
        "stdio" => {
            if data.command.is_none() {
                return Err("STDIO MCP requires a command".to_string());
            }
        }
        "http" => {
            if data.url.is_none() {
                return Err("HTTP MCP requires a URL".to_string());
            }
        }
        "sse" => {
            if data.url.is_none() {
                return Err("SSE MCP requires a URL".to_string());
            }
        }
        _ => {
            return Err(format!("Unknown MCP type: {}", data.mcp_type));
        }
    }
    Ok(())
}

/// Start a new MCP execution session
#[tauri::command]
pub fn start_mcp_session(
    db: State<'_, Arc<Mutex<Database>>>,
    session_manager: State<'_, Mutex<McpSessionManager>>,
    mcp_id: i64,
) -> Result<StartSessionResult, String> {
    info!("[MCP Session] Starting session for MCP id={}", mcp_id);

    // Extract MCP data from database
    let data = {
        let db = db.lock().map_err(|e| {
            error!("[MCP Session] Failed to acquire database lock: {}", e);
            e.to_string()
        })?;
        get_mcp_session_data_from_db(&db, mcp_id)?
    };

    let McpSessionData {
        name,
        mcp_type,
        command,
        args,
        env,
        url,
        headers,
        source,
    } = data;

    // Start session based on MCP type
    let manager = session_manager.lock().map_err(|e| {
        error!(
            "[MCP Session] Failed to acquire session manager lock: {}",
            e
        );
        e.to_string()
    })?;

    // System MCPs (Tool Manager and Gateway) use Streamable HTTP transport
    if source == "system" {
        let mcp_url = url.ok_or_else(|| "System MCP requires a URL".to_string())?;
        info!(
            "[MCP Session] Starting Streamable HTTP session for system MCP: {}",
            name
        );
        return manager
            .start_streamable_http_session(mcp_id, &name, &mcp_url, headers.as_ref(), 60)
            .map_err(|e| e.to_string());
    }

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
        error!(
            "[MCP Session] Failed to acquire session manager lock: {}",
            e
        );
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
        error!(
            "[MCP Session] Failed to acquire session manager lock: {}",
            e
        );
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
        error!(
            "[MCP Session] Failed to acquire session manager lock: {}",
            e
        );
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
        error!(
            "[MCP Session] Failed to acquire session manager lock: {}",
            e
        );
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
        error!(
            "[MCP Session] Failed to acquire session manager lock: {}",
            e
        );
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
        error!(
            "[MCP Session] Failed to acquire session manager lock: {}",
            e
        );
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
    use crate::services::mcp_client::McpServerInfo;

    #[test]
    fn test_session_info_type() {
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

    #[test]
    fn test_session_info_serde() {
        let info = SessionInfo {
            id: "session-1".to_string(),
            mcp_id: 42,
            mcp_name: "My MCP".to_string(),
            mcp_type: "stdio".to_string(),
            server_info: Some(McpServerInfo {
                name: "test-server".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            tool_count: 5,
            created_at: "10s".to_string(),
            last_used_at: "5s".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("mcpId"));
        assert!(json.contains("mcpName"));
        assert!(json.contains("mcpType"));
        assert!(json.contains("serverInfo"));
        assert!(json.contains("toolCount"));
        assert!(json.contains("createdAt"));
        assert!(json.contains("lastUsedAt"));

        let deserialized: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "session-1");
        assert_eq!(deserialized.mcp_id, 42);
        assert_eq!(deserialized.tool_count, 5);
        assert!(deserialized.server_info.is_some());
        assert_eq!(deserialized.server_info.unwrap().name, "test-server");
    }

    #[test]
    fn test_session_info_without_server_info() {
        let info = SessionInfo {
            id: "session-2".to_string(),
            mcp_id: 1,
            mcp_name: "Minimal".to_string(),
            mcp_type: "sse".to_string(),
            server_info: None,
            tool_count: 0,
            created_at: "0s".to_string(),
            last_used_at: "0s".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SessionInfo = serde_json::from_str(&json).unwrap();
        assert!(deserialized.server_info.is_none());
        assert_eq!(deserialized.mcp_type, "sse");
    }

    #[test]
    fn test_start_session_result_serde() {
        let result = StartSessionResult {
            session_id: "abc-123".to_string(),
            server_info: None,
            tools: vec![McpTool {
                name: "read_file".to_string(),
                description: Some("Read a file".to_string()),
                input_schema: None,
            }],
            resources_supported: true,
            prompts_supported: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("sessionId"));
        assert!(json.contains("resourcesSupported"));
        assert!(json.contains("promptsSupported"));

        let deserialized: StartSessionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "abc-123");
        assert_eq!(deserialized.tools.len(), 1);
        assert_eq!(deserialized.tools[0].name, "read_file");
        assert!(deserialized.resources_supported);
        assert!(!deserialized.prompts_supported);
    }

    #[test]
    fn test_mcp_tool_serde() {
        let tool = McpTool {
            name: "execute".to_string(),
            description: Some("Execute a command".to_string()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                }
            })),
        };
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: McpTool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "execute");
        assert!(deserialized.input_schema.is_some());
    }

    #[test]
    fn test_tool_call_result_serde() {
        let result = ToolCallResult {
            success: true,
            content: vec![],
            is_error: false,
            error: None,
            execution_time_ms: 150,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("executionTimeMs"));
        assert!(json.contains("isError"));

        let deserialized: ToolCallResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
        assert_eq!(deserialized.execution_time_ms, 150);
    }

    #[test]
    fn test_tool_call_result_with_error() {
        let result = ToolCallResult {
            success: false,
            content: vec![],
            is_error: true,
            error: Some("Command failed".to_string()),
            execution_time_ms: 50,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ToolCallResult = serde_json::from_str(&json).unwrap();
        assert!(!deserialized.success);
        assert!(deserialized.is_error);
        assert_eq!(deserialized.error, Some("Command failed".to_string()));
    }

    #[test]
    fn test_session_manager_new_empty() {
        let manager = McpSessionManager::new();
        let sessions = manager.list_sessions();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_session_manager_get_nonexistent() {
        let manager = McpSessionManager::new();
        let info = manager.get_session_info("nonexistent-id");
        assert!(info.is_none());
    }

    #[test]
    fn test_session_manager_end_nonexistent() {
        let manager = McpSessionManager::new();
        let result = manager.end_session("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_session_manager_cleanup_empty() {
        let manager = McpSessionManager::new();
        let cleaned = manager.cleanup_idle_sessions(300);
        assert_eq!(cleaned, 0);
    }

    // =========================================================================
    // get_mcp_session_data_from_db tests
    // =========================================================================

    #[test]
    fn test_get_mcp_session_data_stdio() {
        let db = Database::in_memory().unwrap();
        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type, command, args, env, source)
                   VALUES (?, ?, ?, ?, ?, ?)"#,
                rusqlite::params![
                    "test-stdio",
                    "stdio",
                    "npx",
                    r#"["-y", "mcp-server"]"#,
                    r#"{"KEY": "value"}"#,
                    "manual"
                ],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let data = get_mcp_session_data_from_db(&db, id).unwrap();
        assert_eq!(data.name, "test-stdio");
        assert_eq!(data.mcp_type, "stdio");
        assert_eq!(data.command, Some("npx".to_string()));
        assert_eq!(data.args, vec!["-y", "mcp-server"]);
        assert_eq!(
            data.env.as_ref().and_then(|e| e.get("KEY")),
            Some(&"value".to_string())
        );
        assert_eq!(data.source, "manual");
    }

    #[test]
    fn test_get_mcp_session_data_http() {
        let db = Database::in_memory().unwrap();
        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type, url, headers, source)
                   VALUES (?, ?, ?, ?, ?)"#,
                rusqlite::params![
                    "test-http",
                    "http",
                    "https://api.example.com/mcp",
                    r#"{"Authorization": "Bearer token"}"#,
                    "manual"
                ],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let data = get_mcp_session_data_from_db(&db, id).unwrap();
        assert_eq!(data.mcp_type, "http");
        assert_eq!(data.url, Some("https://api.example.com/mcp".to_string()));
        assert_eq!(
            data.headers.as_ref().and_then(|h| h.get("Authorization")),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_get_mcp_session_data_system() {
        let db = Database::in_memory().unwrap();
        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type, url, source)
                   VALUES (?, ?, ?, ?)"#,
                rusqlite::params!["gateway", "http", "http://localhost:8080/mcp", "system"],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let data = get_mcp_session_data_from_db(&db, id).unwrap();
        assert_eq!(data.source, "system");
        assert_eq!(data.url, Some("http://localhost:8080/mcp".to_string()));
    }

    #[test]
    fn test_get_mcp_session_data_not_found() {
        let db = Database::in_memory().unwrap();
        let result = get_mcp_session_data_from_db(&db, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("MCP not found"));
    }

    #[test]
    fn test_get_mcp_session_data_null_fields() {
        let db = Database::in_memory().unwrap();
        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type) VALUES (?, ?)"#,
                rusqlite::params!["minimal", "stdio"],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let data = get_mcp_session_data_from_db(&db, id).unwrap();
        assert!(data.command.is_none());
        assert!(data.args.is_empty());
        assert!(data.env.is_none());
        assert!(data.url.is_none());
        assert!(data.headers.is_none());
    }

    // =========================================================================
    // validate_mcp_session_data tests
    // =========================================================================

    #[test]
    fn test_validate_stdio_with_command() {
        let data = McpSessionData {
            name: "test".to_string(),
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: vec![],
            env: None,
            url: None,
            headers: None,
            source: "manual".to_string(),
        };
        assert!(validate_mcp_session_data(&data).is_ok());
    }

    #[test]
    fn test_validate_stdio_without_command() {
        let data = McpSessionData {
            name: "test".to_string(),
            mcp_type: "stdio".to_string(),
            command: None,
            args: vec![],
            env: None,
            url: None,
            headers: None,
            source: "manual".to_string(),
        };
        let result = validate_mcp_session_data(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a command"));
    }

    #[test]
    fn test_validate_http_with_url() {
        let data = McpSessionData {
            name: "test".to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: vec![],
            env: None,
            url: Some("https://example.com".to_string()),
            headers: None,
            source: "manual".to_string(),
        };
        assert!(validate_mcp_session_data(&data).is_ok());
    }

    #[test]
    fn test_validate_http_without_url() {
        let data = McpSessionData {
            name: "test".to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: vec![],
            env: None,
            url: None,
            headers: None,
            source: "manual".to_string(),
        };
        assert!(validate_mcp_session_data(&data).is_err());
    }

    #[test]
    fn test_validate_sse_without_url() {
        let data = McpSessionData {
            name: "test".to_string(),
            mcp_type: "sse".to_string(),
            command: None,
            args: vec![],
            env: None,
            url: None,
            headers: None,
            source: "manual".to_string(),
        };
        assert!(validate_mcp_session_data(&data).is_err());
    }

    #[test]
    fn test_validate_system_with_url() {
        let data = McpSessionData {
            name: "gateway".to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: vec![],
            env: None,
            url: Some("http://localhost:8080".to_string()),
            headers: None,
            source: "system".to_string(),
        };
        assert!(validate_mcp_session_data(&data).is_ok());
    }

    #[test]
    fn test_validate_system_without_url() {
        let data = McpSessionData {
            name: "gateway".to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: vec![],
            env: None,
            url: None,
            headers: None,
            source: "system".to_string(),
        };
        assert!(validate_mcp_session_data(&data).is_err());
    }

    #[test]
    fn test_validate_unknown_type() {
        let data = McpSessionData {
            name: "test".to_string(),
            mcp_type: "unknown".to_string(),
            command: None,
            args: vec![],
            env: None,
            url: None,
            headers: None,
            source: "manual".to_string(),
        };
        let result = validate_mcp_session_data(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown MCP type"));
    }
}
