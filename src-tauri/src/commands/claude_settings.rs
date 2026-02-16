use crate::services::claude_settings::{self, AllClaudeSettings, ClaudeSettings};
use crate::services::permission_writer::PermissionScope;
use log::info;
use std::path::Path;

fn parse_scope(scope: &str) -> Result<PermissionScope, String> {
    match scope {
        "user" => Ok(PermissionScope::User),
        "project" => Ok(PermissionScope::Project),
        "local" => Ok(PermissionScope::Local),
        _ => Err(format!("Invalid scope: {}", scope)),
    }
}

/// Get all claude settings from all three scopes
#[tauri::command]
pub fn get_all_claude_settings(
    project_path: Option<String>,
) -> Result<AllClaudeSettings, String> {
    info!(
        "[ClaudeSettings] Reading all settings (project={:?})",
        project_path
    );
    let pp = project_path.as_deref().map(Path::new);
    claude_settings::read_all_claude_settings(pp).map_err(|e| e.to_string())
}

/// Get claude settings for a specific scope
#[tauri::command]
pub fn get_claude_settings(
    scope: String,
    project_path: Option<String>,
) -> Result<ClaudeSettings, String> {
    info!(
        "[ClaudeSettings] Reading settings for scope={} (project={:?})",
        scope, project_path
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    let path =
        crate::services::permission_writer::resolve_settings_path(&ps, pp)
            .map_err(|e| e.to_string())?;
    claude_settings::read_claude_settings_from_file(&path, &scope).map_err(|e| e.to_string())
}

/// Save claude settings for a specific scope
#[tauri::command]
pub fn save_claude_settings(
    scope: String,
    project_path: Option<String>,
    settings: ClaudeSettings,
) -> Result<ClaudeSettings, String> {
    info!(
        "[ClaudeSettings] Saving settings for scope={} (project={:?})",
        scope, project_path
    );
    let ps = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);

    claude_settings::write_claude_settings(&ps, pp, &settings).map_err(|e| e.to_string())?;

    // Re-read to return updated state
    let path =
        crate::services::permission_writer::resolve_settings_path(&ps, pp)
            .map_err(|e| e.to_string())?;
    claude_settings::read_claude_settings_from_file(&path, &scope).map_err(|e| e.to_string())
}
