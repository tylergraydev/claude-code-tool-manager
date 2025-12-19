use crate::utils::paths::ClaudePathsInternal;
use anyhow::Result;
use serde_json::{json, Map, Value};
use std::path::Path;

type McpTuple = (
    String,                         // name
    String,                         // type
    Option<String>,                 // command
    Option<String>,                 // args (JSON)
    Option<String>,                 // url
    Option<String>,                 // headers (JSON)
    Option<String>,                 // env (JSON)
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
                    if let Ok(headers_val) = serde_json::from_str::<Map<String, Value>>(headers_json) {
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
    // Read existing settings.json or create new
    let mut settings: Value = if paths.global_settings.exists() {
        let content = std::fs::read_to_string(&paths.global_settings)?;
        serde_json::from_str(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    // Build mcpServers object
    let mcp_config = generate_mcp_config(mcps);
    if let Some(servers) = mcp_config.get("mcpServers") {
        settings["mcpServers"] = servers.clone();
    }

    // Write back
    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&paths.global_settings, content)?;

    Ok(())
}
