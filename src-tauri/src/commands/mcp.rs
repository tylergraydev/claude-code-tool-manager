use crate::db::{CreateMcpRequest, Database, Mcp};
use rusqlite::params;
use std::sync::Mutex;
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
pub fn get_all_mcps(db: State<'_, Mutex<Database>>) -> Result<Vec<Mcp>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let mcps = stmt
        .query_map([], row_to_mcp)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(mcps)
}

#[tauri::command]
pub fn get_mcp(db: State<'_, Mutex<Database>>, id: i64) -> Result<Mcp, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, created_at, updated_at
             FROM mcps WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_mcp)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_mcp(db: State<'_, Mutex<Database>>, mcp: CreateMcpRequest) -> Result<Mcp, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
    let headers_json = mcp.headers.as_ref().map(|h| serde_json::to_string(h).unwrap());
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
    drop(db);

    get_mcp(State::from(&*State::clone(&db.into_inner())), id)
}

#[tauri::command]
pub fn update_mcp(
    db: State<'_, Mutex<Database>>,
    id: i64,
    mcp: CreateMcpRequest,
) -> Result<Mcp, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
    let headers_json = mcp.headers.as_ref().map(|h| serde_json::to_string(h).unwrap());
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

    stmt.query_row([id], row_to_mcp)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_mcp(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM mcps WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn duplicate_mcp(db: State<'_, Mutex<Database>>, id: i64) -> Result<Mcp, String> {
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
    db: State<'_, Mutex<Database>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "UPDATE mcps SET is_enabled_global = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}
