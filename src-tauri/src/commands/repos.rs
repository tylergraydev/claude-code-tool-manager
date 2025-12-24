use crate::db::{
    CreateRepoRequest, Database, ImportResult, RateLimitInfo, Repo, RepoItem, SyncResult,
};
use crate::services::github_client::{parse_github_url, GitHubClient};
use crate::services::repo_parser::parse_frontmatter;
use crate::services::repo_sync;
use chrono::Utc;
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Get all repositories
#[tauri::command]
pub fn get_all_repos(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Repo>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    repo_sync::get_all_repos(&db).map_err(|e| e.to_string())
}

/// Add a new repository
#[tauri::command]
pub fn add_repo(
    db: State<'_, Arc<Mutex<Database>>>,
    request: CreateRepoRequest,
) -> Result<Repo, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let (owner, repo) =
        parse_github_url(&request.github_url).ok_or_else(|| "Invalid GitHub URL".to_string())?;

    let name = format!("{}/{}", owner, repo);

    db.conn()
        .execute(
            r#"INSERT INTO repos (name, owner, repo, repo_type, content_type, github_url)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            params![
                name,
                owner,
                repo,
                request.repo_type,
                request.content_type,
                request.github_url
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();

    // Fetch and return the created repo
    let created_repo = db
        .conn()
        .query_row(
            r#"SELECT id, name, owner, repo, repo_type, content_type, github_url, description,
                      is_default, is_enabled, last_fetched_at, etag, created_at, updated_at
               FROM repos WHERE id = ?"#,
            params![id],
            |row| {
                Ok(Repo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    owner: row.get(2)?,
                    repo: row.get(3)?,
                    repo_type: row.get(4)?,
                    content_type: row.get(5)?,
                    github_url: row.get(6)?,
                    description: row.get(7)?,
                    is_default: row.get::<_, i32>(8)? != 0,
                    is_enabled: row.get::<_, i32>(9)? != 0,
                    last_fetched_at: row.get(10)?,
                    etag: row.get(11)?,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(created_repo)
}

/// Remove a repository (only non-default repos can be removed)
#[tauri::command]
pub fn remove_repo(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Check if it's a default repo
    let is_default: bool = db
        .conn()
        .query_row(
            "SELECT is_default FROM repos WHERE id = ?",
            params![id],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        )
        .map_err(|e| e.to_string())?;

    if is_default {
        return Err(
            "Cannot remove default repositories. You can disable them instead.".to_string(),
        );
    }

    db.conn()
        .execute("DELETE FROM repos WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Toggle repository enabled status
#[tauri::command]
pub fn toggle_repo(db: State<'_, Arc<Mutex<Database>>>, id: i64, enabled: bool) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    db.conn()
        .execute(
            "UPDATE repos SET is_enabled = ?, updated_at = ? WHERE id = ?",
            params![enabled as i32, Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get items from a specific repository
#[tauri::command]
pub fn get_repo_items(
    db: State<'_, Arc<Mutex<Database>>>,
    repo_id: i64,
) -> Result<Vec<RepoItem>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    repo_sync::get_repo_items(&db, repo_id).map_err(|e| e.to_string())
}

/// Get all repository items, optionally filtered by type
#[tauri::command]
pub fn get_all_repo_items(
    db: State<'_, Arc<Mutex<Database>>>,
    item_type: Option<String>,
) -> Result<Vec<RepoItem>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    repo_sync::get_all_repo_items(&db, item_type).map_err(|e| e.to_string())
}

/// Sync a single repository
#[tauri::command]
pub async fn sync_repo(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<SyncResult, String> {
    // Get repo info first (need to release lock before async)
    let repo = {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.conn()
            .query_row(
                r#"SELECT id, name, owner, repo, repo_type, content_type, github_url, description,
                          is_default, is_enabled, last_fetched_at, etag, created_at, updated_at
                   FROM repos WHERE id = ?"#,
                params![id],
                |row| {
                    Ok(Repo {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        owner: row.get(2)?,
                        repo: row.get(3)?,
                        repo_type: row.get(4)?,
                        content_type: row.get(5)?,
                        github_url: row.get(6)?,
                        description: row.get(7)?,
                        is_default: row.get::<_, i32>(8)? != 0,
                        is_enabled: row.get::<_, i32>(9)? != 0,
                        last_fetched_at: row.get(10)?,
                        etag: row.get(11)?,
                        created_at: row.get(12)?,
                        updated_at: row.get(13)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?
    };

    // Perform async fetch (no db access)
    let items = repo_sync::fetch_repo_items(&repo)
        .await
        .map_err(|e| e.to_string())?;

    // Save items to database
    let db_guard = db.lock().map_err(|e| e.to_string())?;
    repo_sync::save_repo_items(&db_guard, id, &items).map_err(|e| e.to_string())
}

/// Sync all enabled repositories
#[tauri::command]
pub async fn sync_all_repos(db: State<'_, Arc<Mutex<Database>>>) -> Result<SyncResult, String> {
    let repos = {
        let db = db.lock().map_err(|e| e.to_string())?;
        repo_sync::get_all_repos(&db)
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|r| r.is_enabled)
            .collect::<Vec<_>>()
    };

    let mut total_added = 0;
    let mut total_updated = 0;
    let mut total_removed = 0;
    let mut all_errors = Vec::new();

    for repo in repos {
        // Fetch items (async, no db access)
        match repo_sync::fetch_repo_items(&repo).await {
            Ok(items) => {
                // Save to database
                let db_guard = db.lock().map_err(|e| e.to_string())?;
                match repo_sync::save_repo_items(&db_guard, repo.id, &items) {
                    Ok(result) => {
                        total_added += result.added;
                        total_updated += result.updated;
                        total_removed += result.removed;
                        all_errors.extend(result.errors);
                    }
                    Err(e) => {
                        all_errors.push(format!("{}: {}", repo.name, e));
                    }
                }
            }
            Err(e) => {
                all_errors.push(format!("{}: {}", repo.name, e));
            }
        }
    }

    Ok(SyncResult {
        added: total_added,
        updated: total_updated,
        removed: total_removed,
        errors: all_errors,
    })
}

/// Fetch raw content from a GitHub URL (converts blob URLs to raw URLs)
async fn fetch_content_from_url(url: &str) -> Result<String, String> {
    // Convert GitHub blob URLs to raw URLs
    // https://github.com/owner/repo/blob/main/path -> https://raw.githubusercontent.com/owner/repo/main/path
    let raw_url = if url.contains("github.com") && url.contains("/blob/") {
        url.replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/")
    } else {
        url.to_string()
    };

    let client = reqwest::Client::new();
    let response = client
        .get(&raw_url)
        .header("User-Agent", "claude-code-tool-manager/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch content: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch content: HTTP {}",
            response.status()
        ));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read content: {}", e))
}

/// Import a repository item to the local library
#[tauri::command]
pub async fn import_repo_item(
    db: State<'_, Arc<Mutex<Database>>>,
    item_id: i64,
) -> Result<ImportResult, String> {
    // Get the repo item (scope the lock)
    let item: RepoItem = {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.conn()
            .query_row(
                r#"SELECT id, repo_id, item_type, name, description, source_url, raw_content,
                          file_path, metadata, stars, is_imported, imported_item_id, created_at, updated_at
                   FROM repo_items WHERE id = ?"#,
                params![item_id],
                |row| {
                    Ok(RepoItem {
                        id: row.get(0)?,
                        repo_id: row.get(1)?,
                        item_type: row.get(2)?,
                        name: row.get(3)?,
                        description: row.get(4)?,
                        source_url: row.get(5)?,
                        raw_content: row.get(6)?,
                        file_path: row.get(7)?,
                        metadata: row.get(8)?,
                        stars: row.get(9)?,
                        is_imported: row.get::<_, i32>(10)? != 0,
                        imported_item_id: row.get(11)?,
                        created_at: row.get(12)?,
                        updated_at: row.get(13)?,
                    })
                },
            )
            .map_err(|e| e.to_string())?
    };

    if item.is_imported {
        return Ok(ImportResult {
            success: false,
            item_type: item.item_type,
            item_id: item.imported_item_id.unwrap_or(0),
            message: Some("Item already imported".to_string()),
        });
    }

    // If raw_content is empty but we have a source_url, fetch the content
    let raw_content = match item.raw_content {
        Some(content) if !content.trim().is_empty() => content,
        _ => {
            if let Some(ref url) = item.source_url {
                fetch_content_from_url(url).await?
            } else {
                return Err("No content available for this item".to_string());
            }
        }
    };

    // Re-acquire lock for database operations
    let db = db.lock().map_err(|e| e.to_string())?;

    // Import based on item type
    let imported_id = match item.item_type.as_str() {
        "skill" => {
            // Parse frontmatter to extract body and allowed_tools
            let (frontmatter, body) = parse_frontmatter(&raw_content);
            let content = body.trim().to_string();

            // Determine skill type and allowed tools
            let allowed_tools = frontmatter
                .get("allowed-tools")
                .or_else(|| frontmatter.get("allowedtools"))
                .cloned();
            let skill_type = if allowed_tools.is_some() {
                "skill"
            } else {
                "command"
            };

            db.conn()
                .execute(
                    r#"INSERT INTO skills (name, description, content, skill_type, allowed_tools, source)
                       VALUES (?, ?, ?, ?, ?, 'imported')"#,
                    params![item.name, item.description, content, skill_type, allowed_tools],
                )
                .map_err(|e| e.to_string())?;
            db.conn().last_insert_rowid()
        }
        "subagent" => {
            // Parse frontmatter to extract body content
            let (frontmatter, body) = parse_frontmatter(&raw_content);
            let content = body.trim().to_string();

            // Extract all fields from frontmatter
            let model = frontmatter.get("model").cloned();
            let permission_mode = frontmatter
                .get("permissionmode")
                .or_else(|| frontmatter.get("permission-mode"))
                .cloned();
            let tools = frontmatter.get("tools").map(|t| {
                t.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            });
            let skills = frontmatter.get("skills").map(|t| {
                t.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            });

            let tools_json = tools.as_ref().map(|t| serde_json::to_string(t).unwrap());
            let skills_json = skills.as_ref().map(|t| serde_json::to_string(t).unwrap());

            let description = item
                .description
                .unwrap_or_else(|| "Imported from marketplace".to_string());
            db.conn()
                .execute(
                    r#"INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, source)
                       VALUES (?, ?, ?, ?, ?, ?, ?, 'imported')"#,
                    params![item.name, description, content, tools_json, model, permission_mode, skills_json],
                )
                .map_err(|e| e.to_string())?;
            db.conn().last_insert_rowid()
        }
        "mcp" => {
            // For MCPs from README, we just store the reference
            // User would need to configure the MCP manually
            db.conn()
                .execute(
                    r#"INSERT INTO mcps (name, description, type, source, source_path)
                       VALUES (?, ?, 'stdio', 'imported', ?)"#,
                    params![item.name, item.description, item.source_url],
                )
                .map_err(|e| e.to_string())?;
            db.conn().last_insert_rowid()
        }
        _ => return Err("Unknown item type".to_string()),
    };

    // Mark as imported
    db.conn()
        .execute(
            "UPDATE repo_items SET is_imported = 1, imported_item_id = ? WHERE id = ?",
            params![imported_id, item_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(ImportResult {
        success: true,
        item_type: item.item_type,
        item_id: imported_id,
        message: Some("Successfully imported".to_string()),
    })
}

/// Get GitHub API rate limit information
#[tauri::command]
pub async fn get_github_rate_limit() -> Result<RateLimitInfo, String> {
    let client = GitHubClient::new(None);

    let (limit, remaining, reset) = client.get_rate_limit().await.map_err(|e| e.to_string())?;

    let reset_at = chrono::DateTime::from_timestamp(reset, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default();

    Ok(RateLimitInfo {
        limit,
        remaining,
        reset_at,
    })
}

/// Seed default repositories on first run
#[tauri::command]
pub fn seed_default_repos(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    repo_sync::seed_default_repos(&db).map_err(|e| e.to_string())
}

/// Reset repos to defaults (removes all repos and items, then re-seeds)
#[tauri::command]
pub fn reset_repos_to_defaults(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Delete all repo items
    db.conn()
        .execute("DELETE FROM repo_items", [])
        .map_err(|e| e.to_string())?;

    // Delete all repos
    db.conn()
        .execute("DELETE FROM repos", [])
        .map_err(|e| e.to_string())?;

    // Re-seed defaults
    repo_sync::seed_default_repos(&db).map_err(|e| e.to_string())
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Add a repo directly in the database (for testing)
pub fn add_repo_in_db(db: &Database, request: &CreateRepoRequest) -> Result<Repo, String> {
    let (owner, repo) =
        parse_github_url(&request.github_url).ok_or_else(|| "Invalid GitHub URL".to_string())?;

    let name = format!("{}/{}", owner, repo);

    db.conn()
        .execute(
            r#"INSERT INTO repos (name, owner, repo, repo_type, content_type, github_url)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            params![
                name,
                owner,
                repo,
                request.repo_type,
                request.content_type,
                request.github_url
            ],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_repo_by_id(db, id)
}

/// Get a repo by ID directly from the database (for testing)
pub fn get_repo_by_id(db: &Database, id: i64) -> Result<Repo, String> {
    db.conn()
        .query_row(
            r#"SELECT id, name, owner, repo, repo_type, content_type, github_url, description,
                      is_default, is_enabled, last_fetched_at, etag, created_at, updated_at
               FROM repos WHERE id = ?"#,
            params![id],
            |row| {
                Ok(Repo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    owner: row.get(2)?,
                    repo: row.get(3)?,
                    repo_type: row.get(4)?,
                    content_type: row.get(5)?,
                    github_url: row.get(6)?,
                    description: row.get(7)?,
                    is_default: row.get::<_, i32>(8)? != 0,
                    is_enabled: row.get::<_, i32>(9)? != 0,
                    last_fetched_at: row.get(10)?,
                    etag: row.get(11)?,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        )
        .map_err(|e| e.to_string())
}

/// Toggle a repo directly in the database (for testing)
pub fn toggle_repo_in_db(db: &Database, id: i64, enabled: bool) -> Result<(), String> {
    db.conn()
        .execute(
            "UPDATE repos SET is_enabled = ?, updated_at = ? WHERE id = ?",
            params![enabled as i32, Utc::now().to_rfc3339(), id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Remove a repo directly in the database (for testing)
pub fn remove_repo_in_db(db: &Database, id: i64) -> Result<(), String> {
    // Check if it's a default repo
    let is_default: bool = db
        .conn()
        .query_row(
            "SELECT is_default FROM repos WHERE id = ?",
            params![id],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        )
        .map_err(|e| e.to_string())?;

    if is_default {
        return Err(
            "Cannot remove default repositories. You can disable them instead.".to_string(),
        );
    }

    db.conn()
        .execute("DELETE FROM repos WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Add a repo item directly in the database (for testing)
pub fn add_repo_item_in_db(
    db: &Database,
    repo_id: i64,
    item_type: &str,
    name: &str,
    description: Option<&str>,
    source_url: Option<&str>,
    raw_content: Option<&str>,
) -> Result<i64, String> {
    db.conn()
        .execute(
            r#"INSERT INTO repo_items (repo_id, item_type, name, description, source_url, raw_content)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            params![repo_id, item_type, name, description, source_url, raw_content],
        )
        .map_err(|e| e.to_string())?;

    Ok(db.conn().last_insert_rowid())
}

/// Get a repo item by ID directly from the database (for testing)
pub fn get_repo_item_by_id(db: &Database, id: i64) -> Result<RepoItem, String> {
    db.conn()
        .query_row(
            r#"SELECT id, repo_id, item_type, name, description, source_url, raw_content,
                      file_path, metadata, stars, is_imported, imported_item_id, created_at, updated_at
               FROM repo_items WHERE id = ?"#,
            params![id],
            |row| {
                Ok(RepoItem {
                    id: row.get(0)?,
                    repo_id: row.get(1)?,
                    item_type: row.get(2)?,
                    name: row.get(3)?,
                    description: row.get(4)?,
                    source_url: row.get(5)?,
                    raw_content: row.get(6)?,
                    file_path: row.get(7)?,
                    metadata: row.get(8)?,
                    stars: row.get(9)?,
                    is_imported: row.get::<_, i32>(10)? != 0,
                    imported_item_id: row.get(11)?,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        )
        .map_err(|e| e.to_string())
}

/// Mark a repo item as imported directly in the database (for testing)
pub fn mark_item_imported_in_db(
    db: &Database,
    item_id: i64,
    imported_id: i64,
) -> Result<(), String> {
    db.conn()
        .execute(
            "UPDATE repo_items SET is_imported = 1, imported_item_id = ? WHERE id = ?",
            params![imported_id, item_id],
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Convert GitHub blob URL to raw URL (for testing)
pub(crate) fn convert_to_raw_url(url: &str) -> String {
    if url.contains("github.com") && url.contains("/blob/") {
        url.replace("github.com", "raw.githubusercontent.com")
            .replace("/blob/", "/")
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI64, Ordering};

    static TEST_REPO_COUNTER: AtomicI64 = AtomicI64::new(1000);

    fn unique_github_url() -> String {
        let id = TEST_REPO_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("https://github.com/testowner/testrepo{}", id)
    }

    // =========================================================================
    // Repo CRUD tests
    // =========================================================================

    #[test]
    fn test_add_repo() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: unique_github_url(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
        };

        let created = add_repo_in_db(&db, &request).unwrap();

        assert!(created.name.contains("testowner"));
        assert!(created.name.contains("testrepo"));
        assert_eq!(created.repo_type, "file_based");
        assert_eq!(created.content_type, "skill");
        assert!(created.is_enabled); // Default enabled
        assert!(!created.is_default); // Not a default repo
    }

    #[test]
    fn test_add_repo_with_standard_github_url() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: "https://github.com/anthropics/claude-code".to_string(),
            repo_type: "readme_based".to_string(),
            content_type: "mcp".to_string(),
        };

        let created = add_repo_in_db(&db, &request).unwrap();

        assert_eq!(created.owner, "anthropics");
        assert_eq!(created.repo, "claude-code");
        assert_eq!(created.name, "anthropics/claude-code");
    }

    #[test]
    fn test_add_repo_invalid_url() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: "not-a-valid-url".to_string(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
        };

        let result = add_repo_in_db(&db, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid GitHub URL"));
    }

    #[test]
    fn test_get_repo_by_id() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: unique_github_url(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
        };

        let created = add_repo_in_db(&db, &request).unwrap();
        let found = get_repo_by_id(&db, created.id).unwrap();

        assert_eq!(found.id, created.id);
        assert_eq!(found.name, created.name);
    }

    #[test]
    fn test_get_repo_by_id_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_repo_by_id(&db, 9999);

        assert!(result.is_err());
    }

    #[test]
    fn test_toggle_repo() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: unique_github_url(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
        };

        let created = add_repo_in_db(&db, &request).unwrap();
        assert!(created.is_enabled);

        // Disable
        toggle_repo_in_db(&db, created.id, false).unwrap();
        let repo = get_repo_by_id(&db, created.id).unwrap();
        assert!(!repo.is_enabled);

        // Re-enable
        toggle_repo_in_db(&db, created.id, true).unwrap();
        let repo = get_repo_by_id(&db, created.id).unwrap();
        assert!(repo.is_enabled);
    }

    #[test]
    fn test_remove_repo() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: unique_github_url(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
        };

        let created = add_repo_in_db(&db, &request).unwrap();
        remove_repo_in_db(&db, created.id).unwrap();

        let result = get_repo_by_id(&db, created.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_remove_default_repo() {
        let db = Database::in_memory().unwrap();

        // Create a default repo directly
        db.conn()
            .execute(
                r#"INSERT INTO repos (name, owner, repo, repo_type, content_type, github_url, is_default)
                   VALUES ('default/repo', 'default', 'repo', 'file_based', 'skill', 'https://github.com/default/repo', 1)"#,
                [],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let result = remove_repo_in_db(&db, id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot remove default"));
    }

    // =========================================================================
    // Repo Item tests
    // =========================================================================

    fn create_test_repo(db: &Database) -> i64 {
        let request = CreateRepoRequest {
            github_url: unique_github_url(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
        };
        add_repo_in_db(db, &request).unwrap().id
    }

    #[test]
    fn test_add_repo_item() {
        let db = Database::in_memory().unwrap();
        let repo_id = create_test_repo(&db);

        let item_id = add_repo_item_in_db(
            &db,
            repo_id,
            "skill",
            "test-skill",
            Some("A test skill"),
            Some("https://github.com/test/repo/blob/main/skill.md"),
            Some("# Test Skill\n\nContent here"),
        )
        .unwrap();

        let item = get_repo_item_by_id(&db, item_id).unwrap();

        assert_eq!(item.repo_id, repo_id);
        assert_eq!(item.item_type, "skill");
        assert_eq!(item.name, "test-skill");
        assert_eq!(item.description, Some("A test skill".to_string()));
        assert!(!item.is_imported);
    }

    #[test]
    fn test_add_repo_item_minimal() {
        let db = Database::in_memory().unwrap();
        let repo_id = create_test_repo(&db);

        let item_id =
            add_repo_item_in_db(&db, repo_id, "mcp", "minimal-mcp", None, None, None).unwrap();

        let item = get_repo_item_by_id(&db, item_id).unwrap();

        assert_eq!(item.name, "minimal-mcp");
        assert!(item.description.is_none());
        assert!(item.source_url.is_none());
        assert!(item.raw_content.is_none());
    }

    #[test]
    fn test_mark_item_imported() {
        let db = Database::in_memory().unwrap();
        let repo_id = create_test_repo(&db);

        let item_id =
            add_repo_item_in_db(&db, repo_id, "skill", "importable", None, None, None).unwrap();

        // Initially not imported
        let item = get_repo_item_by_id(&db, item_id).unwrap();
        assert!(!item.is_imported);
        assert!(item.imported_item_id.is_none());

        // Mark as imported
        mark_item_imported_in_db(&db, item_id, 42).unwrap();

        let item = get_repo_item_by_id(&db, item_id).unwrap();
        assert!(item.is_imported);
        assert_eq!(item.imported_item_id, Some(42));
    }

    // =========================================================================
    // URL conversion tests
    // =========================================================================

    #[test]
    fn test_convert_blob_url_to_raw() {
        let blob_url = "https://github.com/owner/repo/blob/main/path/to/file.md";
        let raw = convert_to_raw_url(blob_url);
        assert_eq!(
            raw,
            "https://raw.githubusercontent.com/owner/repo/main/path/to/file.md"
        );
    }

    #[test]
    fn test_convert_blob_url_with_branch() {
        let blob_url = "https://github.com/owner/repo/blob/feature-branch/file.md";
        let raw = convert_to_raw_url(blob_url);
        assert_eq!(
            raw,
            "https://raw.githubusercontent.com/owner/repo/feature-branch/file.md"
        );
    }

    #[test]
    fn test_convert_non_blob_url_unchanged() {
        let url = "https://raw.githubusercontent.com/owner/repo/main/file.md";
        let result = convert_to_raw_url(url);
        assert_eq!(result, url);
    }

    #[test]
    fn test_convert_non_github_url_unchanged() {
        let url = "https://example.com/file.md";
        let result = convert_to_raw_url(url);
        assert_eq!(result, url);
    }

    // =========================================================================
    // GitHub URL parsing tests
    // =========================================================================

    #[test]
    fn test_parse_github_url_standard() {
        let result = parse_github_url("https://github.com/owner/repo");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_with_trailing_slash() {
        let result = parse_github_url("https://github.com/owner/repo/");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_with_path() {
        let result = parse_github_url("https://github.com/owner/repo/tree/main/path");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_invalid() {
        assert!(parse_github_url("not-a-url").is_none());
        assert!(parse_github_url("https://gitlab.com/owner/repo").is_none());
        assert!(parse_github_url("https://github.com/owner").is_none());
    }

    // =========================================================================
    // Repo type tests
    // =========================================================================

    #[test]
    fn test_add_readme_based_repo() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: unique_github_url(),
            repo_type: "readme_based".to_string(),
            content_type: "mcp".to_string(),
        };

        let created = add_repo_in_db(&db, &request).unwrap();

        assert_eq!(created.repo_type, "readme_based");
        assert_eq!(created.content_type, "mcp");
    }

    #[test]
    fn test_add_subagent_repo() {
        let db = Database::in_memory().unwrap();

        let request = CreateRepoRequest {
            github_url: unique_github_url(),
            repo_type: "file_based".to_string(),
            content_type: "subagent".to_string(),
        };

        let created = add_repo_in_db(&db, &request).unwrap();

        assert_eq!(created.content_type, "subagent");
    }

    // =========================================================================
    // Edge case tests
    // =========================================================================

    #[test]
    fn test_repo_items_deleted_with_repo() {
        let db = Database::in_memory().unwrap();
        let repo_id = create_test_repo(&db);

        // Add some items
        add_repo_item_in_db(&db, repo_id, "skill", "skill1", None, None, None).unwrap();
        add_repo_item_in_db(&db, repo_id, "skill", "skill2", None, None, None).unwrap();

        // Verify items exist
        let items = repo_sync::get_repo_items(&db, repo_id).unwrap();
        assert_eq!(items.len(), 2);

        // Delete the repo
        remove_repo_in_db(&db, repo_id).unwrap();

        // Items should be deleted via cascade
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM repo_items WHERE repo_id = ?",
                [repo_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}
