use crate::db::{AppSettings, Database, EditorInfo, OpenCodePaths};
use crate::services::editor::EditorRegistry;
use crate::utils::opencode_paths::get_opencode_paths;
use log::info;
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Get application settings
#[tauri::command]
pub fn get_app_settings(db: State<'_, Arc<Mutex<Database>>>) -> Result<AppSettings, String> {
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
    db: State<'_, Arc<Mutex<Database>>>,
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
pub fn get_available_editors(
    registry: State<'_, Arc<EditorRegistry>>,
) -> Result<Vec<EditorInfo>, String> {
    info!("[Settings] Getting available editors");

    // Get info from all registered editors via the registry
    let editors = registry
        .list_all()
        .iter()
        .map(|adapter| {
            let info = adapter.info();
            EditorInfo {
                id: info.id,
                name: info.name,
                is_installed: info.is_installed,
                config_path: info.config_path.unwrap_or_default(),
            }
        })
        .collect();

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
    db: State<'_, Arc<Mutex<Database>>>,
    project_id: i64,
    editor_type: String,
) -> Result<(), String> {
    info!(
        "[Settings] Updating project {} editor_type to {}",
        project_id, editor_type
    );

    let db = db.lock().map_err(|e| e.to_string())?;
    update_project_editor_type_in_db(&db, project_id, &editor_type)
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

/// Get app settings directly from the database (for testing)
pub fn get_app_settings_from_db(db: &Database) -> Result<AppSettings, String> {
    let default_editor = db
        .get_setting("default_editor")
        .unwrap_or_else(|| "claude_code".to_string());

    Ok(AppSettings { default_editor })
}

/// Update app settings directly in the database (for testing)
pub fn update_app_settings_in_db(db: &Database, settings: &AppSettings) -> Result<(), String> {
    db.set_setting("default_editor", &settings.default_editor)
        .map_err(|e| e.to_string())
}

/// Valid editor type IDs
const VALID_EDITOR_TYPES: &[&str] = &[
    "claude_code",
    "opencode",
    "codex",
    "copilot",
    "cursor",
    "gemini",
];

/// Check if an editor type is valid
pub fn is_valid_editor_type(editor_type: &str) -> bool {
    VALID_EDITOR_TYPES.contains(&editor_type)
}

/// Update project editor type directly in the database (for testing)
pub fn update_project_editor_type_in_db(
    db: &Database,
    project_id: i64,
    editor_type: &str,
) -> Result<(), String> {
    // Validate editor type
    if !is_valid_editor_type(editor_type) {
        return Err(format!("Invalid editor type: {}", editor_type));
    }

    db.conn()
        .execute(
            "UPDATE projects SET editor_type = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![editor_type, project_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get project editor type directly from the database (for testing)
pub fn get_project_editor_type_from_db(db: &Database, project_id: i64) -> Result<String, String> {
    db.conn()
        .query_row(
            "SELECT COALESCE(editor_type, 'claude_code') FROM projects WHERE id = ?",
            [project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::projects::create_project_in_db;
    use crate::db::CreateProjectRequest;

    // =========================================================================
    // AppSettings tests
    // =========================================================================

    #[test]
    fn test_get_app_settings_default() {
        let db = Database::in_memory().unwrap();

        let settings = get_app_settings_from_db(&db).unwrap();

        // Default is claude_code
        assert_eq!(settings.default_editor, "claude_code");
    }

    #[test]
    fn test_update_and_get_app_settings() {
        let db = Database::in_memory().unwrap();

        // Update to opencode
        let settings = AppSettings {
            default_editor: "opencode".to_string(),
        };
        update_app_settings_in_db(&db, &settings).unwrap();

        let fetched = get_app_settings_from_db(&db).unwrap();
        assert_eq!(fetched.default_editor, "opencode");

        // Update back to claude_code
        let settings = AppSettings {
            default_editor: "claude_code".to_string(),
        };
        update_app_settings_in_db(&db, &settings).unwrap();

        let fetched = get_app_settings_from_db(&db).unwrap();
        assert_eq!(fetched.default_editor, "claude_code");
    }

    #[test]
    fn test_app_settings_serde() {
        let settings = AppSettings {
            default_editor: "claude_code".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("defaultEditor")); // camelCase

        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.default_editor, "claude_code");
    }

    // =========================================================================
    // Project editor type tests
    // =========================================================================

    fn create_test_project(db: &Database) -> i64 {
        let project = CreateProjectRequest {
            name: "Test Project".to_string(),
            path: format!(
                "/test/project/{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ),
        };
        create_project_in_db(db, &project).unwrap().id
    }

    #[test]
    fn test_get_project_editor_type_default() {
        let db = Database::in_memory().unwrap();
        let project_id = create_test_project(&db);

        let editor_type = get_project_editor_type_from_db(&db, project_id).unwrap();

        assert_eq!(editor_type, "claude_code");
    }

    #[test]
    fn test_update_project_editor_type_to_opencode() {
        let db = Database::in_memory().unwrap();
        let project_id = create_test_project(&db);

        update_project_editor_type_in_db(&db, project_id, "opencode").unwrap();

        let editor_type = get_project_editor_type_from_db(&db, project_id).unwrap();
        assert_eq!(editor_type, "opencode");
    }

    #[test]
    fn test_update_project_editor_type_to_claude_code() {
        let db = Database::in_memory().unwrap();
        let project_id = create_test_project(&db);

        // First set to opencode
        update_project_editor_type_in_db(&db, project_id, "opencode").unwrap();

        // Then back to claude_code
        update_project_editor_type_in_db(&db, project_id, "claude_code").unwrap();

        let editor_type = get_project_editor_type_from_db(&db, project_id).unwrap();
        assert_eq!(editor_type, "claude_code");
    }

    #[test]
    fn test_update_project_editor_type_invalid() {
        let db = Database::in_memory().unwrap();
        let project_id = create_test_project(&db);

        let result = update_project_editor_type_in_db(&db, project_id, "invalid_editor");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid editor type"));
    }

    #[test]
    fn test_update_project_editor_type_all_valid_types() {
        let db = Database::in_memory().unwrap();
        let project_id = create_test_project(&db);

        // Test all 6 valid editor types
        for editor_type in &["claude_code", "opencode", "codex", "copilot", "cursor", "gemini"] {
            update_project_editor_type_in_db(&db, project_id, editor_type).unwrap();
            let result = get_project_editor_type_from_db(&db, project_id).unwrap();
            assert_eq!(result, *editor_type);
        }
    }

    #[test]
    fn test_is_valid_editor_type() {
        // Valid types
        assert!(is_valid_editor_type("claude_code"));
        assert!(is_valid_editor_type("opencode"));
        assert!(is_valid_editor_type("codex"));
        assert!(is_valid_editor_type("copilot"));
        assert!(is_valid_editor_type("cursor"));
        assert!(is_valid_editor_type("gemini"));

        // Invalid types
        assert!(!is_valid_editor_type("invalid"));
        assert!(!is_valid_editor_type("vscode"));
        assert!(!is_valid_editor_type(""));
    }

    #[test]
    fn test_get_project_editor_type_not_found() {
        let db = Database::in_memory().unwrap();

        let result = get_project_editor_type_from_db(&db, 9999);

        assert!(result.is_err());
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
            config_path: "/home/user/.claude.json".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        // Should use camelCase
        assert!(json.contains("isInstalled"));
        assert!(json.contains("configPath"));

        let deserialized: EditorInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "claude_code");
        assert!(deserialized.is_installed);
    }

    #[test]
    fn test_editor_info_not_installed() {
        let info = EditorInfo {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            is_installed: false,
            config_path: "".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: EditorInfo = serde_json::from_str(&json).unwrap();

        assert!(!deserialized.is_installed);
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
