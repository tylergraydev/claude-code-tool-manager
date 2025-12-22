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
    #[serde(default = "default_editor_type")]
    pub editor_type: String,  // "claude_code" or "opencode"
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub assigned_mcps: Vec<ProjectMcp>,
}

fn default_editor_type() -> String {
    "claude_code".to_string()
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
    pub claude_json: String,  // ~/.claude.json - main config with global MCPs
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
    pub model: Option<String>,
    pub disable_model_invocation: bool,
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
    pub model: Option<String>,
    pub disable_model_invocation: Option<bool>,
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

// Skill Files (references, assets, scripts)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillFile {
    pub id: i64,
    pub skill_id: i64,
    pub file_type: String,  // "reference", "asset", "script"
    pub name: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillFileRequest {
    pub skill_id: i64,
    pub file_type: String,
    pub name: String,
    pub content: String,
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
    pub permission_mode: Option<String>,
    pub skills: Option<Vec<String>>,
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
    pub permission_mode: Option<String>,
    pub skills: Option<Vec<String>>,
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

// Repository sources (Marketplace)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub owner: String,
    pub repo: String,
    pub repo_type: String,    // "file_based" or "readme_based"
    pub content_type: String, // "mcp", "skill", "subagent", or "mixed"
    pub github_url: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub is_enabled: bool,
    pub last_fetched_at: Option<String>,
    pub etag: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepoRequest {
    pub github_url: String,
    pub repo_type: String,
    pub content_type: String,
}

// Cached items from repositories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoItem {
    pub id: i64,
    pub repo_id: i64,
    pub item_type: String, // "mcp", "skill", or "subagent"
    pub name: String,
    pub description: Option<String>,
    pub source_url: Option<String>,
    pub raw_content: Option<String>,
    pub file_path: Option<String>,
    pub metadata: Option<String>,
    pub stars: Option<i32>,
    pub is_imported: bool,
    pub imported_item_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub added: i32,
    pub updated: i32,
    pub removed: i32,
    pub errors: Vec<String>,
}

// GitHub rate limit info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    pub limit: i32,
    pub remaining: i32,
    pub reset_at: String,
}

// Import result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub success: bool,
    pub item_type: String,
    pub item_id: i64,
    pub message: Option<String>,
}

// Hooks (Event-triggered actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hook {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub event_type: String, // PreToolUse, PostToolUse, Notification, etc.
    pub matcher: Option<String>,
    pub hook_type: String, // "command" or "prompt"
    pub command: Option<String>,
    pub prompt: Option<String>,
    pub timeout: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub source: String,
    pub is_template: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHookRequest {
    pub name: String,
    pub description: Option<String>,
    pub event_type: String,
    pub matcher: Option<String>,
    pub hook_type: String,
    pub command: Option<String>,
    pub prompt: Option<String>,
    pub timeout: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHook {
    pub id: i64,
    pub hook_id: i64,
    pub hook: Hook,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalHook {
    pub id: i64,
    pub hook_id: i64,
    pub hook: Hook,
    pub is_enabled: bool,
}

// App Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_editor: String,  // "claude_code" or "opencode"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_editor: "claude_code".to_string(),
        }
    }
}

// OpenCode paths (for OpenCode support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodePaths {
    pub config_dir: String,      // ~/.config/opencode/
    pub config_file: String,     // ~/.config/opencode/opencode.json
    pub command_dir: String,     // ~/.config/opencode/command/
    pub agent_dir: String,       // ~/.config/opencode/agent/
    pub plugin_dir: String,      // ~/.config/opencode/plugin/
    pub tool_dir: String,        // ~/.config/opencode/tool/
    pub knowledge_dir: String,   // ~/.config/opencode/knowledge/
}

// Editor info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    pub id: String,              // "claude_code" or "opencode"
    pub name: String,            // "Claude Code" or "OpenCode"
    pub is_installed: bool,      // Whether config directory exists
    pub config_path: String,     // Path to main config file
}
