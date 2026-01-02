use crate::db::models::{CreateHookRequest, GlobalHook, Hook, ProjectHook};
use crate::db::schema::Database;
use crate::services::hook_writer;
use log::{error, info};
use rusqlite::params;
use std::path::Path;
use std::sync::{Arc, Mutex};
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

// Table-prefixed version for JOIN queries to avoid ambiguous column names
const HOOK_SELECT_FIELDS_H: &str = "h.id, h.name, h.description, h.event_type, h.matcher, h.hook_type, h.command, h.prompt, h.timeout, h.tags, h.source, h.is_template, h.created_at, h.updated_at";

// Helper to get all enabled global hooks and write to settings.json
fn sync_global_hooks(db: &Database) -> Result<(), String> {
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT {} FROM global_hooks gh JOIN hooks h ON gh.hook_id = h.id WHERE gh.is_enabled = 1",
            HOOK_SELECT_FIELDS_H
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
            "SELECT {} FROM project_hooks ph
             JOIN hooks h ON ph.hook_id = h.id
             JOIN projects p ON ph.project_id = p.id
             WHERE ph.is_enabled = 1 AND p.path = ?",
            HOOK_SELECT_FIELDS_H
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
pub fn get_all_hooks(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Hook>, String> {
    info!("[Hooks] Loading all hooks");
    let db = db.lock().map_err(|e| {
        error!("[Hooks] Failed to acquire database lock: {}", e);
        e.to_string()
    })?;
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE is_template = 0 ORDER BY name",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| {
            error!("[Hooks] Failed to prepare query: {}", e);
            e.to_string()
        })?;

    let hooks: Vec<Hook> = stmt
        .query_map([], row_to_hook)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    info!("[Hooks] Loaded {} hooks", hooks.len());
    Ok(hooks)
}

#[tauri::command]
pub fn get_hook_templates(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Hook>, String> {
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
pub fn create_hook(
    db: State<'_, Arc<Mutex<Database>>>,
    hook: CreateHookRequest,
) -> Result<Hook, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let tags_json = hook
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());

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
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE id = ?",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_hook).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_hook_from_template(
    db: State<'_, Arc<Mutex<Database>>>,
    template_id: i64,
    name: String,
) -> Result<Hook, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the template
    let mut stmt = db_guard
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE id = ?",
            HOOK_SELECT_FIELDS
        ))
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
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE id = ?",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_hook).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_hook(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    hook: CreateHookRequest,
) -> Result<Hook, String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let tags_json = hook
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());

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
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE id = ?",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_hook).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_hook(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
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
pub fn get_global_hooks(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<GlobalHook>, String> {
    info!("[Hooks] Loading global hooks");
    let db = db.lock().map_err(|e| {
        error!("[Hooks] Failed to acquire database lock: {}", e);
        e.to_string()
    })?;
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT gh.id, gh.hook_id, gh.is_enabled, {}
             FROM global_hooks gh
             JOIN hooks h ON gh.hook_id = h.id
             ORDER BY h.name",
            HOOK_SELECT_FIELDS_H
        ))
        .map_err(|e| {
            error!("[Hooks] Failed to prepare global hooks query: {}", e);
            e.to_string()
        })?;

    let hooks: Vec<GlobalHook> = stmt
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

    info!("[Hooks] Loaded {} global hooks", hooks.len());
    Ok(hooks)
}

#[tauri::command]
pub fn add_global_hook(db: State<'_, Arc<Mutex<Database>>>, hook_id: i64) -> Result<(), String> {
    info!("[Hooks] Adding global hook id={}", hook_id);
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
pub fn remove_global_hook(db: State<'_, Arc<Mutex<Database>>>, hook_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute("DELETE FROM global_hooks WHERE hook_id = ?", [hook_id])
        .map_err(|e| e.to_string())?;

    sync_global_hooks(&db_guard)
}

#[tauri::command]
pub fn toggle_global_hook(
    db: State<'_, Arc<Mutex<Database>>>,
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
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
) -> Result<Vec<ProjectHook>, String> {
    info!(
        "[Hooks] Loading project hooks for project_id={}",
        project_id
    );
    let db = db.lock().map_err(|e| {
        error!("[Hooks] Failed to acquire database lock: {}", e);
        e.to_string()
    })?;
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT ph.id, ph.hook_id, ph.is_enabled, {}
             FROM project_hooks ph
             JOIN hooks h ON ph.hook_id = h.id
             WHERE ph.project_id = ?
             ORDER BY h.name",
            HOOK_SELECT_FIELDS_H
        ))
        .map_err(|e| {
            error!("[Hooks] Failed to prepare project hooks query: {}", e);
            e.to_string()
        })?;

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
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    hook_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path
    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
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
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    hook_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get project path
    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
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
    db: State<'_, Arc<Mutex<Database>>>,
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
pub fn seed_hook_templates(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Check if templates already exist
    let count: i64 = db_guard
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM hooks WHERE is_template = 1",
            [],
            |row| row.get(0),
        )
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

// ============================================================================
// Hook Export and Sound Hook Creation
// ============================================================================

/// Export hooks to Claude Code settings.json format
#[tauri::command]
pub fn export_hooks_to_json(
    db: State<'_, Arc<Mutex<Database>>>,
    hook_ids: Vec<i64>,
) -> Result<String, String> {
    info!("[Hooks] Exporting {} hooks to JSON", hook_ids.len());
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Fetch the specified hooks
    let mut hooks: Vec<Hook> = Vec::new();
    for id in hook_ids {
        let mut stmt = db_guard
            .conn()
            .prepare(&format!(
                "SELECT {} FROM hooks WHERE id = ?",
                HOOK_SELECT_FIELDS
            ))
            .map_err(|e| e.to_string())?;

        if let Ok(hook) = stmt.query_row([id], row_to_hook) {
            hooks.push(hook);
        }
    }

    // Convert to Claude Code settings.json format
    let export = hook_writer::hooks_to_settings_format(&hooks);
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

/// Create sound notification hooks for common events
#[tauri::command]
pub fn create_sound_notification_hooks(
    db: State<'_, Arc<Mutex<Database>>>,
    events: Vec<String>,
    sound_path: String,
    method: String,
) -> Result<Vec<Hook>, String> {
    use crate::services::sound_player;

    info!(
        "[Hooks] Creating sound notification hooks for events: {:?}",
        events
    );

    let db_guard = db.lock().map_err(|e| e.to_string())?;
    let command = sound_player::generate_play_command(&sound_path, &method);

    let mut created_hooks = Vec::new();

    for event in events {
        // Generate a descriptive name
        let name = format!("Sound-{}", event);
        let description = format!("Play notification sound on {} event", event);

        // Check if a hook with this name already exists
        let existing: Option<i64> = db_guard
            .conn()
            .query_row(
                "SELECT id FROM hooks WHERE name = ? AND is_template = 0",
                [&name],
                |row| row.get(0),
            )
            .ok();

        if existing.is_some() {
            info!("[Hooks] Hook '{}' already exists, skipping", name);
            continue;
        }

        // Create the hook
        db_guard
            .conn()
            .execute(
                "INSERT INTO hooks (name, description, event_type, hook_type, command, timeout, tags, source)
                 VALUES (?, ?, ?, 'command', ?, 5, '[\"sound\",\"notification\"]', 'sound-wizard')",
                params![name, description, event, command],
            )
            .map_err(|e| e.to_string())?;

        let id = db_guard.conn().last_insert_rowid();

        // Fetch the created hook
        let mut stmt = db_guard
            .conn()
            .prepare(&format!(
                "SELECT {} FROM hooks WHERE id = ?",
                HOOK_SELECT_FIELDS
            ))
            .map_err(|e| e.to_string())?;

        let hook = stmt
            .query_row([id], row_to_hook)
            .map_err(|e| e.to_string())?;

        // Add to global hooks and enable
        db_guard
            .conn()
            .execute(
                "INSERT OR IGNORE INTO global_hooks (hook_id, is_enabled) VALUES (?, 1)",
                [id],
            )
            .map_err(|e| e.to_string())?;

        created_hooks.push(hook);
    }

    // Sync global hooks to settings.json
    if !created_hooks.is_empty() {
        sync_global_hooks(&db_guard)?;
    }

    info!(
        "[Hooks] Created {} sound notification hooks",
        created_hooks.len()
    );
    Ok(created_hooks)
}

/// Duplicate a hook with a new name
#[tauri::command]
pub fn duplicate_hook(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    new_name: String,
) -> Result<Hook, String> {
    info!("[Hooks] Duplicating hook id={} with name '{}'", id, new_name);
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    // Get the original hook
    let mut stmt = db_guard
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE id = ?",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let original: Hook = stmt
        .query_row([id], row_to_hook)
        .map_err(|e| e.to_string())?;

    // Create the duplicate
    db_guard
        .conn()
        .execute(
            "INSERT INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source, is_template)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual', 0)",
            params![
                new_name,
                original.description,
                original.event_type,
                original.matcher,
                original.hook_type,
                original.command,
                original.prompt,
                original.timeout,
                original.tags.as_ref().map(|t| serde_json::to_string(t).unwrap())
            ],
        )
        .map_err(|e| e.to_string())?;

    let new_id = db_guard.conn().last_insert_rowid();

    let mut stmt = db_guard
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE id = ?",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    stmt.query_row([new_id], row_to_hook)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Create a hook directly in the database (for testing)
pub fn create_hook_in_db(db: &Database, hook: &CreateHookRequest) -> Result<Hook, String> {
    let tags_json = hook
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());

    db.conn()
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

    let id = db.conn().last_insert_rowid();
    get_hook_by_id(db, id)
}

/// Get a hook by ID directly from the database (for testing)
pub fn get_hook_by_id(db: &Database, id: i64) -> Result<Hook, String> {
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE id = ?",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_hook).map_err(|e| e.to_string())
}

/// Get all hooks directly from the database (for testing)
pub fn get_all_hooks_from_db(db: &Database) -> Result<Vec<Hook>, String> {
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT {} FROM hooks WHERE is_template = 0 ORDER BY name",
            HOOK_SELECT_FIELDS
        ))
        .map_err(|e| e.to_string())?;

    let hooks: Vec<Hook> = stmt
        .query_map([], row_to_hook)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hooks)
}

/// Update a hook directly in the database (for testing)
pub fn update_hook_in_db(db: &Database, id: i64, hook: &CreateHookRequest) -> Result<Hook, String> {
    let tags_json = hook
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap());

    db.conn()
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

    get_hook_by_id(db, id)
}

/// Delete a hook directly from the database (for testing)
pub fn delete_hook_from_db(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM hooks WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Add a hook to global hooks directly in the database (for testing)
pub fn add_global_hook_in_db(db: &Database, hook_id: i64) -> Result<(), String> {
    db.conn()
        .execute(
            "INSERT OR IGNORE INTO global_hooks (hook_id) VALUES (?)",
            [hook_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Get all global hooks directly from the database (for testing)
pub fn get_global_hooks_from_db(db: &Database) -> Result<Vec<GlobalHook>, String> {
    let mut stmt = db
        .conn()
        .prepare(&format!(
            "SELECT gh.id, gh.hook_id, gh.is_enabled, {}
             FROM global_hooks gh
             JOIN hooks h ON gh.hook_id = h.id
             ORDER BY h.name",
            HOOK_SELECT_FIELDS_H
        ))
        .map_err(|e| e.to_string())?;

    let hooks: Vec<GlobalHook> = stmt
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

/// Toggle a global hook directly in the database (for testing)
pub fn toggle_global_hook_in_db(db: &Database, id: i64, enabled: bool) -> Result<(), String> {
    db.conn()
        .execute(
            "UPDATE global_hooks SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Remove a global hook directly from the database (for testing)
pub fn remove_global_hook_from_db(db: &Database, hook_id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM global_hooks WHERE hook_id = ?", [hook_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Hook CRUD tests
    // =========================================================================

    #[test]
    fn test_create_hook_command_type() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "test-hook".to_string(),
            description: Some("Test hook".to_string()),
            event_type: "PostToolUse".to_string(),
            matcher: Some("Write|Edit".to_string()),
            hook_type: "command".to_string(),
            command: Some("npm run lint".to_string()),
            prompt: None,
            timeout: Some(30),
            tags: Some(vec!["lint".to_string(), "format".to_string()]),
        };

        let created = create_hook_in_db(&db, &hook).unwrap();

        assert_eq!(created.name, "test-hook");
        assert_eq!(created.description, Some("Test hook".to_string()));
        assert_eq!(created.event_type, "PostToolUse");
        assert_eq!(created.matcher, Some("Write|Edit".to_string()));
        assert_eq!(created.hook_type, "command");
        assert_eq!(created.command, Some("npm run lint".to_string()));
        assert_eq!(created.prompt, None);
        assert_eq!(created.timeout, Some(30));
        assert_eq!(
            created.tags,
            Some(vec!["lint".to_string(), "format".to_string()])
        );
        assert_eq!(created.source, "manual");
        assert!(!created.is_template);
    }

    #[test]
    fn test_create_hook_prompt_type() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "prompt-hook".to_string(),
            description: None,
            event_type: "PreToolUse".to_string(),
            matcher: None,
            hook_type: "prompt".to_string(),
            command: None,
            prompt: Some("Always verify before writing".to_string()),
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();

        assert_eq!(created.name, "prompt-hook");
        assert_eq!(created.hook_type, "prompt");
        assert_eq!(
            created.prompt,
            Some("Always verify before writing".to_string())
        );
        assert_eq!(created.command, None);
    }

    #[test]
    fn test_get_hook_by_id() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "findable-hook".to_string(),
            description: Some("Can be found".to_string()),
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("echo hello".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();
        let found = get_hook_by_id(&db, created.id).unwrap();

        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "findable-hook");
    }

    #[test]
    fn test_get_hook_by_id_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_hook_by_id(&db, 9999);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_hooks_from_db() {
        let db = Database::in_memory().unwrap();

        // Create some hooks
        for i in 1..=3 {
            let hook = CreateHookRequest {
                name: format!("hook-{}", i),
                description: None,
                event_type: "PostToolUse".to_string(),
                matcher: None,
                hook_type: "command".to_string(),
                command: Some("cmd".to_string()),
                prompt: None,
                timeout: None,
                tags: None,
            };
            create_hook_in_db(&db, &hook).unwrap();
        }

        let hooks = get_all_hooks_from_db(&db).unwrap();

        assert_eq!(hooks.len(), 3);
        // Should be sorted by name
        assert_eq!(hooks[0].name, "hook-1");
        assert_eq!(hooks[1].name, "hook-2");
        assert_eq!(hooks[2].name, "hook-3");
    }

    #[test]
    fn test_get_all_hooks_excludes_templates() {
        let db = Database::in_memory().unwrap();

        // Create a regular hook
        let hook = CreateHookRequest {
            name: "regular-hook".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };
        create_hook_in_db(&db, &hook).unwrap();

        // Create a template hook directly
        db.conn()
            .execute(
                "INSERT INTO hooks (name, event_type, hook_type, source, is_template) VALUES (?, ?, ?, ?, 1)",
                params!["template-hook", "PostToolUse", "command", "system"],
            )
            .unwrap();

        let hooks = get_all_hooks_from_db(&db).unwrap();

        // Should only have the regular hook
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].name, "regular-hook");
    }

    #[test]
    fn test_update_hook_in_db() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "original".to_string(),
            description: Some("Original description".to_string()),
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("original-cmd".to_string()),
            prompt: None,
            timeout: Some(10),
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();

        let update = CreateHookRequest {
            name: "updated".to_string(),
            description: Some("Updated description".to_string()),
            event_type: "PreToolUse".to_string(),
            matcher: Some("Bash".to_string()),
            hook_type: "command".to_string(),
            command: Some("updated-cmd".to_string()),
            prompt: None,
            timeout: Some(60),
            tags: Some(vec!["updated".to_string()]),
        };

        let updated = update_hook_in_db(&db, created.id, &update).unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "updated");
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.event_type, "PreToolUse");
        assert_eq!(updated.matcher, Some("Bash".to_string()));
        assert_eq!(updated.command, Some("updated-cmd".to_string()));
        assert_eq!(updated.timeout, Some(60));
        assert_eq!(updated.tags, Some(vec!["updated".to_string()]));
        // Verify the hook was actually updated in the database
        let refetched = get_hook_by_id(&db, created.id).unwrap();
        assert_eq!(refetched.name, "updated");
    }

    #[test]
    fn test_delete_hook_from_db() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "to-delete".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();
        delete_hook_from_db(&db, created.id).unwrap();

        let result = get_hook_by_id(&db, created.id);
        assert!(result.is_err());
    }

    // =========================================================================
    // Global Hook tests
    // =========================================================================

    #[test]
    fn test_add_global_hook() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "global-hook".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();
        add_global_hook_in_db(&db, created.id).unwrap();

        let global_hooks = get_global_hooks_from_db(&db).unwrap();

        assert_eq!(global_hooks.len(), 1);
        assert_eq!(global_hooks[0].hook_id, created.id);
        assert!(global_hooks[0].is_enabled); // Default enabled
    }

    #[test]
    fn test_toggle_global_hook() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "toggle-hook".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();
        add_global_hook_in_db(&db, created.id).unwrap();

        let global_hooks = get_global_hooks_from_db(&db).unwrap();
        let global_id = global_hooks[0].id;

        // Disable
        toggle_global_hook_in_db(&db, global_id, false).unwrap();
        let hooks = get_global_hooks_from_db(&db).unwrap();
        assert!(!hooks[0].is_enabled);

        // Re-enable
        toggle_global_hook_in_db(&db, global_id, true).unwrap();
        let hooks = get_global_hooks_from_db(&db).unwrap();
        assert!(hooks[0].is_enabled);
    }

    #[test]
    fn test_remove_global_hook() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "removable-hook".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();
        add_global_hook_in_db(&db, created.id).unwrap();

        // Verify it's there
        let hooks = get_global_hooks_from_db(&db).unwrap();
        assert_eq!(hooks.len(), 1);

        // Remove
        remove_global_hook_from_db(&db, created.id).unwrap();

        // Verify it's gone
        let hooks = get_global_hooks_from_db(&db).unwrap();
        assert!(hooks.is_empty());

        // But the hook itself should still exist
        let found = get_hook_by_id(&db, created.id).unwrap();
        assert_eq!(found.name, "removable-hook");
    }

    #[test]
    fn test_add_global_hook_duplicate_ignored() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "dup-hook".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();

        // Add twice
        add_global_hook_in_db(&db, created.id).unwrap();
        add_global_hook_in_db(&db, created.id).unwrap();

        // Should only have one
        let hooks = get_global_hooks_from_db(&db).unwrap();
        assert_eq!(hooks.len(), 1);
    }

    // =========================================================================
    // Edge case tests
    // =========================================================================

    #[test]
    fn test_hook_with_all_event_types() {
        let db = Database::in_memory().unwrap();

        let event_types = vec!["PreToolUse", "PostToolUse", "Notification", "Stop"];

        for event_type in event_types {
            let hook = CreateHookRequest {
                name: format!("hook-{}", event_type),
                description: None,
                event_type: event_type.to_string(),
                matcher: None,
                hook_type: "command".to_string(),
                command: Some("cmd".to_string()),
                prompt: None,
                timeout: None,
                tags: None,
            };

            let created = create_hook_in_db(&db, &hook).unwrap();
            assert_eq!(created.event_type, event_type);
        }
    }

    #[test]
    fn test_hook_with_complex_matcher() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "complex-matcher".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: Some("Write|Edit|Bash|Task".to_string()),
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let created = create_hook_in_db(&db, &hook).unwrap();
        assert_eq!(created.matcher, Some("Write|Edit|Bash|Task".to_string()));
    }

    #[test]
    fn test_hook_with_empty_tags() {
        let db = Database::in_memory().unwrap();

        let hook = CreateHookRequest {
            name: "empty-tags".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("cmd".to_string()),
            prompt: None,
            timeout: None,
            tags: Some(vec![]),
        };

        let created = create_hook_in_db(&db, &hook).unwrap();
        // Empty vec serializes to "[]" which deserializes back to Some(vec![])
        assert_eq!(created.tags, Some(vec![]));
    }
}
