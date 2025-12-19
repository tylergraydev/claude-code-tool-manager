use anyhow::Result;
use directories::BaseDirs;
use std::path::PathBuf;

pub struct ClaudePathsInternal {
    pub home: PathBuf,
    pub claude_dir: PathBuf,
    pub global_settings: PathBuf,
    pub plugins_dir: PathBuf,
    pub marketplaces_dir: PathBuf,
}

pub fn get_claude_paths() -> Result<ClaudePathsInternal> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let home = base_dirs.home_dir().to_path_buf();
    let claude_dir = home.join(".claude");

    Ok(ClaudePathsInternal {
        global_settings: claude_dir.join("settings.json"),
        plugins_dir: claude_dir.join("plugins"),
        marketplaces_dir: claude_dir.join("plugins").join("marketplaces"),
        home,
        claude_dir,
    })
}

pub fn project_mcp_file(project_path: &PathBuf) -> PathBuf {
    project_path.join(".claude").join(".mcp.json")
}

pub fn project_settings_file(project_path: &PathBuf) -> PathBuf {
    project_path.join(".claude").join("settings.local.json")
}
