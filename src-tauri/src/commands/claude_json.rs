use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::services::claude_json::{
    self, ClaudeJsonMcpServer, DetectedMcp,
};

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

    claude_json::add_global_mcp_to_claude_json(&request.mcp_name, server)
        .map_err(|e| e.to_string())
}

/// Remove a global MCP from claude.json
#[tauri::command]
pub fn remove_global_mcp_from_claude_json(mcp_name: String) -> Result<(), String> {
    claude_json::remove_global_mcp_from_claude_json(&mcp_name)
        .map_err(|e| e.to_string())
}
