use crate::db::models::StatusLineGalleryEntry;
use crate::db::schema::Database;
use anyhow::Result;

/// Fetch gallery entries from a remote URL (JSON array)
pub async fn fetch_gallery_from_url(url: &str, github_token: Option<&str>) -> Result<Vec<StatusLineGalleryEntry>> {
    let client = reqwest::Client::new();
    let mut request = client.get(url);

    if let Some(token) = github_token {
        request = request.header("Authorization", format!("token {}", token));
    }

    request = request.header("User-Agent", "claude-code-tool-manager");

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch gallery: HTTP {}",
            response.status()
        ));
    }

    let entries: Vec<StatusLineGalleryEntry> = response.json().await?;
    Ok(entries)
}

/// Get the gallery URL from app_settings, falling back to default
pub fn get_gallery_url(db: &Database) -> String {
    db.get_setting("statusline_gallery_url")
        .unwrap_or_else(|| get_default_gallery_url())
}

/// Set the gallery URL in app_settings
pub fn set_gallery_url(db: &Database, url: &str) -> Result<()> {
    db.set_setting("statusline_gallery_url", url)
}

/// Default gallery URL
fn get_default_gallery_url() -> String {
    "https://raw.githubusercontent.com/nicholasgriffintn/awesome-claude-statuslines/main/statuslines.json".to_string()
}

/// Get cached gallery entries from app_settings
pub fn get_cached_gallery(db: &Database) -> Option<Vec<StatusLineGalleryEntry>> {
    db.get_setting("statusline_gallery_cache")
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Cache gallery entries in app_settings
pub fn cache_gallery(db: &Database, entries: &[StatusLineGalleryEntry]) -> Result<()> {
    let json = serde_json::to_string(entries)?;
    db.set_setting("statusline_gallery_cache", &json)
}

/// Get seed gallery entries (built-in defaults when no remote is available)
pub fn get_seed_gallery_entries() -> Vec<StatusLineGalleryEntry> {
    vec![
        StatusLineGalleryEntry {
            name: "claude-limitline".to_string(),
            description: Some("Powerline-style status line with API usage limits, git branch, model info, and 6 themes".to_string()),
            author: Some("tylergraydev".to_string()),
            homepage_url: Some("https://github.com/tylergraydev/claude-limitline".to_string()),
            install_command: Some("npm install -g claude-limitline".to_string()),
            run_command: Some("claude-limitline".to_string()),
            package_name: Some("claude-limitline".to_string()),
            icon: Some("terminal".to_string()),
            tags: Some(vec!["powerline".to_string(), "usage-limits".to_string(), "themes".to_string(), "git".to_string()]),
            preview_text: Some(" main  Opus 4.5  12% (3h20m)  45%".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_seed_gallery_entries_returns_valid_entries() {
        let entries = get_seed_gallery_entries();
        assert!(!entries.is_empty());
        for entry in &entries {
            assert!(!entry.name.is_empty());
            // All seed entries should have install and run commands
            assert!(entry.install_command.is_some());
            assert!(entry.run_command.is_some());
        }
    }

    #[test]
    fn test_get_seed_gallery_entries_have_metadata() {
        let entries = get_seed_gallery_entries();
        let first = &entries[0];
        assert!(first.description.is_some());
        assert!(first.author.is_some());
        assert!(first.tags.is_some());
    }

    #[test]
    fn test_get_default_gallery_url() {
        let url = get_default_gallery_url();
        assert!(url.starts_with("https://"));
        assert!(url.contains("github"));
    }
}
