use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum McpConfigFile {
    WithServers {
        #[serde(rename = "mcpServers")]
        mcp_servers: HashMap<String, McpConfig>,
    },
    Direct(HashMap<String, McpConfig>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum McpConfig {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
    Remote {
        #[serde(rename = "type")]
        mcp_type: String,
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}

#[derive(Debug)]
pub struct ParsedMcp {
    pub name: String,
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

pub fn parse_mcp_file(path: &Path) -> Result<Vec<ParsedMcp>> {
    let content = std::fs::read_to_string(path)?;
    let config: McpConfigFile = serde_json::from_str(&content)?;

    let servers = match config {
        McpConfigFile::WithServers { mcp_servers } => mcp_servers,
        McpConfigFile::Direct(servers) => servers,
    };

    let mut mcps = Vec::new();

    for (name, config) in servers {
        let mcp = match config {
            McpConfig::Stdio { command, args, env } => ParsedMcp {
                name,
                mcp_type: "stdio".to_string(),
                command: Some(command),
                args: if args.is_empty() { None } else { Some(args) },
                url: None,
                headers: None,
                env: if env.is_empty() { None } else { Some(env) },
            },
            McpConfig::Remote { mcp_type, url, headers } => ParsedMcp {
                name,
                mcp_type: if mcp_type == "sse" { "sse".to_string() } else { "http".to_string() },
                command: None,
                args: None,
                url: Some(url),
                headers: if headers.is_empty() { None } else { Some(headers) },
                env: None,
            },
        };
        mcps.push(mcp);
    }

    Ok(mcps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // =========================================================================
    // parse_mcp_file tests - stdio MCPs
    // =========================================================================

    #[test]
    fn test_parse_stdio_mcp_with_servers_wrapper() {
        let json = r#"{
            "mcpServers": {
                "github": {
                    "command": "npx",
                    "args": ["-y", "@github/mcp-server"],
                    "env": {"GITHUB_TOKEN": "xxx"}
                }
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].name, "github");
        assert_eq!(mcps[0].mcp_type, "stdio");
        assert_eq!(mcps[0].command, Some("npx".to_string()));
        assert_eq!(mcps[0].args, Some(vec!["-y".to_string(), "@github/mcp-server".to_string()]));
        assert!(mcps[0].env.is_some());
    }

    #[test]
    fn test_parse_stdio_mcp_direct_format() {
        let json = r#"{
            "local-server": {
                "command": "node",
                "args": ["server.js"]
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].mcp_type, "stdio");
        assert_eq!(mcps[0].command, Some("node".to_string()));
    }

    #[test]
    fn test_parse_stdio_mcp_minimal() {
        let json = r#"{
            "mcpServers": {
                "simple": {
                    "command": "python"
                }
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0].args, None);
        assert_eq!(mcps[0].env, None);
    }

    // =========================================================================
    // parse_mcp_file tests - remote MCPs (SSE/HTTP)
    // =========================================================================

    #[test]
    fn test_parse_sse_mcp() {
        let json = r#"{
            "mcpServers": {
                "remote": {
                    "type": "sse",
                    "url": "https://mcp.example.com/sse"
                }
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        assert_eq!(mcps[0].mcp_type, "sse");
        assert_eq!(mcps[0].url, Some("https://mcp.example.com/sse".to_string()));
        assert!(mcps[0].command.is_none());
    }

    #[test]
    fn test_parse_http_mcp() {
        let json = r#"{
            "mcpServers": {
                "api": {
                    "type": "http",
                    "url": "https://api.example.com/mcp"
                }
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        assert_eq!(mcps[0].mcp_type, "http");
    }

    #[test]
    fn test_parse_mcp_with_headers() {
        let json = r#"{
            "mcpServers": {
                "authenticated": {
                    "type": "sse",
                    "url": "https://mcp.example.com",
                    "headers": {
                        "Authorization": "Bearer token123",
                        "X-Custom": "value"
                    }
                }
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        let headers = mcps[0].headers.as_ref().unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        assert_eq!(headers.get("X-Custom"), Some(&"value".to_string()));
    }

    // =========================================================================
    // parse_mcp_file tests - multiple MCPs
    // =========================================================================

    #[test]
    fn test_parse_multiple_mcps() {
        let json = r#"{
            "mcpServers": {
                "server1": {
                    "command": "npx",
                    "args": ["-y", "server1"]
                },
                "server2": {
                    "type": "sse",
                    "url": "https://example.com"
                }
            }
        }"#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", json).unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        assert_eq!(mcps.len(), 2);
    }

    // =========================================================================
    // parse_mcp_file tests - error cases
    // =========================================================================

    #[test]
    fn test_parse_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "not valid json").unwrap();

        let result = parse_mcp_file(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_json() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{{}}").unwrap();

        let mcps = parse_mcp_file(file.path()).unwrap();
        assert_eq!(mcps.len(), 0);
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_mcp_file(Path::new("/nonexistent/path/file.json"));
        assert!(result.is_err());
    }

    // =========================================================================
    // McpConfig deserialization tests
    // =========================================================================

    #[test]
    fn test_mcp_config_stdio_deserialization() {
        let json = r#"{"command": "test", "args": ["arg1"], "env": {"KEY": "value"}}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();

        match config {
            McpConfig::Stdio { command, args, env } => {
                assert_eq!(command, "test");
                assert_eq!(args, vec!["arg1"]);
                assert_eq!(env.get("KEY"), Some(&"value".to_string()));
            }
            _ => panic!("Expected Stdio variant"),
        }
    }

    #[test]
    fn test_mcp_config_remote_deserialization() {
        let json = r#"{"type": "sse", "url": "https://example.com"}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();

        match config {
            McpConfig::Remote { mcp_type, url, .. } => {
                assert_eq!(mcp_type, "sse");
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Expected Remote variant"),
        }
    }
}
