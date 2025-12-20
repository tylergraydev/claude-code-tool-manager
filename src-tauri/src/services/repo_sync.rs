use crate::db::{Database, Repo, RepoItem, SyncResult};
use crate::services::github_client::{parse_github_url, GitHubClient};
use crate::services::repo_parser::{
    detect_item_type, parse_readme_for_mcps, parse_readme_for_skills, parse_skill_file,
    parse_subagent_file, should_skip_file, ParsedItem,
};
use anyhow::Result;
use chrono::Utc;
use rusqlite::params;

/// Default repositories to seed on first run
pub const DEFAULT_REPOS: &[(&str, &str, &str, &str, &str)] = &[
    // (name, github_url, repo_type, content_type, description)
    (
        "Claude Code Commands",
        "https://github.com/wshobson/commands",
        "file_based",
        "skill",
        "Production-ready slash commands for Claude Code",
    ),
    (
        "Awesome Claude Code",
        "https://github.com/hesreallyhim/awesome-claude-code",
        "readme_based",
        "skill",
        "Curated list of Claude Code resources",
    ),
];

/// Seed default repositories if none exist
pub fn seed_default_repos(db: &Database) -> Result<()> {
    let count: i64 = db.conn().query_row("SELECT COUNT(*) FROM repos", [], |row| row.get(0))?;

    if count > 0 {
        return Ok(());
    }

    for (name, github_url, repo_type, content_type, description) in DEFAULT_REPOS {
        if let Some((owner, repo)) = parse_github_url(github_url) {
            db.conn().execute(
                r#"INSERT INTO repos (name, owner, repo, repo_type, content_type, github_url, description, is_default)
                   VALUES (?, ?, ?, ?, ?, ?, ?, 1)"#,
                params![name, owner, repo, repo_type, content_type, github_url, description],
            )?;
        }
    }

    Ok(())
}

/// Fetch items from a repository (async, no database access)
pub async fn fetch_repo_items(repo: &Repo) -> Result<Vec<ParsedItem>> {
    let client = GitHubClient::new(None); // TODO: support tokens

    match repo.repo_type.as_str() {
        "file_based" => sync_file_based_repo(&client, repo).await,
        "readme_based" => sync_readme_based_repo(&client, repo).await,
        _ => Ok(Vec::new()),
    }
}

/// Update database with fetched items (sync, requires database)
pub fn save_repo_items(db: &Database, repo_id: i64, items: &[ParsedItem]) -> Result<SyncResult> {
    let result = update_repo_items(db, repo_id, items)?;

    // Update last_fetched_at
    db.conn().execute(
        "UPDATE repos SET last_fetched_at = ?, updated_at = ? WHERE id = ?",
        params![Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), repo_id],
    )?;

    Ok(result)
}

/// Sync a file-based repository (scans for .md files)
async fn sync_file_based_repo(client: &GitHubClient, repo: &Repo) -> Result<Vec<ParsedItem>> {
    let mut items = Vec::new();

    // Determine which directories to scan based on content type
    // Include common directory names that repos might use
    let dirs_to_scan: Vec<&str> = match repo.content_type.as_str() {
        "skill" => vec!["", "commands", "skills", "workflows", "tools", "examples", "prompts"],
        "subagent" => vec!["", "agents", "subagents"],
        "mcp" => vec!["", "src"],
        "mixed" => vec!["", "commands", "skills", "workflows", "tools", "examples", "prompts", "agents", "subagents"],
        _ => vec![""],
    };

    if let Ok(files) = client.get_markdown_files(&repo.owner, &repo.repo, &dirs_to_scan).await {
        for (path, content) in files {
            // Skip junk files like README.md, CONTRIBUTING.md, etc.
            if should_skip_file(&path) {
                continue;
            }

            let item_type = detect_item_type(&path, &content);

            let parsed = match item_type.as_str() {
                "subagent" => parse_subagent_file(&content, &path),
                _ => parse_skill_file(&content, &path),
            };

            if let Some(mut item) = parsed {
                item.item_type = item_type;
                // Build GitHub URL for the file
                item.source_url = Some(format!(
                    "https://github.com/{}/{}/blob/main/{}",
                    repo.owner, repo.repo, path
                ));
                items.push(item);
            }
        }
    }

    Ok(items)
}

/// Sync a README-based repository (parses README for links)
async fn sync_readme_based_repo(client: &GitHubClient, repo: &Repo) -> Result<Vec<ParsedItem>> {
    let readme = client.get_readme(&repo.owner, &repo.repo).await?;

    let mut items = match repo.content_type.as_str() {
        "mcp" => parse_readme_for_mcps(&readme),
        "skill" => parse_readme_for_skills(&readme),
        "mixed" => {
            let mut all = parse_readme_for_mcps(&readme);
            all.extend(parse_readme_for_skills(&readme));
            all
        }
        _ => Vec::new(),
    };

    // Fix relative URLs to be proper GitHub URLs
    for item in &mut items {
        if let Some(ref url) = item.source_url {
            // If it's a relative path (doesn't start with http), convert to GitHub URL
            if !url.starts_with("http") {
                let clean_path = url.trim_start_matches("./").trim_start_matches('/');
                item.source_url = Some(format!(
                    "https://github.com/{}/{}/blob/main/{}",
                    repo.owner, repo.repo, clean_path
                ));
            }
        }
    }

    Ok(items)
}

/// Update repository items in the database
fn update_repo_items(db: &Database, repo_id: i64, items: &[ParsedItem]) -> Result<SyncResult> {
    // Don't delete existing items if we got nothing new (likely a fetch error)
    if items.is_empty() {
        return Ok(SyncResult {
            added: 0,
            updated: 0,
            removed: 0,
            errors: vec!["No items fetched - keeping existing data".to_string()],
        });
    }

    // Delete all existing items for this repo (clean slate approach)
    // This ensures junk items that no longer pass filters are removed
    let removed = db.conn().execute(
        "DELETE FROM repo_items WHERE repo_id = ?",
        params![repo_id],
    )? as i32;

    let mut added = 0;

    // Insert all items fresh
    for item in items {
        db.conn().execute(
            r#"INSERT INTO repo_items (repo_id, item_type, name, description, source_url, raw_content, file_path, metadata)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
            params![
                repo_id,
                &item.item_type,
                &item.name,
                &item.description,
                &item.source_url,
                &item.raw_content,
                &item.file_path,
                &item.metadata
            ],
        )?;
        added += 1;
    }

    Ok(SyncResult {
        added,
        updated: 0,
        removed,
        errors: Vec::new(),
    })
}

/// Get all repos from database
pub fn get_all_repos(db: &Database) -> Result<Vec<Repo>> {
    let mut stmt = db.conn().prepare(
        r#"SELECT id, name, owner, repo, repo_type, content_type, github_url, description,
                  is_default, is_enabled, last_fetched_at, etag, created_at, updated_at
           FROM repos ORDER BY is_default DESC, name ASC"#,
    )?;

    let repos = stmt
        .query_map([], |row| {
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
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(repos)
}

/// Get items from a specific repo
pub fn get_repo_items(db: &Database, repo_id: i64) -> Result<Vec<RepoItem>> {
    let mut stmt = db.conn().prepare(
        r#"SELECT id, repo_id, item_type, name, description, source_url, raw_content,
                  file_path, metadata, stars, is_imported, imported_item_id, created_at, updated_at
           FROM repo_items WHERE repo_id = ? ORDER BY name ASC"#,
    )?;

    let items = stmt
        .query_map(params![repo_id], |row| {
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
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

/// Get all items from all repos, optionally filtered by type
pub fn get_all_repo_items(db: &Database, item_type: Option<String>) -> Result<Vec<RepoItem>> {
    let sql = if item_type.is_some() {
        r#"SELECT id, repo_id, item_type, name, description, source_url, raw_content,
                  file_path, metadata, stars, is_imported, imported_item_id, created_at, updated_at
           FROM repo_items WHERE item_type = ? ORDER BY name ASC"#
    } else {
        r#"SELECT id, repo_id, item_type, name, description, source_url, raw_content,
                  file_path, metadata, stars, is_imported, imported_item_id, created_at, updated_at
           FROM repo_items ORDER BY name ASC"#
    };

    let mut stmt = db.conn().prepare(sql)?;

    let items: Vec<RepoItem> = if let Some(ref t) = item_type {
        stmt.query_map(params![t], |row| {
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
        })?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        stmt.query_map([], |row| {
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
        })?
        .filter_map(|r| r.ok())
        .collect()
    };

    Ok(items)
}
