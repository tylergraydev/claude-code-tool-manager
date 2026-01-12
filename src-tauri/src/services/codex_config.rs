use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use toml_edit::{value, Array, DocumentMut, InlineTable, Item, Table};

/// Codex MCP server configuration (STDIO transport)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CodexMcpStdio {
    #[serde(default)]
    pub enabled: Option<bool>,
    pub command: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub startup_timeout_sec: Option<f64>,
    #[serde(default)]
    pub tool_timeout_sec: Option<f64>,
    #[serde(default)]
    pub enabled_tools: Option<Vec<String>>,
    #[serde(default)]
    pub disabled_tools: Option<Vec<String>>,
}

/// Codex MCP server configuration (HTTP transport)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CodexMcpHttp {
    #[serde(default)]
    pub enabled: Option<bool>,
    pub url: String,
    #[serde(default)]
    pub bearer_token_env_var: Option<String>,
    #[serde(default)]
    pub http_headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub env_http_headers: Option<HashMap<String, String>>,
}

/// Codex MCP config (either STDIO or HTTP)
#[derive(Debug, Clone)]
pub enum CodexMcp {
    Stdio(CodexMcpStdio),
    Http(CodexMcpHttp),
}

/// Parsed MCP from Codex format (normalized to internal format)
#[derive(Debug)]
pub struct ParsedCodexMcp {
    pub name: String,
    pub mcp_type: String, // "stdio" or "http"
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

/// Parse Codex config.toml file and extract MCP servers
pub fn parse_codex_mcps(path: &Path) -> Result<Vec<ParsedCodexMcp>> {
    let content = std::fs::read_to_string(path)?;
    let doc: toml::Value = toml::from_str(&content)?;

    let mut mcps = Vec::new();

    // Get mcp_servers table
    if let Some(mcp_servers) = doc.get("mcp_servers").and_then(|v| v.as_table()) {
        for (name, config) in mcp_servers {
            let parsed = parse_mcp_entry(name, config)?;
            mcps.push(parsed);
        }
    }

    Ok(mcps)
}

/// Parse a single MCP entry from the config
fn parse_mcp_entry(name: &str, config: &toml::Value) -> Result<ParsedCodexMcp> {
    // Determine type based on presence of url vs command
    let has_url = config.get("url").is_some();
    let has_command = config.get("command").is_some();

    if has_url {
        // HTTP transport
        let url = config.get("url").and_then(|v| v.as_str()).map(String::from);

        // Handle headers - combine http_headers and bearer_token_env_var
        let mut headers: HashMap<String, String> = HashMap::new();

        if let Some(http_headers) = config.get("http_headers").and_then(|v| v.as_table()) {
            for (k, v) in http_headers {
                if let Some(val) = v.as_str() {
                    headers.insert(k.clone(), val.to_string());
                }
            }
        }

        // If bearer_token_env_var is set, add it as Authorization header reference
        if let Some(bearer_var) = config.get("bearer_token_env_var").and_then(|v| v.as_str()) {
            headers.insert("Authorization".to_string(), format!("${}", bearer_var));
        }

        Ok(ParsedCodexMcp {
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
        let command = config
            .get("command")
            .and_then(|v| v.as_str())
            .map(String::from);

        let args = config.get("args").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

        let env = config.get("env").and_then(|v| v.as_table()).map(|table| {
            table
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|val| (k.clone(), val.to_string())))
                .collect()
        });

        Ok(ParsedCodexMcp {
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
        Ok(ParsedCodexMcp {
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

/// Write MCP servers to Codex config.toml, preserving existing content
pub fn write_codex_config(path: &Path, mcps: &[McpTuple]) -> Result<()> {
    // Read existing config or create new
    let content = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };

    let mut doc: DocumentMut = content.parse().unwrap_or_default();

    // Create or get mcp_servers table
    if doc.get("mcp_servers").is_none() {
        doc["mcp_servers"] = Item::Table(Table::new());
    }

    let mcp_servers = doc["mcp_servers"].as_table_mut().unwrap();

    // Clear existing MCP entries (we'll rebuild them)
    mcp_servers.clear();

    // Add each MCP
    for mcp in mcps {
        let (name, mcp_type, command, args, url, headers, env) = mcp;

        let mut server_table = Table::new();

        match mcp_type.as_str() {
            "stdio" => {
                server_table.insert("enabled", value(true));

                if let Some(cmd) = command {
                    server_table.insert("command", value(cmd.as_str()));
                }

                if let Some(args_json) = args {
                    if let Ok(args_vec) = serde_json::from_str::<Vec<String>>(args_json) {
                        let mut arr = Array::new();
                        for arg in args_vec {
                            arr.push(arg);
                        }
                        server_table.insert("args", value(arr));
                    }
                }

                if let Some(env_json) = env {
                    if let Ok(env_map) = serde_json::from_str::<HashMap<String, String>>(env_json) {
                        let mut inline = InlineTable::new();
                        for (k, v) in env_map {
                            inline.insert(&k, v.into());
                        }
                        server_table.insert("env", value(inline));
                    }
                }
            }
            "http" | "sse" => {
                server_table.insert("enabled", value(true));

                if let Some(u) = url {
                    server_table.insert("url", value(u.as_str()));
                }

                if let Some(headers_json) = headers {
                    if let Ok(headers_map) =
                        serde_json::from_str::<HashMap<String, String>>(headers_json)
                    {
                        // Check for Authorization header with env var reference
                        let mut bearer_var: Option<String> = None;
                        let mut http_headers = InlineTable::new();

                        for (k, v) in &headers_map {
                            if k == "Authorization" && v.starts_with('$') {
                                // Convert $ENV_VAR to bearer_token_env_var
                                bearer_var = Some(v.trim_start_matches('$').to_string());
                            } else {
                                http_headers.insert(k, v.clone().into());
                            }
                        }

                        if let Some(var) = bearer_var {
                            server_table.insert("bearer_token_env_var", value(var));
                        }

                        if !http_headers.is_empty() {
                            server_table.insert("http_headers", value(http_headers));
                        }
                    }
                }
            }
            _ => continue,
        }

        mcp_servers.insert(name, Item::Table(server_table));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write back
    std::fs::write(path, doc.to_string())?;

    Ok(())
}

/// Add a single MCP to Codex config
pub fn add_mcp_to_codex_config(path: &Path, mcp: &McpTuple) -> Result<()> {
    // Read existing MCPs
    let existing_mcps = if path.exists() {
        parse_codex_mcps(path)?
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
    write_codex_config(path, &all_mcps)
}

/// Remove an MCP from Codex config
pub fn remove_mcp_from_codex_config(path: &Path, name: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // Read existing config
    let content = std::fs::read_to_string(path)?;
    let mut doc: DocumentMut = content.parse()?;

    // Remove the MCP from mcp_servers table
    if let Some(mcp_servers) = doc.get_mut("mcp_servers").and_then(|v| v.as_table_mut()) {
        mcp_servers.remove(name);
    }

    // Write back
    std::fs::write(path, doc.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // =========================================================================
    // parse_codex_mcps tests
    // =========================================================================

    #[test]
    fn test_parse_codex_mcps_stdio() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
[mcp_servers.my-server]
enabled = true
command = "npx"
args = ["-y", "@test/server"]
env = { API_KEY = "secret" }
"#,
        )
        .unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
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
    fn test_parse_codex_mcps_http() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
[mcp_servers.github]
enabled = true
url = "https://api.example.com/mcp"
bearer_token_env_var = "GITHUB_TOKEN"
http_headers = { "X-Custom" = "value" }
"#,
        )
        .unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);

        let mcp = &mcps[0];
        assert_eq!(mcp.name, "github");
        assert_eq!(mcp.mcp_type, "http");
        assert_eq!(mcp.url, Some("https://api.example.com/mcp".to_string()));
        assert!(mcp.headers.is_some());

        let headers = mcp.headers.as_ref().unwrap();
        assert_eq!(
            headers.get("Authorization"),
            Some(&"$GITHUB_TOKEN".to_string())
        );
        assert_eq!(headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_codex_mcps_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
[mcp_servers.local-one]
command = "node"
args = ["server.js"]

[mcp_servers.remote-one]
url = "https://example.com/mcp"
"#,
        )
        .unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 2);
    }

    #[test]
    fn test_parse_codex_mcps_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(&config_path, "").unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
        assert!(mcps.is_empty());
    }

    #[test]
    fn test_parse_codex_mcps_no_mcp_servers() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
model = "gpt-5-codex"
approval_policy = "on-request"
"#,
        )
        .unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
        assert!(mcps.is_empty());
    }

    // =========================================================================
    // write_codex_config tests
    // =========================================================================

    #[test]
    fn test_write_codex_config_stdio() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mcps: Vec<McpTuple> = vec![(
            "test-mcp".to_string(),
            "stdio".to_string(),
            Some("npx".to_string()),
            Some(r#"["-y", "@test/server"]"#.to_string()),
            None,
            None,
            Some(r#"{"API_KEY": "secret"}"#.to_string()),
        )];

        write_codex_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[mcp_servers.test-mcp]"));
        assert!(content.contains("command = \"npx\""));
        assert!(content.contains("enabled = true"));
    }

    #[test]
    fn test_write_codex_config_http() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mcps: Vec<McpTuple> = vec![(
            "http-mcp".to_string(),
            "http".to_string(),
            None,
            None,
            Some("https://api.example.com".to_string()),
            Some(r#"{"Authorization": "$GITHUB_TOKEN", "X-Custom": "value"}"#.to_string()),
            None,
        )];

        write_codex_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("[mcp_servers.http-mcp]"));
        assert!(content.contains("url = \"https://api.example.com\""));
        assert!(content.contains("bearer_token_env_var = \"GITHUB_TOKEN\""));
    }

    #[test]
    fn test_write_codex_config_preserves_other_content() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write existing config with other settings
        fs::write(
            &config_path,
            r#"
model = "gpt-5-codex"
approval_policy = "on-request"

[features]
shell_tool = true
"#,
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

        write_codex_config(&config_path, &mcps).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        // Existing content preserved
        assert!(content.contains("model = \"gpt-5-codex\""));
        assert!(content.contains("approval_policy = \"on-request\""));
        assert!(content.contains("[features]"));
        // New MCP added
        assert!(content.contains("[mcp_servers.new-mcp]"));
    }

    #[test]
    fn test_write_codex_config_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("subdir").join("config.toml");

        let mcps: Vec<McpTuple> = vec![(
            "test".to_string(),
            "stdio".to_string(),
            Some("node".to_string()),
            None,
            None,
            None,
            None,
        )];

        write_codex_config(&config_path, &mcps).unwrap();
        assert!(config_path.exists());
    }

    // =========================================================================
    // add_mcp_to_codex_config tests
    // =========================================================================

    #[test]
    fn test_add_mcp_to_codex_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

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
        add_mcp_to_codex_config(&config_path, &mcp1).unwrap();

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
        add_mcp_to_codex_config(&config_path, &mcp2).unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 2);
    }

    #[test]
    fn test_add_mcp_replaces_existing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

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
        add_mcp_to_codex_config(&config_path, &mcp1).unwrap();

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
        add_mcp_to_codex_config(&config_path, &mcp2).unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].command, Some("python".to_string()));
    }

    // =========================================================================
    // remove_mcp_from_codex_config tests
    // =========================================================================

    #[test]
    fn test_remove_mcp_from_codex_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(
            &config_path,
            r#"
[mcp_servers.to-remove]
command = "node"

[mcp_servers.to-keep]
command = "python"
"#,
        )
        .unwrap();

        remove_mcp_from_codex_config(&config_path, "to-remove").unwrap();

        let mcps = parse_codex_mcps(&config_path).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].name, "to-keep");
    }

    #[test]
    fn test_remove_mcp_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Should not error if file does not exist
        remove_mcp_from_codex_config(&config_path, "nonexistent").unwrap();
    }
}
