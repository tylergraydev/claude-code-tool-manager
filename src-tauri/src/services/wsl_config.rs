use crate::utils::wsl;
use anyhow::Result;
use log::info;
use serde_json::{json, Map, Value};

/// MCP tuple type (same as used by other config writers)
pub type McpTuple = (
    String,         // name
    String,         // type (stdio, sse, http)
    Option<String>, // command
    Option<String>, // args (JSON)
    Option<String>, // url
    Option<String>, // headers (JSON)
    Option<String>, // env (JSON)
);

/// Write global MCP config to a WSL distro's ~/.claude.json
pub fn write_wsl_global_config(distro: &str, mcps: &[McpTuple]) -> Result<()> {
    let config_path = "$HOME/.claude.json";

    // Read existing config or start fresh
    let mut claude_json: Value = match wsl::read_wsl_file(distro, config_path) {
        Ok(content) => serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse existing Claude config in WSL distro '{}': {}. \
                 Refusing to overwrite to prevent data loss.",
                distro,
                e
            )
        })?,
        Err(_) => json!({}),
    };

    // Build mcpServers object
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
                Some(Value::Object(obj))
            }
            "sse" => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), json!("sse"));
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
            servers.insert(name.clone(), cfg);
        }
    }

    claude_json["mcpServers"] = Value::Object(servers);

    // Backup existing file
    let _ = wsl::backup_wsl_file(distro, config_path);

    // Write the config
    let content = serde_json::to_string_pretty(&claude_json)?;
    wsl::write_wsl_file(distro, config_path, &content)?;

    info!(
        "[WSL] Wrote global config to distro '{}' with {} MCPs",
        distro,
        mcps.len()
    );

    Ok(())
}

/// Write a command/skill markdown file to a WSL distro
pub fn write_wsl_command_file(
    distro: &str,
    dir: &str,
    filename: &str,
    content: &str,
) -> Result<()> {
    wsl::mkdir_wsl(distro, dir)?;
    let path = format!("{}/{}", dir, filename);
    wsl::write_wsl_file(distro, &path, content)?;
    info!("[WSL] Wrote command file '{}' to distro '{}'", path, distro);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tuple_type_alias() {
        // Verify the type alias works correctly
        let mcp: McpTuple = (
            "test".to_string(),
            "stdio".to_string(),
            Some("npx".to_string()),
            None,
            None,
            None,
            None,
        );
        assert_eq!(mcp.0, "test");
        assert_eq!(mcp.1, "stdio");
    }
}
