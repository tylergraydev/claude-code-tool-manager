use crate::db::models::Command;
use crate::utils::opencode_paths::get_opencode_paths;
use anyhow::Result;
use directories::BaseDirs;
use std::path::Path;

// ============================================================================
// Claude Code Command Writer
// ============================================================================
// Commands are stored as single .md files in .claude/commands/{name}.md
// Unlike skills (which have directories with SKILL.md + references/assets/scripts),
// commands are simple single-file structures.

/// Generate markdown content for a slash command (.claude/commands/name.md)
/// Commands use frontmatter similar to skills but without name field
/// (the filename IS the command name)
pub(crate) fn generate_command_markdown(command: &Command) -> String {
    let mut frontmatter = String::from("---\n");

    if let Some(ref desc) = command.description {
        if !desc.is_empty() {
            frontmatter.push_str(&format!("description: {}\n", desc));
        }
    }

    if let Some(ref tools) = command.allowed_tools {
        if !tools.is_empty() {
            frontmatter.push_str(&format!("allowed-tools: {}\n", tools.join(", ")));
        }
    }

    if let Some(ref hint) = command.argument_hint {
        if !hint.is_empty() {
            frontmatter.push_str(&format!("argument-hint: {}\n", hint));
        }
    }

    if let Some(ref model) = command.model {
        if !model.is_empty() {
            frontmatter.push_str(&format!("model: {}\n", model));
        }
    }

    frontmatter.push_str("---\n\n");
    format!("{}{}", frontmatter, command.content)
}

/// Write a command to the appropriate location
/// Commands go to {base_path}/.claude/commands/{name}.md
pub fn write_command_file(base_path: &Path, command: &Command) -> Result<()> {
    let commands_dir = base_path.join(".claude").join("commands");
    std::fs::create_dir_all(&commands_dir)?;

    let file_path = commands_dir.join(format!("{}.md", command.name));
    let content = generate_command_markdown(command);
    std::fs::write(file_path, content)?;

    Ok(())
}

/// Delete a command file from the appropriate location
pub fn delete_command_file(base_path: &Path, command: &Command) -> Result<()> {
    let file_path = base_path
        .join(".claude")
        .join("commands")
        .join(format!("{}.md", command.name));

    if file_path.exists() {
        std::fs::remove_file(&file_path)?;
    }

    // Clean up empty commands directory
    let commands_dir = base_path.join(".claude").join("commands");
    if commands_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&commands_dir) {
            if entries.count() == 0 {
                let _ = std::fs::remove_dir(&commands_dir);
            }
        }
    }

    Ok(())
}

/// Write a command to the global Claude config (~/.claude/)
pub fn write_global_command(command: &Command) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_command_file(home, command)
}

/// Delete a command from the global Claude config (~/.claude/)
pub fn delete_global_command(command: &Command) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    delete_command_file(home, command)
}

/// Write a command to a project's Claude config ({project}/.claude/)
pub fn write_project_command(project_path: &Path, command: &Command) -> Result<()> {
    write_command_file(project_path, command)
}

/// Delete a command from a project's Claude config ({project}/.claude/)
pub fn delete_project_command(project_path: &Path, command: &Command) -> Result<()> {
    delete_command_file(project_path, command)
}

// ============================================================================
// OpenCode Support
// ============================================================================
// OpenCode uses a slightly different structure: .opencode/command/{name}.md

/// Write a command to OpenCode's format
/// Commands go to {base_path}/command/{name}.md
pub fn write_command_file_opencode(base_path: &Path, command: &Command) -> Result<()> {
    let command_dir = base_path.join("command");
    std::fs::create_dir_all(&command_dir)?;

    let file_path = command_dir.join(format!("{}.md", command.name));
    let content = generate_command_markdown(command);
    std::fs::write(file_path, content)?;

    Ok(())
}

/// Delete a command from OpenCode's format
pub fn delete_command_file_opencode(base_path: &Path, command: &Command) -> Result<()> {
    let file_path = base_path
        .join("command")
        .join(format!("{}.md", command.name));

    if file_path.exists() {
        std::fs::remove_file(&file_path)?;
    }

    Ok(())
}

/// Write a command to the global OpenCode config (~/.config/opencode/)
pub fn write_global_command_opencode(command: &Command) -> Result<()> {
    let paths = get_opencode_paths()?;
    write_command_file_opencode(&paths.config_dir, command)
}

/// Delete a command from the global OpenCode config
pub fn delete_global_command_opencode(command: &Command) -> Result<()> {
    let paths = get_opencode_paths()?;
    delete_command_file_opencode(&paths.config_dir, command)
}

/// Write a command to a project's OpenCode config ({project}/.opencode/)
pub fn write_project_command_opencode(project_path: &Path, command: &Command) -> Result<()> {
    let opencode_dir = project_path.join(".opencode");
    write_command_file_opencode(&opencode_dir, command)
}

/// Delete a command from a project's OpenCode config
pub fn delete_project_command_opencode(project_path: &Path, command: &Command) -> Result<()> {
    let opencode_dir = project_path.join(".opencode");
    delete_command_file_opencode(&opencode_dir, command)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_command() -> Command {
        Command {
            id: 1,
            name: "test-command".to_string(),
            description: Some("A test slash command".to_string()),
            content: "Execute this task for the user.".to_string(),
            allowed_tools: Some(vec!["Bash".to_string(), "Read".to_string()]),
            argument_hint: Some("<file_path>".to_string()),
            model: Some("sonnet".to_string()),
            tags: None,
            source: "manual".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    fn sample_minimal_command() -> Command {
        Command {
            id: 2,
            name: "minimal".to_string(),
            description: None,
            content: "Minimal content.".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            tags: None,
            source: "manual".to_string(),
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    // =========================================================================
    // generate_command_markdown tests
    // =========================================================================

    #[test]
    fn test_generate_command_markdown_full() {
        let command = sample_command();
        let md = generate_command_markdown(&command);

        assert!(md.starts_with("---\n"));
        assert!(md.contains("description: A test slash command\n"));
        assert!(md.contains("allowed-tools: Bash, Read\n"));
        assert!(md.contains("argument-hint: <file_path>\n"));
        assert!(md.contains("model: sonnet\n"));
        assert!(md.contains("---\n\nExecute this task for the user."));
        // Commands should NOT include name in frontmatter (filename is the name)
        assert!(!md.contains("name:"));
    }

    #[test]
    fn test_generate_command_markdown_minimal() {
        let command = sample_minimal_command();
        let md = generate_command_markdown(&command);

        assert!(!md.contains("description:"));
        assert!(!md.contains("allowed-tools:"));
        assert!(!md.contains("argument-hint:"));
        assert!(!md.contains("model:"));
        assert!(md.contains("Minimal content."));
    }

    // =========================================================================
    // write_command_file tests
    // =========================================================================

    #[test]
    fn test_write_command_file_creates_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let command = sample_command();

        write_command_file(temp_dir.path(), &command).unwrap();

        let expected_path = temp_dir
            .path()
            .join(".claude")
            .join("commands")
            .join("test-command.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_write_command_file_content_matches() {
        let temp_dir = TempDir::new().unwrap();
        let command = sample_command();

        write_command_file(temp_dir.path(), &command).unwrap();

        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("commands")
            .join("test-command.md");
        let content = std::fs::read_to_string(file_path).unwrap();

        assert!(content.contains("description: A test slash command"));
        assert!(content.contains("Execute this task for the user."));
    }

    // =========================================================================
    // delete_command_file tests
    // =========================================================================

    #[test]
    fn test_delete_command_file_removes_file() {
        let temp_dir = TempDir::new().unwrap();
        let command = sample_command();

        // Write first
        write_command_file(temp_dir.path(), &command).unwrap();
        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("commands")
            .join("test-command.md");
        assert!(file_path.exists());

        // Delete
        delete_command_file(temp_dir.path(), &command).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_command_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let command = sample_command();

        // Should not error when file doesn't exist
        let result = delete_command_file(temp_dir.path(), &command);
        assert!(result.is_ok());
    }

    // =========================================================================
    // OpenCode format tests
    // =========================================================================

    #[test]
    fn test_write_command_file_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let command = sample_command();

        write_command_file_opencode(temp_dir.path(), &command).unwrap();

        // OpenCode uses "command" not "commands"
        let expected_path = temp_dir.path().join("command").join("test-command.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_delete_command_file_opencode() {
        let temp_dir = TempDir::new().unwrap();
        let command = sample_command();

        write_command_file_opencode(temp_dir.path(), &command).unwrap();
        let file_path = temp_dir.path().join("command").join("test-command.md");
        assert!(file_path.exists());

        delete_command_file_opencode(temp_dir.path(), &command).unwrap();
        assert!(!file_path.exists());
    }
}
