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
pub fn get_all_claude_settings(project_path: Option<String>) -> Result<AllClaudeSettings, String> {
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
    let path = crate::services::permission_writer::resolve_settings_path(&ps, pp)
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
    let path = crate::services::permission_writer::resolve_settings_path(&ps, pp)
        .map_err(|e| e.to_string())?;
    claude_settings::read_claude_settings_from_file(&path, &scope).map_err(|e| e.to_string())
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
        let result = parse_scope("admin");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid scope"));
    }

    #[test]
    fn test_parse_scope_case_sensitive() {
        // Uppercase should not match
        assert!(parse_scope("User").is_err());
        assert!(parse_scope("PROJECT").is_err());
    }

    #[test]
    fn test_parse_scope_empty() {
        assert!(parse_scope("").is_err());
    }

    #[test]
    fn test_parse_scope_whitespace() {
        assert!(parse_scope(" user ").is_err());
    }

    #[test]
    fn test_claude_settings_serde_minimal() {
        // ClaudeSettings is Serialize+Deserialize - test the scope field
        let json = r#"{"scope":"project","availableModels":["opus","sonnet"]}"#;
        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.scope, "project");
        assert_eq!(settings.available_models.len(), 2);
    }

    #[test]
    fn test_all_claude_settings_serde_round_trip() {
        let json = r#"{"user":{"scope":"user","availableModels":[]},"project":null,"local":null}"#;
        let deser: AllClaudeSettings = serde_json::from_str(json).unwrap();
        assert_eq!(deser.user.scope, "user");
        assert!(deser.project.is_none());
        assert!(deser.local.is_none());
    }
}
