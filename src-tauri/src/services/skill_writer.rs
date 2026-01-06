use crate::db::models::{Skill, SkillFile};
use crate::utils::opencode_paths::get_opencode_paths;
use anyhow::Result;
use directories::BaseDirs;
use std::path::Path;

/// Editor type for routing skill writes
pub enum EditorType {
    ClaudeCode,
    OpenCode,
}

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

    frontmatter.push_str("---\n\n");
    format!("{}{}", frontmatter, skill.content)
}

/// Write a skill to the appropriate location
/// Skills go to {base_path}/.claude/skills/{name}/SKILL.md
pub fn write_skill_file(base_path: &Path, skill: &Skill) -> Result<()> {
    let skill_dir = base_path.join(".claude").join("skills").join(&skill.name);
    std::fs::create_dir_all(&skill_dir)?;

    let file_path = skill_dir.join("SKILL.md");
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

// Skill Files (references, assets, scripts)

/// Get the directory name for a file type
pub(crate) fn file_type_to_dir(file_type: &str) -> &str {
    match file_type {
        "reference" => "references",
        "asset" => "assets",
        "script" => "scripts",
        _ => "assets", // Default fallback
    }
}

/// Write a skill file to the skill directory
pub fn write_skill_subfile(base_path: &Path, skill: &Skill, file: &SkillFile) -> Result<()> {
    let skill_dir = base_path.join(".claude").join("skills").join(&skill.name);
    let type_dir = skill_dir.join(file_type_to_dir(&file.file_type));
    std::fs::create_dir_all(&type_dir)?;

    let file_path = type_dir.join(&file.name);
    std::fs::write(file_path, &file.content)?;

    Ok(())
}

/// Delete a skill file from the skill directory
pub fn delete_skill_subfile(base_path: &Path, skill: &Skill, file: &SkillFile) -> Result<()> {
    let skill_dir = base_path.join(".claude").join("skills").join(&skill.name);
    let type_dir = skill_dir.join(file_type_to_dir(&file.file_type));
    let file_path = type_dir.join(&file.name);

    if file_path.exists() {
        std::fs::remove_file(&file_path)?;
    }

    // Clean up empty directories
    if type_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&type_dir) {
            if entries.count() == 0 {
                let _ = std::fs::remove_dir(&type_dir);
            }
        }
    }

    Ok(())
}

/// Write all skill files for a skill
pub fn write_skill_files(base_path: &Path, skill: &Skill, files: &[SkillFile]) -> Result<()> {
    for file in files {
        write_skill_subfile(base_path, skill, file)?;
    }
    Ok(())
}

/// Write a skill file to global config
pub fn write_global_skill_file(skill: &Skill, file: &SkillFile) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_skill_subfile(home, skill, file)
}

/// Delete a skill file from global config
pub fn delete_global_skill_file(skill: &Skill, file: &SkillFile) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    delete_skill_subfile(home, skill, file)
}

/// Write a skill file to project config
pub fn write_project_skill_file(
    project_path: &Path,
    skill: &Skill,
    file: &SkillFile,
) -> Result<()> {
    write_skill_subfile(project_path, skill, file)
}

/// Delete a skill file from project config
pub fn delete_project_skill_file(
    project_path: &Path,
    skill: &Skill,
    file: &SkillFile,
) -> Result<()> {
    delete_skill_subfile(project_path, skill, file)
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

/// Write a skill based on editor type
pub fn write_skill_for_editor(base_path: &Path, skill: &Skill, editor: EditorType) -> Result<()> {
    match editor {
        EditorType::ClaudeCode => write_skill_file(base_path, skill),
        EditorType::OpenCode => {
            let opencode_dir = base_path.join(".opencode");
            write_skill_file_opencode(&opencode_dir, skill)
        }
    }
}

/// Delete a skill based on editor type
pub fn delete_skill_for_editor(base_path: &Path, skill: &Skill, editor: EditorType) -> Result<()> {
    match editor {
        EditorType::ClaudeCode => delete_skill_file(base_path, skill),
        EditorType::OpenCode => {
            let opencode_dir = base_path.join(".opencode");
            delete_skill_file_opencode(&opencode_dir, skill)
        }
    }
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
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    fn sample_skill_file() -> SkillFile {
        SkillFile {
            id: 1,
            skill_id: 1,
            file_type: "reference".to_string(),
            name: "api-docs.md".to_string(),
            content: "# API Documentation\n\nSome docs here.".to_string(),
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
    // file_type_to_dir tests
    // =========================================================================

    #[test]
    fn test_file_type_to_dir_reference() {
        assert_eq!(file_type_to_dir("reference"), "references");
    }

    #[test]
    fn test_file_type_to_dir_asset() {
        assert_eq!(file_type_to_dir("asset"), "assets");
    }

    #[test]
    fn test_file_type_to_dir_script() {
        assert_eq!(file_type_to_dir("script"), "scripts");
    }

    #[test]
    fn test_file_type_to_dir_unknown_defaults_to_assets() {
        assert_eq!(file_type_to_dir("unknown"), "assets");
        assert_eq!(file_type_to_dir(""), "assets");
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
    // write_skill_subfile tests
    // =========================================================================

    #[test]
    fn test_write_skill_subfile_creates_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let skill = sample_skill();
        let file = sample_skill_file();

        write_skill_subfile(temp_dir.path(), &skill, &file).unwrap();

        let expected_path = temp_dir
            .path()
            .join(".claude")
            .join("skills")
            .join("test-agent")
            .join("references")
            .join("api-docs.md");
        assert!(expected_path.exists());
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
}
