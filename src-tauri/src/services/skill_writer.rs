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

/// Generate markdown content for a slash command (.claude/commands/name.md)
fn generate_command_markdown(skill: &Skill) -> String {
    let mut frontmatter = String::from("---\n");

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

    if let Some(ref hint) = skill.argument_hint {
        if !hint.is_empty() {
            frontmatter.push_str(&format!("argument-hint: {}\n", hint));
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

/// Generate markdown content for an agent skill (.claude/skills/name/SKILL.md)
fn generate_skill_markdown(skill: &Skill) -> String {
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

/// Write a skill to the appropriate location based on its type
/// - Commands go to {base_path}/commands/{name}.md
/// - Skills go to {base_path}/skills/{name}/SKILL.md
pub fn write_skill_file(base_path: &Path, skill: &Skill) -> Result<()> {
    let claude_dir = base_path.join(".claude");

    match skill.skill_type.as_str() {
        "command" => {
            let commands_dir = claude_dir.join("commands");
            std::fs::create_dir_all(&commands_dir)?;

            let file_path = commands_dir.join(format!("{}.md", skill.name));
            let content = generate_command_markdown(skill);
            std::fs::write(file_path, content)?;
        }
        "skill" => {
            let skill_dir = claude_dir.join("skills").join(&skill.name);
            std::fs::create_dir_all(&skill_dir)?;

            let file_path = skill_dir.join("SKILL.md");
            let content = generate_skill_markdown(skill);
            std::fs::write(file_path, content)?;
        }
        _ => {
            // Default to command type
            let commands_dir = claude_dir.join("commands");
            std::fs::create_dir_all(&commands_dir)?;

            let file_path = commands_dir.join(format!("{}.md", skill.name));
            let content = generate_command_markdown(skill);
            std::fs::write(file_path, content)?;
        }
    }

    Ok(())
}

/// Delete a skill file from the appropriate location
pub fn delete_skill_file(base_path: &Path, skill: &Skill) -> Result<()> {
    let claude_dir = base_path.join(".claude");

    match skill.skill_type.as_str() {
        "command" => {
            let file_path = claude_dir.join("commands").join(format!("{}.md", skill.name));
            if file_path.exists() {
                std::fs::remove_file(file_path)?;
            }
        }
        "skill" => {
            let skill_dir = claude_dir.join("skills").join(&skill.name);
            if skill_dir.exists() {
                std::fs::remove_dir_all(skill_dir)?;
            }
        }
        _ => {
            // Try both locations
            let cmd_path = claude_dir.join("commands").join(format!("{}.md", skill.name));
            if cmd_path.exists() {
                std::fs::remove_file(cmd_path)?;
            }
            let skill_dir = claude_dir.join("skills").join(&skill.name);
            if skill_dir.exists() {
                std::fs::remove_dir_all(skill_dir)?;
            }
        }
    }

    Ok(())
}

/// Write a skill to the global Claude config (~/.claude/)
pub fn write_global_skill(skill: &Skill) -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_skill_file(home, skill)
}

/// Delete a skill from the global Claude config (~/.claude/)
pub fn delete_global_skill(skill: &Skill) -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
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
fn file_type_to_dir(file_type: &str) -> &str {
    match file_type {
        "reference" => "references",
        "asset" => "assets",
        "script" => "scripts",
        _ => "assets", // Default fallback
    }
}

/// Write a skill file to the skill directory
/// Only works for agent skills (type "skill"), not commands
pub fn write_skill_subfile(base_path: &Path, skill: &Skill, file: &SkillFile) -> Result<()> {
    if skill.skill_type != "skill" {
        return Err(anyhow::anyhow!("Skill files are only supported for agent skills, not commands"));
    }

    let skill_dir = base_path.join(".claude").join("skills").join(&skill.name);
    let type_dir = skill_dir.join(file_type_to_dir(&file.file_type));
    std::fs::create_dir_all(&type_dir)?;

    let file_path = type_dir.join(&file.name);
    std::fs::write(file_path, &file.content)?;

    Ok(())
}

/// Delete a skill file from the skill directory
pub fn delete_skill_subfile(base_path: &Path, skill: &Skill, file: &SkillFile) -> Result<()> {
    if skill.skill_type != "skill" {
        return Ok(()); // Nothing to delete for commands
    }

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
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_skill_subfile(home, skill, file)
}

/// Delete a skill file from global config
pub fn delete_global_skill_file(skill: &Skill, file: &SkillFile) -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    delete_skill_subfile(home, skill, file)
}

/// Write a skill file to project config
pub fn write_project_skill_file(project_path: &Path, skill: &Skill, file: &SkillFile) -> Result<()> {
    write_skill_subfile(project_path, skill, file)
}

/// Delete a skill file from project config
pub fn delete_project_skill_file(project_path: &Path, skill: &Skill, file: &SkillFile) -> Result<()> {
    delete_skill_subfile(project_path, skill, file)
}

// ============================================================================
// OpenCode Support
// ============================================================================

/// Write a skill to OpenCode's format
/// - Commands go to {base_path}/command/{name}.md (note: singular "command")
/// - Agent skills go to {base_path}/agent/{name}.md (OpenCode uses agent/ not skills/)
pub fn write_skill_file_opencode(base_path: &Path, skill: &Skill) -> Result<()> {
    match skill.skill_type.as_str() {
        "command" => {
            let commands_dir = base_path.join("command"); // OpenCode uses singular
            std::fs::create_dir_all(&commands_dir)?;

            let file_path = commands_dir.join(format!("{}.md", skill.name));
            let content = generate_command_markdown(skill);
            std::fs::write(file_path, content)?;
        }
        "skill" => {
            // OpenCode doesn't have the same skills directory structure
            // It uses agent/ for agent definitions
            let agent_dir = base_path.join("agent");
            std::fs::create_dir_all(&agent_dir)?;

            let file_path = agent_dir.join(format!("{}.md", skill.name));
            let content = generate_skill_markdown(skill);
            std::fs::write(file_path, content)?;
        }
        _ => {
            let commands_dir = base_path.join("command");
            std::fs::create_dir_all(&commands_dir)?;

            let file_path = commands_dir.join(format!("{}.md", skill.name));
            let content = generate_command_markdown(skill);
            std::fs::write(file_path, content)?;
        }
    }

    Ok(())
}

/// Delete a skill from OpenCode's format
pub fn delete_skill_file_opencode(base_path: &Path, skill: &Skill) -> Result<()> {
    match skill.skill_type.as_str() {
        "command" => {
            let file_path = base_path.join("command").join(format!("{}.md", skill.name));
            if file_path.exists() {
                std::fs::remove_file(file_path)?;
            }
        }
        "skill" => {
            let file_path = base_path.join("agent").join(format!("{}.md", skill.name));
            if file_path.exists() {
                std::fs::remove_file(file_path)?;
            }
        }
        _ => {
            // Try both locations
            let cmd_path = base_path.join("command").join(format!("{}.md", skill.name));
            if cmd_path.exists() {
                std::fs::remove_file(cmd_path)?;
            }
            let agent_path = base_path.join("agent").join(format!("{}.md", skill.name));
            if agent_path.exists() {
                std::fs::remove_file(agent_path)?;
            }
        }
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
