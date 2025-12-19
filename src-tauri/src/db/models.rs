use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mcp {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source: String,
    pub source_path: Option<String>,
    pub is_enabled_global: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMcpRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub mcp_type: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub has_mcp_file: bool,
    pub has_settings_file: bool,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub assigned_mcps: Vec<ProjectMcp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMcp {
    pub id: i64,
    pub mcp_id: i64,
    pub mcp: Mcp,
    pub is_enabled: bool,
    pub env_overrides: Option<HashMap<String, String>>,
    pub display_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalMcp {
    pub id: i64,
    pub mcp_id: i64,
    pub mcp: Mcp,
    pub is_enabled: bool,
    pub env_overrides: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudePaths {
    pub claude_dir: String,
    pub global_settings: String,
    pub plugins_dir: String,
}

// Skills (Slash Commands and Agent Skills)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub skill_type: String,  // "command" or "skill"
    pub allowed_tools: Option<Vec<String>>,
    pub argument_hint: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub skill_type: String,
    pub allowed_tools: Option<Vec<String>>,
    pub argument_hint: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSkill {
    pub id: i64,
    pub skill_id: i64,
    pub skill: Skill,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSkill {
    pub id: i64,
    pub skill_id: i64,
    pub skill: Skill,
    pub is_enabled: bool,
}

// Sub-Agents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAgent {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub content: String,
    pub tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubAgentRequest {
    pub name: String,
    pub description: String,
    pub content: String,
    pub tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSubAgent {
    pub id: i64,
    pub subagent_id: i64,
    pub subagent: SubAgent,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSubAgent {
    pub id: i64,
    pub subagent_id: i64,
    pub subagent: SubAgent,
    pub is_enabled: bool,
}
