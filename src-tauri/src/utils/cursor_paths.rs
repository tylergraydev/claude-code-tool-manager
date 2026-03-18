use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

/// Cursor IDE configuration paths
/// Cursor stores its config in ~/.cursor/ on all platforms
pub struct CursorPathsInternal {
    #[allow(dead_code)]
    pub home: PathBuf,
    pub config_dir: PathBuf,      // ~/.cursor/
    pub mcp_config_file: PathBuf, // ~/.cursor/mcp.json
}

pub fn get_cursor_paths() -> Result<CursorPathsInternal> {
    let base_dirs =
        BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let home = base_dirs.home_dir().to_path_buf();

    // Cursor uses ~/.cursor/ for config
    let config_dir = home.join(".cursor");

    Ok(CursorPathsInternal {
        mcp_config_file: config_dir.join("mcp.json"),
        home,
        config_dir,
    })
}

/// Check if Cursor IDE is installed
/// Checks for: 1) config directory exists, 2) 'cursor' binary in PATH, or 3) Cursor.app exists
pub fn is_cursor_installed() -> bool {
    // Check if config directory exists
    if let Ok(paths) = get_cursor_paths() {
        if paths.config_dir.exists() {
            return true;
        }
    }
    // Check if binary is in PATH
    if std::process::Command::new("which")
        .arg("cursor")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }
    // Check for macOS app bundle
    #[cfg(target_os = "macos")]
    {
        if std::path::Path::new("/Applications/Cursor.app").exists() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cursor_paths_returns_valid_structure() {
        let paths = get_cursor_paths().expect("Should get cursor paths");

        // Verify paths are constructed correctly
        assert!(paths.config_dir.to_string_lossy().contains("cursor"));
        assert!(paths.mcp_config_file.ends_with("mcp.json"));
    }

    #[test]
    fn test_get_cursor_paths_config_dir_contains_cursor() {
        let paths = get_cursor_paths().expect("Should get cursor paths");

        // MCP config file should be under the cursor config dir
        assert!(paths.mcp_config_file.starts_with(&paths.config_dir));
    }

    #[test]
    fn test_is_cursor_installed_returns_bool() {
        // This test just verifies the function runs without error
        // The actual result depends on whether Cursor is installed
        let _result = is_cursor_installed();
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_cursor_paths_home_not_empty() {
        let paths = get_cursor_paths().expect("Should get cursor paths");

        // Home should be a valid directory path
        assert!(!paths.home.as_os_str().is_empty());
    }

    #[test]
    fn test_cursor_config_dir_ends_with_cursor() {
        let paths = get_cursor_paths().expect("Should get cursor paths");

        // Config dir should end with ".cursor"
        assert!(paths.config_dir.ends_with(".cursor"));
    }

    #[test]
    fn test_cursor_mcp_config_is_json_file() {
        let paths = get_cursor_paths().expect("Should get cursor paths");
        let file_name = paths.mcp_config_file.file_name().unwrap().to_str().unwrap();
        assert!(file_name.ends_with(".json"));
    }

    #[test]
    fn test_cursor_config_dir_is_under_home() {
        let paths = get_cursor_paths().expect("Should get cursor paths");
        assert!(paths.config_dir.starts_with(&paths.home));
    }

    #[test]
    fn test_cursor_mcp_config_under_config_dir() {
        let paths = get_cursor_paths().expect("Should get cursor paths");
        assert!(paths.mcp_config_file.starts_with(&paths.config_dir));
    }
}
