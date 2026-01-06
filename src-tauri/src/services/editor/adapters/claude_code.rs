//! Claude Code editor adapter.
//!
//! This is the primary editor adapter, wrapping the existing
//! skill_writer, command_writer, and config_writer functionality.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::db::models::{Command, Skill};
use crate::services::command_writer;
use crate::services::config_writer::{self, McpTuple};
use crate::services::editor::{EditorAdapter, EditorPaths};
use crate::services::skill_writer;
use crate::utils::paths::get_claude_paths;

/// Adapter for Claude Code CLI.
pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    /// Check if Claude Code is installed by looking for config files
    fn check_installed() -> bool {
        if let Ok(paths) = get_claude_paths() {
            // Check for either the config file or the config directory
            paths.claude_json.exists() || paths.claude_dir.exists()
        } else {
            false
        }
    }
}

impl EditorAdapter for ClaudeCodeAdapter {
    // =========================================================================
    // Identity
    // =========================================================================

    fn id(&self) -> &str {
        "claude_code"
    }

    fn name(&self) -> &str {
        "Claude Code"
    }

    fn is_installed(&self) -> bool {
        Self::check_installed()
    }

    // =========================================================================
    // Paths
    // =========================================================================

    fn get_paths(&self) -> Result<EditorPaths> {
        let paths = get_claude_paths()?;
        Ok(EditorPaths {
            config_dir: paths.claude_dir,
            global_config: paths.claude_json,
            skills_dir: paths.skills_dir,
            commands_dir: paths.commands_dir,
            agents_dir: Some(paths.agents_dir),
        })
    }

    fn project_config_filename(&self) -> &str {
        ".mcp.json"
    }

    // =========================================================================
    // Skills
    // =========================================================================

    fn write_skill(&self, base_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::write_skill_file(base_path, skill)
    }

    fn delete_skill(&self, base_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::delete_skill_file(base_path, skill)
    }

    fn write_global_skill(&self, skill: &Skill) -> Result<()> {
        skill_writer::write_global_skill(skill)
    }

    fn delete_global_skill(&self, skill: &Skill) -> Result<()> {
        skill_writer::delete_global_skill(skill)
    }

    fn write_project_skill(&self, project_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::write_project_skill(project_path, skill)
    }

    fn delete_project_skill(&self, project_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::delete_project_skill(project_path, skill)
    }

    // =========================================================================
    // Commands
    // =========================================================================

    fn write_command(&self, base_path: &Path, command: &Command) -> Result<()> {
        command_writer::write_command_file(base_path, command)
    }

    fn delete_command(&self, base_path: &Path, command: &Command) -> Result<()> {
        command_writer::delete_command_file(base_path, command)
    }

    fn write_global_command(&self, command: &Command) -> Result<()> {
        command_writer::write_global_command(command)
    }

    fn delete_global_command(&self, command: &Command) -> Result<()> {
        command_writer::delete_global_command(command)
    }

    fn write_project_command(&self, project_path: &Path, command: &Command) -> Result<()> {
        command_writer::write_project_command(project_path, command)
    }

    fn delete_project_command(&self, project_path: &Path, command: &Command) -> Result<()> {
        command_writer::delete_project_command(project_path, command)
    }

    // =========================================================================
    // MCP Config
    // =========================================================================

    fn write_global_mcp_config(&self, mcps: &[McpTuple]) -> Result<()> {
        let paths = get_claude_paths()?;
        config_writer::write_global_config(&paths, mcps)
    }

    fn write_project_mcp_config(&self, project_path: &Path, mcps: &[McpTuple]) -> Result<()> {
        config_writer::write_project_config(project_path, mcps)
    }

    fn read_global_mcp_config(&self) -> Result<Vec<McpTuple>> {
        // TODO: Implement reading from ~/.claude.json
        // For now, return empty - this will be implemented when needed
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_code_adapter_identity() {
        let adapter = ClaudeCodeAdapter;
        assert_eq!(adapter.id(), "claude_code");
        assert_eq!(adapter.name(), "Claude Code");
    }

    #[test]
    fn test_claude_code_adapter_project_config_filename() {
        let adapter = ClaudeCodeAdapter;
        assert_eq!(adapter.project_config_filename(), ".mcp.json");
    }

    #[test]
    fn test_claude_code_adapter_paths() {
        let adapter = ClaudeCodeAdapter;
        // This test may fail if home dir is not available, but that's expected
        if let Ok(paths) = adapter.get_paths() {
            assert!(paths.config_dir.to_string_lossy().contains(".claude"));
            assert!(paths.skills_dir.to_string_lossy().contains("skills"));
            assert!(paths.commands_dir.to_string_lossy().contains("commands"));
        }
    }

    #[test]
    fn test_claude_code_adapter_info() {
        let adapter = ClaudeCodeAdapter;
        let info = adapter.info();
        assert_eq!(info.id, "claude_code");
        assert_eq!(info.name, "Claude Code");
    }
}
