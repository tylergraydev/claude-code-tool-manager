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
    // Request more items since we filter for only latest versions
    // (many entries are old versions that get skipped)
    let effective_limit = limit.unwrap_or(100);
    log::info!("[Registry] list_mcp_registry called, limit: {}, cursor: {:?}", effective_limit, cursor);

    let client = RegistryClient::new();

    let (servers, next_cursor) = client
        .list(effective_limit, cursor.as_deref())
        .await
        .map_err(|e| {
            log::error!("[Registry] Failed to fetch from registry: {}", e);
            e.to_string()
        })?;

    log::info!("[Registry] Got {} servers from API", servers.len());

    // Log details about what each server has
    for s in &servers {
        log::info!("[Registry] Server '{}': packages={}, remotes={}",
            s.name,
            s.packages.as_ref().map(|p| p.len()).unwrap_or(0),
            s.remotes.as_ref().map(|r| r.len()).unwrap_or(0)
        );
    }

    let mut success_count = 0;
    let mut fail_count = 0;
    let entries: Vec<RegistryMcpEntry> = servers
        .iter()
        .filter_map(|s| {
            match s.to_mcp_entry() {
                Ok(entry) => {
                    success_count += 1;
                    Some(entry)
                },
                Err(e) => {
                    fail_count += 1;
                    log::warn!("[Registry] SKIPPED '{}': {} (packages: {:?}, remotes: {:?})",
                        s.name, e,
                        s.packages.as_ref().map(|p| p.iter().map(|pkg| &pkg.registry_type).collect::<Vec<_>>()),
                        s.remotes.as_ref().map(|r| r.iter().map(|rem| &rem.transport_type).collect::<Vec<_>>())
                    );
                    None
                }
            }
        })
        .collect();

    log::warn!("[Registry] RESULT: Converted {}/{} servers ({} skipped)", success_count, servers.len(), fail_count);

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
