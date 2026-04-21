use crate::utils::paths::ClaudePathsInternal;
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::path::Path;

use crate::utils::backup::backup_file as backup_config_file;

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
                // SSE supports headers per official spec
                if let Some(headers_json) = headers {
                    if let Ok(headers_val) =
                        serde_json::from_str::<Map<String, Value>>(headers_json)
                    {
                        obj.insert("headers".to_string(), Value::Object(headers_val));
                    }
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
            "ws" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("ws"));
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
    let config_path = project_path.join(".mcp.json");

    // Read existing .mcp.json or create new
    let mut existing: Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse existing .mcp.json at {}: {}. \
                 Refusing to overwrite to prevent data loss.",
                config_path.display(),
                e
            )
        })?
    } else {
        json!({})
    };

    // Merge DB-managed mcpServers into existing config
    // Skip overwrite when DB has no servers — preserves externally-managed .mcp.json
    let mcp_config = generate_mcp_config(mcps);
    if let Some(Value::Object(servers)) = mcp_config.get("mcpServers") {
        if !servers.is_empty() {
            existing["mcpServers"] = Value::Object(servers.clone());
        }
    }

    // Back up existing file before writing
    backup_config_file(&config_path)?;

    let content = serde_json::to_string_pretty(&existing)?;
    std::fs::write(&config_path, content)?;
    Ok(())
}

pub fn write_global_config(paths: &ClaudePathsInternal, mcps: &[McpTuple]) -> Result<()> {
    // Read existing ~/.claude.json or create new
    let mut claude_json: Value = if paths.claude_json.exists() {
        let content = std::fs::read_to_string(&paths.claude_json)?;
        serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse existing Claude config at {}: {}. \
                 Refusing to overwrite to prevent data loss.",
                paths.claude_json.display(),
                e
            )
        })?
    } else {
        json!({})
    };

    // Build mcpServers object
    // Skip overwrite when DB has no servers — preserves externally-managed config
    let mcp_config = generate_mcp_config(mcps);
    if let Some(Value::Object(servers)) = mcp_config.get("mcpServers") {
        if !servers.is_empty() {
            claude_json["mcpServers"] = Value::Object(servers.clone());
        }
    }

    // Back up the existing file before modifying it
    backup_config_file(&paths.claude_json)?;

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
        serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse existing Claude config at {}: {}. \
                 Refusing to overwrite to prevent data loss.",
                paths.claude_json.display(),
                e
            )
        })?
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
                // stdio servers don't have explicit type field per official spec
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
                Some(Value::Object(obj))
            }
            "sse" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("sse"));
                if let Some(u) = url {
                    obj.insert("url".to_string(), json!(u));
                }
                // SSE supports headers per official spec
                if let Some(headers_json) = headers {
                    if let Ok(headers_val) =
                        serde_json::from_str::<Map<String, Value>>(headers_json)
                    {
                        obj.insert("headers".to_string(), Value::Object(headers_val));
                    }
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

    // Only update mcpServers if DB has servers — preserves externally-managed configs
    if !mcp_servers.is_empty() {
        project["mcpServers"] = Value::Object(mcp_servers);
    }
    if !disabled_mcps.is_empty() {
        project["disabledMcpServers"] = json!(disabled_mcps);
    }

    // Back up the existing file before modifying it
    backup_config_file(&paths.claude_json)?;

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
            None, // no headers for basic test
            None,
        )
    }

    fn sample_sse_mcp_with_headers() -> McpTuple {
        (
            "remote-sse-auth".to_string(),
            "sse".to_string(),
            None,
            None,
            Some("https://mcp.example.com/sse".to_string()),
            Some(r#"{"Authorization": "Bearer sse-token"}"#.to_string()),
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
    fn test_generate_mcp_config_sse_with_headers() {
        let mcps = vec![sample_sse_mcp_with_headers()];
        let config = generate_mcp_config(&mcps);

        let servers = config.get("mcpServers").unwrap().as_object().unwrap();
        let mcp = servers.get("remote-sse-auth").unwrap();

        assert_eq!(mcp.get("type").unwrap(), "sse");
        assert_eq!(mcp.get("url").unwrap(), "https://mcp.example.com/sse");
        assert!(mcp.get("headers").is_some());
        let headers = mcp.get("headers").unwrap().as_object().unwrap();
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer sse-token");
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
    fn test_write_project_config_creates_file_in_project_root() {
        let temp_dir = TempDir::new().unwrap();
        let mcps = vec![sample_stdio_mcp()];

        write_project_config(temp_dir.path(), &mcps).unwrap();

        // Per official spec, .mcp.json should be in project root, not .claude/
        let config_path = temp_dir.path().join(".mcp.json");
        assert!(config_path.exists());
    }

    #[test]
    fn test_write_project_config_content_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let mcps = vec![sample_stdio_mcp()];

        write_project_config(temp_dir.path(), &mcps).unwrap();

        let config_path = temp_dir.path().join(".mcp.json");
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
        let config_path = temp_dir.path().join(".mcp.json");
        let content = std::fs::read_to_string(config_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();

        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("remote-sse"));
    }

    #[test]
    fn test_write_project_config_preserves_existing_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".mcp.json");

        // Pre-populate .mcp.json with extra top-level keys
        let existing = serde_json::json!({
            "mcpServers": {
                "old-server": { "command": "old", "args": [] }
            },
            "customKey": "should-survive"
        });
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        // Sync with new MCPs
        let mcps = vec![sample_stdio_mcp()];
        write_project_config(temp_dir.path(), &mcps).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        // mcpServers should be replaced with DB MCPs
        let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
        assert!(servers.contains_key("test-mcp"));
        assert!(!servers.contains_key("old-server"));

        // Other top-level keys should be preserved
        assert_eq!(parsed.get("customKey").unwrap(), "should-survive");
    }

    #[test]
    fn test_write_project_config_creates_backup() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".mcp.json");
        let backup_path = temp_dir.path().join(".mcp.json.bak");

        // Pre-populate .mcp.json
        std::fs::write(&config_path, r#"{"mcpServers": {}}"#).unwrap();

        let mcps = vec![sample_stdio_mcp()];
        write_project_config(temp_dir.path(), &mcps).unwrap();

        assert!(backup_path.exists());
    }

    #[test]
    fn test_write_project_config_refuses_corrupt_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".mcp.json");

        // Write invalid JSON
        std::fs::write(&config_path, "not valid json {{{").unwrap();

        let mcps = vec![sample_stdio_mcp()];
        let result = write_project_config(temp_dir.path(), &mcps);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Refusing to overwrite"));
    }

    #[test]
    fn test_write_project_config_empty_mcps_preserves_existing_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".mcp.json");

        // Pre-populate with content
        let existing = serde_json::json!({
            "mcpServers": {
                "external-server": { "command": "ext", "args": [] }
            },
            "someOtherConfig": true
        });
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .unwrap();

        // Sync with empty MCPs (the original bug scenario)
        let mcps: Vec<McpTuple> = vec![];
        write_project_config(temp_dir.path(), &mcps).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        // mcpServers should be preserved (DB has none, so we don't overwrite)
        let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("external-server"));

        // Other keys should also be preserved
        assert_eq!(parsed.get("someOtherConfig").unwrap(), true);
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

    // =========================================================================
    // backup_config_file tests
    // =========================================================================
    #[test]
    fn test_backup_config_file_nonexistent_succeeds() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");
        // Should succeed (no-op) for nonexistent file
        let result = backup_config_file(&path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_backup_config_file_creates_bak() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"test": true}"#).unwrap();

        backup_config_file(&path).unwrap();

        let bak_path = path.with_extension("json.bak");
        assert!(bak_path.exists());
        let content = std::fs::read_to_string(&bak_path).unwrap();
        assert!(content.contains("test"));
    }

    // =========================================================================
    // write_global_config tests
    // =========================================================================
    #[test]
    fn test_write_global_config_new_file() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        let mcps = vec![sample_stdio_mcp()];
        write_global_config(&paths, &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("mcpServers").is_some());
    }

    #[test]
    fn test_write_global_config_preserves_existing() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        // Write existing config
        std::fs::write(
            &paths.claude_json,
            r#"{"projects": {}, "otherKey": "preserved"}"#,
        )
        .unwrap();

        let mcps = vec![sample_stdio_mcp()];
        write_global_config(&paths, &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["otherKey"], "preserved");
        assert!(parsed.get("mcpServers").is_some());
    }

    #[test]
    fn test_write_global_config_invalid_existing_json_errors() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "not valid json").unwrap();

        let mcps = vec![sample_stdio_mcp()];
        let result = write_global_config(&paths, &mcps);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Refusing to overwrite"));
    }

    // =========================================================================
    // write_project_to_claude_json tests
    // =========================================================================
    #[test]
    fn test_write_project_to_claude_json_creates_new_project() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "{}").unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "test-mcp".to_string(),
            "stdio".to_string(),
            Some("npx".to_string()),
            Some(r#"["-y", "mcp"]"#.to_string()),
            None,
            None,
            Some(r#"{"KEY": "val"}"#.to_string()),
            true,
        )];

        write_project_to_claude_json(&paths, "/tmp/project", &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("projects").is_some());
    }

    #[test]
    fn test_write_project_to_claude_json_disabled_mcp() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "{}").unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "disabled-mcp".to_string(),
            "stdio".to_string(),
            Some("cmd".to_string()),
            None,
            None,
            None,
            None,
            false, // disabled
        )];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        // Find the project and check disabledMcpServers
        let projects = parsed.get("projects").unwrap().as_object().unwrap();
        let project = projects.values().next().unwrap();
        let disabled = project["disabledMcpServers"].as_array().unwrap();
        assert!(disabled.iter().any(|v| v == "disabled-mcp"));
    }

    #[test]
    fn test_write_project_to_claude_json_sse_type() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "{}").unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "sse-mcp".to_string(),
            "sse".to_string(),
            None,
            None,
            Some("https://example.com/sse".to_string()),
            Some(r#"{"Auth": "Bearer x"}"#.to_string()),
            None,
            true,
        )];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let projects = parsed.get("projects").unwrap().as_object().unwrap();
        let project = projects.values().next().unwrap();
        let servers = project["mcpServers"].as_object().unwrap();
        let mcp = servers.get("sse-mcp").unwrap();
        assert_eq!(mcp["type"], "sse");
        assert!(mcp.get("headers").is_some());
    }

    #[test]
    fn test_write_project_to_claude_json_http_type() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "{}").unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "http-mcp".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://example.com/api".to_string()),
            Some(r#"{"Auth": "Bearer x"}"#.to_string()),
            None,
            true,
        )];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let projects = parsed.get("projects").unwrap().as_object().unwrap();
        let project = projects.values().next().unwrap();
        let servers = project["mcpServers"].as_object().unwrap();
        let mcp = servers.get("http-mcp").unwrap();
        assert_eq!(mcp["type"], "http");
    }

    #[test]
    fn test_write_project_to_claude_json_unknown_type_skipped() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "{}").unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "weird".to_string(),
            "unknown_type".to_string(),
            None,
            None,
            None,
            None,
            None,
            true,
        )];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let projects = parsed.get("projects").unwrap().as_object().unwrap();
        let project = projects.values().next().unwrap();
        let servers = project["mcpServers"].as_object().unwrap();
        assert!(servers.is_empty());
    }

    #[test]
    fn test_write_project_to_claude_json_no_existing_file() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        // No file exists, should create new
        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "mcp".to_string(),
            "stdio".to_string(),
            Some("cmd".to_string()),
            None,
            None,
            None,
            None,
            true,
        )];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();
        assert!(paths.claude_json.exists());
    }

    // =========================================================================
    // generate_mcp_config edge cases
    // =========================================================================
    #[test]
    fn test_generate_mcp_config_stdio_invalid_args_json() {
        let mcp: McpTuple = (
            "bad-args".to_string(),
            "stdio".to_string(),
            Some("cmd".to_string()),
            Some("not valid json".to_string()),
            None,
            None,
            None,
        );
        let config = generate_mcp_config(&[mcp]);
        let servers = config["mcpServers"].as_object().unwrap();
        let m = servers.get("bad-args").unwrap();
        // args should be absent since parsing failed
        assert!(m.get("args").is_none());
    }

    #[test]
    fn test_generate_mcp_config_stdio_invalid_env_json() {
        let mcp: McpTuple = (
            "bad-env".to_string(),
            "stdio".to_string(),
            Some("cmd".to_string()),
            None,
            None,
            None,
            Some("not json".to_string()),
        );
        let config = generate_mcp_config(&[mcp]);
        let servers = config["mcpServers"].as_object().unwrap();
        let m = servers.get("bad-env").unwrap();
        assert!(m.get("env").is_none());
    }

    #[test]
    fn test_generate_mcp_config_sse_invalid_headers_json() {
        let mcp: McpTuple = (
            "bad-headers".to_string(),
            "sse".to_string(),
            None,
            None,
            Some("https://example.com".to_string()),
            Some("not json".to_string()),
            None,
        );
        let config = generate_mcp_config(&[mcp]);
        let servers = config["mcpServers"].as_object().unwrap();
        let m = servers.get("bad-headers").unwrap();
        assert!(m.get("headers").is_none());
    }

    #[test]
    fn test_generate_mcp_config_http_invalid_headers_json() {
        let mcp: McpTuple = (
            "bad-headers".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://example.com".to_string()),
            Some("not json".to_string()),
            None,
        );
        let config = generate_mcp_config(&[mcp]);
        let servers = config["mcpServers"].as_object().unwrap();
        let m = servers.get("bad-headers").unwrap();
        assert!(m.get("headers").is_none());
    }

    #[test]
    fn test_generate_mcp_config_http_no_url() {
        let mcp: McpTuple = (
            "no-url".to_string(),
            "http".to_string(),
            None,
            None,
            None, // no url
            None,
            None,
        );
        let config = generate_mcp_config(&[mcp]);
        let servers = config["mcpServers"].as_object().unwrap();
        let m = servers.get("no-url").unwrap();
        assert_eq!(m["type"], "http");
        assert!(m.get("url").is_none());
    }

    // =========================================================================
    // Additional coverage: write_project_to_claude_json with existing project
    // =========================================================================

    #[test]
    fn test_write_project_to_claude_json_updates_existing_project() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        // Create existing project with an MCP
        std::fs::write(
            &paths.claude_json,
            r#"{
                "projects": {
                    "/tmp/proj": {
                        "mcpServers": {"old-mcp": {"command": "old"}},
                        "allowedTools": ["Read"],
                        "hasTrustDialogAccepted": true
                    }
                }
            }"#,
        )
        .unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "new-mcp".to_string(),
            "stdio".to_string(),
            Some("new-cmd".to_string()),
            None,
            None,
            None,
            None,
            true,
        )];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let project = &parsed["projects"]["/tmp/proj"];

        // MCPs should be replaced
        let servers = project["mcpServers"].as_object().unwrap();
        assert!(servers.contains_key("new-mcp"));
        // Other fields should be preserved
        assert_eq!(project["hasTrustDialogAccepted"], true);
    }

    #[test]
    fn test_write_project_to_claude_json_creates_backup() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, r#"{"original": true}"#).unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "mcp".to_string(),
            "stdio".to_string(),
            Some("cmd".to_string()),
            None,
            None,
            None,
            None,
            true,
        )];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();

        // Backup should exist
        let bak_path = paths.claude_json.with_extension("json.bak");
        assert!(bak_path.exists());
        let bak_content = std::fs::read_to_string(&bak_path).unwrap();
        assert!(bak_content.contains("original"));
    }

    #[test]
    fn test_write_project_to_claude_json_multiple_mcps_mixed_enabled() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "{}").unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![
            (
                "mcp-a".to_string(),
                "stdio".to_string(),
                Some("cmd-a".to_string()),
                None,
                None,
                None,
                None,
                true,
            ),
            (
                "mcp-b".to_string(),
                "stdio".to_string(),
                Some("cmd-b".to_string()),
                None,
                None,
                None,
                None,
                false,
            ),
            (
                "mcp-c".to_string(),
                "sse".to_string(),
                None,
                None,
                Some("https://example.com".to_string()),
                None,
                None,
                true,
            ),
        ];

        write_project_to_claude_json(&paths, "/tmp/proj", &mcps).unwrap();

        let content = std::fs::read_to_string(&paths.claude_json).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let projects = parsed.get("projects").unwrap().as_object().unwrap();
        let project = projects.values().next().unwrap();
        let servers = project["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 3);

        let disabled = project["disabledMcpServers"].as_array().unwrap();
        assert_eq!(disabled.len(), 1);
        assert_eq!(disabled[0], "mcp-b");
    }

    // =========================================================================
    // Additional coverage: write_global_config creates backup
    // =========================================================================

    #[test]
    fn test_write_global_config_creates_backup() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, r#"{"existing": true}"#).unwrap();

        let mcps = vec![sample_stdio_mcp()];
        write_global_config(&paths, &mcps).unwrap();

        let bak_path = paths.claude_json.with_extension("json.bak");
        assert!(bak_path.exists());
    }

    // =========================================================================
    // Additional coverage: generate_mcp_config with all None fields
    // =========================================================================

    #[test]
    fn test_generate_mcp_config_stdio_no_command() {
        let mcp: McpTuple = (
            "no-cmd".to_string(),
            "stdio".to_string(),
            None, // no command
            None,
            None,
            None,
            None,
        );
        let config = generate_mcp_config(&[mcp]);
        let servers = config["mcpServers"].as_object().unwrap();
        let m = servers.get("no-cmd").unwrap();
        assert!(m.get("command").is_none());
    }

    #[test]
    fn test_generate_mcp_config_sse_no_url() {
        let mcp: McpTuple = (
            "no-url-sse".to_string(),
            "sse".to_string(),
            None,
            None,
            None, // no url
            None,
            None,
        );
        let config = generate_mcp_config(&[mcp]);
        let servers = config["mcpServers"].as_object().unwrap();
        let m = servers.get("no-url-sse").unwrap();
        assert_eq!(m["type"], "sse");
        assert!(m.get("url").is_none());
    }

    // =========================================================================
    // Additional coverage: write_project_config with multiple mcps
    // =========================================================================

    #[test]
    fn test_write_project_config_multiple_types() {
        let temp_dir = TempDir::new().unwrap();
        let mcps = vec![sample_stdio_mcp(), sample_sse_mcp(), sample_http_mcp()];

        write_project_config(temp_dir.path(), &mcps).unwrap();

        let config_path = temp_dir.path().join(".mcp.json");
        let content = std::fs::read_to_string(config_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let servers = parsed["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 3);
    }

    // =========================================================================
    // Additional coverage: write_project_to_claude_json invalid existing JSON
    // =========================================================================

    #[test]
    fn test_write_project_to_claude_json_invalid_existing_json() {
        let dir = TempDir::new().unwrap();
        let paths = ClaudePathsInternal {
            home: dir.path().to_path_buf(),
            claude_json: dir.path().join("claude.json"),
            claude_dir: dir.path().to_path_buf(),
            global_settings: dir.path().join("settings.json"),
            plugins_dir: dir.path().join("plugins"),
            marketplaces_dir: dir.path().join("plugins").join("marketplaces"),
            commands_dir: dir.path().join("commands"),
            skills_dir: dir.path().join("skills"),
            agents_dir: dir.path().join("agents"),
        };

        std::fs::write(&paths.claude_json, "not valid json").unwrap();

        let mcps: Vec<McpWithEnabledTuple> = vec![(
            "mcp".to_string(),
            "stdio".to_string(),
            Some("cmd".to_string()),
            None,
            None,
            None,
            None,
            true,
        )];

        let result = write_project_to_claude_json(&paths, "/tmp/proj", &mcps);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Refusing to overwrite"));
    }
}
