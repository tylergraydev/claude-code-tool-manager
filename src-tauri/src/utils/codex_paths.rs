use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

/// Codex CLI configuration paths
/// Codex stores its config in ~/.codex/ on all platforms
pub struct CodexPathsInternal {
    #[allow(dead_code)]
    pub home: PathBuf,
    pub config_dir: PathBuf,  // ~/.codex/
    pub config_file: PathBuf, // ~/.codex/config.toml
}

pub fn get_codex_paths() -> Result<CodexPathsInternal> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let home = base_dirs.home_dir().to_path_buf();

    // Codex uses ~/.codex/ on ALL platforms
    let config_dir = home.join(".codex");

    Ok(CodexPathsInternal {
        config_file: config_dir.join("config.toml"),
        home,
        config_dir,
    })
}

/// Check if Codex CLI is installed
/// Checks for: 1) config directory exists, or 2) 'codex' binary in PATH
pub fn is_codex_installed() -> bool {
    // Check if config directory exists
    if let Ok(paths) = get_codex_paths() {
        if paths.config_dir.exists() {
            return true;
        }
    }
    // Check if binary is in PATH
    std::process::Command::new("which")
        .arg("codex")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_codex_paths_returns_valid_structure() {
        let paths = get_codex_paths().expect("Should get codex paths");

        // Verify paths are constructed correctly
        assert!(paths.config_dir.to_string_lossy().contains(".codex"));
        assert!(paths.config_file.ends_with("config.toml"));
    }

    #[test]
    fn test_get_codex_paths_config_dir_contains_codex() {
        let paths = get_codex_paths().expect("Should get codex paths");

        // Config file should be under the codex config dir
        assert!(paths.config_file.starts_with(&paths.config_dir));
    }

    #[test]
    fn test_is_codex_installed_returns_bool() {
        // This test just verifies the function runs without error
        // The actual result depends on whether codex is installed
        let _result = is_codex_installed();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_codex_paths_home_not_empty() {
        let paths = get_codex_paths().expect("Should get codex paths");

        // Home should be a valid directory path
        assert!(!paths.home.as_os_str().is_empty());
    }
}
