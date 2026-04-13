use crate::db::Database;
use crate::services::gist_sync::{
    self, GistSyncService, ProjectMapping, SyncAuthStatus, SyncConfig, SyncItemCounts, SyncMeta,
    SyncResult, SyncStatus,
};
use log::info;
use std::sync::{Arc, Mutex};
use tauri::State;

// ─── Authentication ─────────────────────────────────────────────────────────

/// Resolve the `gh` binary path.
///
/// macOS GUI apps inherit a minimal PATH that excludes Homebrew directories.
/// This checks common install locations before falling back to bare `gh`.
fn find_gh() -> std::path::PathBuf {
    for candidate in &[
        "/opt/homebrew/bin/gh", // Homebrew on Apple Silicon
        "/usr/local/bin/gh",   // Homebrew on Intel / manual install
        "/usr/bin/gh",         // System install
    ] {
        if std::path::Path::new(candidate).exists() {
            return candidate.into();
        }
    }
    "gh".into()
}

/// Get a GitHub token from the gh CLI
#[tauri::command]
pub async fn get_gh_cli_token() -> Result<String, String> {
    info!("[CloudSync] Getting token from gh CLI");
    tokio::task::spawn_blocking(|| {
        let output = std::process::Command::new(find_gh())
            .args(["auth", "token"])
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    "gh CLI not installed. Install it from https://cli.github.com".to_string()
                } else {
                    format!("Failed to run gh CLI: {}", e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "gh CLI not authenticated. Run 'gh auth login' first. Error: {}",
                stderr.trim()
            ));
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            return Err("gh CLI returned empty token".to_string());
        }

        Ok(token)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Check if gh CLI is installed
#[tauri::command]
pub fn has_gh_cli() -> bool {
    std::process::Command::new(find_gh())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Connect via gh CLI — gets token, validates it, finds/creates gist
#[tauri::command]
pub async fn connect_cloud_sync(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<SyncAuthStatus, String> {
    info!("[CloudSync] Connecting via gh CLI");

    // Get token from gh CLI
    let token = get_gh_cli_token().await?;

    let service = GistSyncService::new();

    // Validate token and get username
    let username = service
        .get_authenticated_user(&token)
        .await
        .map_err(|e| format!("Failed to validate token: {}", e))?;

    // Find or create sync gist
    let (gist_id, gist_url) = service
        .find_or_create_gist(&token)
        .await
        .map_err(|e| format!("Failed to find/create sync gist: {}", e))?;

    // Store in DB
    {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.set_setting("github_token", &token)
            .map_err(|e| e.to_string())?;
        db.set_setting("sync_gist_id", &gist_id)
            .map_err(|e| e.to_string())?;
        db.set_setting("sync_username", &username)
            .map_err(|e| e.to_string())?;
        db.set_setting("sync_gist_url", &gist_url)
            .map_err(|e| e.to_string())?;
    }

    info!(
        "[CloudSync] Connected as {} with gist {}",
        username, gist_id
    );

    Ok(SyncAuthStatus {
        is_authenticated: true,
        username: Some(username),
        has_gh_cli: true,
        gist_id: Some(gist_id),
        gist_url: Some(gist_url),
    })
}

/// Get current sync auth status
#[tauri::command]
pub fn get_sync_auth_status(db: State<'_, Arc<Mutex<Database>>>) -> Result<SyncAuthStatus, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let token = db.get_setting("github_token");
    let username = db.get_setting("sync_username");
    let gist_id = db.get_setting("sync_gist_id");
    let gist_url = db.get_setting("sync_gist_url");
    let has_gh = has_gh_cli();

    Ok(SyncAuthStatus {
        is_authenticated: token.is_some() && gist_id.is_some(),
        username,
        has_gh_cli: has_gh,
        gist_id,
        gist_url,
    })
}

/// Disconnect cloud sync — clear stored credentials
#[tauri::command]
pub fn disconnect_cloud_sync(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    info!("[CloudSync] Disconnecting");
    let db = db.lock().map_err(|e| e.to_string())?;
    // Clear sync-specific settings but leave github_token for other uses
    let _ = db.set_setting("sync_gist_id", "");
    let _ = db.set_setting("sync_username", "");
    let _ = db.set_setting("sync_gist_url", "");
    let _ = db.set_setting("sync_last_pushed_at", "");
    let _ = db.set_setting("sync_last_pulled_at", "");
    Ok(())
}

// ─── Sync Config ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_sync_config(db: State<'_, Arc<Mutex<Database>>>) -> Result<SyncConfig, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let json = db.get_setting("sync_config").unwrap_or_default();
    if json.is_empty() {
        return Ok(SyncConfig::default());
    }
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_sync_config(
    db: State<'_, Arc<Mutex<Database>>>,
    config: SyncConfig,
) -> Result<(), String> {
    info!("[CloudSync] Saving sync config");
    let db = db.lock().map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    db.set_setting("sync_config", &json)
        .map_err(|e| e.to_string())
}

// ─── Project Mappings ───────────────────────────────────────────────────────

#[tauri::command]
pub fn get_project_mappings(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<ProjectMapping>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let json = db.get_setting("sync_project_mappings").unwrap_or_default();
    if json.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_project_mappings(
    db: State<'_, Arc<Mutex<Database>>>,
    mappings: Vec<ProjectMapping>,
) -> Result<(), String> {
    info!("[CloudSync] Saving {} project mappings", mappings.len());
    let db = db.lock().map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&mappings).map_err(|e| e.to_string())?;
    db.set_setting("sync_project_mappings", &json)
        .map_err(|e| e.to_string())
}

// ─── Push / Pull ────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn push_sync(db: State<'_, Arc<Mutex<Database>>>) -> Result<SyncResult, String> {
    info!("[CloudSync] Pushing to cloud");

    let (token, gist_id, config, mappings) = {
        let db = db.lock().map_err(|e| e.to_string())?;
        let token = db.get_setting("github_token").ok_or("Not authenticated")?;
        let gist_id = db
            .get_setting("sync_gist_id")
            .ok_or("No sync gist configured")?;
        let config_json = db.get_setting("sync_config").unwrap_or_default();
        let config: SyncConfig = if config_json.is_empty() {
            SyncConfig::default()
        } else {
            serde_json::from_str(&config_json).map_err(|e| e.to_string())?
        };
        let mappings_json = db.get_setting("sync_project_mappings").unwrap_or_default();
        let mappings: Vec<ProjectMapping> = if mappings_json.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&mappings_json).map_err(|e| e.to_string())?
        };
        (token, gist_id, config, mappings)
    };

    // Build payload from local state
    let payload = {
        let db = db.lock().map_err(|e| e.to_string())?;
        gist_sync::build_local_payload(&db, &config).map_err(|e| e.to_string())?
    };

    let meta = SyncMeta {
        machine_id: get_machine_id(),
        last_synced_at: chrono::Utc::now().to_rfc3339(),
        project_mappings: mappings,
        schema_version: 1,
    };

    let service = GistSyncService::new();
    let result = service
        .push(&token, &gist_id, &payload, &meta)
        .await
        .map_err(|e| e.to_string())?;

    // Record push time
    {
        let db = db.lock().map_err(|e| e.to_string())?;
        let _ = db.set_setting("sync_last_pushed_at", &result.synced_at);
    }

    info!("[CloudSync] Push complete: {:?}", result.pushed);
    Ok(result)
}

#[tauri::command]
pub async fn pull_sync(db: State<'_, Arc<Mutex<Database>>>) -> Result<SyncResult, String> {
    info!("[CloudSync] Pulling from cloud");

    let (token, gist_id, config, mappings) = {
        let db = db.lock().map_err(|e| e.to_string())?;
        let token = db.get_setting("github_token").ok_or("Not authenticated")?;
        let gist_id = db
            .get_setting("sync_gist_id")
            .ok_or("No sync gist configured")?;
        let config_json = db.get_setting("sync_config").unwrap_or_default();
        let config: SyncConfig = if config_json.is_empty() {
            SyncConfig::default()
        } else {
            serde_json::from_str(&config_json).map_err(|e| e.to_string())?
        };
        let mappings_json = db.get_setting("sync_project_mappings").unwrap_or_default();
        let mappings: Vec<ProjectMapping> = if mappings_json.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&mappings_json).map_err(|e| e.to_string())?
        };
        (token, gist_id, config, mappings)
    };

    let service = GistSyncService::new();
    let (payload, _remote_meta) = service
        .pull(&token, &gist_id)
        .await
        .map_err(|e| e.to_string())?;

    // Apply to local state
    let mut result = {
        let db = db.lock().map_err(|e| e.to_string())?;
        gist_sync::apply_pulled_payload(&db, &payload, &config, &mappings)
            .map_err(|e| e.to_string())?
    };

    // Fill in gist URL
    {
        let db = db.lock().map_err(|e| e.to_string())?;
        result.gist_url = db.get_setting("sync_gist_url").unwrap_or_default();
        let _ = db.set_setting("sync_last_pulled_at", &result.synced_at);
    }

    info!(
        "[CloudSync] Pull complete: {:?}, conflicts: {:?}",
        result.pulled, result.conflicts
    );
    Ok(result)
}

// ─── Status ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_sync_status(db: State<'_, Arc<Mutex<Database>>>) -> Result<SyncStatus, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    let config_json = db.get_setting("sync_config").unwrap_or_default();
    let config: SyncConfig = if config_json.is_empty() {
        SyncConfig::default()
    } else {
        serde_json::from_str(&config_json).unwrap_or_default()
    };

    let mcp_count = if config.sync_mcps {
        db.get_all_mcps()
            .map(|m| m.iter().filter(|m| m.source != "system").count())
            .unwrap_or(0)
    } else {
        0
    };

    let skill_count = if config.sync_skills {
        db.get_all_skills()
            .map(|s| s.iter().filter(|s| s.source != "system").count())
            .unwrap_or(0)
    } else {
        0
    };

    Ok(SyncStatus {
        last_pushed_at: db
            .get_setting("sync_last_pushed_at")
            .filter(|s| !s.is_empty()),
        last_pulled_at: db
            .get_setting("sync_last_pulled_at")
            .filter(|s| !s.is_empty()),
        gist_id: db.get_setting("sync_gist_id").filter(|s| !s.is_empty()),
        gist_url: db.get_setting("sync_gist_url").filter(|s| !s.is_empty()),
        item_counts: SyncItemCounts {
            mcps: mcp_count,
            skills: skill_count,
            projects: config.sync_project_claude_mds.len(),
            has_global_claude_md: config.sync_global_claude_md,
        },
    })
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn get_machine_id() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
