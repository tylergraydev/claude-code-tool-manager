use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::claude_settings::{read_claude_settings_from_file, ClaudeSettings};

/// Info about the managed-settings.json file and its contents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSettingsInfo {
    pub file_path: String,
    pub exists: bool,
    pub settings: Option<ClaudeSettings>,
}

/// Get the OS-specific path where managed-settings.json lives
pub fn managed_settings_path() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\Program Files\ClaudeCode\managed-settings.json")
    } else {
        // Linux / WSL
        PathBuf::from("/etc/claude-code/managed-settings.json")
    }
}

/// Read managed settings from the OS-specific default path
pub fn read_managed_settings() -> Result<ManagedSettingsInfo> {
    let path = managed_settings_path();
    read_managed_settings_from_path(&path)
}

/// Read managed settings from a given path (testable variant)
pub fn read_managed_settings_from_path(path: &Path) -> Result<ManagedSettingsInfo> {
    let file_path = path.to_string_lossy().to_string();

    if !path.exists() {
        return Ok(ManagedSettingsInfo {
            file_path,
            exists: false,
            settings: None,
        });
    }

    let settings = read_claude_settings_from_file(path, "managed")?;
    Ok(ManagedSettingsInfo {
        file_path,
        exists: true,
        settings: Some(settings),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonexistent_file_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("managed-settings.json");

        let info = read_managed_settings_from_path(&path).unwrap();
        assert!(!info.exists);
        assert!(info.settings.is_none());
        assert!(!info.file_path.is_empty());
    }

    #[test]
    fn test_valid_file_parses_standard_and_managed_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("managed-settings.json");
        std::fs::write(
            &path,
            r#"{
                "model": "claude-sonnet-4-5-20250929",
                "allowManagedHooksOnly": true,
                "allowManagedPermissionRulesOnly": false,
                "disableBypassPermissionsMode": true,
                "allowedMcpServers": ["server-a", "server-b"],
                "deniedMcpServers": ["server-c"],
                "strictKnownMarketplaces": true,
                "companyAnnouncements": ["Welcome!", "Read the policy"],
                "forceLoginMethod": "console",
                "forceLoginOrgUUID": "org-123-abc"
            }"#,
        )
        .unwrap();

        let info = read_managed_settings_from_path(&path).unwrap();
        assert!(info.exists);

        let s = info.settings.unwrap();
        assert_eq!(s.model, Some("claude-sonnet-4-5-20250929".to_string()));
        assert_eq!(s.allow_managed_hooks_only, Some(true));
        assert_eq!(s.allow_managed_permission_rules_only, Some(false));
        assert_eq!(s.disable_bypass_permissions_mode, Some(true));
        assert_eq!(
            s.allowed_mcp_servers,
            Some(vec!["server-a".to_string(), "server-b".to_string()])
        );
        assert_eq!(s.denied_mcp_servers, Some(vec!["server-c".to_string()]));
        assert_eq!(s.strict_known_marketplaces, Some(true));
        assert_eq!(
            s.company_announcements,
            Some(vec!["Welcome!".to_string(), "Read the policy".to_string()])
        );
        assert_eq!(s.force_login_method, Some("console".to_string()));
        assert_eq!(s.force_login_org_uuid, Some("org-123-abc".to_string()));
    }

    #[test]
    fn test_path_function_returns_non_empty() {
        let path = managed_settings_path();
        assert!(!path.to_string_lossy().is_empty());
    }

    #[test]
    fn test_empty_file_returns_settings_with_all_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("managed-settings.json");
        std::fs::write(&path, "{}").unwrap();

        let info = read_managed_settings_from_path(&path).unwrap();
        assert!(info.exists);

        let s = info.settings.unwrap();
        assert!(s.model.is_none());
        assert!(s.allow_managed_hooks_only.is_none());
        assert!(s.allow_managed_permission_rules_only.is_none());
        assert!(s.disable_bypass_permissions_mode.is_none());
        assert!(s.allowed_mcp_servers.is_none());
        assert!(s.denied_mcp_servers.is_none());
        assert!(s.strict_known_marketplaces.is_none());
        assert!(s.company_announcements.is_none());
        assert!(s.force_login_method.is_none());
        assert!(s.force_login_org_uuid.is_none());
    }
}
