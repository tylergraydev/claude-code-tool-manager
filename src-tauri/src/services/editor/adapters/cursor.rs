//! Cursor CLI editor adapter (stub).
//!
//! This adapter is a placeholder for future Cursor CLI support.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::db::models::{Command, Skill};
use crate::services::config_writer::McpTuple;
use crate::services::editor::{EditorAdapter, EditorPaths};

/// Adapter for Cursor CLI.
pub struct CursorAdapter;

impl CursorAdapter {
    fn not_implemented<T>(&self) -> Result<T> {
        bail!("Cursor CLI support is not yet implemented")
    }
}

impl EditorAdapter for CursorAdapter {
    fn id(&self) -> &str {
        "cursor"
    }

    fn name(&self) -> &str {
        "Cursor"
    }

    fn is_installed(&self) -> bool {
        // TODO: Check for Cursor installation
        // Check for ~/.cursor/ or Cursor application
        false
    }

    fn get_paths(&self) -> Result<EditorPaths> {
        // Placeholder paths - will be updated when implemented
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Ok(EditorPaths {
            config_dir: home.join(".cursor"),
            global_config: home.join(".cursor").join("settings.json"),
            skills_dir: home.join(".cursor").join("agents"),
            commands_dir: home.join(".cursor").join("commands"),
            agents_dir: None,
        })
    }

    fn project_config_filename(&self) -> &str {
        ".cursor.json"
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
    fn test_cursor_adapter_identity() {
        let adapter = CursorAdapter;
        assert_eq!(adapter.id(), "cursor");
        assert_eq!(adapter.name(), "Cursor");
        assert!(!adapter.is_installed());
    }
}
