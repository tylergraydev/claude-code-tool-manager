use crate::db::models::Skill;
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
