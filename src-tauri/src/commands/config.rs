use crate::db::{ClaudePaths, Database, GlobalMcp, Mcp};
use crate::services::config_writer;
use crate::utils::paths;
use rusqlite::params;
use std::sync::Mutex;
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
        created_at: row.get(offset + 14)?,
        updated_at: row.get(offset + 15)?,
    })
}

#[tauri::command]
pub fn get_global_mcps(db: State<'_, Mutex<Database>>) -> Result<Vec<GlobalMcp>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT gm.id, gm.mcp_id, gm.is_enabled, gm.env_overrides,
                    m.id, m.name, m.description, m.type, m.command, m.args, m.url, m.headers, m.env,
                    m.icon, m.tags, m.source, m.source_path, m.is_enabled_global, m.created_at, m.updated_at
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
pub fn add_global_mcp(db: State<'_, Mutex<Database>>, mcp_id: i64) -> Result<(), String> {
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
pub fn remove_global_mcp(db: State<'_, Mutex<Database>>, mcp_id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM global_mcps WHERE mcp_id = ?", [mcp_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_global_mcp_assignment(
    db: State<'_, Mutex<Database>>,
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
pub fn sync_global_config(db: State<'_, Mutex<Database>>) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

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

    let mcps: Vec<(String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)> = stmt
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

    let claude_paths = paths::get_claude_paths().map_err(|e| e.to_string())?;
    config_writer::write_global_config(&claude_paths, &mcps).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_claude_paths() -> Result<ClaudePaths, String> {
    let claude_paths = paths::get_claude_paths().map_err(|e| e.to_string())?;
    Ok(ClaudePaths {
        claude_dir: claude_paths.claude_dir.to_string_lossy().to_string(),
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

    // Copy settings.json if exists
    if claude_paths.global_settings.exists() {
        let dest = backup_path.join("settings.json");
        std::fs::copy(&claude_paths.global_settings, dest).map_err(|e| e.to_string())?;
    }

    Ok(())
}
