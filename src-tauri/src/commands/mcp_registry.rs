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
    let effective_limit = limit.unwrap_or(100);
    let client = RegistryClient::new();

    let (servers, next_cursor) = client
        .list(effective_limit, cursor.as_deref())
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
    import_mcp_from_registry_in_db(&db, &entry)
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Import an MCP from the registry to the local library (for testing)
pub fn import_mcp_from_registry_in_db(db: &Database, entry: &RegistryMcpEntry) -> Result<i64, String> {
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

/// Get an imported registry MCP by ID (for testing)
pub fn get_registry_mcp_by_id(db: &Database, id: i64) -> Result<RegistryMcpEntry, String> {
    db.conn()
        .query_row(
            r#"SELECT name, description, type, command, args, url, headers, env, source_path
               FROM mcps WHERE id = ?"#,
            [id],
            |row| {
                let args: Option<String> = row.get(4)?;
                let headers: Option<String> = row.get(6)?;
                let env: Option<String> = row.get(7)?;

                Ok(RegistryMcpEntry {
                    registry_id: id.to_string(), // Use DB id as registry_id since we don't store it
                    name: row.get(0)?,
                    description: row.get(1)?,
                    mcp_type: row.get(2)?,
                    command: row.get(3)?,
                    args: args.and_then(|a| serde_json::from_str(&a).ok()),
                    url: row.get(5)?,
                    headers: headers.and_then(|h| serde_json::from_str(&h).ok()),
                    env: env.and_then(|e| serde_json::from_str(&e).ok()),
                    env_placeholders: None, // Not stored in DB
                    source_url: row.get(8)?,
                    version: None, // Not stored in DB
                    registry_type: None, // Not stored in DB
                    updated_at: None, // Not stored in DB
                })
            },
        )
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // =========================================================================
    // RegistrySearchResult serde tests
    // =========================================================================

    #[test]
    fn test_registry_search_result_serde() {
        let result = RegistrySearchResult {
            entries: vec![],
            next_cursor: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("nextCursor")); // camelCase
        assert!(json.contains("abc123"));

        let deserialized: RegistrySearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.next_cursor, Some("abc123".to_string()));
    }

    #[test]
    fn test_registry_search_result_no_cursor() {
        let result = RegistrySearchResult {
            entries: vec![],
            next_cursor: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: RegistrySearchResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.next_cursor.is_none());
    }

    // =========================================================================
    // import_mcp_from_registry_in_db tests
    // =========================================================================

    #[test]
    fn test_import_stdio_mcp_from_registry() {
        let db = Database::in_memory().unwrap();

        let entry = RegistryMcpEntry {
            registry_id: "test-mcp-123".to_string(),
            name: "test-mcp".to_string(),
            description: Some("A test MCP".to_string()),
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@test/mcp".to_string()]),
            url: None,
            headers: None,
            env: Some({
                let mut m = HashMap::new();
                m.insert("API_KEY".to_string(), "test".to_string());
                m
            }),
            env_placeholders: None,
            source_url: Some("https://registry.example.com/test-mcp".to_string()),
            version: Some("1.0.0".to_string()),
            registry_type: Some("npm".to_string()),
            updated_at: None,
        };

        let id = import_mcp_from_registry_in_db(&db, &entry).unwrap();
        assert!(id > 0);

        let fetched = get_registry_mcp_by_id(&db, id).unwrap();

        assert_eq!(fetched.name, "test-mcp");
        assert_eq!(fetched.description, Some("A test MCP".to_string()));
        assert_eq!(fetched.mcp_type, "stdio");
        assert_eq!(fetched.command, Some("npx".to_string()));
        assert_eq!(fetched.args, Some(vec!["-y".to_string(), "@test/mcp".to_string()]));
        assert_eq!(fetched.env.as_ref().and_then(|e| e.get("API_KEY")), Some(&"test".to_string()));
    }

    #[test]
    fn test_import_sse_mcp_from_registry() {
        let db = Database::in_memory().unwrap();

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let entry = RegistryMcpEntry {
            registry_id: "sse-mcp-456".to_string(),
            name: "sse-mcp".to_string(),
            description: Some("An SSE MCP".to_string()),
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://api.example.com/sse".to_string()),
            headers: Some(headers),
            env: None,
            env_placeholders: None,
            source_url: None,
            version: None,
            registry_type: None,
            updated_at: None,
        };

        let id = import_mcp_from_registry_in_db(&db, &entry).unwrap();
        let fetched = get_registry_mcp_by_id(&db, id).unwrap();

        assert_eq!(fetched.name, "sse-mcp");
        assert_eq!(fetched.mcp_type, "sse");
        assert_eq!(fetched.url, Some("https://api.example.com/sse".to_string()));
        assert_eq!(
            fetched.headers.as_ref().and_then(|h| h.get("Authorization")),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_import_minimal_mcp_from_registry() {
        let db = Database::in_memory().unwrap();

        let entry = RegistryMcpEntry {
            registry_id: "minimal-789".to_string(),
            name: "minimal".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("mcp-server".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            env_placeholders: None,
            source_url: None,
            version: None,
            registry_type: None,
            updated_at: None,
        };

        let id = import_mcp_from_registry_in_db(&db, &entry).unwrap();
        let fetched = get_registry_mcp_by_id(&db, id).unwrap();

        assert_eq!(fetched.name, "minimal");
        assert!(fetched.description.is_none());
        assert!(fetched.args.is_none());
        assert!(fetched.env.is_none());
        assert!(fetched.headers.is_none());
    }

    #[test]
    fn test_import_mcp_source_set_to_registry() {
        let db = Database::in_memory().unwrap();

        let entry = RegistryMcpEntry {
            registry_id: "sourced-111".to_string(),
            name: "sourced-mcp".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("cmd".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            env_placeholders: None,
            source_url: Some("https://registry.example.com/sourced".to_string()),
            version: None,
            registry_type: None,
            updated_at: None,
        };

        let id = import_mcp_from_registry_in_db(&db, &entry).unwrap();

        // Verify source is set to 'registry'
        let source: String = db
            .conn()
            .query_row("SELECT source FROM mcps WHERE id = ?", [id], |row| row.get(0))
            .unwrap();

        assert_eq!(source, "registry");
    }

    #[test]
    fn test_import_mcp_duplicate_name_fails() {
        let db = Database::in_memory().unwrap();

        let entry = RegistryMcpEntry {
            registry_id: "dup-222".to_string(),
            name: "duplicate-mcp".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("cmd".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            env_placeholders: None,
            source_url: None,
            version: None,
            registry_type: None,
            updated_at: None,
        };

        // First import succeeds
        import_mcp_from_registry_in_db(&db, &entry).unwrap();

        // Second import with same name fails (UNIQUE constraint)
        let result = import_mcp_from_registry_in_db(&db, &entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_registry_mcp_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_registry_mcp_by_id(&db, 9999);
        assert!(result.is_err());
    }
}
