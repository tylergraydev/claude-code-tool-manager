use crate::db::models::Skill;
use crate::utils::opencode_paths::get_opencode_paths;
use anyhow::Result;
use directories::BaseDirs;
use std::path::Path;

/// Generate markdown content for an agent skill (.claude/skills/name/SKILL.md)
pub(crate) fn generate_skill_markdown(skill: &Skill) -> String {
    let mut frontmatter = String::from("---\n");

    frontmatter.push_str(&format!("name: {}\n", skill.name));

    if let Some(ref desc) = skill.description {
        if !desc.is_empty() {
            frontmatter.push_str(&format!("description: {}\n", desc));
        }
    }

    if let Some(ref tools) = skill.allowed_tools {
        if !tools.is_empty() {
            frontmatter.push_str(&format!("allowed-tools: {}\n", tools.join(", ")));
        }
    }

    if let Some(ref model) = skill.model {
        if !model.is_empty() {
            frontmatter.push_str(&format!("model: {}\n", model));
        }
    }

    if skill.disable_model_invocation {
        frontmatter.push_str("disable-model-invocation: true\n");
    }

    if let Some(ref context) = skill.context {
        if !context.is_empty() {
            frontmatter.push_str(&format!("context: {}\n", context));
        }
    }
    if let Some(ref agent) = skill.agent {
        if !agent.is_empty() {
            frontmatter.push_str(&format!("agent: {}\n", agent));
        }
    }
    if let Some(ref shell) = skill.shell {
        if !shell.is_empty() {
            frontmatter.push_str(&format!("shell: {}\n", shell));
        }
    }
    if let Some(once) = skill.once {
        if once {
            frontmatter.push_str("once: true\n");
        }
    }
    if let Some(ref paths) = skill.paths {
        if !paths.is_empty() {
            frontmatter.push_str(&format!("paths: {}\n", paths.join(", ")));
        }
    }
    if let Some(ref hooks) = skill.hooks {
        if !hooks.is_empty() {
            frontmatter.push_str(&format!("hooks: {}\n", hooks));
        }
    }
    if let Some(ref effort) = skill.effort {
        if !effort.is_empty() {
            frontmatter.push_str(&format!("effort: {}\n", effort));
        }
    }

    frontmatter.push_str("---\n\n");
    format!("{}{}", frontmatter, skill.content)
}

/// Write a skill to the appropriate location
/// Skills go to {base_path}/.claude/skills/{name}/SKILL.md
pub fn write_skill_file(base_path: &Path, skill: &Skill) -> Result<()> {
    let skill_dir = base_path.join(".claude").join("skills").join(&skill.name);
    std::fs::create_dir_all(&skill_dir)?;

    let file_path = skill_dir.join("SKILL.md");
    crate::utils::backup::backup_file(&file_path)?;
    let content = generate_skill_markdown(skill);
    std::fs::write(file_path, content)?;

    Ok(())
}

/// Delete a skill file from the appropriate location
pub fn delete_skill_file(base_path: &Path, skill: &Skill) -> Result<()> {
    let skill_dir = base_path.join(".claude").join("skills").join(&skill.name);
    if skill_dir.exists() {
        std::fs::remove_dir_all(skill_dir)?;
    }

    Ok(())
}

/// Write a skill to the global Claude config (~/.claude/)
pub fn write_global_skill(skill: &Skill) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_skill_file(home, skill)
}

/// Delete a skill from the global Claude config (~/.claude/)
pub fn delete_global_skill(skill: &Skill) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    delete_skill_file(home, skill)
}

/// Write a skill to a project's Claude config ({project}/.claude/)
pub fn write_project_skill(project_path: &Path, skill: &Skill) -> Result<()> {
    write_skill_file(project_path, skill)
}

/// Delete a skill from a project's Claude config ({project}/.claude/)
pub fn delete_project_skill(project_path: &Path, skill: &Skill) -> Result<()> {
    delete_skill_file(project_path, skill)
}

// ============================================================================
// OpenCode Support
// ============================================================================

/// Write a skill to OpenCode's format
/// Agent skills go to {base_path}/agent/{name}.md (OpenCode uses agent/ not skills/)
pub fn write_skill_file_opencode(base_path: &Path, skill: &Skill) -> Result<()> {
    let agent_dir = base_path.join("agent");
    std::fs::create_dir_all(&agent_dir)?;

    let file_path = agent_dir.join(format!("{}.md", skill.name));
    crate::utils::backup::backup_file(&file_path)?;
    let content = generate_skill_markdown(skill);
    std::fs::write(file_path, content)?;

    Ok(())
}

/// Delete a skill from OpenCode's format
pub fn delete_skill_file_opencode(base_path: &Path, skill: &Skill) -> Result<()> {
    let file_path = base_path.join("agent").join(format!("{}.md", skill.name));
    if file_path.exists() {
        std::fs::remove_file(file_path)?;
    }

    Ok(())
}

/// Write a skill to the global OpenCode config (~/.config/opencode/)
pub fn write_global_skill_opencode(skill: &Skill) -> Result<()> {
    let paths = get_opencode_paths()?;
    write_skill_file_opencode(&paths.config_dir, skill)
}

/// Delete a skill from the global OpenCode config
pub fn delete_global_skill_opencode(skill: &Skill) -> Result<()> {
    let paths = get_opencode_paths()?;
    delete_skill_file_opencode(&paths.config_dir, skill)
}

/// Write a skill to a project's OpenCode config ({project}/.opencode/)
pub fn write_project_skill_opencode(project_path: &Path, skill: &Skill) -> Result<()> {
    let opencode_dir = project_path.join(".opencode");
    write_skill_file_opencode(&opencode_dir, skill)
}

/// Delete a skill from a project's OpenCode config
pub fn delete_project_skill_opencode(project_path: &Path, skill: &Skill) -> Result<()> {
    let opencode_dir = project_path.join(".opencode");
    delete_skill_file_opencode(&opencode_dir, skill)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // Helper functions to create test skills
    // =========================================================================

    fn sample_skill() -> Skill {
        Skill {
            id: 1,
            name: "test-agent".to_string(),
            description: Some("An agent skill".to_string()),
            content: "You are a helpful assistant.".to_string(),
            allowed_tools: Some(vec!["Bash".to_string(), "Glob".to_string()]),
            model: Some("opus".to_string()),
            disable_model_invocation: true,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            context: None,
            agent: None,
            hooks: None,
            paths: None,
            shell: None,
            once: None,
            effort: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    fn sample_minimal_skill() -> Skill {
        Skill {
            id: 2,
            name: "minimal".to_string(),
            description: None,
            content: "Minimal content.".to_string(),
            allowed_tools: None,
            model: None,
            disable_model_invocation: false,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_favorite: false,
            context: None,
            agent: None,
            hooks: None,
            paths: None,
            shell: None,
            once: None,
            effort: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    // =========================================================================
    // generate_skill_markdown tests
    // =========================================================================

    #[test]
    fn test_generate_skill_markdown_full() {
        let skill = sample_skill();
        let md = generate_skill_markdown(&skill);

        assert!(md.starts_with("---\n"));
        assert!(md.contains("name: test-agent\n"));
        assert!(md.contains("description: An agent skill\n"));
        assert!(md.contains("allowed-tools: Bash, Glob\n"));
        assert!(md.contains("model: opus\n"));
        assert!(md.contains("disable-model-invocation: true\n"));
        assert!(md.contains("---\n\nYou are a helpful assistant."));
    }

    #[test]
    fn test_generate_skill_markdown_minimal() {
        let skill = sample_minimal_skill();
        let md = generate_skill_markdown(&skill);

        assert!(md.contains("name: minimal\n"));
        assert!(!md.contains("description:"));
        assert!(!md.contains("allowed-tools:"));
        assert!(!md.contains("model:"));
        assert!(!md.contains("disable-model-invocation:"));
    }

    #[test]
    fn test_generate_skill_markdown_always_includes_name() {
        let skill = sample_minimal_skill();
        let md = generate_skill_markdown(&skill);

        assert!(md.contains("name: minimal\n"));
    }

    // =========================================================================
    // write_skill_file tests (file system)
    // =========================================================================

    #[test]
    fn test_write_skill_file_creates_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();

        write_skill_file(temp_dir.path(), &skill).unwrap();

        let expected_path = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("test-agent")
            .join("SKILL.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_write_skill_file_content_matches() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();

        write_skill_file(temp_dir.path(), &skill).unwrap();

        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("test-agent")
            .join("SKILL.md");
        let content = std::fs::read_to_string(file_path).unwrap();

        assert!(content.contains("name: test-agent"));
        assert!(content.contains("You are a helpful assistant."));
    }

    // =========================================================================
    // delete_skill_file tests
    // =========================================================================

    #[test]
    fn test_delete_skill_file_removes_directory() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();

        // Write first
        write_skill_file(temp_dir.path(), &skill).unwrap();
        let skill_dir = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("test-agent");
        assert!(skill_dir.exists());

        // Delete
        delete_skill_file(temp_dir.path(), &skill).unwrap();
        assert!(!skill_dir.exists());
    }

    #[test]
    fn test_delete_nonexistent_skill_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();

        // Should not error when file doesn't exist
        let result = delete_skill_file(temp_dir.path(), &skill);
        assert!(result.is_ok());
    }

    // =========================================================================
    // OpenCode format tests
    // =========================================================================

    #[test]
    fn test_write_skill_file_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();

        write_skill_file_opencode(temp_dir.path(), &skill).unwrap();

        // OpenCode uses "agent" not "skills"
        let expected_path = temp_dir.path().join("agent").join("test-agent.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_delete_skill_file_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();

        write_skill_file_opencode(temp_dir.path(), &skill).unwrap();
        let file_path = temp_dir.path().join("agent").join("test-agent.md");
        assert!(file_path.exists());

        delete_skill_file_opencode(temp_dir.path(), &skill).unwrap();
        assert!(!file_path.exists());
    }

    // =========================================================================
    // Additional coverage: edge cases
    // =========================================================================
    #[test]
    fn test_generate_skill_markdown_empty_description() {
        let mut skill = sample_skill();
        skill.description = Some("".to_string());
        let md = generate_skill_markdown(&skill);
        assert!(!md.contains("description:"));
    }

    #[test]
    fn test_generate_skill_markdown_empty_tools() {
        let mut skill = sample_skill();
        skill.allowed_tools = Some(vec![]);
        let md = generate_skill_markdown(&skill);
        assert!(!md.contains("allowed-tools:"));
    }

    #[test]
    fn test_generate_skill_markdown_empty_model() {
        let mut skill = sample_skill();
        skill.model = Some("".to_string());
        let md = generate_skill_markdown(&skill);
        assert!(!md.contains("model:"));
    }

    #[test]
    fn test_write_project_skill() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();
        write_project_skill(temp_dir.path(), &skill).unwrap();

        let expected = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("test-agent")
            .join("SKILL.md");
        assert!(expected.exists());
    }

    #[test]
    fn test_delete_project_skill() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();
        write_project_skill(temp_dir.path(), &skill).unwrap();
        delete_project_skill(temp_dir.path(), &skill).unwrap();

        let expected = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("test-agent");
        assert!(!expected.exists());
    }

    #[test]
    fn test_write_project_skill_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();
        write_project_skill_opencode(temp_dir.path(), &skill).unwrap();

        let expected = temp_dir
            .path()
            .join(".opencode")
            .join("agent")
            .join("test-agent.md");
        assert!(expected.exists());
    }

    #[test]
    fn test_delete_project_skill_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();
        write_project_skill_opencode(temp_dir.path(), &skill).unwrap();
        delete_project_skill_opencode(temp_dir.path(), &skill).unwrap();

        let expected = temp_dir
            .path()
            .join(".opencode")
            .join("agent")
            .join("test-agent.md");
        assert!(!expected.exists());
    }

    #[test]
    fn test_delete_skill_file_opencode_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();
        // Should not error
        let result = delete_skill_file_opencode(temp_dir.path(), &skill);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_skill_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let mut skill = sample_skill();
        write_skill_file(temp_dir.path(), &skill).unwrap();

        // Update content and write again
        skill.content = "Updated content".to_string();
        write_skill_file(temp_dir.path(), &skill).unwrap();

        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("test-agent")
            .join("SKILL.md");
        let content = std::fs::read_to_string(file_path).unwrap();
        assert!(content.contains("Updated content"));
    }
}
