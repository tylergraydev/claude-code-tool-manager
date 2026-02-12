use crate::db::{
    AppSettings, CodexPaths, CopilotPaths, CursorPaths, Database, EditorInfo, GeminiPaths,
    OpenCodePaths,
};
use crate::utils::codex_paths::{get_codex_paths, is_codex_installed};
use crate::utils::copilot_paths::{get_copilot_paths, is_copilot_installed};
use crate::utils::cursor_paths::{get_cursor_paths, is_cursor_installed};
use crate::utils::gemini_paths::{get_gemini_paths, is_gemini_installed};
use crate::utils::opencode_paths::{get_opencode_paths, is_opencode_installed};
use crate::utils::paths::get_claude_paths;
use log::info;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Get application settings
#[tauri::command]
pub fn get_app_settings(db: State<'_, Arc<Mutex<Database>>>) -> Result<AppSettings, String> {
    info!("[Settings] Getting app settings");
    let db = db.lock().map_err(|e| e.to_string())?;
    get_app_settings_from_db(&db)
}

/// Update application settings
#[tauri::command]
pub fn update_app_settings(
    db: State<'_, Arc<Mutex<Database>>>,
    settings: AppSettings,
) -> Result<(), String> {
    info!(
        "[Settings] Updating app settings: enabled_editors={:?}",
        settings.enabled_editors
    );
    let db = db.lock().map_err(|e| e.to_string())?;
    update_app_settings_in_db(&db, &settings)
}

/// Get info about available editors with their enabled status
#[tauri::command]
pub fn get_available_editors(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<EditorInfo>, String> {
    info!("[Settings] Getting available editors");
    let db = db.lock().map_err(|e| e.to_string())?;

    let enabled = get_enabled_editors_from_db(&db);
    let mut editors = Vec::new();

    // Claude Code
    if let Ok(paths) = get_claude_paths() {
        editors.push(EditorInfo {
            id: "claude_code".to_string(),
            name: "Claude Code".to_string(),
            is_installed: paths.claude_dir.exists(),
            is_enabled: enabled.contains(&"claude_code".to_string()),
            config_path: paths.claude_json.to_string_lossy().to_string(),
        });
    }

    // OpenCode
    if let Ok(paths) = get_opencode_paths() {
        editors.push(EditorInfo {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            is_installed: is_opencode_installed(),
            is_enabled: enabled.contains(&"opencode".to_string()),
            config_path: paths.config_file.to_string_lossy().to_string(),
        });
    }

    // Codex CLI
    if let Ok(paths) = get_codex_paths() {
        editors.push(EditorInfo {
            id: "codex".to_string(),
            name: "Codex CLI".to_string(),
            is_installed: is_codex_installed(),
            is_enabled: enabled.contains(&"codex".to_string()),
            config_path: paths.config_file.to_string_lossy().to_string(),
        });
    }

    // GitHub Copilot CLI
    if let Ok(paths) = get_copilot_paths() {
        editors.push(EditorInfo {
            id: "copilot".to_string(),
            name: "Copilot CLI".to_string(),
            is_installed: is_copilot_installed(),
            is_enabled: enabled.contains(&"copilot".to_string()),
            config_path: paths.mcp_config_file.to_string_lossy().to_string(),
        });
    }

    // Cursor IDE
    if let Ok(paths) = get_cursor_paths() {
        editors.push(EditorInfo {
            id: "cursor".to_string(),
            name: "Cursor".to_string(),
            is_installed: is_cursor_installed(),
            is_enabled: enabled.contains(&"cursor".to_string()),
            config_path: paths.mcp_config_file.to_string_lossy().to_string(),
        });
    }

    // Gemini CLI
    if let Ok(paths) = get_gemini_paths() {
        editors.push(EditorInfo {
            id: "gemini".to_string(),
            name: "Gemini CLI".to_string(),
            is_installed: is_gemini_installed(),
            is_enabled: enabled.contains(&"gemini".to_string()),
            config_path: paths.settings_file.to_string_lossy().to_string(),
        });
    }

    Ok(editors)
}

/// Toggle an editor's enabled status
#[tauri::command]
pub fn toggle_editor(
    db: State<'_, Arc<Mutex<Database>>>,
    editor_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!("[Settings] Toggling editor {} to {}", editor_id, enabled);
    let db = db.lock().map_err(|e| e.to_string())?;

    let mut editors = get_enabled_editors_from_db(&db);

    if enabled {
        if !editors.contains(&editor_id) {
            editors.push(editor_id);
        }
    } else {
        editors.retain(|e| e != &editor_id);
    }

    let settings = AppSettings {
        enabled_editors: editors,
    };
    update_app_settings_in_db(&db, &settings)
}

/// Set GitHub personal access token
#[tauri::command]
pub fn set_github_token(db: State<'_, Arc<Mutex<Database>>>, token: String) -> Result<(), String> {
    info!("[Settings] Setting GitHub token");
    let db = db.lock().map_err(|e| e.to_string())?;
    let value = if token.trim().is_empty() { "" } else { token.trim() };
    db.set_setting("github_token", value)
        .map_err(|e| e.to_string())
}

/// Clear GitHub personal access token
#[tauri::command]
pub fn clear_github_token(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    info!("[Settings] Clearing GitHub token");
    let db = db.lock().map_err(|e| e.to_string())?;
    db.set_setting("github_token", "")
        .map_err(|e| e.to_string())
}

/// Check if a GitHub token is configured (without returning the token itself)
#[tauri::command]
pub fn has_github_token(db: State<'_, Arc<Mutex<Database>>>) -> Result<bool, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    Ok(get_github_token_from_db(&db).is_some())
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

/// Get Codex CLI paths
#[tauri::command]
pub fn get_codex_paths_cmd() -> Result<CodexPaths, String> {
    info!("[Settings] Getting Codex paths");

    let paths = get_codex_paths().map_err(|e| e.to_string())?;

    Ok(CodexPaths {
        config_dir: paths.config_dir.to_string_lossy().to_string(),
        config_file: paths.config_file.to_string_lossy().to_string(),
    })
}

/// Get GitHub Copilot CLI paths
#[tauri::command]
pub fn get_copilot_paths_cmd() -> Result<CopilotPaths, String> {
    info!("[Settings] Getting Copilot CLI paths");

    let paths = get_copilot_paths().map_err(|e| e.to_string())?;

    Ok(CopilotPaths {
        config_dir: paths.config_dir.to_string_lossy().to_string(),
        config_file: paths.config_file.to_string_lossy().to_string(),
        mcp_config_file: paths.mcp_config_file.to_string_lossy().to_string(),
        agents_dir: paths.agents_dir.to_string_lossy().to_string(),
    })
}

/// Get Cursor IDE paths
#[tauri::command]
pub fn get_cursor_paths_cmd() -> Result<CursorPaths, String> {
    info!("[Settings] Getting Cursor paths");

    let paths = get_cursor_paths().map_err(|e| e.to_string())?;

    Ok(CursorPaths {
        config_dir: paths.config_dir.to_string_lossy().to_string(),
        mcp_config_file: paths.mcp_config_file.to_string_lossy().to_string(),
    })
}

/// Get Gemini CLI paths
#[tauri::command]
pub fn get_gemini_paths_cmd() -> Result<GeminiPaths, String> {
    info!("[Settings] Getting Gemini CLI paths");

    let paths = get_gemini_paths().map_err(|e| e.to_string())?;

    Ok(GeminiPaths {
        config_dir: paths.config_dir.to_string_lossy().to_string(),
        settings_file: paths.settings_file.to_string_lossy().to_string(),
    })
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Get GitHub token from database (returns None if empty/whitespace)
pub fn get_github_token_from_db(db: &Database) -> Option<String> {
    db.get_setting("github_token")
        .filter(|s| !s.trim().is_empty())
}

/// Get app settings directly from the database
pub fn get_app_settings_from_db(db: &Database) -> Result<AppSettings, String> {
    let enabled_editors = get_enabled_editors_from_db(db);
    Ok(AppSettings { enabled_editors })
}

/// Get list of enabled editors from the database
pub fn get_enabled_editors_from_db(db: &Database) -> Vec<String> {
    db.get_setting("enabled_editors")
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| vec!["claude_code".to_string()])
}

/// Update app settings directly in the database
pub fn update_app_settings_in_db(db: &Database, settings: &AppSettings) -> Result<(), String> {
    let json = serde_json::to_string(&settings.enabled_editors).map_err(|e| e.to_string())?;
    db.set_setting("enabled_editors", &json)
        .map_err(|e| e.to_string())
}

/// Check if a specific editor is enabled
#[cfg(test)]
pub fn is_editor_enabled(db: &Database, editor_id: &str) -> bool {
    get_enabled_editors_from_db(db).contains(&editor_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // AppSettings tests
    // =========================================================================

    #[test]
    fn test_get_app_settings_default() {
        let db = Database::in_memory().unwrap();

        let settings = get_app_settings_from_db(&db).unwrap();

        // Default is claude_code only
        assert_eq!(settings.enabled_editors, vec!["claude_code".to_string()]);
    }

    #[test]
    fn test_update_and_get_app_settings() {
        let db = Database::in_memory().unwrap();

        // Enable both editors
        let settings = AppSettings {
            enabled_editors: vec!["claude_code".to_string(), "opencode".to_string()],
        };
        update_app_settings_in_db(&db, &settings).unwrap();

        let fetched = get_app_settings_from_db(&db).unwrap();
        assert_eq!(fetched.enabled_editors.len(), 2);
        assert!(fetched.enabled_editors.contains(&"claude_code".to_string()));
        assert!(fetched.enabled_editors.contains(&"opencode".to_string()));
    }

    #[test]
    fn test_disable_all_editors() {
        let db = Database::in_memory().unwrap();

        // Disable all editors
        let settings = AppSettings {
            enabled_editors: vec![],
        };
        update_app_settings_in_db(&db, &settings).unwrap();

        let fetched = get_app_settings_from_db(&db).unwrap();
        assert!(fetched.enabled_editors.is_empty());
    }

    #[test]
    fn test_is_editor_enabled() {
        let db = Database::in_memory().unwrap();

        // Default: only claude_code is enabled
        assert!(is_editor_enabled(&db, "claude_code"));
        assert!(!is_editor_enabled(&db, "opencode"));

        // Enable opencode
        let settings = AppSettings {
            enabled_editors: vec!["claude_code".to_string(), "opencode".to_string()],
        };
        update_app_settings_in_db(&db, &settings).unwrap();

        assert!(is_editor_enabled(&db, "claude_code"));
        assert!(is_editor_enabled(&db, "opencode"));
    }

    #[test]
    fn test_app_settings_serde() {
        let settings = AppSettings {
            enabled_editors: vec!["claude_code".to_string(), "opencode".to_string()],
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("enabledEditors")); // camelCase

        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled_editors.len(), 2);
    }

    // =========================================================================
    // EditorInfo tests
    // =========================================================================

    #[test]
    fn test_editor_info_serde() {
        let info = EditorInfo {
            id: "claude_code".to_string(),
            name: "Claude Code".to_string(),
            is_installed: true,
            is_enabled: true,
            config_path: "/home/user/.claude.json".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        // Should use camelCase
        assert!(json.contains("isInstalled"));
        assert!(json.contains("isEnabled"));
        assert!(json.contains("configPath"));

        let deserialized: EditorInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "claude_code");
        assert!(deserialized.is_installed);
        assert!(deserialized.is_enabled);
    }

    #[test]
    fn test_editor_info_disabled() {
        let info = EditorInfo {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            is_installed: true,
            is_enabled: false,
            config_path: "/home/user/.config/opencode/opencode.json".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: EditorInfo = serde_json::from_str(&json).unwrap();

        assert!(deserialized.is_installed);
        assert!(!deserialized.is_enabled);
    }

    // =========================================================================
    // OpenCodePaths tests
    // =========================================================================

    #[test]
    fn test_opencode_paths_serde() {
        let paths = OpenCodePaths {
            config_dir: "/home/user/.config/opencode".to_string(),
            config_file: "/home/user/.config/opencode/opencode.json".to_string(),
            command_dir: "/home/user/.config/opencode/command".to_string(),
            agent_dir: "/home/user/.config/opencode/agent".to_string(),
            plugin_dir: "/home/user/.config/opencode/plugin".to_string(),
            tool_dir: "/home/user/.config/opencode/tool".to_string(),
            knowledge_dir: "/home/user/.config/opencode/knowledge".to_string(),
        };

        let json = serde_json::to_string(&paths).unwrap();
        // Should use camelCase
        assert!(json.contains("configDir"));
        assert!(json.contains("configFile"));
        assert!(json.contains("commandDir"));
        assert!(json.contains("agentDir"));
        assert!(json.contains("pluginDir"));
        assert!(json.contains("toolDir"));
        assert!(json.contains("knowledgeDir"));

        let deserialized: OpenCodePaths = serde_json::from_str(&json).unwrap();
        assert!(deserialized.config_dir.contains("opencode"));
    }
}
