use crate::db::models::{CreateRuleRequest, GlobalRule, ProjectRule, Rule};
use crate::db::schema::Database;
use crate::services::rule_writer;
use rusqlite::params;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::State;

fn parse_json_array(s: Option<String>) -> Option<Vec<String>> {
    s.and_then(|v| serde_json::from_str(&v).ok())
}

const RULE_SELECT_FIELDS: &str = "id, name, description, content, paths, tags, source, source_path, is_symlink, symlink_target, is_favorite, created_at, updated_at";

fn row_to_rule(row: &rusqlite::Row) -> rusqlite::Result<Rule> {
    Ok(Rule {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        content: row.get(3)?,
        paths: parse_json_array(row.get(4)?),
        tags: parse_json_array(row.get(5)?),
        source: row.get(6)?,
        source_path: row.get(7)?,
        is_symlink: row.get::<_, i32>(8).unwrap_or(0) != 0,
        symlink_target: row.get(9)?,
        is_favorite: row.get::<_, i32>(10).unwrap_or(0) != 0,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn row_to_rule_with_offset(row: &rusqlite::Row, offset: usize) -> rusqlite::Result<Rule> {
    Ok(Rule {
        id: row.get(offset)?,
        name: row.get(offset + 1)?,
        description: row.get(offset + 2)?,
        content: row.get(offset + 3)?,
        paths: parse_json_array(row.get(offset + 4)?),
        tags: parse_json_array(row.get(offset + 5)?),
        source: row.get(offset + 6)?,
        source_path: row.get(offset + 7)?,
        is_symlink: row.get::<_, i32>(offset + 8).unwrap_or(0) != 0,
        symlink_target: row.get(offset + 9)?,
        is_favorite: row.get::<_, i32>(offset + 10).unwrap_or(0) != 0,
        created_at: row.get(offset + 11)?,
        updated_at: row.get(offset + 12)?,
    })
}

// ============================================================================
// Library CRUD
// ============================================================================

#[tauri::command]
pub fn get_all_rules(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Rule>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!("SELECT {} FROM rules ORDER BY name", RULE_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([], row_to_rule)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rules)
}

#[tauri::command]
pub fn create_rule(
    db: State<'_, Arc<Mutex<Database>>>,
    rule: CreateRuleRequest,
) -> Result<Rule, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let paths_json = rule
        .paths
        .as_ref()
        .filter(|p| !p.is_empty())
        .map(|p| serde_json::to_string(p).unwrap());

    let tags_json = rule
        .tags
        .as_ref()
        .filter(|t| !t.is_empty())
        .map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "INSERT INTO rules (name, description, content, paths, tags) VALUES (?, ?, ?, ?, ?)",
            params![
                rule.name,
                rule.description,
                rule.content,
                paths_json,
                tags_json
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    let query = format!("SELECT {} FROM rules WHERE id = ?", RULE_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_rule(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    rule: CreateRuleRequest,
) -> Result<Rule, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let paths_json = rule
        .paths
        .as_ref()
        .filter(|p| !p.is_empty())
        .map(|p| serde_json::to_string(p).unwrap());

    let tags_json = rule
        .tags
        .as_ref()
        .filter(|t| !t.is_empty())
        .map(|t| serde_json::to_string(t).unwrap());

    db.conn()
        .execute(
            "UPDATE rules SET name = ?, description = ?, content = ?, paths = ?, tags = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![rule.name, rule.description, rule.content, paths_json, tags_json, id],
        )
        .map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM rules WHERE id = ?", RULE_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    stmt.query_row([id], row_to_rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_rule(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute("DELETE FROM rules WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_rule_favorite(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    favorite: bool,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.conn()
        .execute(
            "UPDATE rules SET is_favorite = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![favorite as i32, id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ============================================================================
// Global Rules
// ============================================================================

#[tauri::command]
pub fn get_global_rules(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<GlobalRule>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT gr.id, gr.rule_id, gr.is_enabled,
                r.{}
         FROM global_rules gr
         JOIN rules r ON gr.rule_id = r.id
         ORDER BY r.name",
        RULE_SELECT_FIELDS
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([], |row| {
            Ok(GlobalRule {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                rule: row_to_rule_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rules)
}

#[tauri::command]
pub fn add_global_rule(db: State<'_, Arc<Mutex<Database>>>, rule_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM rules WHERE id = ?", RULE_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;
    let rule: Rule = stmt
        .query_row([rule_id], row_to_rule)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT OR IGNORE INTO global_rules (rule_id) VALUES (?)",
            [rule_id],
        )
        .map_err(|e| e.to_string())?;

    rule_writer::write_global_rule(&rule).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_global_rule(db: State<'_, Arc<Mutex<Database>>>, rule_id: i64) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM rules WHERE id = ?", RULE_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;
    let rule: Rule = stmt
        .query_row([rule_id], row_to_rule)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute("DELETE FROM global_rules WHERE rule_id = ?", [rule_id])
        .map_err(|e| e.to_string())?;

    rule_writer::delete_global_rule(&rule).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn toggle_global_rule(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE global_rules SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, id],
        )
        .map_err(|e| e.to_string())?;

    let query = format!(
        "SELECT r.{}
         FROM global_rules gr
         JOIN rules r ON gr.rule_id = r.id
         WHERE gr.id = ?",
        RULE_SELECT_FIELDS
    );
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;
    let rule: Rule = stmt
        .query_row([id], row_to_rule)
        .map_err(|e| e.to_string())?;

    if enabled {
        rule_writer::write_global_rule(&rule).map_err(|e| e.to_string())?;
    } else {
        rule_writer::delete_global_rule(&rule).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Project Rules
// ============================================================================

#[tauri::command(rename_all = "camelCase")]
pub fn get_project_rules(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
) -> Result<Vec<ProjectRule>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!(
        "SELECT pr.id, pr.rule_id, pr.is_enabled,
                r.{}
         FROM project_rules pr
         JOIN rules r ON pr.rule_id = r.id
         WHERE pr.project_id = ?
         ORDER BY r.name",
        RULE_SELECT_FIELDS
    );
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([project_id], |row| {
            Ok(ProjectRule {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                is_enabled: row.get::<_, i32>(2)? != 0,
                rule: row_to_rule_with_offset(row, 3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rules)
}

#[tauri::command(rename_all = "camelCase")]
pub fn assign_rule_to_project(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    rule_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM rules WHERE id = ?", RULE_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;
    let rule: Rule = stmt
        .query_row([rule_id], row_to_rule)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "INSERT OR IGNORE INTO project_rules (project_id, rule_id) VALUES (?, ?)",
            params![project_id, rule_id],
        )
        .map_err(|e| e.to_string())?;

    rule_writer::write_project_rule(Path::new(&project_path), &rule).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn remove_rule_from_project(
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    rule_id: i64,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    let project_path: String = db_guard
        .conn()
        .query_row(
            "SELECT path FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let query = format!("SELECT {} FROM rules WHERE id = ?", RULE_SELECT_FIELDS);
    let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;
    let rule: Rule = stmt
        .query_row([rule_id], row_to_rule)
        .map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "DELETE FROM project_rules WHERE project_id = ? AND rule_id = ?",
            params![project_id, rule_id],
        )
        .map_err(|e| e.to_string())?;

    rule_writer::delete_project_rule(Path::new(&project_path), &rule).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub fn toggle_project_rule(
    db: State<'_, Arc<Mutex<Database>>>,
    assignment_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| e.to_string())?;

    db_guard
        .conn()
        .execute(
            "UPDATE project_rules SET is_enabled = ? WHERE id = ?",
            params![enabled as i32, assignment_id],
        )
        .map_err(|e| e.to_string())?;

    // Get the project path and rule
    let (project_path, rule): (String, Rule) = {
        let query = format!(
            "SELECT p.path, r.{}
             FROM project_rules pr
             JOIN rules r ON pr.rule_id = r.id
             JOIN projects p ON pr.project_id = p.id
             WHERE pr.id = ?",
            RULE_SELECT_FIELDS
        );
        let mut stmt = db_guard.conn().prepare(&query).map_err(|e| e.to_string())?;
        stmt.query_row([assignment_id], |row| {
            Ok((row.get::<_, String>(0)?, row_to_rule_with_offset(row, 1)?))
        })
        .map_err(|e| e.to_string())?
    };

    if enabled {
        rule_writer::write_project_rule(Path::new(&project_path), &rule)
            .map_err(|e| e.to_string())?;
    } else {
        rule_writer::delete_project_rule(Path::new(&project_path), &rule)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Active Rules (glob matching)
// ============================================================================

#[tauri::command(rename_all = "camelCase")]
pub fn get_active_rules_for_path(
    db: State<'_, Arc<Mutex<Database>>>,
    file_path: String,
) -> Result<Vec<Rule>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let query = format!("SELECT {} FROM rules", RULE_SELECT_FIELDS);
    let mut stmt = db.conn().prepare(&query).map_err(|e| e.to_string())?;

    let all_rules: Vec<Rule> = stmt
        .query_map([], row_to_rule)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let active = all_rules
        .into_iter()
        .filter(|rule| {
            match &rule.paths {
                None => true, // No paths = always active
                Some(patterns) if patterns.is_empty() => true,
                Some(patterns) => patterns
                    .iter()
                    .any(|pattern| glob_match(pattern, &file_path)),
            }
        })
        .collect();

    Ok(active)
}

/// Simple glob matching (supports * and ** patterns)
fn glob_match(pattern: &str, path: &str) -> bool {
    // Convert glob pattern to a simple matcher
    let pattern = pattern.trim();
    let path = path.trim();

    if pattern == "*" || pattern == "**" || pattern == "**/*" {
        return true;
    }

    // Use simple suffix matching for common patterns like "src/**/*.ts"
    if let Some(ext_pattern) = pattern.strip_prefix("**/*") {
        return path.ends_with(ext_pattern);
    }

    // For patterns like "src/**/*.ts", check prefix and extension
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.splitn(2, "**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            let prefix_match = prefix.is_empty() || path.starts_with(prefix);
            let suffix_match = if suffix.starts_with('*') {
                let ext = suffix.trim_start_matches('*');
                ext.is_empty() || path.ends_with(ext)
            } else {
                suffix.is_empty() || path.ends_with(suffix)
            };

            return prefix_match && suffix_match;
        }
    }

    // For patterns with single *, do basic matching (single * does not cross /)
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            if path.starts_with(parts[0]) && path.ends_with(parts[1]) {
                // Ensure the matched middle portion contains no path separators
                let middle = &path[parts[0].len()..path.len() - parts[1].len()];
                return !middle.contains('/');
            }
            return false;
        }
    }

    // Exact match
    path == pattern || path.ends_with(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_wildcard() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("**", "anything"));
    }

    #[test]
    fn test_glob_match_extension() {
        assert!(glob_match("**/*.ts", "src/foo/bar.ts"));
        assert!(!glob_match("**/*.ts", "src/foo/bar.js"));
    }

    #[test]
    fn test_glob_match_prefix_and_extension() {
        assert!(glob_match("src/**/*.ts", "src/foo/bar.ts"));
        assert!(!glob_match("src/**/*.ts", "tests/foo/bar.ts"));
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("README.md", "README.md"));
    }

    #[test]
    fn test_glob_match_simple_star() {
        assert!(glob_match("src/*.ts", "src/index.ts"));
        assert!(!glob_match("src/*.ts", "src/deep/index.ts"));
    }
}
