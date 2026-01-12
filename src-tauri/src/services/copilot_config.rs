use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::Path;

/// GitHub Copilot CLI MCP server configuration (STDIO transport)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CopilotMcpStdio {
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

/// HTTP request initialization options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CopilotRequestInit {
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

/// GitHub Copilot CLI MCP server configuration (HTTP/SSE transport)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CopilotMcpHttp {
    pub url: String,
    #[serde(default, rename = "type")]
    pub mcp_type: Option<String>,
    #[serde(default, rename = "requestInit")]
    pub request_init: Option<CopilotRequestInit>,
}

/// Copilot MCP config (either STDIO or HTTP)
#[derive(Debug, Clone)]
pub enum CopilotMcp {
    Stdio(CopilotMcpStdio),
    Http(CopilotMcpHttp),
}

/// Parsed MCP from Copilot format (normalized to internal format)
#[derive(Debug)]
pub struct ParsedCopilotMcp {
    pub name: String,
    pub mcp_type: String, // "stdio" or "http"
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

/// Copilot mcp-config.json structure
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CopilotMcpConfig {
    #[serde(default)]
    pub servers: HashMap<String, Value>,
    #[serde(default)]
    pub inputs: Option<Vec<Value>>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Parse Copilot mcp-config.json file and extract MCP servers
pub fn parse_copilot_mcps(path: &Path) -> Result<Vec<ParsedCopilotMcp>> {
    let content = std::fs::read_to_string(path)?;
    let config: CopilotMcpConfig = serde_json::from_str(&content)?;

    let mut mcps = Vec::new();

    for (name, server_value) in &config.servers {
        let parsed = parse_mcp_entry(name, server_value)?;
        mcps.push(parsed);
    }

    Ok(mcps)
}

/// Parse a single MCP entry from the config
fn parse_mcp_entry(name: &str, config: &Value) -> Result<ParsedCopilotMcp> {
    let obj = config
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("MCP config entry is not an object"))?;

    // Determine type based on presence of url vs command
    let has_url = obj.contains_key("url");
    let has_command = obj.contains_key("command");

    if has_url {
        // HTTP/SSE transport
        let url = obj.get("url").and_then(|v| v.as_str()).map(String::from);

        // Handle headers from requestInit
        let mut headers: HashMap<String, String> = HashMap::new();

        if let Some(request_init) = obj.get("requestInit").and_then(|v| v.as_object()) {
            if let Some(h) = request_init.get("headers").and_then(|v| v.as_object()) {
                for (k, v) in h {
                    if let Some(val) = v.as_str() {
                        headers.insert(k.clone(), val.to_string());
                    }
                }
            }
        }

        Ok(ParsedCopilotMcp {
            name: name.to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: None,
            url,
            headers: if headers.is_empty() {
                None
            } else {
                Some(headers)
            },
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

        Ok(ParsedCopilotMcp {
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
        Ok(ParsedCopilotMcp {
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

/// Write MCP servers to Copilot mcp-config.json, preserving existing content
pub fn write_copilot_config(path: &Path, mcps: &[McpTuple]) -> Result<()> {
    // Read existing config or create new
    let mut config: CopilotMcpConfig = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        CopilotMcpConfig::default()
    };

    // Clear existing servers (we'll rebuild them)
    config.servers.clear();

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

                if let Some(headers_json) = headers {
                    if let Ok(headers_map) =
                        serde_json::from_str::<HashMap<String, String>>(headers_json)
                    {
                        let headers_obj: Map<String, Value> = headers_map
                            .into_iter()
                            .map(|(k, v)| (k, Value::String(v)))
                            .collect();

                        let mut request_init = Map::new();
                        request_init.insert("headers".to_string(), Value::Object(headers_obj));
                        server_obj.insert("requestInit".to_string(), Value::Object(request_init));
                    }
                }
            }
            _ => continue,
        }

        config
            .servers
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

/// Add a single MCP to Copilot config
pub fn add_mcp_to_copilot_config(path: &Path, mcp: &McpTuple) -> Result<()> {
    // Read existing MCPs
    let existing_mcps = if path.exists() {
        parse_copilot_mcps(path)?
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
    write_copilot_config(path, &all_mcps)
}

/// Remove an MCP from Copilot config
pub fn remove_mcp_from_copilot_config(path: &Path, name: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // Read existing config
    let content = std::fs::read_to_string(path)?;
    let mut config: CopilotMcpConfig = serde_json::from_str(&content)?;

    // Remove the MCP from servers
    config.servers.remove(name);

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
    // parse_copilot_mcps tests
    // =========================================================================

    #[test]
    fn test_parse_copilot_mcps_stdio() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        fs::write(
            &config_path,
            r#"{
                "servers": {
                    "my-server": {
                        "command": "npx",
                        "args": ["-y", "@test/server"],
                        "env": { "API_KEY": "secret" }
                    }
                }
            }"#,
        )
        .unwrap();

        let mcps = parse_copilot_mcps(&config_path).unwrap();
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
    fn test_parse_copilot_mcps_http() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        fs::write(
            &config_path,
            r#"{
                "servers": {
                    "github": {
                        "url": "https://api.example.com/mcp",
                        "requestInit": {
                            "headers": {
                                "Authorization": "Bearer token123",
                                "X-Custom": "value"
                            }
                        }
                    }
                }
            }"#,
        )
        .unwrap();

        let mcps = parse_copilot_mcps(&config_path).unwrap();
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
    fn test_parse_copilot_mcps_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        fs::write(
            &config_path,
            r#"{
                "servers": {
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

        let mcps = parse_copilot_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 2);
    }

    #[test]
    fn test_parse_copilot_mcps_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        fs::write(&config_path, "{}").unwrap();

        let mcps = parse_copilot_mcps(&config_path).unwrap();
        assert!(mcps.is_empty());
    }

    #[test]
    fn test_parse_copilot_mcps_no_servers() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        fs::write(
            &config_path,
            r#"{
                "inputs": []
            }"#,
        )
        .unwrap();

        let mcps = parse_copilot_mcps(&config_path).unwrap();
        assert!(mcps.is_empty());
    }

    // =========================================================================
    // write_copilot_config tests
    // =========================================================================

    #[test]
    fn test_write_copilot_config_stdio() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        let mcps: Vec<McpTuple> = vec![(
            "test-mcp".to_string(),
            "stdio".to_string(),
            Some("npx".to_string()),
            Some(r#"["-y", "@test/server"]"#.to_string()),
            None,
            None,
            Some(r#"{"API_KEY": "secret"}"#.to_string()),
        )];

        write_copilot_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"test-mcp\""));
        assert!(content.contains("\"command\": \"npx\""));
    }

    #[test]
    fn test_write_copilot_config_http() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        let mcps: Vec<McpTuple> = vec![(
            "http-mcp".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://api.example.com".to_string()),
            Some(r#"{"Authorization": "Bearer token"}"#.to_string()),
            None,
        )];

        write_copilot_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"http-mcp\""));
        assert!(content.contains("\"url\": \"https://api.example.com\""));
        assert!(content.contains("\"requestInit\""));
        assert!(content.contains("\"headers\""));
    }

    #[test]
    fn test_write_copilot_config_preserves_inputs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        // Write existing config with inputs
        fs::write(
            &config_path,
            r#"{
                "servers": {},
                "inputs": [{"type": "test"}]
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

        write_copilot_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        // New MCP added
        assert!(content.contains("\"new-mcp\""));
        // Inputs preserved
        assert!(content.contains("\"inputs\""));
    }

    #[test]
    fn test_write_copilot_config_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("subdir").join("mcp-config.json");

        let mcps: Vec<McpTuple> = vec![(
            "test".to_string(),
            "stdio".to_string(),
            Some("node".to_string()),
            None,
            None,
            None,
            None,
        )];

        write_copilot_config(&config_path, &mcps).unwrap();
        assert!(config_path.exists());
    }

    // =========================================================================
    // add_mcp_to_copilot_config tests
    // =========================================================================

    #[test]
    fn test_add_mcp_to_copilot_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

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
        add_mcp_to_copilot_config(&config_path, &mcp1).unwrap();

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
        add_mcp_to_copilot_config(&config_path, &mcp2).unwrap();

        let mcps = parse_copilot_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 2);
    }

    #[test]
    fn test_add_mcp_replaces_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

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
        add_mcp_to_copilot_config(&config_path, &mcp1).unwrap();

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
        add_mcp_to_copilot_config(&config_path, &mcp2).unwrap();

        let mcps = parse_copilot_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].command, Some("python".to_string()));
    }

    // =========================================================================
    // remove_mcp_from_copilot_config tests
    // =========================================================================

    #[test]
    fn test_remove_mcp_from_copilot_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        fs::write(
            &config_path,
            r#"{
                "servers": {
                    "to-remove": { "command": "node" },
                    "to-keep": { "command": "python" }
                }
            }"#,
        )
        .unwrap();

        remove_mcp_from_copilot_config(&config_path, "to-remove").unwrap();

        let mcps = parse_copilot_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].name, "to-keep");
    }

    #[test]
    fn test_remove_mcp_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mcp-config.json");

        // Should not error if file does not exist
        remove_mcp_from_copilot_config(&config_path, "nonexistent").unwrap();
    }
}
