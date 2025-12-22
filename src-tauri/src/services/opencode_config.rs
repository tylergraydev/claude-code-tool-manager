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
