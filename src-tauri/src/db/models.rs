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
}
