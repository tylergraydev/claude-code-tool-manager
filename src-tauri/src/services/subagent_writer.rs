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
fn generate_subagent_markdown(subagent: &SubAgent) -> String {
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
    let file_path = base_path.join(".claude").join("agents").join(format!("{}.md", name));
    if file_path.exists() {
        std::fs::remove_file(file_path)?;
    }
    Ok(())
}

/// Write a sub-agent to the global Claude config (~/.claude/agents/)
pub fn write_global_subagent(subagent: &SubAgent) -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_subagent_file(home, subagent)
}

/// Delete a sub-agent from the global Claude config (~/.claude/agents/)
pub fn delete_global_subagent(name: &str) -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
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
pub fn write_subagent_for_editor(base_path: &Path, subagent: &SubAgent, editor: EditorType) -> Result<()> {
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
