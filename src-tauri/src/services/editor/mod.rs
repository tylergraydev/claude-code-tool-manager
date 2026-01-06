//! Editor abstraction layer for multi-CLI tool support.
//!
//! This module provides a trait-based abstraction for supporting multiple
//! CLI tools (Claude Code, OpenCode, Codex, Copilot, Cursor, Gemini).

pub mod adapters;
pub mod registry;

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::db::models::{Command, Skill};
use crate::services::config_writer::McpTuple;

/// Paths configuration for an editor
#[derive(Debug, Clone)]
pub struct EditorPaths {
    /// Base config directory (e.g., ~/.claude/ or ~/.config/opencode/)
    pub config_dir: PathBuf,
    /// Global config file path (e.g., ~/.claude.json)
    pub global_config: PathBuf,
    /// Directory for skills/agents
    pub skills_dir: PathBuf,
    /// Directory for commands
    pub commands_dir: PathBuf,
    /// Optional separate agents directory
    pub agents_dir: Option<PathBuf>,
}

/// Information about an editor for display purposes
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    pub id: String,
    pub name: String,
    pub is_installed: bool,
    pub config_path: Option<String>,
}

/// Trait for editor-specific operations.
///
/// Each supported CLI tool implements this trait to provide
/// consistent operations across different editors.
pub trait EditorAdapter: Send + Sync {
    // =========================================================================
    // Identity
    // =========================================================================

    /// Unique identifier for this editor (e.g., "claude_code", "opencode")
    fn id(&self) -> &str;

    /// Human-readable name (e.g., "Claude Code", "OpenCode")
    fn name(&self) -> &str;

    /// Check if this editor is installed on the system
    fn is_installed(&self) -> bool;

    /// Get editor info for frontend display
    fn info(&self) -> EditorInfo {
        EditorInfo {
            id: self.id().to_string(),
            name: self.name().to_string(),
            is_installed: self.is_installed(),
            config_path: self.get_paths().ok().map(|p| p.config_dir.to_string_lossy().to_string()),
        }
    }

    // =========================================================================
    // Paths
    // =========================================================================

    /// Get all paths for this editor
    fn get_paths(&self) -> Result<EditorPaths>;

    /// Get the global config file path
    fn global_config_path(&self) -> Result<PathBuf> {
        Ok(self.get_paths()?.global_config)
    }

    /// Get the project config filename (e.g., ".mcp.json" or "opencode.json")
    fn project_config_filename(&self) -> &str;

    // =========================================================================
    // Skills/Agents
    // =========================================================================

    /// Write a skill to the filesystem
    fn write_skill(&self, base_path: &Path, skill: &Skill) -> Result<()>;

    /// Delete a skill from the filesystem
    fn delete_skill(&self, base_path: &Path, skill: &Skill) -> Result<()>;

    /// Write a global skill (to user's home config)
    fn write_global_skill(&self, skill: &Skill) -> Result<()>;

    /// Delete a global skill
    fn delete_global_skill(&self, skill: &Skill) -> Result<()>;

    /// Write a project-level skill
    fn write_project_skill(&self, project_path: &Path, skill: &Skill) -> Result<()>;

    /// Delete a project-level skill
    fn delete_project_skill(&self, project_path: &Path, skill: &Skill) -> Result<()>;

    // =========================================================================
    // Commands
    // =========================================================================

    /// Write a command to the filesystem
    fn write_command(&self, base_path: &Path, command: &Command) -> Result<()>;

    /// Delete a command from the filesystem
    fn delete_command(&self, base_path: &Path, command: &Command) -> Result<()>;

    /// Write a global command
    fn write_global_command(&self, command: &Command) -> Result<()>;

    /// Delete a global command
    fn delete_global_command(&self, command: &Command) -> Result<()>;

    /// Write a project-level command
    fn write_project_command(&self, project_path: &Path, command: &Command) -> Result<()>;

    /// Delete a project-level command
    fn delete_project_command(&self, project_path: &Path, command: &Command) -> Result<()>;

    // =========================================================================
    // MCP Config
    // =========================================================================

    /// Write MCPs to global config
    fn write_global_mcp_config(&self, mcps: &[McpTuple]) -> Result<()>;

    /// Write MCPs to project config
    fn write_project_mcp_config(&self, project_path: &Path, mcps: &[McpTuple]) -> Result<()>;

    /// Read MCPs from global config
    fn read_global_mcp_config(&self) -> Result<Vec<McpTuple>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_paths_creation() {
        let paths = EditorPaths {
            config_dir: PathBuf::from("/test/config"),
            global_config: PathBuf::from("/test/config.json"),
            skills_dir: PathBuf::from("/test/skills"),
            commands_dir: PathBuf::from("/test/commands"),
            agents_dir: None,
        };

        assert_eq!(paths.config_dir, PathBuf::from("/test/config"));
        assert!(paths.agents_dir.is_none());
    }

    #[test]
    fn test_editor_info_serialization() {
        let info = EditorInfo {
            id: "test".to_string(),
            name: "Test Editor".to_string(),
            is_installed: true,
            config_path: Some("/test/path".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"id\":\"test\""));
        assert!(json.contains("\"isInstalled\":true"));
    }
}
