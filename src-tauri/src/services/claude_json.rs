use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

use crate::utils::paths::{get_claude_paths, normalize_path};

/// Default MCP type when not specified (stdio is the Claude Code default)
pub(crate) fn default_mcp_type() -> String {
    "stdio".to_string()
}

/// MCP server configuration as stored in claude.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeJsonMcpServer {
    #[serde(rename = "type", default = "default_mcp_type")]
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
    pub project_path: Option<String>, // None for global MCPs
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
            if let Ok(server) = serde_json::from_value::<ClaudeJsonMcpServer>(server_value.clone())
            {
                mcps.push(DetectedMcp {
                    name: name.clone(),
                    mcp_type: server.mcp_type,
                    command: server.command,
                    args: server.args,
                    url: server.url,
                    headers: server.headers,
                    env: server.env,
                    project_path: None,
                    is_enabled: true, // Global MCPs are enabled by default
                });
            }
        }
    }

    // Get project-specific MCPs
    if let Some(projects) = json.get("projects").and_then(|v| v.as_object()) {
        for (project_path, project_value) in projects {
            if let Ok(project) = serde_json::from_value::<ClaudeJsonProject>(project_value.clone())
            {
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
        projects.insert(
            normalized_path.clone(),
            serde_json::json!({
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
            }),
        );
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
                if let Some(servers) = project
                    .get_mut("mcpServers")
                    .and_then(|v| v.as_object_mut())
                {
                    servers.remove(mcp_name);
                }
                // Also remove from disabled list if present
                if let Some(disabled) = project
                    .get_mut("disabledMcpServers")
                    .and_then(|v| v.as_array_mut())
                {
                    disabled.retain(|v| v.as_str() != Some(mcp_name));
                }
            }
        }
    }

    write_claude_json(&json)?;
    Ok(())
}

/// Toggle an MCP's enabled state in a project
pub fn toggle_mcp_in_project_claude_json(
    project_path: &str,
    mcp_name: &str,
    enabled: bool,
) -> Result<()> {
    let mut json = read_claude_json()?;
    let normalized_path = normalize_path(project_path);

    if let Some(projects) = json.get_mut("projects").and_then(|v| v.as_object_mut()) {
        for key in [project_path, &normalized_path] {
            if let Some(project) = projects.get_mut(key) {
                // Ensure disabledMcpServers array exists
                if project.get("disabledMcpServers").is_none() {
                    project["disabledMcpServers"] = serde_json::json!([]);
                }

                let disabled = project
                    .get_mut("disabledMcpServers")
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

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // default_mcp_type tests
    // =========================================================================

    #[test]
    fn test_default_mcp_type_returns_stdio() {
        assert_eq!(default_mcp_type(), "stdio");
    }

    // =========================================================================
    // ClaudeJsonMcpServer deserialization tests
    // =========================================================================

    #[test]
    fn test_deserialize_stdio_mcp_server() {
        let json = r#"{
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-memory"]
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert_eq!(server.mcp_type, "stdio");
        assert_eq!(server.command, Some("npx".to_string()));
        assert_eq!(
            server.args,
            Some(vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-memory".to_string()
            ])
        );
        assert!(server.url.is_none());
        assert!(server.headers.is_none());
    }

    #[test]
    fn test_deserialize_sse_mcp_server() {
        let json = r#"{
            "type": "sse",
            "url": "https://mcp.example.com/sse",
            "headers": {"Authorization": "Bearer token123"}
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert_eq!(server.mcp_type, "sse");
        assert_eq!(server.url, Some("https://mcp.example.com/sse".to_string()));
        assert!(server.headers.is_some());
        assert_eq!(
            server.headers.as_ref().unwrap().get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert!(server.command.is_none());
    }

    #[test]
    fn test_deserialize_http_mcp_server() {
        let json = r#"{
            "type": "http",
            "url": "https://mcp.example.com/api"
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert_eq!(server.mcp_type, "http");
        assert_eq!(server.url, Some("https://mcp.example.com/api".to_string()));
    }

    #[test]
    fn test_deserialize_mcp_server_default_type() {
        // When type is not specified, should default to "stdio"
        let json = r#"{
            "command": "npx",
            "args": ["-y", "some-mcp"]
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert_eq!(server.mcp_type, "stdio");
    }

    #[test]
    fn test_deserialize_mcp_server_with_env() {
        let json = r#"{
            "type": "stdio",
            "command": "node",
            "args": ["server.js"],
            "env": {"API_KEY": "secret123", "DEBUG": "true"}
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert!(server.env.is_some());
        let env = server.env.unwrap();
        assert_eq!(env.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_deserialize_minimal_mcp_server() {
        let json = r#"{}"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert_eq!(server.mcp_type, "stdio"); // Default
        assert!(server.command.is_none());
        assert!(server.args.is_none());
        assert!(server.url.is_none());
        assert!(server.headers.is_none());
        assert!(server.env.is_none());
    }

    // =========================================================================
    // ClaudeJsonMcpServer serialization tests
    // =========================================================================

    #[test]
    fn test_serialize_stdio_mcp_server() {
        let server = ClaudeJsonMcpServer {
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["mcp-server".to_string()]),
            url: None,
            headers: None,
            env: None,
        };

        let json = serde_json::to_string(&server).unwrap();

        // Should not include None fields
        assert!(!json.contains("\"url\""));
        assert!(!json.contains("\"headers\""));
        assert!(!json.contains("\"env\""));
        assert!(json.contains("\"type\":\"stdio\""));
        assert!(json.contains("\"command\":\"npx\""));
    }

    #[test]
    fn test_serialize_sse_mcp_server() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer xyz".to_string());

        let server = ClaudeJsonMcpServer {
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://example.com/sse".to_string()),
            headers: Some(headers),
            env: None,
        };

        let json = serde_json::to_string(&server).unwrap();

        assert!(json.contains("\"type\":\"sse\""));
        assert!(json.contains("\"url\":\"https://example.com/sse\""));
        assert!(json.contains("\"Authorization\""));
        // Should not include None fields
        assert!(!json.contains("\"command\""));
        assert!(!json.contains("\"args\""));
    }

    // =========================================================================
    // ClaudeJsonProject deserialization tests
    // =========================================================================

    #[test]
    fn test_deserialize_project_with_mcps() {
        let json = r#"{
            "mcpServers": {
                "memory": {
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-memory"]
                }
            },
            "disabledMcpServers": ["other-mcp"]
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();

        assert_eq!(project.mcp_servers.len(), 1);
        assert!(project.mcp_servers.contains_key("memory"));
        assert_eq!(project.disabled_mcp_servers, vec!["other-mcp"]);
    }

    #[test]
    fn test_deserialize_empty_project() {
        let json = r#"{}"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();

        assert!(project.mcp_servers.is_empty());
        assert!(project.disabled_mcp_servers.is_empty());
        assert!(project.enabled_mcpjson_servers.is_empty());
        assert!(project.disabled_mcpjson_servers.is_empty());
    }

    #[test]
    fn test_deserialize_project_preserves_other_fields() {
        let json = r#"{
            "mcpServers": {},
            "allowedTools": ["Read", "Write"],
            "customField": "custom-value",
            "hasTrustDialogAccepted": true
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();

        assert!(project.other.contains_key("allowedTools"));
        assert!(project.other.contains_key("customField"));
        assert!(project.other.contains_key("hasTrustDialogAccepted"));
    }

    #[test]
    fn test_deserialize_project_with_multiple_mcps() {
        let json = r#"{
            "mcpServers": {
                "mcp1": {"command": "cmd1"},
                "mcp2": {"type": "sse", "url": "https://example.com"},
                "mcp3": {"command": "cmd3", "env": {"KEY": "value"}}
            }
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();

        assert_eq!(project.mcp_servers.len(), 3);
        assert!(project.mcp_servers.contains_key("mcp1"));
        assert!(project.mcp_servers.contains_key("mcp2"));
        assert!(project.mcp_servers.contains_key("mcp3"));
    }

    // =========================================================================
    // DetectedMcp construction tests
    // =========================================================================

    #[test]
    fn test_detected_mcp_global() {
        let mcp = DetectedMcp {
            name: "test-mcp".to_string(),
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["arg1".to_string()]),
            url: None,
            headers: None,
            env: None,
            project_path: None, // Global MCP
            is_enabled: true,
        };

        assert!(mcp.project_path.is_none());
        assert!(mcp.is_enabled);
    }

    #[test]
    fn test_detected_mcp_project_enabled() {
        let mcp = DetectedMcp {
            name: "project-mcp".to_string(),
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://example.com".to_string()),
            headers: None,
            env: None,
            project_path: Some("/path/to/project".to_string()),
            is_enabled: true,
        };

        assert_eq!(mcp.project_path, Some("/path/to/project".to_string()));
        assert!(mcp.is_enabled);
    }

    #[test]
    fn test_detected_mcp_project_disabled() {
        let mcp = DetectedMcp {
            name: "disabled-mcp".to_string(),
            mcp_type: "stdio".to_string(),
            command: Some("cmd".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            project_path: Some("/path/to/project".to_string()),
            is_enabled: false,
        };

        assert!(!mcp.is_enabled);
    }

    // =========================================================================
    // Round-trip serialization tests
    // =========================================================================

    #[test]
    fn test_mcp_server_round_trip() {
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "value".to_string());

        let original = ClaudeJsonMcpServer {
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "server".to_string()]),
            url: None,
            headers: None,
            env: Some(env),
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ClaudeJsonMcpServer = serde_json::from_str(&json).unwrap();

        assert_eq!(original.mcp_type, parsed.mcp_type);
        assert_eq!(original.command, parsed.command);
        assert_eq!(original.args, parsed.args);
        assert_eq!(original.url, parsed.url);
        assert_eq!(original.env, parsed.env);
    }

    #[test]
    fn test_project_round_trip() {
        let json = r#"{
            "mcpServers": {
                "test": {"type": "stdio", "command": "cmd"}
            },
            "disabledMcpServers": ["disabled"],
            "enabledMcpjsonServers": ["enabled"],
            "allowedTools": ["Read"]
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&project).unwrap();
        let reparsed: ClaudeJsonProject = serde_json::from_str(&serialized).unwrap();

        assert_eq!(project.mcp_servers.len(), reparsed.mcp_servers.len());
        assert_eq!(project.disabled_mcp_servers, reparsed.disabled_mcp_servers);
        assert_eq!(
            project.enabled_mcpjson_servers,
            reparsed.enabled_mcpjson_servers
        );
    }

    // =========================================================================
    // Edge case tests
    // =========================================================================

    #[test]
    fn test_mcp_server_with_empty_arrays() {
        let json = r#"{
            "type": "stdio",
            "command": "cmd",
            "args": []
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert_eq!(server.args, Some(vec![]));
    }

    #[test]
    fn test_mcp_server_with_empty_env() {
        let json = r#"{
            "type": "stdio",
            "command": "cmd",
            "env": {}
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();

        assert!(server.env.is_some());
        assert!(server.env.unwrap().is_empty());
    }

    #[test]
    fn test_project_camel_case_fields() {
        // Verify camelCase conversion works for the serde rename
        let json = r#"{
            "mcpServers": {},
            "disabledMcpServers": [],
            "enabledMcpjsonServers": [],
            "disabledMcpjsonServers": []
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();

        assert!(project.mcp_servers.is_empty());
        assert!(project.disabled_mcp_servers.is_empty());
        assert!(project.enabled_mcpjson_servers.is_empty());
        assert!(project.disabled_mcpjson_servers.is_empty());
    }
}
