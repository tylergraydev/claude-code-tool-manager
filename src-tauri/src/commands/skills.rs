use crate::db::models::{CreateSkillRequest, Skill, GlobalSkill, ProjectSkill};
use crate::db::schema::Database;
use crate::services::skill_writer;
use rusqlite::params;
use std::sync::Mutex;
use std::path::Path;
use tauri::State;

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn row_to_skill(row: &rusqlite::Row) -> rusqlite::Result<Skill> {
    Ok(Skill {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        content: row.get(3)?,
        skill_type: row.get(4)?,
        allowed_tools: parse_json_array(row.get(5)?),
        argument_hint: row.get(6)?,
        tags: parse_json_array(row.get(7)?),
        source: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_skill_with_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Skill> {
    Ok(Skill {
        id: row.get(offset)?,
        name: row.get(offset + 1)?,
        description: row.get(offset + 2)?,
        content: row.get(offset + 3)?,
        skill_type: row.get(offset + 4)?,
        allowed_tools: parse_json_array(row.get(offset + 5)?),
        argument_hint: row.get(offset + 6)?,
        tags: parse_json_array(row.get(offset + 7)?),
        source: row.get(offset + 8)?,
        created_at: row.get(offset + 9)?,
        updated_at: row.get(offset + 10)?,
    })
}

#[tauri::command]
pub fn get_all_skills(db: State<'_, Mutex<Database>>) -> Result<Vec<Skill>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, skill_type, allowed_tools, argument_hint, tags, source, created_at, updated_at
             FROM skills ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([], row_to_skill)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(skills)
}

#[tauri::command]
pub fn create_skill(db: State<'_, Mutex<Database>>, skill: CreateSkillRequest) -> Result<Skill, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let allowed_tools_json = skill.allowed_tools.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = skill.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db_guard.conn()
        .execute(
            "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![skill.name, skill.description, skill.content, skill.skill_type, allowed_tools_json, skill.argument_hint, tags_json],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let mut stmt = db_guard
        .conn()
        .prepare(
            "SELECT id, name, description, content, skill_type, allowed_tools, argument_hint, tags, source, created_at, updated_at
             FROM skills WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_skill(
    db: State<'_, Mutex<Database>>,
    id: i64,
    skill: CreateSkillRequest,
) -> Result<Skill, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let allowed_tools_json = skill.allowed_tools.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let tags_json = skill.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE skills SET name = ?, description = ?, content = ?, skill_type = ?, allowed_tools = ?, argument_hint = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![skill.name, skill.description, skill.content, skill.skill_type, allowed_tools_json, skill.argument_hint, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, content, skill_type, allowed_tools, argument_hint, tags, source, created_at, updated_at
             FROM skills WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_skill(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM skills WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Global Skills
#[tauri::command]
pub fn get_global_skills(db: State<'_, Mutex<Database>>) -> Result<Vec<GlobalSkill>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT gs.id, gs.skill_id, gs.is_enabled,
                    s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.tags, s.source, s.created_at, s.updated_at
             FROM global_skills gs
             JOIN skills s ON gs.skill_id = s.id
             ORDER BY s.name",
        )
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([], |row| {
            Ok(GlobalSkill {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                skill: row_to_skill_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(skills)
}

#[tauri::command]
pub fn add_global_skill(db: State<'_, Mutex<Database>>, skill_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the skill details for file writing
    let mut stmt = db_guard.conn()
        .prepare("SELECT id, name, description, content, skill_type, allowed_tools, argument_hint, tags, source, created_at, updated_at FROM skills WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let skill: Skill = stmt.query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "INSERT OR IGNORE INTO global_skills (skill_id) VALUES (?)",
            [skill_id],
        )
        .map_err(|e| e.to_string())?;

    // Write the skill file to global config
    skill_writer::write_global_skill(&skill)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_global_skill(db: State<'_, Mutex<Database>>, skill_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the skill for file deletion
    let mut stmt = db_guard.conn()
        .prepare("SELECT id, name, description, content, skill_type, allowed_tools, argument_hint, tags, source, created_at, updated_at FROM skills WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let skill: Skill = stmt.query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute("DELETE FROM global_skills WHERE skill_id = ?", [skill_id])
        .map_err(|e| e.to_string())?;

    // Delete the skill file from global config
    skill_writer::delete_global_skill(&skill)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_global_skill(db: State<'_, Mutex<Database>>, id: i64, enabled: bool) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "UPDATE global_skills SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

    // Get the skill details
    let mut stmt = db_guard.conn()
        .prepare(
            "SELECT s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.tags, s.source, s.created_at, s.updated_at
             FROM global_skills gs
             JOIN skills s ON gs.skill_id = s.id
             WHERE gs.id = ?"
        )
        .map_err(|e| e.to_string())?;

    let skill: Skill = stmt.query_row([id], row_to_skill)
        .map_err(|e| e.to_string())?;

    // Write or delete the file based on enabled state
    if enabled {
        skill_writer::write_global_skill(&skill)
            .map_err(|e| e.to_string())?;
    } else {
        skill_writer::delete_global_skill(&skill)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Project Skills
#[tauri::command]
pub fn assign_skill_to_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    skill_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and skill details
    let project_path: String = db_guard.conn()
        .query_row("SELECT path FROM projects WHERE id = ?", [project_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = db_guard.conn()
        .prepare("SELECT id, name, description, content, skill_type, allowed_tools, argument_hint, tags, source, created_at, updated_at FROM skills WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let skill: Skill = stmt.query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "INSERT OR IGNORE INTO project_skills (project_id, skill_id) VALUES (?, ?)",
            params![project_id, skill_id],
        )
        .map_err(|e| e.to_string())?;

    // Write the skill file to project config
    skill_writer::write_project_skill(Path::new(&project_path), &skill)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_skill_from_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    skill_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and skill details
    let project_path: String = db_guard.conn()
        .query_row("SELECT path FROM projects WHERE id = ?", [project_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let mut stmt = db_guard.conn()
        .prepare("SELECT id, name, description, content, skill_type, allowed_tools, argument_hint, tags, source, created_at, updated_at FROM skills WHERE id = ?")
        .map_err(|e| e.to_string())?;

    let skill: Skill = stmt.query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "DELETE FROM project_skills WHERE project_id = ? AND skill_id = ?",
            params![project_id, skill_id],
        )
        .map_err(|e| e.to_string())?;

    // Delete the skill file from project config
    skill_writer::delete_project_skill(Path::new(&project_path), &skill)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_project_skill(
    db: State<'_, Mutex<Database>>,
    assignment_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard.conn()
        .execute(
            "UPDATE project_skills SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, assignment_id],
        )
        .map_err(|e| e.to_string())?;

    // Get project path and skill details
    let mut stmt = db_guard.conn()
        .prepare(
            "SELECT p.path, s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.tags, s.source, s.created_at, s.updated_at
             FROM project_skills ps
             JOIN projects p ON ps.project_id = p.id
             JOIN skills s ON ps.skill_id = s.id
             WHERE ps.id = ?"
        )
        .map_err(|e| e.to_string())?;

    let (project_path, skill): (String, Skill) = stmt.query_row([assignment_id], |row| {
        Ok((row.get(0)?, row_to_skill_with_offset(row, 1)?))
    }).map_err(|e| e.to_string())?;

    // Write or delete the file based on enabled state
    if enabled {
        skill_writer::write_project_skill(Path::new(&project_path), &skill)
            .map_err(|e| e.to_string())?;
    } else {
        skill_writer::delete_project_skill(Path::new(&project_path), &skill)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_project_skills(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Vec<ProjectSkill>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT ps.id, ps.skill_id, ps.is_enabled,
                    s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.tags, s.source, s.created_at, s.updated_at
             FROM project_skills ps
             JOIN skills s ON ps.skill_id = s.id
             WHERE ps.project_id = ?
             ORDER BY s.name",
        )
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([project_id], |row| {
            Ok(ProjectSkill {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                skill: row_to_skill_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(skills)
}
