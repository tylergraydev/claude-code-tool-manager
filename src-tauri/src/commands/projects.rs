use crate::db::{CreateProjectRequest, Database, Mcp, Project, ProjectMcp};
use crate::services::config_writer;
use rusqlite::params;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

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
pub fn get_all_projects(db: State<'_, Mutex<Database>>) -> Result<Vec<Project>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Get all projects
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, path, has_mcp_file, has_settings_file, last_scanned_at, created_at, updated_at
             FROM projects ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let projects: Vec<Project> = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                has_mcp_file: row.get::<_, i32>(3)? != 0,
                has_settings_file: row.get::<_, i32>(4)? != 0,
                last_scanned_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                assigned_mcps: vec![],
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Get assignments for each project
    let mut result = Vec::new();
    for mut project in projects {
        let mut stmt = db
            .conn()
            .prepare(
                "SELECT pm.id, pm.mcp_id, pm.is_enabled, pm.env_overrides, pm.display_order,
                        m.id, m.name, m.description, m.type, m.command, m.args, m.url, m.headers, m.env,
                        m.icon, m.tags, m.source, m.source_path, m.is_enabled_global, m.created_at, m.updated_at
                 FROM project_mcps pm
                 JOIN mcps m ON pm.mcp_id = m.id
                 WHERE pm.project_id = ?
                 ORDER BY pm.display_order",
            )
            .map_err(|e| e.to_string())?;

        project.assigned_mcps = stmt
            .query_map([project.id], |row| {
                Ok(ProjectMcp {
                    id: row.get(0)?,
                    mcp_id: row.get(1)?,
                    is_enabled: row.get::<_, i32>(2)? != 0,
                    env_overrides: parse_json_map(row.get(3)?),
                    display_order: row.get(4)?,
                    mcp: row_to_mcp(row, 5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        result.push(project);
    }

    Ok(result)
}

#[tauri::command]
pub fn add_project(
    db: State<'_, Mutex<Database>>,
    project: CreateProjectRequest,
) -> Result<Project, String> {
    use crate::utils::paths::get_claude_paths;

    let db = db.lock().map_err(|e| e.to_string())?;

    // Check if .claude/.mcp.json exists
    let project_path = PathBuf::from(&project.path);
    let mcp_file = project_path.join(".claude").join(".mcp.json");
    let settings_file = project_path.join(".claude").join("settings.local.json");

    let has_mcp_file = mcp_file.exists();
    let has_settings_file = settings_file.exists();

    db.conn()
        .execute(
            "INSERT INTO projects (name, path, has_mcp_file, has_settings_file)
             VALUES (?, ?, ?, ?)",
            params![project.name, project.path, has_mcp_file as i32, has_settings_file as i32],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();

    // Register project in claude.json (even with no MCPs)
    if let Ok(paths) = get_claude_paths() {
        let empty_mcps: Vec<config_writer::McpWithEnabledTuple> = vec![];
        let _ = config_writer::write_project_to_claude_json(&paths, &project.path, &empty_mcps);
    }

    Ok(Project {
        id,
        name: project.name,
        path: project.path,
        has_mcp_file,
        has_settings_file,
        last_scanned_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        assigned_mcps: vec![],
    })
}

#[tauri::command]
pub fn remove_project(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM projects WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn browse_for_project(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .pick_folder(move |folder| {
            let _ = tx.send(folder.map(|f| f.to_string()));
        });

    rx.recv()
        .map_err(|e| e.to_string())?
        .map(|s| Ok(Some(s)))
        .unwrap_or(Ok(None))
}

#[tauri::command]
pub fn assign_mcp_to_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    mcp_id: i64,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Get next display order
    let order: i32 = db
        .conn()
        .query_row(
            "SELECT COALESCE(MAX(display_order), 0) + 1 FROM project_mcps WHERE project_id = ?",
            [project_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    db.conn()
        .execute(
            "INSERT OR IGNORE INTO project_mcps (project_id, mcp_id, display_order) VALUES (?, ?, ?)",
            params![project_id, mcp_id, order],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_mcp_from_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    mcp_id: i64,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "DELETE FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
            params![project_id, mcp_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_project_mcp(
    db: State<'_, Mutex<Database>>,
    assignment_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "UPDATE project_mcps SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, assignment_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn sync_project_config(db: State<'_, Mutex<Database>>, project_id: i64) -> Result<(), String> {
    use crate::utils::paths::get_claude_paths;

    let db = db.lock().map_err(|e| e.to_string())?;

    // Get project path
    let path: String = db
        .conn()
        .query_row("SELECT path FROM projects WHERE id = ?", [project_id], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    // Get ALL MCPs for this project (including disabled ones for claude.json)
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT m.name, m.type, m.command, m.args, m.url, m.headers, m.env, pm.is_enabled
             FROM project_mcps pm
             JOIN mcps m ON pm.mcp_id = m.id
             WHERE pm.project_id = ?
             ORDER BY pm.display_order",
        )
        .map_err(|e| e.to_string())?;

    let mcps_with_enabled: Vec<(String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, bool)> = stmt
        .query_map([project_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get::<_, i32>(7)? != 0,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Write to claude.json (includes disabled state)
    let paths = get_claude_paths().map_err(|e| e.to_string())?;
    config_writer::write_project_to_claude_json(&paths, &path, &mcps_with_enabled)
        .map_err(|e| e.to_string())?;

    // Also write .mcp.json for enabled MCPs only (legacy support)
    let enabled_mcps: Vec<_> = mcps_with_enabled
        .iter()
        .filter(|(_, _, _, _, _, _, _, enabled)| *enabled)
        .map(|(n, t, cmd, args, url, headers, env, _)| {
            (n.clone(), t.clone(), cmd.clone(), args.clone(), url.clone(), headers.clone(), env.clone())
        })
        .collect();

    let project_path = PathBuf::from(&path);
    config_writer::write_project_config(&project_path, &enabled_mcps).map_err(|e| e.to_string())?;

    // Update has_mcp_file flag
    db.conn()
        .execute(
            "UPDATE projects SET has_mcp_file = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            [project_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}
