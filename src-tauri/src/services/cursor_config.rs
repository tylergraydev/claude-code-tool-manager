use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;

/// Cursor IDE MCP server configuration (STDIO transport)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorMcpStdio {
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    #[serde(default, rename = "envFile")]
    pub env_file: Option<String>,
}

/// Cursor IDE MCP server configuration (HTTP/SSE transport)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorMcpHttp {
    pub url: String,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

/// Cursor MCP config (either STDIO or HTTP)
#[derive(Debug, Clone)]
pub enum CursorMcp {
    Stdio(CursorMcpStdio),
    Http(CursorMcpHttp),
}

/// Parsed MCP from Cursor format (normalized to internal format)
#[derive(Debug)]
pub struct ParsedCursorMcp {
    pub name: String,
    pub mcp_type: String, // "stdio" or "http"
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

/// Cursor mcp.json structure (uses mcpServers key like Claude Code)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CursorMcpConfig {
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Parse Cursor mcp.json file and extract MCP servers
pub fn parse_cursor_mcps(path: &Path) -> Result<Vec<ParsedCursorMcp>> {
    let content = std::fs::read_to_string(path)?;
    let config: CursorMcpConfig = serde_json::from_str(&content)?;

    let mut mcps = Vec::new();

    for (name, server_value) in &config.mcp_servers {
        let parsed = parse_mcp_entry(name, server_value)?;
        mcps.push(parsed);
    }

    Ok(mcps)
}

/// Parse a single MCP entry from the config
fn parse_mcp_entry(name: &str, config: &Value) -> Result<ParsedCursorMcp> {
    let obj = config
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("MCP config entry is not an object"))?;

    // Determine type based on presence of url vs command
    let has_url = obj.contains_key("url");
    let has_command = obj.contains_key("command");

    if has_url {
        // HTTP/SSE transport
        let url = obj.get("url").and_then(|v| v.as_str()).map(String::from);

        // Cursor uses direct headers (like Claude Code)
        let headers = obj.get("headers").and_then(|v| v.as_object()).map(|h| {
            h.iter()
                .filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string())))
                .collect()
        });

        Ok(ParsedCursorMcp {
            name: name.to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: None,
            url,
            headers,
            env: None,
        })
    } else if has_command {
        // STDIO transport
        let command = obj
            .get("command")
            .and_then(|v| v.as_str())
            .map(String::from);

        let args = obj.get("args").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

        let env = obj.get("env").and_then(|v| v.as_object()).map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string())))
                .collect()
        });

        Ok(ParsedCursorMcp {
            name: name.to_string(),
            mcp_type: "stdio".to_string(),
            command,
            args,
            url: None,
            headers: None,
            env,
        })
    } else {
        // Unknown type, default to stdio
        Ok(ParsedCursorMcp {
            name: name.to_string(),
            mcp_type: "stdio".to_string(),
            command: None,
            args: None,
            url: None,
            headers: None,
            env: None,
        })
    }
}

/// MCP tuple for writing (same format used by other editors)
pub type McpTuple = (
    String,         // name
    String,         // type (stdio, sse, http)
    Option<String>, // command
    Option<String>, // args (JSON)
    Option<String>, // url
    Option<String>, // headers (JSON)
    Option<String>, // env (JSON)
);

/// Write MCP servers to Cursor mcp.json, preserving existing content
pub fn write_cursor_config(path: &Path, mcps: &[McpTuple]) -> Result<()> {
    // Read existing config or create new
    let mut config: CursorMcpConfig = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        CursorMcpConfig::default()
    };

    // Clear existing servers (we'll rebuild them)
    config.mcp_servers.clear();

    // Add each MCP
    for mcp in mcps {
        let (name, mcp_type, command, args, url, headers, env) = mcp;

        let mut server_obj = Map::new();

        match mcp_type.as_str() {
            "stdio" => {
                if let Some(cmd) = command {
                    server_obj.insert("command".to_string(), Value::String(cmd.clone()));
                }

                if let Some(args_json) = args {
                    if let Ok(args_vec) = serde_json::from_str::<Vec<String>>(args_json) {
                        server_obj.insert(
                            "args".to_string(),
                            Value::Array(args_vec.into_iter().map(Value::String).collect()),
                        );
                    }
                }

                if let Some(env_json) = env {
                    if let Ok(env_map) = serde_json::from_str::<HashMap<String, String>>(env_json) {
                        let env_obj: Map<String, Value> = env_map
                            .into_iter()
                            .map(|(k, v)| (k, Value::String(v)))
                            .collect();
                        server_obj.insert("env".to_string(), Value::Object(env_obj));
                    }
                }
            }
            "http" | "sse" => {
                if let Some(u) = url {
                    server_obj.insert("url".to_string(), Value::String(u.clone()));
                }

                // Cursor uses direct headers (like Claude Code)
                if let Some(headers_json) = headers {
                    if let Ok(headers_map) =
                        serde_json::from_str::<HashMap<String, String>>(headers_json)
                    {
                        let headers_obj: Map<String, Value> = headers_map
                            .into_iter()
                            .map(|(k, v)| (k, Value::String(v)))
                            .collect();
                        server_obj.insert("headers".to_string(), Value::Object(headers_obj));
                    }
                }
            }
            _ => continue,
        }

        config
            .mcp_servers
            .insert(name.clone(), Value::Object(server_obj));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write back with pretty formatting
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, json)?;

    Ok(())
}

/// Add a single MCP to Cursor config
pub fn add_mcp_to_cursor_config(path: &Path, mcp: &McpTuple) -> Result<()> {
    // Read existing MCPs
    let existing_mcps = if path.exists() {
        parse_cursor_mcps(path)?
    } else {
        Vec::new()
    };

    // Convert existing to tuples
    let mut all_mcps: Vec<McpTuple> = existing_mcps
        .into_iter()
        .map(|m| {
            (
                m.name,
                m.mcp_type,
                m.command,
                m.args
                    .map(|a| serde_json::to_string(&a).unwrap_or_default()),
                m.url,
                m.headers
                    .map(|h| serde_json::to_string(&h).unwrap_or_default()),
                m.env.map(|e| serde_json::to_string(&e).unwrap_or_default()),
            )
        })
        .collect();

    // Remove existing MCP with same name if exists
    all_mcps.retain(|m| m.0 != mcp.0);

    // Add new MCP
    all_mcps.push(mcp.clone());

    // Write all MCPs
    write_cursor_config(path, &all_mcps)
}

/// Remove an MCP from Cursor config
pub fn remove_mcp_from_cursor_config(path: &Path, name: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // Read existing config
    let content = std::fs::read_to_string(path)?;
    let mut config: CursorMcpConfig = serde_json::from_str(&content)?;

    // Remove the MCP from servers
    config.mcp_servers.remove(name);

    // Write back
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(path, json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // =========================================================================
    // parse_cursor_mcps tests
    // =========================================================================

    #[test]
    fn test_parse_cursor_mcps_stdio() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        fs::write(
            &config_path,
            r#"{
                "mcpServers": {
                    "my-server": {
                        "command": "npx",
                        "args": ["-y", "@test/server"],
                        "env": { "API_KEY": "secret" }
                    }
                }
            }"#,
        )
        .unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);

        let mcp = &mcps[0];
        assert_eq!(mcp.name, "my-server");
        assert_eq!(mcp.mcp_type, "stdio");
        assert_eq!(mcp.command, Some("npx".to_string()));
        assert_eq!(
            mcp.args,
            Some(vec!["-y".to_string(), "@test/server".to_string()])
        );
        assert!(mcp.env.is_some());
    }

    #[test]
    fn test_parse_cursor_mcps_http() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        fs::write(
            &config_path,
            r#"{
                "mcpServers": {
                    "github": {
                        "url": "https://api.example.com/mcp",
                        "headers": {
                            "Authorization": "Bearer token123",
                            "X-Custom": "value"
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);

        let mcp = &mcps[0];
        assert_eq!(mcp.name, "github");
        assert_eq!(mcp.mcp_type, "http");
        assert_eq!(mcp.url, Some("https://api.example.com/mcp".to_string()));
        assert!(mcp.headers.is_some());

        let headers = mcp.headers.as_ref().unwrap();
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_cursor_mcps_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        fs::write(
            &config_path,
            r#"{
                "mcpServers": {
                    "local-one": {
                        "command": "node",
                        "args": ["server.js"]
                    },
                    "remote-one": {
                        "url": "https://example.com/mcp"
                    }
                }
            }"#,
        )
        .unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 2);
    }

    #[test]
    fn test_parse_cursor_mcps_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        fs::write(&config_path, "{}").unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert!(mcps.is_empty());
    }

    #[test]
    fn test_parse_cursor_mcps_no_mcp_servers() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        fs::write(
            &config_path,
            r#"{
                "someOtherConfig": "value"
            }"#,
        )
        .unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert!(mcps.is_empty());
    }

    // =========================================================================
    // write_cursor_config tests
    // =========================================================================

    #[test]
    fn test_write_cursor_config_stdio() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        let mcps: Vec<McpTuple> = vec![(
            "test-mcp".to_string(),
            "stdio".to_string(),
            Some("npx".to_string()),
            Some(r#"["-y", "@test/server"]"#.to_string()),
            None,
            None,
            Some(r#"{"API_KEY": "secret"}"#.to_string()),
        )];

        write_cursor_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"test-mcp\""));
        assert!(content.contains("\"command\": \"npx\""));
        assert!(content.contains("mcpServers"));
    }

    #[test]
    fn test_write_cursor_config_http() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        let mcps: Vec<McpTuple> = vec![(
            "http-mcp".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://api.example.com".to_string()),
            Some(r#"{"Authorization": "Bearer token"}"#.to_string()),
            None,
        )];

        write_cursor_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"http-mcp\""));
        assert!(content.contains("\"url\": \"https://api.example.com\""));
        assert!(content.contains("\"headers\""));
        // Cursor uses direct headers, not requestInit
        assert!(!content.contains("requestInit"));
    }

    #[test]
    fn test_write_cursor_config_preserves_other_content() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        // Write existing config with other settings
        fs::write(
            &config_path,
            r#"{
                "mcpServers": {},
                "someOtherSetting": true
            }"#,
        )
        .unwrap();

        let mcps: Vec<McpTuple> = vec![(
            "new-mcp".to_string(),
            "stdio".to_string(),
            Some("python".to_string()),
            None,
            None,
            None,
            None,
        )];

        write_cursor_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        // New MCP added
        assert!(content.contains("\"new-mcp\""));
        // Other setting preserved
        assert!(content.contains("\"someOtherSetting\""));
    }

    #[test]
    fn test_write_cursor_config_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("subdir").join("mcp.json");

        let mcps: Vec<McpTuple> = vec![(
            "test".to_string(),
            "stdio".to_string(),
            Some("node".to_string()),
            None,
            None,
            None,
            None,
        )];

        write_cursor_config(&config_path, &mcps).unwrap();
        assert!(config_path.exists());
    }

    // =========================================================================
    // add_mcp_to_cursor_config tests
    // =========================================================================

    #[test]
    fn test_add_mcp_to_cursor_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        // Add first MCP
        let mcp1: McpTuple = (
            "first".to_string(),
            "stdio".to_string(),
            Some("node".to_string()),
            None,
            None,
            None,
            None,
        );
        add_mcp_to_cursor_config(&config_path, &mcp1).unwrap();

        // Add second MCP
        let mcp2: McpTuple = (
            "second".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://example.com".to_string()),
            None,
            None,
        );
        add_mcp_to_cursor_config(&config_path, &mcp2).unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 2);
    }

    #[test]
    fn test_add_mcp_replaces_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        // Add MCP
        let mcp1: McpTuple = (
            "test".to_string(),
            "stdio".to_string(),
            Some("node".to_string()),
            None,
            None,
            None,
            None,
        );
        add_mcp_to_cursor_config(&config_path, &mcp1).unwrap();

        // Add MCP with same name but different command
        let mcp2: McpTuple = (
            "test".to_string(),
            "stdio".to_string(),
            Some("python".to_string()),
            None,
            None,
            None,
            None,
        );
        add_mcp_to_cursor_config(&config_path, &mcp2).unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].command, Some("python".to_string()));
    }

    // =========================================================================
    // remove_mcp_from_cursor_config tests
    // =========================================================================

    #[test]
    fn test_remove_mcp_from_cursor_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        fs::write(
            &config_path,
            r#"{
                "mcpServers": {
                    "to-remove": { "command": "node" },
                    "to-keep": { "command": "python" }
                }
            }"#,
        )
        .unwrap();

        remove_mcp_from_cursor_config(&config_path, "to-remove").unwrap();

        let mcps = parse_cursor_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].name, "to-keep");
    }

    #[test]
    fn test_remove_mcp_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp.json");

        // Should not error if file does not exist
        remove_mcp_from_cursor_config(&config_path, "nonexistent").unwrap();
    }
}
