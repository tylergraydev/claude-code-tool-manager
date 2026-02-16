use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

use super::permission_writer::{resolve_settings_path, PermissionScope};

/// Sandbox network configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SandboxNetworkSettings {
    pub allow_unix_sockets: Option<Vec<String>>,
    pub allow_all_unix_sockets: Option<bool>,
    pub allow_local_binding: Option<bool>,
    pub allowed_domains: Option<Vec<String>>,
    pub http_proxy_port: Option<u16>,
    pub socks_proxy_port: Option<u16>,
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SandboxSettings {
    pub enabled: Option<bool>,
    pub auto_allow_bash_if_sandboxed: Option<bool>,
    pub excluded_commands: Option<Vec<String>>,
    pub allow_unsandboxed_commands: Option<bool>,
    pub enable_weaker_nested_sandbox: Option<bool>,
    pub network: Option<SandboxNetworkSettings>,
}

/// Claude settings from a single scope (model config + attribution + sandbox)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeSettings {
    pub scope: String,
    // Model config
    pub model: Option<String>,
    pub available_models: Vec<String>,
    pub output_style: Option<String>,
    pub language: Option<String>,
    pub always_thinking_enabled: Option<bool>,
    // Attribution
    pub attribution_commit: Option<String>,
    pub attribution_pr: Option<String>,
    // Sandbox
    pub sandbox: Option<SandboxSettings>,
}

/// All claude settings across all three scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllClaudeSettings {
    pub user: ClaudeSettings,
    pub project: Option<ClaudeSettings>,
    pub local: Option<ClaudeSettings>,
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

/// Write settings.json file
fn write_settings_file(path: &Path, settings: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)?;
    Ok(())
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

/// Read claude settings from a single settings file
pub fn read_claude_settings_from_file(path: &Path, scope: &str) -> Result<ClaudeSettings> {
    let settings = read_settings_file(path)?;

    let model = settings
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let available_models = extract_string_array(&settings, "availableModels");

    let output_style = settings
        .get("outputStyle")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let language = settings
        .get("language")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let always_thinking_enabled = settings
        .get("alwaysThinkingEnabled")
        .and_then(|v| v.as_bool());

    // Attribution is nested: { "attribution": { "commit": "...", "pr": "..." } }
    let attribution = settings.get("attribution").cloned().unwrap_or(json!({}));

    let attribution_commit = attribution
        .get("commit")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let attribution_pr = attribution
        .get("pr")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Sandbox settings (nested object)
    let sandbox = settings.get("sandbox").and_then(|v| {
        if v.is_object() {
            serde_json::from_value::<SandboxSettings>(v.clone()).ok()
        } else {
            None
        }
    });

    Ok(ClaudeSettings {
        scope: scope.to_string(),
        model,
        available_models,
        output_style,
        language,
        always_thinking_enabled,
        attribution_commit,
        attribution_pr,
        sandbox,
    })
}

/// Read claude settings from all three scopes
pub fn read_all_claude_settings(project_path: Option<&Path>) -> Result<AllClaudeSettings> {
    // User scope (always available)
    let user_path = resolve_settings_path(&PermissionScope::User, None)?;
    let user = read_claude_settings_from_file(&user_path, "user")?;

    // Project + Local scopes (only if project path provided)
    let (project, local) = if let Some(pp) = project_path {
        let project_path_buf = resolve_settings_path(&PermissionScope::Project, Some(pp))?;
        let local_path = resolve_settings_path(&PermissionScope::Local, Some(pp))?;

        let project_settings = if project_path_buf.exists() {
            Some(read_claude_settings_from_file(
                &project_path_buf,
                "project",
            )?)
        } else {
            Some(ClaudeSettings {
                scope: "project".to_string(),
                model: None,
                available_models: vec![],
                output_style: None,
                language: None,
                always_thinking_enabled: None,
                attribution_commit: None,
                attribution_pr: None,
                sandbox: None,
            })
        };

        let local_settings = if local_path.exists() {
            Some(read_claude_settings_from_file(&local_path, "local")?)
        } else {
            Some(ClaudeSettings {
                scope: "local".to_string(),
                model: None,
                available_models: vec![],
                output_style: None,
                language: None,
                always_thinking_enabled: None,
                attribution_commit: None,
                attribution_pr: None,
                sandbox: None,
            })
        };

        (project_settings, local_settings)
    } else {
        (None, None)
    };

    Ok(AllClaudeSettings {
        user,
        project,
        local,
    })
}

/// Write claude settings to a settings file, preserving all other keys
pub fn write_claude_settings(
    scope: &PermissionScope,
    project_path: Option<&Path>,
    settings: &ClaudeSettings,
) -> Result<()> {
    let path = resolve_settings_path(scope, project_path)?;
    let mut file_settings = read_settings_file(&path)?;

    // Model config: set or remove top-level keys
    set_or_remove_string(&mut file_settings, "model", &settings.model);
    set_or_remove_string(&mut file_settings, "outputStyle", &settings.output_style);
    set_or_remove_string(&mut file_settings, "language", &settings.language);

    // Available models: set or remove array
    if settings.available_models.is_empty() {
        if let Some(obj) = file_settings.as_object_mut() {
            obj.remove("availableModels");
        }
    } else {
        file_settings["availableModels"] = json!(settings.available_models);
    }

    // Always thinking enabled: set or remove boolean
    match settings.always_thinking_enabled {
        Some(val) => {
            file_settings["alwaysThinkingEnabled"] = json!(val);
        }
        None => {
            if let Some(obj) = file_settings.as_object_mut() {
                obj.remove("alwaysThinkingEnabled");
            }
        }
    }

    // Attribution: manage nested object
    let has_commit = settings.attribution_commit.is_some();
    let has_pr = settings.attribution_pr.is_some();

    if has_commit || has_pr {
        let mut attribution = file_settings
            .get("attribution")
            .cloned()
            .unwrap_or(json!({}));

        set_or_remove_string_in(&mut attribution, "commit", &settings.attribution_commit);
        set_or_remove_string_in(&mut attribution, "pr", &settings.attribution_pr);

        // If attribution object is now empty, remove it
        if attribution.as_object().map_or(true, |o| o.is_empty()) {
            if let Some(obj) = file_settings.as_object_mut() {
                obj.remove("attribution");
            }
        } else {
            file_settings["attribution"] = attribution;
        }
    } else {
        // Both None — remove attribution object entirely
        if let Some(obj) = file_settings.as_object_mut() {
            obj.remove("attribution");
        }
    }

    // Sandbox: write nested object or remove if None/empty
    match &settings.sandbox {
        Some(sandbox) => {
            let sandbox_value = serde_json::to_value(sandbox)?;
            // Check if the serialized sandbox object has any non-null values
            if sandbox_value
                .as_object()
                .map_or(true, |o| o.values().all(|v| v.is_null()))
            {
                if let Some(obj) = file_settings.as_object_mut() {
                    obj.remove("sandbox");
                }
            } else {
                // Remove null keys from the sandbox object before writing
                let mut clean = serde_json::Map::new();
                if let Some(obj) = sandbox_value.as_object() {
                    for (k, v) in obj {
                        if !v.is_null() {
                            clean.insert(k.clone(), v.clone());
                        }
                    }
                }
                if clean.is_empty() {
                    if let Some(obj) = file_settings.as_object_mut() {
                        obj.remove("sandbox");
                    }
                } else {
                    // Also clean network sub-object of nulls
                    if let Some(network) = clean.get("network") {
                        if let Some(net_obj) = network.as_object() {
                            let mut clean_net = serde_json::Map::new();
                            for (k, v) in net_obj {
                                if !v.is_null() {
                                    clean_net.insert(k.clone(), v.clone());
                                }
                            }
                            if clean_net.is_empty() {
                                clean.remove("network");
                            } else {
                                clean.insert("network".to_string(), Value::Object(clean_net));
                            }
                        }
                    }
                    if clean.is_empty() {
                        if let Some(obj) = file_settings.as_object_mut() {
                            obj.remove("sandbox");
                        }
                    } else {
                        file_settings["sandbox"] = Value::Object(clean);
                    }
                }
            }
        }
        None => {
            if let Some(obj) = file_settings.as_object_mut() {
                obj.remove("sandbox");
            }
        }
    }

    write_settings_file(&path, &file_settings)
}

/// Helper: set a string key or remove it if None
fn set_or_remove_string(settings: &mut Value, key: &str, value: &Option<String>) {
    match value {
        Some(v) => {
            settings[key] = json!(v);
        }
        None => {
            if let Some(obj) = settings.as_object_mut() {
                obj.remove(key);
            }
        }
    }
}

/// Helper: set a string key within a nested object, or remove it if None
fn set_or_remove_string_in(parent: &mut Value, key: &str, value: &Option<String>) {
    match value {
        Some(v) => {
            parent[key] = json!(v);
        }
        None => {
            if let Some(obj) = parent.as_object_mut() {
                obj.remove(key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_claude_settings_from_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "{}").unwrap();

        let settings = read_claude_settings_from_file(&path, "user").unwrap();
        assert!(settings.model.is_none());
        assert!(settings.available_models.is_empty());
        assert!(settings.output_style.is_none());
        assert!(settings.language.is_none());
        assert!(settings.always_thinking_enabled.is_none());
        assert!(settings.attribution_commit.is_none());
        assert!(settings.attribution_pr.is_none());
        assert!(settings.sandbox.is_none());
    }

    #[test]
    fn test_read_claude_settings_with_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{
                "model": "claude-sonnet-4-5-20250929",
                "availableModels": ["sonnet", "haiku"],
                "outputStyle": "Concise",
                "language": "japanese",
                "alwaysThinkingEnabled": true,
                "attribution": {
                    "commit": "Generated by Claude",
                    "pr": "Created with AI"
                }
            }"#,
        )
        .unwrap();

        let settings = read_claude_settings_from_file(&path, "user").unwrap();
        assert_eq!(
            settings.model,
            Some("claude-sonnet-4-5-20250929".to_string())
        );
        assert_eq!(settings.available_models, vec!["sonnet", "haiku"]);
        assert_eq!(settings.output_style, Some("Concise".to_string()));
        assert_eq!(settings.language, Some("japanese".to_string()));
        assert_eq!(settings.always_thinking_enabled, Some(true));
        assert_eq!(
            settings.attribution_commit,
            Some("Generated by Claude".to_string())
        );
        assert_eq!(settings.attribution_pr, Some("Created with AI".to_string()));
    }

    #[test]
    fn test_write_claude_settings() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        let settings = ClaudeSettings {
            scope: "local".to_string(),
            model: Some("claude-sonnet-4-5-20250929".to_string()),
            available_models: vec!["sonnet".to_string(), "haiku".to_string()],
            output_style: Some("Concise".to_string()),
            language: Some("japanese".to_string()),
            always_thinking_enabled: Some(true),
            attribution_commit: Some("Generated by Claude".to_string()),
            attribution_pr: Some("Created with AI".to_string()),
            sandbox: None,
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &settings).unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(json["model"], "claude-sonnet-4-5-20250929");
        assert_eq!(json["availableModels"][0], "sonnet");
        assert_eq!(json["availableModels"][1], "haiku");
        assert_eq!(json["outputStyle"], "Concise");
        assert_eq!(json["language"], "japanese");
        assert_eq!(json["alwaysThinkingEnabled"], true);
        assert_eq!(json["attribution"]["commit"], "Generated by Claude");
        assert_eq!(json["attribution"]["pr"], "Created with AI");
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

        let settings = ClaudeSettings {
            scope: "local".to_string(),
            model: Some("claude-sonnet-4-5-20250929".to_string()),
            available_models: vec![],
            output_style: None,
            language: None,
            always_thinking_enabled: None,
            attribution_commit: None,
            attribution_pr: None,
            sandbox: None,
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &settings).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        // Hooks preserved
        assert!(json.get("hooks").is_some());
        // Permissions preserved
        assert_eq!(json["permissions"]["deny"][0], "Bash(rm -rf *)");
        // New model setting added
        assert_eq!(json["model"], "claude-sonnet-4-5-20250929");
        // Empty available_models not written
        assert!(json.get("availableModels").is_none());
    }

    #[test]
    fn test_write_removes_keys_when_none() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        // First write with values
        let settings = ClaudeSettings {
            scope: "local".to_string(),
            model: Some("claude-sonnet-4-5-20250929".to_string()),
            available_models: vec!["sonnet".to_string()],
            output_style: Some("Concise".to_string()),
            language: Some("japanese".to_string()),
            always_thinking_enabled: Some(true),
            attribution_commit: Some("test".to_string()),
            attribution_pr: Some("test".to_string()),
            sandbox: None,
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &settings).unwrap();

        // Now write with None values to clear
        let clear_settings = ClaudeSettings {
            scope: "local".to_string(),
            model: None,
            available_models: vec![],
            output_style: None,
            language: None,
            always_thinking_enabled: None,
            attribution_commit: None,
            attribution_pr: None,
            sandbox: None,
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &clear_settings)
            .unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        assert!(json.get("model").is_none());
        assert!(json.get("availableModels").is_none());
        assert!(json.get("outputStyle").is_none());
        assert!(json.get("language").is_none());
        assert!(json.get("alwaysThinkingEnabled").is_none());
        assert!(json.get("attribution").is_none());
    }

    #[test]
    fn test_read_nonexistent_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let settings = read_claude_settings_from_file(&path, "user").unwrap();
        assert!(settings.model.is_none());
        assert!(settings.available_models.is_empty());
    }

    #[test]
    fn test_read_sandbox_settings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(
            &path,
            r#"{
                "sandbox": {
                    "enabled": true,
                    "autoAllowBashIfSandboxed": true,
                    "excludedCommands": ["git", "docker"],
                    "allowUnsandboxedCommands": false,
                    "enableWeakerNestedSandbox": true,
                    "network": {
                        "allowAllUnixSockets": false,
                        "allowUnixSockets": ["/tmp/sock"],
                        "allowLocalBinding": true,
                        "allowedDomains": ["*.example.com", "api.github.com"],
                        "httpProxyPort": 8080,
                        "socksProxyPort": 1080
                    }
                }
            }"#,
        )
        .unwrap();

        let settings = read_claude_settings_from_file(&path, "user").unwrap();
        let sandbox = settings.sandbox.unwrap();
        assert_eq!(sandbox.enabled, Some(true));
        assert_eq!(sandbox.auto_allow_bash_if_sandboxed, Some(true));
        assert_eq!(
            sandbox.excluded_commands,
            Some(vec!["git".to_string(), "docker".to_string()])
        );
        assert_eq!(sandbox.allow_unsandboxed_commands, Some(false));
        assert_eq!(sandbox.enable_weaker_nested_sandbox, Some(true));

        let network = sandbox.network.unwrap();
        assert_eq!(network.allow_all_unix_sockets, Some(false));
        assert_eq!(
            network.allow_unix_sockets,
            Some(vec!["/tmp/sock".to_string()])
        );
        assert_eq!(network.allow_local_binding, Some(true));
        assert_eq!(
            network.allowed_domains,
            Some(vec![
                "*.example.com".to_string(),
                "api.github.com".to_string()
            ])
        );
        assert_eq!(network.http_proxy_port, Some(8080));
        assert_eq!(network.socks_proxy_port, Some(1080));
    }

    #[test]
    fn test_write_sandbox_settings() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        let settings = ClaudeSettings {
            scope: "local".to_string(),
            model: None,
            available_models: vec![],
            output_style: None,
            language: None,
            always_thinking_enabled: None,
            attribution_commit: None,
            attribution_pr: None,
            sandbox: Some(SandboxSettings {
                enabled: Some(true),
                auto_allow_bash_if_sandboxed: Some(true),
                excluded_commands: Some(vec!["git".to_string()]),
                allow_unsandboxed_commands: None,
                enable_weaker_nested_sandbox: None,
                network: Some(SandboxNetworkSettings {
                    allowed_domains: Some(vec!["*.example.com".to_string()]),
                    http_proxy_port: Some(8080),
                    ..Default::default()
                }),
            }),
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &settings).unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(json["sandbox"]["enabled"], true);
        assert_eq!(json["sandbox"]["autoAllowBashIfSandboxed"], true);
        assert_eq!(json["sandbox"]["excludedCommands"][0], "git");
        assert_eq!(json["sandbox"]["network"]["allowedDomains"][0], "*.example.com");
        assert_eq!(json["sandbox"]["network"]["httpProxyPort"], 8080);
        // None fields should not be present
        assert!(json["sandbox"].get("allowUnsandboxedCommands").is_none());
        assert!(json["sandbox"]["network"].get("socksProxyPort").is_none());
    }

    #[test]
    fn test_write_sandbox_preserves_other_keys() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        // Create existing settings with hooks
        let claude_dir = project_path.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        std::fs::write(
            &settings_path,
            r#"{"hooks":{"PreToolUse":[]},"model":"claude-sonnet-4-5-20250929"}"#,
        )
        .unwrap();

        let settings = ClaudeSettings {
            scope: "local".to_string(),
            model: Some("claude-sonnet-4-5-20250929".to_string()),
            available_models: vec![],
            output_style: None,
            language: None,
            always_thinking_enabled: None,
            attribution_commit: None,
            attribution_pr: None,
            sandbox: Some(SandboxSettings {
                enabled: Some(true),
                ..Default::default()
            }),
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &settings).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        // Hooks preserved
        assert!(json.get("hooks").is_some());
        // Model preserved
        assert_eq!(json["model"], "claude-sonnet-4-5-20250929");
        // Sandbox added
        assert_eq!(json["sandbox"]["enabled"], true);
    }

    #[test]
    fn test_write_clears_sandbox_when_none() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        // First write with sandbox
        let settings = ClaudeSettings {
            scope: "local".to_string(),
            model: None,
            available_models: vec![],
            output_style: None,
            language: None,
            always_thinking_enabled: None,
            attribution_commit: None,
            attribution_pr: None,
            sandbox: Some(SandboxSettings {
                enabled: Some(true),
                ..Default::default()
            }),
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &settings).unwrap();

        let path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        assert!(json.get("sandbox").is_some());

        // Now clear sandbox
        let clear_settings = ClaudeSettings {
            scope: "local".to_string(),
            model: None,
            available_models: vec![],
            output_style: None,
            language: None,
            always_thinking_enabled: None,
            attribution_commit: None,
            attribution_pr: None,
            sandbox: None,
        };

        write_claude_settings(&PermissionScope::Local, Some(project_path), &clear_settings)
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        assert!(json.get("sandbox").is_none());
    }
}
