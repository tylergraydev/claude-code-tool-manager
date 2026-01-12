use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

/// Gemini CLI configuration paths
/// Gemini CLI stores its config in ~/.gemini/ on all platforms
pub struct GeminiPathsInternal {
    #[allow(dead_code)]
    pub home: PathBuf,
    pub config_dir: PathBuf,    // ~/.gemini/
    pub settings_file: PathBuf, // ~/.gemini/settings.json
}

pub fn get_gemini_paths() -> Result<GeminiPathsInternal> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let home = base_dirs.home_dir().to_path_buf();

    // Gemini CLI uses ~/.gemini/ for config
    let config_dir = home.join(".gemini");

    Ok(GeminiPathsInternal {
        settings_file: config_dir.join("settings.json"),
        home,
        config_dir,
    })
}

/// Check if Gemini CLI is installed
/// Checks for: 1) config directory exists, or 2) 'gemini' binary in PATH
pub fn is_gemini_installed() -> bool {
    // Check if config directory exists
    if let Ok(paths) = get_gemini_paths() {
        if paths.config_dir.exists() {
            return true;
        }
    }
    // Check if binary is in PATH
    std::process::Command::new("which")
        .arg("gemini")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gemini_paths_returns_valid_structure() {
        let paths = get_gemini_paths().expect("Should get gemini paths");

        // Verify paths are constructed correctly
        assert!(paths.config_dir.to_string_lossy().contains("gemini"));
        assert!(paths.settings_file.ends_with("settings.json"));
    }

    #[test]
    fn test_get_gemini_paths_config_dir_contains_gemini() {
        let paths = get_gemini_paths().expect("Should get gemini paths");

        // Settings file should be under the gemini config dir
        assert!(paths.settings_file.starts_with(&paths.config_dir));
    }

    #[test]
    fn test_is_gemini_installed_returns_bool() {
        // This test just verifies the function runs without error
        // The actual result depends on whether Gemini CLI is installed
        let _result = is_gemini_installed();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_gemini_paths_home_not_empty() {
        let paths = get_gemini_paths().expect("Should get gemini paths");

        // Home should be a valid directory path
        assert!(!paths.home.as_os_str().is_empty());
    }

    #[test]
    fn test_gemini_config_dir_ends_with_gemini() {
        let paths = get_gemini_paths().expect("Should get gemini paths");

        // Config dir should end with ".gemini"
        assert!(paths.config_dir.ends_with(".gemini"));
    }
}
