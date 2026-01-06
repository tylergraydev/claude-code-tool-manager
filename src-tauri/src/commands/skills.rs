use crate::db::models::{
    CreateSkillFileRequest, CreateSkillRequest, GlobalSkill, ProjectSkill, Skill, SkillFile,
};
use crate::db::schema::Database;
use crate::services::skill_writer;
use regex::Regex;
use rusqlite::params;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::State;

// ============================================================================
// Validation Constants (based on official Claude Code documentation)
// ============================================================================

/// Maximum length for skill name (per official docs)
const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for skill description (per official docs)
const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Recommended maximum lines for SKILL.md content (per official best practices)
const RECOMMENDED_MAX_CONTENT_LINES: usize = 500;

/// Reserved words that cannot appear in skill names (per official docs)
const RESERVED_WORDS: &[&str] = &["anthropic", "claude"];

/// Validation result with optional warning
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error: Option<String>,
    pub warning: Option<String>,
}

impl ValidationResult {
    fn ok() -> Self {
        Self {
            is_valid: true,
            error: None,
            warning: None,
        }
    }

    fn ok_with_warning(warning: String) -> Self {
        Self {
            is_valid: true,
            error: None,
            warning: Some(warning),
        }
    }

    fn err(error: String) -> Self {
        Self {
            is_valid: false,
            error: Some(error),
            warning: None,
        }
    }
}

/// Validate a skill name according to official Claude Code documentation
fn validate_skill_name(name: &str) -> ValidationResult {
    let name = name.trim();

    // Check if empty
    if name.is_empty() {
        return ValidationResult::err("Name is required".to_string());
    }

    // Check maximum length (64 characters)
    if name.len() > MAX_NAME_LENGTH {
        return ValidationResult::err(format!(
            "Name must be {} characters or less (currently {})",
            MAX_NAME_LENGTH,
            name.len()
        ));
    }

    // Check format: lowercase letters, numbers, and hyphens only
    // This also implicitly rejects XML tags (< or >) and other special characters
    let name_regex = Regex::new(r"^[a-z0-9-]+$").unwrap();
    if !name_regex.is_match(name) {
        return ValidationResult::err(
            "Name must contain only lowercase letters, numbers, and hyphens".to_string(),
        );
    }

    // Check for reserved words
    for reserved in RESERVED_WORDS {
        if name.contains(reserved) {
            return ValidationResult::err(format!(
                "Name cannot contain reserved word '{}'",
                reserved
            ));
        }
    }

    ValidationResult::ok()
}

/// Validate a skill description according to official Claude Code documentation
fn validate_skill_description(description: Option<&str>) -> ValidationResult {
    let description = match description {
        Some(d) => d.trim(),
        None => return ValidationResult::ok(), // Description is optional
    };

    if description.is_empty() {
        return ValidationResult::ok(); // Empty is allowed (treated as None)
    }

    // Check maximum length (1024 characters)
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return ValidationResult::err(format!(
            "Description must be {} characters or less (currently {})",
            MAX_DESCRIPTION_LENGTH,
            description.len()
        ));
    }

    // Check for XML tags (security requirement)
    if description.contains('<') || description.contains('>') {
        return ValidationResult::err("Description cannot contain XML tags (< or >)".to_string());
    }

    ValidationResult::ok()
}

/// Validate skill content and return a warning if it exceeds recommended lines
fn validate_skill_content(content: &str) -> ValidationResult {
    let content = content.trim();

    if content.is_empty() {
        return ValidationResult::err("Content is required".to_string());
    }

    // Count lines and warn if exceeding recommended maximum
    let line_count = content.lines().count();
    if line_count > RECOMMENDED_MAX_CONTENT_LINES {
        return ValidationResult::ok_with_warning(format!(
            "Content has {} lines, exceeding the recommended {} lines for optimal performance. Consider splitting into separate reference files.",
            line_count,
            RECOMMENDED_MAX_CONTENT_LINES
        ));
    }

    ValidationResult::ok()
}

/// Validate a complete skill request
pub fn validate_skill_request(skill: &CreateSkillRequest) -> Result<Option<String>, String> {
    // Validate name
    let name_result = validate_skill_name(&skill.name);
    if !name_result.is_valid {
        return Err(name_result.error.unwrap());
    }

    // Validate description
    let desc_result = validate_skill_description(skill.description.as_deref());
    if !desc_result.is_valid {
        return Err(desc_result.error.unwrap());
    }

    // Validate content
    let content_result = validate_skill_content(&skill.content);
    if !content_result.is_valid {
        return Err(content_result.error.unwrap());
    }

    // Collect warnings
    let warning = content_result.warning;

    Ok(warning)
}

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

const SKILL_SELECT_FIELDS: &str = "id, name, description, content, skill_type, allowed_tools, argument_hint, model, disable_model_invocation, tags, source, created_at, updated_at";

fn row_to_skill(row: &rusqlite::Row) -> rusqlite::Result<Skill> {
    Ok(Skill {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        content: row.get(3)?,
        skill_type: row.get(4)?,
        allowed_tools: parse_json_array(row.get(5)?),
        argument_hint: row.get(6)?,
        model: row.get(7)?,
        disable_model_invocation: row.get::<_, i32>(8).unwrap_or(0) != 0,
        tags: parse_json_array(row.get(9)?),
        source: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
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
        model: row.get(offset + 7)?,
        disable_model_invocation: row.get::<_, i32>(offset + 8).unwrap_or(0) != 0,
        tags: parse_json_array(row.get(offset + 9)?),
        source: row.get(offset + 10)?,
        created_at: row.get(offset + 11)?,
        updated_at: row.get(offset + 12)?,
    })
}

#[tauri::command]
pub fn get_all_skills(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Skill>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!("SELECT {} FROM skills ORDER BY name", SKILL_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([], row_to_skill)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(skills)
}

#[tauri::command]
pub fn create_skill(
    db: State<'_, Arc<Mutex<Database>>>,
    skill: CreateSkillRequest,
) -> Result<Skill, String> {
    // Validate the skill request
    let _warning = validate_skill_request(&skill)?;
    // Note: warning could be returned to frontend if needed in the future

    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let allowed_tools_json = skill
        .allowed_tools
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let tags_json = skill
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let disable_model_invocation = skill.disable_model_invocation.unwrap_or(false) as i32;

    db_guard.conn()
        .execute(
            "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, model, disable_model_invocation, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![skill.name, skill.description, skill.content, skill.skill_type, allowed_tools_json, skill.argument_hint, skill.model, disable_model_invocation, tags_json],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let query = format!("SELECT {} FROM skills WHERE id = ?", SKILL_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_skill(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    skill: CreateSkillRequest,
) -> Result<Skill, String> {
    // Validate the skill request
    let _warning = validate_skill_request(&skill)?;
    // Note: warning could be returned to frontend if needed in the future

    let db = db.lock().map_err(|e| e.to_string())?;

    let allowed_tools_json = skill
        .allowed_tools
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let tags_json = skill
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let disable_model_invocation = skill.disable_model_invocation.unwrap_or(false) as i32;

    db.conn()
        .execute(
            "UPDATE skills SET name = ?, description = ?, content = ?, skill_type = ?, allowed_tools = ?, argument_hint = ?, model = ?, disable_model_invocation = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![skill.name, skill.description, skill.content, skill.skill_type, allowed_tools_json, skill.argument_hint, skill.model, disable_model_invocation, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM skills WHERE id = ?", SKILL_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_skill(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Reset is_imported flag in repo_items for this skill
    db.conn()
        .execute(
            "UPDATE repo_items SET is_imported = 0, imported_item_id = NULL WHERE imported_item_id = ? AND item_type = 'skill'",
            [id],
        )
        .map_err(|e| e.to_string())?;

    db.conn()
        .execute("DELETE FROM skills WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Global Skills
#[tauri::command]
pub fn get_global_skills(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<GlobalSkill>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT gs.id, gs.skill_id, gs.is_enabled,
                s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.model, s.disable_model_invocation, s.tags, s.source, s.created_at, s.updated_at
         FROM global_skills gs
         JOIN skills s ON gs.skill_id = s.id
         ORDER BY s.name"
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

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
pub fn add_global_skill(db: State<'_, Arc<Mutex<Database>>>, skill_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the skill details for file writing
    let query = format!("SELECT {} FROM skills WHERE id = ?", SKILL_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let skill: Skill = stmt
        .query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT OR IGNORE INTO global_skills (skill_id) VALUES (?)",
            [skill_id],
        )
        .map_err(|e| e.to_string())?;

    // Write the skill file to global config
    skill_writer::write_global_skill(&skill).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_global_skill(
    db: State<'_, Arc<Mutex<Database>>>,
    skill_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the skill for file deletion
    let query = format!("SELECT {} FROM skills WHERE id = ?", SKILL_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let skill: Skill = stmt
        .query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute("DELETE FROM global_skills WHERE skill_id = ?", [skill_id])
        .map_err(|e| e.to_string())?;

    // Delete the skill file from global config
    skill_writer::delete_global_skill(&skill).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_global_skill(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE global_skills SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

    // Get the skill details
    let query = format!(
        "SELECT s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.model, s.disable_model_invocation, s.tags, s.source, s.created_at, s.updated_at
         FROM global_skills gs
         JOIN skills s ON gs.skill_id = s.id
         WHERE gs.id = ?"
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let skill: Skill = stmt
        .query_row([id], row_to_skill)
        .map_err(|e| e.to_string())?;

    // Write or delete the file based on enabled state
    if enabled {
        skill_writer::write_global_skill(&skill).map_err(|e| e.to_string())?;
    } else {
        skill_writer::delete_global_skill(&skill).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// Project Skills
#[tauri::command]
pub fn assign_skill_to_project(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    skill_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and skill details
    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM skills WHERE id = ?", SKILL_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let skill: Skill = stmt
        .query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
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
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    skill_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and skill details
    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM skills WHERE id = ?", SKILL_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let skill: Skill = stmt
        .query_row([skill_id], row_to_skill)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
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
    db: State<'_, Arc<Mutex<Database>>>,
    assignment_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE project_skills SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, assignment_id],
        )
        .map_err(|e| e.to_string())?;

    // Get project path and skill details
    let query = format!(
        "SELECT p.path, s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.model, s.disable_model_invocation, s.tags, s.source, s.created_at, s.updated_at
         FROM project_skills ps
         JOIN projects p ON ps.project_id = p.id
         JOIN skills s ON ps.skill_id = s.id
         WHERE ps.id = ?"
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let (project_path, skill): (String, Skill) = stmt
        .query_row([assignment_id], |row| {
            Ok((row.get(0)?, row_to_skill_with_offset(row, 1)?))
        })
        .map_err(|e| e.to_string())?;

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
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
) -> Result<Vec<ProjectSkill>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT ps.id, ps.skill_id, ps.is_enabled,
                s.id, s.name, s.description, s.content, s.skill_type, s.allowed_tools, s.argument_hint, s.model, s.disable_model_invocation, s.tags, s.source, s.created_at, s.updated_at
         FROM project_skills ps
         JOIN skills s ON ps.skill_id = s.id
         WHERE ps.project_id = ?
         ORDER BY s.name"
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

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

// Skill Files (references, assets, scripts)

fn row_to_skill_file(row: &rusqlite::Row) -> rusqlite::Result<SkillFile> {
    Ok(SkillFile {
        id: row.get(0)?,
        skill_id: row.get(1)?,
        file_type: row.get(2)?,
        name: row.get(3)?,
        content: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

#[tauri::command]
pub fn get_skill_files(
    db: State<'_, Arc<Mutex<Database>>>,
    skill_id: i64,
) -> Result<Vec<SkillFile>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, skill_id, file_type, name, content, created_at, updated_at
             FROM skill_files WHERE skill_id = ? ORDER BY file_type, name",
        )
        .map_err(|e| e.to_string())?;

    let files = stmt
        .query_map([skill_id], row_to_skill_file)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(files)
}

#[tauri::command]
pub fn create_skill_file(
    db: State<'_, Arc<Mutex<Database>>>,
    file: CreateSkillFileRequest,
) -> Result<SkillFile, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT INTO skill_files (skill_id, file_type, name, content)
             VALUES (?, ?, ?, ?)",
            params![file.skill_id, file.file_type, file.name, file.content],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let mut stmt = db_guard
        .conn()
        .prepare(
            "SELECT id, skill_id, file_type, name, content, created_at, updated_at
             FROM skill_files WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill_file)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_skill_file(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    name: String,
    content: String,
) -> Result<SkillFile, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    db.conn()
        .execute(
            "UPDATE skill_files SET name = ?, content = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![name, content, id],
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, skill_id, file_type, name, content, created_at, updated_at
             FROM skill_files WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill_file)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_skill_file(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    db.conn()
        .execute("DELETE FROM skill_files WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Database operations (for testing without Tauri state)
// ============================================================================

/// Create a skill directly in the database (for testing)
/// Note: Set skip_validation to true to bypass validation in tests
pub fn create_skill_in_db(db: &Database, skill: &CreateSkillRequest) -> Result<Skill, String> {
    create_skill_in_db_internal(db, skill, false)
}

/// Create a skill without validation (for testing edge cases)
pub fn create_skill_in_db_unvalidated(
    db: &Database,
    skill: &CreateSkillRequest,
) -> Result<Skill, String> {
    create_skill_in_db_internal(db, skill, true)
}

fn create_skill_in_db_internal(
    db: &Database,
    skill: &CreateSkillRequest,
    skip_validation: bool,
) -> Result<Skill, String> {
    if !skip_validation {
        validate_skill_request(skill)?;
    }

    let allowed_tools_json = skill
        .allowed_tools
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let tags_json = skill
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let disable_model_invocation = skill.disable_model_invocation.unwrap_or(false) as i32;

    db.conn()
        .execute(
            "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, model, disable_model_invocation, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            rusqlite::params![skill.name, skill.description, skill.content, skill.skill_type, allowed_tools_json, skill.argument_hint, skill.model, disable_model_invocation, tags_json],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_skill_by_id(db, id)
}

/// Get a skill by ID directly from the database (for testing)
pub fn get_skill_by_id(db: &Database, id: i64) -> Result<Skill, String> {
    let query = format!("SELECT {} FROM skills WHERE id = ?", SKILL_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill)
        .map_err(|e| e.to_string())
}

/// Get all skills directly from the database (for testing)
pub fn get_all_skills_from_db(db: &Database) -> Result<Vec<Skill>, String> {
    let query = format!("SELECT {} FROM skills ORDER BY name", SKILL_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([], row_to_skill)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(skills)
}

/// Update a skill directly in the database (for testing)
pub fn update_skill_in_db(
    db: &Database,
    id: i64,
    skill: &CreateSkillRequest,
) -> Result<Skill, String> {
    // Validate the skill request
    validate_skill_request(skill)?;

    let allowed_tools_json = skill
        .allowed_tools
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let tags_json = skill
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let disable_model_invocation = skill.disable_model_invocation.unwrap_or(false) as i32;

    db.conn()
        .execute(
            "UPDATE skills SET name = ?, description = ?, content = ?, skill_type = ?, allowed_tools = ?, argument_hint = ?, model = ?, disable_model_invocation = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            rusqlite::params![skill.name, skill.description, skill.content, skill.skill_type, allowed_tools_json, skill.argument_hint, skill.model, disable_model_invocation, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    get_skill_by_id(db, id)
}

/// Delete a skill directly from the database (for testing)
pub fn delete_skill_from_db(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM skills WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Create a skill file directly in the database (for testing)
pub fn create_skill_file_in_db(
    db: &Database,
    file: &CreateSkillFileRequest,
) -> Result<SkillFile, String> {
    db.conn()
        .execute(
            "INSERT INTO skill_files (skill_id, file_type, name, content)
             VALUES (?, ?, ?, ?)",
            rusqlite::params![file.skill_id, file.file_type, file.name, file.content],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, skill_id, file_type, name, content, created_at, updated_at
             FROM skill_files WHERE id = ?",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_skill_file)
        .map_err(|e| e.to_string())
}

/// Get skill files directly from the database (for testing)
pub fn get_skill_files_from_db(db: &Database, skill_id: i64) -> Result<Vec<SkillFile>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, skill_id, file_type, name, content, created_at, updated_at
             FROM skill_files WHERE skill_id = ? ORDER BY file_type, name",
        )
        .map_err(|e| e.to_string())?;

    let files = stmt
        .query_map([skill_id], row_to_skill_file)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(files)
}

/// Delete a skill file directly from the database (for testing)
pub fn delete_skill_file_from_db(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM skill_files WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_command_skill() -> CreateSkillRequest {
        CreateSkillRequest {
            name: "test-command".to_string(),
            description: Some("A test command skill".to_string()),
            content: "You are a helpful assistant for testing.".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: Some(vec!["Read".to_string(), "Write".to_string()]),
            argument_hint: Some("<file>".to_string()),
            model: Some("sonnet".to_string()),
            disable_model_invocation: Some(false),
            tags: Some(vec!["test".to_string(), "example".to_string()]),
        }
    }

    fn sample_agent_skill() -> CreateSkillRequest {
        CreateSkillRequest {
            name: "test-agent".to_string(),
            description: Some("A test agent skill".to_string()),
            content: "You are an agent that helps with code reviews.".to_string(),
            skill_type: "skill".to_string(),
            allowed_tools: Some(vec![
                "Read".to_string(),
                "Grep".to_string(),
                "Glob".to_string(),
            ]),
            argument_hint: None,
            model: Some("opus".to_string()),
            disable_model_invocation: Some(true),
            tags: Some(vec!["review".to_string()]),
        }
    }

    fn sample_minimal_skill() -> CreateSkillRequest {
        CreateSkillRequest {
            name: "minimal".to_string(),
            description: None,
            content: "Minimal skill content.".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: None,
            tags: None,
        }
    }

    // ========================================================================
    // Create Skill tests
    // ========================================================================

    #[test]
    fn test_create_command_skill() {
        let db = Database::in_memory().unwrap();
        let req = sample_command_skill();

        let skill = create_skill_in_db(&db, &req).unwrap();

        assert_eq!(skill.name, "test-command");
        assert_eq!(skill.description, Some("A test command skill".to_string()));
        assert_eq!(skill.content, "You are a helpful assistant for testing.");
        assert_eq!(skill.skill_type, "command");
        assert_eq!(
            skill.allowed_tools,
            Some(vec!["Read".to_string(), "Write".to_string()])
        );
        assert_eq!(skill.argument_hint, Some("<file>".to_string()));
        assert_eq!(skill.model, Some("sonnet".to_string()));
        assert!(!skill.disable_model_invocation);
        assert_eq!(
            skill.tags,
            Some(vec!["test".to_string(), "example".to_string()])
        );
        assert_eq!(skill.source, "manual");
    }

    #[test]
    fn test_create_agent_skill() {
        let db = Database::in_memory().unwrap();
        let req = sample_agent_skill();

        let skill = create_skill_in_db(&db, &req).unwrap();

        assert_eq!(skill.name, "test-agent");
        assert_eq!(skill.skill_type, "skill");
        assert!(skill.disable_model_invocation);
        assert_eq!(skill.model, Some("opus".to_string()));
    }

    #[test]
    fn test_create_minimal_skill() {
        let db = Database::in_memory().unwrap();
        let req = sample_minimal_skill();

        let skill = create_skill_in_db(&db, &req).unwrap();

        assert_eq!(skill.name, "minimal");
        assert!(skill.description.is_none());
        assert!(skill.allowed_tools.is_none());
        assert!(skill.argument_hint.is_none());
        assert!(skill.model.is_none());
        assert!(!skill.disable_model_invocation);
        assert!(skill.tags.is_none());
    }

    #[test]
    fn test_create_duplicate_skill_fails() {
        let db = Database::in_memory().unwrap();
        let req = sample_command_skill();

        create_skill_in_db(&db, &req).unwrap();
        let result = create_skill_in_db(&db, &req);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UNIQUE constraint failed"));
    }

    // ========================================================================
    // Get Skill tests
    // ========================================================================

    #[test]
    fn test_get_skill_by_id() {
        let db = Database::in_memory().unwrap();
        let req = sample_command_skill();
        let created = create_skill_in_db(&db, &req).unwrap();

        let fetched = get_skill_by_id(&db, created.id).unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, created.name);
    }

    #[test]
    fn test_get_skill_by_id_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_skill_by_id(&db, 9999);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_skills_empty() {
        let db = Database::in_memory().unwrap();

        let skills = get_all_skills_from_db(&db).unwrap();

        assert!(skills.is_empty());
    }

    #[test]
    fn test_get_all_skills_sorted_by_name() {
        let db = Database::in_memory().unwrap();

        create_skill_in_db(
            &db,
            &CreateSkillRequest {
                name: "zebra-skill".to_string(),
                ..sample_minimal_skill()
            },
        )
        .unwrap();

        create_skill_in_db(
            &db,
            &CreateSkillRequest {
                name: "alpha-skill".to_string(),
                ..sample_minimal_skill()
            },
        )
        .unwrap();

        create_skill_in_db(
            &db,
            &CreateSkillRequest {
                name: "middle-skill".to_string(),
                ..sample_minimal_skill()
            },
        )
        .unwrap();

        let skills = get_all_skills_from_db(&db).unwrap();

        assert_eq!(skills.len(), 3);
        assert_eq!(skills[0].name, "alpha-skill");
        assert_eq!(skills[1].name, "middle-skill");
        assert_eq!(skills[2].name, "zebra-skill");
    }

    // ========================================================================
    // Update Skill tests
    // ========================================================================

    #[test]
    fn test_update_skill() {
        let db = Database::in_memory().unwrap();
        let req = sample_command_skill();
        let created = create_skill_in_db(&db, &req).unwrap();

        let update_req = CreateSkillRequest {
            name: "updated-skill".to_string(),
            description: Some("Updated description".to_string()),
            content: "Updated content.".to_string(),
            skill_type: "skill".to_string(),
            allowed_tools: Some(vec!["Bash".to_string()]),
            argument_hint: Some("<new-hint>".to_string()),
            model: Some("haiku".to_string()),
            disable_model_invocation: Some(true),
            tags: Some(vec!["updated".to_string()]),
        };

        let updated = update_skill_in_db(&db, created.id, &update_req).unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "updated-skill");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.content, "Updated content.");
        assert_eq!(updated.skill_type, "skill");
        assert_eq!(updated.model, Some("haiku".to_string()));
        assert!(updated.disable_model_invocation);
    }

    #[test]
    fn test_update_skill_not_found() {
        let db = Database::in_memory().unwrap();
        let req = sample_minimal_skill();

        let result = update_skill_in_db(&db, 9999, &req);

        assert!(result.is_err());
    }

    // ========================================================================
    // Delete Skill tests
    // ========================================================================

    #[test]
    fn test_delete_skill() {
        let db = Database::in_memory().unwrap();
        let req = sample_command_skill();
        let created = create_skill_in_db(&db, &req).unwrap();

        let result = delete_skill_from_db(&db, created.id);
        assert!(result.is_ok());

        let fetch_result = get_skill_by_id(&db, created.id);
        assert!(fetch_result.is_err());
    }

    #[test]
    fn test_delete_skill_cascades_to_files() {
        let db = Database::in_memory().unwrap();
        let skill = create_skill_in_db(&db, &sample_command_skill()).unwrap();

        let file = create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "reference".to_string(),
                name: "test.md".to_string(),
                content: "Test content".to_string(),
            },
        )
        .unwrap();

        delete_skill_from_db(&db, skill.id).unwrap();

        // Verify files were also deleted (foreign key cascade)
        let files = get_skill_files_from_db(&db, skill.id).unwrap();
        assert!(files.is_empty());
    }

    // ========================================================================
    // Skill File tests
    // ========================================================================

    #[test]
    fn test_create_skill_file() {
        let db = Database::in_memory().unwrap();
        let skill = create_skill_in_db(&db, &sample_command_skill()).unwrap();

        let file = create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "reference".to_string(),
                name: "example.md".to_string(),
                content: "# Reference\n\nSome reference content.".to_string(),
            },
        )
        .unwrap();

        assert_eq!(file.skill_id, skill.id);
        assert_eq!(file.file_type, "reference");
        assert_eq!(file.name, "example.md");
        assert!(file.content.contains("Reference"));
    }

    #[test]
    fn test_create_skill_file_types() {
        let db = Database::in_memory().unwrap();
        let skill = create_skill_in_db(&db, &sample_command_skill()).unwrap();

        // Reference file
        let ref_file = create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "reference".to_string(),
                name: "ref.md".to_string(),
                content: "Reference".to_string(),
            },
        )
        .unwrap();
        assert_eq!(ref_file.file_type, "reference");

        // Asset file
        let asset_file = create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "asset".to_string(),
                name: "data.json".to_string(),
                content: "{}".to_string(),
            },
        )
        .unwrap();
        assert_eq!(asset_file.file_type, "asset");

        // Script file
        let script_file = create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "script".to_string(),
                name: "run.sh".to_string(),
                content: "#!/bin/bash".to_string(),
            },
        )
        .unwrap();
        assert_eq!(script_file.file_type, "script");
    }

    #[test]
    fn test_get_skill_files_sorted() {
        let db = Database::in_memory().unwrap();
        let skill = create_skill_in_db(&db, &sample_command_skill()).unwrap();

        create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "script".to_string(),
                name: "z-script.sh".to_string(),
                content: "content".to_string(),
            },
        )
        .unwrap();

        create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "asset".to_string(),
                name: "a-asset.json".to_string(),
                content: "content".to_string(),
            },
        )
        .unwrap();

        create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "reference".to_string(),
                name: "m-ref.md".to_string(),
                content: "content".to_string(),
            },
        )
        .unwrap();

        let files = get_skill_files_from_db(&db, skill.id).unwrap();

        assert_eq!(files.len(), 3);
        // Sorted by file_type first (asset, reference, script), then name
        assert_eq!(files[0].file_type, "asset");
        assert_eq!(files[1].file_type, "reference");
        assert_eq!(files[2].file_type, "script");
    }

    #[test]
    fn test_delete_skill_file() {
        let db = Database::in_memory().unwrap();
        let skill = create_skill_in_db(&db, &sample_command_skill()).unwrap();

        let file = create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "reference".to_string(),
                name: "to-delete.md".to_string(),
                content: "content".to_string(),
            },
        )
        .unwrap();

        delete_skill_file_from_db(&db, file.id).unwrap();

        let files = get_skill_files_from_db(&db, skill.id).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_skill_file_unique_constraint() {
        let db = Database::in_memory().unwrap();
        let skill = create_skill_in_db(&db, &sample_command_skill()).unwrap();

        create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "reference".to_string(),
                name: "same-name.md".to_string(),
                content: "first".to_string(),
            },
        )
        .unwrap();

        // Same skill, same file_type, same name should fail
        let result = create_skill_file_in_db(
            &db,
            &CreateSkillFileRequest {
                skill_id: skill.id,
                file_type: "reference".to_string(),
                name: "same-name.md".to_string(),
                content: "second".to_string(),
            },
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UNIQUE constraint failed"));
    }

    // ========================================================================
    // parse_json_array tests
    // ========================================================================

    #[test]
    fn test_parse_json_array_valid() {
        let result = parse_json_array(Some(r#"["Read", "Write", "Edit"]"#.to_string()));
        assert_eq!(
            result,
            Some(vec![
                "Read".to_string(),
                "Write".to_string(),
                "Edit".to_string()
            ])
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

    // ========================================================================
    // Validation tests (based on official Claude Code documentation)
    // ========================================================================

    #[test]
    fn test_validate_name_valid() {
        let result = validate_skill_name("my-skill-123");
        assert!(result.is_valid);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_validate_name_empty() {
        let result = validate_skill_name("");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("required"));
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_skill_name(&long_name);
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("64 characters"));
    }

    #[test]
    fn test_validate_name_max_length_ok() {
        let max_name = "a".repeat(64);
        let result = validate_skill_name(&max_name);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_name_uppercase_rejected() {
        let result = validate_skill_name("MySkill");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("lowercase"));
    }

    #[test]
    fn test_validate_name_underscore_rejected() {
        let result = validate_skill_name("my_skill");
        assert!(!result.is_valid);
        assert!(result
            .error
            .unwrap()
            .contains("lowercase letters, numbers, and hyphens"));
    }

    #[test]
    fn test_validate_name_spaces_rejected() {
        let result = validate_skill_name("my skill");
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_name_xml_tags_rejected() {
        // XML tags are rejected by the regex pattern check (not lowercase letters/numbers/hyphens)
        let result = validate_skill_name("<script>");
        assert!(!result.is_valid);
        // The error comes from the regex pattern, which doesn't allow < or >
        assert!(result.error.unwrap().contains("lowercase"));
    }

    #[test]
    fn test_validate_name_reserved_word_anthropic() {
        let result = validate_skill_name("my-anthropic-skill");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("anthropic"));
    }

    #[test]
    fn test_validate_name_reserved_word_claude() {
        let result = validate_skill_name("claude-helper");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("claude"));
    }

    #[test]
    fn test_validate_description_valid() {
        let result = validate_skill_description(Some("A helpful skill for testing"));
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_description_none_ok() {
        let result = validate_skill_description(None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_description_empty_ok() {
        let result = validate_skill_description(Some(""));
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_description_too_long() {
        let long_desc = "a".repeat(1025);
        let result = validate_skill_description(Some(&long_desc));
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("1024 characters"));
    }

    #[test]
    fn test_validate_description_max_length_ok() {
        let max_desc = "a".repeat(1024);
        let result = validate_skill_description(Some(&max_desc));
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_description_xml_tags_rejected() {
        let result = validate_skill_description(Some("Use this for <script>injection</script>"));
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("XML"));
    }

    #[test]
    fn test_validate_content_valid() {
        let result = validate_skill_content("Some skill instructions here.");
        assert!(result.is_valid);
        assert!(result.warning.is_none());
    }

    #[test]
    fn test_validate_content_empty() {
        let result = validate_skill_content("");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("required"));
    }

    #[test]
    fn test_validate_content_warns_over_500_lines() {
        let long_content = "line\n".repeat(501);
        let result = validate_skill_content(&long_content);
        assert!(result.is_valid);
        assert!(result.warning.is_some());
        assert!(result.warning.unwrap().contains("500 lines"));
    }

    #[test]
    fn test_validate_content_no_warning_at_500_lines() {
        let content = "line\n".repeat(500);
        let result = validate_skill_content(&content);
        assert!(result.is_valid);
        assert!(result.warning.is_none());
    }

    #[test]
    fn test_validate_skill_request_valid() {
        let skill = CreateSkillRequest {
            name: "valid-skill".to_string(),
            description: Some("A valid description".to_string()),
            content: "Valid content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: None,
            tags: None,
        };
        let result = validate_skill_request(&skill);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_skill_request_invalid_name() {
        let skill = CreateSkillRequest {
            name: "Invalid_Name".to_string(), // Underscores not allowed
            description: Some("A description".to_string()),
            content: "Content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: None,
            tags: None,
        };
        let result = validate_skill_request(&skill);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_skill_request_invalid_description() {
        let skill = CreateSkillRequest {
            name: "valid-name".to_string(),
            description: Some("<script>alert('xss')</script>".to_string()), // XML tags
            content: "Content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: None,
            tags: None,
        };
        let result = validate_skill_request(&skill);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_skill_with_invalid_name_fails() {
        let db = Database::in_memory().unwrap();
        let skill = CreateSkillRequest {
            name: "Invalid_Name".to_string(), // Underscores not allowed
            description: None,
            content: "Content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: None,
            tags: None,
        };
        let result = create_skill_in_db(&db, &skill);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("lowercase"));
    }

    #[test]
    fn test_create_skill_with_reserved_word_fails() {
        let db = Database::in_memory().unwrap();
        let skill = CreateSkillRequest {
            name: "my-claude-helper".to_string(), // Contains "claude"
            description: None,
            content: "Content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: None,
            tags: None,
        };
        let result = create_skill_in_db(&db, &skill);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("claude"));
    }

    #[test]
    fn test_update_skill_with_invalid_name_fails() {
        let db = Database::in_memory().unwrap();
        let skill = create_skill_in_db(&db, &sample_command_skill()).unwrap();

        let update = CreateSkillRequest {
            name: "UPPERCASE".to_string(), // Invalid
            description: None,
            content: "Content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: None,
            tags: None,
        };
        let result = update_skill_in_db(&db, skill.id, &update);
        assert!(result.is_err());
    }
}
