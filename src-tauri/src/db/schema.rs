use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn })
    }

    /// Create a Database from an existing connection (for testing with in-memory databases)
    #[cfg(test)]
    pub fn from_connection(conn: Connection) -> Self {
        Self { conn }
    }

    /// Create an in-memory database for testing
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let db = Self { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- MCP Library
            CREATE TABLE IF NOT EXISTS mcps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                type TEXT NOT NULL CHECK (type IN ('stdio', 'sse', 'http')),
                command TEXT,
                args TEXT,
                url TEXT,
                headers TEXT,
                env TEXT,
                icon TEXT,
                tags TEXT,
                source TEXT DEFAULT 'manual',
                source_path TEXT,
                is_enabled_global INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Projects
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                has_mcp_file INTEGER DEFAULT 0,
                has_settings_file INTEGER DEFAULT 0,
                last_scanned_at TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Project MCP Assignments
            CREATE TABLE IF NOT EXISTS project_mcps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                mcp_id INTEGER NOT NULL,
                is_enabled INTEGER DEFAULT 1,
                env_overrides TEXT,
                display_order INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (mcp_id) REFERENCES mcps(id) ON DELETE CASCADE,
                UNIQUE (project_id, mcp_id)
            );

            -- Global MCP Settings
            CREATE TABLE IF NOT EXISTS global_mcps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mcp_id INTEGER NOT NULL UNIQUE,
                is_enabled INTEGER DEFAULT 1,
                env_overrides TEXT,
                display_order INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (mcp_id) REFERENCES mcps(id) ON DELETE CASCADE
            );

            -- Slash Commands (user-invoked with /name)
            CREATE TABLE IF NOT EXISTS commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                content TEXT NOT NULL,
                allowed_tools TEXT,
                argument_hint TEXT,
                model TEXT,
                tags TEXT,
                source TEXT DEFAULT 'manual',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Skills (Agent Skills - auto-invoked by Claude)
            CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                content TEXT NOT NULL,
                allowed_tools TEXT,
                model TEXT,
                disable_model_invocation INTEGER DEFAULT 0,
                tags TEXT,
                source TEXT DEFAULT 'manual',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Sub-Agents
            CREATE TABLE IF NOT EXISTS subagents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL,
                content TEXT NOT NULL,
                tools TEXT,
                model TEXT,
                permission_mode TEXT,
                skills TEXT,
                tags TEXT,
                source TEXT DEFAULT 'manual',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Project Skill Assignments
            CREATE TABLE IF NOT EXISTS project_skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                skill_id INTEGER NOT NULL,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE,
                UNIQUE (project_id, skill_id)
            );

            -- Project Sub-Agent Assignments
            CREATE TABLE IF NOT EXISTS project_subagents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                subagent_id INTEGER NOT NULL,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (subagent_id) REFERENCES subagents(id) ON DELETE CASCADE,
                UNIQUE (project_id, subagent_id)
            );

            -- Global Command Settings
            CREATE TABLE IF NOT EXISTS global_commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command_id INTEGER NOT NULL UNIQUE,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (command_id) REFERENCES commands(id) ON DELETE CASCADE
            );

            -- Project Command Assignments
            CREATE TABLE IF NOT EXISTS project_commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                command_id INTEGER NOT NULL,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (command_id) REFERENCES commands(id) ON DELETE CASCADE,
                UNIQUE (project_id, command_id)
            );

            -- Global Skill Settings
            CREATE TABLE IF NOT EXISTS global_skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                skill_id INTEGER NOT NULL UNIQUE,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE
            );

            -- Global Sub-Agent Settings
            CREATE TABLE IF NOT EXISTS global_subagents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                subagent_id INTEGER NOT NULL UNIQUE,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (subagent_id) REFERENCES subagents(id) ON DELETE CASCADE
            );

            -- Hooks (Event-triggered actions)
            CREATE TABLE IF NOT EXISTS hooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                event_type TEXT NOT NULL,
                matcher TEXT,
                hook_type TEXT NOT NULL CHECK (hook_type IN ('command', 'prompt')),
                command TEXT,
                prompt TEXT,
                timeout INTEGER,
                tags TEXT,
                source TEXT DEFAULT 'manual',
                is_template INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Project Hook Assignments
            CREATE TABLE IF NOT EXISTS project_hooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                hook_id INTEGER NOT NULL,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (hook_id) REFERENCES hooks(id) ON DELETE CASCADE,
                UNIQUE (project_id, hook_id)
            );

            -- Global Hook Settings
            CREATE TABLE IF NOT EXISTS global_hooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                hook_id INTEGER NOT NULL UNIQUE,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (hook_id) REFERENCES hooks(id) ON DELETE CASCADE
            );

            -- Gateway MCP Assignments (MCPs included in the gateway)
            CREATE TABLE IF NOT EXISTS gateway_mcps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mcp_id INTEGER NOT NULL UNIQUE,
                is_enabled INTEGER DEFAULT 1,
                auto_restart INTEGER DEFAULT 1,
                display_order INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (mcp_id) REFERENCES mcps(id) ON DELETE CASCADE
            );

            -- Repository sources (awesome lists, skill repos, etc.)
            CREATE TABLE IF NOT EXISTS repos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                owner TEXT NOT NULL,
                repo TEXT NOT NULL,
                repo_type TEXT NOT NULL CHECK (repo_type IN ('file_based', 'readme_based')),
                content_type TEXT NOT NULL CHECK (content_type IN ('mcp', 'skill', 'subagent', 'mixed')),
                github_url TEXT NOT NULL UNIQUE,
                description TEXT,
                is_default INTEGER DEFAULT 0,
                is_enabled INTEGER DEFAULT 1,
                last_fetched_at TEXT,
                etag TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Cached items from repositories
            CREATE TABLE IF NOT EXISTS repo_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_id INTEGER NOT NULL,
                item_type TEXT NOT NULL CHECK (item_type IN ('mcp', 'skill', 'subagent')),
                name TEXT NOT NULL,
                description TEXT,
                source_url TEXT,
                raw_content TEXT,
                file_path TEXT,
                metadata TEXT,
                stars INTEGER,
                is_imported INTEGER DEFAULT 0,
                imported_item_id INTEGER,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (repo_id) REFERENCES repos(id) ON DELETE CASCADE,
                UNIQUE (repo_id, name, item_type)
            );

            -- Skill Files (references, assets, scripts)
            CREATE TABLE IF NOT EXISTS skill_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                skill_id INTEGER NOT NULL,
                file_type TEXT NOT NULL CHECK (file_type IN ('reference', 'asset', 'script')),
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (skill_id) REFERENCES skills(id) ON DELETE CASCADE,
                UNIQUE (skill_id, file_type, name)
            );

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_skill_files_skill ON skill_files(skill_id);
            CREATE INDEX IF NOT EXISTS idx_skill_files_type ON skill_files(file_type);
            CREATE INDEX IF NOT EXISTS idx_mcps_type ON mcps(type);
            CREATE INDEX IF NOT EXISTS idx_mcps_source ON mcps(source);
            CREATE INDEX IF NOT EXISTS idx_project_mcps_project ON project_mcps(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_mcps_mcp ON project_mcps(mcp_id);
            CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
            CREATE INDEX IF NOT EXISTS idx_project_commands_project ON project_commands(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_commands_command ON project_commands(command_id);
            CREATE INDEX IF NOT EXISTS idx_project_skills_project ON project_skills(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_subagents_project ON project_subagents(project_id);
            CREATE INDEX IF NOT EXISTS idx_repos_content_type ON repos(content_type);
            CREATE INDEX IF NOT EXISTS idx_repos_enabled ON repos(is_enabled);
            CREATE INDEX IF NOT EXISTS idx_repo_items_repo ON repo_items(repo_id);
            CREATE INDEX IF NOT EXISTS idx_repo_items_type ON repo_items(item_type);
            CREATE INDEX IF NOT EXISTS idx_repo_items_imported ON repo_items(is_imported);
            CREATE INDEX IF NOT EXISTS idx_gateway_mcps_mcp ON gateway_mcps(mcp_id);
            CREATE INDEX IF NOT EXISTS idx_gateway_mcps_enabled ON gateway_mcps(is_enabled);
            "#,
        )?;

        // Run migrations for existing databases
        self.run_schema_migrations()?;

        Ok(())
    }

    fn run_schema_migrations(&self) -> Result<()> {
        // Migration 1: Add allowed_tools to skills table (for databases created before this column existed)
        // Note: skill_type and argument_hint were moved to the commands table in a later migration
        let has_allowed_tools: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('skills') WHERE name = 'allowed_tools'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_allowed_tools {
            self.conn.execute_batch(
                r#"
                ALTER TABLE skills ADD COLUMN allowed_tools TEXT;
                "#,
            )?;
        }

        // Migration 2: Add permission_mode and skills columns to subagents
        let has_permission_mode: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('subagents') WHERE name = 'permission_mode'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !has_permission_mode {
            self.conn.execute_batch(
                r#"
                ALTER TABLE subagents ADD COLUMN permission_mode TEXT;
                ALTER TABLE subagents ADD COLUMN skills TEXT;
                "#,
            )?;
        }

        // Migration 3: Rename agents tables to subagents
        let has_agents_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='agents'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if has_agents_table {
            self.conn.execute_batch(
                r#"
                -- Migrate agents to subagents
                INSERT OR IGNORE INTO subagents (id, name, description, content, tools, model, tags, source, created_at, updated_at)
                SELECT id, name, description, content, tools, model, tags, source, created_at, updated_at FROM agents;

                -- Migrate global_agents to global_subagents
                INSERT OR IGNORE INTO global_subagents (id, subagent_id, is_enabled, created_at)
                SELECT id, agent_id, is_enabled, created_at FROM global_agents;

                -- Migrate project_agents to project_subagents
                INSERT OR IGNORE INTO project_subagents (id, project_id, subagent_id, is_enabled, created_at)
                SELECT id, project_id, agent_id, is_enabled, created_at FROM project_agents;

                -- Drop old tables
                DROP TABLE IF EXISTS project_agents;
                DROP TABLE IF EXISTS global_agents;
                DROP TABLE IF EXISTS agents;

                -- Drop old indexes
                DROP INDEX IF EXISTS idx_project_agents_project;
                "#,
            )?;
        }

        // Migration 4: Add model and disable_model_invocation columns to skills
        let has_skill_model: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('skills') WHERE name = 'model'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_skill_model {
            self.conn.execute_batch(
                r#"
                ALTER TABLE skills ADD COLUMN model TEXT;
                ALTER TABLE skills ADD COLUMN disable_model_invocation INTEGER DEFAULT 0;
                "#,
            )?;
        }

        // Migration 5: Add editor_type column to projects for OpenCode support
        let has_editor_type: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('projects') WHERE name = 'editor_type'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_editor_type {
            self.conn.execute_batch(
                r#"
                ALTER TABLE projects ADD COLUMN editor_type TEXT DEFAULT 'claude_code';
                "#,
            )?;
        }

        // Migration 6: Migrate commands from skills table to new commands table
        let has_commands_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='commands'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        let has_skill_type_column: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('skills') WHERE name = 'skill_type'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        // Only run migration if we have the old skill_type column and the new commands table
        if has_commands_table && has_skill_type_column {
            // Migrate commands from skills to commands table
            // Note: skills table doesn't have argument_hint column, so we use NULL for it
            self.conn.execute_batch(
                r#"
                -- Move skill_type='command' entries to commands table
                INSERT OR IGNORE INTO commands (name, description, content, allowed_tools, argument_hint, model, tags, source, created_at, updated_at)
                SELECT name, description, content, allowed_tools, NULL, model, tags, source, created_at, updated_at
                FROM skills WHERE skill_type = 'command';

                -- Migrate global_skills entries for commands to global_commands
                INSERT OR IGNORE INTO global_commands (command_id, is_enabled, created_at)
                SELECT c.id, gs.is_enabled, gs.created_at
                FROM global_skills gs
                JOIN skills s ON gs.skill_id = s.id
                JOIN commands c ON s.name = c.name
                WHERE s.skill_type = 'command';

                -- Migrate project_skills entries for commands to project_commands
                INSERT OR IGNORE INTO project_commands (project_id, command_id, is_enabled, created_at)
                SELECT ps.project_id, c.id, ps.is_enabled, ps.created_at
                FROM project_skills ps
                JOIN skills s ON ps.skill_id = s.id
                JOIN commands c ON s.name = c.name
                WHERE s.skill_type = 'command';

                -- Remove migrated commands from global_skills
                DELETE FROM global_skills WHERE skill_id IN (SELECT id FROM skills WHERE skill_type = 'command');

                -- Remove migrated commands from project_skills
                DELETE FROM project_skills WHERE skill_id IN (SELECT id FROM skills WHERE skill_type = 'command');

                -- Delete commands from skills table (now in commands table)
                DELETE FROM skills WHERE skill_type = 'command';
                "#,
            )?;
        }

        // Migration 7: Create app_settings table for application preferences
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Insert default editor setting if not exists
            INSERT OR IGNORE INTO app_settings (key, value) VALUES ('default_editor', 'claude_code');

            -- Insert default gateway settings if not exists
            INSERT OR IGNORE INTO app_settings (key, value) VALUES ('gateway_enabled', 'false');
            INSERT OR IGNORE INTO app_settings (key, value) VALUES ('gateway_port', '23848');
            INSERT OR IGNORE INTO app_settings (key, value) VALUES ('gateway_auto_start', 'false');
            "#,
        )?;

        // Migration 8: Add source_path column to skills and commands tables
        let has_skills_source_path: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('skills') WHERE name = 'source_path'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_skills_source_path {
            self.conn.execute_batch(
                r#"
                ALTER TABLE skills ADD COLUMN source_path TEXT;
                "#,
            )?;
        }

        let has_commands_source_path: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('commands') WHERE name = 'source_path'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_commands_source_path {
            self.conn.execute_batch(
                r#"
                ALTER TABLE commands ADD COLUMN source_path TEXT;
                "#,
            )?;
        }

        // Migration 9: Add source_path column to subagents table
        let has_subagents_source_path: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('subagents') WHERE name = 'source_path'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_subagents_source_path {
            self.conn.execute_batch(
                r#"
                ALTER TABLE subagents ADD COLUMN source_path TEXT;
                "#,
            )?;
        }

        // Migration 10: Add is_favorite column to mcps, commands, skills, and subagents tables
        let has_mcps_favorite: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('mcps') WHERE name = 'is_favorite'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_mcps_favorite {
            self.conn.execute_batch(
                r#"
                ALTER TABLE mcps ADD COLUMN is_favorite INTEGER DEFAULT 0;
                ALTER TABLE commands ADD COLUMN is_favorite INTEGER DEFAULT 0;
                ALTER TABLE skills ADD COLUMN is_favorite INTEGER DEFAULT 0;
                ALTER TABLE subagents ADD COLUMN is_favorite INTEGER DEFAULT 0;
                "#,
            )?;
        }

        // Migration 11: Add profiles and profile_items tables
        let has_profiles_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='profiles'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_profiles_table {
            self.conn.execute_batch(
                r#"
                CREATE TABLE profiles (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    icon TEXT,
                    is_active INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE profile_items (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    profile_id INTEGER NOT NULL,
                    item_type TEXT NOT NULL CHECK (item_type IN ('mcp', 'skill', 'command', 'subagent', 'hook')),
                    item_id INTEGER NOT NULL,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
                    UNIQUE(profile_id, item_type, item_id)
                );

                CREATE INDEX idx_profile_items_profile ON profile_items(profile_id);
                CREATE INDEX idx_profiles_active ON profiles(is_active);
                "#,
            )?;
        }

        // Migration 12: Add is_favorite column to projects table
        let has_projects_favorite: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('projects') WHERE name = 'is_favorite'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_projects_favorite {
            self.conn.execute(
                "ALTER TABLE projects ADD COLUMN is_favorite INTEGER DEFAULT 0",
                [],
            )?;
        }

        // Migration 13: Add statuslines table
        let has_statuslines_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='statuslines'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_statuslines_table {
            self.conn.execute_batch(
                r#"
                CREATE TABLE statuslines (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    statusline_type TEXT NOT NULL CHECK (statusline_type IN ('custom', 'premade', 'raw')),
                    package_name TEXT,
                    install_command TEXT,
                    run_command TEXT,
                    raw_command TEXT,
                    padding INTEGER DEFAULT 0,
                    is_active INTEGER DEFAULT 0,
                    segments_json TEXT,
                    generated_script TEXT,
                    icon TEXT,
                    author TEXT,
                    homepage_url TEXT,
                    tags TEXT,
                    source TEXT DEFAULT 'manual',
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );
                CREATE INDEX idx_statuslines_active ON statuslines(is_active);
                "#,
            )?;
        }

        Ok(())
    }

    // App settings methods
    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?",
                [key],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
            [key, value],
        )?;
        Ok(())
    }

    // ========================================================================
    // MCP Methods
    // ========================================================================

    pub fn get_all_mcps(&self) -> Result<Vec<crate::db::models::Mcp>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, is_favorite, created_at, updated_at
             FROM mcps ORDER BY name",
        )?;

        let mcps = stmt
            .query_map([], |row| {
                Ok(crate::db::models::Mcp {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    mcp_type: row.get(3)?,
                    command: row.get(4)?,
                    args: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    url: row.get(6)?,
                    headers: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    env: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    icon: row.get(9)?,
                    tags: row
                        .get::<_, Option<String>>(10)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(11)?,
                    source_path: row.get(12)?,
                    is_enabled_global: row.get::<_, i32>(13)? != 0,
                    is_favorite: row.get::<_, i32>(14)? != 0,
                    created_at: row.get(15)?,
                    updated_at: row.get(16)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(mcps)
    }

    pub fn get_mcp_by_id(&self, id: i64) -> Result<Option<crate::db::models::Mcp>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, is_favorite, created_at, updated_at
             FROM mcps WHERE id = ?",
            [id],
            |row| {
                Ok(crate::db::models::Mcp {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    mcp_type: row.get(3)?,
                    command: row.get(4)?,
                    args: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    url: row.get(6)?,
                    headers: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    env: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    icon: row.get(9)?,
                    tags: row
                        .get::<_, Option<String>>(10)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(11)?,
                    source_path: row.get(12)?,
                    is_enabled_global: row.get::<_, i32>(13)? != 0,
                    is_favorite: row.get::<_, i32>(14)? != 0,
                    created_at: row.get(15)?,
                    updated_at: row.get(16)?,
                })
            },
        );

        match result {
            Ok(mcp) => Ok(Some(mcp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_mcp_by_name(&self, name: &str) -> Result<Option<crate::db::models::Mcp>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, type, command, args, url, headers, env,
                    icon, tags, source, source_path, is_enabled_global, is_favorite, created_at, updated_at
             FROM mcps WHERE name = ?",
            [name],
            |row| {
                Ok(crate::db::models::Mcp {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    mcp_type: row.get(3)?,
                    command: row.get(4)?,
                    args: row
                        .get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    url: row.get(6)?,
                    headers: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    env: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    icon: row.get(9)?,
                    tags: row
                        .get::<_, Option<String>>(10)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(11)?,
                    source_path: row.get(12)?,
                    is_enabled_global: row.get::<_, i32>(13)? != 0,
                    is_favorite: row.get::<_, i32>(14)? != 0,
                    created_at: row.get(15)?,
                    updated_at: row.get(16)?,
                })
            },
        );

        match result {
            Ok(mcp) => Ok(Some(mcp)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_mcp(
        &self,
        req: &crate::db::models::CreateMcpRequest,
    ) -> Result<crate::db::models::Mcp> {
        self.create_mcp_with_source(req, "manual")
    }

    /// Create a system MCP that cannot be edited by users
    pub fn create_system_mcp(
        &self,
        req: &crate::db::models::CreateMcpRequest,
    ) -> Result<crate::db::models::Mcp> {
        self.create_mcp_with_source(req, "system")
    }

    fn create_mcp_with_source(
        &self,
        req: &crate::db::models::CreateMcpRequest,
        source: &str,
    ) -> Result<crate::db::models::Mcp> {
        let args_json = req.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
        let headers_json = req
            .headers
            .as_ref()
            .map(|h| serde_json::to_string(h).unwrap());
        let env_json = req.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
        let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        self.conn.execute(
            "INSERT INTO mcps (name, description, type, command, args, url, headers, env, icon, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                req.name, req.description, req.mcp_type, req.command,
                args_json, req.url, headers_json, env_json, req.icon, tags_json, source
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_mcp_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created MCP"))
    }

    pub fn update_mcp(&self, mcp: &crate::db::models::Mcp) -> Result<crate::db::models::Mcp> {
        let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
        let headers_json = mcp
            .headers
            .as_ref()
            .map(|h| serde_json::to_string(h).unwrap());
        let env_json = mcp.env.as_ref().map(|e| serde_json::to_string(e).unwrap());
        let tags_json = mcp.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        self.conn.execute(
            "UPDATE mcps SET name = ?, description = ?, type = ?, command = ?, args = ?,
             url = ?, headers = ?, env = ?, icon = ?, tags = ?, source = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            rusqlite::params![
                mcp.name, mcp.description, mcp.mcp_type, mcp.command, args_json,
                mcp.url, headers_json, env_json, mcp.icon, tags_json, mcp.source, mcp.id
            ],
        )?;

        self.get_mcp_by_id(mcp.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated MCP"))
    }

    pub fn delete_mcp(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM mcps WHERE id = ?", [id])?;
        Ok(())
    }

    // ========================================================================
    // Project Methods
    // ========================================================================

    pub fn get_all_projects(&self) -> Result<Vec<crate::db::models::Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, path, has_mcp_file, has_settings_file, last_scanned_at,
                    editor_type, is_favorite, created_at, updated_at
             FROM projects ORDER BY name",
        )?;

        let projects: Vec<crate::db::models::Project> = stmt
            .query_map([], |row| {
                Ok(crate::db::models::Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    has_mcp_file: row.get::<_, i32>(3)? != 0,
                    has_settings_file: row.get::<_, i32>(4)? != 0,
                    last_scanned_at: row.get(5)?,
                    editor_type: row
                        .get::<_, Option<String>>(6)?
                        .unwrap_or_else(|| "claude_code".to_string()),
                    is_favorite: row.get::<_, i32>(7).unwrap_or(0) != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    assigned_mcps: vec![],
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(projects)
    }

    pub fn get_project_by_id(&self, id: i64) -> Result<Option<crate::db::models::Project>> {
        let result = self.conn.query_row(
            "SELECT id, name, path, has_mcp_file, has_settings_file, last_scanned_at,
                    editor_type, is_favorite, created_at, updated_at
             FROM projects WHERE id = ?",
            [id],
            |row| {
                Ok(crate::db::models::Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    has_mcp_file: row.get::<_, i32>(3)? != 0,
                    has_settings_file: row.get::<_, i32>(4)? != 0,
                    last_scanned_at: row.get(5)?,
                    editor_type: row
                        .get::<_, Option<String>>(6)?
                        .unwrap_or_else(|| "claude_code".to_string()),
                    is_favorite: row.get::<_, i32>(7).unwrap_or(0) != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    assigned_mcps: vec![],
                })
            },
        );

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn assign_mcp_to_project(&self, project_id: i64, mcp_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO project_mcps (project_id, mcp_id) VALUES (?, ?)",
            [project_id, mcp_id],
        )?;
        Ok(())
    }

    pub fn remove_mcp_from_project(&self, project_id: i64, mcp_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
            [project_id, mcp_id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Global MCP Methods
    // ========================================================================

    pub fn get_global_mcps(&self) -> Result<Vec<crate::db::models::GlobalMcp>> {
        let mut stmt = self.conn.prepare(
            "SELECT gm.id, gm.mcp_id, gm.is_enabled, gm.env_overrides,
                    m.id, m.name, m.description, m.type, m.command, m.args, m.url, m.headers, m.env,
                    m.icon, m.tags, m.source, m.source_path, m.is_enabled_global, m.is_favorite, m.created_at, m.updated_at
             FROM global_mcps gm
             JOIN mcps m ON gm.mcp_id = m.id
             ORDER BY gm.display_order"
        )?;

        let results = stmt
            .query_map([], |row| {
                let mcp = crate::db::models::Mcp {
                    id: row.get(4)?,
                    name: row.get(5)?,
                    description: row.get(6)?,
                    mcp_type: row.get(7)?,
                    command: row.get(8)?,
                    args: row
                        .get::<_, Option<String>>(9)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    url: row.get(10)?,
                    headers: row
                        .get::<_, Option<String>>(11)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    env: row
                        .get::<_, Option<String>>(12)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    icon: row.get(13)?,
                    tags: row
                        .get::<_, Option<String>>(14)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(15)?,
                    source_path: row.get(16)?,
                    is_enabled_global: row.get::<_, i32>(17)? != 0,
                    is_favorite: row.get::<_, i32>(18)? != 0,
                    created_at: row.get(19)?,
                    updated_at: row.get(20)?,
                };

                Ok(crate::db::models::GlobalMcp {
                    id: row.get(0)?,
                    mcp_id: row.get(1)?,
                    mcp,
                    is_enabled: row.get::<_, i32>(2)? != 0,
                    env_overrides: row
                        .get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn add_global_mcp(&self, mcp_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO global_mcps (mcp_id) VALUES (?)",
            [mcp_id],
        )?;
        Ok(())
    }

    pub fn remove_global_mcp(&self, mcp_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM global_mcps WHERE mcp_id = ?", [mcp_id])?;
        Ok(())
    }

    // ========================================================================
    // Gateway MCP Methods
    // ========================================================================

    pub fn get_gateway_mcps(&self) -> Result<Vec<crate::db::models::GatewayMcp>> {
        let mut stmt = self.conn.prepare(
            "SELECT gm.id, gm.mcp_id, gm.is_enabled, gm.auto_restart, gm.display_order, gm.created_at,
                    m.id, m.name, m.description, m.type, m.command, m.args, m.url, m.headers, m.env,
                    m.icon, m.tags, m.source, m.source_path, m.is_enabled_global, m.is_favorite, m.created_at, m.updated_at
             FROM gateway_mcps gm
             JOIN mcps m ON gm.mcp_id = m.id
             ORDER BY gm.display_order, m.name"
        )?;

        let results = stmt
            .query_map([], |row| {
                let mcp = crate::db::models::Mcp {
                    id: row.get(6)?,
                    name: row.get(7)?,
                    description: row.get(8)?,
                    mcp_type: row.get(9)?,
                    command: row.get(10)?,
                    args: row
                        .get::<_, Option<String>>(11)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    url: row.get(12)?,
                    headers: row
                        .get::<_, Option<String>>(13)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    env: row
                        .get::<_, Option<String>>(14)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    icon: row.get(15)?,
                    tags: row
                        .get::<_, Option<String>>(16)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(17)?,
                    source_path: row.get(18)?,
                    is_enabled_global: row.get::<_, i32>(19)? != 0,
                    is_favorite: row.get::<_, i32>(20)? != 0,
                    created_at: row.get(21)?,
                    updated_at: row.get(22)?,
                };
                Ok(crate::db::models::GatewayMcp {
                    id: row.get(0)?,
                    mcp_id: row.get(1)?,
                    mcp,
                    is_enabled: row.get::<_, i32>(2)? != 0,
                    auto_restart: row.get::<_, i32>(3)? != 0,
                    display_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn get_enabled_gateway_mcps(&self) -> Result<Vec<crate::db::models::GatewayMcp>> {
        let mut stmt = self.conn.prepare(
            "SELECT gm.id, gm.mcp_id, gm.is_enabled, gm.auto_restart, gm.display_order, gm.created_at,
                    m.id, m.name, m.description, m.type, m.command, m.args, m.url, m.headers, m.env,
                    m.icon, m.tags, m.source, m.source_path, m.is_enabled_global, m.is_favorite, m.created_at, m.updated_at
             FROM gateway_mcps gm
             JOIN mcps m ON gm.mcp_id = m.id
             WHERE gm.is_enabled = 1
             ORDER BY gm.display_order, m.name"
        )?;

        let results = stmt
            .query_map([], |row| {
                let mcp = crate::db::models::Mcp {
                    id: row.get(6)?,
                    name: row.get(7)?,
                    description: row.get(8)?,
                    mcp_type: row.get(9)?,
                    command: row.get(10)?,
                    args: row
                        .get::<_, Option<String>>(11)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    url: row.get(12)?,
                    headers: row
                        .get::<_, Option<String>>(13)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    env: row
                        .get::<_, Option<String>>(14)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    icon: row.get(15)?,
                    tags: row
                        .get::<_, Option<String>>(16)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(17)?,
                    source_path: row.get(18)?,
                    is_enabled_global: row.get::<_, i32>(19)? != 0,
                    is_favorite: row.get::<_, i32>(20)? != 0,
                    created_at: row.get(21)?,
                    updated_at: row.get(22)?,
                };
                Ok(crate::db::models::GatewayMcp {
                    id: row.get(0)?,
                    mcp_id: row.get(1)?,
                    mcp,
                    is_enabled: row.get::<_, i32>(2)? != 0,
                    auto_restart: row.get::<_, i32>(3)? != 0,
                    display_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn add_gateway_mcp(&self, mcp_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO gateway_mcps (mcp_id) VALUES (?)",
            [mcp_id],
        )?;
        Ok(())
    }

    pub fn remove_gateway_mcp(&self, mcp_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM gateway_mcps WHERE mcp_id = ?", [mcp_id])?;
        Ok(())
    }

    pub fn toggle_gateway_mcp(&self, id: i64, enabled: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE gateway_mcps SET is_enabled = ? WHERE id = ?",
            rusqlite::params![if enabled { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn set_gateway_mcp_auto_restart(&self, id: i64, auto_restart: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE gateway_mcps SET auto_restart = ? WHERE id = ?",
            rusqlite::params![if auto_restart { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn is_mcp_in_gateway(&self, mcp_id: i64) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM gateway_mcps WHERE mcp_id = ?",
            [mcp_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ========================================================================
    // Skills Methods
    // ========================================================================

    pub fn get_all_skills(&self) -> Result<Vec<crate::db::models::Skill>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, content, allowed_tools, model, disable_model_invocation, tags, source, source_path, is_favorite, created_at, updated_at
             FROM skills ORDER BY name"
        )?;

        let skills = stmt
            .query_map([], |row| {
                Ok(crate::db::models::Skill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    content: row.get(3)?,
                    allowed_tools: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    model: row.get(5)?,
                    disable_model_invocation: row.get::<_, i32>(6)? != 0,
                    tags: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(8)?,
                    source_path: row.get(9)?,
                    is_favorite: row.get::<_, i32>(10)? != 0,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(skills)
    }

    pub fn get_skill_by_id(&self, id: i64) -> Result<Option<crate::db::models::Skill>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, content, allowed_tools, model, disable_model_invocation, tags, source, source_path, is_favorite, created_at, updated_at
             FROM skills WHERE id = ?",
            [id],
            |row| {
                Ok(crate::db::models::Skill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    content: row.get(3)?,
                    allowed_tools: row.get::<_, Option<String>>(4)?.and_then(|s| serde_json::from_str(&s).ok()),
                    model: row.get(5)?,
                    disable_model_invocation: row.get::<_, i32>(6)? != 0,
                    tags: row.get::<_, Option<String>>(7)?.and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(8)?,
                    source_path: row.get(9)?,
                    is_favorite: row.get::<_, i32>(10)? != 0,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            },
        );

        match result {
            Ok(skill) => Ok(Some(skill)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_skill(
        &self,
        req: &crate::db::models::CreateSkillRequest,
    ) -> Result<crate::db::models::Skill> {
        let allowed_tools_json = req
            .allowed_tools
            .as_ref()
            .map(|a| serde_json::to_string(a).unwrap());
        let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());
        let disable_model_invocation = req.disable_model_invocation.unwrap_or(false) as i32;

        self.conn.execute(
            "INSERT INTO skills (name, description, content, allowed_tools, model, disable_model_invocation, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'manual')",
            rusqlite::params![
                req.name, req.description, req.content,
                allowed_tools_json, req.model, disable_model_invocation, tags_json
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_skill_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created skill"))
    }

    pub fn delete_skill(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM skills WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn add_global_skill(&self, skill_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO global_skills (skill_id) VALUES (?)",
            [skill_id],
        )?;
        Ok(())
    }

    pub fn remove_global_skill(&self, skill_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM global_skills WHERE skill_id = ?", [skill_id])?;
        Ok(())
    }

    // ========================================================================
    // SubAgent Methods
    // ========================================================================

    pub fn get_all_subagents(&self) -> Result<Vec<crate::db::models::SubAgent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, source_path, is_favorite, created_at, updated_at
             FROM subagents ORDER BY name"
        )?;

        let subagents = stmt
            .query_map([], |row| {
                Ok(crate::db::models::SubAgent {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    content: row.get(3)?,
                    tools: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    model: row.get(5)?,
                    permission_mode: row.get(6)?,
                    skills: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    tags: row
                        .get::<_, Option<String>>(8)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(9)?,
                    source_path: row.get(10)?,
                    is_favorite: row.get::<_, i32>(11)? != 0,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(subagents)
    }

    pub fn get_subagent_by_id(&self, id: i64) -> Result<Option<crate::db::models::SubAgent>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, content, tools, model, permission_mode, skills, tags, source, source_path, is_favorite, created_at, updated_at
             FROM subagents WHERE id = ?",
            [id],
            |row| {
                Ok(crate::db::models::SubAgent {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    content: row.get(3)?,
                    tools: row.get::<_, Option<String>>(4)?.and_then(|s| serde_json::from_str(&s).ok()),
                    model: row.get(5)?,
                    permission_mode: row.get(6)?,
                    skills: row.get::<_, Option<String>>(7)?.and_then(|s| serde_json::from_str(&s).ok()),
                    tags: row.get::<_, Option<String>>(8)?.and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(9)?,
                    source_path: row.get(10)?,
                    is_favorite: row.get::<_, i32>(11)? != 0,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        );

        match result {
            Ok(subagent) => Ok(Some(subagent)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_subagent(
        &self,
        req: &crate::db::models::CreateSubAgentRequest,
    ) -> Result<crate::db::models::SubAgent> {
        let tools_json = req
            .tools
            .as_ref()
            .map(|t| serde_json::to_string(t).unwrap());
        let skills_json = req
            .skills
            .as_ref()
            .map(|s| serde_json::to_string(s).unwrap());
        let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        self.conn.execute(
            "INSERT INTO subagents (name, description, content, tools, model, permission_mode, skills, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            rusqlite::params![
                req.name, req.description, req.content, tools_json,
                req.model, req.permission_mode, skills_json, tags_json
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_subagent_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created subagent"))
    }

    pub fn delete_subagent(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM subagents WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn add_global_subagent(&self, subagent_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO global_subagents (subagent_id) VALUES (?)",
            [subagent_id],
        )?;
        Ok(())
    }

    pub fn remove_global_subagent(&self, subagent_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM global_subagents WHERE subagent_id = ?",
            [subagent_id],
        )?;
        Ok(())
    }

    // ========================================================================
    // Hook Methods
    // ========================================================================

    pub fn get_all_hooks(&self) -> Result<Vec<crate::db::models::Hook>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source, is_template, created_at, updated_at
             FROM hooks WHERE is_template = 0 ORDER BY name"
        )?;

        let hooks = stmt
            .query_map([], |row| {
                Ok(crate::db::models::Hook {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    event_type: row.get(3)?,
                    matcher: row.get(4)?,
                    hook_type: row.get(5)?,
                    command: row.get(6)?,
                    prompt: row.get(7)?,
                    timeout: row.get(8)?,
                    tags: row
                        .get::<_, Option<String>>(9)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(10)?,
                    is_template: row.get::<_, i32>(11)? != 0,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(hooks)
    }

    pub fn get_hook_by_id(&self, id: i64) -> Result<Option<crate::db::models::Hook>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source, is_template, created_at, updated_at
             FROM hooks WHERE id = ?",
            [id],
            |row| {
                Ok(crate::db::models::Hook {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    event_type: row.get(3)?,
                    matcher: row.get(4)?,
                    hook_type: row.get(5)?,
                    command: row.get(6)?,
                    prompt: row.get(7)?,
                    timeout: row.get(8)?,
                    tags: row.get::<_, Option<String>>(9)?.and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(10)?,
                    is_template: row.get::<_, i32>(11)? != 0,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        );

        match result {
            Ok(hook) => Ok(Some(hook)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_hook(
        &self,
        req: &crate::db::models::CreateHookRequest,
    ) -> Result<crate::db::models::Hook> {
        let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        self.conn.execute(
            "INSERT INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            rusqlite::params![
                req.name, req.description, req.event_type, req.matcher,
                req.hook_type, req.command, req.prompt, req.timeout, tags_json
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_hook_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created hook"))
    }

    pub fn delete_hook(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM hooks WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn add_global_hook(&self, hook_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO global_hooks (hook_id) VALUES (?)",
            [hook_id],
        )?;
        Ok(())
    }

    pub fn remove_global_hook(&self, hook_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM global_hooks WHERE hook_id = ?", [hook_id])?;
        Ok(())
    }

    // ========================================================================
    // StatusLine Methods
    // ========================================================================

    pub fn get_all_statuslines(&self) -> Result<Vec<crate::db::models::StatusLine>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, statusline_type, package_name, install_command,
                    run_command, raw_command, padding, is_active, segments_json, generated_script,
                    icon, author, homepage_url, tags, source, created_at, updated_at
             FROM statuslines ORDER BY name",
        )?;

        let statuslines = stmt
            .query_map([], |row| {
                Ok(crate::db::models::StatusLine {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    statusline_type: row.get(3)?,
                    package_name: row.get(4)?,
                    install_command: row.get(5)?,
                    run_command: row.get(6)?,
                    raw_command: row.get(7)?,
                    padding: row.get(8)?,
                    is_active: row.get::<_, i32>(9)? != 0,
                    segments_json: row.get(10)?,
                    generated_script: row.get(11)?,
                    icon: row.get(12)?,
                    author: row.get(13)?,
                    homepage_url: row.get(14)?,
                    tags: row
                        .get::<_, Option<String>>(15)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(16)?,
                    created_at: row.get(17)?,
                    updated_at: row.get(18)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(statuslines)
    }

    pub fn get_statusline_by_id(&self, id: i64) -> Result<Option<crate::db::models::StatusLine>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, statusline_type, package_name, install_command,
                    run_command, raw_command, padding, is_active, segments_json, generated_script,
                    icon, author, homepage_url, tags, source, created_at, updated_at
             FROM statuslines WHERE id = ?",
            [id],
            |row| {
                Ok(crate::db::models::StatusLine {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    statusline_type: row.get(3)?,
                    package_name: row.get(4)?,
                    install_command: row.get(5)?,
                    run_command: row.get(6)?,
                    raw_command: row.get(7)?,
                    padding: row.get(8)?,
                    is_active: row.get::<_, i32>(9)? != 0,
                    segments_json: row.get(10)?,
                    generated_script: row.get(11)?,
                    icon: row.get(12)?,
                    author: row.get(13)?,
                    homepage_url: row.get(14)?,
                    tags: row
                        .get::<_, Option<String>>(15)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(16)?,
                    created_at: row.get(17)?,
                    updated_at: row.get(18)?,
                })
            },
        );

        match result {
            Ok(sl) => Ok(Some(sl)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_statusline(
        &self,
        req: &crate::db::models::CreateStatusLineRequest,
    ) -> Result<crate::db::models::StatusLine> {
        let tags_json = req.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        self.conn.execute(
            "INSERT INTO statuslines (name, description, statusline_type, package_name, install_command,
             run_command, raw_command, padding, segments_json, generated_script, icon, author,
             homepage_url, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            rusqlite::params![
                req.name, req.description, req.statusline_type, req.package_name,
                req.install_command, req.run_command, req.raw_command,
                req.padding.unwrap_or(0), req.segments_json, req.generated_script,
                req.icon, req.author, req.homepage_url, tags_json
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_statusline_by_id(id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created statusline"))
    }

    pub fn update_statusline(
        &self,
        sl: &crate::db::models::StatusLine,
    ) -> Result<crate::db::models::StatusLine> {
        let tags_json = sl.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());

        self.conn.execute(
            "UPDATE statuslines SET name = ?, description = ?, statusline_type = ?, package_name = ?,
             install_command = ?, run_command = ?, raw_command = ?, padding = ?, segments_json = ?,
             generated_script = ?, icon = ?, author = ?, homepage_url = ?, tags = ?,
             updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
            rusqlite::params![
                sl.name, sl.description, sl.statusline_type, sl.package_name,
                sl.install_command, sl.run_command, sl.raw_command, sl.padding,
                sl.segments_json, sl.generated_script, sl.icon, sl.author,
                sl.homepage_url, tags_json, sl.id
            ],
        )?;

        self.get_statusline_by_id(sl.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve updated statusline"))
    }

    pub fn delete_statusline(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM statuslines WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn set_active_statusline(&self, id: i64) -> Result<()> {
        // Deactivate all first
        self.conn
            .execute("UPDATE statuslines SET is_active = 0", [])?;
        // Activate the specified one
        self.conn
            .execute("UPDATE statuslines SET is_active = 1 WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn deactivate_all_statuslines(&self) -> Result<()> {
        self.conn
            .execute("UPDATE statuslines SET is_active = 0", [])?;
        Ok(())
    }

    pub fn get_active_statusline(&self) -> Result<Option<crate::db::models::StatusLine>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, statusline_type, package_name, install_command,
                    run_command, raw_command, padding, is_active, segments_json, generated_script,
                    icon, author, homepage_url, tags, source, created_at, updated_at
             FROM statuslines WHERE is_active = 1",
            [],
            |row| {
                Ok(crate::db::models::StatusLine {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    statusline_type: row.get(3)?,
                    package_name: row.get(4)?,
                    install_command: row.get(5)?,
                    run_command: row.get(6)?,
                    raw_command: row.get(7)?,
                    padding: row.get(8)?,
                    is_active: row.get::<_, i32>(9)? != 0,
                    segments_json: row.get(10)?,
                    generated_script: row.get(11)?,
                    icon: row.get(12)?,
                    author: row.get(13)?,
                    homepage_url: row.get(14)?,
                    tags: row
                        .get::<_, Option<String>>(15)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(16)?,
                    created_at: row.get(17)?,
                    updated_at: row.get(18)?,
                })
            },
        );

        match result {
            Ok(sl) => Ok(Some(sl)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
