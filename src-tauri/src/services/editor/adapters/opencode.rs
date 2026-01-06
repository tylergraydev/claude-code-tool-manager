//! OpenCode editor adapter.
//!
//! Wraps the existing opencode_config and related functionality.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::db::models::{Command, Skill};
use crate::services::command_writer;
use crate::services::config_writer::McpTuple;
use crate::services::editor::{EditorAdapter, EditorPaths};
use crate::services::opencode_config;
use crate::services::skill_writer;
use crate::utils::opencode_paths::{get_opencode_paths, is_opencode_installed};

/// Adapter for OpenCode CLI.
pub struct OpenCodeAdapter;

impl EditorAdapter for OpenCodeAdapter {
    // =========================================================================
    // Identity
    // =========================================================================

    fn id(&self) -> &str {
        "opencode"
    }

    fn name(&self) -> &str {
        "OpenCode"
    }

    fn is_installed(&self) -> bool {
        is_opencode_installed()
    }

    // =========================================================================
    // Paths
    // =========================================================================

    fn get_paths(&self) -> Result<EditorPaths> {
        let paths = get_opencode_paths()?;
        Ok(EditorPaths {
            config_dir: paths.config_dir,
            global_config: paths.config_file,
            skills_dir: paths.agent_dir.clone(), // OpenCode uses "agent" for skills
            commands_dir: paths.command_dir,
            agents_dir: Some(paths.agent_dir),
        })
    }

    fn project_config_filename(&self) -> &str {
        "opencode.json"
    }

    // =========================================================================
    // Skills (OpenCode calls these "agents")
    // =========================================================================

    fn write_skill(&self, base_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::write_skill_file_opencode(base_path, skill)
    }

    fn delete_skill(&self, base_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::delete_skill_file_opencode(base_path, skill)
    }

    fn write_global_skill(&self, skill: &Skill) -> Result<()> {
        skill_writer::write_global_skill_opencode(skill)
    }

    fn delete_global_skill(&self, skill: &Skill) -> Result<()> {
        skill_writer::delete_global_skill_opencode(skill)
    }

    fn write_project_skill(&self, project_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::write_project_skill_opencode(project_path, skill)
    }

    fn delete_project_skill(&self, project_path: &Path, skill: &Skill) -> Result<()> {
        skill_writer::delete_project_skill_opencode(project_path, skill)
    }

    // =========================================================================
    // Commands
    // =========================================================================

    fn write_command(&self, base_path: &Path, command: &Command) -> Result<()> {
        command_writer::write_command_file_opencode(base_path, command)
    }

    fn delete_command(&self, base_path: &Path, command: &Command) -> Result<()> {
        command_writer::delete_command_file_opencode(base_path, command)
    }

    fn write_global_command(&self, command: &Command) -> Result<()> {
        command_writer::write_global_command_opencode(command)
    }

    fn delete_global_command(&self, command: &Command) -> Result<()> {
        command_writer::delete_global_command_opencode(command)
    }

    fn write_project_command(&self, project_path: &Path, command: &Command) -> Result<()> {
        command_writer::write_project_command_opencode(project_path, command)
    }

    fn delete_project_command(&self, project_path: &Path, command: &Command) -> Result<()> {
        command_writer::delete_project_command_opencode(project_path, command)
    }

    // =========================================================================
    // MCP Config
    // =========================================================================

    fn write_global_mcp_config(&self, mcps: &[McpTuple]) -> Result<()> {
        opencode_config::write_opencode_global_config(mcps)
    }

    fn write_project_mcp_config(&self, project_path: &Path, mcps: &[McpTuple]) -> Result<()> {
        opencode_config::write_opencode_project_config(project_path, mcps)
    }

    fn read_global_mcp_config(&self) -> Result<Vec<McpTuple>> {
        // TODO: Implement reading from ~/.config/opencode/opencode.json
        // For now, return empty
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencode_adapter_identity() {
        let adapter = OpenCodeAdapter;
        assert_eq!(adapter.id(), "opencode");
        assert_eq!(adapter.name(), "OpenCode");
    }

    #[test]
    fn test_opencode_adapter_project_config_filename() {
        let adapter = OpenCodeAdapter;
        assert_eq!(adapter.project_config_filename(), "opencode.json");
    }

    #[test]
    fn test_opencode_adapter_info() {
        let adapter = OpenCodeAdapter;
        let info = adapter.info();
        assert_eq!(info.id, "opencode");
        assert_eq!(info.name, "OpenCode");
    }
}
