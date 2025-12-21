use crate::db::models::{CreateHookRequest, GlobalHook, Hook, ProjectHook};
use crate::db::schema::Database;
use crate::services::hook_writer;
use rusqlite::params;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

fn row_to_hook(row: &rusqlite::Row) -> rusqlite::Result<Hook> {
    Ok(Hook {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        event_type: row.get(3)?,
        matcher: row.get(4)?,
        hook_type: row.get(5)?,
        command: row.get(6)?,
        prompt: row.get(7)?,
        timeout: row.get(8)?,
        tags: parse_json_array(row.get(9)?),
        source: row.get(10)?,
        is_template: row.get::<_, i32>(11)? != 0,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn row_to_hook_with_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Hook> {
    Ok(Hook {
        id: row.get(offset)?,
        name: row.get(offset + 1)?,
        description: row.get(offset + 2)?,
        event_type: row.get(offset + 3)?,
        matcher: row.get(offset + 4)?,
        hook_type: row.get(offset + 5)?,
        command: row.get(offset + 6)?,
        prompt: row.get(offset + 7)?,
        timeout: row.get(offset + 8)?,
        tags: parse_json_array(row.get(offset + 9)?),
        source: row.get(offset + 10)?,
        is_template: row.get::<_, i32>(offset + 11)? != 0,
        created_at: row.get(offset + 12)?,
        updated_at: row.get(offset + 13)?,
    })
}

const HOOK_SELECT_FIELDS: &str = "id, name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source, is_template, created_at, updated_at";

// Helper to get all enabled global hooks and write to settings.json
fn sync_global_hooks(db: &Database) -> Result<(), String> {
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT h.{} FROM global_hooks gh JOIN hooks h ON gh.hook_id = h.id WHERE gh.is_enabled = 1",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let hooks: Vec<Hook> = stmt
        .query_map([], row_to_hook)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    hook_writer::write_global_hooks(&hooks).map_err(|e| e.to_string())
}

// Helper to get all enabled project hooks and write to settings.local.json
fn sync_project_hooks(db: &Database, project_path: &str) -> Result<(), String> {
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT h.{} FROM project_hooks ph
             JOIN hooks h ON ph.hook_id = h.id
             JOIN projects p ON ph.project_id = p.id
             WHERE ph.is_enabled = 1 AND p.path = ?",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let hooks: Vec<Hook> = stmt
        .query_map([project_path], row_to_hook)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    hook_writer::write_project_hooks(Path::new(project_path), &hooks).map_err(|e| e.to_string())
}

// CRUD Operations

#[tauri::command]
pub fn get_all_hooks(db: State<'_, Mutex<Database>>) -> Result<Vec<Hook>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE is_template = 0 ORDER BY name",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let hooks = stmt
        .query_map([], row_to_hook)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hooks)
}

#[tauri::command]
pub fn get_hook_templates(db: State<'_, Mutex<Database>>) -> Result<Vec<Hook>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE is_template = 1 ORDER BY name",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let hooks = stmt
        .query_map([], row_to_hook)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hooks)
}

#[tauri::command]
pub fn create_hook(db: State<'_, Mutex<Database>>, hook: CreateHookRequest) -> Result<Hook, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let tags_json = hook.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db_guard
        .conn()
        .execute(
            "INSERT INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            params![
                hook.name,
                hook.description,
                hook.event_type,
                hook.matcher,
                hook.hook_type,
                hook.command,
                hook.prompt,
                hook.timeout,
                tags_json
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let mut stmt = db_guard
        .conn()
        .prepare(&format!("SELECT {} FROM hooks WHERE id = ?", HOOK_SELECT_FIELDS))
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_hook).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_hook_from_template(
    db: State<'_, Mutex<Database>>,
    template_id: i64,
    name: String,
) -> Result<Hook, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the template
    let mut stmt = db_guard
        .conn()
        .prepare(&format!("SELECT {} FROM hooks WHERE id = ?", HOOK_SELECT_FIELDS))
        .map_err(|e| e.to_string())?;

    let template: Hook = stmt
        .query_row([template_id], row_to_hook)
        .map_err(|e| e.to_string())?;

    // Create new hook based on template
    db_guard
        .conn()
        .execute(
            "INSERT INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source, is_template)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'template', 0)",
            params![
                name,
                template.description,
                template.event_type,
                template.matcher,
                template.hook_type,
                template.command,
                template.prompt,
                template.timeout,
                template.tags.as_ref().map(|t| serde_json::to_string(t).unwrap())
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db_guard.conn().last_insert_rowid();

    let mut stmt = db_guard
        .conn()
        .prepare(&format!("SELECT {} FROM hooks WHERE id = ?", HOOK_SELECT_FIELDS))
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_hook).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_hook(
    db: State<'_, Mutex<Database>>,
    id: i64,
    hook: CreateHookRequest,
) -> Result<Hook, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let tags_json = hook.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

    db_guard
        .conn()
        .execute(
            "UPDATE hooks SET name = ?, description = ?, event_type = ?, matcher = ?, hook_type = ?, command = ?, prompt = ?, timeout = ?, tags = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            params![
                hook.name,
                hook.description,
                hook.event_type,
                hook.matcher,
                hook.hook_type,
                hook.command,
                hook.prompt,
                hook.timeout,
                tags_json,
                id
            ],
        )
        .map_err(|e| e.to_string())?;

    // Sync global hooks if this hook is assigned globally
    let is_global: bool = db_guard
        .conn()
        .query_row(
            "SELECT COUNT(*) > 0 FROM global_hooks WHERE hook_id = ?",
            [id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if is_global {
        sync_global_hooks(&db_guard)?;
    }

    // Sync project hooks for all projects that have this hook
    let mut stmt = db_guard
        .conn()
        .prepare("SELECT DISTINCT p.path FROM project_hooks ph JOIN projects p ON ph.project_id = p.id WHERE ph.hook_id = ?")
        .map_err(|e| e.to_string())?;

    let project_paths: Vec<String> = stmt
        .query_map([id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for path in project_paths {
        sync_project_hooks(&db_guard, &path)?;
    }

    let mut stmt = db_guard
        .conn()
        .prepare(&format!("SELECT {} FROM hooks WHERE id = ?", HOOK_SELECT_FIELDS))
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_hook).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_hook(db: State<'_, Mutex<Database>>, id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Check if hook is assigned globally
    let is_global: bool = db_guard
        .conn()
        .query_row(
            "SELECT COUNT(*) > 0 FROM global_hooks WHERE hook_id = ?",
            [id],
            |row| row.get(0),
        )
        .unwrap_or(false);

    // Get project paths before deleting
    let mut stmt = db_guard
        .conn()
        .prepare("SELECT DISTINCT p.path FROM project_hooks ph JOIN projects p ON ph.project_id = p.id WHERE ph.hook_id = ?")
        .map_err(|e| e.to_string())?;

    let project_paths: Vec<String> = stmt
        .query_map([id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Delete the hook (cascades to global_hooks and project_hooks)
    db_guard
        .conn()
        .execute("DELETE FROM hooks WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    // Sync global hooks if it was assigned globally
    if is_global {
        sync_global_hooks(&db_guard)?;
    }

    // Sync project hooks for affected projects
    for path in project_paths {
        sync_project_hooks(&db_guard, &path)?;
    }

    Ok(())
}

// Global Hooks

#[tauri::command]
pub fn get_global_hooks(db: State<'_, Mutex<Database>>) -> Result<Vec<GlobalHook>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT gh.id, gh.hook_id, gh.is_enabled, h.{}
             FROM global_hooks gh
             JOIN hooks h ON gh.hook_id = h.id
             ORDER BY h.name",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let hooks = stmt
        .query_map([], |row| {
            Ok(GlobalHook {
                id: row.get(0)?,
                hook_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                hook: row_to_hook_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hooks)
}

#[tauri::command]
pub fn add_global_hook(db: State<'_, Mutex<Database>>, hook_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT OR IGNORE INTO global_hooks (hook_id) VALUES (?)",
            [hook_id],
        )
        .map_err(|e| e.to_string())?;

    sync_global_hooks(&db_guard)
}

#[tauri::command]
pub fn remove_global_hook(db: State<'_, Mutex<Database>>, hook_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute("DELETE FROM global_hooks WHERE hook_id = ?", [hook_id])
        .map_err(|e| e.to_string())?;

    sync_global_hooks(&db_guard)
}

#[tauri::command]
pub fn toggle_global_hook(
    db: State<'_, Mutex<Database>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE global_hooks SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

    sync_global_hooks(&db_guard)
}

// Project Hooks

#[tauri::command]
pub fn get_project_hooks(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Vec<ProjectHook>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT ph.id, ph.hook_id, ph.is_enabled, h.{}
             FROM project_hooks ph
             JOIN hooks h ON ph.hook_id = h.id
             WHERE ph.project_id = ?
             ORDER BY h.name",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let hooks = stmt
        .query_map([project_id], |row| {
            Ok(ProjectHook {
                id: row.get(0)?,
                hook_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                hook: row_to_hook_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hooks)
}

#[tauri::command]
pub fn assign_hook_to_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    hook_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path
    let project_path: String = db_guard
        .conn()
        .query_row("SELECT path FROM projects WHERE id = ?", [project_id], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT OR IGNORE INTO project_hooks (project_id, hook_id) VALUES (?, ?)",
            params![project_id, hook_id],
        )
        .map_err(|e| e.to_string())?;

    sync_project_hooks(&db_guard, &project_path)
}

#[tauri::command]
pub fn remove_hook_from_project(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    hook_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path
    let project_path: String = db_guard
        .conn()
        .query_row("SELECT path FROM projects WHERE id = ?", [project_id], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "DELETE FROM project_hooks WHERE project_id = ? AND hook_id = ?",
            params![project_id, hook_id],
        )
        .map_err(|e| e.to_string())?;

    sync_project_hooks(&db_guard, &project_path)
}

#[tauri::command]
pub fn toggle_project_hook(
    db: State<'_, Mutex<Database>>,
    assignment_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE project_hooks SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, assignment_id],
        )
        .map_err(|e| e.to_string())?;

    // Get project path
    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT p.path FROM project_hooks ph JOIN projects p ON ph.project_id = p.id WHERE ph.id = ?",
            [assignment_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    sync_project_hooks(&db_guard, &project_path)
}

// Seed default templates
#[tauri::command]
pub fn seed_hook_templates(db: State<'_, Mutex<Database>>) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Check if templates already exist
    let count: i64 = db_guard
        .conn()
        .query_row("SELECT COUNT(*) FROM hooks WHERE is_template = 1", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    let templates = vec![
        (
            "Auto-format Prettier",
            "Run Prettier after file changes",
            "PostToolUse",
            Some("Write|Edit"),
            "command",
            Some("npx prettier --write \"$CLAUDE_FILE_PATHS\""),
            None::<&str>,
            Some(30),
        ),
        (
            "Protect .env files",
            "Block modifications to .env files",
            "PreToolUse",
            Some("Write|Edit"),
            "command",
            Some("if echo \"$CLAUDE_TOOL_INPUT\" | grep -q '\\.env'; then echo 'BLOCK: Cannot modify .env files'; exit 2; fi"),
            None,
            Some(5),
        ),
        (
            "Log tool usage",
            "Log all tool invocations to a file",
            "PostToolUse",
            None::<&str>,
            "command",
            Some("echo \"$(date): $CLAUDE_TOOL_NAME\" >> ~/.claude/tool-log.txt"),
            None,
            Some(5),
        ),
        (
            "Session greeting",
            "Show a custom greeting at session start",
            "Notification",
            Some("session_start"),
            "command",
            Some("echo 'Welcome! Type /help for available commands.'"),
            None,
            None::<i32>,
        ),
        (
            "Lint on save",
            "Run ESLint after editing JS/TS files",
            "PostToolUse",
            Some("Write|Edit"),
            "command",
            Some("if echo \"$CLAUDE_FILE_PATHS\" | grep -qE '\\.(js|ts|jsx|tsx)$'; then npx eslint --fix \"$CLAUDE_FILE_PATHS\"; fi"),
            None,
            Some(60),
        ),
    ];

    for (name, desc, event, matcher, hook_type, command, prompt, timeout) in templates {
        db_guard
            .conn()
            .execute(
                "INSERT OR IGNORE INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, source, is_template)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'system', 1)",
                params![name, desc, event, matcher, hook_type, command, prompt, timeout],
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
