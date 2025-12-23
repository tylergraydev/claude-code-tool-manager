use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

/// OpenCode configuration paths
/// OpenCode stores its config in ~/.config/opencode/ on all platforms
pub struct OpenCodePathsInternal {
    #[allow(dead_code)]
    pub home: PathBuf,
    pub config_dir: PathBuf,    // ~/.config/opencode/
    pub config_file: PathBuf,   // ~/.config/opencode/opencode.json
    pub command_dir: PathBuf,   // ~/.config/opencode/command/ (slash commands)
    pub agent_dir: PathBuf,     // ~/.config/opencode/agent/ (sub-agents)
    pub plugin_dir: PathBuf,    // ~/.config/opencode/plugin/ (hooks)
    pub tool_dir: PathBuf,      // ~/.config/opencode/tool/ (custom tools)
    pub knowledge_dir: PathBuf, // ~/.config/opencode/knowledge/ (context files)
}

pub fn get_opencode_paths() -> Result<OpenCodePathsInternal> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let home = base_dirs.home_dir().to_path_buf();

    // OpenCode uses ~/.config/opencode/ on all platforms
    let config_dir = if cfg!(windows) {
        // On Windows, BaseDirs::config_dir() returns AppData\Roaming
        // but OpenCode uses .config in home directory
        home.join(".config").join("opencode")
    } else {
        // On Unix-like systems, use XDG config dir or ~/.config
        base_dirs.config_dir().join("opencode")
    };

    Ok(OpenCodePathsInternal {
        config_file: config_dir.join("opencode.json"),
        command_dir: config_dir.join("command"),
        agent_dir: config_dir.join("agent"),
        plugin_dir: config_dir.join("plugin"),
        tool_dir: config_dir.join("tool"),
        knowledge_dir: config_dir.join("knowledge"),
        home,
        config_dir,
    })
}

/// Check if OpenCode is installed (has config directory)
pub fn is_opencode_installed() -> bool {
    if let Ok(paths) = get_opencode_paths() {
        paths.config_dir.exists()
    } else {
        false
    }
}

/// Get project-level OpenCode directory
pub fn project_opencode_dir(project_path: &PathBuf) -> PathBuf {
    project_path.join(".opencode")
}

/// Get project-level OpenCode config file
pub fn project_opencode_config(project_path: &PathBuf) -> PathBuf {
    project_path.join("opencode.json")
}

/// Get project-level OpenCode command directory
pub fn project_opencode_command_dir(project_path: &PathBuf) -> PathBuf {
    project_path.join(".opencode").join("command")
}

/// Get project-level OpenCode agent directory
pub fn project_opencode_agent_dir(project_path: &PathBuf) -> PathBuf {
    project_path.join(".opencode").join("agent")
}

/// Get project-level OpenCode plugin directory
pub fn project_opencode_plugin_dir(project_path: &PathBuf) -> PathBuf {
    project_path.join(".opencode").join("plugin")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_opencode_paths_returns_valid_structure() {
        let paths = get_opencode_paths().expect("Should get opencode paths");

        // Verify paths are constructed correctly
        assert!(paths.config_dir.to_string_lossy().contains("opencode"));
        assert!(paths.config_file.ends_with("opencode.json"));
        assert!(paths.command_dir.ends_with("command"));
        assert!(paths.agent_dir.ends_with("agent"));
        assert!(paths.plugin_dir.ends_with("plugin"));
        assert!(paths.tool_dir.ends_with("tool"));
        assert!(paths.knowledge_dir.ends_with("knowledge"));
    }

    #[test]
    fn test_get_opencode_paths_config_dir_contains_opencode() {
        let paths = get_opencode_paths().expect("Should get opencode paths");

        // All paths should be under the opencode config dir
        assert!(paths.config_file.starts_with(&paths.config_dir));
        assert!(paths.command_dir.starts_with(&paths.config_dir));
        assert!(paths.agent_dir.starts_with(&paths.config_dir));
        assert!(paths.plugin_dir.starts_with(&paths.config_dir));
        assert!(paths.tool_dir.starts_with(&paths.config_dir));
        assert!(paths.knowledge_dir.starts_with(&paths.config_dir));
    }

    #[test]
    fn test_is_opencode_installed_returns_bool() {
        // This test just verifies the function runs without error
        // The actual result depends on whether opencode is installed
        let _result = is_opencode_installed();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_project_opencode_dir() {
        let project_path = PathBuf::from("/home/user/myproject");
        let opencode_dir = project_opencode_dir(&project_path);

        assert!(opencode_dir.ends_with(".opencode"));
        assert!(opencode_dir.starts_with(&project_path));
    }

    #[test]
    fn test_project_opencode_config() {
        let project_path = PathBuf::from("/home/user/myproject");
        let config_file = project_opencode_config(&project_path);

        assert!(config_file.ends_with("opencode.json"));
        assert!(config_file.starts_with(&project_path));
    }

    #[test]
    fn test_project_opencode_command_dir() {
        let project_path = PathBuf::from("/home/user/myproject");
        let command_dir = project_opencode_command_dir(&project_path);

        assert!(command_dir.ends_with("command"));
        assert!(command_dir.to_string_lossy().contains(".opencode"));
    }

    #[test]
    fn test_project_opencode_agent_dir() {
        let project_path = PathBuf::from("/home/user/myproject");
        let agent_dir = project_opencode_agent_dir(&project_path);

        assert!(agent_dir.ends_with("agent"));
        assert!(agent_dir.to_string_lossy().contains(".opencode"));
    }

    #[test]
    fn test_project_opencode_plugin_dir() {
        let project_path = PathBuf::from("/home/user/myproject");
        let plugin_dir = project_opencode_plugin_dir(&project_path);

        assert!(plugin_dir.ends_with("plugin"));
        assert!(plugin_dir.to_string_lossy().contains(".opencode"));
    }

    #[test]
    fn test_opencode_paths_home_not_empty() {
        let paths = get_opencode_paths().expect("Should get opencode paths");

        // Home should be a valid directory path
        assert!(!paths.home.as_os_str().is_empty());
    }

    #[test]
    fn test_project_paths_with_windows_style_path() {
        let project_path = PathBuf::from("C:\\Users\\Test\\project");

        let opencode_dir = project_opencode_dir(&project_path);
        let config = project_opencode_config(&project_path);
        let command_dir = project_opencode_command_dir(&project_path);
        let agent_dir = project_opencode_agent_dir(&project_path);
        let plugin_dir = project_opencode_plugin_dir(&project_path);

        // All should start with the project path
        assert!(opencode_dir.starts_with(&project_path));
        assert!(config.starts_with(&project_path));
        assert!(command_dir.starts_with(&project_path));
        assert!(agent_dir.starts_with(&project_path));
        assert!(plugin_dir.starts_with(&project_path));
    }
}
