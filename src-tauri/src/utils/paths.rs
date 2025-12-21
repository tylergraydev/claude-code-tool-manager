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
