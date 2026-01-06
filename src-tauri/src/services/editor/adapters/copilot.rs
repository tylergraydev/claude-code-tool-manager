//! GitHub Copilot CLI editor adapter (stub).
//!
//! This adapter is a placeholder for future Copilot CLI support.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::db::models::{Command, Skill};
use crate::services::config_writer::McpTuple;
use crate::services::editor::{EditorAdapter, EditorPaths};

/// Adapter for GitHub Copilot CLI.
pub struct CopilotAdapter;

impl CopilotAdapter {
    fn not_implemented<T>(&self) -> Result<T> {
        bail!("Copilot CLI support is not yet implemented")
    }
}

impl EditorAdapter for CopilotAdapter {
    fn id(&self) -> &str {
        "copilot"
    }

    fn name(&self) -> &str {
        "GitHub Copilot"
    }

    fn is_installed(&self) -> bool {
        // TODO: Check for Copilot CLI installation
        // Check for gh copilot extension or similar
        false
    }

    fn get_paths(&self) -> Result<EditorPaths> {
        // Placeholder paths - will be updated when implemented
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Ok(EditorPaths {
            config_dir: home.join(".config").join("gh-copilot"),
            global_config: home.join(".config").join("gh-copilot").join("config.json"),
            skills_dir: home.join(".config").join("gh-copilot").join("agents"),
            commands_dir: home.join(".config").join("gh-copilot").join("commands"),
            agents_dir: None,
        })
    }

    fn project_config_filename(&self) -> &str {
        ".copilot.json"
    }

    fn write_skill(&self, _base_path: &Path, _skill: &Skill) -> Result<()> {
        self.not_implemented()
    }

    fn delete_skill(&self, _base_path: &Path, _skill: &Skill) -> Result<()> {
        self.not_implemented()
    }

    fn write_global_skill(&self, _skill: &Skill) -> Result<()> {
        self.not_implemented()
    }

    fn delete_global_skill(&self, _skill: &Skill) -> Result<()> {
        self.not_implemented()
    }

    fn write_project_skill(&self, _project_path: &Path, _skill: &Skill) -> Result<()> {
        self.not_implemented()
    }

    fn delete_project_skill(&self, _project_path: &Path, _skill: &Skill) -> Result<()> {
        self.not_implemented()
    }

    fn write_command(&self, _base_path: &Path, _command: &Command) -> Result<()> {
        self.not_implemented()
    }

    fn delete_command(&self, _base_path: &Path, _command: &Command) -> Result<()> {
        self.not_implemented()
    }

    fn write_global_command(&self, _command: &Command) -> Result<()> {
        self.not_implemented()
    }

    fn delete_global_command(&self, _command: &Command) -> Result<()> {
        self.not_implemented()
    }

    fn write_project_command(&self, _project_path: &Path, _command: &Command) -> Result<()> {
        self.not_implemented()
    }

    fn delete_project_command(&self, _project_path: &Path, _command: &Command) -> Result<()> {
        self.not_implemented()
    }

    fn write_global_mcp_config(&self, _mcps: &[McpTuple]) -> Result<()> {
        self.not_implemented()
    }

    fn write_project_mcp_config(&self, _project_path: &Path, _mcps: &[McpTuple]) -> Result<()> {
        self.not_implemented()
    }

    fn read_global_mcp_config(&self) -> Result<Vec<McpTuple>> {
        self.not_implemented()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copilot_adapter_identity() {
        let adapter = CopilotAdapter;
        assert_eq!(adapter.id(), "copilot");
        assert_eq!(adapter.name(), "GitHub Copilot");
        assert!(!adapter.is_installed());
    }
}
