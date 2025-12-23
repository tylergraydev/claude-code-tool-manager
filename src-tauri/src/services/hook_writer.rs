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
                _ => {}
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

/// Remove all hooks from the global settings file
pub fn clear_global_hooks() -> Result<()> {
    write_global_hooks(&[])
}

/// Remove all hooks from a project's settings file
pub fn clear_project_hooks(project_path: &Path) -> Result<()> {
    write_project_hooks(project_path, &[])
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
}
