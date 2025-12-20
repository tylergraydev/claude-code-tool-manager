use crate::db::models::{CreateSubAgentRequest, SubAgent, GlobalSubAgent, ProjectSubAgent};
use crate::db::schema::Database;
use crate::services::subagent_writer;
use rusqlite::params;
use std::sync::Mutex;
use std::path::Path;
use tauri::State;

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn row_to_subagent(row: &rusqlite::Row) -> rusqlite::Result<SubAgent> {
    Ok(SubAgent {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        content: row.get(3)?,
        tools: parse_json_array(row.get(4)?),
        model: row.get(5)?,
        tags: parse_json_array(row.get(6)?),
        source: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn row_to_subagent_with_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<SubAgent> {
    Ok(SubAgent {
        id: row.get(offset)?,
        name: row.get(offset + 1)?,
        description: row.get(offset + 2)?,
        content: row.get(offset + 3)?,
        tools: parse_json_array(row.get(offset + 4)?),
        model: row.get(offset + 5)?,
        tags: parse_json_array(row.get(offset + 6)?),
        source: row.get(offset + 7)?,
        created_at: row.get(offset + 8)?,
        updated_at: row.get(offset + 9)?,
    })
}

#[tauri::command]
pub fn get_all_subagents(db: State<'_, Mutex<Database>>) -> Result<Vec<SubAgent>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, tags, source, created_at, updated_at
             FROM subagents ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let subagents = stmt
        .query_map([], row_to_subagent)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(subagents)
}

#[tauri::command]
pub fn create_subagent(db: State<'_, Mutex<Database>>, subagent: CreateSubAgentRequest) -> Result<SubAgent, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let tools_json = subagent.tools.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = subagent.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db_guard.conn()
        .execute(
            "INSERT INTO subagents (name, description, content, tools, model, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, 'manual')",
            params![subagent.name, subagent.description, subagent.content, tools_json, subagent.model, tags_json],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let mut stmt = db_guard
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, tags, source, created_at, updated_at
             FROM subagents WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_subagent)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_subagent(
    db: State<'_, Mutex<Database>>,
    id: i64,
    subagent: CreateSubAgentRequest,
) -> Result<SubAgent, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let tools_json = subagent.tools.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = subagent.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE subagents SET name = ?, description = ?, content = ?, tools = ?, model = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![subagent.name, subagent.description, subagent.content, tools_json, subagent.model, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, tags, source, created_at, updated_at
             FROM subagents WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_subagent)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_subagent(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Reset is_imported flag in repo_items for this subagent
    db.conn()
        .execute(
            "UPDATE repo_items SET is_imported = 0, imported_item_id = NULL WHERE imported_item_id = ? AND item_type = 'subagent'",
            [id],
        )
        .map_err(|e| e.to_string())?;

    db.conn()
        .execute("DELETE FROM subagents WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Global Sub-Agents
#[tauri::command]
pub fn get_global_subagents(db: State<'_, Mutex<Database>>) -> Result<Vec<GlobalSubAgent>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT gs.id, gs.subagent_id, gs.is_enabled,
                    s.id, s.name, s.description, s.content, s.tools, s.model, s.tags, s.source, s.created_at, s.updated_at
             FROM global_subagents gs
             JOIN subagents s ON gs.subagent_id = s.id
             ORDER BY s.name",
        )
        .map_err(|e| e.to_string())?;

    let subagents = stmt
        .query_map([], |row| {
            Ok(GlobalSubAgent {
                id: row.get(0)?,
                subagent_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                subagent: row_to_subagent_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(subagents)
}

#[tauri::command]
pub fn add_global_subagent(db: State<'_, Mutex<Database>>, subagent_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the subagent details for file writing
    let mut stmt = db_guard.conn()
        .prepare("SELECT id, name, description, content, tools, model, tags, source, created_at, updated_at FROM subagents WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let subagent: SubAgent = stmt.query_row([subagent_id], row_to_subagent)
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "INSERT OR IGNORE INTO global_subagents (subagent_id) VALUES (?)",
            [subagent_id],
        )
        .map_err(|e| e.to_string())?;

    // Write the subagent file to global config
    subagent_writer::write_global_subagent(&subagent)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_global_subagent(db: State<'_, Mutex<Database>>, subagent_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the subagent name for file deletion
    let name: String = db_guard.conn()
        .query_row("SELECT name FROM subagents WHERE id = ?", [subagent_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute("DELETE FROM global_subagents WHERE subagent_id = ?", [subagent_id])
        .map_err(|e| e.to_string())?;

    // Delete the subagent file from global config
    subagent_writer::delete_global_subagent(&name)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_global_subagent(db: State<'_, Mutex<Database>>, id: i64, enabled: bool) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "UPDATE global_subagents SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

    // Get the subagent details
    let mut stmt = db_guard.conn()
        .prepare(
            "SELECT s.id, s.name, s.description, s.content, s.tools, s.model, s.tags, s.source, s.created_at, s.updated_at
             FROM global_subagents gs
             JOIN subagents s ON gs.subagent_id = s.id
             WHERE gs.id = ?"
        )
        .map_err(|e| e.to_string())?;

    let subagent: SubAgent = stmt.query_row([id], row_to_subagent)
        .map_err(|e| e.to_string())?;

    // Write or delete the file based on enabled state
    if enabled {
        subagent_writer::write_global_subagent(&subagent)
            .map_err(|e| e.to_string())?;
    } else {
        subagent_writer::delete_global_subagent(&subagent.name)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Project Sub-Agents
#[tauri::command]
pub fn assign_subagent_to_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    subagent_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and subagent details
    let project_path: String = db_guard.conn()
        .query_row("SELECT path FROM projects WHERE id = ?", [project_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = db_guard.conn()
        .prepare("SELECT id, name, description, content, tools, model, tags, source, created_at, updated_at FROM subagents WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let subagent: SubAgent = stmt.query_row([subagent_id], row_to_subagent)
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "INSERT OR IGNORE INTO project_subagents (project_id, subagent_id) VALUES (?, ?)",
            params![project_id, subagent_id],
        )
        .map_err(|e| e.to_string())?;

    // Write the subagent file to project config
    subagent_writer::write_project_subagent(Path::new(&project_path), &subagent)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_subagent_from_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    subagent_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and subagent name
    let project_path: String = db_guard.conn()
        .query_row("SELECT path FROM projects WHERE id = ?", [project_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let name: String = db_guard.conn()
        .query_row("SELECT name FROM subagents WHERE id = ?", [subagent_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "DELETE FROM project_subagents WHERE project_id = ? AND subagent_id = ?",
            params![project_id, subagent_id],
        )
        .map_err(|e| e.to_string())?;

    // Delete the subagent file from project config
    subagent_writer::delete_project_subagent(Path::new(&project_path), &name)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_project_subagent(
    db: State<'_, Mutex<Database>>,
    assignment_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "UPDATE project_subagents SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, assignment_id],
        )
        .map_err(|e| e.to_string())?;

    // Get project path and subagent details
    let mut stmt = db_guard.conn()
        .prepare(
            "SELECT p.path, s.id, s.name, s.description, s.content, s.tools, s.model, s.tags, s.source, s.created_at, s.updated_at
             FROM project_subagents ps
             JOIN projects p ON ps.project_id = p.id
             JOIN subagents s ON ps.subagent_id = s.id
             WHERE ps.id = ?"
        )
        .map_err(|e| e.to_string())?;

    let (project_path, subagent): (String, SubAgent) = stmt.query_row([assignment_id], |row: &rusqlite::Row| {
        Ok((row.get(0)?, row_to_subagent_with_offset(row, 1)?))
    }).map_err(|e: rusqlite::Error| e.to_string())?;

    // Write or delete the file based on enabled state
    if enabled {
        subagent_writer::write_project_subagent(Path::new(&project_path), &subagent)
            .map_err(|e| e.to_string())?;
    } else {
        subagent_writer::delete_project_subagent(Path::new(&project_path), &subagent.name)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_project_subagents(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Vec<ProjectSubAgent>, String> {
    let db = db.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT ps.id, ps.subagent_id, ps.is_enabled,
                    s.id, s.name, s.description, s.content, s.tools, s.model, s.tags, s.source, s.created_at, s.updated_at
             FROM project_subagents ps
             JOIN subagents s ON ps.subagent_id = s.id
             WHERE ps.project_id = ?
             ORDER BY s.name",
        )
        .map_err(|e: rusqlite::Error| e.to_string())?;

    let subagents = stmt
        .query_map([project_id], |row: &rusqlite::Row| {
            Ok(ProjectSubAgent {
                id: row.get(0)?,
                subagent_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                subagent: row_to_subagent_with_offset(row, 3)?,
            })
        })
        .map_err(|e: rusqlite::Error| e.to_string())?
        .filter_map(|r: Result<ProjectSubAgent, rusqlite::Error>| r.ok())
        .collect();

    Ok(subagents)
}
