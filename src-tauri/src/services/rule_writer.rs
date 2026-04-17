use crate::db::models::Rule;
use anyhow::Result;
use directories::BaseDirs;
use std::path::Path;

/// Generate markdown content for a rule file (.claude/rules/name.md)
pub(crate) fn generate_rule_markdown(rule: &Rule) -> String {
    let mut frontmatter = String::from("---\n");

    if let Some(ref desc) = rule.description {
        if !desc.is_empty() {
            frontmatter.push_str(&format!("description: {}\n", desc));
        }
    }

    if let Some(ref paths) = rule.paths {
        if !paths.is_empty() {
            frontmatter.push_str(&format!(
                "paths: {}\n",
                serde_json::to_string(paths).unwrap()
            ));
        }
    }

    if let Some(ref tags) = rule.tags {
        if !tags.is_empty() {
            frontmatter.push_str(&format!("tags: {}\n", serde_json::to_string(tags).unwrap()));
        }
    }

    frontmatter.push_str("---\n\n");
    format!("{}{}", frontmatter, rule.content)
}

/// Write a rule to the appropriate location
/// Rules go to {base_path}/.claude/rules/{name}.md
pub fn write_rule_file(base_path: &Path, rule: &Rule) -> Result<()> {
    let rules_dir = base_path.join(".claude").join("rules");
    std::fs::create_dir_all(&rules_dir)?;

    let file_path = rules_dir.join(format!("{}.md", rule.name));
    crate::utils::backup::backup_file(&file_path)?;
    let content = generate_rule_markdown(rule);
    std::fs::write(file_path, content)?;

    Ok(())
}

/// Delete a rule file from the appropriate location
pub fn delete_rule_file(base_path: &Path, rule: &Rule) -> Result<()> {
    let file_path = base_path
        .join(".claude")
        .join("rules")
        .join(format!("{}.md", rule.name));
    if file_path.exists() {
        std::fs::remove_file(file_path)?;
    }

    Ok(())
}

/// Write a rule to the global Claude config (~/.claude/rules/)
pub fn write_global_rule(rule: &Rule) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    write_rule_file(home, rule)
}

/// Delete a rule from the global Claude config (~/.claude/rules/)
pub fn delete_global_rule(rule: &Rule) -> Result<()> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let home = base_dirs.home_dir();
    delete_rule_file(home, rule)
}

/// Write a rule to a project's Claude config ({project}/.claude/rules/)
pub fn write_project_rule(project_path: &Path, rule: &Rule) -> Result<()> {
    write_rule_file(project_path, rule)
}

/// Delete a rule from a project's Claude config ({project}/.claude/rules/)
pub fn delete_project_rule(project_path: &Path, rule: &Rule) -> Result<()> {
    delete_rule_file(project_path, rule)
}

/// Create a symlink from one rule to another location
#[cfg(unix)]
pub fn create_rule_symlink(source_path: &Path, target_path: &Path) -> Result<()> {
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::os::unix::fs::symlink(source_path, target_path)?;
    Ok(())
}

#[cfg(not(unix))]
pub fn create_rule_symlink(source_path: &Path, target_path: &Path) -> Result<()> {
    // On non-Unix, just copy the file
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source_path, target_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_rule() -> Rule {
        Rule {
            id: 1,
            name: "typescript-strict".to_string(),
            description: Some("Enforce TypeScript strict mode".to_string()),
            content: "Always use strict TypeScript with no-any rule.".to_string(),
            paths: Some(vec!["src/**/*.ts".to_string(), "tests/**/*.ts".to_string()]),
            tags: Some(vec!["typescript".to_string(), "quality".to_string()]),
            source: "manual".to_string(),
            source_path: None,
            is_symlink: false,
            symlink_target: None,
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    fn sample_minimal_rule() -> Rule {
        Rule {
            id: 2,
            name: "general".to_string(),
            description: None,
            content: "Be concise.".to_string(),
            paths: None,
            tags: None,
            source: "manual".to_string(),
            source_path: None,
            is_symlink: false,
            symlink_target: None,
            is_favorite: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        }
    }

    #[test]
    fn test_generate_rule_markdown_emits_json_paths() {
        let rule = sample_rule();
        let md = generate_rule_markdown(&rule);

        assert!(md.starts_with("---\n"));
        assert!(md.contains("description: Enforce TypeScript strict mode\n"));
        assert!(md.contains(r#"paths: ["src/**/*.ts","tests/**/*.ts"]"#));
        assert!(md.contains("---\n\nAlways use strict TypeScript"));
    }

    #[test]
    fn test_generate_rule_markdown_emits_json_tags() {
        let rule = sample_rule();
        let md = generate_rule_markdown(&rule);

        assert!(md.contains(r#"tags: ["typescript","quality"]"#));
    }

    #[test]
    fn test_generate_rule_markdown_minimal() {
        let rule = sample_minimal_rule();
        let md = generate_rule_markdown(&rule);

        assert!(md.starts_with("---\n"));
        assert!(!md.contains("description:"));
        assert!(!md.contains("paths:"));
        assert!(!md.contains("tags:"));
        assert!(md.contains("Be concise."));
    }

    #[test]
    fn test_write_rule_file_creates_correct_path() {
        let temp_dir = TempDir::new().unwrap();
        let rule = sample_rule();

        write_rule_file(temp_dir.path(), &rule).unwrap();

        let expected_path = temp_dir
            .path()
            .join(".claude")
            .join("rules")
            .join("typescript-strict.md");
        assert!(expected_path.exists());
    }

    #[test]
    fn test_write_rule_file_content_matches() {
        let temp_dir = TempDir::new().unwrap();
        let rule = sample_rule();

        write_rule_file(temp_dir.path(), &rule).unwrap();

        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("rules")
            .join("typescript-strict.md");
        let content = std::fs::read_to_string(file_path).unwrap();

        assert!(content.contains("description: Enforce TypeScript strict mode"));
        assert!(content.contains("Always use strict TypeScript"));
    }

    #[test]
    fn test_delete_rule_file_removes_file() {
        let temp_dir = TempDir::new().unwrap();
        let rule = sample_rule();

        write_rule_file(temp_dir.path(), &rule).unwrap();
        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("rules")
            .join("typescript-strict.md");
        assert!(file_path.exists());

        delete_rule_file(temp_dir.path(), &rule).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_delete_nonexistent_rule_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let rule = sample_rule();

        let result = delete_rule_file(temp_dir.path(), &rule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_rule_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let mut rule = sample_rule();
        write_rule_file(temp_dir.path(), &rule).unwrap();

        rule.content = "Updated content".to_string();
        write_rule_file(temp_dir.path(), &rule).unwrap();

        let file_path = temp_dir
            .path()
            .join(".claude")
            .join("rules")
            .join("typescript-strict.md");
        let content = std::fs::read_to_string(file_path).unwrap();
        assert!(content.contains("Updated content"));
    }

    #[test]
    fn test_write_project_rule() {
        let temp_dir = TempDir::new().unwrap();
        let rule = sample_rule();
        write_project_rule(temp_dir.path(), &rule).unwrap();

        let expected = temp_dir
            .path()
            .join(".claude")
            .join("rules")
            .join("typescript-strict.md");
        assert!(expected.exists());
    }

    #[test]
    fn test_delete_project_rule() {
        let temp_dir = TempDir::new().unwrap();
        let rule = sample_rule();
        write_project_rule(temp_dir.path(), &rule).unwrap();
        delete_project_rule(temp_dir.path(), &rule).unwrap();

        let expected = temp_dir
            .path()
            .join(".claude")
            .join("rules")
            .join("typescript-strict.md");
        assert!(!expected.exists());
    }

    #[cfg(unix)]
    #[test]
    fn test_create_rule_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let rule = sample_rule();
        write_rule_file(temp_dir.path(), &rule).unwrap();

        let source = temp_dir
            .path()
            .join(".claude")
            .join("rules")
            .join("typescript-strict.md");
        let target = temp_dir.path().join("linked-rule.md");

        create_rule_symlink(&source, &target).unwrap();
        assert!(target.exists());
        assert!(target.symlink_metadata().unwrap().file_type().is_symlink());
    }
}
