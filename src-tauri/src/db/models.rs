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
    pub is_favorite: bool,
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
    pub editor_type: String, // "claude_code" or "opencode"
    pub is_favorite: bool,
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
pub struct GatewayMcp {
    pub id: i64,
    pub mcp_id: i64,
    pub mcp: Mcp,
    pub is_enabled: bool,
    pub auto_restart: bool,
    pub display_order: i32,
    pub created_at: String,
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
    pub claude_json: String, // ~/.claude.json - main config with global MCPs
    pub global_settings: String,
    pub plugins_dir: String,
}

// Commands (Slash Commands - user-invoked)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub allowed_tools: Option<Vec<String>>,
    pub argument_hint: Option<String>,
    pub model: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source: String,
    pub source_path: Option<String>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommandRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub allowed_tools: Option<Vec<String>>,
    pub argument_hint: Option<String>,
    pub model: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCommand {
    pub id: i64,
    pub command_id: i64,
    pub command: Command,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalCommand {
    pub id: i64,
    pub command_id: i64,
    pub command: Command,
    pub is_enabled: bool,
}

// Skills (Agent Skills - auto-invoked by Claude)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub allowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub disable_model_invocation: bool,
    pub tags: Option<Vec<String>>,
    pub source: String,
    pub source_path: Option<String>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSkillRequest {
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub allowed_tools: Option<Vec<String>>,
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
    pub file_type: String, // "reference", "asset", "script"
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
    pub source_path: Option<String>,
    pub is_favorite: bool,
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
    /// List of enabled editor IDs (e.g., ["claude_code", "opencode"])
    /// When an editor is enabled, skills/commands/subagents/MCPs sync to it
    pub enabled_editors: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // By default, only Claude Code is enabled
            enabled_editors: vec!["claude_code".to_string()],
        }
    }
}

// OpenCode paths (for OpenCode support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodePaths {
    pub config_dir: String,    // ~/.config/opencode/
    pub config_file: String,   // ~/.config/opencode/opencode.json
    pub command_dir: String,   // ~/.config/opencode/command/
    pub agent_dir: String,     // ~/.config/opencode/agent/
    pub plugin_dir: String,    // ~/.config/opencode/plugin/
    pub tool_dir: String,      // ~/.config/opencode/tool/
    pub knowledge_dir: String, // ~/.config/opencode/knowledge/
}

// Codex CLI paths (for Codex support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexPaths {
    pub config_dir: String,  // ~/.codex/
    pub config_file: String, // ~/.codex/config.toml
}

// GitHub Copilot CLI paths (for Copilot support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopilotPaths {
    pub config_dir: String,      // ~/.copilot/
    pub config_file: String,     // ~/.copilot/config.json
    pub mcp_config_file: String, // ~/.copilot/mcp-config.json
    pub agents_dir: String,      // ~/.copilot/agents/
}

// Cursor IDE paths (for Cursor support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPaths {
    pub config_dir: String,      // ~/.cursor/
    pub mcp_config_file: String, // ~/.cursor/mcp.json
}

// Gemini CLI paths (for Gemini CLI support)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPaths {
    pub config_dir: String,    // ~/.gemini/
    pub settings_file: String, // ~/.gemini/settings.json
}

// Editor info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    pub id: String,          // "claude_code" or "opencode"
    pub name: String,        // "Claude Code" or "OpenCode"
    pub is_installed: bool,  // Whether config directory exists
    pub is_enabled: bool,    // Whether syncing to this editor is enabled
    pub config_path: String, // Path to main config file
}

// Configuration Profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileItem {
    pub id: i64,
    pub profile_id: i64,
    pub item_type: String,
    pub item_id: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileWithItems {
    pub profile: Profile,
    pub mcps: Vec<i64>,
    pub skills: Vec<i64>,
    pub commands: Vec<i64>,
    pub subagents: Vec<i64>,
    pub hooks: Vec<i64>,
}

// Status Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusLine {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub statusline_type: String, // "custom", "premade", "raw"
    pub package_name: Option<String>,
    pub install_command: Option<String>,
    pub run_command: Option<String>,
    pub raw_command: Option<String>,
    pub padding: i32,
    pub is_active: bool,
    pub segments_json: Option<String>,
    pub generated_script: Option<String>,
    pub icon: Option<String>,
    pub author: Option<String>,
    pub homepage_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStatusLineRequest {
    pub name: String,
    pub description: Option<String>,
    pub statusline_type: String,
    pub package_name: Option<String>,
    pub install_command: Option<String>,
    pub run_command: Option<String>,
    pub raw_command: Option<String>,
    pub padding: Option<i32>,
    pub segments_json: Option<String>,
    pub generated_script: Option<String>,
    pub icon: Option<String>,
    pub author: Option<String>,
    pub homepage_url: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusLineSegment {
    pub id: String,
    #[serde(rename = "type")]
    pub segment_type: String, // model, cost, context, cwd, tokens_in, tokens_out, vim_mode, separator, custom_text
    pub enabled: bool,
    pub label: Option<String>,
    pub format: Option<String>,
    pub color: Option<String>,
    pub bg_color: Option<String>,
    pub separator_char: Option<String>,
    pub custom_text: Option<String>,
    pub position: i32,
}

/// Wrapper for segments_json that can include theme info.
/// Supports legacy format (plain array) and new format (object with theme).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentsPayload {
    #[serde(default = "default_theme")]
    pub theme: String,
    pub segments: Vec<StatusLineSegment>,
}

fn default_theme() -> String {
    "default".to_string()
}

impl SegmentsPayload {
    /// Parse segments_json which may be a plain array (legacy) or an object with theme
    pub fn parse(json: &str) -> Self {
        // Try new object format first
        if let Ok(payload) = serde_json::from_str::<SegmentsPayload>(json) {
            return payload;
        }
        // Fall back to legacy array format
        let segments: Vec<StatusLineSegment> = serde_json::from_str(json).unwrap_or_default();
        SegmentsPayload {
            theme: "default".to_string(),
            segments,
        }
    }

    pub fn is_powerline(&self) -> bool {
        self.theme == "powerline" || self.theme == "powerline_round"
    }

    pub fn is_powerline_round(&self) -> bool {
        self.theme == "powerline_round"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusLineGalleryEntry {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub homepage_url: Option<String>,
    pub install_command: Option<String>,
    pub run_command: Option<String>,
    pub package_name: Option<String>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
    pub preview_text: Option<String>,
}

// Permission Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionTemplate {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub rule: String,
    pub tool_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

// Spinner Verbs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpinnerVerb {
    pub id: i64,
    pub verb: String,
    pub is_enabled: bool,
    pub display_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpinnerVerbConfig {
    pub mode: String, // "append" or "replace"
    pub verbs: Vec<SpinnerVerb>,
}

// ========================================================================
// Docker Host Models
// ========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerHost {
    pub id: i64,
    pub name: String,
    pub host_type: String, // "local", "ssh", "tcp"
    pub connection_uri: Option<String>,
    pub ssh_key_path: Option<String>,
    pub tls_ca_cert: Option<String>,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDockerHostRequest {
    pub name: String,
    pub host_type: String,
    pub connection_uri: Option<String>,
    pub ssh_key_path: Option<String>,
    pub tls_ca_cert: Option<String>,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub is_default: Option<bool>,
}

// ========================================================================
// Container Models
// ========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub container_type: String, // "devcontainer", "docker", "custom"
    pub docker_host_id: i64,
    pub docker_container_id: Option<String>,
    pub image: Option<String>,
    pub dockerfile: Option<String>,
    pub devcontainer_json: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volumes: Option<Vec<VolumeMapping>>,
    pub mounts: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub post_create_command: Option<String>,
    pub post_start_command: Option<String>,
    pub working_dir: Option<String>,
    pub template_id: Option<String>,
    pub repo_url: Option<String>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Option<String>, // "tcp" or "udp"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
    pub read_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerStatus {
    pub container_id: i64,
    pub docker_status: String, // "running", "stopped", "exited", "created", "not_created", "unknown"
    pub docker_container_id: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub exit_code: Option<i64>,
    pub health: Option<String>,
    pub cpu_percent: Option<f64>,
    pub memory_usage: Option<u64>,
    pub memory_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerWithStatus {
    #[serde(flatten)]
    pub container: Container,
    pub status: ContainerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContainerRequest {
    pub name: String,
    pub description: Option<String>,
    pub container_type: String,
    pub docker_host_id: Option<i64>,
    pub image: Option<String>,
    pub dockerfile: Option<String>,
    pub devcontainer_json: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volumes: Option<Vec<VolumeMapping>>,
    pub mounts: Option<Vec<String>>,
    pub features: Option<Vec<String>>,
    pub post_create_command: Option<String>,
    pub post_start_command: Option<String>,
    pub working_dir: Option<String>,
    pub template_id: Option<String>,
    pub repo_url: Option<String>,
    pub icon: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectContainer {
    pub id: i64,
    pub project_id: i64,
    pub container_id: i64,
    pub container: Container,
    pub is_default: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon: String,
    pub image: String,
    pub dockerfile: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volumes: Option<Vec<VolumeMapping>>,
    pub features: Option<Vec<String>>,
    pub post_create_command: Option<String>,
    pub post_start_command: Option<String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerLog {
    pub timestamp: Option<String>,
    pub stream: String, // "stdout" or "stderr"
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerStats {
    pub container_id: i64,
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
    pub pids: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecResult {
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Mcp serde tests
    // =========================================================================

    #[test]
    fn test_mcp_serde_round_trip() {
        let mcp = Mcp {
            id: 1,
            name: "test-mcp".to_string(),
            description: Some("A test MCP".to_string()),
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "mcp-server".to_string()]),
            url: None,
            headers: None,
            env: Some({
                let mut m = HashMap::new();
                m.insert("KEY".to_string(), "value".to_string());
                m
            }),
            icon: Some("🔧".to_string()),
            tags: Some(vec!["test".to_string()]),
            source: "manual".to_string(),
            source_path: Some("/path".to_string()),
            is_enabled_global: true,
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&mcp).unwrap();
        let parsed: Mcp = serde_json::from_str(&json).unwrap();

        assert_eq!(mcp.id, parsed.id);
        assert_eq!(mcp.name, parsed.name);
        assert_eq!(mcp.mcp_type, parsed.mcp_type);
    }

    #[test]
    fn test_mcp_camel_case() {
        let json = r#"{
            "id": 1,
            "name": "test",
            "type": "stdio",
            "source": "manual",
            "isEnabledGlobal": true,
            "isFavorite": false,
            "createdAt": "2024",
            "updatedAt": "2024"
        }"#;

        let mcp: Mcp = serde_json::from_str(json).unwrap();
        assert!(mcp.is_enabled_global);
    }

    // =========================================================================
    // CreateMcpRequest serde tests
    // =========================================================================

    #[test]
    fn test_create_mcp_request_minimal() {
        let json = r#"{
            "name": "test",
            "type": "stdio"
        }"#;

        let req: CreateMcpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "test");
        assert_eq!(req.mcp_type, "stdio");
        assert!(req.description.is_none());
        assert!(req.command.is_none());
    }

    #[test]
    fn test_create_mcp_request_full() {
        let req = CreateMcpRequest {
            name: "full-mcp".to_string(),
            description: Some("Full MCP".to_string()),
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://example.com".to_string()),
            headers: Some({
                let mut m = HashMap::new();
                m.insert("Auth".to_string(), "Bearer token".to_string());
                m
            }),
            env: None,
            icon: Some("icon".to_string()),
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("full-mcp"));
        assert!(json.contains("Bearer token"));
    }

    // =========================================================================
    // Project serde tests
    // =========================================================================

    #[test]
    fn test_project_serde() {
        let project = Project {
            id: 1,
            name: "My Project".to_string(),
            path: "/path/to/project".to_string(),
            has_mcp_file: true,
            has_settings_file: false,
            last_scanned_at: Some("2024-01-01".to_string()),
            editor_type: "claude_code".to_string(),
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            assigned_mcps: vec![],
        };

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("hasMcpFile"));
        assert!(json.contains("hasSettingsFile"));
        assert!(json.contains("editorType"));
        assert!(json.contains("isFavorite"));
    }

    #[test]
    fn test_project_default_editor_type() {
        let json = r#"{
            "id": 1,
            "name": "test",
            "path": "/test",
            "hasMcpFile": false,
            "hasSettingsFile": false,
            "isFavorite": false,
            "createdAt": "2024",
            "updatedAt": "2024",
            "assignedMcps": []
        }"#;

        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.editor_type, "claude_code");
    }

    // =========================================================================
    // Command serde tests
    // =========================================================================

    #[test]
    fn test_command_serde() {
        let command = Command {
            id: 1,
            name: "my-command".to_string(),
            description: Some("My command".to_string()),
            content: "Content here".to_string(),
            allowed_tools: Some(vec!["Read".to_string(), "Write".to_string()]),
            argument_hint: Some("<filename>".to_string()),
            model: Some("sonnet".to_string()),
            tags: Some(vec!["test".to_string()]),
            source: "manual".to_string(),
            source_path: Some("/path/to/command.md".to_string()),
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&command).unwrap();
        let parsed: Command = serde_json::from_str(&json).unwrap();

        assert_eq!(command.name, parsed.name);
        assert_eq!(command.argument_hint, parsed.argument_hint);
        assert_eq!(command.source_path, parsed.source_path);
    }

    #[test]
    fn test_create_command_request() {
        let req = CreateCommandRequest {
            name: "test-command".to_string(),
            description: None,
            content: "# Command content".to_string(),
            allowed_tools: None,
            argument_hint: Some("<file>".to_string()),
            model: None,
            tags: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("argumentHint"));
    }

    // =========================================================================
    // Skill serde tests
    // =========================================================================

    #[test]
    fn test_skill_serde() {
        let skill = Skill {
            id: 1,
            name: "my-skill".to_string(),
            description: Some("My skill".to_string()),
            content: "Content here".to_string(),
            allowed_tools: Some(vec!["Read".to_string(), "Write".to_string()]),
            model: Some("sonnet".to_string()),
            disable_model_invocation: false,
            tags: Some(vec!["test".to_string()]),
            source: "manual".to_string(),
            source_path: Some("/path/to/skill".to_string()),
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&skill).unwrap();
        let parsed: Skill = serde_json::from_str(&json).unwrap();

        assert_eq!(skill.name, parsed.name);
        assert_eq!(
            skill.disable_model_invocation,
            parsed.disable_model_invocation
        );
        assert_eq!(skill.source_path, parsed.source_path);
    }

    #[test]
    fn test_create_skill_request() {
        let req = CreateSkillRequest {
            name: "test-skill".to_string(),
            description: None,
            content: "# Skill content".to_string(),
            allowed_tools: None,
            model: None,
            disable_model_invocation: Some(true),
            tags: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("disableModelInvocation"));
    }

    // =========================================================================
    // SubAgent serde tests
    // =========================================================================

    #[test]
    fn test_subagent_serde() {
        let agent = SubAgent {
            id: 1,
            name: "code-reviewer".to_string(),
            description: "Reviews code".to_string(),
            content: "You are a code reviewer".to_string(),
            tools: Some(vec!["Read".to_string(), "Grep".to_string()]),
            model: Some("opus".to_string()),
            permission_mode: Some("bypassPermissions".to_string()),
            skills: Some(vec!["lint".to_string()]),
            tags: Some(vec!["review".to_string()]),
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&agent).unwrap();
        let parsed: SubAgent = serde_json::from_str(&json).unwrap();

        assert_eq!(agent.name, parsed.name);
        assert_eq!(agent.permission_mode, parsed.permission_mode);
    }

    #[test]
    fn test_create_subagent_request() {
        let json = r#"{
            "name": "test-agent",
            "description": "Test",
            "content": "Content",
            "permissionMode": "askUser"
        }"#;

        let req: CreateSubAgentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "test-agent");
        assert_eq!(req.permission_mode, Some("askUser".to_string()));
    }

    // =========================================================================
    // Hook serde tests
    // =========================================================================

    #[test]
    fn test_hook_serde() {
        let hook = Hook {
            id: 1,
            name: "lint-hook".to_string(),
            description: Some("Run linter".to_string()),
            event_type: "PostToolUse".to_string(),
            matcher: Some("Write|Edit".to_string()),
            hook_type: "command".to_string(),
            command: Some("npm run lint".to_string()),
            prompt: None,
            timeout: Some(30000),
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&hook).unwrap();
        let parsed: Hook = serde_json::from_str(&json).unwrap();

        assert_eq!(hook.event_type, parsed.event_type);
        assert_eq!(hook.matcher, parsed.matcher);
        assert_eq!(hook.is_template, parsed.is_template);
    }

    #[test]
    fn test_create_hook_request_command() {
        let req = CreateHookRequest {
            name: "test-hook".to_string(),
            description: None,
            event_type: "PreToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("echo test".to_string()),
            prompt: None,
            timeout: None,
            tags: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("eventType"));
        assert!(json.contains("hookType"));
    }

    #[test]
    fn test_create_hook_request_prompt() {
        let req = CreateHookRequest {
            name: "prompt-hook".to_string(),
            description: Some("A prompt hook".to_string()),
            event_type: "Notification".to_string(),
            matcher: None,
            hook_type: "prompt".to_string(),
            command: None,
            prompt: Some("Please verify before proceeding".to_string()),
            timeout: None,
            tags: Some(vec!["safety".to_string()]),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("prompt"));
    }

    // =========================================================================
    // Repo and RepoItem serde tests
    // =========================================================================

    #[test]
    fn test_repo_serde() {
        let repo = Repo {
            id: 1,
            name: "anthropics/claude-code".to_string(),
            owner: "anthropics".to_string(),
            repo: "claude-code".to_string(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
            github_url: "https://github.com/anthropics/claude-code".to_string(),
            description: Some("Official repo".to_string()),
            is_default: true,
            is_enabled: true,
            last_fetched_at: None,
            etag: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&repo).unwrap();
        let parsed: Repo = serde_json::from_str(&json).unwrap();

        assert_eq!(repo.github_url, parsed.github_url);
        assert_eq!(repo.is_default, parsed.is_default);
    }

    #[test]
    fn test_repo_item_serde() {
        let item = RepoItem {
            id: 1,
            repo_id: 1,
            item_type: "skill".to_string(),
            name: "test-skill".to_string(),
            description: Some("A skill".to_string()),
            source_url: Some("https://github.com/...".to_string()),
            raw_content: Some("# Content".to_string()),
            file_path: Some("skills/test.md".to_string()),
            metadata: None,
            stars: Some(100),
            is_imported: false,
            imported_item_id: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("itemType"));
        assert!(json.contains("isImported"));
        assert!(json.contains("importedItemId"));
    }

    // =========================================================================
    // SyncResult and ImportResult serde tests
    // =========================================================================

    #[test]
    fn test_sync_result_serde() {
        let result = SyncResult {
            added: 5,
            updated: 3,
            removed: 1,
            errors: vec!["Error 1".to_string()],
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: SyncResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.added, parsed.added);
        assert_eq!(result.errors.len(), parsed.errors.len());
    }

    #[test]
    fn test_import_result_serde() {
        let result = ImportResult {
            success: true,
            item_type: "skill".to_string(),
            item_id: 42,
            message: Some("Imported successfully".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("itemType"));
        assert!(json.contains("itemId"));
    }

    // =========================================================================
    // AppSettings tests
    // =========================================================================

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.enabled_editors, vec!["claude_code".to_string()]);
    }

    #[test]
    fn test_app_settings_serde() {
        let settings = AppSettings {
            enabled_editors: vec!["claude_code".to_string(), "opencode".to_string()],
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("enabledEditors"));

        let parsed: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.enabled_editors.len(), 2);
        assert!(parsed.enabled_editors.contains(&"claude_code".to_string()));
        assert!(parsed.enabled_editors.contains(&"opencode".to_string()));
    }

    // =========================================================================
    // ClaudePaths and RateLimitInfo tests
    // =========================================================================

    #[test]
    fn test_claude_paths_serde() {
        let paths = ClaudePaths {
            claude_dir: "/home/user/.claude".to_string(),
            claude_json: "/home/user/.claude.json".to_string(),
            global_settings: "/home/user/.claude/settings.json".to_string(),
            plugins_dir: "/home/user/.claude/plugins".to_string(),
        };

        let json = serde_json::to_string(&paths).unwrap();
        assert!(json.contains("claudeDir"));
        assert!(json.contains("claudeJson"));
        assert!(json.contains("globalSettings"));
        assert!(json.contains("pluginsDir"));
    }

    #[test]
    fn test_rate_limit_info_serde() {
        let info = RateLimitInfo {
            limit: 60,
            remaining: 45,
            reset_at: "2024-01-01T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("resetAt"));

        let parsed: RateLimitInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.limit, 60);
        assert_eq!(parsed.remaining, 45);
    }

    // =========================================================================
    // SkillFile tests
    // =========================================================================

    #[test]
    fn test_skill_file_serde() {
        let file = SkillFile {
            id: 1,
            skill_id: 10,
            file_type: "reference".to_string(),
            name: "api-docs.md".to_string(),
            content: "# API Documentation".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&file).unwrap();
        assert!(json.contains("fileType"));
        assert!(json.contains("skillId"));
    }

    #[test]
    fn test_create_skill_file_request_serde() {
        let req = CreateSkillFileRequest {
            skill_id: 5,
            file_type: "asset".to_string(),
            name: "schema.json".to_string(),
            content: r#"{"type": "object"}"#.to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateSkillFileRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(req.skill_id, parsed.skill_id);
        assert_eq!(req.file_type, parsed.file_type);
    }

    // =========================================================================
    // Nested struct tests (ProjectMcp, GlobalMcp, etc.)
    // =========================================================================

    #[test]
    fn test_project_mcp_serde() {
        let mcp = Mcp {
            id: 1,
            name: "test".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("cmd".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            icon: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_enabled_global: false,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let project_mcp = ProjectMcp {
            id: 1,
            mcp_id: 1,
            mcp,
            is_enabled: true,
            env_overrides: Some({
                let mut m = HashMap::new();
                m.insert("OVERRIDE".to_string(), "value".to_string());
                m
            }),
            display_order: 1,
        };

        let json = serde_json::to_string(&project_mcp).unwrap();
        assert!(json.contains("mcpId"));
        assert!(json.contains("isEnabled"));
        assert!(json.contains("envOverrides"));
        assert!(json.contains("displayOrder"));
    }

    #[test]
    fn test_default_editor_type_function() {
        assert_eq!(default_editor_type(), "claude_code");
    }

    // =========================================================================
    // SegmentsPayload tests
    // =========================================================================

    #[test]
    fn test_segments_payload_parse_object_format() {
        let json = r#"{"theme":"powerline","segments":[{"id":"s1","type":"model","enabled":true,"position":0}]}"#;
        let payload = SegmentsPayload::parse(json);
        assert_eq!(payload.theme, "powerline");
        assert_eq!(payload.segments.len(), 1);
        assert_eq!(payload.segments[0].id, "s1");
    }

    #[test]
    fn test_segments_payload_parse_legacy_array_format() {
        let json = r#"[{"id":"s1","type":"model","enabled":true,"position":0}]"#;
        let payload = SegmentsPayload::parse(json);
        assert_eq!(payload.theme, "default");
        assert_eq!(payload.segments.len(), 1);
    }

    #[test]
    fn test_segments_payload_parse_invalid_json() {
        let payload = SegmentsPayload::parse("not json at all");
        assert_eq!(payload.theme, "default");
        assert!(payload.segments.is_empty());
    }

    #[test]
    fn test_segments_payload_is_powerline() {
        let mut payload = SegmentsPayload {
            theme: "powerline".to_string(),
            segments: vec![],
        };
        assert!(payload.is_powerline());
        assert!(!payload.is_powerline_round());

        payload.theme = "powerline_round".to_string();
        assert!(payload.is_powerline());
        assert!(payload.is_powerline_round());

        payload.theme = "default".to_string();
        assert!(!payload.is_powerline());
        assert!(!payload.is_powerline_round());
    }

    #[test]
    fn test_default_theme_function() {
        assert_eq!(default_theme(), "default");
    }

    // =========================================================================
    // StatusLineSegment serde tests
    // =========================================================================

    #[test]
    fn test_statusline_segment_serde() {
        let segment = StatusLineSegment {
            id: "seg1".to_string(),
            segment_type: "model".to_string(),
            enabled: true,
            label: Some("Model".to_string()),
            format: None,
            color: Some("#fff".to_string()),
            bg_color: None,
            separator_char: None,
            custom_text: None,
            position: 0,
        };

        let json = serde_json::to_string(&segment).unwrap();
        assert!(json.contains("\"type\""));
        assert!(json.contains("seg1"));

        let parsed: StatusLineSegment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.segment_type, "model");
        assert_eq!(parsed.position, 0);
    }

    // =========================================================================
    // SpinnerVerbConfig serde tests
    // =========================================================================

    #[test]
    fn test_spinner_verb_config_serde() {
        let config = SpinnerVerbConfig {
            mode: "replace".to_string(),
            verbs: vec![SpinnerVerb {
                id: 1,
                verb: "Thinking".to_string(),
                is_enabled: true,
                display_order: 0,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-01".to_string(),
            }],
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: SpinnerVerbConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mode, "replace");
        assert_eq!(parsed.verbs.len(), 1);
        assert_eq!(parsed.verbs[0].verb, "Thinking");
    }

    // =========================================================================
    // Docker / Container model serde tests
    // =========================================================================

    #[test]
    fn test_docker_host_serde() {
        let host = DockerHost {
            id: 1,
            name: "Local Docker".to_string(),
            host_type: "local".to_string(),
            connection_uri: None,
            ssh_key_path: None,
            tls_ca_cert: None,
            tls_cert: None,
            tls_key: None,
            is_default: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&host).unwrap();
        assert!(json.contains("hostType"));
        assert!(json.contains("isDefault"));

        let parsed: DockerHost = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.host_type, "local");
        assert!(parsed.is_default);
    }

    #[test]
    fn test_docker_host_ssh_type() {
        let host = DockerHost {
            id: 2,
            name: "Remote Host".to_string(),
            host_type: "ssh".to_string(),
            connection_uri: Some("ssh://user@host:22".to_string()),
            ssh_key_path: Some("/home/user/.ssh/id_rsa".to_string()),
            tls_ca_cert: None,
            tls_cert: None,
            tls_key: None,
            is_default: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&host).unwrap();
        let parsed: DockerHost = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.connection_uri,
            Some("ssh://user@host:22".to_string())
        );
        assert!(parsed.ssh_key_path.is_some());
    }

    #[test]
    fn test_create_docker_host_request_serde() {
        let req = CreateDockerHostRequest {
            name: "New Host".to_string(),
            host_type: "tcp".to_string(),
            connection_uri: Some("tcp://192.168.1.1:2375".to_string()),
            ssh_key_path: None,
            tls_ca_cert: Some("ca.pem".to_string()),
            tls_cert: Some("cert.pem".to_string()),
            tls_key: Some("key.pem".to_string()),
            is_default: Some(false),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("connectionUri"));
        assert!(json.contains("tlsCaCert"));

        let parsed: CreateDockerHostRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.host_type, "tcp");
    }

    #[test]
    fn test_port_mapping_serde() {
        let pm = PortMapping {
            host_port: 8080,
            container_port: 80,
            protocol: Some("tcp".to_string()),
        };

        let json = serde_json::to_string(&pm).unwrap();
        assert!(json.contains("hostPort"));
        assert!(json.contains("containerPort"));

        let parsed: PortMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pm);
    }

    #[test]
    fn test_volume_mapping_serde() {
        let vm = VolumeMapping {
            host_path: "/data".to_string(),
            container_path: "/app/data".to_string(),
            read_only: Some(true),
        };

        let json = serde_json::to_string(&vm).unwrap();
        assert!(json.contains("hostPath"));
        assert!(json.contains("readOnly"));

        let parsed: VolumeMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, vm);
    }

    #[test]
    fn test_container_serde() {
        let container = Container {
            id: 1,
            name: "dev-env".to_string(),
            description: Some("Development environment".to_string()),
            container_type: "devcontainer".to_string(),
            docker_host_id: 1,
            docker_container_id: Some("abc123".to_string()),
            image: Some("node:18".to_string()),
            dockerfile: None,
            devcontainer_json: Some(r#"{"image":"node:18"}"#.to_string()),
            env: Some({
                let mut m = HashMap::new();
                m.insert("NODE_ENV".to_string(), "development".to_string());
                m
            }),
            ports: Some(vec![PortMapping {
                host_port: 3000,
                container_port: 3000,
                protocol: None,
            }]),
            volumes: Some(vec![VolumeMapping {
                host_path: "/src".to_string(),
                container_path: "/app".to_string(),
                read_only: Some(false),
            }]),
            mounts: None,
            features: Some(vec!["git".to_string()]),
            post_create_command: Some("npm install".to_string()),
            post_start_command: None,
            working_dir: Some("/app".to_string()),
            template_id: None,
            icon: Some("🐳".to_string()),
            tags: Some(vec!["dev".to_string()]),
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&container).unwrap();
        assert!(json.contains("containerType"));
        assert!(json.contains("dockerHostId"));
        assert!(json.contains("postCreateCommand"));

        let parsed: Container = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "dev-env");
        assert_eq!(parsed.container_type, "devcontainer");
    }

    #[test]
    fn test_container_status_serde() {
        let status = ContainerStatus {
            container_id: 1,
            docker_status: "running".to_string(),
            docker_container_id: Some("abc123".to_string()),
            started_at: Some("2024-01-01T00:00:00Z".to_string()),
            finished_at: None,
            exit_code: None,
            health: Some("healthy".to_string()),
            cpu_percent: Some(5.2),
            memory_usage: Some(104857600),
            memory_limit: Some(2147483648),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("dockerStatus"));
        assert!(json.contains("cpuPercent"));

        let parsed: ContainerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.docker_status, "running");
    }

    #[test]
    fn test_container_with_status_serde() {
        let container = Container {
            id: 1,
            name: "test".to_string(),
            description: None,
            container_type: "docker".to_string(),
            docker_host_id: 1,
            docker_container_id: None,
            image: Some("alpine".to_string()),
            dockerfile: None,
            devcontainer_json: None,
            env: None,
            ports: None,
            volumes: None,
            mounts: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: None,
            template_id: None,
            icon: None,
            tags: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };
        let status = ContainerStatus {
            container_id: 1,
            docker_status: "stopped".to_string(),
            docker_container_id: None,
            started_at: None,
            finished_at: None,
            exit_code: Some(0),
            health: None,
            cpu_percent: None,
            memory_usage: None,
            memory_limit: None,
        };

        let cws = ContainerWithStatus { container, status };
        let json = serde_json::to_string(&cws).unwrap();
        // Should have flattened container fields plus status
        assert!(json.contains("\"name\""));
        assert!(json.contains("dockerStatus"));
    }

    #[test]
    fn test_create_container_request_serde() {
        let req = CreateContainerRequest {
            name: "new-container".to_string(),
            description: None,
            container_type: "docker".to_string(),
            docker_host_id: Some(1),
            image: Some("ubuntu:22.04".to_string()),
            dockerfile: None,
            devcontainer_json: None,
            env: None,
            ports: None,
            volumes: None,
            mounts: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: None,
            template_id: None,
            icon: None,
            tags: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateContainerRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "new-container");
    }

    #[test]
    fn test_container_template_serde() {
        let template = ContainerTemplate {
            id: "node-dev".to_string(),
            name: "Node.js Development".to_string(),
            description: "Node.js dev environment".to_string(),
            category: "language".to_string(),
            icon: "🟢".to_string(),
            image: "node:18".to_string(),
            dockerfile: None,
            env: None,
            ports: Some(vec![PortMapping {
                host_port: 3000,
                container_port: 3000,
                protocol: None,
            }]),
            volumes: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: Some("/workspace".to_string()),
        };

        let json = serde_json::to_string(&template).unwrap();
        let parsed: ContainerTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "node-dev");
        assert_eq!(parsed.category, "language");
    }

    #[test]
    fn test_container_log_serde() {
        let log = ContainerLog {
            timestamp: Some("2024-01-01T12:00:00Z".to_string()),
            stream: "stdout".to_string(),
            message: "Server started".to_string(),
        };

        let json = serde_json::to_string(&log).unwrap();
        let parsed: ContainerLog = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stream, "stdout");
        assert_eq!(parsed.message, "Server started");
    }

    #[test]
    fn test_container_stats_serde() {
        let stats = ContainerStats {
            container_id: 1,
            cpu_percent: 12.5,
            memory_usage: 524288000,
            memory_limit: 2147483648,
            memory_percent: 24.4,
            network_rx_bytes: 1024,
            network_tx_bytes: 2048,
            block_read_bytes: 4096,
            block_write_bytes: 8192,
            pids: 15,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("cpuPercent"));
        assert!(json.contains("memoryPercent"));
        assert!(json.contains("networkRxBytes"));

        let parsed: ContainerStats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pids, 15);
    }

    #[test]
    fn test_exec_result_serde() {
        let result = ExecResult {
            exit_code: 0,
            stdout: "hello\n".to_string(),
            stderr: "".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ExecResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.exit_code, 0);
        assert_eq!(parsed.stdout, "hello\n");
    }

    // =========================================================================
    // Editor/Path model serde tests
    // =========================================================================

    #[test]
    fn test_editor_info_serde() {
        let info = EditorInfo {
            id: "claude_code".to_string(),
            name: "Claude Code".to_string(),
            is_installed: true,
            is_enabled: true,
            config_path: "/home/user/.claude.json".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("isInstalled"));
        assert!(json.contains("isEnabled"));
        assert!(json.contains("configPath"));

        let parsed: EditorInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "claude_code");
    }

    #[test]
    fn test_opencode_paths_serde() {
        let paths = OpenCodePaths {
            config_dir: "/home/user/.config/opencode".to_string(),
            config_file: "/home/user/.config/opencode/opencode.json".to_string(),
            command_dir: "/home/user/.config/opencode/command".to_string(),
            agent_dir: "/home/user/.config/opencode/agent".to_string(),
            plugin_dir: "/home/user/.config/opencode/plugin".to_string(),
            tool_dir: "/home/user/.config/opencode/tool".to_string(),
            knowledge_dir: "/home/user/.config/opencode/knowledge".to_string(),
        };

        let json = serde_json::to_string(&paths).unwrap();
        assert!(json.contains("configDir"));
        assert!(json.contains("commandDir"));
        assert!(json.contains("knowledgeDir"));
    }

    #[test]
    fn test_codex_paths_serde() {
        let paths = CodexPaths {
            config_dir: "/home/user/.codex".to_string(),
            config_file: "/home/user/.codex/config.toml".to_string(),
        };

        let json = serde_json::to_string(&paths).unwrap();
        assert!(json.contains("configDir"));

        let parsed: CodexPaths = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.config_dir, "/home/user/.codex");
    }

    #[test]
    fn test_copilot_paths_serde() {
        let paths = CopilotPaths {
            config_dir: "/home/user/.copilot".to_string(),
            config_file: "/home/user/.copilot/config.json".to_string(),
            mcp_config_file: "/home/user/.copilot/mcp-config.json".to_string(),
            agents_dir: "/home/user/.copilot/agents".to_string(),
        };

        let json = serde_json::to_string(&paths).unwrap();
        assert!(json.contains("mcpConfigFile"));
        assert!(json.contains("agentsDir"));
    }

    #[test]
    fn test_cursor_paths_serde() {
        let paths = CursorPaths {
            config_dir: "/home/user/.cursor".to_string(),
            mcp_config_file: "/home/user/.cursor/mcp.json".to_string(),
        };

        let json = serde_json::to_string(&paths).unwrap();
        let parsed: CursorPaths = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mcp_config_file, "/home/user/.cursor/mcp.json");
    }

    #[test]
    fn test_gemini_paths_serde() {
        let paths = GeminiPaths {
            config_dir: "/home/user/.gemini".to_string(),
            settings_file: "/home/user/.gemini/settings.json".to_string(),
        };

        let json = serde_json::to_string(&paths).unwrap();
        assert!(json.contains("settingsFile"));

        let parsed: GeminiPaths = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.settings_file, "/home/user/.gemini/settings.json");
    }

    // =========================================================================
    // Profile model serde tests
    // =========================================================================

    #[test]
    fn test_profile_serde() {
        let profile = Profile {
            id: 1,
            name: "Development".to_string(),
            description: Some("Dev profile".to_string()),
            icon: Some("🛠".to_string()),
            is_active: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("isActive"));

        let parsed: Profile = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_active);
    }

    #[test]
    fn test_create_profile_request_serde() {
        let req = CreateProfileRequest {
            name: "Test Profile".to_string(),
            description: Some("A test".to_string()),
            icon: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateProfileRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test Profile");
    }

    #[test]
    fn test_profile_item_serde() {
        let item = ProfileItem {
            id: 1,
            profile_id: 1,
            item_type: "mcp".to_string(),
            item_id: 5,
            created_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("profileId"));
        assert!(json.contains("itemType"));
        assert!(json.contains("itemId"));
    }

    #[test]
    fn test_profile_with_items_serde() {
        let pwi = ProfileWithItems {
            profile: Profile {
                id: 1,
                name: "Full".to_string(),
                description: None,
                icon: None,
                is_active: false,
                created_at: "2024".to_string(),
                updated_at: "2024".to_string(),
            },
            mcps: vec![1, 2],
            skills: vec![3],
            commands: vec![],
            subagents: vec![4],
            hooks: vec![],
        };

        let json = serde_json::to_string(&pwi).unwrap();
        let parsed: ProfileWithItems = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mcps.len(), 2);
        assert_eq!(parsed.skills.len(), 1);
        assert_eq!(parsed.subagents.len(), 1);
    }

    // =========================================================================
    // StatusLine gallery / permission template tests
    // =========================================================================

    #[test]
    fn test_statusline_gallery_entry_serde() {
        let entry = StatusLineGalleryEntry {
            name: "Fancy Bar".to_string(),
            description: Some("A fancy bar".to_string()),
            author: Some("Author".to_string()),
            homepage_url: Some("https://example.com".to_string()),
            install_command: Some("npm install fancy-bar".to_string()),
            run_command: Some("fancy-bar run".to_string()),
            package_name: Some("fancy-bar".to_string()),
            icon: Some("✨".to_string()),
            tags: Some(vec!["fancy".to_string()]),
            preview_text: Some("▶ Model: opus | Cost: $0.01".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("homepageUrl"));
        assert!(json.contains("installCommand"));
        assert!(json.contains("previewText"));

        let parsed: StatusLineGalleryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Fancy Bar");
    }

    #[test]
    fn test_permission_template_serde() {
        let template = PermissionTemplate {
            id: 1,
            name: "Allow Read".to_string(),
            description: Some("Allow reading files".to_string()),
            category: "allow".to_string(),
            rule: "Read".to_string(),
            tool_name: Some("Read".to_string()),
            tags: Some(vec!["filesystem".to_string()]),
            is_default: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("toolName"));
        assert!(json.contains("isDefault"));

        let parsed: PermissionTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.category, "allow");
        assert!(parsed.is_default);
    }

    // =========================================================================
    // Global/Project association model serde tests
    // =========================================================================

    #[test]
    fn test_global_mcp_serde() {
        let mcp = Mcp {
            id: 1,
            name: "test".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("cmd".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            icon: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_enabled_global: false,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let global = GlobalMcp {
            id: 1,
            mcp_id: 1,
            mcp,
            is_enabled: true,
            env_overrides: None,
        };

        let json = serde_json::to_string(&global).unwrap();
        assert!(json.contains("mcpId"));
        assert!(json.contains("isEnabled"));
    }

    #[test]
    fn test_gateway_mcp_serde() {
        let mcp = Mcp {
            id: 1,
            name: "gw-test".to_string(),
            description: None,
            mcp_type: "stdio".to_string(),
            command: Some("cmd".to_string()),
            args: None,
            url: None,
            headers: None,
            env: None,
            icon: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_enabled_global: false,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let gw = GatewayMcp {
            id: 1,
            mcp_id: 1,
            mcp,
            is_enabled: true,
            auto_restart: true,
            display_order: 0,
            created_at: "2024".to_string(),
        };

        let json = serde_json::to_string(&gw).unwrap();
        assert!(json.contains("autoRestart"));
        assert!(json.contains("displayOrder"));

        let parsed: GatewayMcp = serde_json::from_str(&json).unwrap();
        assert!(parsed.auto_restart);
    }

    #[test]
    fn test_project_skill_serde() {
        let skill = Skill {
            id: 1,
            name: "sk".to_string(),
            description: None,
            content: "c".to_string(),
            allowed_tools: None,
            model: None,
            disable_model_invocation: false,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let ps = ProjectSkill {
            id: 1,
            skill_id: 1,
            skill,
            is_enabled: true,
        };

        let json = serde_json::to_string(&ps).unwrap();
        assert!(json.contains("skillId"));
        assert!(json.contains("isEnabled"));
    }

    #[test]
    fn test_global_skill_serde() {
        let skill = Skill {
            id: 1,
            name: "gs".to_string(),
            description: None,
            content: "c".to_string(),
            allowed_tools: None,
            model: None,
            disable_model_invocation: false,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let gs = GlobalSkill {
            id: 1,
            skill_id: 1,
            skill,
            is_enabled: true,
        };

        let json = serde_json::to_string(&gs).unwrap();
        assert!(json.contains("skillId"));
    }

    #[test]
    fn test_project_subagent_serde() {
        let agent = SubAgent {
            id: 1,
            name: "a".to_string(),
            description: "d".to_string(),
            content: "c".to_string(),
            tools: None,
            model: None,
            permission_mode: None,
            skills: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let psa = ProjectSubAgent {
            id: 1,
            subagent_id: 1,
            subagent: agent,
            is_enabled: true,
        };

        let json = serde_json::to_string(&psa).unwrap();
        assert!(json.contains("subagentId"));
    }

    #[test]
    fn test_global_subagent_serde() {
        let agent = SubAgent {
            id: 1,
            name: "a".to_string(),
            description: "d".to_string(),
            content: "c".to_string(),
            tools: None,
            model: None,
            permission_mode: None,
            skills: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let gsa = GlobalSubAgent {
            id: 1,
            subagent_id: 1,
            subagent: agent,
            is_enabled: true,
        };

        let json = serde_json::to_string(&gsa).unwrap();
        assert!(json.contains("subagentId"));
    }

    #[test]
    fn test_project_hook_serde() {
        let hook = Hook {
            id: 1,
            name: "h".to_string(),
            description: None,
            event_type: "PreToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: None,
            prompt: None,
            timeout: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let ph = ProjectHook {
            id: 1,
            hook_id: 1,
            hook,
            is_enabled: true,
        };

        let json = serde_json::to_string(&ph).unwrap();
        assert!(json.contains("hookId"));
    }

    #[test]
    fn test_global_hook_serde() {
        let hook = Hook {
            id: 1,
            name: "gh".to_string(),
            description: None,
            event_type: "PostToolUse".to_string(),
            matcher: None,
            hook_type: "prompt".to_string(),
            command: None,
            prompt: Some("Review".to_string()),
            timeout: None,
            tags: None,
            source: "manual".to_string(),
            is_template: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let gh = GlobalHook {
            id: 1,
            hook_id: 1,
            hook,
            is_enabled: true,
        };

        let json = serde_json::to_string(&gh).unwrap();
        assert!(json.contains("hookId"));
    }

    #[test]
    fn test_project_command_serde() {
        let command = Command {
            id: 1,
            name: "cmd".to_string(),
            description: None,
            content: "c".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let pc = ProjectCommand {
            id: 1,
            command_id: 1,
            command,
            is_enabled: true,
        };

        let json = serde_json::to_string(&pc).unwrap();
        assert!(json.contains("commandId"));
        assert!(json.contains("isEnabled"));
    }

    #[test]
    fn test_global_command_serde() {
        let command = Command {
            id: 1,
            name: "gc".to_string(),
            description: None,
            content: "c".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let gc = GlobalCommand {
            id: 1,
            command_id: 1,
            command,
            is_enabled: true,
        };

        let json = serde_json::to_string(&gc).unwrap();
        assert!(json.contains("commandId"));
    }

    #[test]
    fn test_project_container_serde() {
        let container = Container {
            id: 1,
            name: "c".to_string(),
            description: None,
            container_type: "docker".to_string(),
            docker_host_id: 1,
            docker_container_id: None,
            image: None,
            dockerfile: None,
            devcontainer_json: None,
            env: None,
            ports: None,
            volumes: None,
            mounts: None,
            features: None,
            post_create_command: None,
            post_start_command: None,
            working_dir: None,
            template_id: None,
            icon: None,
            tags: None,
            is_favorite: false,
            created_at: "2024".to_string(),
            updated_at: "2024".to_string(),
        };

        let pc = ProjectContainer {
            id: 1,
            project_id: 1,
            container_id: 1,
            container,
            is_default: true,
            created_at: "2024".to_string(),
        };

        let json = serde_json::to_string(&pc).unwrap();
        assert!(json.contains("projectId"));
        assert!(json.contains("containerId"));
        assert!(json.contains("isDefault"));
    }

    #[test]
    fn test_create_repo_request_serde() {
        let req = CreateRepoRequest {
            github_url: "https://github.com/user/repo".to_string(),
            repo_type: "file_based".to_string(),
            content_type: "skill".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("githubUrl"));
        assert!(json.contains("repoType"));
        assert!(json.contains("contentType"));

        let parsed: CreateRepoRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.github_url, "https://github.com/user/repo");
    }

    // =========================================================================
    // StatusLine model serde tests
    // =========================================================================

    #[test]
    fn test_statusline_serde() {
        let sl = StatusLine {
            id: 1,
            name: "My Bar".to_string(),
            description: Some("Custom bar".to_string()),
            statusline_type: "custom".to_string(),
            package_name: None,
            install_command: None,
            run_command: Some("my-bar".to_string()),
            raw_command: None,
            padding: 1,
            is_active: true,
            segments_json: Some("[]".to_string()),
            generated_script: None,
            icon: None,
            author: None,
            homepage_url: None,
            tags: None,
            source: "manual".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&sl).unwrap();
        assert!(json.contains("statuslineType"));
        assert!(json.contains("isActive"));
        assert!(json.contains("segmentsJson"));

        let parsed: StatusLine = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "My Bar");
        assert!(parsed.is_active);
    }

    #[test]
    fn test_create_statusline_request_serde() {
        let req = CreateStatusLineRequest {
            name: "New SL".to_string(),
            description: None,
            statusline_type: "premade".to_string(),
            package_name: Some("cool-bar".to_string()),
            install_command: Some("npm i cool-bar".to_string()),
            run_command: Some("cool-bar".to_string()),
            raw_command: None,
            padding: Some(2),
            segments_json: None,
            generated_script: None,
            icon: Some("🎨".to_string()),
            author: Some("Author".to_string()),
            homepage_url: Some("https://example.com".to_string()),
            tags: Some(vec!["premade".to_string()]),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("packageName"));
        assert!(json.contains("installCommand"));

        let parsed: CreateStatusLineRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.statusline_type, "premade");
    }

    // =========================================================================
    // Additional coverage tests
    // =========================================================================

    #[test]
    fn test_create_project_request_serde() {
        let req = CreateProjectRequest {
            name: "my-project".to_string(),
            path: "/home/user/projects/my-project".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateProjectRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "my-project");
        assert_eq!(parsed.path, "/home/user/projects/my-project");
    }

    #[test]
    fn test_spinner_verb_serde() {
        let verb = SpinnerVerb {
            id: 1,
            verb: "Contemplating".to_string(),
            is_enabled: true,
            display_order: 3,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };

        let json = serde_json::to_string(&verb).unwrap();
        assert!(json.contains("displayOrder"));
        assert!(json.contains("isEnabled"));

        let parsed: SpinnerVerb = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.verb, "Contemplating");
        assert_eq!(parsed.display_order, 3);
    }

    #[test]
    fn test_mcp_with_all_none_optionals() {
        let json = r#"{
            "id": 1,
            "name": "minimal",
            "type": "stdio",
            "source": "manual",
            "isEnabledGlobal": false,
            "isFavorite": false,
            "createdAt": "2024",
            "updatedAt": "2024"
        }"#;

        let mcp: Mcp = serde_json::from_str(json).unwrap();
        assert!(mcp.description.is_none());
        assert!(mcp.command.is_none());
        assert!(mcp.args.is_none());
        assert!(mcp.url.is_none());
        assert!(mcp.headers.is_none());
        assert!(mcp.env.is_none());
        assert!(mcp.icon.is_none());
        assert!(mcp.tags.is_none());
        assert!(mcp.source_path.is_none());
    }

    #[test]
    fn test_container_log_stderr() {
        let log = ContainerLog {
            timestamp: None,
            stream: "stderr".to_string(),
            message: "Error occurred".to_string(),
        };

        let json = serde_json::to_string(&log).unwrap();
        let parsed: ContainerLog = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stream, "stderr");
        assert!(parsed.timestamp.is_none());
    }

    #[test]
    fn test_exec_result_nonzero_exit() {
        let result = ExecResult {
            exit_code: 127,
            stdout: "".to_string(),
            stderr: "command not found".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ExecResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.exit_code, 127);
        assert_eq!(parsed.stderr, "command not found");
    }

    #[test]
    fn test_container_template_with_all_fields() {
        let template = ContainerTemplate {
            id: "full-template".to_string(),
            name: "Full Template".to_string(),
            description: "Template with all fields".to_string(),
            category: "tools".to_string(),
            icon: "🔧".to_string(),
            image: "ubuntu:22.04".to_string(),
            dockerfile: Some("FROM ubuntu:22.04\nRUN apt-get update".to_string()),
            env: Some({
                let mut m = HashMap::new();
                m.insert("LANG".to_string(), "en_US.UTF-8".to_string());
                m
            }),
            ports: Some(vec![
                PortMapping {
                    host_port: 8080,
                    container_port: 80,
                    protocol: Some("tcp".to_string()),
                },
                PortMapping {
                    host_port: 8443,
                    container_port: 443,
                    protocol: Some("tcp".to_string()),
                },
            ]),
            volumes: Some(vec![VolumeMapping {
                host_path: "/data".to_string(),
                container_path: "/app/data".to_string(),
                read_only: Some(false),
            }]),
            features: Some(vec!["git".to_string(), "docker-in-docker".to_string()]),
            post_create_command: Some("apt-get install -y curl".to_string()),
            post_start_command: Some("service nginx start".to_string()),
            working_dir: Some("/workspace".to_string()),
        };

        let json = serde_json::to_string(&template).unwrap();
        let parsed: ContainerTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.ports.unwrap().len(), 2);
        assert_eq!(parsed.features.unwrap().len(), 2);
        assert!(parsed.dockerfile.is_some());
        assert!(parsed.post_start_command.is_some());
    }

    #[test]
    fn test_create_container_request_full() {
        let req = CreateContainerRequest {
            name: "full-container".to_string(),
            description: Some("Full container".to_string()),
            container_type: "devcontainer".to_string(),
            docker_host_id: Some(2),
            image: Some("node:20".to_string()),
            dockerfile: Some("FROM node:20".to_string()),
            devcontainer_json: Some(r#"{"image":"node:20"}"#.to_string()),
            env: Some({
                let mut m = HashMap::new();
                m.insert("NODE_ENV".to_string(), "dev".to_string());
                m
            }),
            ports: Some(vec![PortMapping {
                host_port: 3000,
                container_port: 3000,
                protocol: None,
            }]),
            volumes: Some(vec![VolumeMapping {
                host_path: ".".to_string(),
                container_path: "/app".to_string(),
                read_only: None,
            }]),
            mounts: Some(vec!["/tmp:/tmp".to_string()]),
            features: Some(vec!["git".to_string()]),
            post_create_command: Some("npm install".to_string()),
            post_start_command: Some("npm run dev".to_string()),
            working_dir: Some("/app".to_string()),
            template_id: Some("node-dev".to_string()),
            icon: Some("🟢".to_string()),
            tags: Some(vec!["nodejs".to_string()]),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateContainerRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.mounts.unwrap().len(), 1);
        assert_eq!(parsed.template_id, Some("node-dev".to_string()));
    }

    #[test]
    fn test_port_mapping_without_protocol() {
        let pm = PortMapping {
            host_port: 5432,
            container_port: 5432,
            protocol: None,
        };

        let json = serde_json::to_string(&pm).unwrap();
        let parsed: PortMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, pm);
        assert!(parsed.protocol.is_none());
    }

    #[test]
    fn test_volume_mapping_without_read_only() {
        let vm = VolumeMapping {
            host_path: "/src".to_string(),
            container_path: "/app".to_string(),
            read_only: None,
        };

        let json = serde_json::to_string(&vm).unwrap();
        let parsed: VolumeMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, vm);
        assert!(parsed.read_only.is_none());
    }

    #[test]
    fn test_container_stats_zero_values() {
        let stats = ContainerStats {
            container_id: 1,
            cpu_percent: 0.0,
            memory_usage: 0,
            memory_limit: 0,
            memory_percent: 0.0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            block_read_bytes: 0,
            block_write_bytes: 0,
            pids: 0,
        };

        let json = serde_json::to_string(&stats).unwrap();
        let parsed: ContainerStats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.cpu_percent, 0.0);
        assert_eq!(parsed.pids, 0);
    }

    #[test]
    fn test_container_status_exited() {
        let status = ContainerStatus {
            container_id: 5,
            docker_status: "exited".to_string(),
            docker_container_id: Some("def456".to_string()),
            started_at: Some("2024-01-01T00:00:00Z".to_string()),
            finished_at: Some("2024-01-01T01:00:00Z".to_string()),
            exit_code: Some(1),
            health: None,
            cpu_percent: None,
            memory_usage: None,
            memory_limit: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        let parsed: ContainerStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.docker_status, "exited");
        assert_eq!(parsed.exit_code, Some(1));
        assert!(parsed.finished_at.is_some());
    }

    #[test]
    fn test_create_docker_host_request_minimal() {
        let json = r#"{
            "name": "Minimal Host",
            "hostType": "local"
        }"#;

        let req: CreateDockerHostRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Minimal Host");
        assert_eq!(req.host_type, "local");
        assert!(req.connection_uri.is_none());
        assert!(req.is_default.is_none());
    }

    #[test]
    fn test_statusline_gallery_entry_minimal() {
        let json = r#"{
            "name": "Minimal Bar"
        }"#;

        let entry: StatusLineGalleryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.name, "Minimal Bar");
        assert!(entry.description.is_none());
        assert!(entry.author.is_none());
    }

    #[test]
    fn test_create_skill_request_full() {
        let req = CreateSkillRequest {
            name: "full-skill".to_string(),
            description: Some("Full skill".to_string()),
            content: "Skill content".to_string(),
            allowed_tools: Some(vec![
                "Read".to_string(),
                "Write".to_string(),
                "Grep".to_string(),
            ]),
            model: Some("opus".to_string()),
            disable_model_invocation: Some(false),
            tags: Some(vec!["code".to_string(), "review".to_string()]),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateSkillRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.allowed_tools.unwrap().len(), 3);
        assert_eq!(parsed.tags.unwrap().len(), 2);
        assert_eq!(parsed.disable_model_invocation, Some(false));
    }

    #[test]
    fn test_create_subagent_request_full() {
        let req = CreateSubAgentRequest {
            name: "full-agent".to_string(),
            description: "Full agent".to_string(),
            content: "Agent content".to_string(),
            tools: Some(vec!["Read".to_string()]),
            model: Some("sonnet".to_string()),
            permission_mode: Some("bypassPermissions".to_string()),
            skills: Some(vec!["lint".to_string(), "test".to_string()]),
            tags: Some(vec!["agent".to_string()]),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: CreateSubAgentRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.skills.unwrap().len(), 2);
        assert_eq!(
            parsed.permission_mode,
            Some("bypassPermissions".to_string())
        );
    }

    #[test]
    fn test_segments_payload_roundtrip() {
        let payload = SegmentsPayload {
            theme: "powerline_round".to_string(),
            segments: vec![
                StatusLineSegment {
                    id: "s1".to_string(),
                    segment_type: "model".to_string(),
                    enabled: true,
                    label: Some("Model".to_string()),
                    format: Some("{model}".to_string()),
                    color: Some("#fff".to_string()),
                    bg_color: Some("#333".to_string()),
                    separator_char: Some("".to_string()),
                    custom_text: None,
                    position: 0,
                },
                StatusLineSegment {
                    id: "s2".to_string(),
                    segment_type: "custom".to_string(),
                    enabled: false,
                    label: None,
                    format: None,
                    color: None,
                    bg_color: None,
                    separator_char: None,
                    custom_text: Some("Hello".to_string()),
                    position: 1,
                },
            ],
        };

        let json = serde_json::to_string(&payload).unwrap();
        let parsed = SegmentsPayload::parse(&json);
        assert_eq!(parsed.theme, "powerline_round");
        assert_eq!(parsed.segments.len(), 2);
        assert!(parsed.is_powerline());
        assert!(parsed.is_powerline_round());
        assert!(parsed.segments[0].enabled);
        assert!(!parsed.segments[1].enabled);
    }
}
