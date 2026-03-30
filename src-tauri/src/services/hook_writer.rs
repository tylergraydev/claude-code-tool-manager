use crate::db::models::Hook;
use anyhow::Result;
use directories::BaseDirs;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::Path;

/// Generate the hooks configuration format for settings.json
///
/// Claude Code hooks format:
/// ```json
/// {
///   "hooks": {
///     "PreToolUse": [
///       { "matcher": "Bash", "hooks": [{ "type": "command", "command": "echo test" }] }
///     ],
///     "PostToolUse": [
///       { "hooks": [{ "type": "command", "command": "prettier --write ." }] }
///     ]
///   }
/// }
/// ```
fn generate_hooks_config(hooks: &[Hook]) -> Value {
    // Group hooks by event_type
    let mut by_event: HashMap<String, Vec<&Hook>> = HashMap::new();
    for hook in hooks {
        by_event
            .entry(hook.event_type.clone())
            .or_default()
            .push(hook);
    }

    let mut hooks_obj = Map::new();

    for (event_type, event_hooks) in by_event {
        let mut event_array: Vec<Value> = Vec::new();

        for hook in event_hooks {
            let mut hook_entry = Map::new();

            // Add matcher if present
            if let Some(ref matcher) = hook.matcher {
                if !matcher.is_empty() {
                    hook_entry.insert("matcher".to_string(), json!(matcher));
                }
            }

            // Build the hook action object
            let mut hook_action = Map::new();
            hook_action.insert("type".to_string(), json!(hook.hook_type));

            match hook.hook_type.as_str() {
                "command" => {
                    if let Some(ref cmd) = hook.command {
                        hook_action.insert("command".to_string(), json!(cmd));
                    }
                    if let Some(timeout) = hook.timeout {
                        hook_action.insert("timeout".to_string(), json!(timeout));
                    }
                }
                "prompt" => {
                    if let Some(ref prompt) = hook.prompt {
                        hook_action.insert("prompt".to_string(), json!(prompt));
                    }
                }
                "http" => {
                    if let Some(ref url) = hook.url {
                        hook_action.insert("url".to_string(), json!(url));
                    }
                    if let Some(ref headers) = hook.headers {
                        hook_action.insert("headers".to_string(), headers.clone());
                    }
                    if let Some(ref env_vars) = hook.allowed_env_vars {
                        hook_action.insert("allowedEnvVars".to_string(), json!(env_vars));
                    }
                    if let Some(timeout) = hook.timeout {
                        hook_action.insert("timeout".to_string(), json!(timeout));
                    }
                }
                "agent" => {
                    // agent type has no additional type-specific fields
                }
                _ => {}
            }

            // Universal fields (all hook types)
            if let Some(ref if_cond) = hook.if_condition {
                hook_action.insert("if".to_string(), json!(if_cond));
            }
            if let Some(ref status) = hook.status_message {
                hook_action.insert("statusMessage".to_string(), json!(status));
            }
            if hook.once {
                hook_action.insert("once".to_string(), json!(true));
            }
            if hook.async_mode {
                hook_action.insert("async".to_string(), json!(true));
            }
            if let Some(ref shell) = hook.shell {
                if shell != "bash" {
                    hook_action.insert("shell".to_string(), json!(shell));
                }
            }

            hook_entry.insert("hooks".to_string(), json!([Value::Object(hook_action)]));
            event_array.push(Value::Object(hook_entry));
        }

        hooks_obj.insert(event_type, Value::Array(event_array));
    }

    Value::Object(hooks_obj)
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
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Write hooks to the global settings file (~/.claude/settings.json)
pub fn write_global_hooks(hooks: &[Hook]) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    let settings_path = home.join(".claude").join("settings.json");

    let mut settings = read_settings_file(&settings_path)?;

    if hooks.is_empty() {
        // Remove hooks key if no hooks
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("hooks");
        }
    } else {
        settings["hooks"] = generate_hooks_config(hooks);
    }

    write_settings_file(&settings_path, &settings)
}

/// Write hooks to a project's settings file ({project}/.claude/settings.local.json)
pub fn write_project_hooks(project_path: &Path, hooks: &[Hook]) -> Result<()> {
    let settings_path = project_path.join(".claude").join("settings.local.json");

    let mut settings = read_settings_file(&settings_path)?;

    if hooks.is_empty() {
        // Remove hooks key if no hooks
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("hooks");
        }
    } else {
        settings["hooks"] = generate_hooks_config(hooks);
    }

    write_settings_file(&settings_path, &settings)
}

/// Convert hooks to Claude Code settings.json format for export
/// This returns a serde_json::Value that can be serialized for export
pub fn hooks_to_settings_format(hooks: &[Hook]) -> Value {
    json!({
        "hooks": generate_hooks_config(hooks)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hooks_config() {
        let hooks = vec![
            Hook {
                id: 1,
                name: "format-on-save".to_string(),
                description: Some("Run prettier after file changes".to_string()),
                event_type: "PostToolUse".to_string(),
                matcher: Some("Write|Edit".to_string()),
                hook_type: "command".to_string(),
                command: Some("prettier --write .".to_string()),
                prompt: None,
                timeout: Some(30),
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
            Hook {
                id: 2,
                name: "log-bash".to_string(),
                description: Some("Log bash commands".to_string()),
                event_type: "PreToolUse".to_string(),
                matcher: Some("Bash".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo \"Running bash command\"".to_string()),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
        ];

        let config = generate_hooks_config(&hooks);

        assert!(config.get("PostToolUse").is_some());
        assert!(config.get("PreToolUse").is_some());

        let post_tool = config.get("PostToolUse").unwrap().as_array().unwrap();
        assert_eq!(post_tool.len(), 1);
        assert_eq!(post_tool[0].get("matcher").unwrap(), "Write|Edit");
    }

    #[test]
    fn test_generate_hooks_config_no_matcher() {
        let hooks = vec![Hook {
            id: 1,
            name: "session-greeting".to_string(),
            description: None,
            event_type: "SessionStart".to_string(),
            matcher: None,
            hook_type: "prompt".to_string(),
            command: None,
            prompt: Some("Welcome to the session!".to_string()),
            timeout: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: false,
            async_mode: false,
            shell: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }];

        let config = generate_hooks_config(&hooks);

        let session_start = config.get("SessionStart").unwrap().as_array().unwrap();
        assert_eq!(session_start.len(), 1);
        assert!(session_start[0].get("matcher").is_none());

        let hook_actions = session_start[0].get("hooks").unwrap().as_array().unwrap();
        assert_eq!(hook_actions[0].get("type").unwrap(), "prompt");
        assert_eq!(
            hook_actions[0].get("prompt").unwrap(),
            "Welcome to the session!"
        );
    }

    #[test]
    fn test_multiple_hooks_same_event_type() {
        let hooks = vec![
            Hook {
                id: 1,
                name: "hook-a".to_string(),
                description: None,
                event_type: "PreToolUse".to_string(),
                matcher: Some("Bash".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo a".to_string()),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
            Hook {
                id: 2,
                name: "hook-b".to_string(),
                description: None,
                event_type: "PreToolUse".to_string(),
                matcher: Some("Write".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo b".to_string()),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
        ];

        let config = generate_hooks_config(&hooks);
        let pre_tool = config.get("PreToolUse").unwrap().as_array().unwrap();
        assert_eq!(pre_tool.len(), 2);
    }

    #[test]
    fn test_empty_hooks_produces_empty_object() {
        let hooks: Vec<Hook> = vec![];
        let config = generate_hooks_config(&hooks);
        assert!(config.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_hook_with_timeout() {
        let hooks = vec![Hook {
            id: 1,
            name: "timeout-hook".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: Some("Bash".to_string()),
            hook_type: "command".to_string(),
            command: Some("lint .".to_string()),
            prompt: None,
            timeout: Some(60),
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: false,
            async_mode: false,
            shell: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }];

        let config = generate_hooks_config(&hooks);
        let post_tool = config.get("PostToolUse").unwrap().as_array().unwrap();
        let hook_actions = post_tool[0].get("hooks").unwrap().as_array().unwrap();
        assert_eq!(hook_actions[0].get("timeout").unwrap(), 60);
    }

    #[test]
    fn test_hooks_to_settings_format() {
        let hooks = vec![Hook {
            id: 1,
            name: "test".to_string(),
            description: None,
            event_type: "Stop".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("echo done".to_string()),
            prompt: None,
            timeout: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: false,
            async_mode: false,
            shell: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }];

        let result = hooks_to_settings_format(&hooks);
        assert!(result.get("hooks").is_some());
        assert!(result["hooks"].get("Stop").is_some());
    }

    #[test]
    fn test_write_project_hooks_creates_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path().join("my-project");
        // Directory does not exist yet

        let hooks = vec![Hook {
            id: 1,
            name: "test".to_string(),
            description: None,
            event_type: "PreToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("echo test".to_string()),
            prompt: None,
            timeout: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: false,
            async_mode: false,
            shell: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }];

        let result = write_project_hooks(&project_path, &hooks);
        assert!(result.is_ok());

        let settings_path = project_path.join(".claude").join("settings.local.json");
        assert!(settings_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        assert!(json.get("hooks").is_some());
    }

    #[test]
    fn test_write_project_hooks_preserves_other_settings() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        // Create existing settings file with other keys
        let claude_dir = project_path.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        std::fs::write(
            &settings_path,
            r#"{"permissions":{"allow":["Bash"]},"hooks":{}}"#,
        )
        .unwrap();

        let hooks = vec![Hook {
            id: 1,
            name: "test".to_string(),
            description: None,
            event_type: "Stop".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("echo done".to_string()),
            prompt: None,
            timeout: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: false,
            async_mode: false,
            shell: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }];

        write_project_hooks(project_path, &hooks).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        // Other keys preserved
        assert!(json.get("permissions").is_some());
        // Hooks updated
        assert!(json.get("hooks").is_some());
        assert!(json["hooks"].get("Stop").is_some());
    }

    // =========================================================================
    // Additional coverage
    // =========================================================================

    #[test]
    fn test_write_project_hooks_empty_removes_hooks_key() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();
        let claude_dir = project_path.join(".claude");
        std::fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        std::fs::write(&settings_path, r#"{"hooks":{"Stop":[]},"other":"val"}"#).unwrap();

        let hooks: Vec<Hook> = vec![];
        write_project_hooks(project_path, &hooks).unwrap();

        let content = std::fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        assert!(json.get("hooks").is_none());
        assert_eq!(json["other"], "val");
    }

    #[test]
    fn test_generate_hooks_config_empty_matcher_excluded() {
        let hooks = vec![Hook {
            id: 1,
            name: "empty-matcher".to_string(),
            description: None,
            event_type: "PreToolUse".to_string(),
            matcher: Some("".to_string()), // empty matcher
            hook_type: "command".to_string(),
            command: Some("echo test".to_string()),
            prompt: None,
            timeout: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: false,
            async_mode: false,
            shell: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }];

        let config = generate_hooks_config(&hooks);
        let pre_tool = config.get("PreToolUse").unwrap().as_array().unwrap();
        // Empty matcher should not be included
        assert!(pre_tool[0].get("matcher").is_none());
    }

    #[test]
    fn test_generate_hooks_config_unknown_hook_type() {
        let hooks = vec![Hook {
            id: 1,
            name: "unknown".to_string(),
            description: None,
            event_type: "PreToolUse".to_string(),
            matcher: None,
            hook_type: "unknown_type".to_string(),
            command: None,
            prompt: None,
            timeout: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: false,
            async_mode: false,
            shell: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }];

        let config = generate_hooks_config(&hooks);
        let pre_tool = config.get("PreToolUse").unwrap().as_array().unwrap();
        let hook_actions = pre_tool[0].get("hooks").unwrap().as_array().unwrap();
        assert_eq!(hook_actions[0]["type"], "unknown_type");
        // Should not have command or prompt
        assert!(hook_actions[0].get("command").is_none());
        assert!(hook_actions[0].get("prompt").is_none());
    }

    #[test]
    fn test_read_settings_file_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let val = read_settings_file(&path).unwrap();
        assert_eq!(val, json!({}));
    }

    #[test]
    fn test_read_settings_file_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not valid json").unwrap();
        let val = read_settings_file(&path).unwrap();
        assert_eq!(val, json!({}));
    }

    #[test]
    fn test_write_settings_file_creates_parents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("deep").join("settings.json");
        write_settings_file(&path, &json!({"ok": true})).unwrap();
        assert!(path.exists());
    }

    // =========================================================================
    // Tests for new hook event types
    // =========================================================================

    #[test]
    fn test_generate_hooks_config_new_event_types() {
        let hooks = vec![
            Hook {
                id: 1,
                name: "failure-log".to_string(),
                description: Some("Log tool failures".to_string()),
                event_type: "PostToolUseFailure".to_string(),
                matcher: Some("Bash".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo failed".to_string()),
                prompt: None,
                timeout: Some(5),
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
            Hook {
                id: 2,
                name: "stop-failure-alert".to_string(),
                description: None,
                event_type: "StopFailure".to_string(),
                matcher: Some("rate_limit".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo rate limited".to_string()),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
            Hook {
                id: 3,
                name: "config-watcher".to_string(),
                description: None,
                event_type: "ConfigChange".to_string(),
                matcher: Some("user_settings".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo config changed".to_string()),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
        ];

        let config = generate_hooks_config(&hooks);

        assert!(config.get("PostToolUseFailure").is_some());
        assert!(config.get("StopFailure").is_some());
        assert!(config.get("ConfigChange").is_some());

        let failure = config.get("PostToolUseFailure").unwrap().as_array().unwrap();
        assert_eq!(failure.len(), 1);
        assert_eq!(failure[0].get("matcher").unwrap(), "Bash");

        let stop_fail = config.get("StopFailure").unwrap().as_array().unwrap();
        assert_eq!(stop_fail[0].get("matcher").unwrap(), "rate_limit");
    }

    #[test]
    fn test_generate_hooks_config_all_new_event_types() {
        // Verify all 14 new event types serialize correctly
        let new_event_types = vec![
            "InstructionsLoaded",
            "PostToolUseFailure",
            "StopFailure",
            "SubagentStart",
            "TaskCompleted",
            "TeammateIdle",
            "PostCompact",
            "ConfigChange",
            "CwdChanged",
            "FileChanged",
            "WorktreeCreate",
            "WorktreeRemove",
            "Elicitation",
            "ElicitationResult",
        ];

        let hooks: Vec<Hook> = new_event_types
            .iter()
            .enumerate()
            .map(|(i, event_type)| Hook {
                id: i as i64 + 1,
                name: format!("hook-{}", event_type),
                description: None,
                event_type: event_type.to_string(),
                matcher: None,
                hook_type: "command".to_string(),
                command: Some(format!("echo {}", event_type)),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            })
            .collect();

        let config = generate_hooks_config(&hooks);

        for event_type in &new_event_types {
            assert!(
                config.get(*event_type).is_some(),
                "Missing event type: {}",
                event_type
            );
            let entries = config.get(*event_type).unwrap().as_array().unwrap();
            assert_eq!(entries.len(), 1);
            let hook_actions = entries[0].get("hooks").unwrap().as_array().unwrap();
            assert_eq!(
                hook_actions[0].get("command").unwrap(),
                &format!("echo {}", event_type)
            );
        }
    }

    #[test]
    fn test_write_project_hooks_new_event_types() {
        let dir = tempfile::tempdir().unwrap();
        let project_path = dir.path();

        let hooks = vec![
            Hook {
                id: 1,
                name: "file-watcher".to_string(),
                description: None,
                event_type: "FileChanged".to_string(),
                matcher: Some("*.rs".to_string()),
                hook_type: "command".to_string(),
                command: Some("cargo check".to_string()),
                prompt: None,
                timeout: Some(120),
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
            Hook {
                id: 2,
                name: "worktree-setup".to_string(),
                description: None,
                event_type: "WorktreeCreate".to_string(),
                matcher: None,
                hook_type: "command".to_string(),
                command: Some("npm install".to_string()),
                prompt: None,
                timeout: Some(300),
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
        ];

        write_project_hooks(project_path, &hooks).unwrap();

        let settings_path = project_path.join(".claude").join("settings.local.json");
        let content = std::fs::read_to_string(&settings_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();

        assert!(json["hooks"].get("FileChanged").is_some());
        assert!(json["hooks"].get("WorktreeCreate").is_some());

        let file_changed = json["hooks"]["FileChanged"].as_array().unwrap();
        assert_eq!(file_changed[0]["matcher"], "*.rs");

        let worktree = json["hooks"]["WorktreeCreate"].as_array().unwrap();
        assert!(worktree[0].get("matcher").is_none());
    }

    #[test]
    fn test_hooks_to_settings_format_mixed_old_and_new_events() {
        let hooks = vec![
            Hook {
                id: 1,
                name: "old-event".to_string(),
                description: None,
                event_type: "PreToolUse".to_string(),
                matcher: Some("Bash".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo old".to_string()),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
            Hook {
                id: 2,
                name: "new-event".to_string(),
                description: None,
                event_type: "SubagentStart".to_string(),
                matcher: Some("code-reviewer".to_string()),
                hook_type: "command".to_string(),
                command: Some("echo new".to_string()),
                prompt: None,
                timeout: None,
                url: None,
                headers: None,
                allowed_env_vars: None,
                if_condition: None,
                status_message: None,
                once: false,
                async_mode: false,
                shell: None,
                tags: None,
                source: "manual".to_string(),
                is_template: false,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            },
        ];

        let result = hooks_to_settings_format(&hooks);
        assert!(result["hooks"].get("PreToolUse").is_some());
        assert!(result["hooks"].get("SubagentStart").is_some());
    }
}
