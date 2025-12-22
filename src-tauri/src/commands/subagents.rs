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
        permission_mode: row.get(6)?,
        skills: parse_json_array(row.get(7)?),
        tags: parse_json_array(row.get(8)?),
        source: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
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
        permission_mode: row.get(offset + 6)?,
        skills: parse_json_array(row.get(offset + 7)?),
        tags: parse_json_array(row.get(offset + 8)?),
        source: row.get(offset + 9)?,
        created_at: row.get(offset + 10)?,
        updated_at: row.get(offset + 11)?,
    })
}

#[tauri::command]
pub fn get_all_subagents(db: State<'_, Mutex<Database>>) -> Result<Vec<SubAgent>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, created_at, updated_at
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
    let skills_json = subagent.skills.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = subagent.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db_guard.conn()
        .execute(
            "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![subagent.name, subagent.description, subagent.content, tools_json, subagent.model, subagent.permission_mode, skills_json, tags_json],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let mut stmt = db_guard
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, created_at, updated_at
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
    let skills_json = subagent.skills.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = subagent.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE subagents SET name = ?, description = ?, content = ?, tools = ?, model = ?, permission_mode = ?, skills = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![subagent.name, subagent.description, subagent.content, tools_json, subagent.model, subagent.permission_mode, skills_json, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, created_at, updated_at
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
                    s.id, s.name, s.description, s.content, s.tools, s.model, s.permission_mode, s.skills, s.tags, s.source, s.created_at, s.updated_at
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
        .prepare("SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, created_at, updated_at FROM subagents WHERE id = ?")
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
            "SELECT s.id, s.name, s.description, s.content, s.tools, s.model, s.permission_mode, s.skills, s.tags, s.source, s.created_at, s.updated_at
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
        .prepare("SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, created_at, updated_at FROM subagents WHERE id = ?")
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
            "SELECT p.path, s.id, s.name, s.description, s.content, s.tools, s.model, s.permission_mode, s.skills, s.tags, s.source, s.created_at, s.updated_at
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
                    s.id, s.name, s.description, s.content, s.tools, s.model, s.permission_mode, s.skills, s.tags, s.source, s.created_at, s.updated_at
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

// ============================================================================
// Database operations (for testing without Tauri state)
// ============================================================================

/// Create a subagent directly in the database (for testing)
pub fn create_subagent_in_db(db: &Database, subagent: &CreateSubAgentRequest) -> Result<SubAgent, String> {
    let tools_json = subagent.tools.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let skills_json = subagent.skills.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = subagent.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![subagent.name, subagent.description, subagent.content, tools_json, subagent.model, subagent.permission_mode, skills_json, tags_json],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_subagent_by_id(db, id)
}

/// Get a subagent by ID directly from the database (for testing)
pub fn get_subagent_by_id(db: &Database, id: i64) -> Result<SubAgent, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, created_at, updated_at
             FROM subagents WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_subagent)
        .map_err(|e| e.to_string())
}

/// Get all subagents directly from the database (for testing)
pub fn get_all_subagents_from_db(db: &Database) -> Result<Vec<SubAgent>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, created_at, updated_at
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

/// Update a subagent directly in the database (for testing)
pub fn update_subagent_in_db(db: &Database, id: i64, subagent: &CreateSubAgentRequest) -> Result<SubAgent, String> {
    let tools_json = subagent.tools.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let skills_json = subagent.skills.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = subagent.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE subagents SET name = ?, description = ?, content = ?, tools = ?, model = ?, permission_mode = ?, skills = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![subagent.name, subagent.description, subagent.content, tools_json, subagent.model, subagent.permission_mode, skills_json, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    get_subagent_by_id(db, id)
}

/// Delete a subagent directly from the database (for testing)
pub fn delete_subagent_from_db(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM subagents WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_code_reviewer() -> CreateSubAgentRequest {
        CreateSubAgentRequest {
            name: "code-reviewer".to_string(),
            description: "Reviews code for bugs and improvements".to_string(),
            content: "You are a code review expert. Analyze code for bugs, security issues, and best practices.".to_string(),
            tools: Some(vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()]),
            model: Some("sonnet".to_string()),
            permission_mode: Some("bypassPermissions".to_string()),
            skills: Some(vec!["lint".to_string(), "format".to_string()]),
            tags: Some(vec!["review".to_string(), "quality".to_string()]),
        }
    }

    fn sample_test_writer() -> CreateSubAgentRequest {
        CreateSubAgentRequest {
            name: "test-writer".to_string(),
            description: "Writes unit tests".to_string(),
            content: "You are a test writing expert. Create comprehensive unit tests with good coverage.".to_string(),
            tools: Some(vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()]),
            model: Some("opus".to_string()),
            permission_mode: None,
            skills: None,
            tags: Some(vec!["testing".to_string()]),
        }
    }

    fn sample_minimal_subagent() -> CreateSubAgentRequest {
        CreateSubAgentRequest {
            name: "minimal".to_string(),
            description: "A minimal subagent".to_string(),
            content: "Minimal content.".to_string(),
            tools: None,
            model: None,
            permission_mode: None,
            skills: None,
            tags: None,
        }
    }

    // ========================================================================
    // Create SubAgent tests
    // ========================================================================

    #[test]
    fn test_create_subagent_full() {
        let db = Database::in_memory().unwrap();
        let req = sample_code_reviewer();

        let subagent = create_subagent_in_db(&db, &req).unwrap();

        assert_eq!(subagent.name, "code-reviewer");
        assert_eq!(subagent.description, "Reviews code for bugs and improvements");
        assert!(subagent.content.contains("code review expert"));
        assert_eq!(subagent.tools, Some(vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()]));
        assert_eq!(subagent.model, Some("sonnet".to_string()));
        assert_eq!(subagent.permission_mode, Some("bypassPermissions".to_string()));
        assert_eq!(subagent.skills, Some(vec!["lint".to_string(), "format".to_string()]));
        assert_eq!(subagent.tags, Some(vec!["review".to_string(), "quality".to_string()]));
        assert_eq!(subagent.source, "manual");
    }

    #[test]
    fn test_create_subagent_minimal() {
        let db = Database::in_memory().unwrap();
        let req = sample_minimal_subagent();

        let subagent = create_subagent_in_db(&db, &req).unwrap();

        assert_eq!(subagent.name, "minimal");
        assert!(subagent.tools.is_none());
        assert!(subagent.model.is_none());
        assert!(subagent.permission_mode.is_none());
        assert!(subagent.skills.is_none());
        assert!(subagent.tags.is_none());
    }

    #[test]
    fn test_create_duplicate_subagent_fails() {
        let db = Database::in_memory().unwrap();
        let req = sample_code_reviewer();

        create_subagent_in_db(&db, &req).unwrap();
        let result = create_subagent_in_db(&db, &req);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UNIQUE constraint failed"));
    }

    // ========================================================================
    // Get SubAgent tests
    // ========================================================================

    #[test]
    fn test_get_subagent_by_id() {
        let db = Database::in_memory().unwrap();
        let req = sample_code_reviewer();
        let created = create_subagent_in_db(&db, &req).unwrap();

        let fetched = get_subagent_by_id(&db, created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, created.name);
    }

    #[test]
    fn test_get_subagent_by_id_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_subagent_by_id(&db, 9999);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_subagents_empty() {
        let db = Database::in_memory().unwrap();

        let subagents = get_all_subagents_from_db(&db).unwrap();

        assert!(subagents.is_empty());
    }

    #[test]
    fn test_get_all_subagents_sorted_by_name() {
        let db = Database::in_memory().unwrap();

        create_subagent_in_db(&db, &CreateSubAgentRequest {
            name: "zebra-agent".to_string(),
            ..sample_minimal_subagent()
        }).unwrap();

        create_subagent_in_db(&db, &CreateSubAgentRequest {
            name: "alpha-agent".to_string(),
            ..sample_minimal_subagent()
        }).unwrap();

        create_subagent_in_db(&db, &CreateSubAgentRequest {
            name: "middle-agent".to_string(),
            ..sample_minimal_subagent()
        }).unwrap();

        let subagents = get_all_subagents_from_db(&db).unwrap();

        assert_eq!(subagents.len(), 3);
        assert_eq!(subagents[0].name, "alpha-agent");
        assert_eq!(subagents[1].name, "middle-agent");
        assert_eq!(subagents[2].name, "zebra-agent");
    }

    // ========================================================================
    // Update SubAgent tests
    // ========================================================================

    #[test]
    fn test_update_subagent() {
        let db = Database::in_memory().unwrap();
        let req = sample_code_reviewer();
        let created = create_subagent_in_db(&db, &req).unwrap();

        let update_req = CreateSubAgentRequest {
            name: "updated-agent".to_string(),
            description: "Updated description".to_string(),
            content: "Updated content.".to_string(),
            tools: Some(vec!["Bash".to_string()]),
            model: Some("haiku".to_string()),
            permission_mode: Some("default".to_string()),
            skills: Some(vec!["new-skill".to_string()]),
            tags: Some(vec!["updated".to_string()]),
        };

        let updated = update_subagent_in_db(&db, created.id, &update_req).unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "updated-agent");
        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.content, "Updated content.");
        assert_eq!(updated.tools, Some(vec!["Bash".to_string()]));
        assert_eq!(updated.model, Some("haiku".to_string()));
        assert_eq!(updated.permission_mode, Some("default".to_string()));
    }

    #[test]
    fn test_update_subagent_not_found() {
        let db = Database::in_memory().unwrap();
        let req = sample_minimal_subagent();

        let result = update_subagent_in_db(&db, 9999, &req);

        assert!(result.is_err());
    }

    // ========================================================================
    // Delete SubAgent tests
    // ========================================================================

    #[test]
    fn test_delete_subagent() {
        let db = Database::in_memory().unwrap();
        let req = sample_code_reviewer();
        let created = create_subagent_in_db(&db, &req).unwrap();

        let result = delete_subagent_from_db(&db, created.id);
        assert!(result.is_ok());

        let fetch_result = get_subagent_by_id(&db, created.id);
        assert!(fetch_result.is_err());
    }

    #[test]
    fn test_delete_multiple_subagents() {
        let db = Database::in_memory().unwrap();

        let s1 = create_subagent_in_db(&db, &sample_code_reviewer()).unwrap();
        let s2 = create_subagent_in_db(&db, &sample_test_writer()).unwrap();
        let s3 = create_subagent_in_db(&db, &sample_minimal_subagent()).unwrap();

        delete_subagent_from_db(&db, s2.id).unwrap();

        let remaining = get_all_subagents_from_db(&db).unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.iter().any(|s| s.id == s1.id));
        assert!(remaining.iter().any(|s| s.id == s3.id));
        assert!(!remaining.iter().any(|s| s.id == s2.id));
    }

    // ========================================================================
    // parse_json_array tests
    // ========================================================================

    #[test]
    fn test_parse_json_array_valid() {
        let result = parse_json_array(Some(r#"["Read", "Write", "Bash"]"#.to_string()));
        assert_eq!(result, Some(vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()]));
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
}
