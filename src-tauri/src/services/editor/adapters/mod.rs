//! Editor adapter implementations.
//!
//! Each adapter implements the EditorAdapter trait for a specific CLI tool.

mod claude_code;
mod codex;
mod copilot;
mod cursor;
mod gemini;
mod opencode;

pub use claude_code::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use copilot::CopilotAdapter;
pub use cursor::CursorAdapter;
pub use gemini::GeminiAdapter;
pub use opencode::OpenCodeAdapter;

use super::registry::EditorRegistry;
use std::sync::Arc;

/// Create a registry with all available editor adapters
pub fn create_default_registry() -> EditorRegistry {
    let mut registry = EditorRegistry::new("claude_code");

    // Register all adapters
    registry.register(Arc::new(ClaudeCodeAdapter));
    registry.register(Arc::new(OpenCodeAdapter));
    registry.register(Arc::new(CodexAdapter));
    registry.register(Arc::new(CopilotAdapter));
    registry.register(Arc::new(CursorAdapter));
    registry.register(Arc::new(GeminiAdapter));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_registry() {
        let registry = create_default_registry();

        // Should have all 6 editors
        assert!(registry.is_valid_editor("claude_code"));
        assert!(registry.is_valid_editor("opencode"));
        assert!(registry.is_valid_editor("codex"));
        assert!(registry.is_valid_editor("copilot"));
        assert!(registry.is_valid_editor("cursor"));
        assert!(registry.is_valid_editor("gemini"));

        // Default should be claude_code
        let default = registry.default_adapter();
        assert!(default.is_some());
        assert_eq!(default.unwrap().id(), "claude_code");
    }
}
