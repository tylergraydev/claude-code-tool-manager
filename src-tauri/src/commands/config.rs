use crate::db::{ClaudePaths, Database, GlobalMcp, Mcp};
use crate::services::config_writer;
use crate::utils::paths;
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::State;

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn parse_json_map(s: Option<String>) -> Option<std::collections::HashMap<String, String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn row_to_mcp(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Mcp> {
    Ok(Mcp {
        id: row.get(offset)?,
        name: row.get(offset + 1)?,
        description: row.get(offset + 2)?,
        mcp_type: row.get(offset + 3)?,
        command: row.get(offset + 4)?,
        args: parse_json_array(row.get(offset + 5)?),
        url: row.get(offset + 6)?,
        headers: parse_json_map(row.get(offset + 7)?),
        env: parse_json_map(row.get(offset + 8)?),
        icon: row.get(offset + 9)?,
        tags: parse_json_array(row.get(offset + 10)?),
        source: row.get(offset + 11)?,
        source_path: row.get(offset + 12)?,
        is_enabled_global: row.get::<_, i32>(offset + 13)? != 0,
        is_favorite: row.get::<_, i32>(offset + 14)? != 0,
        created_at: row.get(offset + 15)?,
        updated_at: row.get(offset + 16)?,
    })
}

#[tauri::command]
pub fn get_global_mcps(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<GlobalMcp>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT gm.id, gm.mcp_id, gm.is_enabled, gm.env_overrides,
                    m.id, m.name, m.description, m.type, m.command, m.args, m.url, m.headers, m.env,
                    m.icon, m.tags, m.source, m.source_path, m.is_enabled_global, m.is_favorite, m.created_at, m.updated_at
             FROM global_mcps gm
             JOIN mcps m ON gm.mcp_id = m.id
             ORDER BY gm.display_order",
        )
        .map_err(|e| e.to_string())?;

    let global_mcps = stmt
        .query_map([], |row| {
            Ok(GlobalMcp {
                id: row.get(0)?,
                mcp_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                env_overrides: parse_json_map(row.get(3)?),
                mcp: row_to_mcp(row, 4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(global_mcps)
}

#[tauri::command]
pub fn add_global_mcp(db: State<'_, Arc<Mutex<Database>>>, mcp_id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let order: i32 = db
        .conn()
        .query_row(
            "SELECT COALESCE(MAX(display_order), 0) + 1 FROM global_mcps",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    db.conn()
        .execute(
            "INSERT OR IGNORE INTO global_mcps (mcp_id, display_order) VALUES (?, ?)",
            params![mcp_id, order],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_global_mcp(db: State<'_, Arc<Mutex<Database>>>, mcp_id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM global_mcps WHERE mcp_id = ?", [mcp_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_global_mcp_assignment(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "UPDATE global_mcps SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn sync_global_config(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    sync_global_config_from_db(&db)
}

/// Sync global config from database to disk (reusable helper without Tauri State)
pub fn sync_global_config_from_db(db: &Database) -> Result<(), String> {
    use crate::commands::settings::get_enabled_editors_from_db;
    use crate::services::{
        codex_config, copilot_config, cursor_config, gemini_config, opencode_config,
    };
    use crate::utils::{codex_paths, copilot_paths, cursor_paths, gemini_paths, opencode_paths};
    use log::{info, warn};

    // Get enabled global MCPs
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT m.name, m.type, m.command, m.args, m.url, m.headers, m.env
             FROM global_mcps gm
             JOIN mcps m ON gm.mcp_id = m.id
             WHERE gm.is_enabled = 1
             ORDER BY gm.display_order",
        )
        .map_err(|e| e.to_string())?;

    let mcps: Vec<(
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = stmt
        .query_map([], |row| {
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
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Write to all enabled editors
    let enabled_editors = get_enabled_editors_from_db(db);
    for editor in &enabled_editors {
        match editor.as_str() {
            "claude_code" => {
                let claude_paths = paths::get_claude_paths().map_err(|e| e.to_string())?;
                config_writer::write_global_config(&claude_paths, &mcps)
                    .map_err(|e| e.to_string())?;
                info!("[Config] Wrote global config to Claude Code");
            }
            "opencode" => {
                if let Ok(paths) = opencode_paths::get_opencode_paths() {
                    opencode_config::write_opencode_global_config(&paths.config_file, &mcps)
                        .map_err(|e| e.to_string())?;
                    info!("[Config] Wrote global config to OpenCode");
                }
            }
            "codex" => {
                if let Ok(paths) = codex_paths::get_codex_paths() {
                    codex_config::write_codex_config(&paths.config_file, &mcps)
                        .map_err(|e| e.to_string())?;
                    info!("[Config] Wrote global config to Codex CLI");
                }
            }
            "copilot" => {
                if let Ok(paths) = copilot_paths::get_copilot_paths() {
                    copilot_config::write_copilot_config(&paths.mcp_config_file, &mcps)
                        .map_err(|e| e.to_string())?;
                    info!("[Config] Wrote global config to Copilot CLI");
                }
            }
            "cursor" => {
                if let Ok(paths) = cursor_paths::get_cursor_paths() {
                    cursor_config::write_cursor_config(&paths.mcp_config_file, &mcps)
                        .map_err(|e| e.to_string())?;
                    info!("[Config] Wrote global config to Cursor");
                }
            }
            "gemini" => {
                if let Ok(paths) = gemini_paths::get_gemini_paths() {
                    gemini_config::write_gemini_config(&paths.settings_file, &mcps)
                        .map_err(|e| e.to_string())?;
                    info!("[Config] Wrote global config to Gemini CLI");
                }
            }
            unknown => warn!("[Config] Unknown editor type '{}'. Skipping.", unknown),
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_claude_paths() -> Result<ClaudePaths, String> {
    let claude_paths = paths::get_claude_paths().map_err(|e| e.to_string())?;
    Ok(ClaudePaths {
        claude_dir: claude_paths.claude_dir.to_string_lossy().to_string(),
        claude_json: claude_paths.claude_json.to_string_lossy().to_string(),
        global_settings: claude_paths.global_settings.to_string_lossy().to_string(),
        plugins_dir: claude_paths.plugins_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn open_config_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn backup_configs() -> Result<(), String> {
    let claude_paths = paths::get_claude_paths().map_err(|e| e.to_string())?;

    let backup_dir = claude_paths.claude_dir.join("backups");
    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("backup_{}", timestamp);
    let backup_path = backup_dir.join(&backup_name);
    std::fs::create_dir_all(&backup_path).map_err(|e| e.to_string())?;

    // Copy claude.json if exists (main config with global MCPs)
    if claude_paths.claude_json.exists() {
        let dest = backup_path.join("claude.json");
        std::fs::copy(&claude_paths.claude_json, dest).map_err(|e| e.to_string())?;
    }

    // Copy settings.json if exists
    if claude_paths.global_settings.exists() {
        let dest = backup_path.join("settings.json");
        std::fs::copy(&claude_paths.global_settings, dest).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Add a global MCP directly in the database (for testing)
#[cfg(test)]
pub fn add_global_mcp_in_db(db: &Database, mcp_id: i64) -> Result<(), String> {
    let order: i32 = db
        .conn()
        .query_row(
            "SELECT COALESCE(MAX(display_order), 0) + 1 FROM global_mcps",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    db.conn()
        .execute(
            "INSERT OR IGNORE INTO global_mcps (mcp_id, display_order) VALUES (?, ?)",
            params![mcp_id, order],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Remove a global MCP directly from the database (for testing)
#[cfg(test)]
pub fn remove_global_mcp_from_db(db: &Database, mcp_id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM global_mcps WHERE mcp_id = ?", [mcp_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Toggle a global MCP directly in the database (for testing)
#[cfg(test)]
pub fn toggle_global_mcp_in_db(db: &Database, id: i64, enabled: bool) -> Result<(), String> {
    db.conn()
        .execute(
            "UPDATE global_mcps SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get all global MCPs directly from the database (for testing)
#[cfg(test)]
pub fn get_global_mcps_from_db(db: &Database) -> Result<Vec<GlobalMcp>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT gm.id, gm.mcp_id, gm.is_enabled, gm.env_overrides,
                    m.id, m.name, m.description, m.type, m.command, m.args, m.url, m.headers, m.env,
                    m.icon, m.tags, m.source, m.source_path, m.is_enabled_global, m.is_favorite, m.created_at, m.updated_at
             FROM global_mcps gm
             JOIN mcps m ON gm.mcp_id = m.id
             ORDER BY gm.display_order",
        )
        .map_err(|e| e.to_string())?;

    let global_mcps = stmt
        .query_map([], |row| {
            Ok(GlobalMcp {
                id: row.get(0)?,
                mcp_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                env_overrides: parse_json_map(row.get(3)?),
                mcp: row_to_mcp(row, 4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(global_mcps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::mcp::create_mcp_in_db;
    use crate::db::models::CreateMcpRequest;

    fn create_test_mcp(db: &Database, name: &str) -> i64 {
        let mcp = CreateMcpRequest {
            name: name.to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            icon: None,
            tags: None,
        };
        create_mcp_in_db(db, &mcp).unwrap().id
    }

    // =========================================================================
    // Global MCP CRUD tests
    // =========================================================================

    #[test]
    fn test_add_global_mcp() {
        let db = Database::in_memory().unwrap();
        let mcp_id = create_test_mcp(&db, "global-mcp");

        add_global_mcp_in_db(&db, mcp_id).unwrap();

        let global_mcps = get_global_mcps_from_db(&db).unwrap();

        assert_eq!(global_mcps.len(), 1);
        assert_eq!(global_mcps[0].mcp_id, mcp_id);
        assert!(global_mcps[0].is_enabled); // Default enabled
    }

    #[test]
    fn test_add_multiple_global_mcps() {
        let db = Database::in_memory().unwrap();

        let mcp1 = create_test_mcp(&db, "mcp-1");
        let mcp2 = create_test_mcp(&db, "mcp-2");
        let mcp3 = create_test_mcp(&db, "mcp-3");

        add_global_mcp_in_db(&db, mcp1).unwrap();
        add_global_mcp_in_db(&db, mcp2).unwrap();
        add_global_mcp_in_db(&db, mcp3).unwrap();

        let global_mcps = get_global_mcps_from_db(&db).unwrap();

        assert_eq!(global_mcps.len(), 3);
    }

    #[test]
    fn test_remove_global_mcp() {
        let db = Database::in_memory().unwrap();
        let mcp_id = create_test_mcp(&db, "removable");

        add_global_mcp_in_db(&db, mcp_id).unwrap();

        // Verify it's there
        let mcps = get_global_mcps_from_db(&db).unwrap();
        assert_eq!(mcps.len(), 1);

        // Remove
        remove_global_mcp_from_db(&db, mcp_id).unwrap();

        // Verify it's gone
        let mcps = get_global_mcps_from_db(&db).unwrap();
        assert!(mcps.is_empty());
    }

    #[test]
    fn test_toggle_global_mcp() {
        let db = Database::in_memory().unwrap();
        let mcp_id = create_test_mcp(&db, "toggleable");

        add_global_mcp_in_db(&db, mcp_id).unwrap();

        let mcps = get_global_mcps_from_db(&db).unwrap();
        let global_id = mcps[0].id;

        // Disable
        toggle_global_mcp_in_db(&db, global_id, false).unwrap();
        let mcps = get_global_mcps_from_db(&db).unwrap();
        assert!(!mcps[0].is_enabled);

        // Re-enable
        toggle_global_mcp_in_db(&db, global_id, true).unwrap();
        let mcps = get_global_mcps_from_db(&db).unwrap();
        assert!(mcps[0].is_enabled);
    }

    #[test]
    fn test_add_duplicate_global_mcp_ignored() {
        let db = Database::in_memory().unwrap();
        let mcp_id = create_test_mcp(&db, "dup-mcp");

        // Add twice
        add_global_mcp_in_db(&db, mcp_id).unwrap();
        add_global_mcp_in_db(&db, mcp_id).unwrap();

        // Should only have one
        let mcps = get_global_mcps_from_db(&db).unwrap();
        assert_eq!(mcps.len(), 1);
    }

    #[test]
    fn test_global_mcp_contains_mcp_details() {
        let db = Database::in_memory().unwrap();

        // Create MCP with specific details
        let mcp = CreateMcpRequest {
            name: "detailed-mcp".to_string(),
            description: Some("A detailed MCP".to_string()),
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://example.com".to_string()),
            headers: None,
            env: None,
            icon: None,
            tags: None,
        };
        let mcp_id = create_mcp_in_db(&db, &mcp).unwrap().id;

        add_global_mcp_in_db(&db, mcp_id).unwrap();

        let global_mcps = get_global_mcps_from_db(&db).unwrap();

        assert_eq!(global_mcps[0].mcp.name, "detailed-mcp");
        assert_eq!(
            global_mcps[0].mcp.description,
            Some("A detailed MCP".to_string())
        );
        assert_eq!(global_mcps[0].mcp.mcp_type, "sse");
        assert_eq!(
            global_mcps[0].mcp.url,
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_remove_global_mcp_does_not_delete_mcp() {
        let db = Database::in_memory().unwrap();
        let mcp_id = create_test_mcp(&db, "persistent-mcp");

        add_global_mcp_in_db(&db, mcp_id).unwrap();
        remove_global_mcp_from_db(&db, mcp_id).unwrap();

        // MCP should still exist
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM mcps WHERE id = ?", [mcp_id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }
}
