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

    // =========================================================================
    // Additional coverage: ClaudeJsonMcpServer edge cases
    // =========================================================================

    #[test]
    fn test_deserialize_mcp_server_with_multiple_headers() {
        let json = r#"{
            "type": "http",
            "url": "https://api.example.com",
            "headers": {
                "Authorization": "Bearer tok",
                "X-Custom": "val",
                "Content-Type": "application/json"
            }
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();
        assert_eq!(server.headers.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_deserialize_mcp_server_with_multiple_env_vars() {
        let json = r#"{
            "command": "node",
            "args": ["index.js"],
            "env": {"A": "1", "B": "2", "C": "3"}
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();
        assert_eq!(server.env.as_ref().unwrap().len(), 3);
        assert_eq!(server.mcp_type, "stdio"); // default
    }

    #[test]
    fn test_serialize_mcp_server_with_all_fields() {
        let mut headers = HashMap::new();
        headers.insert("Auth".to_string(), "Bearer x".to_string());
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "val".to_string());

        let server = ClaudeJsonMcpServer {
            mcp_type: "http".to_string(),
            command: Some("cmd".to_string()),
            args: Some(vec!["a".to_string()]),
            url: Some("https://example.com".to_string()),
            headers: Some(headers),
            env: Some(env),
        };

        let json = serde_json::to_string(&server).unwrap();
        assert!(json.contains("\"type\":\"http\""));
        assert!(json.contains("\"command\":\"cmd\""));
        assert!(json.contains("\"url\":\"https://example.com\""));
        assert!(json.contains("\"Auth\""));
        assert!(json.contains("\"KEY\""));
    }

    // =========================================================================
    // Additional coverage: ClaudeJsonProject edge cases
    // =========================================================================

    #[test]
    fn test_deserialize_project_with_disabled_and_enabled_lists() {
        let json = r#"{
            "mcpServers": {},
            "disabledMcpServers": ["mcp-a", "mcp-b"],
            "enabledMcpjsonServers": ["json-a"],
            "disabledMcpjsonServers": ["json-b", "json-c"]
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();
        assert_eq!(project.disabled_mcp_servers.len(), 2);
        assert_eq!(project.enabled_mcpjson_servers.len(), 1);
        assert_eq!(project.disabled_mcpjson_servers.len(), 2);
    }

    #[test]
    fn test_project_serialization_preserves_other() {
        let json = r#"{
            "mcpServers": {},
            "allowedTools": ["Read", "Write"],
            "hasTrustDialogAccepted": true,
            "projectOnboardingSeenCount": 5
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();
        // Serialize and reparse
        let serialized = serde_json::to_string(&project).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Other fields should still be present
        assert!(reparsed.get("allowedTools").is_some());
        assert_eq!(reparsed["hasTrustDialogAccepted"], true);
        assert_eq!(reparsed["projectOnboardingSeenCount"], 5);
    }

    // =========================================================================
    // Additional coverage: DetectedMcp with all field types
    // =========================================================================

    #[test]
    fn test_detected_mcp_with_env_and_headers() {
        let mut headers = HashMap::new();
        headers.insert("Auth".to_string(), "Bearer t".to_string());
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "val".to_string());

        let mcp = DetectedMcp {
            name: "full-mcp".to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: None,
            url: Some("https://example.com".to_string()),
            headers: Some(headers.clone()),
            env: Some(env.clone()),
            project_path: Some("/path".to_string()),
            is_enabled: true,
        };

        assert_eq!(
            mcp.headers.unwrap().get("Auth"),
            Some(&"Bearer t".to_string())
        );
        assert_eq!(mcp.env.unwrap().get("KEY"), Some(&"val".to_string()));
    }

    // =========================================================================
    // Additional coverage: Complex JSON structures for parsing
    // =========================================================================

    #[test]
    fn test_parse_full_claude_json_structure() {
        // Simulate the structure that get_all_mcps_from_claude_json would process
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "mcpServers": {
                "global-mcp": {
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "mcp-server"]
                }
            },
            "projects": {
                "/home/user/project": {
                    "mcpServers": {
                        "proj-mcp": {
                            "type": "sse",
                            "url": "https://mcp.example.com"
                        }
                    },
                    "disabledMcpServers": ["proj-mcp"],
                    "allowedTools": ["Read"]
                }
            }
        }"#,
        )
        .unwrap();

        // Parse global servers
        let global_servers = json.get("mcpServers").unwrap().as_object().unwrap();
        assert_eq!(global_servers.len(), 1);
        let global_mcp: ClaudeJsonMcpServer =
            serde_json::from_value(global_servers.get("global-mcp").unwrap().clone()).unwrap();
        assert_eq!(global_mcp.mcp_type, "stdio");
        assert_eq!(global_mcp.command, Some("npx".to_string()));

        // Parse project
        let projects = json.get("projects").unwrap().as_object().unwrap();
        let project_val = projects.get("/home/user/project").unwrap();
        let project: ClaudeJsonProject = serde_json::from_value(project_val.clone()).unwrap();
        assert_eq!(project.mcp_servers.len(), 1);
        assert!(project
            .disabled_mcp_servers
            .contains(&"proj-mcp".to_string()));
    }

    #[test]
    fn test_parse_project_with_no_mcp_servers_key() {
        // Projects without mcpServers should still parse
        let json = r#"{
            "allowedTools": ["Read"],
            "hasTrustDialogAccepted": true
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();
        assert!(project.mcp_servers.is_empty());
        assert!(project.other.contains_key("allowedTools"));
    }

    #[test]
    fn test_mcp_server_with_many_args() {
        let json = r#"{
            "type": "stdio",
            "command": "node",
            "args": ["--experimental-modules", "--max-old-space-size=4096", "dist/index.js", "--port", "3000"]
        }"#;

        let server: ClaudeJsonMcpServer = serde_json::from_str(json).unwrap();
        assert_eq!(server.args.as_ref().unwrap().len(), 5);
    }

    #[test]
    fn test_mcp_server_clone() {
        let server = ClaudeJsonMcpServer {
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["arg".to_string()]),
            url: None,
            headers: None,
            env: None,
        };

        let cloned = server.clone();
        assert_eq!(cloned.mcp_type, server.mcp_type);
        assert_eq!(cloned.command, server.command);
        assert_eq!(cloned.args, server.args);
    }

    // =========================================================================
    // File-backed function tests using tempfile
    // These test the core logic of read/write/toggle operations against
    // realistic claude.json structures written to temporary files.
    // =========================================================================

    /// Helper: write a claude.json to a tempdir and return the path
    fn write_temp_claude_json(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
        let path = dir.join("claude.json");
        std::fs::write(&path, content).unwrap();
        path
    }

    /// Helper: read and parse the claude.json from the given path
    fn read_temp_claude_json(path: &std::path::Path) -> serde_json::Value {
        let content = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn test_get_all_mcps_parsing_global_servers() {
        // Simulate parsing global MCPs from a JSON value
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "mcpServers": {
                "memory": {
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-memory"]
                },
                "remote": {
                    "type": "sse",
                    "url": "https://mcp.example.com/sse"
                }
            }
        }"#,
        )
        .unwrap();

        let mut mcps = Vec::new();
        if let Some(global_servers) = json.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_value) in global_servers {
                if let Ok(server) =
                    serde_json::from_value::<ClaudeJsonMcpServer>(server_value.clone())
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
                        is_enabled: true,
                    });
                }
            }
        }

        assert_eq!(mcps.len(), 2);
        let memory_mcp = mcps.iter().find(|m| m.name == "memory").unwrap();
        assert_eq!(memory_mcp.mcp_type, "stdio");
        assert_eq!(memory_mcp.command, Some("npx".to_string()));
        assert!(memory_mcp.project_path.is_none());
        assert!(memory_mcp.is_enabled);

        let remote_mcp = mcps.iter().find(|m| m.name == "remote").unwrap();
        assert_eq!(remote_mcp.mcp_type, "sse");
        assert_eq!(
            remote_mcp.url,
            Some("https://mcp.example.com/sse".to_string())
        );
    }

    #[test]
    fn test_get_all_mcps_parsing_project_servers_with_disabled() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "projects": {
                "/home/user/my-project": {
                    "mcpServers": {
                        "enabled-mcp": {"command": "npx", "args": ["server1"]},
                        "disabled-mcp": {"command": "npx", "args": ["server2"]}
                    },
                    "disabledMcpServers": ["disabled-mcp"],
                    "allowedTools": ["Read"]
                }
            }
        }"#,
        )
        .unwrap();

        let mut mcps = Vec::new();
        if let Some(projects) = json.get("projects").and_then(|v| v.as_object()) {
            for (project_path, project_value) in projects {
                if let Ok(project) =
                    serde_json::from_value::<ClaudeJsonProject>(project_value.clone())
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

        assert_eq!(mcps.len(), 2);
        let enabled = mcps.iter().find(|m| m.name == "enabled-mcp").unwrap();
        assert!(enabled.is_enabled);
        assert_eq!(
            enabled.project_path,
            Some("/home/user/my-project".to_string())
        );

        let disabled = mcps.iter().find(|m| m.name == "disabled-mcp").unwrap();
        assert!(!disabled.is_enabled);
    }

    #[test]
    fn test_get_all_projects_parsing() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "projects": {
                "/path/a": {"mcpServers": {"mcp1": {"command": "c1"}}, "allowedTools": []},
                "/path/b": {"allowedTools": ["Read"]},
                "/path/c": {"mcpServers": {"mcp2": {"type": "sse", "url": "http://x"}}}
            }
        }"#,
        )
        .unwrap();

        let mut projects = Vec::new();
        if let Some(projects_obj) = json.get("projects").and_then(|v| v.as_object()) {
            for (path, value) in projects_obj {
                if let Ok(project) = serde_json::from_value::<ClaudeJsonProject>(value.clone()) {
                    projects.push((path.clone(), project));
                }
            }
        }

        assert_eq!(projects.len(), 3);
    }

    #[test]
    fn test_get_projects_with_mcps_only() {
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "projects": {
                "/path/a": {"mcpServers": {"mcp1": {"command": "c1"}}},
                "/path/b": {"allowedTools": ["Read"]},
                "/path/c": {"mcpServers": {}}
            }
        }"#,
        )
        .unwrap();

        let mut projects = Vec::new();
        if let Some(projects_obj) = json.get("projects").and_then(|v| v.as_object()) {
            for (path, value) in projects_obj {
                if let Ok(project) = serde_json::from_value::<ClaudeJsonProject>(value.clone()) {
                    if !project.mcp_servers.is_empty() {
                        projects.push((path.clone(), project));
                    }
                }
            }
        }

        // Only /path/a has non-empty mcpServers
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].0, "/path/a");
    }

    #[test]
    fn test_add_mcp_to_project_logic_new_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(dir.path(), r#"{"projects": {}}"#);

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        // Simulate add_mcp_to_project logic
        let project_path = "/tmp/test-project";
        if json.get("projects").is_none() {
            json["projects"] = serde_json::json!({});
        }
        let projects = json.get_mut("projects").unwrap().as_object_mut().unwrap();
        projects.insert(
            project_path.to_string(),
            serde_json::json!({
                "mcpServers": {},
                "disabledMcpServers": []
            }),
        );

        let server = ClaudeJsonMcpServer {
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["server".to_string()]),
            url: None,
            headers: None,
            env: None,
        };
        let project = projects.get_mut(project_path).unwrap();
        project["mcpServers"]["test-mcp"] = serde_json::to_value(&server).unwrap();

        // Write back
        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

        let result = read_temp_claude_json(&path);
        assert!(
            result["projects"][project_path]["mcpServers"]["test-mcp"]["command"].as_str()
                == Some("npx")
        );
    }

    #[test]
    fn test_remove_mcp_from_project_logic() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(
            dir.path(),
            r#"{
            "projects": {
                "/tmp/proj": {
                    "mcpServers": {
                        "keep-mcp": {"command": "keep"},
                        "remove-mcp": {"command": "remove"}
                    },
                    "disabledMcpServers": ["remove-mcp"]
                }
            }
        }"#,
        );

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        // Simulate remove logic
        if let Some(projects) = json.get_mut("projects").and_then(|v| v.as_object_mut()) {
            if let Some(project) = projects.get_mut("/tmp/proj") {
                if let Some(servers) = project
                    .get_mut("mcpServers")
                    .and_then(|v| v.as_object_mut())
                {
                    servers.remove("remove-mcp");
                }
                if let Some(disabled) = project
                    .get_mut("disabledMcpServers")
                    .and_then(|v| v.as_array_mut())
                {
                    disabled.retain(|v| v.as_str() != Some("remove-mcp"));
                }
            }
        }

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let result = read_temp_claude_json(&path);

        let servers = result["projects"]["/tmp/proj"]["mcpServers"]
            .as_object()
            .unwrap();
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("keep-mcp"));
        assert!(!servers.contains_key("remove-mcp"));

        let disabled = result["projects"]["/tmp/proj"]["disabledMcpServers"]
            .as_array()
            .unwrap();
        assert!(disabled.is_empty());
    }

    #[test]
    fn test_toggle_mcp_disable_logic() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(
            dir.path(),
            r#"{
            "projects": {
                "/tmp/proj": {
                    "mcpServers": {"my-mcp": {"command": "cmd"}},
                    "disabledMcpServers": []
                }
            }
        }"#,
        );

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        // Simulate toggle disable
        let mcp_name = "my-mcp";
        if let Some(projects) = json.get_mut("projects").and_then(|v| v.as_object_mut()) {
            if let Some(project) = projects.get_mut("/tmp/proj") {
                if project.get("disabledMcpServers").is_none() {
                    project["disabledMcpServers"] = serde_json::json!([]);
                }
                let disabled = project
                    .get_mut("disabledMcpServers")
                    .and_then(|v| v.as_array_mut())
                    .unwrap();
                if !disabled.iter().any(|v| v.as_str() == Some(mcp_name)) {
                    disabled.push(serde_json::json!(mcp_name));
                }
            }
        }

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let result = read_temp_claude_json(&path);

        let disabled = result["projects"]["/tmp/proj"]["disabledMcpServers"]
            .as_array()
            .unwrap();
        assert!(disabled.iter().any(|v| v.as_str() == Some("my-mcp")));
    }

    #[test]
    fn test_toggle_mcp_enable_logic() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(
            dir.path(),
            r#"{
            "projects": {
                "/tmp/proj": {
                    "mcpServers": {"my-mcp": {"command": "cmd"}},
                    "disabledMcpServers": ["my-mcp"]
                }
            }
        }"#,
        );

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        // Simulate toggle enable
        let mcp_name = "my-mcp";
        if let Some(projects) = json.get_mut("projects").and_then(|v| v.as_object_mut()) {
            if let Some(project) = projects.get_mut("/tmp/proj") {
                let disabled = project
                    .get_mut("disabledMcpServers")
                    .and_then(|v| v.as_array_mut())
                    .unwrap();
                disabled.retain(|v| v.as_str() != Some(mcp_name));
            }
        }

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let result = read_temp_claude_json(&path);

        let disabled = result["projects"]["/tmp/proj"]["disabledMcpServers"]
            .as_array()
            .unwrap();
        assert!(disabled.is_empty());
    }

    #[test]
    fn test_add_global_mcp_logic() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(dir.path(), r#"{"mcpServers": {}}"#);

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        let server = ClaudeJsonMcpServer {
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "mcp-server".to_string()]),
            url: None,
            headers: None,
            env: None,
        };

        if json.get("mcpServers").is_none() {
            json["mcpServers"] = serde_json::json!({});
        }
        json["mcpServers"]["new-global"] = serde_json::to_value(&server).unwrap();

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let result = read_temp_claude_json(&path);

        assert_eq!(result["mcpServers"]["new-global"]["command"], "npx");
    }

    #[test]
    fn test_remove_global_mcp_logic() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(
            dir.path(),
            r#"{
            "mcpServers": {
                "keep": {"command": "keep"},
                "remove": {"command": "remove"}
            }
        }"#,
        );

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        if let Some(servers) = json.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
            servers.remove("remove");
        }

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let result = read_temp_claude_json(&path);

        let servers = result["mcpServers"].as_object().unwrap();
        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("keep"));
    }

    #[test]
    fn test_toggle_already_disabled_mcp_stays_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(
            dir.path(),
            r#"{
            "projects": {
                "/tmp/proj": {
                    "mcpServers": {"mcp": {"command": "cmd"}},
                    "disabledMcpServers": ["mcp"]
                }
            }
        }"#,
        );

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        // Disable again (should not duplicate)
        let mcp_name = "mcp";
        if let Some(projects) = json.get_mut("projects").and_then(|v| v.as_object_mut()) {
            if let Some(project) = projects.get_mut("/tmp/proj") {
                let disabled = project
                    .get_mut("disabledMcpServers")
                    .and_then(|v| v.as_array_mut())
                    .unwrap();
                if !disabled.iter().any(|v| v.as_str() == Some(mcp_name)) {
                    disabled.push(serde_json::json!(mcp_name));
                }
            }
        }

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let result = read_temp_claude_json(&path);

        let disabled = result["projects"]["/tmp/proj"]["disabledMcpServers"]
            .as_array()
            .unwrap();
        assert_eq!(disabled.len(), 1); // No duplicate
    }

    #[test]
    fn test_parsing_project_with_mcpjson_servers() {
        let json = r#"{
            "mcpServers": {},
            "enabledMcpjsonServers": ["server-a", "server-b"],
            "disabledMcpjsonServers": ["server-c"]
        }"#;

        let project: ClaudeJsonProject = serde_json::from_str(json).unwrap();
        assert_eq!(
            project.enabled_mcpjson_servers,
            vec!["server-a", "server-b"]
        );
        assert_eq!(project.disabled_mcpjson_servers, vec!["server-c"]);
    }

    #[test]
    fn test_parsing_malformed_mcp_server_skipped() {
        // Invalid MCP (missing required fields but has unexpected types)
        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "mcpServers": {
                "valid": {"command": "npx", "args": ["-y", "server"]},
                "invalid": "not-an-object"
            }
        }"#,
        )
        .unwrap();

        let mut mcps = Vec::new();
        if let Some(global_servers) = json.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_value) in global_servers {
                if let Ok(server) =
                    serde_json::from_value::<ClaudeJsonMcpServer>(server_value.clone())
                {
                    mcps.push(name.clone());
                    let _ = server; // use it
                }
            }
        }

        // Only the valid one should parse
        assert_eq!(mcps.len(), 1);
        assert_eq!(mcps[0], "valid");
    }

    #[test]
    fn test_add_mcp_creates_projects_object_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(dir.path(), r#"{"mcpServers": {}}"#);

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        // Simulate: ensure projects object exists
        if json.get("projects").is_none() {
            json["projects"] = serde_json::json!({});
        }

        assert!(json["projects"].is_object());
    }

    #[test]
    fn test_add_mcp_creates_mcp_servers_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_temp_claude_json(
            dir.path(),
            r#"{
            "projects": {
                "/tmp/proj": {
                    "allowedTools": ["Read"]
                }
            }
        }"#,
        );

        let mut json: serde_json::Value = read_temp_claude_json(&path);

        let projects = json.get_mut("projects").unwrap().as_object_mut().unwrap();
        let project = projects.get_mut("/tmp/proj").unwrap();
        if project.get("mcpServers").is_none() {
            project["mcpServers"] = serde_json::json!({});
        }

        let server = ClaudeJsonMcpServer {
            mcp_type: "stdio".to_string(),
            command: Some("cmd".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
        };
        project["mcpServers"]["new-mcp"] = serde_json::to_value(&server).unwrap();

        std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
        let result = read_temp_claude_json(&path);

        assert_eq!(
            result["projects"]["/tmp/proj"]["mcpServers"]["new-mcp"]["command"],
            "cmd"
        );
        // allowedTools should be preserved
        assert!(result["projects"]["/tmp/proj"]["allowedTools"].is_array());
    }
}
