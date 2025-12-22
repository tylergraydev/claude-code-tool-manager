use crate::db::{AppSettings, Database, EditorInfo, OpenCodePaths};
use crate::utils::opencode_paths::{get_opencode_paths, is_opencode_installed};
use crate::utils::paths::get_claude_paths;
use log::info;
use rusqlite::params;
use std::sync::Mutex;
use tauri::State;

/// Get application settings
#[tauri::command]
pub fn get_app_settings(db: State<'_, Mutex<Database>>) -> Result<AppSettings, String> {
    info!("[Settings] Getting app settings");
    let db = db.lock().map_err(|e| e.to_string())?;

    let default_editor = db
        .get_setting("default_editor")
        .unwrap_or_else(|| "claude_code".to_string());

    Ok(AppSettings { default_editor })
}

/// Update application settings
#[tauri::command]
pub fn update_app_settings(
    db: State<'_, Mutex<Database>>,
    settings: AppSettings,
) -> Result<(), String> {
    info!(
        "[Settings] Updating app settings: default_editor={}",
        settings.default_editor
    );
    let db = db.lock().map_err(|e| e.to_string())?;

    db.set_setting("default_editor", &settings.default_editor)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get info about available editors
#[tauri::command]
pub fn get_available_editors() -> Result<Vec<EditorInfo>, String> {
    info!("[Settings] Getting available editors");

    let mut editors = Vec::new();

    // Claude Code
    if let Ok(paths) = get_claude_paths() {
        editors.push(EditorInfo {
            id: "claude_code".to_string(),
            name: "Claude Code".to_string(),
            is_installed: paths.claude_dir.exists(),
            config_path: paths.claude_json.to_string_lossy().to_string(),
        });
    }

    // OpenCode
    if let Ok(paths) = get_opencode_paths() {
        editors.push(EditorInfo {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            is_installed: is_opencode_installed(),
            config_path: paths.config_file.to_string_lossy().to_string(),
        });
    }

    Ok(editors)
}

/// Get OpenCode paths
#[tauri::command]
pub fn get_opencode_paths_cmd() -> Result<OpenCodePaths, String> {
    info!("[Settings] Getting OpenCode paths");

    let paths = get_opencode_paths().map_err(|e| e.to_string())?;

    Ok(OpenCodePaths {
        config_dir: paths.config_dir.to_string_lossy().to_string(),
        config_file: paths.config_file.to_string_lossy().to_string(),
        command_dir: paths.command_dir.to_string_lossy().to_string(),
        agent_dir: paths.agent_dir.to_string_lossy().to_string(),
        plugin_dir: paths.plugin_dir.to_string_lossy().to_string(),
        tool_dir: paths.tool_dir.to_string_lossy().to_string(),
        knowledge_dir: paths.knowledge_dir.to_string_lossy().to_string(),
    })
}

/// Update project editor type
#[tauri::command]
pub fn update_project_editor_type(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    editor_type: String,
) -> Result<(), String> {
    info!(
        "[Settings] Updating project {} editor_type to {}",
        project_id, editor_type
    );

    // Validate editor type
    if editor_type != "claude_code" && editor_type != "opencode" {
        return Err(format!("Invalid editor type: {}", editor_type));
    }

    let db = db.lock().map_err(|e| e.to_string())?;

    db.conn()
        .execute(
            "UPDATE projects SET editor_type = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![editor_type, project_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}
