use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a devcontainer.json configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevcontainerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docker_compose_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_ports: Option<Vec<u16>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_create_command: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_start_command: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mounts: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_env: Option<HashMap<String, String>>,
}

impl DevcontainerConfig {
    pub fn parse(json_str: &str) -> Result<Self, String> {
        // Strip JSON comments (// and /* */) before parsing
        let stripped = strip_json_comments(json_str);
        serde_json::from_str(&stripped)
            .map_err(|e| format!("Failed to parse devcontainer.json: {}", e))
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize devcontainer.json: {}", e))
    }

    pub fn get_image(&self) -> Option<&str> {
        self.image.as_deref()
    }

    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        if let Some(ref remote_env) = self.remote_env {
            env.extend(remote_env.clone());
        }
        if let Some(ref container_env) = self.container_env {
            env.extend(container_env.clone());
        }
        env
    }

    pub fn get_ports(&self) -> Vec<u16> {
        self.forward_ports.clone().unwrap_or_default()
    }

    pub fn get_post_create_command_str(&self) -> Option<String> {
        self.post_create_command.as_ref().and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) => {
                let parts: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(" "))
                }
            }
            _ => None,
        })
    }

    pub fn get_post_start_command_str(&self) -> Option<String> {
        self.post_start_command.as_ref().and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) => {
                let parts: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(" "))
                }
            }
            _ => None,
        })
    }
}

fn strip_json_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            result.push(ch);
            escape_next = false;
            continue;
        }
        if in_string {
            result.push(ch);
            if ch == '\\' {
                escape_next = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            result.push(ch);
            continue;
        }
        if ch == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    // Line comment — skip until newline
                    for c in chars.by_ref() {
                        if c == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                    continue;
                } else if next == '*' {
                    // Block comment — skip until */
                    chars.next(); // consume *
                    loop {
                        match chars.next() {
                            Some('*') => {
                                if chars.peek() == Some(&'/') {
                                    chars.next();
                                    break;
                                }
                            }
                            Some('\n') => result.push('\n'),
                            None => break,
                            _ => {}
                        }
                    }
                    continue;
                }
            }
        }
        result.push(ch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // strip_json_comments tests
    // =========================================================================

    #[test]
    fn test_strip_line_comments() {
        let input = r#"{
    // This is a comment
    "name": "test"
}"#;
        let stripped = strip_json_comments(input);
        assert!(!stripped.contains("// This is a comment"));
        assert!(stripped.contains("\"name\": \"test\""));
        // Newline should be preserved
        assert!(stripped.contains('\n'));
    }

    #[test]
    fn test_strip_block_comments() {
        let input = r#"{
    /* block comment */
    "name": "test"
}"#;
        let stripped = strip_json_comments(input);
        assert!(!stripped.contains("block comment"));
        assert!(stripped.contains("\"name\": \"test\""));
    }

    #[test]
    fn test_strip_multiline_block_comments() {
        let input = r#"{
    /* multi
       line
       comment */
    "name": "test"
}"#;
        let stripped = strip_json_comments(input);
        assert!(!stripped.contains("multi"));
        assert!(!stripped.contains("comment"));
        assert!(stripped.contains("\"name\": \"test\""));
        // Newlines inside block comments should be preserved
        let newline_count = stripped.chars().filter(|&c| c == '\n').count();
        assert!(newline_count >= 4); // original has 5 lines
    }

    #[test]
    fn test_strings_containing_slashes_preserved() {
        let input = r#"{"url": "http://example.com"}"#;
        let stripped = strip_json_comments(input);
        assert_eq!(stripped, input);
    }

    #[test]
    fn test_strings_with_escaped_quotes() {
        let input = r#"{"msg": "say \"hello\""}"#;
        let stripped = strip_json_comments(input);
        assert_eq!(stripped, input);
    }

    #[test]
    fn test_strip_no_comments() {
        let input = r#"{"name": "test"}"#;
        let stripped = strip_json_comments(input);
        assert_eq!(stripped, input);
    }

    #[test]
    fn test_strip_line_comment_at_end_of_file_no_newline() {
        let input = r#"{"a": 1}// trailing"#;
        let stripped = strip_json_comments(input);
        assert_eq!(stripped, r#"{"a": 1}"#);
    }

    #[test]
    fn test_strip_unterminated_block_comment() {
        let input = r#"{"a": 1}/* unterminated"#;
        let stripped = strip_json_comments(input);
        assert_eq!(stripped, r#"{"a": 1}"#);
    }

    #[test]
    fn test_strip_slash_not_followed_by_comment() {
        // A lone slash that is not part of // or /* should be preserved
        let input = r#"{"a": 1, "b": 2}"#;
        let stripped = strip_json_comments(input);
        assert_eq!(stripped, input);
    }

    #[test]
    fn test_strip_block_comment_with_star_inside() {
        // Block comment containing a * that isn't followed by /
        let input = r#"{"a": /* * not end * */ 1}"#;
        let stripped = strip_json_comments(input);
        assert_eq!(stripped, r#"{"a":  1}"#);
    }

    // =========================================================================
    // DevcontainerConfig::parse tests
    // =========================================================================

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{
            "name": "my-dev",
            "image": "ubuntu:22.04",
            "forwardPorts": [3000, 8080],
            "remoteEnv": {"FOO": "bar"},
            "containerEnv": {"BAZ": "qux"},
            "postCreateCommand": "echo hello",
            "postStartCommand": "echo started",
            "workingDir": "/workspace",
            "remoteUser": "vscode"
        }"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.name.as_deref(), Some("my-dev"));
        assert_eq!(config.image.as_deref(), Some("ubuntu:22.04"));
        assert_eq!(config.forward_ports, Some(vec![3000, 8080]));
        assert_eq!(
            config.remote_env.as_ref().unwrap().get("FOO").unwrap(),
            "bar"
        );
        assert_eq!(
            config.container_env.as_ref().unwrap().get("BAZ").unwrap(),
            "qux"
        );
        assert_eq!(config.working_dir.as_deref(), Some("/workspace"));
        assert_eq!(config.remote_user.as_deref(), Some("vscode"));
    }

    #[test]
    fn test_parse_with_line_comments() {
        let json = r#"{
            // This is the dev container name
            "name": "commented",
            "image": "node:20"
        }"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.name.as_deref(), Some("commented"));
        assert_eq!(config.image.as_deref(), Some("node:20"));
    }

    #[test]
    fn test_parse_with_block_comments() {
        let json = r#"{
            /* The image to use
               for this container */
            "name": "block",
            "image": "python:3.12"
        }"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.name.as_deref(), Some("block"));
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{ not valid json }"#;
        let result = DevcontainerConfig::parse(json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to parse devcontainer.json"));
    }

    #[test]
    fn test_parse_minimal_json() {
        let json = r#"{}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert!(config.name.is_none());
        assert!(config.image.is_none());
        assert!(config.forward_ports.is_none());
    }

    // =========================================================================
    // to_json_string tests
    // =========================================================================

    #[test]
    fn test_to_json_string_round_trip() {
        let json = r#"{"name":"test","image":"ubuntu:22.04"}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        let output = config.to_json_string().unwrap();
        assert!(output.contains("\"name\": \"test\""));
        assert!(output.contains("\"image\": \"ubuntu:22.04\""));
    }

    #[test]
    fn test_to_json_string_skips_none_fields() {
        let config = DevcontainerConfig::parse("{}").unwrap();
        let output = config.to_json_string().unwrap();
        assert!(!output.contains("name"));
        assert!(!output.contains("image"));
        assert!(!output.contains("forwardPorts"));
    }

    // =========================================================================
    // get_image tests
    // =========================================================================

    #[test]
    fn test_get_image_some() {
        let config = DevcontainerConfig::parse(r#"{"image": "node:20"}"#).unwrap();
        assert_eq!(config.get_image(), Some("node:20"));
    }

    #[test]
    fn test_get_image_none() {
        let config = DevcontainerConfig::parse(r#"{}"#).unwrap();
        assert_eq!(config.get_image(), None);
    }

    // =========================================================================
    // get_env_vars tests
    // =========================================================================

    #[test]
    fn test_get_env_vars_both() {
        let json = r#"{
            "remoteEnv": {"A": "1"},
            "containerEnv": {"B": "2"}
        }"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        let env = config.get_env_vars();
        assert_eq!(env.get("A").unwrap(), "1");
        assert_eq!(env.get("B").unwrap(), "2");
    }

    #[test]
    fn test_get_env_vars_none() {
        let config = DevcontainerConfig::parse(r#"{}"#).unwrap();
        let env = config.get_env_vars();
        assert!(env.is_empty());
    }

    #[test]
    fn test_get_env_vars_container_overrides_remote() {
        let json = r#"{
            "remoteEnv": {"KEY": "remote"},
            "containerEnv": {"KEY": "container"}
        }"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        let env = config.get_env_vars();
        // containerEnv is applied after remoteEnv, so it should win
        assert_eq!(env.get("KEY").unwrap(), "container");
    }

    // =========================================================================
    // get_ports tests
    // =========================================================================

    #[test]
    fn test_get_ports_some() {
        let json = r#"{"forwardPorts": [3000, 5432]}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.get_ports(), vec![3000, 5432]);
    }

    #[test]
    fn test_get_ports_none() {
        let config = DevcontainerConfig::parse(r#"{}"#).unwrap();
        assert_eq!(config.get_ports(), Vec::<u16>::new());
    }

    // =========================================================================
    // get_post_create_command_str tests
    // =========================================================================

    #[test]
    fn test_post_create_command_string() {
        let json = r#"{"postCreateCommand": "npm install"}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(
            config.get_post_create_command_str(),
            Some("npm install".to_string())
        );
    }

    #[test]
    fn test_post_create_command_array() {
        let json = r#"{"postCreateCommand": ["npm", "install"]}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(
            config.get_post_create_command_str(),
            Some("npm install".to_string())
        );
    }

    #[test]
    fn test_post_create_command_empty_array() {
        let json = r#"{"postCreateCommand": []}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.get_post_create_command_str(), None);
    }

    #[test]
    fn test_post_create_command_non_matching() {
        let json = r#"{"postCreateCommand": 42}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.get_post_create_command_str(), None);
    }

    #[test]
    fn test_post_create_command_none() {
        let config = DevcontainerConfig::parse(r#"{}"#).unwrap();
        assert_eq!(config.get_post_create_command_str(), None);
    }

    // =========================================================================
    // get_post_start_command_str tests
    // =========================================================================

    #[test]
    fn test_post_start_command_string() {
        let json = r#"{"postStartCommand": "echo hi"}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(
            config.get_post_start_command_str(),
            Some("echo hi".to_string())
        );
    }

    #[test]
    fn test_post_start_command_array() {
        let json = r#"{"postStartCommand": ["echo", "hi"]}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(
            config.get_post_start_command_str(),
            Some("echo hi".to_string())
        );
    }

    #[test]
    fn test_post_start_command_empty_array() {
        let json = r#"{"postStartCommand": []}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.get_post_start_command_str(), None);
    }

    #[test]
    fn test_post_start_command_non_matching() {
        let json = r#"{"postStartCommand": {"cmd": "echo"}}"#;
        let config = DevcontainerConfig::parse(json).unwrap();
        assert_eq!(config.get_post_start_command_str(), None);
    }

    #[test]
    fn test_post_start_command_none() {
        let config = DevcontainerConfig::parse(r#"{}"#).unwrap();
        assert_eq!(config.get_post_start_command_str(), None);
    }
}
