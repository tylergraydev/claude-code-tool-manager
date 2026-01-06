//! Editor registry for managing available editor adapters.

use std::collections::HashMap;
use std::sync::Arc;

use super::{EditorAdapter, EditorInfo};

/// Registry of all available editor adapters.
///
/// The registry maintains a collection of editor adapters and provides
/// methods to access them by ID or list all available/installed editors.
pub struct EditorRegistry {
    adapters: HashMap<String, Arc<dyn EditorAdapter>>,
    default_id: String,
}

impl EditorRegistry {
    /// Create a new empty registry with a default editor ID
    pub fn new(default_id: &str) -> Self {
        Self {
            adapters: HashMap::new(),
            default_id: default_id.to_string(),
        }
    }

    /// Register an editor adapter
    pub fn register(&mut self, adapter: Arc<dyn EditorAdapter>) {
        self.adapters.insert(adapter.id().to_string(), adapter);
    }

    /// Get an adapter by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn EditorAdapter>> {
        self.adapters.get(id).cloned()
    }

    /// Get the default adapter (Claude Code)
    pub fn default_adapter(&self) -> Option<Arc<dyn EditorAdapter>> {
        self.get(&self.default_id)
    }

    /// Get adapter for an editor type, falling back to default if not found
    pub fn get_or_default(&self, id: &str) -> Option<Arc<dyn EditorAdapter>> {
        self.get(id).or_else(|| self.default_adapter())
    }

    /// List all registered adapters
    pub fn list_all(&self) -> Vec<Arc<dyn EditorAdapter>> {
        self.adapters.values().cloned().collect()
    }

    /// List only installed adapters
    pub fn list_installed(&self) -> Vec<Arc<dyn EditorAdapter>> {
        self.adapters
            .values()
            .filter(|a| a.is_installed())
            .cloned()
            .collect()
    }

    /// Get info about all registered editors
    pub fn get_all_info(&self) -> Vec<EditorInfo> {
        self.adapters.values().map(|a| a.info()).collect()
    }

    /// Get info about installed editors only
    pub fn get_installed_info(&self) -> Vec<EditorInfo> {
        self.adapters
            .values()
            .filter(|a| a.is_installed())
            .map(|a| a.info())
            .collect()
    }

    /// Check if an editor ID is valid (registered)
    pub fn is_valid_editor(&self, id: &str) -> bool {
        self.adapters.contains_key(id)
    }

    /// Get list of valid editor IDs
    pub fn valid_editor_ids(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }
}

impl Default for EditorRegistry {
    fn default() -> Self {
        Self::new("claude_code")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Command, Skill};
    use crate::services::config_writer::McpTuple;
    use crate::services::editor::EditorPaths;
    use anyhow::Result;
    use std::path::{Path, PathBuf};

    // Mock adapter for testing
    struct MockAdapter {
        id: String,
        name: String,
        installed: bool,
    }

    impl MockAdapter {
        fn new(id: &str, name: &str, installed: bool) -> Self {
            Self {
                id: id.to_string(),
                name: name.to_string(),
                installed,
            }
        }
    }

    impl EditorAdapter for MockAdapter {
        fn id(&self) -> &str {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn is_installed(&self) -> bool {
            self.installed
        }

        fn get_paths(&self) -> Result<EditorPaths> {
            Ok(EditorPaths {
                config_dir: PathBuf::from("/mock"),
                global_config: PathBuf::from("/mock/config.json"),
                skills_dir: PathBuf::from("/mock/skills"),
                commands_dir: PathBuf::from("/mock/commands"),
                agents_dir: None,
            })
        }

        fn project_config_filename(&self) -> &str {
            ".mock.json"
        }

        fn write_skill(&self, _base_path: &Path, _skill: &Skill) -> Result<()> {
            Ok(())
        }

        fn delete_skill(&self, _base_path: &Path, _skill: &Skill) -> Result<()> {
            Ok(())
        }

        fn write_global_skill(&self, _skill: &Skill) -> Result<()> {
            Ok(())
        }

        fn delete_global_skill(&self, _skill: &Skill) -> Result<()> {
            Ok(())
        }

        fn write_project_skill(&self, _project_path: &Path, _skill: &Skill) -> Result<()> {
            Ok(())
        }

        fn delete_project_skill(&self, _project_path: &Path, _skill: &Skill) -> Result<()> {
            Ok(())
        }

        fn write_command(&self, _base_path: &Path, _command: &Command) -> Result<()> {
            Ok(())
        }

        fn delete_command(&self, _base_path: &Path, _command: &Command) -> Result<()> {
            Ok(())
        }

        fn write_global_command(&self, _command: &Command) -> Result<()> {
            Ok(())
        }

        fn delete_global_command(&self, _command: &Command) -> Result<()> {
            Ok(())
        }

        fn write_project_command(&self, _project_path: &Path, _command: &Command) -> Result<()> {
            Ok(())
        }

        fn delete_project_command(&self, _project_path: &Path, _command: &Command) -> Result<()> {
            Ok(())
        }

        fn write_global_mcp_config(&self, _mcps: &[McpTuple]) -> Result<()> {
            Ok(())
        }

        fn write_project_mcp_config(&self, _project_path: &Path, _mcps: &[McpTuple]) -> Result<()> {
            Ok(())
        }

        fn read_global_mcp_config(&self) -> Result<Vec<McpTuple>> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = EditorRegistry::new("claude_code");
        assert_eq!(registry.default_id, "claude_code");
        assert!(registry.adapters.is_empty());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = EditorRegistry::new("test");
        let adapter = Arc::new(MockAdapter::new("test", "Test", true));

        registry.register(adapter);

        let retrieved = registry.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), "test");
    }

    #[test]
    fn test_registry_get_nonexistent() {
        let registry = EditorRegistry::new("claude_code");
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_list_installed() {
        let mut registry = EditorRegistry::new("claude_code");
        registry.register(Arc::new(MockAdapter::new("installed", "Installed", true)));
        registry.register(Arc::new(MockAdapter::new(
            "not_installed",
            "Not Installed",
            false,
        )));

        let installed = registry.list_installed();
        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].id(), "installed");
    }

    #[test]
    fn test_registry_is_valid_editor() {
        let mut registry = EditorRegistry::new("claude_code");
        registry.register(Arc::new(MockAdapter::new("test", "Test", true)));

        assert!(registry.is_valid_editor("test"));
        assert!(!registry.is_valid_editor("invalid"));
    }

    #[test]
    fn test_registry_get_or_default() {
        let mut registry = EditorRegistry::new("default");
        registry.register(Arc::new(MockAdapter::new("default", "Default", true)));
        registry.register(Arc::new(MockAdapter::new("other", "Other", true)));

        // Get existing
        let adapter = registry.get_or_default("other");
        assert!(adapter.is_some());
        assert_eq!(adapter.unwrap().id(), "other");

        // Fall back to default
        let adapter = registry.get_or_default("nonexistent");
        assert!(adapter.is_some());
        assert_eq!(adapter.unwrap().id(), "default");
    }
}
