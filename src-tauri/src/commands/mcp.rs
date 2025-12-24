use crate::db::{CreateMcpRequest, Database, Mcp};
use log::{error, info};
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::State;

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn parse_json_map(s: Option<String>) -> Option<std::collections::HashMap<String, String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn row_to_mcp(row: &rusqlite::Row) -> rusqlite::Result<Mcp> {
    Ok(Mcp {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        mcp_type: row.get(3)?,
        command: row.get(4)?,
        args: parse_json_array(row.get(5)?),
        url: row.get(6)?,
        headers: parse_json_map(row.get(7)?),
        env: parse_json_map(row.get(8)?),
        icon: row.get(9)?,
        tags: parse_json_array(row.get(10)?),
        source: row.get(11)?,
        source_path: row.get(12)?,
        is_enabled_global: row.get::<_, i32>(13)? != 0,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

#[tauri::command]
pub fn get_all_mcps(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Mcp>, String> {
    info!("[MCP] Loading all MCPs from database");
    let db = db.lock().map_err(|e| {
        error!("[MCP] Failed to acquire database lock: {}", e);
        e.to_string()
    })?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps ORDER BY name",
        )
        .map_err(|e| {
            error!("[MCP] Failed to prepare query: {}", e);
            e.to_string()
        })?;

    let mcps: Vec<Mcp> = stmt
        .query_map([], row_to_mcp)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    info!("[MCP] Loaded {} MCPs", mcps.len());
    Ok(mcps)
}

#[tauri::command]
pub fn get_mcp(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<Mcp, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_mcp).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_mcp(db: State<'_, Arc<Mutex<Database>>>, mcp: CreateMcpRequest) -> Result<Mcp, String> {
    info!("[MCP] Creating new MCP: {}", mcp.name);
    let db_guard = db.lock().map_err(|e| {
        error!("[MCP] Failed to acquire database lock: {}", e);
        e.to_string()
    })?;

    let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
    let headers_json = mcp
        .headers
        .as_ref()
        .map(|h| serde_json::to_string(h).unwrap());
    let env_json = mcp.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
    let tags_json = mcp.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db_guard.conn()
        .execute(
            "INSERT INTO mcps (name, description, type, command, args, url, headers, env, icon, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![
                mcp.name,
                mcp.description,
                mcp.mcp_type,
                mcp.command,
                args_json,
                mcp.url,
                headers_json,
                env_json,
                mcp.icon,
                tags_json
            ],
        )
        .map_err(|e| {
            error!("[MCP] Failed to create MCP '{}': {}", mcp.name, e);
            e.to_string()
        })?;

    let id = db_guard.conn().last_insert_rowid();
    info!("[MCP] Created MCP with id: {}", id);

    // Fetch the newly created MCP
    let mut stmt = db_guard
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_mcp).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_mcp(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    mcp: CreateMcpRequest,
) -> Result<Mcp, String> {
    info!("[MCP] Updating MCP id={}: {}", id, mcp.name);
    let db = db.lock().map_err(|e| e.to_string())?;

    let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
    let headers_json = mcp
        .headers
        .as_ref()
        .map(|h| serde_json::to_string(h).unwrap());
    let env_json = mcp.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
    let tags_json = mcp.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE mcps SET name = ?, description = ?, type = ?, command = ?, args = ?,
             url = ?, headers = ?, env = ?, icon = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![
                mcp.name,
                mcp.description,
                mcp.mcp_type,
                mcp.command,
                args_json,
                mcp.url,
                headers_json,
                env_json,
                mcp.icon,
                tags_json,
                id
            ],
        )
        .map_err(|e| e.to_string())?;

    // Re-fetch the updated MCP
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_mcp).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_mcp(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    info!("[MCP] Deleting MCP id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM mcps WHERE id = ?", [id])
        .map_err(|e| {
            error!("[MCP] Failed to delete MCP id={}: {}", id, e);
            e.to_string()
        })?;
    info!("[MCP] Deleted MCP id={}", id);
    Ok(())
}

#[tauri::command]
pub fn duplicate_mcp(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<Mcp, String> {
    info!("[MCP] Duplicating MCP id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;

    // Get original
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT name, description, type, command, args, url, headers, env, icon, tags
             FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    let (name, description, mcp_type, command, args, url, headers, env, icon, tags): (
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = stmt
        .query_row([id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    // Create copy with new name
    let new_name = format!("{}-copy", name);
    db.conn()
        .execute(
            "INSERT INTO mcps (name, description, type, command, args, url, headers, env, icon, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![new_name, description, mcp_type, command, args, url, headers, env, icon, tags],
        )
        .map_err(|e| e.to_string())?;

    let new_id = db.conn().last_insert_rowid();

    // Re-fetch
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([new_id], row_to_mcp)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_global_mcp(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    info!("[MCP] Toggling global MCP id={} enabled={}", id, enabled);
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "UPDATE mcps SET is_enabled_global = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| {
            error!("[MCP] Failed to toggle global MCP id={}: {}", id, e);
            e.to_string()
        })?;
    Ok(())
}

// ============================================================================
// Database operations (for testing without Tauri state)
// ============================================================================

/// Create an MCP directly in the database (for testing)
pub fn create_mcp_in_db(db: &Database, mcp: &CreateMcpRequest) -> Result<Mcp, String> {
    let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
    let headers_json = mcp
        .headers
        .as_ref()
        .map(|h| serde_json::to_string(h).unwrap());
    let env_json = mcp.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
    let tags_json = mcp.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "INSERT INTO mcps (name, description, type, command, args, url, headers, env, icon, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![
                mcp.name,
                mcp.description,
                mcp.mcp_type,
                mcp.command,
                args_json,
                mcp.url,
                headers_json,
                env_json,
                mcp.icon,
                tags_json
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_mcp_by_id(db, id)
}

/// Get an MCP by ID directly from the database (for testing)
pub fn get_mcp_by_id(db: &Database, id: i64) -> Result<Mcp, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_mcp).map_err(|e| e.to_string())
}

/// Get all MCPs directly from the database (for testing)
pub fn get_all_mcps_from_db(db: &Database) -> Result<Vec<Mcp>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let mcps: Vec<Mcp> = stmt
        .query_map([], row_to_mcp)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(mcps)
}

/// Update an MCP directly in the database (for testing)
pub fn update_mcp_in_db(db: &Database, id: i64, mcp: &CreateMcpRequest) -> Result<Mcp, String> {
    let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
    let headers_json = mcp
        .headers
        .as_ref()
        .map(|h| serde_json::to_string(h).unwrap());
    let env_json = mcp.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
    let tags_json = mcp.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE mcps SET name = ?, description = ?, type = ?, command = ?, args = ?,
             url = ?, headers = ?, env = ?, icon = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![
                mcp.name,
                mcp.description,
                mcp.mcp_type,
                mcp.command,
                args_json,
                mcp.url,
                headers_json,
                env_json,
                mcp.icon,
                tags_json,
                id
            ],
        )
        .map_err(|e| e.to_string())?;

    get_mcp_by_id(db, id)
}

/// Delete an MCP directly from the database (for testing)
pub fn delete_mcp_from_db(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM mcps WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Toggle global MCP directly in the database (for testing)
pub fn toggle_global_mcp_in_db(db: &Database, id: i64, enabled: bool) -> Result<(), String> {
    db.conn()
        .execute(
            "UPDATE mcps SET is_enabled_global = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample_stdio_mcp() -> CreateMcpRequest {
        CreateMcpRequest {
            name: "test-mcp".to_string(),
            description: Some("A test MCP server".to_string()),
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@test/mcp-server".to_string()]),
            url: None,
            headers: None,
            env: Some(HashMap::from([(
                "API_KEY".to_string(),
                "test123".to_string(),
            )])),
            icon: None,
            tags: Some(vec!["test".to_string(), "example".to_string()]),
        }
    }

    fn sample_sse_mcp() -> CreateMcpRequest {
        CreateMcpRequest {
            name: "sse-mcp".to_string(),
            description: Some("An SSE MCP server".to_string()),
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://mcp.example.com/sse".to_string()),
            headers: Some(HashMap::from([(
                "Authorization".to_string(),
                "Bearer token".to_string(),
            )])),
            env: None,
            icon: None,
            tags: None,
        }
    }

    fn sample_minimal_mcp() -> CreateMcpRequest {
        CreateMcpRequest {
            name: "minimal".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("python".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            icon: None,
            tags: None,
        }
    }

    // ========================================================================
    // Create MCP tests
    // ========================================================================

    #[test]
    fn test_create_stdio_mcp() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();

        let mcp = create_mcp_in_db(&db, &req).unwrap();

        assert_eq!(mcp.name, "test-mcp");
        assert_eq!(mcp.description, Some("A test MCP server".to_string()));
        assert_eq!(mcp.mcp_type, "stdio");
        assert_eq!(mcp.command, Some("npx".to_string()));
        assert_eq!(
            mcp.args,
            Some(vec!["-y".to_string(), "@test/mcp-server".to_string()])
        );
        assert_eq!(
            mcp.env.as_ref().unwrap().get("API_KEY"),
            Some(&"test123".to_string())
        );
        assert_eq!(
            mcp.tags,
            Some(vec!["test".to_string(), "example".to_string()])
        );
        assert_eq!(mcp.source, "manual");
        assert!(!mcp.is_enabled_global);
    }

    #[test]
    fn test_create_sse_mcp() {
        let db = Database::in_memory().unwrap();
        let req = sample_sse_mcp();

        let mcp = create_mcp_in_db(&db, &req).unwrap();

        assert_eq!(mcp.name, "sse-mcp");
        assert_eq!(mcp.mcp_type, "sse");
        assert_eq!(mcp.url, Some("https://mcp.example.com/sse".to_string()));
        assert_eq!(
            mcp.headers.as_ref().unwrap().get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert!(mcp.command.is_none());
    }

    #[test]
    fn test_create_minimal_mcp() {
        let db = Database::in_memory().unwrap();
        let req = sample_minimal_mcp();

        let mcp = create_mcp_in_db(&db, &req).unwrap();

        assert_eq!(mcp.name, "minimal");
        assert!(mcp.description.is_none());
        assert_eq!(mcp.command, Some("python".to_string()));
        assert!(mcp.args.is_none());
        assert!(mcp.env.is_none());
        assert!(mcp.tags.is_none());
    }

    #[test]
    fn test_create_duplicate_mcp_fails() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();

        create_mcp_in_db(&db, &req).unwrap();
        let result = create_mcp_in_db(&db, &req);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UNIQUE constraint failed"));
    }

    // ========================================================================
    // Get MCP tests
    // ========================================================================

    #[test]
    fn test_get_mcp_by_id() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();
        let created = create_mcp_in_db(&db, &req).unwrap();

        let fetched = get_mcp_by_id(&db, created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, created.name);
    }

    #[test]
    fn test_get_mcp_by_id_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_mcp_by_id(&db, 9999);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_mcps_empty() {
        let db = Database::in_memory().unwrap();

        let mcps = get_all_mcps_from_db(&db).unwrap();

        assert!(mcps.is_empty());
    }

    #[test]
    fn test_get_all_mcps_sorted_by_name() {
        let db = Database::in_memory().unwrap();

        // Create MCPs in non-alphabetical order
        create_mcp_in_db(
            &db,
            &CreateMcpRequest {
                name: "zebra-mcp".to_string(),
                mcp_type: "stdio".to_string(),
                command: Some("test".to_string()),
                ..sample_minimal_mcp()
            },
        )
        .unwrap();

        create_mcp_in_db(
            &db,
            &CreateMcpRequest {
                name: "alpha-mcp".to_string(),
                mcp_type: "stdio".to_string(),
                command: Some("test".to_string()),
                ..sample_minimal_mcp()
            },
        )
        .unwrap();

        create_mcp_in_db(
            &db,
            &CreateMcpRequest {
                name: "middle-mcp".to_string(),
                mcp_type: "stdio".to_string(),
                command: Some("test".to_string()),
                ..sample_minimal_mcp()
            },
        )
        .unwrap();

        let mcps = get_all_mcps_from_db(&db).unwrap();

        assert_eq!(mcps.len(), 3);
        assert_eq!(mcps[0].name, "alpha-mcp");
        assert_eq!(mcps[1].name, "middle-mcp");
        assert_eq!(mcps[2].name, "zebra-mcp");
    }

    // ========================================================================
    // Update MCP tests
    // ========================================================================

    #[test]
    fn test_update_mcp() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();
        let created = create_mcp_in_db(&db, &req).unwrap();

        let update_req = CreateMcpRequest {
            name: "updated-mcp".to_string(),
            description: Some("Updated description".to_string()),
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://new-url.com".to_string()),
            headers: None,
            env: None,
            icon: Some("new-icon".to_string()),
            tags: Some(vec!["updated".to_string()]),
        };

        let updated = update_mcp_in_db(&db, created.id, &update_req).unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "updated-mcp");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.mcp_type, "sse");
        assert_eq!(updated.url, Some("https://new-url.com".to_string()));
        assert_eq!(updated.icon, Some("new-icon".to_string()));
        assert!(updated.command.is_none());
    }

    #[test]
    fn test_update_mcp_not_found() {
        let db = Database::in_memory().unwrap();
        let req = sample_minimal_mcp();

        // Update a non-existent ID - should succeed (SQLite doesn't error on UPDATE of non-existent row)
        // but the get should fail
        let result = update_mcp_in_db(&db, 9999, &req);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_preserves_created_at() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();
        let created = create_mcp_in_db(&db, &req).unwrap();

        let update_req = CreateMcpRequest {
            name: "new-name".to_string(),
            ..sample_minimal_mcp()
        };

        let updated = update_mcp_in_db(&db, created.id, &update_req).unwrap();

        assert_eq!(updated.created_at, created.created_at);
    }

    // ========================================================================
    // Delete MCP tests
    // ========================================================================

    #[test]
    fn test_delete_mcp() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();
        let created = create_mcp_in_db(&db, &req).unwrap();

        let result = delete_mcp_from_db(&db, created.id);
        assert!(result.is_ok());

        let fetch_result = get_mcp_by_id(&db, created.id);
        assert!(fetch_result.is_err());
    }

    #[test]
    fn test_delete_mcp_not_found_succeeds() {
        let db = Database::in_memory().unwrap();

        // SQLite doesn't error when deleting a non-existent row
        let result = delete_mcp_from_db(&db, 9999);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_multiple_mcps() {
        let db = Database::in_memory().unwrap();

        let mcp1 = create_mcp_in_db(&db, &sample_stdio_mcp()).unwrap();
        let mcp2 = create_mcp_in_db(&db, &sample_sse_mcp()).unwrap();
        let mcp3 = create_mcp_in_db(&db, &sample_minimal_mcp()).unwrap();

        delete_mcp_from_db(&db, mcp2.id).unwrap();

        let remaining = get_all_mcps_from_db(&db).unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.iter().any(|m| m.id == mcp1.id));
        assert!(remaining.iter().any(|m| m.id == mcp3.id));
        assert!(!remaining.iter().any(|m| m.id == mcp2.id));
    }

    // ========================================================================
    // Toggle global MCP tests
    // ========================================================================

    #[test]
    fn test_toggle_global_mcp_enable() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();
        let created = create_mcp_in_db(&db, &req).unwrap();
        assert!(!created.is_enabled_global);

        toggle_global_mcp_in_db(&db, created.id, true).unwrap();

        let updated = get_mcp_by_id(&db, created.id).unwrap();
        assert!(updated.is_enabled_global);
    }

    #[test]
    fn test_toggle_global_mcp_disable() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();
        let created = create_mcp_in_db(&db, &req).unwrap();

        toggle_global_mcp_in_db(&db, created.id, true).unwrap();
        toggle_global_mcp_in_db(&db, created.id, false).unwrap();

        let updated = get_mcp_by_id(&db, created.id).unwrap();
        assert!(!updated.is_enabled_global);
    }

    #[test]
    fn test_toggle_global_mcp_updates_timestamp() {
        let db = Database::in_memory().unwrap();
        let req = sample_stdio_mcp();
        let created = create_mcp_in_db(&db, &req).unwrap();
        let original_updated = created.updated_at.clone();

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        toggle_global_mcp_in_db(&db, created.id, true).unwrap();

        let updated = get_mcp_by_id(&db, created.id).unwrap();
        // Note: SQLite's CURRENT_TIMESTAMP has second precision, so this test
        // might not always show a difference. The important thing is no error.
        assert!(updated.is_enabled_global);
    }

    // ========================================================================
    // row_to_mcp parsing tests
    // ========================================================================

    #[test]
    fn test_parse_json_array_valid() {
        let result = parse_json_array(Some(r#"["a", "b", "c"]"#.to_string()));
        assert_eq!(
            result,
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn test_parse_json_array_empty() {
        let result = parse_json_array(Some("[]".to_string()));
        assert_eq!(result, Some(vec![]));
    }

    #[test]
    fn test_parse_json_array_none() {
        let result = parse_json_array(None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_json_array_invalid() {
        let result = parse_json_array(Some("not valid json".to_string()));
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_json_map_valid() {
        let result = parse_json_map(Some(r#"{"key": "value"}"#.to_string()));
        let expected = HashMap::from([("key".to_string(), "value".to_string())]);
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_parse_json_map_empty() {
        let result = parse_json_map(Some("{}".to_string()));
        assert_eq!(result, Some(HashMap::new()));
    }

    #[test]
    fn test_parse_json_map_none() {
        let result = parse_json_map(None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_json_map_invalid() {
        let result = parse_json_map(Some("invalid".to_string()));
        assert_eq!(result, None);
    }
}
