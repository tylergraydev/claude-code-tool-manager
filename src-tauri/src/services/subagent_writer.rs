use crate::db::models::SubAgent;
use crate::utils::opencode_paths::get_opencode_paths;
use anyhow::Result;
use directories::BaseDirs;
use std::path::Path;

/// Editor type for routing agent writes
pub enum EditorType {
    ClaudeCode,
    OpenCode,
}

/// Generate markdown content for a sub-agent (.claude/agents/name.md)
pub(crate) fn generate_subagent_markdown(subagent: &SubAgent) -> String {
    let mut frontmatter = String::from("---\n");

    frontmatter.push_str(&format!("name: {}\n", subagent.name));
    frontmatter.push_str(&format!("description: {}\n", subagent.description));

    if let Some(ref tools) = subagent.tools {
        if !tools.is_empty() {
            frontmatter.push_str(&format!("tools: {}\n", tools.join(", ")));
        }
    }

    if let Some(ref model) = subagent.model {
        if !model.is_empty() {
            frontmatter.push_str(&format!("model: {}\n", model));
        }
    }

    if let Some(ref permission_mode) = subagent.permission_mode {
        if !permission_mode.is_empty() {
            frontmatter.push_str(&format!("permissionMode: {}\n", permission_mode));
        }
    }

    if let Some(ref skills) = subagent.skills {
        if !skills.is_empty() {
            frontmatter.push_str(&format!("skills: {}\n", skills.join(", ")));
        }
    }

    frontmatter.push_str("---\n\n");
    format!("{}{}", frontmatter, subagent.content)
}

/// Write a sub-agent to {base_path}/.claude/agents/{name}.md
pub fn write_subagent_file(base_path: &Path, subagent: &SubAgent) -> Result<()> {
    let agents_dir = base_path.join(".claude").join("agents");
    std::fs::create_dir_all(&agents_dir)?;

    let file_path = agents_dir.join(format!("{}.md", subagent.name));
    let content = generate_subagent_markdown(subagent);
    std::fs::write(file_path, content)?;

    Ok(())
}

/// Delete a sub-agent file from {base_path}/.claude/agents/{name}.md
pub fn delete_subagent_file(base_path: &Path, name: &str) -> Result<()> {
    let file_path = base_path
        .join(".claude")
        .join("agents")
        .join(format!("{}.md", name));
    if file_path.exists() {
        std::fs::remove_file(file_path)?;
    }
    Ok(())
}

/// Write a sub-agent to the global Claude config (~/.claude/agents/)
pub fn write_global_subagent(subagent: &SubAgent) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_subagent_file(home, subagent)
}

/// Delete a sub-agent from the global Claude config (~/.claude/agents/)
pub fn delete_global_subagent(name: &str) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    delete_subagent_file(home, name)
}

/// Write a sub-agent to a project's Claude config ({project}/.claude/agents/)
pub fn write_project_subagent(project_path: &Path, subagent: &SubAgent) -> Result<()> {
    write_subagent_file(project_path, subagent)
}

/// Delete a sub-agent from a project's Claude config ({project}/.claude/agents/)
pub fn delete_project_subagent(project_path: &Path, name: &str) -> Result<()> {
    delete_subagent_file(project_path, name)
}

// ============================================================================
// OpenCode Support
// ============================================================================

/// Write a sub-agent to OpenCode's format
/// OpenCode uses {base_path}/agent/{name}.md (singular "agent")
pub fn write_subagent_file_opencode(base_path: &Path, subagent: &SubAgent) -> Result<()> {
    let agents_dir = base_path.join("agent"); // OpenCode uses singular
    std::fs::create_dir_all(&agents_dir)?;

    let file_path = agents_dir.join(format!("{}.md", subagent.name));
    let content = generate_subagent_markdown(subagent);
    std::fs::write(file_path, content)?;

    Ok(())
}

/// Delete a sub-agent file from OpenCode's format
pub fn delete_subagent_file_opencode(base_path: &Path, name: &str) -> Result<()> {
    let file_path = base_path.join("agent").join(format!("{}.md", name));
    if file_path.exists() {
        std::fs::remove_file(file_path)?;
    }
    Ok(())
}

/// Write a sub-agent to the global OpenCode config (~/.config/opencode/agent/)
pub fn write_global_subagent_opencode(subagent: &SubAgent) -> Result<()> {
    let paths = get_opencode_paths()?;
    write_subagent_file_opencode(&paths.config_dir, subagent)
}

/// Delete a sub-agent from the global OpenCode config
pub fn delete_global_subagent_opencode(name: &str) -> Result<()> {
    let paths = get_opencode_paths()?;
    delete_subagent_file_opencode(&paths.config_dir, name)
}

/// Write a sub-agent to a project's OpenCode config ({project}/.opencode/agent/)
pub fn write_project_subagent_opencode(project_path: &Path, subagent: &SubAgent) -> Result<()> {
    let opencode_dir = project_path.join(".opencode");
    write_subagent_file_opencode(&opencode_dir, subagent)
}

/// Delete a sub-agent from a project's OpenCode config
pub fn delete_project_subagent_opencode(project_path: &Path, name: &str) -> Result<()> {
    let opencode_dir = project_path.join(".opencode");
    delete_subagent_file_opencode(&opencode_dir, name)
}

/// Write a sub-agent based on editor type
pub fn write_subagent_for_editor(
    base_path: &Path,
    subagent: &SubAgent,
    editor: EditorType,
) -> Result<()> {
    match editor {
        EditorType::ClaudeCode => write_subagent_file(base_path, subagent),
        EditorType::OpenCode => {
            let opencode_dir = base_path.join(".opencode");
            write_subagent_file_opencode(&opencode_dir, subagent)
        }
    }
}

/// Delete a sub-agent based on editor type
pub fn delete_subagent_for_editor(base_path: &Path, name: &str, editor: EditorType) -> Result<()> {
    match editor {
        EditorType::ClaudeCode => delete_subagent_file(base_path, name),
        EditorType::OpenCode => {
            let opencode_dir = base_path.join(".opencode");
            delete_subagent_file_opencode(&opencode_dir, name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // Helper functions to create test subagents
    // =========================================================================

    fn sample_full_subagent() -> SubAgent {
        SubAgent {
            id: 1,
            name: "code-reviewer".to_string(),
            description: "Reviews code for bugs and improvements".to_string(),
            content: "You are a code review expert. Analyze code for bugs, security issues, and best practices.".to_string(),
            tools: Some(vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()]),
            model: Some("sonnet".to_string()),
            permission_mode: Some("bypassPermissions".to_string()),
            skills: Some(vec!["lint".to_string(), "format".to_string()]),
            tags: Some(vec!["review".to_string(), "quality".to_string()]),
            source: "manual".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    fn sample_minimal_subagent() -> SubAgent {
        SubAgent {
            id: 2,
            name: "simple-agent".to_string(),
            description: "A simple agent".to_string(),
            content: "You are a helpful assistant.".to_string(),
            tools: None,
            model: None,
            permission_mode: None,
            skills: None,
            tags: None,
            source: "manual".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    // =========================================================================
    // generate_subagent_markdown tests
    // =========================================================================

    #[test]
    fn test_generate_subagent_markdown_full() {
        let subagent = sample_full_subagent();
        let md = generate_subagent_markdown(&subagent);

        assert!(md.starts_with("---\n"));
        assert!(md.contains("name: code-reviewer\n"));
        assert!(md.contains("description: Reviews code for bugs and improvements\n"));
        assert!(md.contains("tools: Read, Grep, Glob\n"));
        assert!(md.contains("model: sonnet\n"));
        assert!(md.contains("permissionMode: bypassPermissions\n"));
        assert!(md.contains("skills: lint, format\n"));
        assert!(md.contains("---\n\nYou are a code review expert."));
    }

    #[test]
    fn test_generate_subagent_markdown_minimal() {
        let subagent = sample_minimal_subagent();
        let md = generate_subagent_markdown(&subagent);

        assert!(md.contains("name: simple-agent\n"));
        assert!(md.contains("description: A simple agent\n"));
        assert!(!md.contains("tools:"));
        assert!(!md.contains("model:"));
        assert!(!md.contains("permissionMode:"));
        assert!(!md.contains("skills:"));
        assert!(md.contains("---\n\nYou are a helpful assistant."));
    }

    #[test]
    fn test_generate_subagent_markdown_empty_tools_skipped() {
        let mut subagent = sample_full_subagent();
        subagent.tools = Some(vec![]);
        let md = generate_subagent_markdown(&subagent);

        assert!(!md.contains("tools:"));
    }

    #[test]
    fn test_generate_subagent_markdown_empty_model_skipped() {
        let mut subagent = sample_full_subagent();
        subagent.model = Some("".to_string());
        let md = generate_subagent_markdown(&subagent);

        assert!(!md.contains("model:"));
    }

    #[test]
    fn test_generate_subagent_markdown_empty_permission_mode_skipped() {
        let mut subagent = sample_full_subagent();
        subagent.permission_mode = Some("".to_string());
        let md = generate_subagent_markdown(&subagent);

        assert!(!md.contains("permissionMode:"));
    }

    #[test]
    fn test_generate_subagent_markdown_empty_skills_skipped() {
        let mut subagent = sample_full_subagent();
        subagent.skills = Some(vec![]);
        let md = generate_subagent_markdown(&subagent);

        assert!(!md.contains("skills:"));
    }

    // =========================================================================
    // write_subagent_file tests
    // =========================================================================

    #[test]
    fn test_write_subagent_file_creates_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        write_subagent_file(temp_dir.path(), &subagent).unwrap();

        let expected_path = temp_dir
            .path()
            .join(".claude")
            .join("agents")
            .join("code-reviewer.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_write_subagent_file_content_matches() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        write_subagent_file(temp_dir.path(), &subagent).unwrap();

        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("agents")
            .join("code-reviewer.md");
        let content = std::fs::read_to_string(file_path).unwrap();

        assert!(content.contains("name: code-reviewer"));
        assert!(content.contains("You are a code review expert."));
    }

    #[test]
    fn test_write_subagent_file_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_minimal_subagent();

        // Directory doesn't exist yet
        let agents_dir = temp_dir.path().join(".claude").join("agents");
        assert!(!agents_dir.exists());

        write_subagent_file(temp_dir.path(), &subagent).unwrap();

        assert!(agents_dir.exists());
    }

    // =========================================================================
    // delete_subagent_file tests
    // =========================================================================

    #[test]
    fn test_delete_subagent_file() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        // Write first
        write_subagent_file(temp_dir.path(), &subagent).unwrap();
        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("agents")
            .join("code-reviewer.md");
        assert!(file_path.exists());

        // Delete
        delete_subagent_file(temp_dir.path(), &subagent.name).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_subagent_succeeds() {
        let temp_dir = TempDir::new().unwrap();

        // Should not error when file doesn't exist
        let result = delete_subagent_file(temp_dir.path(), "nonexistent");
        assert!(result.is_ok());
    }

    // =========================================================================
    // OpenCode format tests
    // =========================================================================

    #[test]
    fn test_write_subagent_file_opencode_creates_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        write_subagent_file_opencode(temp_dir.path(), &subagent).unwrap();

        // OpenCode uses singular "agent" not "agents"
        let expected_path = temp_dir.path().join("agent").join("code-reviewer.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_delete_subagent_file_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        write_subagent_file_opencode(temp_dir.path(), &subagent).unwrap();
        let file_path = temp_dir.path().join("agent").join("code-reviewer.md");
        assert!(file_path.exists());

        delete_subagent_file_opencode(temp_dir.path(), &subagent.name).unwrap();
        assert!(!file_path.exists());
    }

    // =========================================================================
    // Editor type routing tests
    // =========================================================================

    #[test]
    fn test_write_subagent_for_editor_claude_code() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        write_subagent_for_editor(temp_dir.path(), &subagent, EditorType::ClaudeCode).unwrap();

        let expected_path = temp_dir
            .path()
            .join(".claude")
            .join("agents")
            .join("code-reviewer.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_write_subagent_for_editor_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        write_subagent_for_editor(temp_dir.path(), &subagent, EditorType::OpenCode).unwrap();

        let expected_path = temp_dir
            .path()
            .join(".opencode")
            .join("agent")
            .join("code-reviewer.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_delete_subagent_for_editor_claude_code() {
        let temp_dir = TempDir::new().unwrap();
        let subagent = sample_full_subagent();

        write_subagent_for_editor(temp_dir.path(), &subagent, EditorType::ClaudeCode).unwrap();
        delete_subagent_for_editor(temp_dir.path(), &subagent.name, EditorType::ClaudeCode)
            .unwrap();

        let expected_path = temp_dir
            .path()
            .join(".claude")
            .join("agents")
            .join("code-reviewer.md");
        assert!(!expected_path.exists());
    }
}
