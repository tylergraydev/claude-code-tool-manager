use crate::db::Database;
use crate::services::claude_json;
use crate::services::config_parser;
use crate::services::opencode_config;
use crate::utils::opencode_paths::get_opencode_paths;
use crate::utils::paths::{get_claude_paths, normalize_path};
use anyhow::Result;
use rusqlite::params;
use std::collections::HashSet;
use std::path::Path;
use tauri::Manager;
use walkdir::WalkDir;

pub async fn run_startup_scan(app: &tauri::AppHandle) -> Result<()> {
    let db = app.state::<std::sync::Mutex<Database>>();
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

    // Scan global command skills from ~/.claude/commands/
    let skill_count = scan_global_skills(&db)?;
    log::info!(
        "Found {} command skills from ~/.claude/commands/",
        skill_count
    );

    // Scan global agent skills from ~/.claude/skills/
    let agent_skill_count = scan_global_agent_skills(&db)?;
    log::info!(
        "Found {} agent skills from ~/.claude/skills/",
        agent_skill_count
    );

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

        // Scan project-level command skills from .claude/commands/
        let project_commands_dir = Path::new(&path_to_check).join(".claude").join("commands");
        if project_commands_dir.exists() {
            scan_project_skills(db, project_id, &project_commands_dir)?;
        }

        // Scan project-level agent skills from .claude/skills/
        let project_skills_dir = Path::new(&path_to_check).join(".claude").join("skills");
        if project_skills_dir.exists() {
            scan_project_agent_skills(db, project_id, &project_skills_dir)?;
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
                            // Check if already exists
                            let exists: bool = db
                                .conn()
                                .query_row("SELECT 1 FROM mcps WHERE name = ?", [&mcp.name], |_| {
                                    Ok(true)
                                })
                                .unwrap_or(false);

                            if !exists {
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
                                        entry.path().to_string_lossy().to_string()
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

/// Scan global skills from ~/.claude/commands/
pub fn scan_global_skills(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let mut count = 0;

    if paths.commands_dir.exists() {
        for entry in std::fs::read_dir(&paths.commands_dir)? {
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
                            "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, model, disable_model_invocation, tags, source)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
                            params![
                                skill.name,
                                skill.description,
                                skill.content,
                                skill.skill_type,
                                skill.allowed_tools,
                                skill.argument_hint,
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
    }

    Ok(count)
}

/// Scan global agent skills from ~/.claude/skills/ directories
pub fn scan_global_agent_skills(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let mut count = 0;

    if paths.skills_dir.exists() {
        for entry in std::fs::read_dir(&paths.skills_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process directories (each skill is a directory with SKILL.md)
            if path.is_dir() {
                if let Some((skill, files)) = parse_agent_skill_dir(&path) {
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
                            "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, model, disable_model_invocation, tags, source)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
                            params![
                                skill.name,
                                skill.description,
                                skill.content,
                                skill.skill_type,
                                skill.allowed_tools,
                                skill.argument_hint,
                                skill.model,
                                skill.disable_model_invocation,
                                tags_json
                            ],
                        );

                        if result.is_ok() {
                            let skill_id = db.conn().last_insert_rowid();
                            // Insert associated skill files
                            if !files.is_empty() {
                                let _ = insert_skill_files(db, skill_id, &files);
                            }
                            count += 1;
                        }
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
                    // Check if already exists
                    let exists: bool = db
                        .conn()
                        .query_row(
                            "SELECT 1 FROM subagents WHERE name = ?",
                            [&agent.name],
                            |_| Ok(true),
                        )
                        .unwrap_or(false);

                    if !exists {
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
                            "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
                            params![
                                agent.name,
                                agent.description,
                                agent.content,
                                tools_json,
                                agent.model,
                                agent.permission_mode,
                                skills_json,
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

/// Scan project-level command skills and assign to project
fn scan_project_skills(db: &Database, project_id: i64, commands_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(commands_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .md files
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(skill) = parse_skill_file(&path) {
                // Get or create the skill in the library
                let (skill_id, _) = get_or_create_skill(db, &skill)?;

                // Assign skill to project if not already assigned
                assign_skill_to_project(db, project_id, skill_id)?;

                count += 1;
            }
        }
    }

    Ok(count)
}

/// Scan project-level agent skills from .claude/skills/ directories
fn scan_project_agent_skills(db: &Database, project_id: i64, skills_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process directories (each skill is a directory with SKILL.md)
        if path.is_dir() {
            if let Some((skill, files)) = parse_agent_skill_dir(&path) {
                // Get or create the skill in the library
                let (skill_id, was_created) = get_or_create_skill(db, &skill)?;

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
                let agent_id = get_or_create_agent(db, &agent)?;

                // Assign agent to project if not already assigned
                assign_agent_to_project(db, project_id, agent_id)?;

                count += 1;
            }
        }
    }

    Ok(count)
}

/// Get or create a skill in the database, returning (skill_id, was_created)
fn get_or_create_skill(db: &Database, skill: &ParsedSkill) -> Result<(i64, bool)> {
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
        return Ok((id, false));
    }

    // Create new skill
    let tags_json = if skill.tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&skill.tags).unwrap())
    };

    db.conn().execute(
        "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, model, disable_model_invocation, tags, source)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
        params![
            skill.name,
            skill.description,
            skill.content,
            skill.skill_type,
            skill.allowed_tools,
            skill.argument_hint,
            skill.model,
            skill.disable_model_invocation,
            tags_json
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
fn get_or_create_agent(db: &Database, agent: &ParsedAgent) -> Result<i64> {
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
        "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
        params![
            agent.name,
            agent.description,
            agent.content,
            tools_json,
            agent.model,
            agent.permission_mode,
            skills_json,
            tags_json
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

        // Check if already exists
        let exists: bool = db
            .conn()
            .query_row("SELECT 1 FROM mcps WHERE name = ?", [&mcp.name], |_| {
                Ok(true)
            })
            .unwrap_or(false);

        if !exists {
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
                    paths.config_file.to_string_lossy().to_string()
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
                        "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, model, disable_model_invocation, tags, source)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'opencode')",
                        params![
                            skill.name,
                            skill.description,
                            skill.content,
                            skill.skill_type,
                            skill.allowed_tools,
                            skill.argument_hint,
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
                // Check if already exists
                let exists: bool = db
                    .conn()
                    .query_row(
                        "SELECT 1 FROM subagents WHERE name = ?",
                        [&agent.name],
                        |_| Ok(true),
                    )
                    .unwrap_or(false);

                if !exists {
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
                        "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'opencode')",
                        params![
                            agent.name,
                            agent.description,
                            agent.content,
                            tools_json,
                            agent.model,
                            agent.permission_mode,
                            skills_json,
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

/// Scan OpenCode project configs and import their MCPs, commands, and agents
/// Called for each project detected with .opencode/ directory
pub fn scan_opencode_project(db: &Database, project_path: &Path) -> Result<(usize, usize, usize)> {
    let opencode_dir = project_path.join(".opencode");
    if !opencode_dir.exists() {
        return Ok((0, 0, 0));
    }

    let mut mcp_count = 0;
    let mut command_count = 0;
    let mut agent_count = 0;

    // Get or create project
    let project_name = project_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| project_path.to_string_lossy().to_string());

    let project_id =
        get_or_create_opencode_project(db, &project_name, &project_path.to_string_lossy())?;

    // Scan opencode.json in project root for MCPs
    let opencode_json = project_path.join("opencode.json");
    if opencode_json.exists() {
        if let Ok(mcps) = opencode_config::parse_opencode_mcps(&opencode_json) {
            for mcp in mcps {
                let mcp_type = match mcp.mcp_type.as_str() {
                    "local" => "stdio",
                    "remote" => "http",
                    other => other,
                };

                let mcp_id = get_or_create_mcp(
                    db,
                    &mcp.name,
                    mcp_type,
                    mcp.command.as_deref(),
                    mcp.args.as_ref(),
                    mcp.url.as_deref(),
                    mcp.headers.as_ref(),
                    mcp.env.as_ref(),
                    &project_path.to_string_lossy(),
                )?;

                assign_mcp_to_project(db, project_id, mcp_id, true)?;
                mcp_count += 1;
            }
        }
    }

    // Scan .opencode/command/ for commands
    let command_dir = opencode_dir.join("command");
    if command_dir.exists() {
        command_count = scan_opencode_project_commands(db, project_id, &command_dir)?;
    }

    // Scan .opencode/agent/ for agents
    let agent_dir = opencode_dir.join("agent");
    if agent_dir.exists() {
        agent_count = scan_opencode_project_agents(db, project_id, &agent_dir)?;
    }

    Ok((mcp_count, command_count, agent_count))
}

/// Get or create an OpenCode project in the database
fn get_or_create_opencode_project(db: &Database, name: &str, path: &str) -> Result<i64> {
    let normalized = normalize_path(path);

    // Try to find existing project by path
    let existing_id: Option<i64> = db
        .conn()
        .query_row(
            "SELECT id FROM projects WHERE path = ? OR path = ?",
            params![path, normalized],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        // Update editor_type to opencode if it exists
        let _ = db.conn().execute(
            "UPDATE projects SET editor_type = 'opencode' WHERE id = ?",
            [id],
        );
        return Ok(id);
    }

    // Create new project with editor_type = 'opencode'
    db.conn().execute(
        "INSERT INTO projects (name, path, has_mcp_file, has_settings_file, editor_type) VALUES (?, ?, 0, 0, 'opencode')",
        params![name, normalized],
    )?;

    Ok(db.conn().last_insert_rowid())
}

/// Scan OpenCode project commands and assign to project
fn scan_opencode_project_commands(
    db: &Database,
    project_id: i64,
    command_dir: &Path,
) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(command_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(skill) = parse_skill_file(&path) {
                let (skill_id, _) = get_or_create_skill(db, &skill)?;
                assign_skill_to_project(db, project_id, skill_id)?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Scan OpenCode project agents and assign to project
fn scan_opencode_project_agents(db: &Database, project_id: i64, agent_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(agent_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(agent) = parse_agent_file(&path) {
                let agent_id = get_or_create_agent(db, &agent)?;
                assign_agent_to_project(db, project_id, agent_id)?;
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
}
