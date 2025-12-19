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
