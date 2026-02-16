use anyhow::Result;
use directories::BaseDirs;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Memory scope determines which CLAUDE.md file to read/write
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryScope {
    User,
    Project,
    Local,
}

/// Information about a single CLAUDE.md file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileInfo {
    pub scope: String,
    pub exists: bool,
    pub file_path: String,
    pub content: String,
    pub last_modified: Option<String>,
    pub size_bytes: Option<u64>,
}

/// All memory files across all three scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllMemoryFiles {
    pub user: MemoryFileInfo,
    pub project: Option<MemoryFileInfo>,
    pub local: Option<MemoryFileInfo>,
}

/// Resolve the CLAUDE.md file path for a given scope
pub fn resolve_memory_path(scope: &MemoryScope, project_path: Option<&Path>) -> Result<PathBuf> {
    match scope {
        MemoryScope::User => {
            let base_dirs =
                BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
            Ok(base_dirs.home_dir().join(".claude").join("CLAUDE.md"))
        }
        MemoryScope::Project => {
            let project = project_path
                .ok_or_else(|| anyhow::anyhow!("Project path required for project scope"))?;
            // Check .claude/CLAUDE.md first, then CLAUDE.md at root
            let dotclaude_path = project.join(".claude").join("CLAUDE.md");
            let root_path = project.join("CLAUDE.md");
            if dotclaude_path.exists() {
                Ok(dotclaude_path)
            } else if root_path.exists() {
                Ok(root_path)
            } else {
                // Default to root CLAUDE.md for new files
                Ok(root_path)
            }
        }
        MemoryScope::Local => {
            let project = project_path
                .ok_or_else(|| anyhow::anyhow!("Project path required for local scope"))?;
            Ok(project.join("CLAUDE.local.md"))
        }
    }
}

/// Detect which project memory location variant is in use
/// Returns (path, variant) where variant is "root" or ".claude"
pub fn detect_project_memory_location(project_path: &Path) -> Result<(PathBuf, String)> {
    let dotclaude_path = project_path.join(".claude").join("CLAUDE.md");
    let root_path = project_path.join("CLAUDE.md");

    if dotclaude_path.exists() {
        Ok((dotclaude_path, ".claude".to_string()))
    } else if root_path.exists() {
        Ok((root_path, "root".to_string()))
    } else {
        // Neither exists yet — default to root
        Ok((root_path, "root".to_string()))
    }
}

/// Read a single memory file and return its info
pub fn read_memory_file(scope: &MemoryScope, project_path: Option<&Path>) -> Result<MemoryFileInfo> {
    let scope_str = match scope {
        MemoryScope::User => "user",
        MemoryScope::Project => "project",
        MemoryScope::Local => "local",
    };

    let path = resolve_memory_path(scope, project_path)?;
    let path_str = path.to_string_lossy().to_string();

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        // Normalize \r\n to \n
        let content = content.replace("\r\n", "\n");
        let metadata = std::fs::metadata(&path)?;
        let last_modified = metadata
            .modified()
            .ok()
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.to_rfc3339()
            });
        let size_bytes = Some(metadata.len());

        Ok(MemoryFileInfo {
            scope: scope_str.to_string(),
            exists: true,
            file_path: path_str,
            content,
            last_modified,
            size_bytes,
        })
    } else {
        Ok(MemoryFileInfo {
            scope: scope_str.to_string(),
            exists: false,
            file_path: path_str,
            content: String::new(),
            last_modified: None,
            size_bytes: None,
        })
    }
}

/// Read all memory files across all three scopes
pub fn read_all_memory_files(project_path: Option<&Path>) -> Result<AllMemoryFiles> {
    let user = read_memory_file(&MemoryScope::User, None)?;

    let (project, local) = if let Some(pp) = project_path {
        let project_info = read_memory_file(&MemoryScope::Project, Some(pp))?;
        let local_info = read_memory_file(&MemoryScope::Local, Some(pp))?;
        (Some(project_info), Some(local_info))
    } else {
        (None, None)
    };

    Ok(AllMemoryFiles {
        user,
        project,
        local,
    })
}

/// Write content to a memory file, creating directories if needed
pub fn write_memory_file(
    scope: &MemoryScope,
    project_path: Option<&Path>,
    content: &str,
) -> Result<MemoryFileInfo> {
    let path = resolve_memory_path(scope, project_path)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&path, content)?;

    // Read back the file info to return updated metadata
    read_memory_file(scope, project_path)
}

/// Delete a memory file. No error if the file doesn't exist.
pub fn delete_memory_file(scope: &MemoryScope, project_path: Option<&Path>) -> Result<()> {
    let path = resolve_memory_path(scope, project_path)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Render markdown content to HTML using pulldown-cmark
pub fn render_markdown(content: &str) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    Ok(html_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_nonexistent_memory_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let info = read_memory_file(&MemoryScope::Project, Some(path)).unwrap();
        assert!(!info.exists);
        assert!(info.content.is_empty());
        assert_eq!(info.scope, "project");
    }

    #[test]
    fn test_write_and_read_memory_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let content = "# My Project\n\nSome instructions here.";
        let info = write_memory_file(&MemoryScope::Project, Some(path), content).unwrap();

        assert!(info.exists);
        assert_eq!(info.content, content);
        assert!(info.size_bytes.unwrap() > 0);
        assert!(info.last_modified.is_some());
    }

    #[test]
    fn test_write_creates_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        // Writing to user scope would create ~/.claude/ but we test local scope
        let content = "# Local overrides";
        let info = write_memory_file(&MemoryScope::Local, Some(path), content).unwrap();
        assert!(info.exists);
        assert_eq!(info.content, content);
    }

    #[test]
    fn test_delete_memory_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        // Create then delete
        write_memory_file(&MemoryScope::Local, Some(path), "temp content").unwrap();
        delete_memory_file(&MemoryScope::Local, Some(path)).unwrap();

        let info = read_memory_file(&MemoryScope::Local, Some(path)).unwrap();
        assert!(!info.exists);
    }

    #[test]
    fn test_delete_nonexistent_file_no_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        // Should not error
        delete_memory_file(&MemoryScope::Local, Some(path)).unwrap();
    }

    #[test]
    fn test_project_scope_prefers_dotclaude() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        // Create both files
        std::fs::write(path.join("CLAUDE.md"), "root content").unwrap();
        let dotclaude = path.join(".claude");
        std::fs::create_dir_all(&dotclaude).unwrap();
        std::fs::write(dotclaude.join("CLAUDE.md"), "dotclaude content").unwrap();

        let info = read_memory_file(&MemoryScope::Project, Some(path)).unwrap();
        assert!(info.exists);
        assert_eq!(info.content, "dotclaude content");
    }

    #[test]
    fn test_project_scope_falls_back_to_root() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        std::fs::write(path.join("CLAUDE.md"), "root content").unwrap();

        let info = read_memory_file(&MemoryScope::Project, Some(path)).unwrap();
        assert!(info.exists);
        assert_eq!(info.content, "root content");
    }

    #[test]
    fn test_detect_project_memory_location() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        // Neither exists
        let (_, variant) = detect_project_memory_location(path).unwrap();
        assert_eq!(variant, "root");

        // Create root file
        std::fs::write(path.join("CLAUDE.md"), "root").unwrap();
        let (_, variant) = detect_project_memory_location(path).unwrap();
        assert_eq!(variant, "root");

        // Create .claude file (should take priority)
        let dotclaude = path.join(".claude");
        std::fs::create_dir_all(&dotclaude).unwrap();
        std::fs::write(dotclaude.join("CLAUDE.md"), "dotclaude").unwrap();
        let (_, variant) = detect_project_memory_location(path).unwrap();
        assert_eq!(variant, ".claude");
    }

    #[test]
    fn test_render_markdown() {
        let content = "# Hello\n\n- item 1\n- item 2\n\n**bold** text";
        let html = render_markdown(content).unwrap();
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<li>item 1</li>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_normalize_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        // Write content with \r\n
        let file_path = path.join("CLAUDE.md");
        std::fs::write(&file_path, "line1\r\nline2\r\nline3").unwrap();

        let info = read_memory_file(&MemoryScope::Project, Some(path)).unwrap();
        assert_eq!(info.content, "line1\nline2\nline3");
    }

    #[test]
    fn test_empty_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        std::fs::write(path.join("CLAUDE.md"), "").unwrap();

        let info = read_memory_file(&MemoryScope::Project, Some(path)).unwrap();
        assert!(info.exists);
        assert!(info.content.is_empty());
        assert_eq!(info.size_bytes, Some(0));
    }

    #[test]
    fn test_read_all_memory_files_no_project() {
        // Without project path, only user is returned, project/local are None
        let all = read_all_memory_files(None);
        assert!(all.is_ok());
        let all = all.unwrap();
        assert!(all.project.is_none());
        assert!(all.local.is_none());
    }

    #[test]
    fn test_read_all_memory_files_with_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let all = read_all_memory_files(Some(path)).unwrap();
        assert!(all.project.is_some());
        assert!(all.local.is_some());
        // Project and local don't exist yet
        assert!(!all.project.unwrap().exists);
        assert!(!all.local.unwrap().exists);
    }
}
