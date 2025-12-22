use crate::utils::opencode_paths::{get_opencode_paths, OpenCodePathsInternal};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::Path;

/// OpenCode MCP server configuration (local/stdio)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenCodeMcpLocal {
    #[serde(rename = "type")]
    pub mcp_type: String,  // "local"
    pub command: Vec<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub timeout: Option<i32>,
}

/// OpenCode MCP server configuration (remote/http)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenCodeMcpRemote {
    #[serde(rename = "type")]
    pub mcp_type: String,  // "remote"
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub timeout: Option<i32>,
    #[serde(default)]
    pub oauth: Option<Value>,
}

/// OpenCode MCP config (either local or remote)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OpenCodeMcp {
    Local(OpenCodeMcpLocal),
    Remote(OpenCodeMcpRemote),
}

/// OpenCode configuration file structure
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OpenCodeConfig {
    #[serde(default)]
    pub mcp: HashMap<String, OpenCodeMcp>,
    #[serde(default)]
    pub agent: HashMap<String, Value>,
    #[serde(default)]
    pub command: HashMap<String, Value>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Parsed MCP from OpenCode format
#[derive(Debug)]
pub struct ParsedOpenCodeMcp {
    pub name: String,
    pub mcp_type: String,  // "stdio", "http", or "sse" (normalized to Claude format)
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

/// Parse OpenCode config file
pub fn parse_opencode_config(path: &Path) -> Result<OpenCodeConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: OpenCodeConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Parse MCPs from OpenCode config file
pub fn parse_opencode_mcps(path: &Path) -> Result<Vec<ParsedOpenCodeMcp>> {
    let config = parse_opencode_config(path)?;
    let mut mcps = Vec::new();

    for (name, mcp) in config.mcp {
        let parsed = match mcp {
            OpenCodeMcp::Local(local) => {
                // Convert OpenCode "local" to Claude "stdio"
                let (command, args) = if local.command.is_empty() {
                    (None, None)
                } else {
                    let cmd = local.command[0].clone();
                    let args = if local.command.len() > 1 {
                        Some(local.command[1..].to_vec())
                    } else {
                        None
                    };
                    (Some(cmd), args)
                };

                ParsedOpenCodeMcp {
                    name,
                    mcp_type: "stdio".to_string(),
                    command,
                    args,
                    url: None,
                    headers: None,
                    env: if local.environment.is_empty() {
                        None
                    } else {
                        Some(local.environment)
                    },
                }
            }
            OpenCodeMcp::Remote(remote) => {
                // Convert OpenCode "remote" to Claude "http"
                ParsedOpenCodeMcp {
                    name,
                    mcp_type: "http".to_string(),
                    command: None,
                    args: None,
                    url: Some(remote.url),
                    headers: if remote.headers.is_empty() {
                        None
                    } else {
                        Some(remote.headers)
                    },
                    env: None,
                }
            }
        };
        mcps.push(parsed);
    }

    Ok(mcps)
}

/// MCP tuple for writing (same format as Claude Code)
type McpTuple = (
    String,         // name
    String,         // type (stdio, sse, http)
    Option<String>, // command
    Option<String>, // args (JSON)
    Option<String>, // url
    Option<String>, // headers (JSON)
    Option<String>, // env (JSON)
);

/// Generate OpenCode MCP config object from MCP tuples
pub fn generate_opencode_mcp_config(mcps: &[McpTuple]) -> Value {
    let mut mcp_obj = Map::new();

    for mcp in mcps {
        let (name, mcp_type, command, args, url, headers, env) = mcp;

        let config = match mcp_type.as_str() {
            "stdio" => {
                // Convert Claude "stdio" to OpenCode "local"
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("local"));

                // Build command array from command + args
                let mut cmd_array: Vec<String> = Vec::new();
                if let Some(cmd) = command {
                    cmd_array.push(cmd.clone());
                }
                if let Some(args_json) = args {
                    if let Ok(args_val) = serde_json::from_str::<Vec<String>>(args_json) {
                        cmd_array.extend(args_val);
                    }
                }
                obj.insert("command".to_string(), json!(cmd_array));

                // Environment (OpenCode uses "environment" instead of "env")
                if let Some(env_json) = env {
                    if let Ok(env_val) = serde_json::from_str::<Map<String, Value>>(env_json) {
                        obj.insert("environment".to_string(), Value::Object(env_val));
                    }
                }

                obj.insert("enabled".to_string(), json!(true));
                Value::Object(obj)
            }
            "http" | "sse" => {
                // Convert Claude "http"/"sse" to OpenCode "remote"
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("remote"));

                if let Some(u) = url {
                    obj.insert("url".to_string(), json!(u));
                }

                if let Some(headers_json) = headers {
                    if let Ok(headers_val) = serde_json::from_str::<Map<String, Value>>(headers_json) {
                        obj.insert("headers".to_string(), Value::Object(headers_val));
                    }
                }

                obj.insert("enabled".to_string(), json!(true));
                Value::Object(obj)
            }
            _ => continue,
        };

        mcp_obj.insert(name.clone(), config);
    }

    json!({ "mcp": mcp_obj })
}

/// Write global OpenCode config
pub fn write_opencode_global_config(mcps: &[McpTuple]) -> Result<()> {
    let paths = get_opencode_paths()?;

    // Ensure config directory exists
    std::fs::create_dir_all(&paths.config_dir)?;

    // Read existing config or create new
    let mut config: Value = if paths.config_file.exists() {
        let content = std::fs::read_to_string(&paths.config_file)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    // Build MCP object
    let mcp_config = generate_opencode_mcp_config(mcps);
    if let Some(mcp) = mcp_config.get("mcp") {
        config["mcp"] = mcp.clone();
    }

    // Write back
    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&paths.config_file, content)?;

    Ok(())
}

/// Write project-level OpenCode config
pub fn write_opencode_project_config(project_path: &Path, mcps: &[McpTuple]) -> Result<()> {
    // OpenCode uses opencode.json in project root (not .opencode/opencode.json)
    let config_path = project_path.join("opencode.json");

    // Read existing config or create new
    let mut config: Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    // Build MCP object
    let mcp_config = generate_opencode_mcp_config(mcps);
    if let Some(mcp) = mcp_config.get("mcp") {
        config["mcp"] = mcp.clone();
    }

    // Write back
    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;

    Ok(())
}

/// Get OpenCode paths (re-export for convenience)
pub fn get_paths() -> Result<OpenCodePathsInternal> {
    get_opencode_paths()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    // =========================================================================
    // OpenCodeMcpLocal serde tests
    // =========================================================================

    #[test]
    fn test_opencode_mcp_local_deserialization() {
        let json = r#"{
            "type": "local",
            "command": ["npx", "-y", "@test/mcp"],
            "environment": {"API_KEY": "secret"},
            "enabled": true
        }"#;

        let mcp: OpenCodeMcpLocal = serde_json::from_str(json).unwrap();
        assert_eq!(mcp.mcp_type, "local");
        assert_eq!(mcp.command, vec!["npx", "-y", "@test/mcp"]);
        assert_eq!(mcp.environment.get("API_KEY"), Some(&"secret".to_string()));
        assert!(mcp.enabled);
    }

    #[test]
    fn test_opencode_mcp_local_defaults() {
        let json = r#"{
            "type": "local",
            "command": ["node", "server.js"]
        }"#;

        let mcp: OpenCodeMcpLocal = serde_json::from_str(json).unwrap();
        assert!(mcp.environment.is_empty());
        assert!(!mcp.enabled); // default is false
        assert!(mcp.timeout.is_none());
    }

    #[test]
    fn test_opencode_mcp_local_serialization() {
        let mcp = OpenCodeMcpLocal {
            mcp_type: "local".to_string(),
            command: vec!["python".to_string(), "-m".to_string(), "mcp".to_string()],
            environment: HashMap::from([("DEBUG".to_string(), "true".to_string())]),
            enabled: true,
            timeout: Some(30),
        };

        let json = serde_json::to_string(&mcp).unwrap();
        assert!(json.contains("\"type\":\"local\""));
        assert!(json.contains("\"command\":[\"python\""));
        assert!(json.contains("\"enabled\":true"));
    }

    // =========================================================================
    // OpenCodeMcpRemote serde tests
    // =========================================================================

    #[test]
    fn test_opencode_mcp_remote_deserialization() {
        let json = r#"{
            "type": "remote",
            "url": "https://api.example.com/mcp",
            "headers": {"Authorization": "Bearer token"},
            "enabled": true
        }"#;

        let mcp: OpenCodeMcpRemote = serde_json::from_str(json).unwrap();
        assert_eq!(mcp.mcp_type, "remote");
        assert_eq!(mcp.url, "https://api.example.com/mcp");
        assert_eq!(mcp.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert!(mcp.enabled);
    }

    #[test]
    fn test_opencode_mcp_remote_defaults() {
        let json = r#"{
            "type": "remote",
            "url": "https://mcp.example.com"
        }"#;

        let mcp: OpenCodeMcpRemote = serde_json::from_str(json).unwrap();
        assert!(mcp.headers.is_empty());
        assert!(!mcp.enabled);
        assert!(mcp.timeout.is_none());
        assert!(mcp.oauth.is_none());
    }

    #[test]
    fn test_opencode_mcp_remote_with_oauth() {
        let json = r#"{
            "type": "remote",
            "url": "https://mcp.example.com",
            "oauth": {"client_id": "abc123"}
        }"#;

        let mcp: OpenCodeMcpRemote = serde_json::from_str(json).unwrap();
        assert!(mcp.oauth.is_some());
    }

    // =========================================================================
    // OpenCodeMcp enum tests
    // =========================================================================

    #[test]
    fn test_opencode_mcp_local_variant() {
        let json = r#"{
            "type": "local",
            "command": ["node", "server.js"]
        }"#;

        let mcp: OpenCodeMcp = serde_json::from_str(json).unwrap();
        match mcp {
            OpenCodeMcp::Local(local) => {
                assert_eq!(local.command, vec!["node", "server.js"]);
            }
            _ => panic!("Expected Local variant"),
        }
    }

    #[test]
    fn test_opencode_mcp_remote_variant() {
        let json = r#"{
            "type": "remote",
            "url": "https://mcp.example.com"
        }"#;

        let mcp: OpenCodeMcp = serde_json::from_str(json).unwrap();
        match mcp {
            OpenCodeMcp::Remote(remote) => {
                assert_eq!(remote.url, "https://mcp.example.com");
            }
            _ => panic!("Expected Remote variant"),
        }
    }

    // =========================================================================
    // OpenCodeConfig tests
    // =========================================================================

    #[test]
    fn test_opencode_config_deserialization() {
        let json = r#"{
            "mcp": {
                "filesystem": {
                    "type": "local",
                    "command": ["npx", "-y", "@modelcontextprotocol/server-filesystem"]
                }
            },
            "agent": {},
            "command": {}
        }"#;

        let config: OpenCodeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp.len(), 1);
        assert!(config.mcp.contains_key("filesystem"));
    }

    #[test]
    fn test_opencode_config_empty() {
        let json = r#"{}"#;

        let config: OpenCodeConfig = serde_json::from_str(json).unwrap();
        assert!(config.mcp.is_empty());
        assert!(config.agent.is_empty());
        assert!(config.command.is_empty());
    }

    #[test]
    fn test_opencode_config_preserves_other_fields() {
        let json = r#"{
            "mcp": {},
            "custom_field": "value",
            "another": 123
        }"#;

        let config: OpenCodeConfig = serde_json::from_str(json).unwrap();
        assert!(config.other.contains_key("custom_field"));
        assert!(config.other.contains_key("another"));
    }

    // =========================================================================
    // parse_opencode_config tests
    // =========================================================================

    #[test]
    fn test_parse_opencode_config_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("opencode.json");

        fs::write(&config_path, r#"{
            "mcp": {
                "test-mcp": {
                    "type": "local",
                    "command": ["node", "server.js"]
                }
            }
        }"#).unwrap();

        let config = parse_opencode_config(&config_path).unwrap();
        assert_eq!(config.mcp.len(), 1);
    }

    #[test]
    fn test_parse_opencode_config_nonexistent() {
        let result = parse_opencode_config(Path::new("/nonexistent/opencode.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_opencode_config_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("opencode.json");

        fs::write(&config_path, "not valid json").unwrap();

        let result = parse_opencode_config(&config_path);
        assert!(result.is_err());
    }

    // =========================================================================
    // parse_opencode_mcps tests
    // =========================================================================

    #[test]
    fn test_parse_opencode_mcps_local() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("opencode.json");

        fs::write(&config_path, r#"{
            "mcp": {
                "my-server": {
                    "type": "local",
                    "command": ["npx", "-y", "@test/server"],
                    "environment": {"API_KEY": "secret"}
                }
            }
        }"#).unwrap();

        let mcps = parse_opencode_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);

        let mcp = &mcps[0];
        assert_eq!(mcp.name, "my-server");
        assert_eq!(mcp.mcp_type, "stdio"); // Converted from "local"
        assert_eq!(mcp.command, Some("npx".to_string()));
        assert_eq!(mcp.args, Some(vec!["-y".to_string(), "@test/server".to_string()]));
        assert!(mcp.env.is_some());
    }

    #[test]
    fn test_parse_opencode_mcps_remote() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("opencode.json");

        fs::write(&config_path, r#"{
            "mcp": {
                "remote-server": {
                    "type": "remote",
                    "url": "https://api.example.com/mcp",
                    "headers": {"X-API-Key": "key123"}
                }
            }
        }"#).unwrap();

        let mcps = parse_opencode_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);

        let mcp = &mcps[0];
        assert_eq!(mcp.name, "remote-server");
        assert_eq!(mcp.mcp_type, "http"); // Converted from "remote"
        assert_eq!(mcp.url, Some("https://api.example.com/mcp".to_string()));
        assert!(mcp.headers.is_some());
    }

    #[test]
    fn test_parse_opencode_mcps_empty_command() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("opencode.json");

        fs::write(&config_path, r#"{
            "mcp": {
                "empty": {
                    "type": "local",
                    "command": []
                }
            }
        }"#).unwrap();

        let mcps = parse_opencode_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);
        assert!(mcps[0].command.is_none());
        assert!(mcps[0].args.is_none());
    }

    #[test]
    fn test_parse_opencode_mcps_command_only() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("opencode.json");

        fs::write(&config_path, r#"{
            "mcp": {
                "single": {
                    "type": "local",
                    "command": ["python"]
                }
            }
        }"#).unwrap();

        let mcps = parse_opencode_mcps(&config_path).unwrap();
        assert_eq!(mcps[0].command, Some("python".to_string()));
        assert!(mcps[0].args.is_none()); // No args when only command
    }

    // =========================================================================
    // generate_opencode_mcp_config tests
    // =========================================================================

    #[test]
    fn test_generate_opencode_mcp_config_stdio() {
        let mcps: Vec<McpTuple> = vec![(
            "test-mcp".to_string(),
            "stdio".to_string(),
            Some("npx".to_string()),
            Some(r#"["-y", "@test/server"]"#.to_string()),
            None,
            None,
            Some(r#"{"API_KEY": "secret"}"#.to_string()),
        )];

        let config = generate_opencode_mcp_config(&mcps);
        let mcp = config.get("mcp").unwrap().get("test-mcp").unwrap();

        assert_eq!(mcp.get("type").unwrap(), "local");
        let command = mcp.get("command").unwrap().as_array().unwrap();
        assert_eq!(command[0], "npx");
        assert_eq!(command[1], "-y");
        assert!(mcp.get("environment").is_some());
        assert_eq!(mcp.get("enabled").unwrap(), true);
    }

    #[test]
    fn test_generate_opencode_mcp_config_http() {
        let mcps: Vec<McpTuple> = vec![(
            "http-mcp".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://api.example.com".to_string()),
            Some(r#"{"Authorization": "Bearer token"}"#.to_string()),
            None,
        )];

        let config = generate_opencode_mcp_config(&mcps);
        let mcp = config.get("mcp").unwrap().get("http-mcp").unwrap();

        assert_eq!(mcp.get("type").unwrap(), "remote");
        assert_eq!(mcp.get("url").unwrap(), "https://api.example.com");
        assert!(mcp.get("headers").is_some());
    }

    #[test]
    fn test_generate_opencode_mcp_config_sse() {
        let mcps: Vec<McpTuple> = vec![(
            "sse-mcp".to_string(),
            "sse".to_string(),
            None,
            None,
            Some("https://sse.example.com".to_string()),
            None,
            None,
        )];

        let config = generate_opencode_mcp_config(&mcps);
        let mcp = config.get("mcp").unwrap().get("sse-mcp").unwrap();

        assert_eq!(mcp.get("type").unwrap(), "remote"); // SSE maps to remote
    }

    #[test]
    fn test_generate_opencode_mcp_config_unknown_type_skipped() {
        let mcps: Vec<McpTuple> = vec![(
            "unknown".to_string(),
            "unknown-type".to_string(),
            None,
            None,
            None,
            None,
            None,
        )];

        let config = generate_opencode_mcp_config(&mcps);
        let mcp_obj = config.get("mcp").unwrap().as_object().unwrap();
        assert!(mcp_obj.is_empty());
    }

    #[test]
    fn test_generate_opencode_mcp_config_multiple() {
        let mcps: Vec<McpTuple> = vec![
            (
                "local-one".to_string(),
                "stdio".to_string(),
                Some("node".to_string()),
                None,
                None,
                None,
                None,
            ),
            (
                "remote-one".to_string(),
                "http".to_string(),
                None,
                None,
                Some("https://example.com".to_string()),
                None,
                None,
            ),
        ];

        let config = generate_opencode_mcp_config(&mcps);
        let mcp_obj = config.get("mcp").unwrap().as_object().unwrap();
        assert_eq!(mcp_obj.len(), 2);
    }

    // =========================================================================
    // write_opencode_project_config tests
    // =========================================================================

    #[test]
    fn test_write_opencode_project_config_creates_file() {
        let temp_dir = TempDir::new().unwrap();

        let mcps: Vec<McpTuple> = vec![(
            "test".to_string(),
            "stdio".to_string(),
            Some("node".to_string()),
            None,
            None,
            None,
            None,
        )];

        write_opencode_project_config(temp_dir.path(), &mcps).unwrap();

        let config_path = temp_dir.path().join("opencode.json");
        assert!(config_path.exists());
    }

    #[test]
    fn test_write_opencode_project_config_preserves_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("opencode.json");

        // Write existing config with custom field
        fs::write(&config_path, r#"{"custom": "value", "mcp": {}}"#).unwrap();

        let mcps: Vec<McpTuple> = vec![(
            "new-mcp".to_string(),
            "stdio".to_string(),
            Some("python".to_string()),
            None,
            None,
            None,
            None,
        )];

        write_opencode_project_config(temp_dir.path(), &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let config: Value = serde_json::from_str(&content).unwrap();

        // Custom field preserved
        assert_eq!(config.get("custom").unwrap(), "value");
        // MCP added
        assert!(config.get("mcp").unwrap().get("new-mcp").is_some());
    }

    #[test]
    fn test_write_opencode_project_config_valid_json() {
        let temp_dir = TempDir::new().unwrap();

        let mcps: Vec<McpTuple> = vec![(
            "server".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://api.test.com".to_string()),
            None,
            None,
        )];

        write_opencode_project_config(temp_dir.path(), &mcps).unwrap();

        let config_path = temp_dir.path().join("opencode.json");
        let content = fs::read_to_string(&config_path).unwrap();

        // Should be valid JSON
        let config: Value = serde_json::from_str(&content).unwrap();
        assert!(config.get("mcp").is_some());
    }
}
