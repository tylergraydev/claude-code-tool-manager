use crate::db::{CreateProjectRequest, Database, Mcp, Project, ProjectMcp};
use crate::services::config_writer;
use log::{error, info};
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
    info!("[Projects] Loading all projects");
    let db = db.lock().map_err(|e| {
        error!("[Projects] Failed to acquire database lock: {}", e);
        e.to_string()
    })?;

    // Get all projects
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, path, has_mcp_file, has_settings_file, last_scanned_at, editor_type, created_at, updated_at
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
                editor_type: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "claude_code".to_string()),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
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

    info!("[Projects] Loaded {} projects", result.len());
    Ok(result)
}

#[tauri::command]
pub fn add_project(
    db: State<'_, Mutex<Database>>,
    project: CreateProjectRequest,
) -> Result<Project, String> {
    use crate::utils::paths::get_claude_paths;

    info!("[Projects] Adding project: {} at {}", project.name, project.path);
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
        editor_type: "claude_code".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        assigned_mcps: vec![],
    })
}

#[tauri::command]
pub fn remove_project(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    info!("[Projects] Removing project id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM projects WHERE id = ?", [id])
        .map_err(|e| {
            error!("[Projects] Failed to remove project id={}: {}", id, e);
            e.to_string()
        })?;
    info!("[Projects] Removed project id={}", id);
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
    use crate::services::opencode_config;
    use crate::utils::paths::get_claude_paths;

    info!("[Projects] Syncing config for project id={}", project_id);
    let db = db.lock().map_err(|e| e.to_string())?;

    // Get project path and editor_type
    let (path, editor_type): (String, String) = db
        .conn()
        .query_row(
            "SELECT path, COALESCE(editor_type, 'claude_code') FROM projects WHERE id = ?",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    // Get ALL MCPs for this project (including disabled ones)
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

    let project_path = PathBuf::from(&path);

    // Route to the correct config writer based on editor_type
    match editor_type.as_str() {
        "opencode" => {
            // Write to OpenCode format (opencode.json in project root)
            let enabled_mcps: Vec<_> = mcps_with_enabled
                .iter()
                .filter(|(_, _, _, _, _, _, _, enabled)| *enabled)
                .map(|(n, t, cmd, args, url, headers, env, _)| {
                    (n.clone(), t.clone(), cmd.clone(), args.clone(), url.clone(), headers.clone(), env.clone())
                })
                .collect();

            opencode_config::write_opencode_project_config(&project_path, &enabled_mcps)
                .map_err(|e| e.to_string())?;

            info!("[Projects] Wrote OpenCode config for project {}", project_id);
        }
        _ => {
            // Claude Code: Write to claude.json (includes disabled state)
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

            config_writer::write_project_config(&project_path, &enabled_mcps)
                .map_err(|e| e.to_string())?;

            info!("[Projects] Wrote Claude Code config for project {}", project_id);
        }
    }

    // Update has_mcp_file flag
    db.conn()
        .execute(
            "UPDATE projects SET has_mcp_file = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            [project_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Create a project directly in the database (for testing)
pub fn create_project_in_db(db: &Database, project: &CreateProjectRequest) -> Result<Project, String> {
    db.conn()
        .execute(
            "INSERT INTO projects (name, path, has_mcp_file, has_settings_file)
             VALUES (?, ?, 0, 0)",
            params![project.name, project.path],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_project_by_id(db, id)
}

/// Get a project by ID directly from the database (for testing)
pub fn get_project_by_id(db: &Database, id: i64) -> Result<Project, String> {
    db.conn()
        .query_row(
            "SELECT id, name, path, has_mcp_file, has_settings_file, last_scanned_at, editor_type, created_at, updated_at
             FROM projects WHERE id = ?",
            [id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    has_mcp_file: row.get::<_, i32>(3)? != 0,
                    has_settings_file: row.get::<_, i32>(4)? != 0,
                    last_scanned_at: row.get(5)?,
                    editor_type: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "claude_code".to_string()),
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    assigned_mcps: vec![],
                })
            },
        )
        .map_err(|e| e.to_string())
}

/// Get a project by path directly from the database (for testing)
pub fn get_project_by_path(db: &Database, path: &str) -> Result<Project, String> {
    db.conn()
        .query_row(
            "SELECT id, name, path, has_mcp_file, has_settings_file, last_scanned_at, editor_type, created_at, updated_at
             FROM projects WHERE path = ?",
            [path],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    has_mcp_file: row.get::<_, i32>(3)? != 0,
                    has_settings_file: row.get::<_, i32>(4)? != 0,
                    last_scanned_at: row.get(5)?,
                    editor_type: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "claude_code".to_string()),
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    assigned_mcps: vec![],
                })
            },
        )
        .map_err(|e| e.to_string())
}

/// Get all projects directly from the database (for testing)
pub fn get_all_projects_from_db(db: &Database) -> Result<Vec<Project>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, path, has_mcp_file, has_settings_file, last_scanned_at, editor_type, created_at, updated_at
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
                editor_type: row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "claude_code".to_string()),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                assigned_mcps: vec![],
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(projects)
}

/// Delete a project directly from the database (for testing)
pub fn delete_project_from_db(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM projects WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Assign an MCP to a project directly in the database (for testing)
pub fn assign_mcp_to_project_in_db(db: &Database, project_id: i64, mcp_id: i64) -> Result<(), String> {
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

/// Remove an MCP from a project directly in the database (for testing)
pub fn remove_mcp_from_project_in_db(db: &Database, project_id: i64, mcp_id: i64) -> Result<(), String> {
    db.conn()
        .execute(
            "DELETE FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
            params![project_id, mcp_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Toggle a project MCP directly in the database (for testing)
pub fn toggle_project_mcp_in_db(db: &Database, assignment_id: i64, enabled: bool) -> Result<(), String> {
    db.conn()
        .execute(
            "UPDATE project_mcps SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, assignment_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get project MCP assignments directly from the database (for testing)
pub fn get_project_mcps_from_db(db: &Database, project_id: i64) -> Result<Vec<ProjectMcp>, String> {
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

    let mcps = stmt
        .query_map([project_id], |row| {
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

    Ok(mcps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::mcp::create_mcp_in_db;
    use crate::db::models::CreateMcpRequest;

    // =========================================================================
    // Project CRUD tests
    // =========================================================================

    #[test]
    fn test_create_project() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Test Project".to_string(),
            path: "/path/to/project".to_string(),
        };

        let created = create_project_in_db(&db, &project).unwrap();

        assert_eq!(created.name, "Test Project");
        assert_eq!(created.path, "/path/to/project");
        assert!(!created.has_mcp_file);
        assert!(!created.has_settings_file);
        assert_eq!(created.editor_type, "claude_code");
    }

    #[test]
    fn test_get_project_by_id() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Findable Project".to_string(),
            path: "/findable/path".to_string(),
        };

        let created = create_project_in_db(&db, &project).unwrap();
        let found = get_project_by_id(&db, created.id).unwrap();

        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "Findable Project");
    }

    #[test]
    fn test_get_project_by_id_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_project_by_id(&db, 9999);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_project_by_path() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Path Project".to_string(),
            path: "/unique/project/path".to_string(),
        };

        create_project_in_db(&db, &project).unwrap();
        let found = get_project_by_path(&db, "/unique/project/path").unwrap();

        assert_eq!(found.path, "/unique/project/path");
        assert_eq!(found.name, "Path Project");
    }

    #[test]
    fn test_get_all_projects_from_db() {
        let db = Database::in_memory().unwrap();

        for i in 1..=3 {
            let project = CreateProjectRequest {
                name: format!("Project {}", i),
                path: format!("/path/{}", i),
            };
            create_project_in_db(&db, &project).unwrap();
        }

        let projects = get_all_projects_from_db(&db).unwrap();

        assert_eq!(projects.len(), 3);
        // Should be sorted by name
        assert_eq!(projects[0].name, "Project 1");
        assert_eq!(projects[1].name, "Project 2");
        assert_eq!(projects[2].name, "Project 3");
    }

    #[test]
    fn test_delete_project() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "To Delete".to_string(),
            path: "/delete/me".to_string(),
        };

        let created = create_project_in_db(&db, &project).unwrap();
        delete_project_from_db(&db, created.id).unwrap();

        let result = get_project_by_id(&db, created.id);
        assert!(result.is_err());
    }

    // =========================================================================
    // Project MCP Assignment tests
    // =========================================================================

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

    #[test]
    fn test_assign_mcp_to_project() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Project with MCP".to_string(),
            path: "/mcp/project".to_string(),
        };
        let created_project = create_project_in_db(&db, &project).unwrap();
        let mcp_id = create_test_mcp(&db, "test-mcp");

        assign_mcp_to_project_in_db(&db, created_project.id, mcp_id).unwrap();

        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();

        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].mcp_id, mcp_id);
        assert!(mcps[0].is_enabled);  // Default enabled
    }

    #[test]
    fn test_assign_multiple_mcps_to_project() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Multi MCP".to_string(),
            path: "/multi/mcp".to_string(),
        };
        let created_project = create_project_in_db(&db, &project).unwrap();

        let mcp1 = create_test_mcp(&db, "mcp-1");
        let mcp2 = create_test_mcp(&db, "mcp-2");
        let mcp3 = create_test_mcp(&db, "mcp-3");

        assign_mcp_to_project_in_db(&db, created_project.id, mcp1).unwrap();
        assign_mcp_to_project_in_db(&db, created_project.id, mcp2).unwrap();
        assign_mcp_to_project_in_db(&db, created_project.id, mcp3).unwrap();

        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();

        assert_eq!(mcps.len(), 3);
        // Should be in display order
        assert_eq!(mcps[0].display_order, 1);
        assert_eq!(mcps[1].display_order, 2);
        assert_eq!(mcps[2].display_order, 3);
    }

    #[test]
    fn test_remove_mcp_from_project() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Remove MCP".to_string(),
            path: "/remove/mcp".to_string(),
        };
        let created_project = create_project_in_db(&db, &project).unwrap();
        let mcp_id = create_test_mcp(&db, "removable");

        assign_mcp_to_project_in_db(&db, created_project.id, mcp_id).unwrap();
        remove_mcp_from_project_in_db(&db, created_project.id, mcp_id).unwrap();

        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();
        assert!(mcps.is_empty());
    }

    #[test]
    fn test_toggle_project_mcp() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Toggle MCP".to_string(),
            path: "/toggle/mcp".to_string(),
        };
        let created_project = create_project_in_db(&db, &project).unwrap();
        let mcp_id = create_test_mcp(&db, "toggleable");

        assign_mcp_to_project_in_db(&db, created_project.id, mcp_id).unwrap();

        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();
        let assignment_id = mcps[0].id;

        // Disable
        toggle_project_mcp_in_db(&db, assignment_id, false).unwrap();
        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();
        assert!(!mcps[0].is_enabled);

        // Re-enable
        toggle_project_mcp_in_db(&db, assignment_id, true).unwrap();
        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();
        assert!(mcps[0].is_enabled);
    }

    #[test]
    fn test_duplicate_mcp_assignment_ignored() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Dup Project".to_string(),
            path: "/dup/project".to_string(),
        };
        let created_project = create_project_in_db(&db, &project).unwrap();
        let mcp_id = create_test_mcp(&db, "dup-mcp");

        // Assign twice
        assign_mcp_to_project_in_db(&db, created_project.id, mcp_id).unwrap();
        assign_mcp_to_project_in_db(&db, created_project.id, mcp_id).unwrap();

        // Should only have one
        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();
        assert_eq!(mcps.len(), 1);
    }

    // =========================================================================
    // Cascade delete tests
    // =========================================================================

    #[test]
    fn test_delete_project_removes_mcp_assignments() {
        let db = Database::in_memory().unwrap();

        let project = CreateProjectRequest {
            name: "Cascade".to_string(),
            path: "/cascade/path".to_string(),
        };
        let created_project = create_project_in_db(&db, &project).unwrap();
        let mcp_id = create_test_mcp(&db, "cascade-mcp");

        assign_mcp_to_project_in_db(&db, created_project.id, mcp_id).unwrap();

        // Verify assignment exists
        let mcps = get_project_mcps_from_db(&db, created_project.id).unwrap();
        assert_eq!(mcps.len(), 1);

        // Delete project
        delete_project_from_db(&db, created_project.id).unwrap();

        // Verify assignment is gone (via foreign key cascade)
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_mcps WHERE project_id = ?",
                [created_project.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}
