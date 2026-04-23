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
    permission_writer::write_permission_rules(&ps, pp, &category, &rules).map_err(|e| e.to_string())
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
    let path = permission_writer::resolve_settings_path(&ps, pp).map_err(|e| e.to_string())?;
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

    permission_writer::write_permission_rules(&ps, pp, &category, &rules).map_err(|e| e.to_string())
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

    let path = permission_writer::resolve_settings_path(&ps, pp).map_err(|e| e.to_string())?;
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

    permission_writer::write_permission_rules(&ps, pp, &category, &rules).map_err(|e| e.to_string())
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
    permission_writer::write_permission_rules(&ps, pp, &category, &rules).map_err(|e| e.to_string())
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

/// Get permission templates from the database (no Tauri State dependency)
pub(crate) fn get_permission_templates_impl(
    db: &Database,
) -> Result<Vec<PermissionTemplate>, String> {
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

    Ok(templates)
}

/// Get permission templates from the database
#[tauri::command]
pub fn get_permission_templates(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<PermissionTemplate>, String> {
    info!("[Permissions] Loading permission templates");
    let db = db.lock().map_err(|e| e.to_string())?;
    let templates = get_permission_templates_impl(&db)?;
    info!(
        "[Permissions] Loaded {} permission templates",
        templates.len()
    );
    Ok(templates)
}

/// Seed default permission templates (no Tauri State dependency)
#[allow(clippy::type_complexity)]
pub(crate) fn seed_permission_templates_impl(db: &Database) -> Result<(), String> {
    // Check if templates already exist
    let count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM permission_templates", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    let templates: Vec<(&str, &str, &str, &str, Option<&str>, &str)> = vec![
        // ALLOW templates
        (
            "Allow npm scripts",
            "Allow running npm/yarn/pnpm scripts",
            "allow",
            "Bash(npm run *)",
            Some("Bash"),
            "development",
        ),
        (
            "Allow git commands",
            "Allow all git operations",
            "allow",
            "Bash(git *)",
            Some("Bash"),
            "git",
        ),
        (
            "Allow cargo commands",
            "Allow Rust cargo commands",
            "allow",
            "Bash(cargo *)",
            Some("Bash"),
            "development",
        ),
        (
            "Allow file reading",
            "Allow reading any file",
            "allow",
            "Read",
            Some("Read"),
            "files",
        ),
        (
            "Allow file writing",
            "Allow writing files",
            "allow",
            "Write",
            Some("Write"),
            "files",
        ),
        (
            "Allow file editing",
            "Allow editing files",
            "allow",
            "Edit",
            Some("Edit"),
            "files",
        ),
        // DENY templates
        (
            "Block curl/wget",
            "Prevent network downloads via shell",
            "deny",
            "Bash(curl *)",
            Some("Bash"),
            "security",
        ),
        (
            "Block rm -rf",
            "Prevent recursive force deletion",
            "deny",
            "Bash(rm -rf *)",
            Some("Bash"),
            "security",
        ),
        (
            "Block .env access",
            "Prevent reading environment files",
            "deny",
            "Read(.env*)",
            Some("Read"),
            "security",
        ),
        (
            "Block secrets access",
            "Prevent reading credential files",
            "deny",
            "Read(*credentials*)",
            Some("Read"),
            "security",
        ),
        (
            "Block sudo",
            "Prevent privilege escalation",
            "deny",
            "Bash(sudo *)",
            Some("Bash"),
            "security",
        ),
        // ASK templates
        (
            "Ask for git push",
            "Confirm before pushing to remote",
            "ask",
            "Bash(git push *)",
            Some("Bash"),
            "git",
        ),
        (
            "Ask for npm install",
            "Confirm before installing packages",
            "ask",
            "Bash(npm install *)",
            Some("Bash"),
            "development",
        ),
        (
            "Ask for file deletion",
            "Confirm before deleting files",
            "ask",
            "Bash(rm *)",
            Some("Bash"),
            "safety",
        ),
        (
            "Ask for web fetch",
            "Confirm before fetching URLs",
            "ask",
            "WebFetch",
            Some("WebFetch"),
            "network",
        ),
    ];

    let _count = templates.len();
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

    Ok(())
}

/// Seed default permission templates
#[tauri::command]
pub fn seed_permission_templates(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    info!("[Permissions] Seeding permission templates");
    let db = db.lock().map_err(|e| e.to_string())?;
    seed_permission_templates_impl(&db)?;
    info!("[Permissions] Permission templates seeded");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scope_user() {
        assert!(matches!(parse_scope("user"), Ok(PermissionScope::User)));
    }

    #[test]
    fn test_parse_scope_project() {
        assert!(matches!(
            parse_scope("project"),
            Ok(PermissionScope::Project)
        ));
    }

    #[test]
    fn test_parse_scope_local() {
        assert!(matches!(parse_scope("local"), Ok(PermissionScope::Local)));
    }

    #[test]
    fn test_parse_scope_invalid() {
        let result = parse_scope("global");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid scope"));
    }

    #[test]
    fn test_parse_scope_empty() {
        assert!(parse_scope("").is_err());
    }

    #[test]
    fn test_parse_json_array_valid() {
        let input = Some(r#"["tag1","tag2"]"#.to_string());
        let result = parse_json_array(input);
        assert_eq!(result, Some(vec!["tag1".to_string(), "tag2".to_string()]));
    }

    #[test]
    fn test_parse_json_array_none() {
        assert_eq!(parse_json_array(None), None);
    }

    #[test]
    fn test_parse_json_array_invalid() {
        assert_eq!(parse_json_array(Some("not json".to_string())), None);
    }

    #[test]
    fn test_permission_template_serde() {
        let template = PermissionTemplate {
            id: 1,
            name: "Allow npm scripts".to_string(),
            description: Some("Allow running npm/yarn/pnpm scripts".to_string()),
            category: "allow".to_string(),
            rule: "Bash(npm run *)".to_string(),
            tool_name: Some("Bash".to_string()),
            tags: Some(vec!["development".to_string()]),
            is_default: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&template).unwrap();
        let deser: PermissionTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "Allow npm scripts");
        assert_eq!(deser.category, "allow");
        assert!(deser.is_default);
    }

    #[test]
    fn test_permission_template_serde_camel_case() {
        let template = PermissionTemplate {
            id: 1,
            name: "test".to_string(),
            description: None,
            category: "deny".to_string(),
            rule: "Bash(rm -rf *)".to_string(),
            tool_name: Some("Bash".to_string()),
            tags: None,
            is_default: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("toolName"));
        assert!(json.contains("isDefault"));
        assert!(json.contains("createdAt"));
    }

    #[test]
    fn test_permission_template_minimal() {
        let template = PermissionTemplate {
            id: 1,
            name: "min".to_string(),
            description: None,
            category: "ask".to_string(),
            rule: "WebFetch".to_string(),
            tool_name: None,
            tags: None,
            is_default: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&template).unwrap();
        let deser: PermissionTemplate = serde_json::from_str(&json).unwrap();
        assert!(deser.tool_name.is_none());
        assert!(deser.tags.is_none());
        assert!(deser.description.is_none());
    }

    #[test]
    fn test_parse_scope_all_valid() {
        // Verify all valid scopes work
        let scopes = vec!["user", "project", "local"];
        for s in scopes {
            assert!(parse_scope(s).is_ok());
        }
    }

    #[test]
    fn test_parse_scope_case_sensitive() {
        assert!(parse_scope("User").is_err());
        assert!(parse_scope("PROJECT").is_err());
        assert!(parse_scope("Local").is_err());
    }

    #[test]
    fn test_parse_json_array_empty_array() {
        let result = parse_json_array(Some("[]".to_string()));
        assert_eq!(result, Some(vec![]));
    }

    #[test]
    fn test_parse_json_array_single_element() {
        let result = parse_json_array(Some(r#"["only"]"#.to_string()));
        assert_eq!(result, Some(vec!["only".to_string()]));
    }

    // =========================================================================
    // _impl function tests (extracted business logic)
    // =========================================================================

    #[test]
    fn test_get_permission_templates_impl_empty() {
        let db = Database::in_memory().unwrap();
        let templates = get_permission_templates_impl(&db).unwrap();
        assert!(templates.is_empty());
    }

    #[test]
    fn test_seed_permission_templates_impl() {
        let db = Database::in_memory().unwrap();

        // First seed should succeed
        seed_permission_templates_impl(&db).unwrap();

        let templates = get_permission_templates_impl(&db).unwrap();
        assert!(!templates.is_empty());
        assert!(templates.len() >= 14); // We have at least 14 templates defined
    }

    #[test]
    fn test_seed_permission_templates_impl_idempotent() {
        let db = Database::in_memory().unwrap();

        seed_permission_templates_impl(&db).unwrap();
        let count1 = get_permission_templates_impl(&db).unwrap().len();

        // Second seed should be a no-op
        seed_permission_templates_impl(&db).unwrap();
        let count2 = get_permission_templates_impl(&db).unwrap().len();

        assert_eq!(count1, count2);
    }

    #[test]
    fn test_seed_permission_templates_impl_categories() {
        let db = Database::in_memory().unwrap();
        seed_permission_templates_impl(&db).unwrap();

        let templates = get_permission_templates_impl(&db).unwrap();
        let allow_count = templates.iter().filter(|t| t.category == "allow").count();
        let deny_count = templates.iter().filter(|t| t.category == "deny").count();
        let ask_count = templates.iter().filter(|t| t.category == "ask").count();

        assert!(allow_count > 0, "Should have allow templates");
        assert!(deny_count > 0, "Should have deny templates");
        assert!(ask_count > 0, "Should have ask templates");
    }

    #[test]
    fn test_seed_permission_templates_impl_all_default() {
        let db = Database::in_memory().unwrap();
        seed_permission_templates_impl(&db).unwrap();

        let templates = get_permission_templates_impl(&db).unwrap();
        for t in &templates {
            assert!(
                t.is_default,
                "Seeded template '{}' should be is_default",
                t.name
            );
        }
    }

    #[test]
    fn test_seed_permission_templates_impl_ordered_by_category_name() {
        let db = Database::in_memory().unwrap();
        seed_permission_templates_impl(&db).unwrap();

        let templates = get_permission_templates_impl(&db).unwrap();
        // Should be ordered by category, then name
        for i in 1..templates.len() {
            let prev = &templates[i - 1];
            let curr = &templates[i];
            assert!(
                (prev.category.as_str(), prev.name.as_str())
                    <= (curr.category.as_str(), curr.name.as_str()),
                "Templates should be ordered by category then name"
            );
        }
    }

    #[test]
    fn test_permission_template_has_tool_name() {
        let db = Database::in_memory().unwrap();
        seed_permission_templates_impl(&db).unwrap();

        let templates = get_permission_templates_impl(&db).unwrap();
        // Most templates should have a tool_name
        let with_tool = templates.iter().filter(|t| t.tool_name.is_some()).count();
        assert!(with_tool > 0, "Some templates should have tool_name set");
    }

    #[test]
    fn test_permission_template_has_tags() {
        let db = Database::in_memory().unwrap();
        seed_permission_templates_impl(&db).unwrap();

        let templates = get_permission_templates_impl(&db).unwrap();
        for t in &templates {
            assert!(t.tags.is_some(), "Template '{}' should have tags", t.name);
            assert!(
                !t.tags.as_ref().unwrap().is_empty(),
                "Template '{}' should have non-empty tags",
                t.name
            );
        }
    }
}
