use crate::db::Database;
use crate::services::mcp_registry::{RegistryClient, RegistryMcpEntry};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySearchResult {
    pub entries: Vec<RegistryMcpEntry>,
    pub next_cursor: Option<String>,
}

/// Search the MCP Registry for servers
#[tauri::command]
pub async fn search_mcp_registry(
    query: String,
    limit: Option<u32>,
) -> Result<Vec<RegistryMcpEntry>, String> {
    let client = RegistryClient::new();

    let servers = client
        .search(&query, limit.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())?;

    let entries: Vec<RegistryMcpEntry> = servers
        .iter()
        .filter_map(|s| s.to_mcp_entry().ok())
        .collect();

    Ok(entries)
}

/// List MCPs from the registry with pagination
#[tauri::command]
pub async fn list_mcp_registry(
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<RegistrySearchResult, String> {
    let client = RegistryClient::new();

    let (servers, next_cursor) = client
        .list(limit.unwrap_or(20), cursor.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let entries: Vec<RegistryMcpEntry> = servers
        .iter()
        .filter_map(|s| s.to_mcp_entry().ok())
        .collect();

    Ok(RegistrySearchResult {
        entries,
        next_cursor,
    })
}

/// Get a specific MCP from the registry by ID
#[tauri::command]
pub async fn get_mcp_from_registry(server_id: String) -> Result<RegistryMcpEntry, String> {
    let client = RegistryClient::new();

    let server = client
        .get_server(&server_id)
        .await
        .map_err(|e| e.to_string())?;

    server.to_mcp_entry().map_err(|e| e.to_string())
}

/// Import an MCP from the registry to the local library
#[tauri::command]
pub fn import_mcp_from_registry(
    db: State<'_, Mutex<Database>>,
    entry: RegistryMcpEntry,
) -> Result<i64, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Convert args and headers to JSON strings for storage
    let args_json = entry.args.as_ref().map(|a| serde_json::to_string(a).ok()).flatten();
    let headers_json = entry.headers.as_ref().map(|h| serde_json::to_string(h).ok()).flatten();
    let env_json = entry.env.as_ref().map(|e| serde_json::to_string(e).ok()).flatten();

    db.conn()
        .execute(
            r#"INSERT INTO mcps (name, description, type, command, args, url, headers, env, source, source_path)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'registry', ?)"#,
            params![
                entry.name,
                entry.description,
                entry.mcp_type,
                entry.command,
                args_json,
                entry.url,
                headers_json,
                env_json,
                entry.source_url
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    Ok(id)
}
