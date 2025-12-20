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
    String,                         // name
    String,                         // type
    Option<String>,                 // command
    Option<String>,                 // args (JSON)
    Option<String>,                 // url
    Option<String>,                 // headers (JSON)
    Option<String>,                 // env (JSON)
    bool,                           // is_enabled
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
    let projects = claude_json.get_mut("projects").unwrap().as_object_mut().unwrap();

    // Find or create project entry (check both path formats)
    let project_key = if projects.contains_key(project_path) {
        project_path.to_string()
    } else if projects.contains_key(&normalized_path) {
        normalized_path.clone()
    } else {
        // Create new project entry
        projects.insert(normalized_path.clone(), json!({
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
        }));
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
                    if let Ok(headers_val) = serde_json::from_str::<Map<String, Value>>(headers_json) {
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
