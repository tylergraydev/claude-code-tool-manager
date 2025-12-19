use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use crate::utils::paths::{get_claude_paths, normalize_path};

/// MCP server configuration as stored in claude.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeJsonMcpServer {
    #[serde(rename = "type")]
    pub mcp_type: String,

    // stdio fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    // http/sse fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    // common
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// Project-specific settings in claude.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeJsonProject {
    #[serde(default)]
    pub mcp_servers: HashMap<String, ClaudeJsonMcpServer>,

    #[serde(default)]
    pub disabled_mcp_servers: Vec<String>,

    #[serde(default)]
    pub enabled_mcpjson_servers: Vec<String>,

    #[serde(default)]
    pub disabled_mcpjson_servers: Vec<String>,

    // Preserve other fields we don't care about
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Represents the detected MCP from claude.json
#[derive(Debug, Clone)]
pub struct DetectedMcp {
    pub name: String,
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
    pub project_path: Option<String>,  // None for global MCPs
    pub is_enabled: bool,
}

/// Read the claude.json file
pub fn read_claude_json() -> Result<Value> {
    let paths = get_claude_paths()?;
    let content = fs::read_to_string(&paths.claude_json)
        .with_context(|| format!("Failed to read {:?}", paths.claude_json))?;
    let json: Value = serde_json::from_str(&content)?;
    Ok(json)
}

/// Write to claude.json file (preserving all other fields)
pub fn write_claude_json(json: &Value) -> Result<()> {
    let paths = get_claude_paths()?;
    let content = serde_json::to_string_pretty(json)?;
    fs::write(&paths.claude_json, content)?;
    Ok(())
}

/// Get all MCPs from claude.json (both global and project-specific)
pub fn get_all_mcps_from_claude_json() -> Result<Vec<DetectedMcp>> {
    let json = read_claude_json()?;
    let mut mcps = Vec::new();

    // Get global MCPs
    if let Some(global_servers) = json.get("mcpServers").and_then(|v| v.as_object()) {
        for (name, server_value) in global_servers {
            if let Ok(server) = serde_json::from_value::<ClaudeJsonMcpServer>(server_value.clone()) {
                mcps.push(DetectedMcp {
                    name: name.clone(),
                    mcp_type: server.mcp_type,
                    command: server.command,
                    args: server.args,
                    url: server.url,
                    headers: server.headers,
                    env: server.env,
                    project_path: None,
                    is_enabled: true,  // Global MCPs are enabled by default
                });
            }
        }
    }

    // Get project-specific MCPs
    if let Some(projects) = json.get("projects").and_then(|v| v.as_object()) {
        for (project_path, project_value) in projects {
            if let Ok(project) = serde_json::from_value::<ClaudeJsonProject>(project_value.clone()) {
                for (name, server) in project.mcp_servers {
                    let is_enabled = !project.disabled_mcp_servers.contains(&name);
                    mcps.push(DetectedMcp {
                        name: name.clone(),
                        mcp_type: server.mcp_type,
                        command: server.command,
                        args: server.args,
                        url: server.url,
                        headers: server.headers,
                        env: server.env,
                        project_path: Some(project_path.clone()),
                        is_enabled,
                    });
                }
            }
        }
    }

    Ok(mcps)
}

/// Get ALL projects from claude.json (including those without MCPs)
pub fn get_all_projects_from_claude_json() -> Result<Vec<(String, ClaudeJsonProject)>> {
    let json = read_claude_json()?;
    let mut projects = Vec::new();

    if let Some(projects_obj) = json.get("projects").and_then(|v| v.as_object()) {
        for (path, value) in projects_obj {
            if let Ok(project) = serde_json::from_value::<ClaudeJsonProject>(value.clone()) {
                projects.push((path.clone(), project));
            }
        }
    }

    Ok(projects)
}

/// Get projects with MCPs from claude.json (only those that have mcpServers)
pub fn get_projects_from_claude_json() -> Result<Vec<(String, ClaudeJsonProject)>> {
    let json = read_claude_json()?;
    let mut projects = Vec::new();

    if let Some(projects_obj) = json.get("projects").and_then(|v| v.as_object()) {
        for (path, value) in projects_obj {
            if let Ok(project) = serde_json::from_value::<ClaudeJsonProject>(value.clone()) {
                // Only include projects that have MCPs
                if !project.mcp_servers.is_empty() {
                    projects.push((path.clone(), project));
                }
            }
        }
    }

    Ok(projects)
}

/// Add an MCP to a project in claude.json
pub fn add_mcp_to_project_in_claude_json(
    project_path: &str,
    mcp_name: &str,
    server: ClaudeJsonMcpServer,
) -> Result<()> {
    let mut json = read_claude_json()?;
    let normalized_path = normalize_path(project_path);

    // Ensure projects object exists
    if json.get("projects").is_none() {
        json["projects"] = serde_json::json!({});
    }

    // Find or create project entry (check both path formats)
    let projects = json.get_mut("projects").unwrap().as_object_mut().unwrap();

    let project_key = if projects.contains_key(project_path) {
        project_path.to_string()
    } else if projects.contains_key(&normalized_path) {
        normalized_path
    } else {
        // Create new project entry
        projects.insert(normalized_path.clone(), serde_json::json!({
            "allowedTools": [],
            "mcpContextUris": [],
            "mcpServers": {},
            "enabledMcpjsonServers": [],
            "disabledMcpjsonServers": [],
            "hasTrustDialogAccepted": false,
            "projectOnboardingSeenCount": 0,
            "hasClaudeMdExternalIncludesApproved": false,
            "hasClaudeMdExternalIncludesWarningShown": false,
            "exampleFiles": []
        }));
        normalized_path
    };

    // Add the MCP server
    let project = projects.get_mut(&project_key).unwrap();
    if project.get("mcpServers").is_none() {
        project["mcpServers"] = serde_json::json!({});
    }
    project["mcpServers"][mcp_name] = serde_json::to_value(server)?;

    write_claude_json(&json)?;
    Ok(())
}

/// Remove an MCP from a project in claude.json
pub fn remove_mcp_from_project_in_claude_json(project_path: &str, mcp_name: &str) -> Result<()> {
    let mut json = read_claude_json()?;
    let normalized_path = normalize_path(project_path);

    if let Some(projects) = json.get_mut("projects").and_then(|v| v.as_object_mut()) {
        // Try both path formats
        for key in [project_path, &normalized_path] {
            if let Some(project) = projects.get_mut(key) {
                if let Some(servers) = project.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                    servers.remove(mcp_name);
                }
                // Also remove from disabled list if present
                if let Some(disabled) = project.get_mut("disabledMcpServers").and_then(|v| v.as_array_mut()) {
                    disabled.retain(|v| v.as_str() != Some(mcp_name));
                }
            }
        }
    }

    write_claude_json(&json)?;
    Ok(())
}

/// Toggle an MCP's enabled state in a project
pub fn toggle_mcp_in_project_claude_json(project_path: &str, mcp_name: &str, enabled: bool) -> Result<()> {
    let mut json = read_claude_json()?;
    let normalized_path = normalize_path(project_path);

    if let Some(projects) = json.get_mut("projects").and_then(|v| v.as_object_mut()) {
        for key in [project_path, &normalized_path] {
            if let Some(project) = projects.get_mut(key) {
                // Ensure disabledMcpServers array exists
                if project.get("disabledMcpServers").is_none() {
                    project["disabledMcpServers"] = serde_json::json!([]);
                }

                let disabled = project.get_mut("disabledMcpServers")
                    .and_then(|v| v.as_array_mut())
                    .unwrap();

                if enabled {
                    // Remove from disabled list
                    disabled.retain(|v| v.as_str() != Some(mcp_name));
                } else {
                    // Add to disabled list if not already there
                    if !disabled.iter().any(|v| v.as_str() == Some(mcp_name)) {
                        disabled.push(serde_json::json!(mcp_name));
                    }
                }
                break;
            }
        }
    }

    write_claude_json(&json)?;
    Ok(())
}

/// Add a global MCP to claude.json
pub fn add_global_mcp_to_claude_json(mcp_name: &str, server: ClaudeJsonMcpServer) -> Result<()> {
    let mut json = read_claude_json()?;

    if json.get("mcpServers").is_none() {
        json["mcpServers"] = serde_json::json!({});
    }

    json["mcpServers"][mcp_name] = serde_json::to_value(server)?;

    write_claude_json(&json)?;
    Ok(())
}

/// Remove a global MCP from claude.json
pub fn remove_global_mcp_from_claude_json(mcp_name: &str) -> Result<()> {
    let mut json = read_claude_json()?;

    if let Some(servers) = json.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
        servers.remove(mcp_name);
    }

    write_claude_json(&json)?;
    Ok(())
}
