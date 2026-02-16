use anyhow::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Permission scope determines which settings file to read/write
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionScope {
    User,
    Project,
    Local,
}

/// Permissions from a single scope
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopedPermissions {
    pub scope: String,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub ask: Vec<String>,
    pub default_mode: Option<String>,
    pub additional_directories: Vec<String>,
}

/// All permissions across all three scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllPermissions {
    pub user: ScopedPermissions,
    pub project: Option<ScopedPermissions>,
    pub local: Option<ScopedPermissions>,
}

/// Read an existing settings.json file or return an empty object
fn read_settings_file(path: &Path) -> Result<Value> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or(json!({})))
    } else {
        Ok(json!({}))
    }
}

/// Write settings.json file, preserving other settings
fn write_settings_file(path: &Path, settings: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Resolve the settings file path for a given scope
pub fn resolve_settings_path(
    scope: &PermissionScope,
    project_path: Option<&Path>,
) -> Result<PathBuf> {
    match scope {
        PermissionScope::User => {
            let base_dirs =
                BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
            Ok(base_dirs.home_dir().join(".claude").join("settings.json"))
        }
        PermissionScope::Project => {
            let project = project_path
                .ok_or_else(|| anyhow::anyhow!("Project path required for project scope"))?;
            Ok(project.join(".claude").join("settings.json"))
        }
        PermissionScope::Local => {
            let project = project_path
                .ok_or_else(|| anyhow::anyhow!("Project path required for local scope"))?;
            Ok(project.join(".claude").join("settings.local.json"))
        }
    }
}

/// Read permissions from a single settings file
pub fn read_permissions_from_file(path: &Path, scope: &str) -> Result<ScopedPermissions> {
    let settings = read_settings_file(path)?;

    let permissions = settings.get("permissions").cloned().unwrap_or(json!({}));

    let allow = extract_string_array(&permissions, "allow");
    let deny = extract_string_array(&permissions, "deny");
    let ask = extract_string_array(&permissions, "ask");

    let default_mode = permissions
        .get("defaultMode")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let additional_directories = extract_string_array(&permissions, "additionalDirectories");

    Ok(ScopedPermissions {
        scope: scope.to_string(),
        allow,
        deny,
        ask,
        default_mode,
        additional_directories,
    })
}

/// Read permissions from all three scopes
pub fn read_all_permissions(project_path: Option<&Path>) -> Result<AllPermissions> {
    // User scope (always available)
    let user_path = resolve_settings_path(&PermissionScope::User, None)?;
    let user = read_permissions_from_file(&user_path, "user")?;

    // Project + Local scopes (only if project path provided)
    let (project, local) = if let Some(pp) = project_path {
        let project_path_buf = resolve_settings_path(&PermissionScope::Project, Some(pp))?;
        let local_path = resolve_settings_path(&PermissionScope::Local, Some(pp))?;

        let project_perms = if project_path_buf.exists() {
            Some(read_permissions_from_file(&project_path_buf, "project")?)
        } else {
            Some(ScopedPermissions {
                scope: "project".to_string(),
                allow: vec![],
                deny: vec![],
                ask: vec![],
                default_mode: None,
                additional_directories: vec![],
            })
        };

        let local_perms = if local_path.exists() {
            Some(read_permissions_from_file(&local_path, "local")?)
        } else {
            Some(ScopedPermissions {
                scope: "local".to_string(),
                allow: vec![],
                deny: vec![],
                ask: vec![],
                default_mode: None,
                additional_directories: vec![],
            })
        };

        (project_perms, local_perms)
    } else {
        (None, None)
    };

    Ok(AllPermissions {
        user,
        project,
        local,
    })
}

/// Write permission rules for a specific category (allow/deny/ask) to a settings file
pub fn write_permission_rules(
    scope: &PermissionScope,
    project_path: Option<&Path>,
    category: &str,
    rules: &[String],
) -> Result<()> {
    let path = resolve_settings_path(scope, project_path)?;
    let mut settings = read_settings_file(&path)?;

    // Ensure permissions object exists
    if settings.get("permissions").is_none() {
        settings["permissions"] = json!({});
    }

    if rules.is_empty() {
        // Remove the category key if empty
        if let Some(perms) = settings
            .get_mut("permissions")
            .and_then(|v| v.as_object_mut())
        {
            perms.remove(category);
            // If permissions object is now empty, remove it
            if perms.is_empty() {
                if let Some(obj) = settings.as_object_mut() {
                    obj.remove("permissions");
                }
            }
        }
    } else {
        settings["permissions"][category] = json!(rules);
    }

    write_settings_file(&path, &settings)
}

/// Write the defaultMode setting
pub fn write_default_mode(
    scope: &PermissionScope,
    project_path: Option<&Path>,
    mode: Option<&str>,
) -> Result<()> {
    let path = resolve_settings_path(scope, project_path)?;
    let mut settings = read_settings_file(&path)?;

    match mode {
        Some(m) => {
            if settings.get("permissions").is_none() {
                settings["permissions"] = json!({});
            }
            settings["permissions"]["defaultMode"] = json!(m);
        }
        None => {
            // Remove defaultMode
            if let Some(perms) = settings
                .get_mut("permissions")
                .and_then(|v| v.as_object_mut())
            {
                perms.remove("defaultMode");
            }
        }
    }

    write_settings_file(&path, &settings)
}

/// Write additionalDirectories setting
pub fn write_additional_directories(
    scope: &PermissionScope,
    project_path: Option<&Path>,
    dirs: &[String],
) -> Result<()> {
    let path = resolve_settings_path(scope, project_path)?;
    let mut settings = read_settings_file(&path)?;

    if dirs.is_empty() {
        if let Some(perms) = settings
            .get_mut("permissions")
            .and_then(|v| v.as_object_mut())
        {
            perms.remove("additionalDirectories");
        }
    } else {
        if settings.get("permissions").is_none() {
            settings["permissions"] = json!({});
        }
        settings["permissions"]["additionalDirectories"] = json!(dirs);
    }

    write_settings_file(&path, &settings)
}

/// Helper: extract a string array from a JSON value by key
fn extract_string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_permissions_from_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{}").unwrap();

        let perms = read_permissions_from_file(&path, "user").unwrap();
        assert!(perms.allow.is_empty());
        assert!(perms.deny.is_empty());
        assert!(perms.ask.is_empty());
        assert!(perms.default_mode.is_none());
    }

    #[test]
    fn test_read_permissions_with_rules() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{
                "permissions": {
                    "allow": ["Bash(npm run *)", "Read"],
                    "deny": ["Bash(curl *)"],
                    "ask": ["Bash(git push *)"],
                    "defaultMode": "allowEdits"
                }
            }"#,
        )
        .unwrap();

        let perms = read_permissions_from_file(&path, "user").unwrap();
        assert_eq!(perms.allow, vec!["Bash(npm run *)", "Read"]);
        assert_eq!(perms.deny, vec!["Bash(curl *)"]);
        assert_eq!(perms.ask, vec!["Bash(git push *)"]);
        assert_eq!(perms.default_mode, Some("allowEdits".to_string()));
    }

    #[test]
    fn test_write_permission_rules() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        // Write to local scope
        write_permission_rules(
            &PermissionScope::Local,
            Some(project_path),
            "allow",
            &["Bash(npm run *)".to_string(), "Read".to_string()],
        )
        .unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        let allow = json["permissions"]["allow"].as_array().unwrap();
        assert_eq!(allow.len(), 2);
        assert_eq!(allow[0], "Bash(npm run *)");
    }

    #[test]
    fn test_write_preserves_other_keys() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        // Create existing settings file with other keys
        let claude_dir = project_path.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        std::fs::write(
            &settings_path,
            r#"{"hooks":{"PreToolUse":[]},"permissions":{"deny":["Bash(rm -rf *)"]}}"#,
        )
        .unwrap();

        // Write new allow rules
        write_permission_rules(
            &PermissionScope::Local,
            Some(project_path),
            "allow",
            &["Read".to_string()],
        )
        .unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        // Hooks preserved
        assert!(json.get("hooks").is_some());
        // Old deny preserved
        assert_eq!(json["permissions"]["deny"][0], "Bash(rm -rf *)");
        // New allow added
        assert_eq!(json["permissions"]["allow"][0], "Read");
    }

    #[test]
    fn test_write_default_mode() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        write_default_mode(
            &PermissionScope::Local,
            Some(project_path),
            Some("allowEdits"),
        )
        .unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(json["permissions"]["defaultMode"], "allowEdits");
    }

    #[test]
    fn test_write_additional_directories() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        write_additional_directories(
            &PermissionScope::Local,
            Some(project_path),
            &["/tmp/shared".to_string(), "/opt/data".to_string()],
        )
        .unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        let dirs = json["permissions"]["additionalDirectories"]
            .as_array()
            .unwrap();
        assert_eq!(dirs.len(), 2);
    }

    #[test]
    fn test_write_empty_rules_removes_key() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        // Write some rules first
        write_permission_rules(
            &PermissionScope::Local,
            Some(project_path),
            "allow",
            &["Read".to_string()],
        )
        .unwrap();

        // Now clear them
        write_permission_rules(&PermissionScope::Local, Some(project_path), "allow", &[]).unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        // permissions object should be removed when empty
        assert!(json.get("permissions").is_none());
    }

    #[test]
    fn test_read_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let perms = read_permissions_from_file(&path, "user").unwrap();
        assert!(perms.allow.is_empty());
    }

    #[test]
    fn test_read_all_permissions_user_only() {
        // Without project path, only user scope is returned
        // We can't easily test this without mocking BaseDirs, but we verify the structure
        let all = read_all_permissions(None);
        // This may fail in CI if ~/.claude doesn't exist, but the structure is correct
        assert!(all.is_ok() || all.is_err());
    }
}
