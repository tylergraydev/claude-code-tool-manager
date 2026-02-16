use crate::db::models::PermissionTemplate;
use crate::db::schema::Database;
use crate::services::permission_writer::{self, AllPermissions, PermissionScope};
use log::info;
use rusqlite::params;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::State;

fn parse_scope(scope: &str) -> Result<PermissionScope, String> {
    match scope {
        "user" => Ok(PermissionScope::User),
        "project" => Ok(PermissionScope::Project),
        "local" => Ok(PermissionScope::Local),
        _ => Err(format!("Invalid scope: {}", scope)),
    }
}

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

/// Get all permissions from all three scopes
#[tauri::command]
pub fn get_all_permissions(project_path: Option<String>) -> Result<AllPermissions, String> {
    info!(
        "[Permissions] Reading all permissions (project={:?})",
        project_path
    );
    let pp = project_path.as_deref().map(Path::new);
    permission_writer::read_all_permissions(pp).map_err(|e| e.to_string())
}

/// Set permission rules for a specific category (replace full array)
#[tauri::command]
pub fn set_permission_rules(
    scope: String,
    project_path: Option<String>,
    category: String,
    rules: Vec<String>,
) -> Result<(), String> {
    info!(
        "[Permissions] Setting {} rules for scope={} ({} rules)",
        category,
        scope,
        rules.len()
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    permission_writer::write_permission_rules(&ps, pp, &category, &rules)
        .map_err(|e| e.to_string())
}

/// Add a single permission rule to a category
#[tauri::command]
pub fn add_permission_rule(
    scope: String,
    project_path: Option<String>,
    category: String,
    rule: String,
) -> Result<(), String> {
    info!(
        "[Permissions] Adding rule '{}' to {} in scope={}",
        rule, category, scope
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);

    // Read current rules, append, write back
    let path =
        permission_writer::resolve_settings_path(&ps, pp).map_err(|e| e.to_string())?;
    let perms =
        permission_writer::read_permissions_from_file(&path, &scope).map_err(|e| e.to_string())?;

    let mut rules = match category.as_str() {
        "allow" => perms.allow,
        "deny" => perms.deny,
        "ask" => perms.ask,
        _ => return Err(format!("Invalid category: {}", category)),
    };

    // Avoid duplicates
    if !rules.contains(&rule) {
        rules.push(rule);
    }

    permission_writer::write_permission_rules(&ps, pp, &category, &rules)
        .map_err(|e| e.to_string())
}

/// Remove a permission rule by index
#[tauri::command]
pub fn remove_permission_rule(
    scope: String,
    project_path: Option<String>,
    category: String,
    index: usize,
) -> Result<(), String> {
    info!(
        "[Permissions] Removing rule at index {} from {} in scope={}",
        index, category, scope
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);

    let path =
        permission_writer::resolve_settings_path(&ps, pp).map_err(|e| e.to_string())?;
    let perms =
        permission_writer::read_permissions_from_file(&path, &scope).map_err(|e| e.to_string())?;

    let mut rules = match category.as_str() {
        "allow" => perms.allow,
        "deny" => perms.deny,
        "ask" => perms.ask,
        _ => return Err(format!("Invalid category: {}", category)),
    };

    if index >= rules.len() {
        return Err(format!(
            "Index {} out of bounds (len={})",
            index,
            rules.len()
        ));
    }

    rules.remove(index);

    permission_writer::write_permission_rules(&ps, pp, &category, &rules)
        .map_err(|e| e.to_string())
}

/// Reorder permission rules (replace array with new order)
#[tauri::command]
pub fn reorder_permission_rules(
    scope: String,
    project_path: Option<String>,
    category: String,
    rules: Vec<String>,
) -> Result<(), String> {
    info!(
        "[Permissions] Reordering {} rules in {} scope={}",
        rules.len(),
        category,
        scope
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    permission_writer::write_permission_rules(&ps, pp, &category, &rules)
        .map_err(|e| e.to_string())
}

/// Set the defaultMode for a scope
#[tauri::command]
pub fn set_default_mode(
    scope: String,
    project_path: Option<String>,
    mode: Option<String>,
) -> Result<(), String> {
    info!(
        "[Permissions] Setting defaultMode={:?} for scope={}",
        mode, scope
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    permission_writer::write_default_mode(&ps, pp, mode.as_deref()).map_err(|e| e.to_string())
}

/// Set additionalDirectories for a scope
#[tauri::command]
pub fn set_additional_directories(
    scope: String,
    project_path: Option<String>,
    directories: Vec<String>,
) -> Result<(), String> {
    info!(
        "[Permissions] Setting {} additionalDirectories for scope={}",
        directories.len(),
        scope
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    permission_writer::write_additional_directories(&ps, pp, &directories)
        .map_err(|e| e.to_string())
}

/// Get permission templates from the database
#[tauri::command]
pub fn get_permission_templates(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<PermissionTemplate>, String> {
    info!("[Permissions] Loading permission templates");
    let db = db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, category, rule, tool_name, tags, is_default, created_at, updated_at
             FROM permission_templates ORDER BY category, name",
        )
        .map_err(|e| e.to_string())?;

    let templates: Vec<PermissionTemplate> = stmt
        .query_map([], |row| {
            Ok(PermissionTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                rule: row.get(4)?,
                tool_name: row.get(5)?,
                tags: parse_json_array(row.get(6)?),
                is_default: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    info!(
        "[Permissions] Loaded {} permission templates",
        templates.len()
    );
    Ok(templates)
}

/// Seed default permission templates
#[tauri::command]
pub fn seed_permission_templates(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    info!("[Permissions] Seeding permission templates");
    let db = db.lock().map_err(|e| e.to_string())?;

    // Check if templates already exist
    let count: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM permission_templates",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    let templates: Vec<(&str, &str, &str, &str, Option<&str>, &str)> = vec![
        // ALLOW templates
        ("Allow npm scripts", "Allow running npm/yarn/pnpm scripts", "allow", "Bash(npm run *)", Some("Bash"), "development"),
        ("Allow git commands", "Allow all git operations", "allow", "Bash(git *)", Some("Bash"), "git"),
        ("Allow cargo commands", "Allow Rust cargo commands", "allow", "Bash(cargo *)", Some("Bash"), "development"),
        ("Allow file reading", "Allow reading any file", "allow", "Read", Some("Read"), "files"),
        ("Allow file writing", "Allow writing files", "allow", "Write", Some("Write"), "files"),
        ("Allow file editing", "Allow editing files", "allow", "Edit", Some("Edit"), "files"),
        // DENY templates
        ("Block curl/wget", "Prevent network downloads via shell", "deny", "Bash(curl *)", Some("Bash"), "security"),
        ("Block rm -rf", "Prevent recursive force deletion", "deny", "Bash(rm -rf *)", Some("Bash"), "security"),
        ("Block .env access", "Prevent reading environment files", "deny", "Read(.env*)", Some("Read"), "security"),
        ("Block secrets access", "Prevent reading credential files", "deny", "Read(*credentials*)", Some("Read"), "security"),
        ("Block sudo", "Prevent privilege escalation", "deny", "Bash(sudo *)", Some("Bash"), "security"),
        // ASK templates
        ("Ask for git push", "Confirm before pushing to remote", "ask", "Bash(git push *)", Some("Bash"), "git"),
        ("Ask for npm install", "Confirm before installing packages", "ask", "Bash(npm install *)", Some("Bash"), "development"),
        ("Ask for file deletion", "Confirm before deleting files", "ask", "Bash(rm *)", Some("Bash"), "safety"),
        ("Ask for web fetch", "Confirm before fetching URLs", "ask", "WebFetch", Some("WebFetch"), "network"),
    ];

    let count = templates.len();
    for (name, desc, category, rule, tool_name, tag) in templates {
        let tags_json = serde_json::to_string(&vec![tag]).unwrap();
        db.conn()
            .execute(
                "INSERT OR IGNORE INTO permission_templates (name, description, category, rule, tool_name, tags, is_default)
                 VALUES (?, ?, ?, ?, ?, ?, 1)",
                params![name, desc, category, rule, tool_name, tags_json],
            )
            .map_err(|e| e.to_string())?;
    }

    info!("[Permissions] Seeded {} permission templates", count);
    Ok(())
}
