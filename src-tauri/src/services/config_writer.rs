use crate::utils::paths::ClaudePathsInternal;
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::path::Path;

type McpTuple = (
    String,         // name
    String,         // type
    Option<String>, // command
    Option<String>, // args (JSON)
    Option<String>, // url
    Option<String>, // headers (JSON)
    Option<String>, // env (JSON)
);

pub fn generate_mcp_config(mcps: &[McpTuple]) -> Value {
    let mut servers = Map::new();

    for mcp in mcps {
        let (name, mcp_type, command, args, url, headers, env) = mcp;

        let config = match mcp_type.as_str() {
            "stdio" => {
                let mut obj = Map::new();
                if let Some(cmd) = command {
                    obj.insert("command".to_string(), json!(cmd));
                }
                if let Some(args_json) = args {
                    if let Ok(args_val) = serde_json::from_str::<Vec<String>>(args_json) {
                        obj.insert("args".to_string(), json!(args_val));
                    }
                }
                if let Some(env_json) = env {
                    if let Ok(env_val) = serde_json::from_str::<Map<String, Value>>(env_json) {
                        obj.insert("env".to_string(), Value::Object(env_val));
                    }
                }
                Value::Object(obj)
            }
            "sse" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("sse"));
                if let Some(u) = url {
                    obj.insert("url".to_string(), json!(u));
                }
                Value::Object(obj)
            }
            "http" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("http"));
                if let Some(u) = url {
                    obj.insert("url".to_string(), json!(u));
                }
                if let Some(headers_json) = headers {
                    if let Ok(headers_val) =
                        serde_json::from_str::<Map<String, Value>>(headers_json)
                    {
                        obj.insert("headers".to_string(), Value::Object(headers_val));
                    }
                }
                Value::Object(obj)
            }
            _ => continue,
        };

        servers.insert(name.clone(), config);
    }

    json!({ "mcpServers": servers })
}

pub fn write_project_config(project_path: &Path, mcps: &[McpTuple]) -> Result<()> {
    let claude_dir = project_path.join(".claude");
    std::fs::create_dir_all(&claude_dir)?;

    let config_path = claude_dir.join(".mcp.json");
    let config = generate_mcp_config(mcps);
    let content = serde_json::to_string_pretty(&config)?;

    std::fs::write(config_path, content)?;
    Ok(())
}

pub fn write_global_config(paths: &ClaudePathsInternal, mcps: &[McpTuple]) -> Result<()> {
    // Read existing ~/.claude.json or create new
    let mut claude_json: Value = if paths.claude_json.exists() {
        let content = std::fs::read_to_string(&paths.claude_json)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    // Build mcpServers object
    let mcp_config = generate_mcp_config(mcps);
    if let Some(servers) = mcp_config.get("mcpServers") {
        claude_json["mcpServers"] = servers.clone();
    }

    // Write back to ~/.claude.json
    let content = serde_json::to_string_pretty(&claude_json)?;
    std::fs::write(&paths.claude_json, content)?;

    Ok(())
}

/// Tuple for MCP with enabled state for claude.json
pub type McpWithEnabledTuple = (
    String,         // name
    String,         // type
    Option<String>, // command
    Option<String>, // args (JSON)
    Option<String>, // url
    Option<String>, // headers (JSON)
    Option<String>, // env (JSON)
    bool,           // is_enabled
);

/// Write project MCPs to claude.json (the main Claude Code config)
pub fn write_project_to_claude_json(
    paths: &ClaudePathsInternal,
    project_path: &str,
    mcps: &[McpWithEnabledTuple],
) -> Result<()> {
    use crate::utils::paths::normalize_path;

    // Read existing claude.json
    let mut claude_json: Value = if paths.claude_json.exists() {
        let content = std::fs::read_to_string(&paths.claude_json)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    // Ensure projects object exists
    if claude_json.get("projects").is_none() {
        claude_json["projects"] = json!({});
    }

    let normalized_path = normalize_path(project_path);
    let projects = claude_json
        .get_mut("projects")
        .unwrap()
        .as_object_mut()
        .unwrap();

    // Find or create project entry (check both path formats)
    let project_key = if projects.contains_key(project_path) {
        project_path.to_string()
    } else if projects.contains_key(&normalized_path) {
        normalized_path.clone()
    } else {
        // Create new project entry
        projects.insert(
            normalized_path.clone(),
            json!({
                "allowedTools": [],
                "mcpContextUris": [],
                "mcpServers": {},
                "enabledMcpjsonServers": [],
                "disabledMcpjsonServers": [],
                "disabledMcpServers": [],
                "hasTrustDialogAccepted": false,
                "projectOnboardingSeenCount": 0,
                "hasClaudeMdExternalIncludesApproved": false,
                "hasClaudeMdExternalIncludesWarningShown": false,
                "exampleFiles": []
            }),
        );
        normalized_path
    };

    let project = projects.get_mut(&project_key).unwrap();

    // Build mcpServers and disabledMcpServers
    let mut mcp_servers = Map::new();
    let mut disabled_mcps: Vec<String> = Vec::new();

    for (name, mcp_type, command, args, url, headers, env, is_enabled) in mcps {
        let config = match mcp_type.as_str() {
            "stdio" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("stdio"));
                if let Some(cmd) = command {
                    obj.insert("command".to_string(), json!(cmd));
                }
                if let Some(args_json) = args {
                    if let Ok(args_val) = serde_json::from_str::<Vec<String>>(args_json) {
                        obj.insert("args".to_string(), json!(args_val));
                    }
                }
                if let Some(env_json) = env {
                    if let Ok(env_val) = serde_json::from_str::<Map<String, Value>>(env_json) {
                        obj.insert("env".to_string(), Value::Object(env_val));
                    }
                }
                Some(Value::Object(obj))
            }
            "sse" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("sse"));
                if let Some(u) = url {
                    obj.insert("url".to_string(), json!(u));
                }
                Some(Value::Object(obj))
            }
            "http" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("http"));
                if let Some(u) = url {
                    obj.insert("url".to_string(), json!(u));
                }
                if let Some(headers_json) = headers {
                    if let Ok(headers_val) =
                        serde_json::from_str::<Map<String, Value>>(headers_json)
                    {
                        obj.insert("headers".to_string(), Value::Object(headers_val));
                    }
                }
                Some(Value::Object(obj))
            }
            _ => None,
        };

        if let Some(cfg) = config {
            mcp_servers.insert(name.clone(), cfg);
            if !*is_enabled {
                disabled_mcps.push(name.clone());
            }
        }
    }

    project["mcpServers"] = Value::Object(mcp_servers);
    project["disabledMcpServers"] = json!(disabled_mcps);

    // Write back
    let content = serde_json::to_string_pretty(&claude_json)?;
    std::fs::write(&paths.claude_json, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;
    use tempfile::TempDir;

    // =========================================================================
    // Helper functions
    // =========================================================================

    fn sample_stdio_mcp() -> McpTuple {
        (
            "test-mcp".to_string(),
            "stdio".to_string(),
            Some("npx".to_string()),
            Some(r#"["-y", "@test/mcp-server"]"#.to_string()),
            None,
            None,
            Some(r#"{"API_KEY": "test123"}"#.to_string()),
        )
    }

    fn sample_sse_mcp() -> McpTuple {
        (
            "remote-sse".to_string(),
            "sse".to_string(),
            None,
            None,
            Some("https://mcp.example.com/sse".to_string()),
            None,
            None,
        )
    }

    fn sample_http_mcp() -> McpTuple {
        (
            "remote-http".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://api.example.com/mcp".to_string()),
            Some(r#"{"Authorization": "Bearer token123"}"#.to_string()),
            None,
        )
    }

    // =========================================================================
    // generate_mcp_config tests
    // =========================================================================

    #[test]
    fn test_generate_mcp_config_stdio() {
        let mcps = vec![sample_stdio_mcp()];
        let config = generate_mcp_config(&mcps);
        assert_json_snapshot!(config);
    }

    #[test]
    fn test_generate_mcp_config_sse() {
        let mcps = vec![sample_sse_mcp()];
        let config = generate_mcp_config(&mcps);
        assert_json_snapshot!(config);
    }

    #[test]
    fn test_generate_mcp_config_http_with_headers() {
        let mcps = vec![sample_http_mcp()];
        let config = generate_mcp_config(&mcps);
        assert_json_snapshot!(config);
    }

    #[test]
    fn test_generate_mcp_config_multiple() {
        let mcps = vec![sample_stdio_mcp(), sample_sse_mcp(), sample_http_mcp()];
        let config = generate_mcp_config(&mcps);

        let servers = config.get("mcpServers").unwrap().as_object().unwrap();
        assert_eq!(servers.len(), 3);
        assert!(servers.contains_key("test-mcp"));
        assert!(servers.contains_key("remote-sse"));
        assert!(servers.contains_key("remote-http"));
    }

    #[test]
    fn test_generate_mcp_config_empty() {
        let mcps: Vec<McpTuple> = vec![];
        let config = generate_mcp_config(&mcps);
        assert_json_snapshot!(config);
    }

    #[test]
    fn test_generate_mcp_config_stdio_minimal() {
        let mcp: McpTuple = (
            "minimal".to_string(),
            "stdio".to_string(),
            Some("python".to_string()),
            None, // no args
            None,
            None,
            None, // no env
        );
        let config = generate_mcp_config(&vec![mcp]);
        assert_json_snapshot!(config);
    }

    #[test]
    fn test_generate_mcp_config_invalid_type_skipped() {
        let mcp: McpTuple = (
            "invalid".to_string(),
            "unknown-type".to_string(),
            None,
            None,
            None,
            None,
            None,
        );
        let config = generate_mcp_config(&vec![mcp]);
        let servers = config.get("mcpServers").unwrap().as_object().unwrap();
        assert_eq!(servers.len(), 0);
    }

    // =========================================================================
    // write_project_config tests
    // =========================================================================

    #[test]
    fn test_write_project_config_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mcps = vec![sample_stdio_mcp()];

        write_project_config(temp_dir.path(), &mcps).unwrap();

        let claude_dir = temp_dir.path().join(".claude");
        assert!(claude_dir.exists());
        assert!(claude_dir.is_dir());
    }

    #[test]
    fn test_write_project_config_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let mcps = vec![sample_stdio_mcp()];

        write_project_config(temp_dir.path(), &mcps).unwrap();

        let config_path = temp_dir.path().join(".claude").join(".mcp.json");
        assert!(config_path.exists());
    }

    #[test]
    fn test_write_project_config_content_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let mcps = vec![sample_stdio_mcp()];

        write_project_config(temp_dir.path(), &mcps).unwrap();

        let config_path = temp_dir.path().join(".claude").join(".mcp.json");
        let content = std::fs::read_to_string(config_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.get("mcpServers").is_some());
    }

    #[test]
    fn test_write_project_config_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();

        // Write first config
        let mcps1 = vec![sample_stdio_mcp()];
        write_project_config(temp_dir.path(), &mcps1).unwrap();

        // Write second config
        let mcps2 = vec![sample_sse_mcp()];
        write_project_config(temp_dir.path(), &mcps2).unwrap();

        // Verify second config is written
        let config_path = temp_dir.path().join(".claude").join(".mcp.json");
        let content = std::fs::read_to_string(config_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();

        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("remote-sse"));
    }

    // =========================================================================
    // JSON structure tests
    // =========================================================================

    #[test]
    fn test_stdio_config_structure() {
        let mcps = vec![sample_stdio_mcp()];
        let config = generate_mcp_config(&mcps);

        let servers = config.get("mcpServers").unwrap().as_object().unwrap();
        let mcp = servers.get("test-mcp").unwrap();

        assert_eq!(mcp.get("command").unwrap(), "npx");
        assert!(mcp.get("args").is_some());
        assert!(mcp.get("env").is_some());
        // stdio type shouldn't have explicit type field
        assert!(mcp.get("type").is_none());
    }

    #[test]
    fn test_sse_config_structure() {
        let mcps = vec![sample_sse_mcp()];
        let config = generate_mcp_config(&mcps);

        let servers = config.get("mcpServers").unwrap().as_object().unwrap();
        let mcp = servers.get("remote-sse").unwrap();

        assert_eq!(mcp.get("type").unwrap(), "sse");
        assert_eq!(mcp.get("url").unwrap(), "https://mcp.example.com/sse");
    }

    #[test]
    fn test_http_config_structure() {
        let mcps = vec![sample_http_mcp()];
        let config = generate_mcp_config(&mcps);

        let servers = config.get("mcpServers").unwrap().as_object().unwrap();
        let mcp = servers.get("remote-http").unwrap();

        assert_eq!(mcp.get("type").unwrap(), "http");
        assert!(mcp.get("headers").is_some());
    }
}
