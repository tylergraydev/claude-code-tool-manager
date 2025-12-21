use crate::db::models::{Skill, SkillFile};
use anyhow::Result;
use directories::BaseDirs;
use std::path::Path;

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
