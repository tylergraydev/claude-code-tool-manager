use crate::db::Database;
use crate::services::claude_json;
use crate::services::codex_config;
use crate::services::config_parser;
use crate::services::copilot_config;
use crate::services::cursor_config;
use crate::services::gemini_config;
use crate::services::opencode_config;
use crate::utils::codex_paths::get_codex_paths;
use crate::utils::copilot_paths::get_copilot_paths;
use crate::utils::cursor_paths::get_cursor_paths;
use crate::utils::gemini_paths::get_gemini_paths;
use crate::utils::opencode_paths::get_opencode_paths;
use crate::utils::paths::{get_claude_paths, normalize_path};
use anyhow::Result;
use rusqlite::params;
use std::collections::HashSet;
use std::path::Path;
use tauri::Manager;
use walkdir::WalkDir;

pub async fn run_startup_scan(app: &tauri::AppHandle) -> Result<()> {
    let db = app.state::<std::sync::Arc<std::sync::Mutex<Database>>>();
    let db = db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

    // First scan global MCPs from claude.json
    let global_mcp_count = scan_global_mcps_from_claude_json(&db)?;
    log::info!("Imported {} global MCPs from claude.json", global_mcp_count);

    // Then scan claude.json for projects and their MCPs
    let claude_json_count = scan_claude_json(&db)?;
    log::info!(
        "Imported {} project MCPs from claude.json",
        claude_json_count
    );

    // Then scan plugins/marketplaces for additional MCPs
    let plugin_count = scan_plugins(&db)?;
    log::info!("Found {} MCPs from plugins", plugin_count);

    // Scan global commands from ~/.claude/commands/
    let command_count = scan_global_commands(&db)?;
    log::info!("Found {} commands from ~/.claude/commands/", command_count);

    // Scan global skills from ~/.claude/skills/
    let skill_count = scan_global_skills(&db)?;
    log::info!("Found {} skills from ~/.claude/skills/", skill_count);

    // Scan global agents from ~/.claude/agents/
    let agent_count = scan_global_agents(&db)?;
    log::info!("Found {} agents from ~/.claude/agents/", agent_count);

    // Scan global hooks from ~/.claude/settings.json
    let hook_count = scan_global_hooks(&db)?;
    log::info!("Found {} hooks from ~/.claude/settings.json", hook_count);

    // ============================================================================
    // OpenCode Scanning
    // ============================================================================

    // Scan OpenCode global config for MCPs
    let opencode_mcp_count = scan_opencode_config(&db)?;
    log::info!("Found {} MCPs from OpenCode config", opencode_mcp_count);

    // Scan OpenCode global commands from ~/.config/opencode/command/
    let opencode_command_count = scan_opencode_global_commands(&db)?;
    log::info!("Found {} commands from OpenCode", opencode_command_count);

    // Scan OpenCode global agents from ~/.config/opencode/agent/
    let opencode_agent_count = scan_opencode_global_agents(&db)?;
    log::info!("Found {} agents from OpenCode", opencode_agent_count);

    // ============================================================================
    // Codex CLI Scanning
    // ============================================================================

    // Scan Codex global config for MCPs
    let codex_mcp_count = scan_codex_config(&db)?;
    log::info!("Found {} MCPs from Codex config", codex_mcp_count);

    // ============================================================================
    // GitHub Copilot CLI Scanning
    // ============================================================================

    // Scan Copilot global config for MCPs
    let copilot_mcp_count = scan_copilot_config(&db)?;
    log::info!("Found {} MCPs from Copilot CLI config", copilot_mcp_count);

    // ============================================================================
    // Cursor IDE Scanning
    // ============================================================================

    // Scan Cursor global config for MCPs
    let cursor_mcp_count = scan_cursor_config(&db)?;
    log::info!("Found {} MCPs from Cursor config", cursor_mcp_count);

    // ============================================================================
    // Gemini CLI Scanning
    // ============================================================================

    // Scan Gemini global config for MCPs
    let gemini_mcp_count = scan_gemini_config(&db)?;
    log::info!("Found {} MCPs from Gemini CLI config", gemini_mcp_count);

    Ok(())
}

/// Scan global MCPs from claude.json (root mcpServers)
pub fn scan_global_mcps_from_claude_json(db: &Database) -> Result<usize> {
    let all_mcps = match claude_json::get_all_mcps_from_claude_json() {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to read MCPs from claude.json: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;

    // Only process global MCPs (those without a project_path)
    for mcp in all_mcps.iter().filter(|m| m.project_path.is_none()) {
        // Get or create the MCP in the library
        let mcp_id = get_or_create_mcp(
            db,
            &mcp.name,
            &mcp.mcp_type,
            mcp.command.as_deref(),
            mcp.args.as_ref(),
            mcp.url.as_deref(),
            mcp.headers.as_ref(),
            mcp.env.as_ref(),
            "~/.claude.json",
        )?;

        // Add to global_mcps table if not already there
        let existing: Option<i64> = db
            .conn()
            .query_row(
                "SELECT id FROM global_mcps WHERE mcp_id = ?",
                params![mcp_id],
                |row| row.get(0),
            )
            .ok();

        if existing.is_none() {
            // Get next display order
            let next_order: i64 = db
                .conn()
                .query_row(
                    "SELECT COALESCE(MAX(display_order), 0) + 1 FROM global_mcps",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(1);

            db.conn().execute(
                "INSERT INTO global_mcps (mcp_id, is_enabled, display_order) VALUES (?, ?, ?)",
                params![mcp_id, mcp.is_enabled, next_order],
            )?;
        }

        count += 1;
    }

    Ok(count)
}

/// Scan claude.json for projects and their MCPs
pub fn scan_claude_json(db: &Database) -> Result<usize> {
    let all_projects = match claude_json::get_all_projects_from_claude_json() {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to read claude.json: {}", e);
            return Ok(0);
        }
    };

    let mut mcp_count = 0;
    let mut seen_projects: HashSet<String> = HashSet::new();

    for (project_path, project_config) in all_projects {
        // Normalize path for deduplication
        let normalized_path = normalize_path(&project_path);
        if seen_projects.contains(&normalized_path) {
            continue;
        }
        seen_projects.insert(normalized_path.clone());

        // Check if project directory exists
        let path_to_check = if project_path.starts_with("C:") || project_path.starts_with("c:") {
            project_path.clone()
        } else {
            normalized_path.clone()
        };

        if !Path::new(&path_to_check).exists() {
            log::debug!("Skipping non-existent project: {}", project_path);
            continue;
        }

        // Create or get project (always, even without MCPs)
        let project_name = Path::new(&project_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| project_path.clone());

        let project_id = get_or_create_project(db, &project_name, &project_path)?;

        // Import each MCP (if any)
        for (mcp_name, mcp_server) in project_config.mcp_servers {
            let is_disabled = project_config.disabled_mcp_servers.contains(&mcp_name);

            // Get or create the MCP in the library
            let mcp_id = get_or_create_mcp(
                db,
                &mcp_name,
                &mcp_server.mcp_type,
                mcp_server.command.as_deref(),
                mcp_server.args.as_ref(),
                mcp_server.url.as_deref(),
                mcp_server.headers.as_ref(),
                mcp_server.env.as_ref(),
                &project_path,
            )?;

            // Assign MCP to project if not already assigned
            assign_mcp_to_project(db, project_id, mcp_id, !is_disabled)?;

            mcp_count += 1;
        }

        // Scan project-level commands from .claude/commands/
        let project_commands_dir = Path::new(&path_to_check).join(".claude").join("commands");
        if project_commands_dir.exists() {
            scan_project_commands(db, project_id, &project_commands_dir)?;
        }

        // Scan project-level skills from .claude/skills/
        let project_skills_dir = Path::new(&path_to_check).join(".claude").join("skills");
        if project_skills_dir.exists() {
            scan_project_skills(db, project_id, &project_skills_dir)?;
        }

        // Scan project-level agents from .claude/agents/
        let project_agents_dir = Path::new(&path_to_check).join(".claude").join("agents");
        if project_agents_dir.exists() {
            scan_project_agents(db, project_id, &project_agents_dir)?;
        }

        // Scan project-level hooks from .claude/settings.json and .claude/settings.local.json
        let project_settings_file = Path::new(&path_to_check)
            .join(".claude")
            .join("settings.json");
        if project_settings_file.exists() {
            scan_project_hooks(db, project_id, &project_settings_file)?;
        }
        let project_settings_local_file = Path::new(&path_to_check)
            .join(".claude")
            .join("settings.local.json");
        if project_settings_local_file.exists() {
            scan_project_hooks(db, project_id, &project_settings_local_file)?;
        }
    }

    Ok(mcp_count)
}

/// Get or create a project in the database
fn get_or_create_project(db: &Database, name: &str, path: &str) -> Result<i64> {
    let normalized = normalize_path(path);

    // Try to find existing project by path (check both formats)
    let existing_id: Option<i64> = db
        .conn()
        .query_row(
            "SELECT id FROM projects WHERE path = ? OR path = ?",
            params![path, normalized],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        return Ok(id);
    }

    // Create new project
    db.conn().execute(
        "INSERT INTO projects (name, path, has_mcp_file, has_settings_file) VALUES (?, ?, 0, 0)",
        params![name, normalized],
    )?;

    Ok(db.conn().last_insert_rowid())
}

/// Get or create an MCP in the library
fn get_or_create_mcp(
    db: &Database,
    name: &str,
    mcp_type: &str,
    command: Option<&str>,
    args: Option<&Vec<String>>,
    url: Option<&str>,
    headers: Option<&std::collections::HashMap<String, String>>,
    env: Option<&std::collections::HashMap<String, String>>,
    source_path: &str,
) -> Result<i64> {
    // Try to find existing MCP by name
    let existing_id: Option<i64> = db
        .conn()
        .query_row("SELECT id FROM mcps WHERE name = ?", params![name], |row| {
            row.get(0)
        })
        .ok();

    if let Some(id) = existing_id {
        // Update source_path if not already set
        db.conn().execute(
            "UPDATE mcps SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
            params![source_path, id],
        )?;
        return Ok(id);
    }

    // Create new MCP
    let args_json = args.map(|a| serde_json::to_string(a).unwrap());
    let headers_json = headers.map(|h| serde_json::to_string(h).unwrap());
    let env_json = env.map(|e| serde_json::to_string(e).unwrap());

    db.conn().execute(
        "INSERT INTO mcps (name, type, command, args, url, headers, env, source, source_path)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'auto-detected', ?)",
        params![
            name,
            mcp_type,
            command,
            args_json,
            url,
            headers_json,
            env_json,
            source_path
        ],
    )?;

    Ok(db.conn().last_insert_rowid())
}

/// Assign an MCP to a project
fn assign_mcp_to_project(
    db: &Database,
    project_id: i64,
    mcp_id: i64,
    is_enabled: bool,
) -> Result<()> {
    // Check if already assigned
    let exists: bool = db
        .conn()
        .query_row(
            "SELECT 1 FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
            params![project_id, mcp_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if exists {
        // Update enabled state
        db.conn().execute(
            "UPDATE project_mcps SET is_enabled = ? WHERE project_id = ? AND mcp_id = ?",
            params![is_enabled, project_id, mcp_id],
        )?;
    } else {
        // Create assignment
        let display_order: i32 = db
            .conn()
            .query_row(
                "SELECT COALESCE(MAX(display_order), 0) + 1 FROM project_mcps WHERE project_id = ?",
                params![project_id],
                |row| row.get(0),
            )
            .unwrap_or(1);

        db.conn().execute(
            "INSERT INTO project_mcps (project_id, mcp_id, is_enabled, display_order)
             VALUES (?, ?, ?, ?)",
            params![project_id, mcp_id, is_enabled, display_order],
        )?;
    }

    Ok(())
}

/// Import MCPs from a project's .mcp.json into the database.
/// Called when a project is added so externally-configured servers show up in the UI.
pub fn import_mcps_from_project_mcp_json(
    db: &Database,
    project_id: i64,
    project_path: &str,
) -> Result<usize> {
    let path = std::path::PathBuf::from(project_path);
    let mcp_file = path.join(".mcp.json");

    if !mcp_file.exists() {
        return Ok(0);
    }

    let mcps = match config_parser::parse_mcp_file(&mcp_file) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to parse .mcp.json at {}: {}", mcp_file.display(), e);
            return Ok(0);
        }
    };

    let mut count = 0;
    for mcp in &mcps {
        let mcp_id = get_or_create_mcp(
            db,
            &mcp.name,
            &mcp.mcp_type,
            mcp.command.as_deref(),
            mcp.args.as_ref(),
            mcp.url.as_deref(),
            mcp.headers.as_ref(),
            mcp.env.as_ref(),
            project_path,
        )?;

        assign_mcp_to_project(db, project_id, mcp_id, true)?;
        count += 1;
    }

    log::info!(
        "Imported {} MCPs from .mcp.json for project {}",
        count,
        project_path
    );
    Ok(count)
}

/// Scan plugins/marketplaces directory for MCPs
pub fn scan_plugins(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let mut count = 0;

    if paths.marketplaces_dir.exists() {
        for entry in WalkDir::new(&paths.marketplaces_dir)
            .max_depth(6)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == ".mcp.json" {
                match config_parser::parse_mcp_file(entry.path()) {
                    Ok(mcps) => {
                        for mcp in mcps {
                            let source_path = entry.path().to_string_lossy().to_string();

                            // Check if already exists
                            let existing_id: Option<i64> = db
                                .conn()
                                .query_row(
                                    "SELECT id FROM mcps WHERE name = ?",
                                    [&mcp.name],
                                    |row| row.get(0),
                                )
                                .ok();

                            if let Some(id) = existing_id {
                                // Update source_path if not already set
                                db.conn().execute(
                                    "UPDATE mcps SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                                    params![&source_path, id],
                                )?;
                            } else {
                                let args_json =
                                    mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
                                let headers_json = mcp
                                    .headers
                                    .as_ref()
                                    .map(|h| serde_json::to_string(h).unwrap());
                                let env_json =
                                    mcp.env.as_ref().map(|e| serde_json::to_string(e).unwrap());

                                let result = db.conn().execute(
                                    "INSERT INTO mcps (name, type, command, args, url, headers, env, source, source_path)
                                     VALUES (?, ?, ?, ?, ?, ?, ?, 'auto-detected', ?)",
                                    params![
                                        mcp.name,
                                        mcp.mcp_type,
                                        mcp.command,
                                        args_json,
                                        mcp.url,
                                        headers_json,
                                        env_json,
                                        source_path
                                    ],
                                );

                                if result.is_ok() {
                                    count += 1;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to parse {:?}: {}", entry.path(), e);
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Scan global commands from ~/.claude/commands/
pub fn scan_global_commands(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let mut count = 0;

    if paths.commands_dir.exists() {
        for entry in std::fs::read_dir(&paths.commands_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .md files
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Some(command) = parse_skill_file(&path) {
                    // Use get_or_create_command to insert into commands table
                    let source_path = path.to_string_lossy().to_string();
                    let (_, was_created) = get_or_create_command(db, &command, &source_path)?;
                    if was_created {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Scan global skills from ~/.claude/skills/ directories
pub fn scan_global_skills(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let mut count = 0;

    if paths.skills_dir.exists() {
        for entry in std::fs::read_dir(&paths.skills_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process directories (each skill is a directory with SKILL.md)
            if path.is_dir() {
                if let Some((skill, files)) = parse_agent_skill_dir(&path) {
                    // Use get_or_create_skill to insert into skills table
                    let source_path = path.to_string_lossy().to_string();
                    let (skill_id, was_created) = get_or_create_skill(db, &skill, &source_path)?;

                    // Insert skill files if this is a new skill
                    if was_created {
                        if !files.is_empty() {
                            let _ = insert_skill_files(db, skill_id, &files);
                        }
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Scan global agents from ~/.claude/agents/
pub fn scan_global_agents(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let mut count = 0;

    if paths.agents_dir.exists() {
        for entry in std::fs::read_dir(&paths.agents_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .md files
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Some(agent) = parse_agent_file(&path) {
                    let source_path = path.to_string_lossy().to_string();

                    // Check if already exists
                    let existing_id: Option<i64> = db
                        .conn()
                        .query_row(
                            "SELECT id FROM subagents WHERE name = ?",
                            [&agent.name],
                            |row| row.get(0),
                        )
                        .ok();

                    if let Some(id) = existing_id {
                        // Update source_path if not already set
                        db.conn().execute(
                            "UPDATE subagents SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                            params![&source_path, id],
                        )?;
                    } else {
                        let tools_json = if agent.tools.is_empty() {
                            None
                        } else {
                            Some(serde_json::to_string(&agent.tools).unwrap())
                        };
                        let skills_json = if agent.skills.is_empty() {
                            None
                        } else {
                            Some(serde_json::to_string(&agent.skills).unwrap())
                        };
                        let tags_json = if agent.tags.is_empty() {
                            None
                        } else {
                            Some(serde_json::to_string(&agent.tags).unwrap())
                        };

                        let result = db.conn().execute(
                            "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source, source_path)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected', ?)",
                            params![
                                agent.name,
                                agent.description,
                                agent.content,
                                tools_json,
                                agent.model,
                                agent.permission_mode,
                                skills_json,
                                tags_json,
                                source_path
                            ],
                        );

                        if result.is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Parsed skill data from markdown file
#[derive(Debug, PartialEq)]
pub(crate) struct ParsedSkill {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) content: String,
    pub(crate) skill_type: String,
    pub(crate) allowed_tools: Option<String>,
    pub(crate) argument_hint: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) disable_model_invocation: bool,
    pub(crate) tags: Vec<String>,
}

/// Parsed skill file data (references, assets, scripts)
#[derive(Debug, PartialEq)]
pub(crate) struct ParsedSkillFile {
    pub(crate) file_type: String, // "reference", "asset", "script"
    pub(crate) name: String,
    pub(crate) content: String,
}

/// Parsed agent data from markdown file
#[derive(Debug, PartialEq)]
pub(crate) struct ParsedAgent {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) content: String,
    pub(crate) tools: Vec<String>,
    pub(crate) model: Option<String>,
    pub(crate) permission_mode: Option<String>,
    pub(crate) skills: Vec<String>,
    pub(crate) tags: Vec<String>,
}

/// Parse a skill markdown file
pub(crate) fn parse_skill_file(path: &Path) -> Option<ParsedSkill> {
    let content = std::fs::read_to_string(path).ok()?;
    let file_name = path.file_stem()?.to_string_lossy().to_string();

    // Parse frontmatter if present (between --- markers)
    let (frontmatter, body) = parse_frontmatter(&content);

    // Extract metadata from frontmatter
    let description = frontmatter.get("description").cloned();
    let skill_type = frontmatter
        .get("type")
        .cloned()
        .unwrap_or_else(|| "command".to_string());
    // Support multiple formats: allowed-tools (official), allowed_tools, allowedTools
    let allowed_tools = frontmatter
        .get("allowed-tools")
        .or_else(|| frontmatter.get("allowed_tools"))
        .or_else(|| frontmatter.get("allowedTools"))
        .cloned();
    // Support multiple formats: argument-hint (official), argument_hint, argumentHint
    let argument_hint = frontmatter
        .get("argument-hint")
        .or_else(|| frontmatter.get("argument_hint"))
        .or_else(|| frontmatter.get("argumentHint"))
        .cloned();
    // Model (optional)
    let model = frontmatter.get("model").cloned();
    // disable-model-invocation / disableModelInvocation
    let disable_model_invocation = frontmatter
        .get("disable-model-invocation")
        .or_else(|| frontmatter.get("disableModelInvocation"))
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let tags = frontmatter
        .get("tags")
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Some(ParsedSkill {
        name: file_name,
        description,
        content: body,
        skill_type,
        allowed_tools,
        argument_hint,
        model,
        disable_model_invocation,
        tags,
    })
}

/// Parse an agent skill directory (e.g., .claude/skills/my-skill/)
/// Returns the skill from SKILL.md and any files from references/assets/scripts subdirs
fn parse_agent_skill_dir(skill_dir: &Path) -> Option<(ParsedSkill, Vec<ParsedSkillFile>)> {
    let skill_md_path = skill_dir.join("SKILL.md");
    if !skill_md_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&skill_md_path).ok()?;
    let skill_name = skill_dir.file_name()?.to_string_lossy().to_string();

    // Parse frontmatter if present
    let (frontmatter, body) = parse_frontmatter(&content);

    // Extract metadata - agent skills are always type "skill"
    let description = frontmatter.get("description").cloned();
    let allowed_tools = frontmatter
        .get("allowed-tools")
        .or_else(|| frontmatter.get("allowed_tools"))
        .or_else(|| frontmatter.get("allowedTools"))
        .cloned();
    let argument_hint = frontmatter
        .get("argument-hint")
        .or_else(|| frontmatter.get("argument_hint"))
        .or_else(|| frontmatter.get("argumentHint"))
        .cloned();
    let model = frontmatter.get("model").cloned();
    let disable_model_invocation = frontmatter
        .get("disable-model-invocation")
        .or_else(|| frontmatter.get("disableModelInvocation"))
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    let tags = frontmatter
        .get("tags")
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let skill = ParsedSkill {
        name: skill_name,
        description,
        content: body,
        skill_type: "skill".to_string(), // Agent skills are always type "skill"
        allowed_tools,
        argument_hint,
        model,
        disable_model_invocation,
        tags,
    };

    // Scan subdirectories for files
    let mut files = Vec::new();

    // Map directory names to file types
    let subdir_mappings = [
        ("references", "reference"),
        ("assets", "asset"),
        ("scripts", "script"),
    ];

    for (dir_name, file_type) in subdir_mappings {
        let subdir = skill_dir.join(dir_name);
        if subdir.exists() && subdir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&subdir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name() {
                            let name = file_name.to_string_lossy().to_string();
                            if let Ok(file_content) = std::fs::read_to_string(&path) {
                                files.push(ParsedSkillFile {
                                    file_type: file_type.to_string(),
                                    name,
                                    content: file_content,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Some((skill, files))
}

/// Parse an agent markdown file
pub(crate) fn parse_agent_file(path: &Path) -> Option<ParsedAgent> {
    let content = std::fs::read_to_string(path).ok()?;
    let file_name = path.file_stem()?.to_string_lossy().to_string();

    // Parse frontmatter if present
    let (frontmatter, body) = parse_frontmatter(&content);

    // Extract metadata from frontmatter
    let description = frontmatter
        .get("description")
        .cloned()
        .unwrap_or_else(|| file_name.clone());
    let model = frontmatter.get("model").cloned();
    let permission_mode = frontmatter
        .get("permissionMode")
        .or_else(|| frontmatter.get("permission_mode"))
        .cloned();
    let tools = frontmatter
        .get("tools")
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let skills = frontmatter
        .get("skills")
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let tags = frontmatter
        .get("tags")
        .map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Some(ParsedAgent {
        name: file_name,
        description,
        content: body,
        tools,
        model,
        permission_mode,
        skills,
        tags,
    })
}

/// Parse YAML-like frontmatter from markdown content
pub(crate) fn parse_frontmatter(
    content: &str,
) -> (std::collections::HashMap<String, String>, String) {
    let mut frontmatter = std::collections::HashMap::new();

    if content.starts_with("---") {
        // Find the closing ---
        if let Some(end_pos) = content[3..].find("\n---") {
            let fm_content = &content[3..end_pos + 3];
            let body = content[end_pos + 7..].trim_start().to_string();

            // Parse simple key: value pairs
            for line in fm_content.lines() {
                let line = line.trim();
                if let Some(colon_pos) = line.find(':') {
                    let key = line[..colon_pos].trim().to_string();
                    let value = line[colon_pos + 1..].trim().to_string();
                    if !key.is_empty() && !value.is_empty() {
                        frontmatter.insert(key, value);
                    }
                }
            }

            return (frontmatter, body);
        }
    }

    // No frontmatter, return content as-is
    (frontmatter, content.to_string())
}

/// Scan project-level commands from .claude/commands/ and assign to project
fn scan_project_commands(db: &Database, project_id: i64, commands_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(commands_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .md files
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(command) = parse_skill_file(&path) {
                // Get or create the command in the library
                let source_path = path.to_string_lossy().to_string();
                let (command_id, _) = get_or_create_command(db, &command, &source_path)?;

                // Assign command to project if not already assigned
                assign_command_to_project(db, project_id, command_id)?;

                count += 1;
            }
        }
    }

    Ok(count)
}

/// Scan project-level skills from .claude/skills/ and assign to project
fn scan_project_skills(db: &Database, project_id: i64, skills_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process directories (each skill is a directory with SKILL.md)
        if path.is_dir() {
            if let Some((skill, files)) = parse_agent_skill_dir(&path) {
                // Get or create the skill in the library
                let source_path = path.to_string_lossy().to_string();
                let (skill_id, was_created) = get_or_create_skill(db, &skill, &source_path)?;

                // Insert skill files if this is a new skill
                if was_created && !files.is_empty() {
                    let _ = insert_skill_files(db, skill_id, &files);
                }

                // Assign skill to project if not already assigned
                assign_skill_to_project(db, project_id, skill_id)?;

                count += 1;
            }
        }
    }

    Ok(count)
}

/// Scan project-level agents and assign to project
fn scan_project_agents(db: &Database, project_id: i64, agents_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(agents_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .md files
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(agent) = parse_agent_file(&path) {
                // Get or create the agent in the library
                let source_path = path.to_string_lossy().to_string();
                let agent_id = get_or_create_agent(db, &agent, &source_path)?;

                // Assign agent to project if not already assigned
                assign_agent_to_project(db, project_id, agent_id)?;

                count += 1;
            }
        }
    }

    Ok(count)
}

/// Get or create a skill in the database, returning (skill_id, was_created)
fn get_or_create_skill(
    db: &Database,
    skill: &ParsedSkill,
    source_path: &str,
) -> Result<(i64, bool)> {
    // Try to find existing skill by name
    let existing_id: Option<i64> = db
        .conn()
        .query_row(
            "SELECT id FROM skills WHERE name = ?",
            [&skill.name],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        // Update source_path if not already set
        db.conn().execute(
            "UPDATE skills SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
            params![source_path, id],
        )?;
        return Ok((id, false));
    }

    // Create new skill
    let tags_json = if skill.tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&skill.tags).unwrap())
    };

    db.conn().execute(
        "INSERT INTO skills (name, description, content, allowed_tools, model, disable_model_invocation, tags, source, source_path)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'auto-detected', ?)",
        params![
            skill.name,
            skill.description,
            skill.content,
            skill.allowed_tools,
            skill.model,
            skill.disable_model_invocation,
            tags_json,
            source_path
        ],
    )?;

    Ok((db.conn().last_insert_rowid(), true))
}

/// Insert skill files into the database
fn insert_skill_files(db: &Database, skill_id: i64, files: &[ParsedSkillFile]) -> Result<usize> {
    let mut count = 0;
    for file in files {
        // Check if file already exists
        let exists: bool = db
            .conn()
            .query_row(
                "SELECT 1 FROM skill_files WHERE skill_id = ? AND file_type = ? AND name = ?",
                params![skill_id, file.file_type, file.name],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !exists {
            db.conn().execute(
                "INSERT INTO skill_files (skill_id, file_type, name, content) VALUES (?, ?, ?, ?)",
                params![skill_id, file.file_type, file.name, file.content],
            )?;
            count += 1;
        }
    }
    Ok(count)
}

/// Get or create an agent in the database
fn get_or_create_agent(db: &Database, agent: &ParsedAgent, source_path: &str) -> Result<i64> {
    // Try to find existing agent by name
    let existing_id: Option<i64> = db
        .conn()
        .query_row(
            "SELECT id FROM subagents WHERE name = ?",
            [&agent.name],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        // Update source_path if not already set
        db.conn().execute(
            "UPDATE subagents SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
            params![source_path, id],
        )?;
        return Ok(id);
    }

    // Create new agent
    let tools_json = if agent.tools.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&agent.tools).unwrap())
    };
    let skills_json = if agent.skills.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&agent.skills).unwrap())
    };
    let tags_json = if agent.tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&agent.tags).unwrap())
    };

    db.conn().execute(
        "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source, source_path)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected', ?)",
        params![
            agent.name,
            agent.description,
            agent.content,
            tools_json,
            agent.model,
            agent.permission_mode,
            skills_json,
            tags_json,
            source_path
        ],
    )?;

    Ok(db.conn().last_insert_rowid())
}

/// Assign a skill to a project
fn assign_skill_to_project(db: &Database, project_id: i64, skill_id: i64) -> Result<()> {
    // Check if already assigned
    let exists: bool = db
        .conn()
        .query_row(
            "SELECT 1 FROM project_skills WHERE project_id = ? AND skill_id = ?",
            params![project_id, skill_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !exists {
        db.conn().execute(
            "INSERT INTO project_skills (project_id, skill_id, is_enabled) VALUES (?, ?, 1)",
            params![project_id, skill_id],
        )?;
    }

    Ok(())
}

/// Get or create a command in the database, returning (command_id, was_created)
fn get_or_create_command(
    db: &Database,
    command: &ParsedSkill,
    source_path: &str,
) -> Result<(i64, bool)> {
    // Try to find existing command by name
    let existing_id: Option<i64> = db
        .conn()
        .query_row(
            "SELECT id FROM commands WHERE name = ?",
            [&command.name],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        // Update source_path if not already set
        db.conn().execute(
            "UPDATE commands SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
            params![source_path, id],
        )?;
        return Ok((id, false));
    }

    // Create new command
    let tags_json = if command.tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&command.tags).unwrap())
    };

    db.conn().execute(
        "INSERT INTO commands (name, description, content, allowed_tools, model, tags, source, source_path)
         VALUES (?, ?, ?, ?, ?, ?, 'auto-detected', ?)",
        params![
            command.name,
            command.description,
            command.content,
            command.allowed_tools,
            command.model,
            tags_json,
            source_path
        ],
    )?;

    Ok((db.conn().last_insert_rowid(), true))
}

/// Assign a command to a project
fn assign_command_to_project(db: &Database, project_id: i64, command_id: i64) -> Result<()> {
    // Check if already assigned
    let exists: bool = db
        .conn()
        .query_row(
            "SELECT 1 FROM project_commands WHERE project_id = ? AND command_id = ?",
            params![project_id, command_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !exists {
        db.conn().execute(
            "INSERT INTO project_commands (project_id, command_id, is_enabled) VALUES (?, ?, 1)",
            params![project_id, command_id],
        )?;
    }

    Ok(())
}

/// Assign an agent to a project
fn assign_agent_to_project(db: &Database, project_id: i64, agent_id: i64) -> Result<()> {
    // Check if already assigned
    let exists: bool = db
        .conn()
        .query_row(
            "SELECT 1 FROM project_subagents WHERE project_id = ? AND subagent_id = ?",
            params![project_id, agent_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !exists {
        db.conn().execute(
            "INSERT INTO project_subagents (project_id, subagent_id, is_enabled) VALUES (?, ?, 1)",
            params![project_id, agent_id],
        )?;
    }

    Ok(())
}

/// Parsed hook data from settings.json
#[derive(Debug)]
pub(crate) struct ParsedHook {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) event_type: String,
    pub(crate) matcher: Option<String>,
    pub(crate) hook_type: String,
    pub(crate) command: Option<String>,
    pub(crate) prompt: Option<String>,
    pub(crate) timeout: Option<i32>,
}

/// Parse hooks from a settings.json file
pub(crate) fn parse_hooks_from_settings(path: &Path) -> Vec<ParsedHook> {
    let mut hooks = Vec::new();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return hooks,
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return hooks,
    };

    // Parse the "hooks" object
    if let Some(hooks_obj) = json.get("hooks").and_then(|h| h.as_object()) {
        for (event_type, event_hooks) in hooks_obj {
            if let Some(hook_array) = event_hooks.as_array() {
                for (idx, hook_entry) in hook_array.iter().enumerate() {
                    let matcher = hook_entry
                        .get("matcher")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string());

                    if let Some(inner_hooks) = hook_entry.get("hooks").and_then(|h| h.as_array()) {
                        for (inner_idx, inner_hook) in inner_hooks.iter().enumerate() {
                            let hook_type = inner_hook
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("command")
                                .to_string();

                            let command = inner_hook
                                .get("command")
                                .and_then(|c| c.as_str())
                                .map(|s| s.to_string());
                            let prompt = inner_hook
                                .get("prompt")
                                .and_then(|p| p.as_str())
                                .map(|s| s.to_string());
                            let timeout = inner_hook
                                .get("timeout")
                                .and_then(|t| t.as_i64())
                                .map(|t| t as i32);

                            // Generate a name from the event type and index
                            let name = if let Some(ref m) = matcher {
                                format!(
                                    "{}-{}-{}",
                                    event_type.to_lowercase(),
                                    m.replace('|', "-"),
                                    inner_idx
                                )
                            } else {
                                format!("{}-{}-{}", event_type.to_lowercase(), idx, inner_idx)
                            };

                            // Generate description
                            let description = Some(format!(
                                "{} hook{}",
                                event_type,
                                matcher
                                    .as_ref()
                                    .map(|m| format!(" for {}", m))
                                    .unwrap_or_default()
                            ));

                            hooks.push(ParsedHook {
                                name,
                                description,
                                event_type: event_type.clone(),
                                matcher: matcher.clone(),
                                hook_type,
                                command,
                                prompt,
                                timeout,
                            });
                        }
                    }
                }
            }
        }
    }

    hooks
}

/// Scan global hooks from ~/.claude/settings.json
pub fn scan_global_hooks(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let settings_path = paths.global_settings;

    if !settings_path.exists() {
        return Ok(0);
    }

    let hooks = parse_hooks_from_settings(&settings_path);
    let mut count = 0;

    for hook in hooks {
        // Check if already exists by name
        let existing_id: Option<i64> = db
            .conn()
            .query_row("SELECT id FROM hooks WHERE name = ?", [&hook.name], |row| {
                row.get(0)
            })
            .ok();

        let hook_id = if let Some(id) = existing_id {
            id
        } else {
            // Create new hook
            db.conn().execute(
                "INSERT INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, source)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
                params![
                    hook.name,
                    hook.description,
                    hook.event_type,
                    hook.matcher,
                    hook.hook_type,
                    hook.command,
                    hook.prompt,
                    hook.timeout
                ],
            )?;
            db.conn().last_insert_rowid()
        };

        // Always ensure it's in global_hooks
        let _ = db.conn().execute(
            "INSERT OR IGNORE INTO global_hooks (hook_id, is_enabled) VALUES (?, 1)",
            [hook_id],
        );
        count += 1;
    }

    Ok(count)
}

/// Scan project hooks from a settings file (.claude/settings.json or .claude/settings.local.json)
fn scan_project_hooks(db: &Database, project_id: i64, settings_path: &Path) -> Result<usize> {
    let hooks = parse_hooks_from_settings(settings_path);
    let mut count = 0;

    for hook in hooks {
        // Get or create the hook in the library
        let hook_id = get_or_create_hook(db, &hook)?;

        // Assign hook to project if not already assigned
        assign_hook_to_project(db, project_id, hook_id)?;

        count += 1;
    }

    Ok(count)
}

/// Get or create a hook in the database
fn get_or_create_hook(db: &Database, hook: &ParsedHook) -> Result<i64> {
    // Try to find existing hook by name
    let existing_id: Option<i64> = db
        .conn()
        .query_row("SELECT id FROM hooks WHERE name = ?", [&hook.name], |row| {
            row.get(0)
        })
        .ok();

    if let Some(id) = existing_id {
        return Ok(id);
    }

    // Create new hook
    db.conn().execute(
        "INSERT INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, source)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
        params![
            hook.name,
            hook.description,
            hook.event_type,
            hook.matcher,
            hook.hook_type,
            hook.command,
            hook.prompt,
            hook.timeout
        ],
    )?;

    Ok(db.conn().last_insert_rowid())
}

/// Assign a hook to a project
fn assign_hook_to_project(db: &Database, project_id: i64, hook_id: i64) -> Result<()> {
    // Check if already assigned
    let exists: bool = db
        .conn()
        .query_row(
            "SELECT 1 FROM project_hooks WHERE project_id = ? AND hook_id = ?",
            params![project_id, hook_id],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !exists {
        db.conn().execute(
            "INSERT INTO project_hooks (project_id, hook_id, is_enabled) VALUES (?, ?, 1)",
            params![project_id, hook_id],
        )?;
    }

    Ok(())
}

// ============================================================================
// OpenCode Scanning Functions
// ============================================================================

/// Scan OpenCode global config for MCPs
pub fn scan_opencode_config(db: &Database) -> Result<usize> {
    let paths = match get_opencode_paths() {
        Ok(p) => p,
        Err(e) => {
            log::debug!("OpenCode paths not available: {}", e);
            return Ok(0);
        }
    };

    if !paths.config_file.exists() {
        log::debug!("OpenCode config not found at {:?}", paths.config_file);
        return Ok(0);
    }

    // Parse MCPs from opencode.json
    let mcps = match opencode_config::parse_opencode_mcps(&paths.config_file) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to parse OpenCode config: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;

    for mcp in mcps {
        // Map OpenCode types to our internal types
        // OpenCode "local" -> our "stdio", OpenCode "remote" -> our "http"
        let mcp_type = match mcp.mcp_type.as_str() {
            "local" => "stdio",
            "remote" => "http",
            other => other,
        };

        let source_path = paths.config_file.to_string_lossy().to_string();

        // Check if already exists
        let existing_id: Option<i64> = db
            .conn()
            .query_row("SELECT id FROM mcps WHERE name = ?", [&mcp.name], |row| {
                row.get(0)
            })
            .ok();

        if let Some(id) = existing_id {
            // Update source_path if not already set
            db.conn().execute(
                "UPDATE mcps SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                params![&source_path, id],
            )?;
        } else {
            let args_json = match &mcp.args {
                Some(args) if !args.is_empty() => Some(serde_json::to_string(args).unwrap()),
                _ => None,
            };
            let env_json = match &mcp.env {
                Some(env) if !env.is_empty() => Some(serde_json::to_string(env).unwrap()),
                _ => None,
            };
            let headers_json = match &mcp.headers {
                Some(headers) if !headers.is_empty() => {
                    Some(serde_json::to_string(headers).unwrap())
                }
                _ => None,
            };

            let result = db.conn().execute(
                "INSERT INTO mcps (name, type, command, args, url, headers, env, source, source_path)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'opencode', ?)",
                params![
                    mcp.name,
                    mcp_type,
                    mcp.command,
                    args_json,
                    mcp.url,
                    headers_json,
                    env_json,
                    source_path
                ],
            );

            if result.is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Scan OpenCode global commands from ~/.config/opencode/command/
pub fn scan_opencode_global_commands(db: &Database) -> Result<usize> {
    let paths = match get_opencode_paths() {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };

    if !paths.command_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;

    for entry in std::fs::read_dir(&paths.command_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .md files
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(skill) = parse_skill_file(&path) {
                // Check if already exists
                let exists: bool = db
                    .conn()
                    .query_row("SELECT 1 FROM skills WHERE name = ?", [&skill.name], |_| {
                        Ok(true)
                    })
                    .unwrap_or(false);

                if !exists {
                    let tags_json = if skill.tags.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&skill.tags).unwrap())
                    };

                    let result = db.conn().execute(
                        "INSERT INTO skills (name, description, content, allowed_tools, model, disable_model_invocation, tags, source)
                         VALUES (?, ?, ?, ?, ?, ?, ?, 'opencode')",
                        params![
                            skill.name,
                            skill.description,
                            skill.content,
                            skill.allowed_tools,
                            skill.model,
                            skill.disable_model_invocation,
                            tags_json
                        ],
                    );

                    if result.is_ok() {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Scan OpenCode global agents from ~/.config/opencode/agent/
pub fn scan_opencode_global_agents(db: &Database) -> Result<usize> {
    let paths = match get_opencode_paths() {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };

    if !paths.agent_dir.exists() {
        return Ok(0);
    }

    let mut count = 0;

    for entry in std::fs::read_dir(&paths.agent_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .md files
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(agent) = parse_agent_file(&path) {
                let source_path = path.to_string_lossy().to_string();

                // Check if already exists
                let existing_id: Option<i64> = db
                    .conn()
                    .query_row(
                        "SELECT id FROM subagents WHERE name = ?",
                        [&agent.name],
                        |row| row.get(0),
                    )
                    .ok();

                if let Some(id) = existing_id {
                    // Update source_path if not already set
                    db.conn().execute(
                        "UPDATE subagents SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                        params![&source_path, id],
                    )?;
                } else {
                    let tools_json = if agent.tools.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&agent.tools).unwrap())
                    };
                    let skills_json = if agent.skills.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&agent.skills).unwrap())
                    };
                    let tags_json = if agent.tags.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&agent.tags).unwrap())
                    };

                    let result = db.conn().execute(
                        "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source, source_path)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'opencode', ?)",
                        params![
                            agent.name,
                            agent.description,
                            agent.content,
                            tools_json,
                            agent.model,
                            agent.permission_mode,
                            skills_json,
                            tags_json,
                            source_path
                        ],
                    );

                    if result.is_ok() {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}

// ============================================================================
// Codex CLI Scanning Functions
// ============================================================================

/// Scan Codex CLI global config for MCPs
pub fn scan_codex_config(db: &Database) -> Result<usize> {
    let paths = match get_codex_paths() {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Codex paths not available: {}", e);
            return Ok(0);
        }
    };

    if !paths.config_file.exists() {
        log::debug!("Codex config not found at {:?}", paths.config_file);
        return Ok(0);
    }

    // Parse MCPs from config.toml
    let mcps = match codex_config::parse_codex_mcps(&paths.config_file) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to parse Codex config: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;

    for mcp in mcps {
        let source_path = paths.config_file.to_string_lossy().to_string();

        // Check if already exists
        let existing_id: Option<i64> = db
            .conn()
            .query_row("SELECT id FROM mcps WHERE name = ?", [&mcp.name], |row| {
                row.get(0)
            })
            .ok();

        if let Some(id) = existing_id {
            // Update source_path if not already set
            db.conn().execute(
                "UPDATE mcps SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                params![&source_path, id],
            )?;
        } else {
            let args_json = match &mcp.args {
                Some(args) if !args.is_empty() => Some(serde_json::to_string(args).unwrap()),
                _ => None,
            };
            let env_json = match &mcp.env {
                Some(env) if !env.is_empty() => Some(serde_json::to_string(env).unwrap()),
                _ => None,
            };
            let headers_json = match &mcp.headers {
                Some(headers) if !headers.is_empty() => {
                    Some(serde_json::to_string(headers).unwrap())
                }
                _ => None,
            };

            let result = db.conn().execute(
                "INSERT INTO mcps (name, type, command, args, url, headers, env, source, source_path)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'codex', ?)",
                params![
                    mcp.name,
                    mcp.mcp_type,
                    mcp.command,
                    args_json,
                    mcp.url,
                    headers_json,
                    env_json,
                    source_path
                ],
            );

            if result.is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

// ============================================================================
// GitHub Copilot CLI Scanning Functions
// ============================================================================

/// Scan GitHub Copilot CLI mcp-config.json for MCPs
pub fn scan_copilot_config(db: &Database) -> Result<usize> {
    let paths = match get_copilot_paths() {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Copilot paths not available: {}", e);
            return Ok(0);
        }
    };

    if !paths.mcp_config_file.exists() {
        log::debug!(
            "Copilot mcp-config.json not found at {:?}",
            paths.mcp_config_file
        );
        return Ok(0);
    }

    // Parse MCPs from mcp-config.json
    let mcps = match copilot_config::parse_copilot_mcps(&paths.mcp_config_file) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to parse Copilot mcp-config.json: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;

    for mcp in mcps {
        let source_path = paths.mcp_config_file.to_string_lossy().to_string();

        // Check if already exists
        let existing_id: Option<i64> = db
            .conn()
            .query_row("SELECT id FROM mcps WHERE name = ?", [&mcp.name], |row| {
                row.get(0)
            })
            .ok();

        if let Some(id) = existing_id {
            // Update source_path if not already set
            db.conn().execute(
                "UPDATE mcps SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                params![&source_path, id],
            )?;
        } else {
            let args_json = match &mcp.args {
                Some(args) if !args.is_empty() => Some(serde_json::to_string(args).unwrap()),
                _ => None,
            };
            let env_json = match &mcp.env {
                Some(env) if !env.is_empty() => Some(serde_json::to_string(env).unwrap()),
                _ => None,
            };
            let headers_json = match &mcp.headers {
                Some(headers) if !headers.is_empty() => {
                    Some(serde_json::to_string(headers).unwrap())
                }
                _ => None,
            };

            let result = db.conn().execute(
                "INSERT INTO mcps (name, type, command, args, url, headers, env, source, source_path)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'copilot', ?)",
                params![
                    mcp.name,
                    mcp.mcp_type,
                    mcp.command,
                    args_json,
                    mcp.url,
                    headers_json,
                    env_json,
                    source_path
                ],
            );

            if result.is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

// ============================================================================
// Cursor IDE Scanning Functions
// ============================================================================

/// Scan Cursor IDE mcp.json for MCPs
pub fn scan_cursor_config(db: &Database) -> Result<usize> {
    let paths = match get_cursor_paths() {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Cursor paths not available: {}", e);
            return Ok(0);
        }
    };

    if !paths.mcp_config_file.exists() {
        log::debug!("Cursor mcp.json not found at {:?}", paths.mcp_config_file);
        return Ok(0);
    }

    // Parse MCPs from mcp.json
    let mcps = match cursor_config::parse_cursor_mcps(&paths.mcp_config_file) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to parse Cursor mcp.json: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;

    for mcp in mcps {
        let source_path = paths.mcp_config_file.to_string_lossy().to_string();

        // Check if already exists
        let existing_id: Option<i64> = db
            .conn()
            .query_row("SELECT id FROM mcps WHERE name = ?", [&mcp.name], |row| {
                row.get(0)
            })
            .ok();

        if let Some(id) = existing_id {
            // Update source_path if not already set
            db.conn().execute(
                "UPDATE mcps SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                params![&source_path, id],
            )?;
        } else {
            let args_json = match &mcp.args {
                Some(args) if !args.is_empty() => Some(serde_json::to_string(args).unwrap()),
                _ => None,
            };
            let env_json = match &mcp.env {
                Some(env) if !env.is_empty() => Some(serde_json::to_string(env).unwrap()),
                _ => None,
            };
            let headers_json = match &mcp.headers {
                Some(headers) if !headers.is_empty() => {
                    Some(serde_json::to_string(headers).unwrap())
                }
                _ => None,
            };

            let result = db.conn().execute(
                "INSERT INTO mcps (name, type, command, args, url, headers, env, source, source_path)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'cursor', ?)",
                params![
                    mcp.name,
                    mcp.mcp_type,
                    mcp.command,
                    args_json,
                    mcp.url,
                    headers_json,
                    env_json,
                    source_path
                ],
            );

            if result.is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

// ============================================================================
// Gemini CLI Scanning Functions
// ============================================================================

/// Scan Gemini CLI settings.json for MCPs
pub fn scan_gemini_config(db: &Database) -> Result<usize> {
    let paths = match get_gemini_paths() {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Gemini paths not available: {}", e);
            return Ok(0);
        }
    };

    if !paths.settings_file.exists() {
        log::debug!(
            "Gemini settings.json not found at {:?}",
            paths.settings_file
        );
        return Ok(0);
    }

    // Parse MCPs from settings.json
    let mcps = match gemini_config::parse_gemini_mcps(&paths.settings_file) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to parse Gemini settings.json: {}", e);
            return Ok(0);
        }
    };

    let mut count = 0;

    for mcp in mcps {
        let source_path = paths.settings_file.to_string_lossy().to_string();

        // Check if already exists
        let existing_id: Option<i64> = db
            .conn()
            .query_row("SELECT id FROM mcps WHERE name = ?", [&mcp.name], |row| {
                row.get(0)
            })
            .ok();

        if let Some(id) = existing_id {
            // Update source_path if not already set
            db.conn().execute(
                "UPDATE mcps SET source_path = ? WHERE id = ? AND (source_path IS NULL OR source_path = '')",
                params![&source_path, id],
            )?;
        } else {
            let args_json = match &mcp.args {
                Some(args) if !args.is_empty() => Some(serde_json::to_string(args).unwrap()),
                _ => None,
            };
            let env_json = match &mcp.env {
                Some(env) if !env.is_empty() => Some(serde_json::to_string(env).unwrap()),
                _ => None,
            };
            let headers_json = match &mcp.headers {
                Some(headers) if !headers.is_empty() => {
                    Some(serde_json::to_string(headers).unwrap())
                }
                _ => None,
            };

            let result = db.conn().execute(
                "INSERT INTO mcps (name, type, command, args, url, headers, env, source, source_path)
                 VALUES (?, ?, ?, ?, ?, ?, ?, 'gemini', ?)",
                params![
                    mcp.name,
                    mcp.mcp_type,
                    mcp.command,
                    args_json,
                    mcp.url,
                    headers_json,
                    env_json,
                    source_path
                ],
            );

            if result.is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // =========================================================================
    // parse_frontmatter tests
    // =========================================================================

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = r#"---
name: test-skill
description: A test skill
type: command
---
This is the body content."#;

        let (fm, body) = parse_frontmatter(content);

        assert_eq!(fm.get("name"), Some(&"test-skill".to_string()));
        assert_eq!(fm.get("description"), Some(&"A test skill".to_string()));
        assert_eq!(fm.get("type"), Some(&"command".to_string()));
        assert_eq!(body, "This is the body content.");
    }

    #[test]
    fn test_parse_frontmatter_with_multiline_body() {
        let content = r#"---
name: test
---
Line 1
Line 2
Line 3"#;

        let (fm, body) = parse_frontmatter(content);

        assert_eq!(fm.get("name"), Some(&"test".to_string()));
        assert!(body.contains("Line 1"));
        assert!(body.contains("Line 2"));
        assert!(body.contains("Line 3"));
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "Just regular content without frontmatter.";

        let (fm, body) = parse_frontmatter(content);

        assert!(fm.is_empty());
        assert_eq!(body, "Just regular content without frontmatter.");
    }

    #[test]
    fn test_parse_frontmatter_only_opening() {
        let content = r#"---
name: test
No closing delimiter"#;

        let (fm, body) = parse_frontmatter(content);

        // Should return content as-is when no closing delimiter
        assert!(fm.is_empty());
        assert!(body.starts_with("---"));
    }

    #[test]
    fn test_parse_frontmatter_empty_values_skipped() {
        let content = r#"---
name: test
empty_key:
another: value
---
Body"#;

        let (fm, body) = parse_frontmatter(content);

        assert_eq!(fm.get("name"), Some(&"test".to_string()));
        assert_eq!(fm.get("another"), Some(&"value".to_string()));
        assert!(fm.get("empty_key").is_none());
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_parse_frontmatter_with_comma_separated_values() {
        let content = r#"---
tags: tag1, tag2, tag3
tools: Read, Write, Bash
---
Content"#;

        let (fm, _) = parse_frontmatter(content);

        assert_eq!(fm.get("tags"), Some(&"tag1, tag2, tag3".to_string()));
        assert_eq!(fm.get("tools"), Some(&"Read, Write, Bash".to_string()));
    }

    #[test]
    fn test_parse_frontmatter_colon_in_value() {
        let content = r#"---
url: https://example.com:8080
---
Body"#;

        let (fm, _) = parse_frontmatter(content);

        // Value after first colon should be preserved (including subsequent colons)
        assert_eq!(fm.get("url"), Some(&"https://example.com:8080".to_string()));
    }

    // =========================================================================
    // parse_skill_file tests
    // =========================================================================

    #[test]
    fn test_parse_skill_file_command_type() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("test-command.md");

        fs::write(
            &skill_path,
            r#"---
description: Test command skill
type: command
allowed-tools: Read, Write
argument-hint: <filename>
---
You are a helpful assistant."#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();

        assert_eq!(skill.name, "test-command");
        assert_eq!(skill.description, Some("Test command skill".to_string()));
        assert_eq!(skill.skill_type, "command");
        assert_eq!(skill.allowed_tools, Some("Read, Write".to_string()));
        assert_eq!(skill.argument_hint, Some("<filename>".to_string()));
        assert!(skill.content.contains("You are a helpful assistant."));
    }

    #[test]
    fn test_parse_skill_file_skill_type() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("auto-skill.md");

        fs::write(
            &skill_path,
            r#"---
description: Auto-invoked skill
type: skill
model: sonnet
disableModelInvocation: true
---
This skill is invoked by the model."#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();

        assert_eq!(skill.name, "auto-skill");
        assert_eq!(skill.skill_type, "skill");
        assert_eq!(skill.model, Some("sonnet".to_string()));
        assert!(skill.disable_model_invocation);
    }

    #[test]
    fn test_parse_skill_file_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("minimal.md");

        fs::write(&skill_path, "Just content, no frontmatter.").unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();

        assert_eq!(skill.name, "minimal");
        assert_eq!(skill.description, None);
        assert_eq!(skill.skill_type, "command"); // Default type
        assert_eq!(skill.allowed_tools, None);
        assert!(!skill.disable_model_invocation); // Default false
        assert!(skill.tags.is_empty());
    }

    #[test]
    fn test_parse_skill_file_with_tags() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("tagged.md");

        fs::write(
            &skill_path,
            r#"---
description: Tagged skill
tags: development, testing, automation
---
Skill content."#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();

        assert_eq!(skill.tags, vec!["development", "testing", "automation"]);
    }

    #[test]
    fn test_parse_skill_file_alternate_key_formats() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("alternate.md");

        // Test allowed_tools instead of allowed-tools
        fs::write(
            &skill_path,
            r#"---
allowed_tools: Bash
---
Content."#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();

        assert_eq!(skill.allowed_tools, Some("Bash".to_string()));
    }

    #[test]
    fn test_parse_skill_file_nonexistent() {
        let result = parse_skill_file(Path::new("/nonexistent/path/skill.md"));
        assert!(result.is_none());
    }

    // =========================================================================
    // parse_agent_file tests
    // =========================================================================

    #[test]
    fn test_parse_agent_file_full() {
        let temp_dir = TempDir::new().unwrap();
        let agent_path = temp_dir.path().join("code-reviewer.md");

        fs::write(
            &agent_path,
            r#"---
description: Reviews code for quality
tools: Read, Grep, Glob
model: opus
permissionMode: bypassPermissions
skills: lint, format
tags: review, code-quality
---
You are a code review expert."#,
        )
        .unwrap();

        let agent = parse_agent_file(&agent_path).unwrap();

        assert_eq!(agent.name, "code-reviewer");
        assert_eq!(agent.description, "Reviews code for quality");
        assert_eq!(agent.tools, vec!["Read", "Grep", "Glob"]);
        assert_eq!(agent.model, Some("opus".to_string()));
        assert_eq!(agent.permission_mode, Some("bypassPermissions".to_string()));
        assert_eq!(agent.skills, vec!["lint", "format"]);
        assert_eq!(agent.tags, vec!["review", "code-quality"]);
        assert!(agent.content.contains("code review expert"));
    }

    #[test]
    fn test_parse_agent_file_minimal() {
        let temp_dir = TempDir::new().unwrap();
        let agent_path = temp_dir.path().join("simple-agent.md");

        fs::write(&agent_path, "Just some agent instructions.").unwrap();

        let agent = parse_agent_file(&agent_path).unwrap();

        assert_eq!(agent.name, "simple-agent");
        assert_eq!(agent.description, "simple-agent"); // Falls back to filename
        assert!(agent.tools.is_empty());
        assert!(agent.model.is_none());
        assert!(agent.permission_mode.is_none());
        assert!(agent.skills.is_empty());
    }

    #[test]
    fn test_parse_agent_file_permission_mode_snake_case() {
        let temp_dir = TempDir::new().unwrap();
        let agent_path = temp_dir.path().join("snake.md");

        fs::write(
            &agent_path,
            r#"---
permission_mode: askUser
---
Content"#,
        )
        .unwrap();

        let agent = parse_agent_file(&agent_path).unwrap();

        assert_eq!(agent.permission_mode, Some("askUser".to_string()));
    }

    #[test]
    fn test_parse_agent_file_nonexistent() {
        let result = parse_agent_file(Path::new("/nonexistent/agent.md"));
        assert!(result.is_none());
    }

    // =========================================================================
    // parse_hooks_from_settings tests
    // =========================================================================

    #[test]
    fn test_parse_hooks_from_settings_command_hook() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    {
                        "matcher": "Write|Edit",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "npm run lint"
                            }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].event_type, "PostToolUse");
        assert_eq!(hooks[0].matcher, Some("Write|Edit".to_string()));
        assert_eq!(hooks[0].hook_type, "command");
        assert_eq!(hooks[0].command, Some("npm run lint".to_string()));
    }

    #[test]
    fn test_parse_hooks_from_settings_prompt_hook() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PreToolUse": [
                    {
                        "hooks": [
                            {
                                "type": "prompt",
                                "prompt": "Always verify before writing"
                            }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].hook_type, "prompt");
        assert_eq!(
            hooks[0].prompt,
            Some("Always verify before writing".to_string())
        );
    }

    #[test]
    fn test_parse_hooks_from_settings_with_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    {
                        "hooks": [
                            {
                                "type": "command",
                                "command": "slow-command",
                                "timeout": 30000
                            }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].timeout, Some(30000));
    }

    #[test]
    fn test_parse_hooks_from_settings_multiple_event_types() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PreToolUse": [
                    { "hooks": [{ "type": "command", "command": "pre-cmd" }] }
                ],
                "PostToolUse": [
                    { "hooks": [{ "type": "command", "command": "post-cmd" }] }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert_eq!(hooks.len(), 2);
        let event_types: Vec<_> = hooks.iter().map(|h| h.event_type.as_str()).collect();
        assert!(event_types.contains(&"PreToolUse"));
        assert!(event_types.contains(&"PostToolUse"));
    }

    #[test]
    fn test_parse_hooks_from_settings_empty_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(&settings_path, r#"{ "hooks": {} }"#).unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert!(hooks.is_empty());
    }

    #[test]
    fn test_parse_hooks_from_settings_no_hooks_key() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(&settings_path, r#"{ "other": "config" }"#).unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert!(hooks.is_empty());
    }

    #[test]
    fn test_parse_hooks_from_settings_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(&settings_path, "not valid json").unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert!(hooks.is_empty());
    }

    #[test]
    fn test_parse_hooks_from_settings_nonexistent_file() {
        let hooks = parse_hooks_from_settings(Path::new("/nonexistent/settings.json"));
        assert!(hooks.is_empty());
    }

    #[test]
    fn test_parse_hooks_from_settings_multiple_hooks_in_entry() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    {
                        "matcher": "Write",
                        "hooks": [
                            { "type": "command", "command": "lint" },
                            { "type": "command", "command": "test" }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert_eq!(hooks.len(), 2);
        let commands: Vec<_> = hooks.iter().filter_map(|h| h.command.as_ref()).collect();
        assert!(commands.contains(&&"lint".to_string()));
        assert!(commands.contains(&&"test".to_string()));
    }

    #[test]
    fn test_parse_hooks_from_settings_default_type() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        // No explicit type - should default to "command"
        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    { "hooks": [{ "command": "some-cmd" }] }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);

        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].hook_type, "command");
    }

    // =========================================================================
    // DB-backed scanner function tests
    // =========================================================================

    fn setup_test_db() -> crate::db::Database {
        crate::db::Database::in_memory().unwrap()
    }

    #[test]
    fn test_get_or_create_project_creates_new() {
        let db = setup_test_db();
        let id = get_or_create_project(&db, "my-project", "/tmp/my-project").unwrap();
        assert!(id > 0);

        // Verify it's in the database
        let name: String = db
            .conn()
            .query_row("SELECT name FROM projects WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "my-project");
    }

    #[test]
    fn test_get_or_create_project_returns_existing() {
        let db = setup_test_db();
        let id1 = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let id2 = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_get_or_create_mcp_creates_new() {
        let db = setup_test_db();
        let id = get_or_create_mcp(
            &db,
            "test-mcp",
            "stdio",
            Some("npx"),
            Some(&vec!["server".to_string()]),
            None,
            None,
            None,
            "/source",
        )
        .unwrap();
        assert!(id > 0);

        let name: String = db
            .conn()
            .query_row("SELECT name FROM mcps WHERE id = ?", [id], |row| row.get(0))
            .unwrap();
        assert_eq!(name, "test-mcp");
    }

    #[test]
    fn test_get_or_create_mcp_returns_existing() {
        let db = setup_test_db();
        let id1 = get_or_create_mcp(
            &db,
            "mcp1",
            "stdio",
            Some("cmd"),
            None,
            None,
            None,
            None,
            "/src1",
        )
        .unwrap();
        let id2 = get_or_create_mcp(
            &db,
            "mcp1",
            "stdio",
            Some("cmd2"),
            None,
            None,
            None,
            None,
            "/src2",
        )
        .unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_assign_mcp_to_project() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let mcp_id = get_or_create_mcp(
            &db,
            "mcp",
            "stdio",
            Some("cmd"),
            None,
            None,
            None,
            None,
            "/src",
        )
        .unwrap();

        assign_mcp_to_project(&db, proj_id, mcp_id, true).unwrap();

        let enabled: bool = db
            .conn()
            .query_row(
                "SELECT is_enabled FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
                params![proj_id, mcp_id],
                |row| row.get::<_, i32>(0).map(|v| v != 0),
            )
            .unwrap();
        assert!(enabled);
    }

    #[test]
    fn test_assign_mcp_to_project_updates_enabled_state() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let mcp_id = get_or_create_mcp(
            &db,
            "mcp",
            "stdio",
            Some("cmd"),
            None,
            None,
            None,
            None,
            "/src",
        )
        .unwrap();

        assign_mcp_to_project(&db, proj_id, mcp_id, true).unwrap();
        assign_mcp_to_project(&db, proj_id, mcp_id, false).unwrap();

        let enabled: bool = db
            .conn()
            .query_row(
                "SELECT is_enabled FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
                params![proj_id, mcp_id],
                |row| row.get::<_, i32>(0).map(|v| v != 0),
            )
            .unwrap();
        assert!(!enabled);
    }

    #[test]
    fn test_assign_mcp_to_project_idempotent() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let mcp_id = get_or_create_mcp(
            &db,
            "mcp",
            "stdio",
            Some("cmd"),
            None,
            None,
            None,
            None,
            "/src",
        )
        .unwrap();

        assign_mcp_to_project(&db, proj_id, mcp_id, true).unwrap();
        assign_mcp_to_project(&db, proj_id, mcp_id, true).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
                params![proj_id, mcp_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_or_create_skill() {
        let db = setup_test_db();
        let skill = ParsedSkill {
            name: "test-skill".to_string(),
            description: Some("A skill".to_string()),
            content: "Do the thing".to_string(),
            skill_type: "skill".to_string(),
            allowed_tools: Some("Read, Write".to_string()),
            argument_hint: None,
            model: Some("opus".to_string()),
            disable_model_invocation: false,
            tags: vec!["test".to_string()],
        };

        let (id, created) = get_or_create_skill(&db, &skill, "/src/skill").unwrap();
        assert!(id > 0);
        assert!(created);

        let (id2, created2) = get_or_create_skill(&db, &skill, "/src/skill").unwrap();
        assert_eq!(id, id2);
        assert!(!created2);
    }

    #[test]
    fn test_get_or_create_command() {
        let db = setup_test_db();
        let cmd = ParsedSkill {
            name: "test-cmd".to_string(),
            description: Some("A command".to_string()),
            content: "Run this".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };

        let (id, created) = get_or_create_command(&db, &cmd, "/src/cmd").unwrap();
        assert!(id > 0);
        assert!(created);

        let (id2, created2) = get_or_create_command(&db, &cmd, "/src/cmd").unwrap();
        assert_eq!(id, id2);
        assert!(!created2);
    }

    #[test]
    fn test_get_or_create_agent() {
        let db = setup_test_db();
        let agent = ParsedAgent {
            name: "test-agent".to_string(),
            description: "An agent".to_string(),
            content: "Agent instructions".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string()],
            model: Some("opus".to_string()),
            permission_mode: Some("bypassPermissions".to_string()),
            skills: vec!["lint".to_string()],
            tags: vec!["dev".to_string()],
        };

        let id1 = get_or_create_agent(&db, &agent, "/src/agent").unwrap();
        assert!(id1 > 0);

        let id2 = get_or_create_agent(&db, &agent, "/src/agent").unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_assign_skill_to_project() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let skill = ParsedSkill {
            name: "sk".to_string(),
            description: None,
            content: "content".to_string(),
            skill_type: "skill".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };
        let (skill_id, _) = get_or_create_skill(&db, &skill, "/src").unwrap();

        assign_skill_to_project(&db, proj_id, skill_id).unwrap();
        // Second call is idempotent
        assign_skill_to_project(&db, proj_id, skill_id).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_skills WHERE project_id = ? AND skill_id = ?",
                params![proj_id, skill_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_assign_command_to_project() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let cmd = ParsedSkill {
            name: "cmd".to_string(),
            description: None,
            content: "content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };
        let (cmd_id, _) = get_or_create_command(&db, &cmd, "/src").unwrap();

        assign_command_to_project(&db, proj_id, cmd_id).unwrap();
        assign_command_to_project(&db, proj_id, cmd_id).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_commands WHERE project_id = ? AND command_id = ?",
                params![proj_id, cmd_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_assign_agent_to_project() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let agent = ParsedAgent {
            name: "ag".to_string(),
            description: "desc".to_string(),
            content: "content".to_string(),
            tools: vec![],
            model: None,
            permission_mode: None,
            skills: vec![],
            tags: vec![],
        };
        let agent_id = get_or_create_agent(&db, &agent, "/src").unwrap();

        assign_agent_to_project(&db, proj_id, agent_id).unwrap();
        assign_agent_to_project(&db, proj_id, agent_id).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_subagents WHERE project_id = ? AND subagent_id = ?",
                params![proj_id, agent_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_or_create_hook() {
        let db = setup_test_db();
        let hook = ParsedHook {
            name: "test-hook".to_string(),
            description: Some("A hook".to_string()),
            event_type: "PostToolUse".to_string(),
            matcher: Some("Write".to_string()),
            hook_type: "command".to_string(),
            command: Some("npm run lint".to_string()),
            prompt: None,
            timeout: Some(5000),
        };

        let id1 = get_or_create_hook(&db, &hook).unwrap();
        assert!(id1 > 0);

        let id2 = get_or_create_hook(&db, &hook).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_assign_hook_to_project() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();
        let hook = ParsedHook {
            name: "hook1".to_string(),
            description: None,
            event_type: "PreToolUse".to_string(),
            matcher: None,
            hook_type: "command".to_string(),
            command: Some("echo hi".to_string()),
            prompt: None,
            timeout: None,
        };
        let hook_id = get_or_create_hook(&db, &hook).unwrap();

        assign_hook_to_project(&db, proj_id, hook_id).unwrap();
        assign_hook_to_project(&db, proj_id, hook_id).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_hooks WHERE project_id = ? AND hook_id = ?",
                params![proj_id, hook_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_skill_files() {
        let db = setup_test_db();
        let skill = ParsedSkill {
            name: "skill-with-files".to_string(),
            description: None,
            content: "content".to_string(),
            skill_type: "skill".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };
        let (skill_id, _) = get_or_create_skill(&db, &skill, "/src").unwrap();

        let files = vec![
            ParsedSkillFile {
                file_type: "reference".to_string(),
                name: "ref.md".to_string(),
                content: "reference content".to_string(),
            },
            ParsedSkillFile {
                file_type: "asset".to_string(),
                name: "data.json".to_string(),
                content: r#"{"key":"value"}"#.to_string(),
            },
        ];

        let count = insert_skill_files(&db, skill_id, &files).unwrap();
        assert_eq!(count, 2);

        // Inserting same files again should not create duplicates
        let count2 = insert_skill_files(&db, skill_id, &files).unwrap();
        assert_eq!(count2, 0);
    }

    #[test]
    fn test_scan_project_commands_from_dir() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        fs::write(
            commands_dir.join("deploy.md"),
            r#"---
description: Deploy the app
allowed-tools: Bash
---
Deploy instructions here."#,
        )
        .unwrap();

        fs::write(commands_dir.join("test.md"), "Run all tests.").unwrap();

        // Non-md file should be ignored
        fs::write(commands_dir.join("readme.txt"), "ignore me").unwrap();

        let count = scan_project_commands(&db, proj_id, &commands_dir).unwrap();
        assert_eq!(count, 2);

        // Verify commands are assigned to project
        let assigned: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_commands WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(assigned, 2);
    }

    #[test]
    fn test_scan_project_skills_from_dir() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir(&skills_dir).unwrap();

        // Create a skill directory with SKILL.md
        let skill1_dir = skills_dir.join("my-skill");
        fs::create_dir(&skill1_dir).unwrap();
        fs::write(
            skill1_dir.join("SKILL.md"),
            r#"---
description: My skill
---
Skill content."#,
        )
        .unwrap();

        // Create references subdir
        let refs_dir = skill1_dir.join("references");
        fs::create_dir(&refs_dir).unwrap();
        fs::write(refs_dir.join("ref.md"), "Reference content").unwrap();

        let count = scan_project_skills(&db, proj_id, &skills_dir).unwrap();
        assert_eq!(count, 1);

        // Verify skill is assigned to project
        let assigned: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_skills WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(assigned, 1);

        // Verify skill file was created
        let file_count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM skill_files", [], |row| row.get(0))
            .unwrap();
        assert_eq!(file_count, 1);
    }

    #[test]
    fn test_scan_project_agents_from_dir() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");
        fs::create_dir(&agents_dir).unwrap();

        fs::write(
            agents_dir.join("reviewer.md"),
            r#"---
description: Code reviewer
tools: Read, Grep
model: opus
---
You review code."#,
        )
        .unwrap();

        let count = scan_project_agents(&db, proj_id, &agents_dir).unwrap();
        assert_eq!(count, 1);

        let assigned: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_subagents WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(assigned, 1);
    }

    #[test]
    fn test_scan_project_hooks_from_settings() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    {
                        "matcher": "Write",
                        "hooks": [
                            { "type": "command", "command": "lint" }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let count = scan_project_hooks(&db, proj_id, &settings_path).unwrap();
        assert_eq!(count, 1);

        let assigned: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_hooks WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(assigned, 1);
    }

    #[test]
    fn test_parse_agent_skill_dir_with_subdirs() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("my-skill");
        fs::create_dir(&skill_dir).unwrap();

        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
description: Complex skill
allowed-tools: Bash
model: sonnet
---
Skill body."#,
        )
        .unwrap();

        // Create references
        let refs = skill_dir.join("references");
        fs::create_dir(&refs).unwrap();
        fs::write(refs.join("guide.md"), "Guide content").unwrap();

        // Create assets
        let assets = skill_dir.join("assets");
        fs::create_dir(&assets).unwrap();
        fs::write(assets.join("config.json"), r#"{"key":"val"}"#).unwrap();

        // Create scripts
        let scripts = skill_dir.join("scripts");
        fs::create_dir(&scripts).unwrap();
        fs::write(scripts.join("setup.sh"), "#!/bin/bash\necho hi").unwrap();

        let (skill, files) = parse_agent_skill_dir(&skill_dir).unwrap();

        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, Some("Complex skill".to_string()));
        assert_eq!(skill.skill_type, "skill"); // Agent skills are always "skill"
        assert_eq!(skill.allowed_tools, Some("Bash".to_string()));
        assert_eq!(skill.model, Some("sonnet".to_string()));

        assert_eq!(files.len(), 3);
        let file_types: Vec<&str> = files.iter().map(|f| f.file_type.as_str()).collect();
        assert!(file_types.contains(&"reference"));
        assert!(file_types.contains(&"asset"));
        assert!(file_types.contains(&"script"));
    }

    #[test]
    fn test_parse_agent_skill_dir_no_skill_md() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("no-skill");
        fs::create_dir(&skill_dir).unwrap();
        // No SKILL.md file
        fs::write(skill_dir.join("README.md"), "Not a skill").unwrap();

        let result = parse_agent_skill_dir(&skill_dir);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_or_create_mcp_with_env_and_headers() {
        let db = setup_test_db();
        let mut env = std::collections::HashMap::new();
        env.insert("API_KEY".to_string(), "secret".to_string());
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let id = get_or_create_mcp(
            &db,
            "http-mcp",
            "http",
            None,
            None,
            Some("https://example.com/mcp"),
            Some(&headers),
            Some(&env),
            "/source",
        )
        .unwrap();

        let (stored_env, stored_headers): (Option<String>, Option<String>) = db
            .conn()
            .query_row("SELECT env, headers FROM mcps WHERE id = ?", [id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();

        assert!(stored_env.unwrap().contains("API_KEY"));
        assert!(stored_headers.unwrap().contains("Authorization"));
    }

    #[test]
    fn test_get_or_create_mcp_updates_source_path() {
        let db = setup_test_db();
        // Create with empty source_path
        db.conn()
            .execute(
                "INSERT INTO mcps (name, type, source_path) VALUES ('mcp-empty', 'stdio', '')",
                [],
            )
            .unwrap();

        let id = get_or_create_mcp(
            &db,
            "mcp-empty",
            "stdio",
            None,
            None,
            None,
            None,
            None,
            "/new/path",
        )
        .unwrap();

        let source_path: String = db
            .conn()
            .query_row("SELECT source_path FROM mcps WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(source_path, "/new/path");
    }

    // =========================================================================
    // Additional coverage: parse_skill_file edge cases
    // =========================================================================

    #[test]
    fn test_parse_skill_file_with_allowedTools_camelCase() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("camel.md");
        fs::write(
            &skill_path,
            r#"---
allowedTools: Read, Write, Bash
argumentHint: <file>
---
Content."#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();
        assert_eq!(skill.allowed_tools, Some("Read, Write, Bash".to_string()));
        assert_eq!(skill.argument_hint, Some("<file>".to_string()));
    }

    #[test]
    fn test_parse_skill_file_disable_model_invocation_with_dash() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("dmi.md");
        fs::write(
            &skill_path,
            r#"---
disable-model-invocation: true
---
Body"#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();
        assert!(skill.disable_model_invocation);
    }

    #[test]
    fn test_parse_skill_file_disable_model_invocation_value_1() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("dmi1.md");
        fs::write(
            &skill_path,
            r#"---
disable-model-invocation: 1
---
Body"#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();
        assert!(skill.disable_model_invocation);
    }

    #[test]
    fn test_parse_skill_file_disable_model_invocation_false_value() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("dmi_false.md");
        fs::write(
            &skill_path,
            r#"---
disable-model-invocation: false
---
Body"#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();
        assert!(!skill.disable_model_invocation);
    }

    #[test]
    fn test_parse_skill_file_with_model() {
        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path().join("model-skill.md");
        fs::write(
            &skill_path,
            r#"---
model: sonnet
description: A sonnet skill
---
Instructions"#,
        )
        .unwrap();

        let skill = parse_skill_file(&skill_path).unwrap();
        assert_eq!(skill.model, Some("sonnet".to_string()));
        assert_eq!(skill.description, Some("A sonnet skill".to_string()));
    }

    // =========================================================================
    // Additional coverage: parse_agent_file edge cases
    // =========================================================================

    #[test]
    fn test_parse_agent_file_empty_tools_and_skills() {
        let temp_dir = TempDir::new().unwrap();
        let agent_path = temp_dir.path().join("empty-lists.md");
        fs::write(
            &agent_path,
            r#"---
description: Agent with no tools
---
Simple agent."#,
        )
        .unwrap();

        let agent = parse_agent_file(&agent_path).unwrap();
        assert!(agent.tools.is_empty());
        assert!(agent.skills.is_empty());
        assert!(agent.tags.is_empty());
    }

    // =========================================================================
    // Additional coverage: parse_agent_skill_dir edge cases
    // =========================================================================

    #[test]
    fn test_parse_agent_skill_dir_empty_subdirs() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("empty-subdirs");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
description: Skill with empty subdirs
---
Content"#,
        )
        .unwrap();

        // Create empty subdirs
        fs::create_dir(skill_dir.join("references")).unwrap();
        fs::create_dir(skill_dir.join("assets")).unwrap();
        fs::create_dir(skill_dir.join("scripts")).unwrap();

        let (skill, files) = parse_agent_skill_dir(&skill_dir).unwrap();
        assert_eq!(skill.name, "empty-subdirs");
        assert!(files.is_empty()); // No files in empty subdirs
    }

    #[test]
    fn test_parse_agent_skill_dir_with_disable_model_invocation() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("dmi-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
disableModelInvocation: true
tags: test, auto
---
Body"#,
        )
        .unwrap();

        let (skill, _) = parse_agent_skill_dir(&skill_dir).unwrap();
        assert!(skill.disable_model_invocation);
        assert_eq!(skill.tags, vec!["test", "auto"]);
        assert_eq!(skill.skill_type, "skill"); // Always "skill" for agent skills
    }

    #[test]
    fn test_parse_agent_skill_dir_skips_dirs_in_subdirs() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("nested-dirs");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "Simple skill").unwrap();

        let refs = skill_dir.join("references");
        fs::create_dir(&refs).unwrap();
        // Create a file and a subdirectory
        fs::write(refs.join("valid.md"), "ref content").unwrap();
        fs::create_dir(refs.join("subdir")).unwrap();

        let (_, files) = parse_agent_skill_dir(&skill_dir).unwrap();
        // Only the file should be included, not the directory
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "valid.md");
        assert_eq!(files[0].file_type, "reference");
    }

    // =========================================================================
    // Additional coverage: parse_hooks_from_settings name generation
    // =========================================================================

    #[test]
    fn test_parse_hooks_name_with_matcher() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    {
                        "matcher": "Write|Edit",
                        "hooks": [
                            { "type": "command", "command": "lint" }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);
        assert_eq!(hooks.len(), 1);
        // Name should use matcher with pipes replaced by dashes
        assert_eq!(hooks[0].name, "posttooluse-Write-Edit-0");
    }

    #[test]
    fn test_parse_hooks_name_without_matcher() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PreToolUse": [
                    {
                        "hooks": [
                            { "type": "command", "command": "check" }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);
        assert_eq!(hooks.len(), 1);
        // Name should use event_type-idx-inner_idx format
        assert_eq!(hooks[0].name, "pretooluse-0-0");
    }

    #[test]
    fn test_parse_hooks_description_with_matcher() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            { "type": "command", "command": "check" }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);
        assert_eq!(
            hooks[0].description,
            Some("PostToolUse hook for Bash".to_string())
        );
    }

    #[test]
    fn test_parse_hooks_description_without_matcher() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "Notification": [
                    {
                        "hooks": [
                            { "type": "command", "command": "notify" }
                        ]
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let hooks = parse_hooks_from_settings(&settings_path);
        assert_eq!(hooks[0].description, Some("Notification hook".to_string()));
    }

    // =========================================================================
    // Additional DB coverage: get_or_create_project path normalization
    // =========================================================================

    #[test]
    fn test_get_or_create_project_finds_by_normalized_path() {
        let db = setup_test_db();
        // Create with a tilde path
        let id1 = get_or_create_project(&db, "proj", "~/projects/myproj").unwrap();
        // Querying with the same path should find it
        let id2 = get_or_create_project(&db, "proj", "~/projects/myproj").unwrap();
        assert_eq!(id1, id2);
    }

    // =========================================================================
    // Additional coverage: get_or_create_skill with empty tags
    // =========================================================================

    #[test]
    fn test_get_or_create_skill_empty_tags() {
        let db = setup_test_db();
        let skill = ParsedSkill {
            name: "no-tags-skill".to_string(),
            description: None,
            content: "content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };
        let (id, created) = get_or_create_skill(&db, &skill, "/src").unwrap();
        assert!(id > 0);
        assert!(created);

        // Verify tags is NULL
        let tags: Option<String> = db
            .conn()
            .query_row("SELECT tags FROM skills WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(tags.is_none());
    }

    // =========================================================================
    // Additional coverage: get_or_create_agent with empty lists
    // =========================================================================

    #[test]
    fn test_get_or_create_agent_empty_tools_skills_tags() {
        let db = setup_test_db();
        let agent = ParsedAgent {
            name: "empty-agent".to_string(),
            description: "desc".to_string(),
            content: "content".to_string(),
            tools: vec![],
            model: None,
            permission_mode: None,
            skills: vec![],
            tags: vec![],
        };
        let id = get_or_create_agent(&db, &agent, "/src").unwrap();
        assert!(id > 0);

        // Verify all JSON fields are NULL
        let (tools, skills, tags): (Option<String>, Option<String>, Option<String>) = db
            .conn()
            .query_row(
                "SELECT tools, skills, tags FROM subagents WHERE id = ?",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert!(tools.is_none());
        assert!(skills.is_none());
        assert!(tags.is_none());
    }

    // =========================================================================
    // Additional coverage: get_or_create_command with tags
    // =========================================================================

    #[test]
    fn test_get_or_create_command_with_tags() {
        let db = setup_test_db();
        let cmd = ParsedSkill {
            name: "tagged-cmd".to_string(),
            description: Some("Cmd with tags".to_string()),
            content: "run it".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: Some("Bash".to_string()),
            argument_hint: Some("<arg>".to_string()),
            model: Some("opus".to_string()),
            disable_model_invocation: false,
            tags: vec!["deploy".to_string(), "ci".to_string()],
        };

        let (id, created) = get_or_create_command(&db, &cmd, "/src/cmd").unwrap();
        assert!(id > 0);
        assert!(created);

        let tags: Option<String> = db
            .conn()
            .query_row("SELECT tags FROM commands WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(tags.is_some());
        assert!(tags.unwrap().contains("deploy"));
    }

    // =========================================================================
    // Additional coverage: insert_skill_files empty input
    // =========================================================================

    #[test]
    fn test_insert_skill_files_empty() {
        let db = setup_test_db();
        let skill = ParsedSkill {
            name: "empty-files-skill".to_string(),
            description: None,
            content: "content".to_string(),
            skill_type: "skill".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };
        let (skill_id, _) = get_or_create_skill(&db, &skill, "/src").unwrap();

        let files: Vec<ParsedSkillFile> = vec![];
        let count = insert_skill_files(&db, skill_id, &files).unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Additional coverage: scan_project_commands ignores non-md
    // =========================================================================

    #[test]
    fn test_scan_project_skills_ignores_files() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let skills_dir = temp_dir.path().join("skills");
        fs::create_dir(&skills_dir).unwrap();

        // Create a regular file (not a directory) - should be ignored
        fs::write(skills_dir.join("not-a-dir.md"), "some content").unwrap();

        let count = scan_project_skills(&db, proj_id, &skills_dir).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_scan_project_agents_ignores_non_md() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let agents_dir = temp_dir.path().join("agents");
        fs::create_dir(&agents_dir).unwrap();

        // Create a non-md file
        fs::write(agents_dir.join("readme.txt"), "not an agent").unwrap();

        let count = scan_project_agents(&db, proj_id, &agents_dir).unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Additional coverage: parse_frontmatter empty content
    // =========================================================================

    #[test]
    fn test_parse_frontmatter_empty_string() {
        let (fm, body) = parse_frontmatter("");
        assert!(fm.is_empty());
        assert_eq!(body, "");
    }

    #[test]
    fn test_parse_frontmatter_just_dashes() {
        let (fm, body) = parse_frontmatter("---");
        assert!(fm.is_empty());
        assert_eq!(body, "---");
    }

    // =========================================================================
    // Additional coverage: scan_project_hooks with empty hooks
    // =========================================================================

    #[test]
    fn test_scan_project_hooks_no_hooks_in_file() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        fs::write(&settings_path, r#"{"other": "stuff"}"#).unwrap();

        let count = scan_project_hooks(&db, proj_id, &settings_path).unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Additional DB coverage: get_or_create_command source_path update
    // =========================================================================

    #[test]
    fn test_get_or_create_command_updates_source_path() {
        let db = setup_test_db();
        // Create with empty source_path
        db.conn()
            .execute(
                "INSERT INTO commands (name, content, source_path) VALUES ('cmd-empty', 'content', '')",
                [],
            )
            .unwrap();

        let cmd = ParsedSkill {
            name: "cmd-empty".to_string(),
            description: None,
            content: "updated content".to_string(),
            skill_type: "command".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };

        let (id, created) = get_or_create_command(&db, &cmd, "/new/source/path").unwrap();
        assert!(id > 0);
        assert!(!created); // existing

        let source_path: String = db
            .conn()
            .query_row(
                "SELECT source_path FROM commands WHERE id = ?",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(source_path, "/new/source/path");
    }

    #[test]
    fn test_get_or_create_skill_updates_source_path() {
        let db = setup_test_db();
        // Create with empty source_path
        db.conn()
            .execute(
                "INSERT INTO skills (name, content, source_path) VALUES ('skill-empty', 'content', '')",
                [],
            )
            .unwrap();

        let skill = ParsedSkill {
            name: "skill-empty".to_string(),
            description: None,
            content: "content".to_string(),
            skill_type: "skill".to_string(),
            allowed_tools: None,
            argument_hint: None,
            model: None,
            disable_model_invocation: false,
            tags: vec![],
        };

        let (id, created) = get_or_create_skill(&db, &skill, "/new/skill/path").unwrap();
        assert!(id > 0);
        assert!(!created);

        let source_path: String = db
            .conn()
            .query_row("SELECT source_path FROM skills WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(source_path, "/new/skill/path");
    }

    #[test]
    fn test_get_or_create_agent_updates_source_path() {
        let db = setup_test_db();
        // Create with empty source_path
        db.conn()
            .execute(
                "INSERT INTO subagents (name, description, content, source_path) VALUES ('agent-empty', 'desc', 'content', '')",
                [],
            )
            .unwrap();

        let agent = ParsedAgent {
            name: "agent-empty".to_string(),
            description: "desc".to_string(),
            content: "content".to_string(),
            tools: vec![],
            model: None,
            permission_mode: None,
            skills: vec![],
            tags: vec![],
        };

        let id = get_or_create_agent(&db, &agent, "/new/agent/path").unwrap();
        assert!(id > 0);

        let source_path: String = db
            .conn()
            .query_row(
                "SELECT source_path FROM subagents WHERE id = ?",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(source_path, "/new/agent/path");
    }

    // =========================================================================
    // Additional coverage: scan_project_hooks idempotent
    // =========================================================================

    #[test]
    fn test_scan_project_hooks_idempotent() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        fs::write(
            &settings_path,
            r#"{
            "hooks": {
                "PostToolUse": [
                    { "hooks": [{ "type": "command", "command": "lint" }] }
                ]
            }
        }"#,
        )
        .unwrap();

        let count1 = scan_project_hooks(&db, proj_id, &settings_path).unwrap();
        assert_eq!(count1, 1);

        // Second call should not duplicate
        let count2 = scan_project_hooks(&db, proj_id, &settings_path).unwrap();
        assert_eq!(count2, 1);

        let total_hooks: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_hooks WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total_hooks, 1);
    }

    // =========================================================================
    // Additional coverage: parse_agent_skill_dir frontmatter variations
    // =========================================================================

    #[test]
    fn test_parse_agent_skill_dir_with_argument_hint() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("hint-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
description: Skill with hint
argument-hint: <filename>
allowed_tools: Read, Write
---
Body content."#,
        )
        .unwrap();

        let (skill, _) = parse_agent_skill_dir(&skill_dir).unwrap();
        assert_eq!(skill.argument_hint, Some("<filename>".to_string()));
        assert_eq!(skill.allowed_tools, Some("Read, Write".to_string()));
    }

    // =========================================================================
    // Additional coverage: get_or_create_agent with all fields
    // =========================================================================

    #[test]
    fn test_get_or_create_agent_with_all_fields() {
        let db = setup_test_db();
        let agent = ParsedAgent {
            name: "full-agent".to_string(),
            description: "Full description".to_string(),
            content: "Detailed instructions".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()],
            model: Some("opus".to_string()),
            permission_mode: Some("bypassPermissions".to_string()),
            skills: vec!["lint".to_string(), "format".to_string()],
            tags: vec!["dev".to_string(), "ci".to_string()],
        };

        let id = get_or_create_agent(&db, &agent, "/full/path").unwrap();
        assert!(id > 0);

        let (tools, skills, tags): (Option<String>, Option<String>, Option<String>) = db
            .conn()
            .query_row(
                "SELECT tools, skills, tags FROM subagents WHERE id = ?",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert!(tools.unwrap().contains("Read"));
        assert!(skills.unwrap().contains("lint"));
        assert!(tags.unwrap().contains("dev"));
    }

    // =========================================================================
    // Additional coverage: scan_project_commands idempotent
    // =========================================================================

    #[test]
    fn test_scan_project_commands_idempotent() {
        let db = setup_test_db();
        let proj_id = get_or_create_project(&db, "proj", "/tmp/proj").unwrap();

        let temp_dir = TempDir::new().unwrap();
        let commands_dir = temp_dir.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();
        fs::write(commands_dir.join("cmd.md"), "Command content.").unwrap();

        let count1 = scan_project_commands(&db, proj_id, &commands_dir).unwrap();
        assert_eq!(count1, 1);

        // Second call should still process (assigns to project again)
        let count2 = scan_project_commands(&db, proj_id, &commands_dir).unwrap();
        assert_eq!(count2, 1);

        // But project_commands should not have duplicates
        let assigned: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_commands WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(assigned, 1);
    }
}
