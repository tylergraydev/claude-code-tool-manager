use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::services::claude_json::{self, ClaudeJsonMcpServer, DetectedMcp};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeJsonMcpInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
    pub project_path: Option<String>,
    pub is_enabled: bool,
}

impl From<DetectedMcp> for ClaudeJsonMcpInfo {
    fn from(mcp: DetectedMcp) -> Self {
        Self {
            name: mcp.name,
            mcp_type: mcp.mcp_type,
            command: mcp.command,
            args: mcp.args,
            url: mcp.url,
            headers: mcp.headers,
            env: mcp.env,
            project_path: mcp.project_path,
            is_enabled: mcp.is_enabled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeJsonProjectInfo {
    pub path: String,
    pub mcps: Vec<ClaudeJsonMcpInfo>,
}

/// Get all MCPs from claude.json
#[tauri::command]
pub fn get_claude_json_mcps() -> Result<Vec<ClaudeJsonMcpInfo>, String> {
    claude_json::get_all_mcps_from_claude_json()
        .map(|mcps| mcps.into_iter().map(ClaudeJsonMcpInfo::from).collect())
        .map_err(|e| e.to_string())
}

/// Get projects with MCPs from claude.json
#[tauri::command]
pub fn get_claude_json_projects() -> Result<Vec<ClaudeJsonProjectInfo>, String> {
    claude_json::get_projects_from_claude_json()
        .map(|projects| {
            projects
                .into_iter()
                .map(|(path, project)| {
                    let mcps = project
                        .mcp_servers
                        .into_iter()
                        .map(|(name, server)| {
                            let is_enabled = !project.disabled_mcp_servers.contains(&name);
                            ClaudeJsonMcpInfo {
                                name,
                                mcp_type: server.mcp_type,
                                command: server.command,
                                args: server.args,
                                url: server.url,
                                headers: server.headers,
                                env: server.env,
                                project_path: Some(path.clone()),
                                is_enabled,
                            }
                        })
                        .collect();
                    ClaudeJsonProjectInfo { path, mcps }
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMcpToClaudeJsonRequest {
    pub project_path: String,
    pub mcp_name: String,
    #[serde(rename = "type")]
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

/// Add an MCP to a project in claude.json
#[tauri::command]
pub fn add_mcp_to_claude_json(request: AddMcpToClaudeJsonRequest) -> Result<(), String> {
    let server = ClaudeJsonMcpServer {
        mcp_type: request.mcp_type,
        command: request.command,
        args: request.args,
        url: request.url,
        headers: request.headers,
        env: request.env,
    };

    claude_json::add_mcp_to_project_in_claude_json(&request.project_path, &request.mcp_name, server)
        .map_err(|e| e.to_string())
}

/// Remove an MCP from a project in claude.json
#[tauri::command]
pub fn remove_mcp_from_claude_json(project_path: String, mcp_name: String) -> Result<(), String> {
    claude_json::remove_mcp_from_project_in_claude_json(&project_path, &mcp_name)
        .map_err(|e| e.to_string())
}

/// Toggle an MCP's enabled state in claude.json
#[tauri::command]
pub fn toggle_mcp_in_claude_json(
    project_path: String,
    mcp_name: String,
    enabled: bool,
) -> Result<(), String> {
    claude_json::toggle_mcp_in_project_claude_json(&project_path, &mcp_name, enabled)
        .map_err(|e| e.to_string())
}

/// Add a global MCP to claude.json
#[tauri::command]
pub fn add_global_mcp_to_claude_json(request: AddMcpToClaudeJsonRequest) -> Result<(), String> {
    let server = ClaudeJsonMcpServer {
        mcp_type: request.mcp_type,
        command: request.command,
        args: request.args,
        url: request.url,
        headers: request.headers,
        env: request.env,
    };

    claude_json::add_global_mcp_to_claude_json(&request.mcp_name, server).map_err(|e| e.to_string())
}

/// Remove a global MCP from claude.json
#[tauri::command]
pub fn remove_global_mcp_from_claude_json(mcp_name: String) -> Result<(), String> {
    claude_json::remove_global_mcp_from_claude_json(&mcp_name).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // ClaudeJsonMcpInfo tests
    // =========================================================================

    #[test]
    fn test_claude_json_mcp_info_serialization() {
        let info = ClaudeJsonMcpInfo {
            name: "test-mcp".to_string(),
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@test/mcp".to_string()]),
            url: None,
            headers: None,
            env: Some(HashMap::from([(
                "API_KEY".to_string(),
                "secret".to_string(),
            )])),
            project_path: Some("/home/user/project".to_string()),
            is_enabled: true,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"test-mcp\""));
        assert!(json.contains("\"type\":\"stdio\"")); // renamed field
        assert!(json.contains("\"isEnabled\":true")); // camelCase
    }

    #[test]
    fn test_claude_json_mcp_info_deserialization() {
        let json = r#"{
            "name": "my-mcp",
            "type": "sse",
            "url": "https://example.com/sse",
            "headers": {"Authorization": "Bearer token"},
            "isEnabled": false
        }"#;

        let info: ClaudeJsonMcpInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "my-mcp");
        assert_eq!(info.mcp_type, "sse");
        assert_eq!(info.url, Some("https://example.com/sse".to_string()));
        assert!(!info.is_enabled);
        assert!(info.command.is_none());
        assert!(info.args.is_none());
    }

    #[test]
    fn test_claude_json_mcp_info_from_detected_mcp() {
        let detected = DetectedMcp {
            name: "detected-mcp".to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: None,
            url: Some("https://api.example.com".to_string()),
            headers: Some(HashMap::from([(
                "X-API-Key".to_string(),
                "key123".to_string(),
            )])),
            env: None,
            project_path: Some("/project/path".to_string()),
            is_enabled: true,
        };

        let info = ClaudeJsonMcpInfo::from(detected);

        assert_eq!(info.name, "detected-mcp");
        assert_eq!(info.mcp_type, "http");
        assert_eq!(info.url, Some("https://api.example.com".to_string()));
        assert!(info.headers.is_some());
        assert_eq!(
            info.headers.unwrap().get("X-API-Key"),
            Some(&"key123".to_string())
        );
        assert_eq!(info.project_path, Some("/project/path".to_string()));
        assert!(info.is_enabled);
    }

    #[test]
    fn test_claude_json_mcp_info_from_detected_mcp_stdio() {
        let detected = DetectedMcp {
            name: "stdio-mcp".to_string(),
            mcp_type: "stdio".to_string(),
            command: Some("node".to_string()),
            args: Some(vec!["server.js".to_string()]),
            url: None,
            headers: None,
            env: Some(HashMap::from([("DEBUG".to_string(), "true".to_string())])),
            project_path: None,
            is_enabled: false,
        };

        let info = ClaudeJsonMcpInfo::from(detected);

        assert_eq!(info.name, "stdio-mcp");
        assert_eq!(info.mcp_type, "stdio");
        assert_eq!(info.command, Some("node".to_string()));
        assert_eq!(info.args, Some(vec!["server.js".to_string()]));
        assert!(info.url.is_none());
        assert!(info.project_path.is_none());
        assert!(!info.is_enabled);
    }

    #[test]
    fn test_claude_json_mcp_info_minimal() {
        let info = ClaudeJsonMcpInfo {
            name: "minimal".to_string(),
            mcp_type: "stdio".to_string(),
            command: None,
            args: None,
            url: None,
            headers: None,
            env: None,
            project_path: None,
            is_enabled: true,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: ClaudeJsonMcpInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "minimal");
        assert!(parsed.command.is_none());
        assert!(parsed.url.is_none());
    }

    // =========================================================================
    // ClaudeJsonProjectInfo tests
    // =========================================================================

    #[test]
    fn test_claude_json_project_info_serialization() {
        let project = ClaudeJsonProjectInfo {
            path: "/home/user/myproject".to_string(),
            mcps: vec![ClaudeJsonMcpInfo {
                name: "mcp1".to_string(),
                mcp_type: "stdio".to_string(),
                command: Some("npx".to_string()),
                args: None,
                url: None,
                headers: None,
                env: None,
                project_path: Some("/home/user/myproject".to_string()),
                is_enabled: true,
            }],
        };

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"path\":\"/home/user/myproject\""));
        assert!(json.contains("\"mcps\":["));
    }

    #[test]
    fn test_claude_json_project_info_deserialization() {
        let json = r#"{
            "path": "/projects/test",
            "mcps": [
                {
                    "name": "test-mcp",
                    "type": "stdio",
                    "command": "node",
                    "isEnabled": true
                }
            ]
        }"#;

        let project: ClaudeJsonProjectInfo = serde_json::from_str(json).unwrap();
        assert_eq!(project.path, "/projects/test");
        assert_eq!(project.mcps.len(), 1);
        assert_eq!(project.mcps[0].name, "test-mcp");
    }

    #[test]
    fn test_claude_json_project_info_empty_mcps() {
        let project = ClaudeJsonProjectInfo {
            path: "/empty/project".to_string(),
            mcps: vec![],
        };

        let json = serde_json::to_string(&project).unwrap();
        let parsed: ClaudeJsonProjectInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.path, "/empty/project");
        assert!(parsed.mcps.is_empty());
    }

    // =========================================================================
    // AddMcpToClaudeJsonRequest tests
    // =========================================================================

    #[test]
    fn test_add_mcp_request_deserialization_stdio() {
        let json = r#"{
            "projectPath": "/home/user/project",
            "mcpName": "new-mcp",
            "type": "stdio",
            "command": "python",
            "args": ["-m", "mcp_server"]
        }"#;

        let request: AddMcpToClaudeJsonRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.project_path, "/home/user/project");
        assert_eq!(request.mcp_name, "new-mcp");
        assert_eq!(request.mcp_type, "stdio");
        assert_eq!(request.command, Some("python".to_string()));
        assert_eq!(
            request.args,
            Some(vec!["-m".to_string(), "mcp_server".to_string()])
        );
        assert!(request.url.is_none());
    }

    #[test]
    fn test_add_mcp_request_deserialization_sse() {
        let json = r#"{
            "projectPath": "/project",
            "mcpName": "sse-mcp",
            "type": "sse",
            "url": "https://mcp.example.com/events",
            "headers": {"X-Token": "abc123"}
        }"#;

        let request: AddMcpToClaudeJsonRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.mcp_type, "sse");
        assert_eq!(
            request.url,
            Some("https://mcp.example.com/events".to_string())
        );
        assert!(request.headers.is_some());
        assert_eq!(
            request.headers.unwrap().get("X-Token"),
            Some(&"abc123".to_string())
        );
        assert!(request.command.is_none());
    }

    #[test]
    fn test_add_mcp_request_deserialization_with_env() {
        let json = r#"{
            "projectPath": "/project",
            "mcpName": "env-mcp",
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "@test/mcp"],
            "env": {
                "API_KEY": "secret",
                "DEBUG": "true"
            }
        }"#;

        let request: AddMcpToClaudeJsonRequest = serde_json::from_str(json).unwrap();
        assert!(request.env.is_some());
        let env = request.env.unwrap();
        assert_eq!(env.get("API_KEY"), Some(&"secret".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_add_mcp_request_minimal() {
        let json = r#"{
            "projectPath": "/project",
            "mcpName": "minimal",
            "type": "stdio"
        }"#;

        let request: AddMcpToClaudeJsonRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.project_path, "/project");
        assert_eq!(request.mcp_name, "minimal");
        assert_eq!(request.mcp_type, "stdio");
        assert!(request.command.is_none());
        assert!(request.args.is_none());
        assert!(request.url.is_none());
        assert!(request.headers.is_none());
        assert!(request.env.is_none());
    }

    // =========================================================================
    // Round-trip tests
    // =========================================================================

    #[test]
    fn test_mcp_info_round_trip() {
        let original = ClaudeJsonMcpInfo {
            name: "round-trip".to_string(),
            mcp_type: "http".to_string(),
            command: None,
            args: None,
            url: Some("https://api.test.com".to_string()),
            headers: Some(HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Authorization".to_string(), "Bearer xyz".to_string()),
            ])),
            env: None,
            project_path: Some("/test/project".to_string()),
            is_enabled: true,
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ClaudeJsonMcpInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.mcp_type, original.mcp_type);
        assert_eq!(parsed.url, original.url);
        assert_eq!(parsed.project_path, original.project_path);
        assert_eq!(parsed.is_enabled, original.is_enabled);
    }

    #[test]
    fn test_project_info_round_trip() {
        let original = ClaudeJsonProjectInfo {
            path: "/my/project".to_string(),
            mcps: vec![
                ClaudeJsonMcpInfo {
                    name: "mcp-a".to_string(),
                    mcp_type: "stdio".to_string(),
                    command: Some("node".to_string()),
                    args: Some(vec!["index.js".to_string()]),
                    url: None,
                    headers: None,
                    env: None,
                    project_path: Some("/my/project".to_string()),
                    is_enabled: true,
                },
                ClaudeJsonMcpInfo {
                    name: "mcp-b".to_string(),
                    mcp_type: "sse".to_string(),
                    command: None,
                    args: None,
                    url: Some("https://sse.example.com".to_string()),
                    headers: None,
                    env: None,
                    project_path: Some("/my/project".to_string()),
                    is_enabled: false,
                },
            ],
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ClaudeJsonProjectInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.path, original.path);
        assert_eq!(parsed.mcps.len(), 2);
        assert_eq!(parsed.mcps[0].name, "mcp-a");
        assert_eq!(parsed.mcps[1].name, "mcp-b");
        assert!(!parsed.mcps[1].is_enabled);
    }
}
