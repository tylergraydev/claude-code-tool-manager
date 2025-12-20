use crate::db::Database;
use crate::services::claude_json;
use crate::services::config_parser;
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

    // First scan claude.json for projects and MCPs
    let claude_json_count = scan_claude_json(&db)?;
    log::info!("Imported {} MCPs from claude.json", claude_json_count);

    // Then scan plugins/marketplaces for additional MCPs
    let plugin_count = scan_plugins(&db)?;
    log::info!("Found {} MCPs from plugins", plugin_count);

    // Scan global skills from ~/.claude/commands/
    let skill_count = scan_global_skills(&db)?;
    log::info!("Found {} skills from ~/.claude/commands/", skill_count);

    // Scan global agents from ~/.claude/agents/
    let agent_count = scan_global_agents(&db)?;
    log::info!("Found {} agents from ~/.claude/agents/", agent_count);

    Ok(())
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

        // Scan project-level skills from .claude/commands/
        let project_commands_dir = Path::new(&path_to_check).join(".claude").join("commands");
        if project_commands_dir.exists() {
            scan_project_skills(db, project_id, &project_commands_dir)?;
        }

        // Scan project-level agents from .claude/agents/
        let project_agents_dir = Path::new(&path_to_check).join(".claude").join("agents");
        if project_agents_dir.exists() {
            scan_project_agents(db, project_id, &project_agents_dir)?;
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
fn assign_mcp_to_project(db: &Database, project_id: i64, mcp_id: i64, is_enabled: bool) -> Result<()> {
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
                                .query_row(
                                    "SELECT 1 FROM mcps WHERE name = ?",
                                    [&mcp.name],
                                    |_| Ok(true),
                                )
                                .unwrap_or(false);

                            if !exists {
                                let args_json =
                                    mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
                                let headers_json =
                                    mcp.headers.as_ref().map(|h| serde_json::to_string(h).unwrap());
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
                        .query_row(
                            "SELECT 1 FROM skills WHERE name = ?",
                            [&skill.name],
                            |_| Ok(true),
                        )
                        .unwrap_or(false);

                    if !exists {
                        let tags_json = if skill.tags.is_empty() {
                            None
                        } else {
                            Some(serde_json::to_string(&skill.tags).unwrap())
                        };

                        let result = db.conn().execute(
                            "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, tags, source)
                             VALUES (?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
                            params![
                                skill.name,
                                skill.description,
                                skill.content,
                                skill.skill_type,
                                skill.allowed_tools,
                                skill.argument_hint,
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
                        let tags_json = if agent.tags.is_empty() {
                            None
                        } else {
                            Some(serde_json::to_string(&agent.tags).unwrap())
                        };

                        let result = db.conn().execute(
                            "INSERT INTO subagents (name, description, content, tools, model, tags, source)
                             VALUES (?, ?, ?, ?, ?, ?, 'auto-detected')",
                            params![
                                agent.name,
                                agent.description,
                                agent.content,
                                tools_json,
                                agent.model,
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
struct ParsedSkill {
    name: String,
    description: Option<String>,
    content: String,
    skill_type: String,
    allowed_tools: Option<String>,
    argument_hint: Option<String>,
    tags: Vec<String>,
}

/// Parsed agent data from markdown file
struct ParsedAgent {
    name: String,
    description: String,
    content: String,
    tools: Vec<String>,
    model: Option<String>,
    tags: Vec<String>,
}

/// Parse a skill markdown file
fn parse_skill_file(path: &Path) -> Option<ParsedSkill> {
    let content = std::fs::read_to_string(path).ok()?;
    let file_name = path.file_stem()?.to_string_lossy().to_string();

    // Parse frontmatter if present (between --- markers)
    let (frontmatter, body) = parse_frontmatter(&content);

    // Extract metadata from frontmatter
    let description = frontmatter.get("description").cloned();
    let skill_type = frontmatter.get("type").cloned().unwrap_or_else(|| "command".to_string());
    let allowed_tools = frontmatter.get("allowed_tools").or_else(|| frontmatter.get("allowedTools")).cloned();
    let argument_hint = frontmatter.get("argument_hint").or_else(|| frontmatter.get("argumentHint")).cloned();
    let tags = frontmatter.get("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    Some(ParsedSkill {
        name: file_name,
        description,
        content: body,
        skill_type,
        allowed_tools,
        argument_hint,
        tags,
    })
}

/// Parse an agent markdown file
fn parse_agent_file(path: &Path) -> Option<ParsedAgent> {
    let content = std::fs::read_to_string(path).ok()?;
    let file_name = path.file_stem()?.to_string_lossy().to_string();

    // Parse frontmatter if present
    let (frontmatter, body) = parse_frontmatter(&content);

    // Extract metadata from frontmatter
    let description = frontmatter.get("description").cloned().unwrap_or_else(|| file_name.clone());
    let model = frontmatter.get("model").cloned();
    let tools = frontmatter.get("tools")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();
    let tags = frontmatter.get("tags")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        .unwrap_or_default();

    Some(ParsedAgent {
        name: file_name,
        description,
        content: body,
        tools,
        model,
        tags,
    })
}

/// Parse YAML-like frontmatter from markdown content
fn parse_frontmatter(content: &str) -> (std::collections::HashMap<String, String>, String) {
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

/// Scan project-level skills and assign to project
fn scan_project_skills(db: &Database, project_id: i64, commands_dir: &Path) -> Result<usize> {
    let mut count = 0;

    for entry in std::fs::read_dir(commands_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .md files
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(skill) = parse_skill_file(&path) {
                // Get or create the skill in the library
                let skill_id = get_or_create_skill(db, &skill)?;

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

/// Get or create a skill in the database
fn get_or_create_skill(db: &Database, skill: &ParsedSkill) -> Result<i64> {
    // Try to find existing skill by name
    let existing_id: Option<i64> = db
        .conn()
        .query_row("SELECT id FROM skills WHERE name = ?", [&skill.name], |row| {
            row.get(0)
        })
        .ok();

    if let Some(id) = existing_id {
        return Ok(id);
    }

    // Create new skill
    let tags_json = if skill.tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&skill.tags).unwrap())
    };

    db.conn().execute(
        "INSERT INTO skills (name, description, content, skill_type, allowed_tools, argument_hint, tags, source)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'auto-detected')",
        params![
            skill.name,
            skill.description,
            skill.content,
            skill.skill_type,
            skill.allowed_tools,
            skill.argument_hint,
            tags_json
        ],
    )?;

    Ok(db.conn().last_insert_rowid())
}

/// Get or create an agent in the database
fn get_or_create_agent(db: &Database, agent: &ParsedAgent) -> Result<i64> {
    // Try to find existing agent by name
    let existing_id: Option<i64> = db
        .conn()
        .query_row("SELECT id FROM subagents WHERE name = ?", [&agent.name], |row| {
            row.get(0)
        })
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
    let tags_json = if agent.tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&agent.tags).unwrap())
    };

    db.conn().execute(
        "INSERT INTO subagents (name, description, content, tools, model, tags, source)
         VALUES (?, ?, ?, ?, ?, ?, 'auto-detected')",
        params![
            agent.name,
            agent.description,
            agent.content,
            tools_json,
            agent.model,
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
