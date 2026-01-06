use crate::db::models::{Command, CreateCommandRequest, GlobalCommand, ProjectCommand};
use crate::db::schema::Database;
use regex::Regex;
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::State;

// ============================================================================
// Validation Constants (based on official Claude Code documentation)
// ============================================================================

/// Maximum length for command name (per official docs)
const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for command description (per official docs)
const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Recommended maximum lines for command content (per official best practices)
const RECOMMENDED_MAX_CONTENT_LINES: usize = 500;

/// Reserved words that cannot appear in command names (per official docs)
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

/// Validate a command name according to official Claude Code documentation
fn validate_command_name(name: &str) -> ValidationResult {
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

/// Validate command description length
fn validate_command_description(description: &Option<String>) -> ValidationResult {
    if let Some(desc) = description {
        if desc.len() > MAX_DESCRIPTION_LENGTH {
            return ValidationResult::err(format!(
                "Description must be {} characters or less (currently {})",
                MAX_DESCRIPTION_LENGTH,
                desc.len()
            ));
        }
    }
    ValidationResult::ok()
}

/// Validate command content - returns warning if too long but still valid
fn validate_command_content(content: &str) -> ValidationResult {
    if content.trim().is_empty() {
        return ValidationResult::err("Content is required".to_string());
    }

    let line_count = content.lines().count();
    if line_count > RECOMMENDED_MAX_CONTENT_LINES {
        return ValidationResult::ok_with_warning(format!(
            "Content is {} lines. For better performance, consider keeping content under {} lines. Long commands may slow down Claude's context processing.",
            line_count, RECOMMENDED_MAX_CONTENT_LINES
        ));
    }

    ValidationResult::ok()
}

/// Validate a complete CreateCommandRequest
pub fn validate_command_request(request: &CreateCommandRequest) -> Result<Option<String>, String> {
    // Validate name
    let name_result = validate_command_name(&request.name);
    if !name_result.is_valid {
        return Err(name_result.error.unwrap());
    }

    // Validate description
    let desc_result = validate_command_description(&request.description);
    if !desc_result.is_valid {
        return Err(desc_result.error.unwrap());
    }

    // Validate content (may return a warning)
    let content_result = validate_command_content(&request.content);
    if !content_result.is_valid {
        return Err(content_result.error.unwrap());
    }

    // Return any warnings (content warning takes precedence)
    Ok(content_result.warning.or(name_result.warning))
}

// ============================================================================
// Database Field Mapping
// ============================================================================

const COMMAND_SELECT_FIELDS: &str =
    "id, name, description, content, allowed_tools, argument_hint, model, tags, source, created_at, updated_at";

fn parse_json_array(json: Option<String>) -> Option<Vec<String>> {
    json.and_then(|s| serde_json::from_str(&s).ok())
}

fn row_to_command(row: &rusqlite::Row) -> Result<Command, rusqlite::Error> {
    Ok(Command {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        content: row.get(3)?,
        allowed_tools: parse_json_array(row.get(4)?),
        argument_hint: row.get(5)?,
        model: row.get(6)?,
        tags: parse_json_array(row.get(7)?),
        source: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_command_with_offset(
    row: &rusqlite::Row,
    offset: usize,
) -> Result<Command, rusqlite::Error> {
    Ok(Command {
        id: row.get(offset)?,
        name: row.get(offset + 1)?,
        description: row.get(offset + 2)?,
        content: row.get(offset + 3)?,
        allowed_tools: parse_json_array(row.get(offset + 4)?),
        argument_hint: row.get(offset + 5)?,
        model: row.get(offset + 6)?,
        tags: parse_json_array(row.get(offset + 7)?),
        source: row.get(offset + 8)?,
        created_at: row.get(offset + 9)?,
        updated_at: row.get(offset + 10)?,
    })
}

// ============================================================================
// Tauri Commands - CRUD Operations
// ============================================================================

#[tauri::command]
pub fn get_all_commands(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Command>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT {} FROM commands ORDER BY name",
        COMMAND_SELECT_FIELDS
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let commands = stmt
        .query_map([], row_to_command)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(commands)
}

#[tauri::command]
pub fn create_command(
    db: State<'_, Arc<Mutex<Database>>>,
    command: CreateCommandRequest,
) -> Result<Command, String> {
    // Validate the command request
    let _warning = validate_command_request(&command)?;

    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let allowed_tools_json = command
        .allowed_tools
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let tags_json = command
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());

    db_guard.conn()
        .execute(
            "INSERT INTO commands (name, description, content, allowed_tools, argument_hint, model, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![command.name, command.description, command.content, allowed_tools_json, command.argument_hint, command.model, tags_json],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let query = format!(
        "SELECT {} FROM commands WHERE id = ?",
        COMMAND_SELECT_FIELDS
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_command)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_command(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    command: CreateCommandRequest,
) -> Result<Command, String> {
    // Validate the command request
    let _warning = validate_command_request(&command)?;

    let db = db.lock().map_err(|e| e.to_string())?;

    let allowed_tools_json = command
        .allowed_tools
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());
    let tags_json = command
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE commands SET name = ?, description = ?, content = ?, allowed_tools = ?, argument_hint = ?, model = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![command.name, command.description, command.content, allowed_tools_json, command.argument_hint, command.model, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    let query = format!(
        "SELECT {} FROM commands WHERE id = ?",
        COMMAND_SELECT_FIELDS
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_command)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_command(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Reset is_imported flag in repo_items for this command
    db.conn()
        .execute(
            "UPDATE repo_items SET is_imported = 0, imported_item_id = NULL WHERE imported_item_id = ? AND item_type = 'command'",
            [id],
        )
        .map_err(|e| e.to_string())?;

    db.conn()
        .execute("DELETE FROM commands WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// Global Commands
// ============================================================================

#[tauri::command]
pub fn get_global_commands(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<GlobalCommand>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT gc.id, gc.command_id, gc.is_enabled,
                c.id, c.name, c.description, c.content, c.allowed_tools, c.argument_hint, c.model, c.tags, c.source, c.created_at, c.updated_at
         FROM global_commands gc
         JOIN commands c ON gc.command_id = c.id
         ORDER BY c.name"
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let commands = stmt
        .query_map([], |row| {
            Ok(GlobalCommand {
                id: row.get(0)?,
                command_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                command: row_to_command_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(commands)
}

#[tauri::command]
pub fn add_global_command(
    db: State<'_, Arc<Mutex<Database>>>,
    command_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the command details for file writing
    let query = format!(
        "SELECT {} FROM commands WHERE id = ?",
        COMMAND_SELECT_FIELDS
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let command: Command = stmt
        .query_row([command_id], row_to_command)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT OR IGNORE INTO global_commands (command_id) VALUES (?)",
            [command_id],
        )
        .map_err(|e| e.to_string())?;

    // Write the command file to global config
    crate::services::command_writer::write_global_command(&command).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_global_command(
    db: State<'_, Arc<Mutex<Database>>>,
    command_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the command for file deletion
    let query = format!(
        "SELECT {} FROM commands WHERE id = ?",
        COMMAND_SELECT_FIELDS
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let command: Command = stmt
        .query_row([command_id], row_to_command)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "DELETE FROM global_commands WHERE command_id = ?",
            [command_id],
        )
        .map_err(|e| e.to_string())?;

    // Delete the command file from global config
    crate::services::command_writer::delete_global_command(&command).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_global_command(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE global_commands SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

    // Get the command details
    let query = format!(
        "SELECT c.id, c.name, c.description, c.content, c.allowed_tools, c.argument_hint, c.model, c.tags, c.source, c.created_at, c.updated_at
         FROM global_commands gc
         JOIN commands c ON gc.command_id = c.id
         WHERE gc.id = ?"
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let command: Command = stmt
        .query_row([id], row_to_command)
        .map_err(|e| e.to_string())?;

    // Write or delete the file based on enabled state
    if enabled {
        crate::services::command_writer::write_global_command(&command)
            .map_err(|e| e.to_string())?;
    } else {
        crate::services::command_writer::delete_global_command(&command)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Project Commands
// ============================================================================

#[tauri::command]
pub fn assign_command_to_project(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    command_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and command details
    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let query = format!(
        "SELECT {} FROM commands WHERE id = ?",
        COMMAND_SELECT_FIELDS
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let command: Command = stmt
        .query_row([command_id], row_to_command)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT OR IGNORE INTO project_commands (project_id, command_id) VALUES (?, ?)",
            params![project_id, command_id],
        )
        .map_err(|e| e.to_string())?;

    // Write the command file to project config
    crate::services::command_writer::write_project_command(
        std::path::Path::new(&project_path),
        &command,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_command_from_project(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    command_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path and command details
    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let query = format!(
        "SELECT {} FROM commands WHERE id = ?",
        COMMAND_SELECT_FIELDS
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let command: Command = stmt
        .query_row([command_id], row_to_command)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "DELETE FROM project_commands WHERE project_id = ? AND command_id = ?",
            params![project_id, command_id],
        )
        .map_err(|e| e.to_string())?;

    // Delete the command file from project config
    crate::services::command_writer::delete_project_command(
        std::path::Path::new(&project_path),
        &command,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_project_command(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE project_commands SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

    // Get the command and project details
    let query = format!(
        "SELECT c.id, c.name, c.description, c.content, c.allowed_tools, c.argument_hint, c.model, c.tags, c.source, c.created_at, c.updated_at, p.path
         FROM project_commands pc
         JOIN commands c ON pc.command_id = c.id
         JOIN projects p ON pc.project_id = p.id
         WHERE pc.id = ?"
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;

    let (command, project_path): (Command, String) = stmt
        .query_row([id], |row| Ok((row_to_command(row)?, row.get(11)?)))
        .map_err(|e| e.to_string())?;

    // Write or delete the file based on enabled state
    if enabled {
        crate::services::command_writer::write_project_command(
            std::path::Path::new(&project_path),
            &command,
        )
        .map_err(|e| e.to_string())?;
    } else {
        crate::services::command_writer::delete_project_command(
            std::path::Path::new(&project_path),
            &command,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_project_commands(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
) -> Result<Vec<ProjectCommand>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT pc.id, pc.command_id, pc.is_enabled,
                c.id, c.name, c.description, c.content, c.allowed_tools, c.argument_hint, c.model, c.tags, c.source, c.created_at, c.updated_at
         FROM project_commands pc
         JOIN commands c ON pc.command_id = c.id
         WHERE pc.project_id = ?
         ORDER BY c.name"
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let commands = stmt
        .query_map([project_id], |row| {
            Ok(ProjectCommand {
                id: row.get(0)?,
                command_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                command: row_to_command_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(commands)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Validation tests
    // =========================================================================

    #[test]
    fn test_validate_command_name_valid() {
        let result = validate_command_name("my-command");
        assert!(result.is_valid);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_validate_command_name_with_numbers() {
        let result = validate_command_name("command-123");
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_command_name_empty() {
        let result = validate_command_name("");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("required"));
    }

    #[test]
    fn test_validate_command_name_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_command_name(&long_name);
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("64 characters"));
    }

    #[test]
    fn test_validate_command_name_invalid_chars() {
        let result = validate_command_name("My Command!");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("lowercase"));
    }

    #[test]
    fn test_validate_command_name_reserved_word() {
        let result = validate_command_name("my-claude-command");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("reserved"));
    }

    #[test]
    fn test_validate_command_description_valid() {
        let result = validate_command_description(&Some("A valid description".to_string()));
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_command_description_too_long() {
        let long_desc = "a".repeat(1025);
        let result = validate_command_description(&Some(long_desc));
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_command_content_empty() {
        let result = validate_command_content("");
        assert!(!result.is_valid);
        assert!(result.error.unwrap().contains("required"));
    }

    #[test]
    fn test_validate_command_content_warning_long() {
        let long_content = "line\n".repeat(501);
        let result = validate_command_content(&long_content);
        assert!(result.is_valid);
        assert!(result.warning.is_some());
    }

    #[test]
    fn test_validate_command_request_valid() {
        let request = CreateCommandRequest {
            name: "test-command".to_string(),
            description: Some("A test command".to_string()),
            content: "Do something useful.".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            tags: None,
        };
        let result = validate_command_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_command_request_invalid_name() {
        let request = CreateCommandRequest {
            name: "".to_string(),
            description: None,
            content: "Content".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            tags: None,
        };
        let result = validate_command_request(&request);
        assert!(result.is_err());
    }
}
