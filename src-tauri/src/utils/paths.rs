use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

pub struct ClaudePathsInternal {
    #[allow(dead_code)]
    pub home: PathBuf,
    pub claude_dir: PathBuf,
    pub claude_json: PathBuf,  // Main Claude Code config file (~/.claude.json)
    pub global_settings: PathBuf,
    pub plugins_dir: PathBuf,
    pub marketplaces_dir: PathBuf,
    pub commands_dir: PathBuf,  // ~/.claude/commands/ for command-type skills
    pub skills_dir: PathBuf,    // ~/.claude/skills/ for agent-type skills
    pub agents_dir: PathBuf,    // ~/.claude/agents/ for sub-agents
}

pub fn get_claude_paths() -> Result<ClaudePathsInternal> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let home = base_dirs.home_dir().to_path_buf();
    let claude_dir = home.join(".claude");

    Ok(ClaudePathsInternal {
        claude_json: home.join(".claude.json"),  // ~/.claude.json
        global_settings: claude_dir.join("settings.json"),
        plugins_dir: claude_dir.join("plugins"),
        marketplaces_dir: claude_dir.join("plugins").join("marketplaces"),
        commands_dir: claude_dir.join("commands"),
        skills_dir: claude_dir.join("skills"),
        agents_dir: claude_dir.join("agents"),
        home,
        claude_dir,
    })
}

/// Normalize a path to use forward slashes for consistent comparison
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[allow(dead_code)]
pub fn project_mcp_file(project_path: &PathBuf) -> PathBuf {
    project_path.join(".claude").join(".mcp.json")
}

#[allow(dead_code)]
pub fn project_settings_file(project_path: &PathBuf) -> PathBuf {
    project_path.join(".claude").join("settings.local.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_claude_paths_returns_valid_structure() {
        let paths = get_claude_paths().expect("Should get claude paths");

        // Verify paths are constructed correctly relative to home
        assert!(paths.claude_dir.ends_with(".claude"));
        assert!(paths.claude_json.to_string_lossy().contains(".claude.json"));
        assert!(paths.global_settings.ends_with("settings.json"));
        assert!(paths.plugins_dir.ends_with("plugins"));
        assert!(paths.commands_dir.ends_with("commands"));
        assert!(paths.skills_dir.ends_with("skills"));
        assert!(paths.agents_dir.ends_with("agents"));
    }

    #[test]
    fn test_get_claude_paths_marketplaces_nested() {
        let paths = get_claude_paths().expect("Should get claude paths");

        // Marketplaces should be nested under plugins
        let marketplaces_str = paths.marketplaces_dir.to_string_lossy();
        assert!(marketplaces_str.contains("plugins"));
        assert!(marketplaces_str.ends_with("marketplaces"));
    }

    #[test]
    fn test_normalize_path_backslashes() {
        assert_eq!(normalize_path("C:\\Users\\Test\\file.txt"), "C:/Users/Test/file.txt");
        assert_eq!(normalize_path("path\\to\\file"), "path/to/file");
    }

    #[test]
    fn test_normalize_path_forward_slashes_unchanged() {
        assert_eq!(normalize_path("path/to/file"), "path/to/file");
        assert_eq!(normalize_path("/home/user/file.txt"), "/home/user/file.txt");
    }

    #[test]
    fn test_normalize_path_mixed_slashes() {
        assert_eq!(normalize_path("path\\to/mixed\\slashes"), "path/to/mixed/slashes");
    }

    #[test]
    fn test_normalize_path_empty() {
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn test_project_mcp_file() {
        let project_path = PathBuf::from("/home/user/myproject");
        let mcp_file = project_mcp_file(&project_path);

        assert!(mcp_file.ends_with(".mcp.json"));
        assert!(mcp_file.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn test_project_settings_file() {
        let project_path = PathBuf::from("/home/user/myproject");
        let settings_file = project_settings_file(&project_path);

        assert!(settings_file.ends_with("settings.local.json"));
        assert!(settings_file.to_string_lossy().contains(".claude"));
    }

    #[test]
    fn test_claude_paths_internal_fields() {
        let paths = get_claude_paths().expect("Should get claude paths");

        // Home should be a valid directory path
        assert!(!paths.home.as_os_str().is_empty());

        // Claude dir should be under home
        assert!(paths.claude_dir.starts_with(&paths.home));
    }
}
