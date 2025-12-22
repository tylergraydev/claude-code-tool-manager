use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

/// OpenCode configuration paths
/// OpenCode stores its config in ~/.config/opencode/ on all platforms
pub struct OpenCodePathsInternal {
    #[allow(dead_code)]
    pub home: PathBuf,
    pub config_dir: PathBuf,           // ~/.config/opencode/
    pub config_file: PathBuf,          // ~/.config/opencode/opencode.json
    pub command_dir: PathBuf,          // ~/.config/opencode/command/ (slash commands)
    pub agent_dir: PathBuf,            // ~/.config/opencode/agent/ (sub-agents)
    pub plugin_dir: PathBuf,           // ~/.config/opencode/plugin/ (hooks)
    pub tool_dir: PathBuf,             // ~/.config/opencode/tool/ (custom tools)
    pub knowledge_dir: PathBuf,        // ~/.config/opencode/knowledge/ (context files)
}

pub fn get_opencode_paths() -> Result<OpenCodePathsInternal> {
    let base_dirs = BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

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
