use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

/// GitHub Copilot CLI configuration paths
/// Copilot CLI stores its config in ~/.copilot/ on all platforms
/// Can be overridden with XDG_CONFIG_HOME environment variable
pub struct CopilotPathsInternal {
    #[allow(dead_code)]
    pub home: PathBuf,
    pub config_dir: PathBuf,      // ~/.copilot/
    pub config_file: PathBuf,     // ~/.copilot/config.json
    pub mcp_config_file: PathBuf, // ~/.copilot/mcp-config.json
    pub agents_dir: PathBuf,      // ~/.copilot/agents/
}

pub fn get_copilot_paths() -> Result<CopilotPathsInternal> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let home = base_dirs.home_dir().to_path_buf();

    // Check for XDG_CONFIG_HOME override
    let config_dir = if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("copilot")
    } else {
        // Default: ~/.copilot/
        home.join(".copilot")
    };

    Ok(CopilotPathsInternal {
        config_file: config_dir.join("config.json"),
        mcp_config_file: config_dir.join("mcp-config.json"),
        agents_dir: config_dir.join("agents"),
        home,
        config_dir,
    })
}

/// Check if GitHub Copilot CLI is installed
/// Checks for: 1) config directory exists, or 2) 'copilot' binary in PATH
pub fn is_copilot_installed() -> bool {
    // Check if config directory exists
    if let Ok(paths) = get_copilot_paths() {
        if paths.config_dir.exists() {
            return true;
        }
    }
    // Check if binary is in PATH
    std::process::Command::new("which")
        .arg("copilot")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_copilot_paths_returns_valid_structure() {
        let paths = get_copilot_paths().expect("Should get copilot paths");

        // Verify paths are constructed correctly
        assert!(paths.config_dir.to_string_lossy().contains("copilot"));
        assert!(paths.mcp_config_file.ends_with("mcp-config.json"));
        assert!(paths.config_file.ends_with("config.json"));
    }

    #[test]
    fn test_get_copilot_paths_config_dir_contains_copilot() {
        let paths = get_copilot_paths().expect("Should get copilot paths");

        // Config files should be under the copilot config dir
        assert!(paths.config_file.starts_with(&paths.config_dir));
        assert!(paths.mcp_config_file.starts_with(&paths.config_dir));
        assert!(paths.agents_dir.starts_with(&paths.config_dir));
    }

    #[test]
    fn test_is_copilot_installed_returns_bool() {
        // This test just verifies the function runs without error
        // The actual result depends on whether copilot is installed
        let _result = is_copilot_installed();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_copilot_paths_home_not_empty() {
        let paths = get_copilot_paths().expect("Should get copilot paths");

        // Home should be a valid directory path
        assert!(!paths.home.as_os_str().is_empty());
    }

    #[test]
    fn test_copilot_paths_agents_dir() {
        let paths = get_copilot_paths().expect("Should get copilot paths");

        // Agents dir should end with "agents"
        assert!(paths.agents_dir.ends_with("agents"));
    }
}
