//! MCP Testing Commands
//!
//! Tauri commands for testing MCP server connections.

use crate::db::Database;
use crate::services::mcp_client::{self, McpTestResult};
use log::{error, info};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Test an MCP by its database ID
#[tauri::command]
pub fn test_mcp(db: State<'_, Arc<Mutex<Database>>>, mcp_id: i64) -> Result<McpTestResult, String> {
    info!("[MCP Test] Testing MCP id={}", mcp_id);

    // Extract MCP data from database in a separate scope to release the lock
    let (mcp_type, command, args, headers, env, url, source) = {
        let db = db.lock().map_err(|e| {
            error!("[MCP Test] Failed to acquire database lock: {}", e);
            e.to_string()
        })?;

        let mut stmt = db
            .conn()
            .prepare("SELECT type, command, args, url, headers, env, source FROM mcps WHERE id = ?")
            .map_err(|e| e.to_string())?;

        let mcp_data: (
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
                ))
            })
            .map_err(|e| {
                error!("[MCP Test] MCP not found: {}", e);
                format!("MCP not found: {}", e)
            })?;

        let (mcp_type, command, args_json, url, headers_json, env_json, source) = mcp_data;

        // Parse JSON fields
        let args: Vec<String> = args_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let headers: Option<HashMap<String, String>> =
            headers_json.and_then(|s| serde_json::from_str(&s).ok());

        let env: Option<HashMap<String, String>> =
            env_json.and_then(|s| serde_json::from_str(&s).ok());

        (mcp_type, command, args, headers, env, url, source)
    };

    // System MCPs (Tool Manager and Gateway) use Streamable HTTP which requires
    // async SSE handling for full protocol test
    if source == "system" {
        let mcp_url = url.ok_or_else(|| "System MCP requires a URL".to_string())?;
        info!("[MCP Test] Testing system MCP with Streamable HTTP: {}", mcp_url);
        return Ok(mcp_client::test_streamable_http_mcp(&mcp_url, headers.as_ref(), 30));
    }

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

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Extract MCP data from database for testing (for testing)
pub fn get_mcp_test_data_from_db(
    db: &Database,
    mcp_id: i64,
) -> Result<
    (
        String,
        Option<String>,
        Vec<String>,
        Option<HashMap<String, String>>,
        Option<HashMap<String, String>>,
        Option<String>,
    ),
    String,
> {
    let mut stmt = db
        .conn()
        .prepare("SELECT type, command, args, url, headers, env FROM mcps WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let mcp_data: (
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
            ))
        })
        .map_err(|e| format!("MCP not found: {}", e))?;

    let (mcp_type, command, args_json, url, headers_json, env_json) = mcp_data;

    let args: Vec<String> = args_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let headers: Option<HashMap<String, String>> =
        headers_json.and_then(|s| serde_json::from_str(&s).ok());

    let env: Option<HashMap<String, String>> = env_json.and_then(|s| serde_json::from_str(&s).ok());

    Ok((mcp_type, command, args, headers, env, url))
}

/// Validate MCP config before testing (for testing)
pub fn validate_mcp_config(
    mcp_type: &str,
    command: Option<&str>,
    url: Option<&str>,
) -> Result<(), String> {
    match mcp_type {
        "stdio" => {
            if command.is_none() {
                return Err("STDIO MCP requires a command".to_string());
            }
        }
        "http" | "sse" => {
            if url.is_none() {
                return Err(format!("{} MCP requires a URL", mcp_type.to_uppercase()));
            }
        }
        _ => {
            return Err(format!("Unknown MCP type: {}", mcp_type));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    // =========================================================================
    // validate_mcp_config tests
    // =========================================================================

    #[test]
    fn test_validate_stdio_with_command() {
        let result = validate_mcp_config("stdio", Some("npx"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_stdio_without_command() {
        let result = validate_mcp_config("stdio", None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a command"));
    }

    #[test]
    fn test_validate_http_with_url() {
        let result = validate_mcp_config("http", None, Some("https://example.com"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_http_without_url() {
        let result = validate_mcp_config("http", None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a URL"));
    }

    #[test]
    fn test_validate_sse_with_url() {
        let result = validate_mcp_config("sse", None, Some("https://example.com/sse"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sse_without_url() {
        let result = validate_mcp_config("sse", None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a URL"));
    }

    #[test]
    fn test_validate_unknown_type() {
        let result = validate_mcp_config("unknown", Some("cmd"), Some("url"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown MCP type"));
    }

    // =========================================================================
    // get_mcp_test_data_from_db tests
    // =========================================================================

    #[test]
    fn test_get_mcp_test_data_stdio() {
        let db = Database::in_memory().unwrap();

        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type, command, args, env)
                   VALUES (?, ?, ?, ?, ?)"#,
                params![
                    "test-stdio",
                    "stdio",
                    "npx",
                    r#"["-y", "mcp-server"]"#,
                    r#"{"KEY": "value"}"#
                ],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let (mcp_type, command, args, _headers, env, url) =
            get_mcp_test_data_from_db(&db, id).unwrap();

        assert_eq!(mcp_type, "stdio");
        assert_eq!(command, Some("npx".to_string()));
        assert_eq!(args, vec!["-y", "mcp-server"]);
        assert_eq!(
            env.as_ref().and_then(|e| e.get("KEY")),
            Some(&"value".to_string())
        );
        assert!(url.is_none());
    }

    #[test]
    fn test_get_mcp_test_data_http() {
        let db = Database::in_memory().unwrap();

        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type, url, headers)
                   VALUES (?, ?, ?, ?)"#,
                params![
                    "test-http",
                    "http",
                    "https://api.example.com/mcp",
                    r#"{"Authorization": "Bearer token"}"#
                ],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let (mcp_type, command, args, headers, _env, url) =
            get_mcp_test_data_from_db(&db, id).unwrap();

        assert_eq!(mcp_type, "http");
        assert!(command.is_none());
        assert!(args.is_empty());
        assert_eq!(url, Some("https://api.example.com/mcp".to_string()));
        assert_eq!(
            headers.as_ref().and_then(|h| h.get("Authorization")),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_get_mcp_test_data_sse() {
        let db = Database::in_memory().unwrap();

        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type, url)
                   VALUES (?, ?, ?)"#,
                params!["test-sse", "sse", "https://api.example.com/sse"],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let (mcp_type, _command, _args, _headers, _env, url) =
            get_mcp_test_data_from_db(&db, id).unwrap();

        assert_eq!(mcp_type, "sse");
        assert_eq!(url, Some("https://api.example.com/sse".to_string()));
    }

    #[test]
    fn test_get_mcp_test_data_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_mcp_test_data_from_db(&db, 9999);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("MCP not found"));
    }

    #[test]
    fn test_get_mcp_test_data_null_fields() {
        let db = Database::in_memory().unwrap();

        // Minimal MCP with only required fields
        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type) VALUES (?, ?)"#,
                params!["minimal", "stdio"],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let (mcp_type, command, args, headers, env, url) =
            get_mcp_test_data_from_db(&db, id).unwrap();

        assert_eq!(mcp_type, "stdio");
        assert!(command.is_none());
        assert!(args.is_empty());
        assert!(headers.is_none());
        assert!(env.is_none());
        assert!(url.is_none());
    }

    #[test]
    fn test_get_mcp_test_data_empty_json_arrays() {
        let db = Database::in_memory().unwrap();

        db.conn()
            .execute(
                r#"INSERT INTO mcps (name, type, command, args, env)
                   VALUES (?, ?, ?, ?, ?)"#,
                params!["empty-json", "stdio", "cmd", "[]", "{}"],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let (_mcp_type, _command, args, _headers, env, _url) =
            get_mcp_test_data_from_db(&db, id).unwrap();

        assert!(args.is_empty());
        assert!(env.is_some());
        assert!(env.unwrap().is_empty());
    }
}
