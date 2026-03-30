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
                hook_type TEXT NOT NULL CHECK (hook_type IN ('command', 'prompt', 'http', 'agent')),
                command TEXT,
                prompt TEXT,
                timeout INTEGER,
                url TEXT,
                headers TEXT,
                allowed_env_vars TEXT,
                if_condition TEXT,
                status_message TEXT,
                once INTEGER DEFAULT 0,
                async_mode INTEGER DEFAULT 0,
                shell TEXT,
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

            -- Rules (conditional instruction files)
            CREATE TABLE IF NOT EXISTS rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                content TEXT NOT NULL,
                paths TEXT,
                tags TEXT,
                source TEXT DEFAULT 'manual',
                source_path TEXT,
                is_symlink INTEGER DEFAULT 0,
                symlink_target TEXT,
                is_favorite INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Global Rule Assignments
            CREATE TABLE IF NOT EXISTS global_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                rule_id INTEGER NOT NULL UNIQUE,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (rule_id) REFERENCES rules(id) ON DELETE CASCADE
            );

            -- Project Rule Assignments
            CREATE TABLE IF NOT EXISTS project_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                rule_id INTEGER NOT NULL,
                is_enabled INTEGER DEFAULT 1,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (rule_id) REFERENCES rules(id) ON DELETE CASCADE,
                UNIQUE (project_id, rule_id)
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
            CREATE INDEX IF NOT EXISTS idx_project_rules_project ON project_rules(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_rules_rule ON project_rules(rule_id);
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

        // Migration 14: Add spinner_verbs and spinner_verb_config tables
        let has_spinner_verbs_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='spinner_verbs'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_spinner_verbs_table {
            self.conn.execute_batch(
                r#"
                CREATE TABLE spinner_verbs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    verb TEXT NOT NULL UNIQUE,
                    is_enabled INTEGER DEFAULT 1,
                    display_order INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE spinner_verb_config (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    mode TEXT NOT NULL DEFAULT 'append' CHECK (mode IN ('append', 'replace')),
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                INSERT OR IGNORE INTO spinner_verb_config (id, mode) VALUES (1, 'append');
                "#,
            )?;
        }

        // Migration 15: Add permission_templates table
        let has_permission_templates_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='permission_templates'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_permission_templates_table {
            self.conn.execute_batch(
                r#"
                CREATE TABLE permission_templates (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    category TEXT NOT NULL CHECK (category IN ('allow', 'deny', 'ask')),
                    rule TEXT NOT NULL,
                    tool_name TEXT,
                    tags TEXT,
                    is_default INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );
                CREATE INDEX idx_permission_templates_category ON permission_templates(category);
                "#,
            )?;
        }

        // Migration 16: Add docker_hosts, containers, and project_containers tables
        let has_docker_hosts_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='docker_hosts'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_docker_hosts_table {
            self.conn.execute_batch(
                r#"
                CREATE TABLE docker_hosts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    host_type TEXT NOT NULL CHECK (host_type IN ('local', 'ssh', 'tcp')),
                    connection_uri TEXT,
                    ssh_key_path TEXT,
                    tls_ca_cert TEXT,
                    tls_cert TEXT,
                    tls_key TEXT,
                    is_default INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                -- Seed default local Docker host
                INSERT INTO docker_hosts (name, host_type, is_default)
                VALUES ('Local Docker', 'local', 1);

                CREATE TABLE containers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    container_type TEXT NOT NULL CHECK (container_type IN ('devcontainer', 'docker', 'custom')),
                    docker_host_id INTEGER NOT NULL DEFAULT 1,
                    docker_container_id TEXT,
                    image TEXT,
                    dockerfile TEXT,
                    devcontainer_json TEXT,
                    env TEXT,
                    ports TEXT,
                    volumes TEXT,
                    mounts TEXT,
                    features TEXT,
                    post_create_command TEXT,
                    post_start_command TEXT,
                    working_dir TEXT,
                    template_id TEXT,
                    repo_url TEXT,
                    icon TEXT,
                    tags TEXT,
                    is_favorite INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (docker_host_id) REFERENCES docker_hosts(id) ON DELETE SET DEFAULT
                );

                CREATE TABLE project_containers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    container_id INTEGER NOT NULL,
                    is_default INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                    FOREIGN KEY (container_id) REFERENCES containers(id) ON DELETE CASCADE,
                    UNIQUE (project_id, container_id)
                );

                CREATE INDEX idx_containers_docker_host ON containers(docker_host_id);
                CREATE INDEX idx_project_containers_project ON project_containers(project_id);
                CREATE INDEX idx_project_containers_container ON project_containers(container_id);
                "#,
            )?;
        }

        // Migration 17: Add repo_url column to containers table
        let has_repo_url: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('containers') WHERE name = 'repo_url'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_repo_url {
            self.conn
                .execute("ALTER TABLE containers ADD COLUMN repo_url TEXT", [])?;
        }

        // Migration 18: Add new hook types and fields to hooks table
        let has_hook_url: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('hooks') WHERE name = 'url'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_hook_url {
            // SQLite cannot ALTER CHECK constraints, so recreate the table
            self.conn.execute_batch(
                r#"
                CREATE TABLE hooks_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    event_type TEXT NOT NULL,
                    matcher TEXT,
                    hook_type TEXT NOT NULL CHECK (hook_type IN ('command', 'prompt', 'http', 'agent')),
                    command TEXT,
                    prompt TEXT,
                    timeout INTEGER,
                    url TEXT,
                    headers TEXT,
                    allowed_env_vars TEXT,
                    if_condition TEXT,
                    status_message TEXT,
                    once INTEGER DEFAULT 0,
                    async_mode INTEGER DEFAULT 0,
                    shell TEXT,
                    tags TEXT,
                    source TEXT DEFAULT 'manual',
                    is_template INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                INSERT INTO hooks_new (id, name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source, is_template, created_at, updated_at)
                SELECT id, name, description, event_type, matcher, hook_type, command, prompt, timeout, tags, source, is_template, created_at, updated_at FROM hooks;

                DROP TABLE hooks;
                ALTER TABLE hooks_new RENAME TO hooks;
                "#,
            )?;
        }

        // Migration 19: Add rules, global_rules, and project_rules tables
        let has_rules_table: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='rules'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_rules_table {
            self.conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS rules (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    content TEXT NOT NULL,
                    paths TEXT,
                    tags TEXT,
                    source TEXT DEFAULT 'manual',
                    source_path TEXT,
                    is_symlink INTEGER DEFAULT 0,
                    symlink_target TEXT,
                    is_favorite INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TABLE IF NOT EXISTS global_rules (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    rule_id INTEGER NOT NULL UNIQUE,
                    is_enabled INTEGER DEFAULT 1,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (rule_id) REFERENCES rules(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS project_rules (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    rule_id INTEGER NOT NULL,
                    is_enabled INTEGER DEFAULT 1,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                    FOREIGN KEY (rule_id) REFERENCES rules(id) ON DELETE CASCADE,
                    UNIQUE (project_id, rule_id)
                );

                CREATE INDEX IF NOT EXISTS idx_project_rules_project ON project_rules(project_id);
                CREATE INDEX IF NOT EXISTS idx_project_rules_rule ON project_rules(rule_id);
                "#,
            )?;
        }

        // Migration 20: Add new frontmatter fields to subagents table
        let has_subagents_max_turns: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('subagents') WHERE name = 'max_turns'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_subagents_max_turns {
            self.conn.execute_batch(
                r#"
                ALTER TABLE subagents ADD COLUMN disallowed_tools TEXT;
                ALTER TABLE subagents ADD COLUMN max_turns INTEGER;
                ALTER TABLE subagents ADD COLUMN memory TEXT;
                ALTER TABLE subagents ADD COLUMN background INTEGER;
                ALTER TABLE subagents ADD COLUMN effort TEXT;
                ALTER TABLE subagents ADD COLUMN isolation TEXT;
                ALTER TABLE subagents ADD COLUMN hooks TEXT;
                ALTER TABLE subagents ADD COLUMN mcp_servers TEXT;
                ALTER TABLE subagents ADD COLUMN initial_prompt TEXT;
                "#,
            )?;
        }

        // Migration 21: Add new frontmatter fields to skills table
        let has_skills_context: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('skills') WHERE name = 'context'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_skills_context {
            self.conn.execute_batch(
                r#"
                ALTER TABLE skills ADD COLUMN context TEXT;
                ALTER TABLE skills ADD COLUMN agent TEXT;
                ALTER TABLE skills ADD COLUMN hooks TEXT;
                ALTER TABLE skills ADD COLUMN paths TEXT;
                ALTER TABLE skills ADD COLUMN shell TEXT;
                ALTER TABLE skills ADD COLUMN once_per_session INTEGER;
                "#,
            )?;
        }

        // Migration 22: Add effort column to skills table
        let has_skills_effort: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('skills') WHERE name = 'effort'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_skills_effort {
            self.conn
                .execute_batch("ALTER TABLE skills ADD COLUMN effort TEXT;")?;
        }

        // Migration 23: Add 'ws' to MCP type CHECK constraint
        // SQLite doesn't support ALTER CONSTRAINT, so we rebuild the table
        // Check if constraint already allows 'ws' by inspecting the table SQL
        let table_sql: String = self
            .conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='mcps'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();

        if !table_sql.contains("'ws'") {
            self.conn.execute_batch(
                r#"
                PRAGMA foreign_keys = OFF;

                CREATE TABLE mcps_new (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    type TEXT NOT NULL CHECK (type IN ('stdio', 'sse', 'http', 'ws')),
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
                    is_favorite INTEGER DEFAULT 0,
                    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
                );

                INSERT INTO mcps_new SELECT * FROM mcps;
                DROP TABLE mcps;
                ALTER TABLE mcps_new RENAME TO mcps;

                CREATE INDEX IF NOT EXISTS idx_mcps_type ON mcps(type);
                CREATE INDEX IF NOT EXISTS idx_mcps_source ON mcps(source);

                PRAGMA foreign_keys = ON;
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
                    context: None,
                    agent: None,
                    hooks: None,
                    paths: None,
                    shell: None,
                    once: None,
                    effort: None,
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
                    context: None,
                    agent: None,
                    hooks: None,
                    paths: None,
                    shell: None,
                    once: None,
                    effort: None,
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
                    disallowed_tools: None,
                    max_turns: None,
                    memory: None,
                    background: None,
                    effort: None,
                    isolation: None,
                    hooks: None,
                    mcp_servers: None,
                    initial_prompt: None,
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
                    disallowed_tools: None,
                    max_turns: None,
                    memory: None,
                    background: None,
                    effort: None,
                    isolation: None,
                    hooks: None,
                    mcp_servers: None,
                    initial_prompt: None,
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
            "SELECT id, name, description, event_type, matcher, hook_type, command, prompt, timeout, url, headers, allowed_env_vars, if_condition, status_message, once, async_mode, shell, tags, source, is_template, created_at, updated_at
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
                    url: row.get(9)?,
                    headers: row
                        .get::<_, Option<String>>(10)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    allowed_env_vars: row
                        .get::<_, Option<String>>(11)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    if_condition: row.get(12)?,
                    status_message: row.get(13)?,
                    once: row.get::<_, i32>(14).unwrap_or(0) != 0,
                    async_mode: row.get::<_, i32>(15).unwrap_or(0) != 0,
                    shell: row.get(16)?,
                    tags: row
                        .get::<_, Option<String>>(17)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(18)?,
                    is_template: row.get::<_, i32>(19)? != 0,
                    created_at: row.get(20)?,
                    updated_at: row.get(21)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(hooks)
    }

    pub fn get_hook_by_id(&self, id: i64) -> Result<Option<crate::db::models::Hook>> {
        let result = self.conn.query_row(
            "SELECT id, name, description, event_type, matcher, hook_type, command, prompt, timeout, url, headers, allowed_env_vars, if_condition, status_message, once, async_mode, shell, tags, source, is_template, created_at, updated_at
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
                    url: row.get(9)?,
                    headers: row.get::<_, Option<String>>(10)?.and_then(|s| serde_json::from_str(&s).ok()),
                    allowed_env_vars: row.get::<_, Option<String>>(11)?.and_then(|s| serde_json::from_str(&s).ok()),
                    if_condition: row.get(12)?,
                    status_message: row.get(13)?,
                    once: row.get::<_, i32>(14).unwrap_or(0) != 0,
                    async_mode: row.get::<_, i32>(15).unwrap_or(0) != 0,
                    shell: row.get(16)?,
                    tags: row.get::<_, Option<String>>(17)?.and_then(|s| serde_json::from_str(&s).ok()),
                    source: row.get(18)?,
                    is_template: row.get::<_, i32>(19)? != 0,
                    created_at: row.get(20)?,
                    updated_at: row.get(21)?,
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
        let headers_json = req
            .headers
            .as_ref()
            .map(|h| serde_json::to_string(h).unwrap());
        let env_vars_json = req
            .allowed_env_vars
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());

        self.conn.execute(
            "INSERT INTO hooks (name, description, event_type, matcher, hook_type, command, prompt, timeout, url, headers, allowed_env_vars, if_condition, status_message, once, async_mode, shell, tags, source)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'manual')",
            rusqlite::params![
                req.name, req.description, req.event_type, req.matcher,
                req.hook_type, req.command, req.prompt, req.timeout,
                req.url, headers_json, env_vars_json,
                req.if_condition, req.status_message,
                req.once.unwrap_or(false) as i32,
                req.async_mode.unwrap_or(false) as i32,
                req.shell, tags_json
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

    // Spinner Verb methods
    pub fn get_all_spinner_verbs(&self) -> Result<Vec<crate::db::models::SpinnerVerb>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, verb, is_enabled, display_order, created_at, updated_at
             FROM spinner_verbs ORDER BY display_order, id",
        )?;

        let verbs = stmt
            .query_map([], |row| {
                Ok(crate::db::models::SpinnerVerb {
                    id: row.get(0)?,
                    verb: row.get(1)?,
                    is_enabled: row.get::<_, i32>(2)? != 0,
                    display_order: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(verbs)
    }

    pub fn get_spinner_verb_by_id(&self, id: i64) -> Result<crate::db::models::SpinnerVerb> {
        self.conn
            .query_row(
                "SELECT id, verb, is_enabled, display_order, created_at, updated_at
             FROM spinner_verbs WHERE id = ?",
                rusqlite::params![id],
                |row| {
                    Ok(crate::db::models::SpinnerVerb {
                        id: row.get(0)?,
                        verb: row.get(1)?,
                        is_enabled: row.get::<_, i32>(2)? != 0,
                        display_order: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .map_err(|e| e.into())
    }

    pub fn create_spinner_verb(&self, verb: &str) -> Result<crate::db::models::SpinnerVerb> {
        // Get the next display_order
        let max_order: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(display_order), -1) FROM spinner_verbs",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        self.conn.execute(
            "INSERT INTO spinner_verbs (verb, display_order) VALUES (?, ?)",
            rusqlite::params![verb, max_order + 1],
        )?;

        let id = self.conn.last_insert_rowid();
        self.get_spinner_verb_by_id(id)
    }

    pub fn update_spinner_verb(
        &self,
        id: i64,
        verb: &str,
        is_enabled: bool,
    ) -> Result<crate::db::models::SpinnerVerb> {
        self.conn.execute(
            "UPDATE spinner_verbs SET verb = ?, is_enabled = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            rusqlite::params![verb, is_enabled as i32, id],
        )?;

        self.get_spinner_verb_by_id(id)
    }

    pub fn delete_spinner_verb(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM spinner_verbs WHERE id = ?",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    pub fn get_spinner_verb_mode(&self) -> Result<String> {
        let mode = self
            .conn
            .query_row(
                "SELECT mode FROM spinner_verb_config WHERE id = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap_or_else(|_| "append".to_string());

        Ok(mode)
    }

    pub fn set_spinner_verb_mode(&self, mode: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO spinner_verb_config (id, mode, updated_at) VALUES (1, ?, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET mode = excluded.mode, updated_at = CURRENT_TIMESTAMP",
            rusqlite::params![mode],
        )?;
        Ok(())
    }

    pub fn reorder_spinner_verbs(&self, ids: &[i64]) -> Result<()> {
        for (index, id) in ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE spinner_verbs SET display_order = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                rusqlite::params![index as i32, id],
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::*;

    fn setup_db() -> Database {
        Database::in_memory().unwrap()
    }

    // =========================================================================
    // Migration / Setup tests
    // =========================================================================

    #[test]
    fn test_in_memory_db_creation() {
        let db = setup_db();
        // Should have all tables
        let table_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_count > 10);
    }

    #[test]
    fn test_migrations_idempotent() {
        let db = setup_db();
        // Running migrations again should not fail
        db.run_migrations().unwrap();
        db.run_schema_migrations().unwrap();
    }

    // =========================================================================
    // App Settings tests
    // =========================================================================

    #[test]
    fn test_get_setting_default_editor() {
        let db = setup_db();
        let val = db.get_setting("default_editor");
        assert_eq!(val, Some("claude_code".to_string()));
    }

    #[test]
    fn test_set_and_get_setting() {
        let db = setup_db();
        db.set_setting("my_key", "my_value").unwrap();
        assert_eq!(db.get_setting("my_key"), Some("my_value".to_string()));

        // Overwrite
        db.set_setting("my_key", "new_value").unwrap();
        assert_eq!(db.get_setting("my_key"), Some("new_value".to_string()));
    }

    #[test]
    fn test_get_setting_nonexistent() {
        let db = setup_db();
        assert_eq!(db.get_setting("nonexistent_key"), None);
    }

    // =========================================================================
    // MCP CRUD tests
    // =========================================================================

    fn create_test_mcp_request(name: &str) -> CreateMcpRequest {
        CreateMcpRequest {
            name: name.to_string(),
            description: Some("Test MCP".to_string()),
            mcp_type: "stdio".to_string(),
            command: Some("npx".to_string()),
            args: Some(vec!["server".to_string()]),
            url: None,
            headers: None,
            env: None,
            icon: None,
            tags: None,
        }
    }

    #[test]
    fn test_create_and_get_mcp() {
        let db = setup_db();
        let req = create_test_mcp_request("test-mcp");
        let mcp = db.create_mcp(&req).unwrap();

        assert_eq!(mcp.name, "test-mcp");
        assert_eq!(mcp.mcp_type, "stdio");
        assert_eq!(mcp.command, Some("npx".to_string()));
        assert_eq!(mcp.source, "manual".to_string());

        let fetched = db.get_mcp_by_id(mcp.id).unwrap().unwrap();
        assert_eq!(fetched.name, "test-mcp");
    }

    #[test]
    fn test_get_mcp_by_name() {
        let db = setup_db();
        db.create_mcp(&create_test_mcp_request("findme")).unwrap();

        let mcp = db.get_mcp_by_name("findme").unwrap().unwrap();
        assert_eq!(mcp.name, "findme");

        let none = db.get_mcp_by_name("nonexistent").unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_get_mcp_by_id_nonexistent() {
        let db = setup_db();
        let result = db.get_mcp_by_id(99999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_create_mcp_duplicate_name() {
        let db = setup_db();
        db.create_mcp(&create_test_mcp_request("dup")).unwrap();
        let result = db.create_mcp(&create_test_mcp_request("dup"));
        assert!(result.is_err());
    }

    #[test]
    fn test_update_mcp() {
        let db = setup_db();
        let mut mcp = db.create_mcp(&create_test_mcp_request("upd")).unwrap();
        mcp.description = Some("Updated desc".to_string());
        mcp.command = Some("node".to_string());

        let updated = db.update_mcp(&mcp).unwrap();
        assert_eq!(updated.description, Some("Updated desc".to_string()));
        assert_eq!(updated.command, Some("node".to_string()));
    }

    #[test]
    fn test_delete_mcp() {
        let db = setup_db();
        let mcp = db.create_mcp(&create_test_mcp_request("del")).unwrap();
        db.delete_mcp(mcp.id).unwrap();
        assert!(db.get_mcp_by_id(mcp.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_mcps() {
        let db = setup_db();
        db.create_mcp(&create_test_mcp_request("a")).unwrap();
        db.create_mcp(&create_test_mcp_request("b")).unwrap();
        db.create_mcp(&create_test_mcp_request("c")).unwrap();

        let all = db.get_all_mcps().unwrap();
        assert_eq!(all.len(), 3);
        // Should be ordered by name
        assert_eq!(all[0].name, "a");
        assert_eq!(all[1].name, "b");
        assert_eq!(all[2].name, "c");
    }

    #[test]
    fn test_create_system_mcp() {
        let db = setup_db();
        let req = create_test_mcp_request("sys-mcp");
        let mcp = db.create_system_mcp(&req).unwrap();
        assert_eq!(mcp.source, "system".to_string());
    }

    // =========================================================================
    // Project tests
    // =========================================================================

    #[test]
    fn test_get_all_projects() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO projects (name, path) VALUES ('proj1', '/tmp/proj1')",
                [],
            )
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO projects (name, path) VALUES ('proj2', '/tmp/proj2')",
                [],
            )
            .unwrap();

        let projects = db.get_all_projects().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_get_project_by_id() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO projects (name, path) VALUES ('proj', '/tmp/proj')",
                [],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let proj = db.get_project_by_id(id).unwrap().unwrap();
        assert_eq!(proj.name, "proj");
        assert_eq!(proj.path, "/tmp/proj");

        let none = db.get_project_by_id(99999).unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_assign_and_remove_mcp_from_project() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO projects (name, path) VALUES ('p', '/tmp/p')",
                [],
            )
            .unwrap();
        let proj_id = db.conn().last_insert_rowid();
        let mcp = db.create_mcp(&create_test_mcp_request("m")).unwrap();

        db.assign_mcp_to_project(proj_id, mcp.id).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
                [proj_id, mcp.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        db.remove_mcp_from_project(proj_id, mcp.id).unwrap();
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
                [proj_id, mcp.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Global MCP tests
    // =========================================================================

    #[test]
    fn test_add_and_remove_global_mcp() {
        let db = setup_db();
        let mcp = db.create_mcp(&create_test_mcp_request("gm")).unwrap();

        db.add_global_mcp(mcp.id).unwrap();
        let globals = db.get_global_mcps().unwrap();
        assert_eq!(globals.len(), 1);
        assert_eq!(globals[0].mcp.name, "gm");

        db.remove_global_mcp(mcp.id).unwrap();
        let globals = db.get_global_mcps().unwrap();
        assert!(globals.is_empty());
    }

    #[test]
    fn test_add_global_mcp_idempotent() {
        let db = setup_db();
        let mcp = db.create_mcp(&create_test_mcp_request("gm2")).unwrap();

        db.add_global_mcp(mcp.id).unwrap();
        db.add_global_mcp(mcp.id).unwrap(); // Should not fail
        let globals = db.get_global_mcps().unwrap();
        assert_eq!(globals.len(), 1);
    }

    // =========================================================================
    // Gateway MCP tests
    // =========================================================================

    #[test]
    fn test_gateway_mcp_lifecycle() {
        let db = setup_db();
        let mcp = db.create_mcp(&create_test_mcp_request("gw")).unwrap();

        db.add_gateway_mcp(mcp.id).unwrap();
        assert!(db.is_mcp_in_gateway(mcp.id).unwrap());

        let gateways = db.get_gateway_mcps().unwrap();
        assert_eq!(gateways.len(), 1);
        assert!(gateways[0].is_enabled);

        // Toggle off
        db.toggle_gateway_mcp(gateways[0].id, false).unwrap();
        let enabled = db.get_enabled_gateway_mcps().unwrap();
        assert!(enabled.is_empty());

        // Toggle on
        db.toggle_gateway_mcp(gateways[0].id, true).unwrap();
        let enabled = db.get_enabled_gateway_mcps().unwrap();
        assert_eq!(enabled.len(), 1);

        // Auto restart
        db.set_gateway_mcp_auto_restart(gateways[0].id, false)
            .unwrap();
        let gw = db.get_gateway_mcps().unwrap();
        assert!(!gw[0].auto_restart);

        // Remove
        db.remove_gateway_mcp(mcp.id).unwrap();
        assert!(!db.is_mcp_in_gateway(mcp.id).unwrap());
    }

    #[test]
    fn test_add_gateway_mcp_idempotent() {
        let db = setup_db();
        let mcp = db.create_mcp(&create_test_mcp_request("gw2")).unwrap();
        db.add_gateway_mcp(mcp.id).unwrap();
        db.add_gateway_mcp(mcp.id).unwrap();
        let gateways = db.get_gateway_mcps().unwrap();
        assert_eq!(gateways.len(), 1);
    }

    // =========================================================================
    // Skill CRUD tests
    // =========================================================================

    fn create_test_skill_request(name: &str) -> CreateSkillRequest {
        CreateSkillRequest {
            name: name.to_string(),
            description: Some("Test skill".to_string()),
            content: "Skill content".to_string(),
            allowed_tools: Some(vec!["Read".to_string()]),
            model: None,
            disable_model_invocation: None,
            tags: None,
            context: None,
            agent: None,
            hooks: None,
            paths: None,
            shell: None,
            once: None,
            effort: None,
        }
    }

    #[test]
    fn test_create_and_get_skill() {
        let db = setup_db();
        let skill = db.create_skill(&create_test_skill_request("sk1")).unwrap();
        assert_eq!(skill.name, "sk1");
        assert_eq!(skill.content, "Skill content");

        let fetched = db.get_skill_by_id(skill.id).unwrap().unwrap();
        assert_eq!(fetched.name, "sk1");
    }

    #[test]
    fn test_get_skill_by_id_nonexistent() {
        let db = setup_db();
        assert!(db.get_skill_by_id(99999).unwrap().is_none());
    }

    #[test]
    fn test_delete_skill() {
        let db = setup_db();
        let skill = db
            .create_skill(&create_test_skill_request("del-sk"))
            .unwrap();
        db.delete_skill(skill.id).unwrap();
        assert!(db.get_skill_by_id(skill.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_skills() {
        let db = setup_db();
        db.create_skill(&create_test_skill_request("sk-a")).unwrap();
        db.create_skill(&create_test_skill_request("sk-b")).unwrap();
        let all = db.get_all_skills().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_add_and_remove_global_skill() {
        let db = setup_db();
        let skill = db.create_skill(&create_test_skill_request("gs")).unwrap();
        db.add_global_skill(skill.id).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM global_skills WHERE skill_id = ?",
                [skill.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        db.remove_global_skill(skill.id).unwrap();
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM global_skills WHERE skill_id = ?",
                [skill.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // SubAgent CRUD tests
    // =========================================================================

    fn create_test_subagent_request(name: &str) -> CreateSubAgentRequest {
        CreateSubAgentRequest {
            name: name.to_string(),
            description: "Test agent".to_string(),
            content: "Agent instructions".to_string(),
            tools: Some(vec!["Read".to_string(), "Write".to_string()]),
            model: Some("opus".to_string()),
            permission_mode: None,
            skills: None,
            tags: None,
            disallowed_tools: None,
            max_turns: None,
            memory: None,
            background: None,
            effort: None,
            isolation: None,
            hooks: None,
            mcp_servers: None,
            initial_prompt: None,
        }
    }

    #[test]
    fn test_create_and_get_subagent() {
        let db = setup_db();
        let agent = db
            .create_subagent(&create_test_subagent_request("agent1"))
            .unwrap();
        assert_eq!(agent.name, "agent1");
        assert_eq!(agent.model, Some("opus".to_string()));

        let fetched = db.get_subagent_by_id(agent.id).unwrap().unwrap();
        assert_eq!(fetched.name, "agent1");
        assert_eq!(
            fetched.tools,
            Some(vec!["Read".to_string(), "Write".to_string()])
        );
    }

    #[test]
    fn test_get_subagent_by_id_nonexistent() {
        let db = setup_db();
        assert!(db.get_subagent_by_id(99999).unwrap().is_none());
    }

    #[test]
    fn test_delete_subagent() {
        let db = setup_db();
        let agent = db
            .create_subagent(&create_test_subagent_request("del-ag"))
            .unwrap();
        db.delete_subagent(agent.id).unwrap();
        assert!(db.get_subagent_by_id(agent.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_subagents() {
        let db = setup_db();
        db.create_subagent(&create_test_subagent_request("a1"))
            .unwrap();
        db.create_subagent(&create_test_subagent_request("a2"))
            .unwrap();
        let all = db.get_all_subagents().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_add_and_remove_global_subagent() {
        let db = setup_db();
        let agent = db
            .create_subagent(&create_test_subagent_request("gsa"))
            .unwrap();
        db.add_global_subagent(agent.id).unwrap();
        db.add_global_subagent(agent.id).unwrap(); // idempotent

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM global_subagents WHERE subagent_id = ?",
                [agent.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        db.remove_global_subagent(agent.id).unwrap();
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM global_subagents WHERE subagent_id = ?",
                [agent.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Hook CRUD tests
    // =========================================================================

    fn create_test_hook_request(name: &str) -> CreateHookRequest {
        CreateHookRequest {
            name: name.to_string(),
            description: Some("Test hook".to_string()),
            event_type: "PostToolUse".to_string(),
            matcher: Some("Write".to_string()),
            hook_type: "command".to_string(),
            command: Some("npm run lint".to_string()),
            prompt: None,
            timeout: Some(5000),
            tags: None,
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: None,
            async_mode: None,
            shell: None,
        }
    }

    #[test]
    fn test_create_and_get_hook() {
        let db = setup_db();
        let hook = db.create_hook(&create_test_hook_request("h1")).unwrap();
        assert_eq!(hook.name, "h1");
        assert_eq!(hook.event_type, "PostToolUse");
        assert_eq!(hook.hook_type, "command");
        assert_eq!(hook.command, Some("npm run lint".to_string()));
        assert_eq!(hook.timeout, Some(5000));
        assert!(!hook.is_template);

        let fetched = db.get_hook_by_id(hook.id).unwrap().unwrap();
        assert_eq!(fetched.name, "h1");
    }

    #[test]
    fn test_get_hook_by_id_nonexistent() {
        let db = setup_db();
        assert!(db.get_hook_by_id(99999).unwrap().is_none());
    }

    #[test]
    fn test_delete_hook() {
        let db = setup_db();
        let hook = db.create_hook(&create_test_hook_request("del-h")).unwrap();
        db.delete_hook(hook.id).unwrap();
        assert!(db.get_hook_by_id(hook.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_hooks_excludes_templates() {
        let db = setup_db();
        db.create_hook(&create_test_hook_request("h-reg")).unwrap();
        // Insert a template hook directly
        db.conn()
            .execute(
                "INSERT INTO hooks (name, event_type, hook_type, is_template) VALUES ('tmpl', 'PreToolUse', 'command', 1)",
                [],
            )
            .unwrap();

        let all = db.get_all_hooks().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "h-reg");
    }

    #[test]
    fn test_add_and_remove_global_hook() {
        let db = setup_db();
        let hook = db.create_hook(&create_test_hook_request("gh")).unwrap();
        db.add_global_hook(hook.id).unwrap();
        db.add_global_hook(hook.id).unwrap(); // idempotent

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM global_hooks WHERE hook_id = ?",
                [hook.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        db.remove_global_hook(hook.id).unwrap();
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM global_hooks WHERE hook_id = ?",
                [hook.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // StatusLine tests
    // =========================================================================

    fn create_test_statusline_request(name: &str) -> CreateStatusLineRequest {
        CreateStatusLineRequest {
            name: name.to_string(),
            description: Some("Test statusline".to_string()),
            statusline_type: "custom".to_string(),
            package_name: None,
            install_command: None,
            run_command: Some("echo status".to_string()),
            raw_command: None,
            padding: Some(1),
            segments_json: None,
            generated_script: None,
            icon: None,
            author: None,
            homepage_url: None,
            tags: None,
        }
    }

    #[test]
    fn test_create_and_get_statusline() {
        let db = setup_db();
        let sl = db
            .create_statusline(&create_test_statusline_request("sl1"))
            .unwrap();
        assert_eq!(sl.name, "sl1");
        assert_eq!(sl.statusline_type, "custom");

        let fetched = db.get_statusline_by_id(sl.id).unwrap().unwrap();
        assert_eq!(fetched.name, "sl1");
    }

    #[test]
    fn test_get_statusline_by_id_nonexistent() {
        let db = setup_db();
        assert!(db.get_statusline_by_id(99999).unwrap().is_none());
    }

    #[test]
    fn test_update_statusline() {
        let db = setup_db();
        let mut sl = db
            .create_statusline(&create_test_statusline_request("upd-sl"))
            .unwrap();
        sl.description = Some("Updated".to_string());
        sl.run_command = Some("new cmd".to_string());

        let updated = db.update_statusline(&sl).unwrap();
        assert_eq!(updated.description, Some("Updated".to_string()));
        assert_eq!(updated.run_command, Some("new cmd".to_string()));
    }

    #[test]
    fn test_delete_statusline() {
        let db = setup_db();
        let sl = db
            .create_statusline(&create_test_statusline_request("del-sl"))
            .unwrap();
        db.delete_statusline(sl.id).unwrap();
        assert!(db.get_statusline_by_id(sl.id).unwrap().is_none());
    }

    #[test]
    fn test_get_all_statuslines() {
        let db = setup_db();
        db.create_statusline(&create_test_statusline_request("sl-a"))
            .unwrap();
        db.create_statusline(&create_test_statusline_request("sl-b"))
            .unwrap();
        let all = db.get_all_statuslines().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_set_and_get_active_statusline() {
        let db = setup_db();
        let sl1 = db
            .create_statusline(&create_test_statusline_request("active-1"))
            .unwrap();
        let sl2 = db
            .create_statusline(&create_test_statusline_request("active-2"))
            .unwrap();

        // No active statusline initially
        assert!(db.get_active_statusline().unwrap().is_none());

        db.set_active_statusline(sl1.id).unwrap();
        let active = db.get_active_statusline().unwrap().unwrap();
        assert_eq!(active.id, sl1.id);

        // Switch active
        db.set_active_statusline(sl2.id).unwrap();
        let active = db.get_active_statusline().unwrap().unwrap();
        assert_eq!(active.id, sl2.id);

        db.deactivate_all_statuslines().unwrap();
        assert!(db.get_active_statusline().unwrap().is_none());
    }

    // =========================================================================
    // Spinner Verb tests
    // =========================================================================

    #[test]
    fn test_create_and_get_spinner_verb() {
        let db = setup_db();
        let verb = db.create_spinner_verb("Pondering").unwrap();
        assert_eq!(verb.verb, "Pondering");
        assert!(verb.is_enabled);

        let fetched = db.get_spinner_verb_by_id(verb.id).unwrap();
        assert_eq!(fetched.verb, "Pondering");
    }

    #[test]
    fn test_update_spinner_verb() {
        let db = setup_db();
        let verb = db.create_spinner_verb("Old").unwrap();
        let updated = db.update_spinner_verb(verb.id, "New", false).unwrap();
        assert_eq!(updated.verb, "New");
        assert!(!updated.is_enabled);
    }

    #[test]
    fn test_delete_spinner_verb() {
        let db = setup_db();
        let verb = db.create_spinner_verb("Temp").unwrap();
        db.delete_spinner_verb(verb.id).unwrap();
        assert!(db.get_spinner_verb_by_id(verb.id).is_err());
    }

    #[test]
    fn test_get_all_spinner_verbs() {
        let db = setup_db();
        db.create_spinner_verb("V1").unwrap();
        db.create_spinner_verb("V2").unwrap();
        db.create_spinner_verb("V3").unwrap();
        let all = db.get_all_spinner_verbs().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_spinner_verb_mode() {
        let db = setup_db();
        // Default should be append
        let mode = db.get_spinner_verb_mode().unwrap();
        assert_eq!(mode, "append");

        db.set_spinner_verb_mode("replace").unwrap();
        let mode = db.get_spinner_verb_mode().unwrap();
        assert_eq!(mode, "replace");
    }

    #[test]
    fn test_reorder_spinner_verbs() {
        let db = setup_db();
        let v1 = db.create_spinner_verb("A").unwrap();
        let v2 = db.create_spinner_verb("B").unwrap();
        let v3 = db.create_spinner_verb("C").unwrap();

        // Reverse order
        db.reorder_spinner_verbs(&[v3.id, v2.id, v1.id]).unwrap();

        let all = db.get_all_spinner_verbs().unwrap();
        assert_eq!(all[0].verb, "C");
        assert_eq!(all[1].verb, "B");
        assert_eq!(all[2].verb, "A");
    }

    // =========================================================================
    // Cascade delete tests
    // =========================================================================

    #[test]
    fn test_delete_mcp_cascades_to_global_mcps() {
        let db = setup_db();
        let mcp = db.create_mcp(&create_test_mcp_request("cascade")).unwrap();
        db.add_global_mcp(mcp.id).unwrap();
        db.add_gateway_mcp(mcp.id).unwrap();

        db.delete_mcp(mcp.id).unwrap();

        let global_count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM global_mcps", [], |row| row.get(0))
            .unwrap();
        assert_eq!(global_count, 0);

        let gateway_count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM gateway_mcps", [], |row| row.get(0))
            .unwrap();
        assert_eq!(gateway_count, 0);
    }

    #[test]
    fn test_delete_skill_cascades_to_global_skills() {
        let db = setup_db();
        let skill = db.create_skill(&create_test_skill_request("cs")).unwrap();
        db.add_global_skill(skill.id).unwrap();

        db.delete_skill(skill.id).unwrap();

        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM global_skills", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_delete_hook_cascades_to_global_hooks() {
        let db = setup_db();
        let hook = db.create_hook(&create_test_hook_request("ch")).unwrap();
        db.add_global_hook(hook.id).unwrap();

        db.delete_hook(hook.id).unwrap();

        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM global_hooks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // MCP with complex fields tests
    // =========================================================================

    #[test]
    fn test_mcp_with_env_and_headers() {
        let db = setup_db();
        let mut env = std::collections::HashMap::new();
        env.insert("KEY".to_string(), "val".to_string());
        let mut headers = std::collections::HashMap::new();
        headers.insert("Auth".to_string(), "Bearer xxx".to_string());

        let req = CreateMcpRequest {
            name: "http-mcp".to_string(),
            description: None,
            mcp_type: "http".to_string(),
            command: None,
            args: None,
            url: Some("https://example.com".to_string()),
            headers: Some(headers.clone()),
            env: Some(env.clone()),
            icon: None,
            tags: Some(vec!["web".to_string()]),
        };

        let mcp = db.create_mcp(&req).unwrap();
        let fetched = db.get_mcp_by_id(mcp.id).unwrap().unwrap();

        assert_eq!(fetched.url, Some("https://example.com".to_string()));
        assert_eq!(fetched.env.unwrap().get("KEY").unwrap(), "val");
        assert_eq!(fetched.headers.unwrap().get("Auth").unwrap(), "Bearer xxx");
        assert_eq!(fetched.tags.unwrap(), vec!["web".to_string()]);
    }

    // =========================================================================
    // Skill with disable_model_invocation test
    // =========================================================================

    #[test]
    fn test_skill_with_disable_model_invocation() {
        let db = setup_db();
        let req = CreateSkillRequest {
            name: "no-invoke".to_string(),
            description: None,
            content: "c".to_string(),
            allowed_tools: None,
            model: Some("sonnet".to_string()),
            disable_model_invocation: Some(true),
            tags: None,
            context: None,
            agent: None,
            hooks: None,
            paths: None,
            shell: None,
            once: None,
            effort: None,
        };

        let skill = db.create_skill(&req).unwrap();
        assert!(skill.disable_model_invocation);
        assert_eq!(skill.model, Some("sonnet".to_string()));
    }

    // =========================================================================
    // Profile tests (via raw SQL since no convenience methods)
    // =========================================================================

    #[test]
    fn test_profiles_table_exists() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO profiles (name, description) VALUES ('Dev', 'Development profile')",
                [],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let name: String = db
            .conn()
            .query_row("SELECT name FROM profiles WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "Dev");
    }

    #[test]
    fn test_profile_items() {
        let db = setup_db();
        db.conn()
            .execute("INSERT INTO profiles (name) VALUES ('Test')", [])
            .unwrap();
        let profile_id = db.conn().last_insert_rowid();

        let mcp = db.create_mcp(&create_test_mcp_request("pi-mcp")).unwrap();

        db.conn()
            .execute(
                "INSERT INTO profile_items (profile_id, item_type, item_id) VALUES (?, 'mcp', ?)",
                rusqlite::params![profile_id, mcp.id],
            )
            .unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM profile_items WHERE profile_id = ?",
                [profile_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Unique constraint: same item cannot be added twice
        let result = db.conn().execute(
            "INSERT INTO profile_items (profile_id, item_type, item_id) VALUES (?, 'mcp', ?)",
            rusqlite::params![profile_id, mcp.id],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_profile_cascades_items() {
        let db = setup_db();
        db.conn()
            .execute("INSERT INTO profiles (name) VALUES ('Del')", [])
            .unwrap();
        let profile_id = db.conn().last_insert_rowid();

        let mcp = db.create_mcp(&create_test_mcp_request("pi2")).unwrap();
        db.conn()
            .execute(
                "INSERT INTO profile_items (profile_id, item_type, item_id) VALUES (?, 'mcp', ?)",
                rusqlite::params![profile_id, mcp.id],
            )
            .unwrap();

        db.conn()
            .execute("DELETE FROM profiles WHERE id = ?", [profile_id])
            .unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM profile_items WHERE profile_id = ?",
                [profile_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Docker hosts table seeding test
    // =========================================================================

    #[test]
    fn test_docker_hosts_default_seeded() {
        let db = setup_db();
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM docker_hosts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let name: String = db
            .conn()
            .query_row(
                "SELECT name FROM docker_hosts WHERE is_default = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "Local Docker");
    }

    // =========================================================================
    // App settings default values
    // =========================================================================

    #[test]
    fn test_default_gateway_settings() {
        let db = setup_db();
        assert_eq!(db.get_setting("gateway_enabled"), Some("false".to_string()));
        assert_eq!(db.get_setting("gateway_port"), Some("23848".to_string()));
        assert_eq!(
            db.get_setting("gateway_auto_start"),
            Some("false".to_string())
        );
    }

    #[test]
    fn test_set_setting_overwrite() {
        let db = setup_db();
        // Overwrite a default setting
        db.set_setting("gateway_port", "9999").unwrap();
        assert_eq!(db.get_setting("gateway_port"), Some("9999".to_string()));
    }

    // =========================================================================
    // Gateway MCP additional edge cases
    // =========================================================================

    #[test]
    fn test_is_mcp_in_gateway_false() {
        let db = setup_db();
        let mcp = db.create_mcp(&create_test_mcp_request("not-gw")).unwrap();
        assert!(!db.is_mcp_in_gateway(mcp.id).unwrap());
    }

    #[test]
    fn test_get_enabled_gateway_mcps_mixed() {
        let db = setup_db();
        let mcp1 = db.create_mcp(&create_test_mcp_request("gw-en")).unwrap();
        let mcp2 = db.create_mcp(&create_test_mcp_request("gw-dis")).unwrap();

        db.add_gateway_mcp(mcp1.id).unwrap();
        db.add_gateway_mcp(mcp2.id).unwrap();

        // Disable one
        let gateways = db.get_gateway_mcps().unwrap();
        let gw2 = gateways.iter().find(|g| g.mcp.name == "gw-dis").unwrap();
        db.toggle_gateway_mcp(gw2.id, false).unwrap();

        let enabled = db.get_enabled_gateway_mcps().unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].mcp.name, "gw-en");
    }

    // =========================================================================
    // MCP update edge cases
    // =========================================================================

    #[test]
    fn test_update_mcp_with_complex_fields() {
        let db = setup_db();
        let req = CreateMcpRequest {
            name: "complex".to_string(),
            description: Some("Desc".to_string()),
            mcp_type: "sse".to_string(),
            command: None,
            args: None,
            url: Some("https://example.com/sse".to_string()),
            headers: Some({
                let mut m = std::collections::HashMap::new();
                m.insert("X-Key".to_string(), "val".to_string());
                m
            }),
            env: Some({
                let mut m = std::collections::HashMap::new();
                m.insert("API_KEY".to_string(), "secret".to_string());
                m
            }),
            icon: Some("🔌".to_string()),
            tags: Some(vec!["web".to_string(), "api".to_string()]),
        };

        let mut mcp = db.create_mcp(&req).unwrap();

        // Update all fields
        mcp.name = "complex-updated".to_string();
        mcp.url = Some("https://new.example.com".to_string());
        mcp.tags = Some(vec!["updated".to_string()]);

        let updated = db.update_mcp(&mcp).unwrap();
        assert_eq!(updated.name, "complex-updated");
        assert_eq!(updated.url, Some("https://new.example.com".to_string()));
        assert_eq!(updated.tags, Some(vec!["updated".to_string()]));
        assert_eq!(
            updated.env.as_ref().unwrap().get("API_KEY").unwrap(),
            "secret"
        );
    }

    // =========================================================================
    // Skill with all fields
    // =========================================================================

    #[test]
    fn test_create_skill_with_all_fields() {
        let db = setup_db();
        let req = CreateSkillRequest {
            name: "full-skill".to_string(),
            description: Some("Full skill".to_string()),
            content: "Full content".to_string(),
            allowed_tools: Some(vec!["Read".to_string(), "Grep".to_string()]),
            model: Some("opus".to_string()),
            disable_model_invocation: Some(true),
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
            context: None,
            agent: None,
            hooks: None,
            paths: None,
            shell: None,
            once: None,
            effort: None,
        };

        let skill = db.create_skill(&req).unwrap();
        assert_eq!(skill.name, "full-skill");
        assert_eq!(skill.model, Some("opus".to_string()));
        assert!(skill.disable_model_invocation);
        assert_eq!(
            skill.allowed_tools,
            Some(vec!["Read".to_string(), "Grep".to_string()])
        );
        assert_eq!(
            skill.tags,
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        );
    }

    // =========================================================================
    // SubAgent with all fields
    // =========================================================================

    #[test]
    fn test_create_subagent_with_all_fields() {
        let db = setup_db();
        let req = CreateSubAgentRequest {
            name: "full-agent".to_string(),
            description: "Full agent".to_string(),
            content: "Agent content".to_string(),
            tools: Some(vec!["Read".to_string(), "Write".to_string()]),
            model: Some("opus".to_string()),
            permission_mode: Some("bypassPermissions".to_string()),
            skills: Some(vec!["lint".to_string()]),
            tags: Some(vec!["review".to_string()]),
            disallowed_tools: None,
            max_turns: None,
            memory: None,
            background: None,
            effort: None,
            isolation: None,
            hooks: None,
            mcp_servers: None,
            initial_prompt: None,
        };

        let agent = db.create_subagent(&req).unwrap();
        assert_eq!(agent.name, "full-agent");
        assert_eq!(agent.permission_mode, Some("bypassPermissions".to_string()));
        assert_eq!(agent.skills, Some(vec!["lint".to_string()]));
        assert_eq!(agent.tags, Some(vec!["review".to_string()]));
    }

    // =========================================================================
    // Hook with prompt type
    // =========================================================================

    #[test]
    fn test_create_prompt_hook() {
        let db = setup_db();
        let req = CreateHookRequest {
            name: "prompt-h".to_string(),
            description: Some("A prompt hook".to_string()),
            event_type: "Notification".to_string(),
            matcher: None,
            hook_type: "prompt".to_string(),
            command: None,
            prompt: Some("Please confirm".to_string()),
            timeout: None,
            tags: Some(vec!["safety".to_string()]),
            url: None,
            headers: None,
            allowed_env_vars: None,
            if_condition: None,
            status_message: None,
            once: None,
            async_mode: None,
            shell: None,
        };

        let hook = db.create_hook(&req).unwrap();
        assert_eq!(hook.hook_type, "prompt");
        assert_eq!(hook.prompt, Some("Please confirm".to_string()));
        assert!(hook.command.is_none());
        assert!(hook.timeout.is_none());
        assert_eq!(hook.tags, Some(vec!["safety".to_string()]));
    }

    // =========================================================================
    // Spinner verb auto-incrementing display_order
    // =========================================================================

    #[test]
    fn test_spinner_verb_auto_display_order() {
        let db = setup_db();
        let v1 = db.create_spinner_verb("First").unwrap();
        let v2 = db.create_spinner_verb("Second").unwrap();
        let v3 = db.create_spinner_verb("Third").unwrap();

        assert_eq!(v1.display_order, 0);
        assert_eq!(v2.display_order, 1);
        assert_eq!(v3.display_order, 2);
    }

    // =========================================================================
    // StatusLine raw command type
    // =========================================================================

    #[test]
    fn test_create_statusline_raw_type() {
        let db = setup_db();
        let req = CreateStatusLineRequest {
            name: "raw-sl".to_string(),
            description: None,
            statusline_type: "raw".to_string(),
            package_name: None,
            install_command: None,
            run_command: None,
            raw_command: Some("echo 'model: $MODEL'".to_string()),
            padding: None,
            segments_json: None,
            generated_script: Some("#!/bin/bash\necho 'model'".to_string()),
            icon: None,
            author: None,
            homepage_url: None,
            tags: None,
        };

        let sl = db.create_statusline(&req).unwrap();
        assert_eq!(sl.statusline_type, "raw");
        assert_eq!(sl.raw_command, Some("echo 'model: $MODEL'".to_string()));
        assert!(sl.generated_script.is_some());
        assert_eq!(sl.padding, 0); // default
    }

    // =========================================================================
    // Containers table exists and constraints work
    // =========================================================================

    #[test]
    fn test_containers_table_exists() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO containers (name, container_type, docker_host_id, image) VALUES ('test-c', 'docker', 1, 'alpine')",
                [],
            )
            .unwrap();
        let id = db.conn().last_insert_rowid();

        let name: String = db
            .conn()
            .query_row("SELECT name FROM containers WHERE id = ?", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "test-c");
    }

    #[test]
    fn test_project_containers_table() {
        let db = setup_db();

        // Create project and container
        db.conn()
            .execute(
                "INSERT INTO projects (name, path) VALUES ('proj', '/tmp/proj')",
                [],
            )
            .unwrap();
        let proj_id = db.conn().last_insert_rowid();

        db.conn()
            .execute(
                "INSERT INTO containers (name, container_type, docker_host_id) VALUES ('c1', 'docker', 1)",
                [],
            )
            .unwrap();
        let container_id = db.conn().last_insert_rowid();

        // Assign container to project
        db.conn()
            .execute(
                "INSERT INTO project_containers (project_id, container_id) VALUES (?, ?)",
                [proj_id, container_id],
            )
            .unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_containers WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    // =========================================================================
    // Permission templates table
    // =========================================================================

    #[test]
    fn test_permission_templates_table_exists() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO permission_templates (name, category, rule) VALUES ('Allow Read', 'allow', 'Read')",
                [],
            )
            .unwrap();

        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM permission_templates", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    // =========================================================================
    // Assign MCP to project idempotent
    // =========================================================================

    #[test]
    fn test_assign_mcp_to_project_idempotent() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO projects (name, path) VALUES ('p', '/tmp/p')",
                [],
            )
            .unwrap();
        let proj_id = db.conn().last_insert_rowid();
        let mcp = db.create_mcp(&create_test_mcp_request("idem")).unwrap();

        db.assign_mcp_to_project(proj_id, mcp.id).unwrap();
        db.assign_mcp_to_project(proj_id, mcp.id).unwrap(); // Should not fail

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_mcps WHERE project_id = ? AND mcp_id = ?",
                [proj_id, mcp.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    // =========================================================================
    // Delete non-existent items (should succeed silently)
    // =========================================================================

    #[test]
    fn test_delete_nonexistent_mcp() {
        let db = setup_db();
        // Deleting non-existent id should not error
        db.delete_mcp(99999).unwrap();
    }

    #[test]
    fn test_delete_nonexistent_skill() {
        let db = setup_db();
        db.delete_skill(99999).unwrap();
    }

    #[test]
    fn test_delete_nonexistent_subagent() {
        let db = setup_db();
        db.delete_subagent(99999).unwrap();
    }

    #[test]
    fn test_delete_nonexistent_hook() {
        let db = setup_db();
        db.delete_hook(99999).unwrap();
    }

    #[test]
    fn test_delete_nonexistent_statusline() {
        let db = setup_db();
        db.delete_statusline(99999).unwrap();
    }

    #[test]
    fn test_delete_nonexistent_spinner_verb() {
        let db = setup_db();
        db.delete_spinner_verb(99999).unwrap();
    }

    // =========================================================================
    // Remove global items that don't exist
    // =========================================================================

    #[test]
    fn test_remove_global_mcp_nonexistent() {
        let db = setup_db();
        db.remove_global_mcp(99999).unwrap();
    }

    #[test]
    fn test_remove_global_skill_nonexistent() {
        let db = setup_db();
        db.remove_global_skill(99999).unwrap();
    }

    #[test]
    fn test_remove_global_subagent_nonexistent() {
        let db = setup_db();
        db.remove_global_subagent(99999).unwrap();
    }

    #[test]
    fn test_remove_global_hook_nonexistent() {
        let db = setup_db();
        db.remove_global_hook(99999).unwrap();
    }

    #[test]
    fn test_remove_gateway_mcp_nonexistent() {
        let db = setup_db();
        db.remove_gateway_mcp(99999).unwrap();
    }

    // =========================================================================
    // Spinner verb mode edge cases
    // =========================================================================

    #[test]
    fn test_set_spinner_verb_mode_idempotent() {
        let db = setup_db();
        db.set_spinner_verb_mode("replace").unwrap();
        db.set_spinner_verb_mode("replace").unwrap();
        assert_eq!(db.get_spinner_verb_mode().unwrap(), "replace");
    }

    // =========================================================================
    // Statusline multiple active constraint
    // =========================================================================

    #[test]
    fn test_set_active_statusline_only_one() {
        let db = setup_db();
        let sl1 = db
            .create_statusline(&create_test_statusline_request("multi-1"))
            .unwrap();
        let sl2 = db
            .create_statusline(&create_test_statusline_request("multi-2"))
            .unwrap();
        let sl3 = db
            .create_statusline(&create_test_statusline_request("multi-3"))
            .unwrap();

        db.set_active_statusline(sl1.id).unwrap();
        db.set_active_statusline(sl2.id).unwrap();
        db.set_active_statusline(sl3.id).unwrap();

        let active_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM statuslines WHERE is_active = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(active_count, 1);

        let active = db.get_active_statusline().unwrap().unwrap();
        assert_eq!(active.id, sl3.id);
    }

    // =========================================================================
    // Skill files table
    // =========================================================================

    #[test]
    fn test_skill_files_table() {
        let db = setup_db();
        let skill = db
            .create_skill(&create_test_skill_request("sf-skill"))
            .unwrap();

        db.conn().execute(
            "INSERT INTO skill_files (skill_id, file_type, name, content) VALUES (?, 'reference', 'docs.md', '# Docs')",
            [skill.id],
        ).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM skill_files WHERE skill_id = ?",
                [skill.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_delete_skill_cascades_to_skill_files() {
        let db = setup_db();
        let skill = db
            .create_skill(&create_test_skill_request("sf-del"))
            .unwrap();

        db.conn().execute(
            "INSERT INTO skill_files (skill_id, file_type, name, content) VALUES (?, 'asset', 'schema.json', '{}')",
            [skill.id],
        ).unwrap();

        db.delete_skill(skill.id).unwrap();

        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM skill_files WHERE skill_id = ?",
                [skill.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // =========================================================================
    // Delete project cascades to project_mcps, project_skills, etc.
    // =========================================================================

    #[test]
    fn test_delete_project_cascades_assignments() {
        let db = setup_db();
        db.conn()
            .execute(
                "INSERT INTO projects (name, path) VALUES ('cascade-proj', '/tmp/cascade')",
                [],
            )
            .unwrap();
        let proj_id = db.conn().last_insert_rowid();

        let mcp = db
            .create_mcp(&create_test_mcp_request("cascade-m"))
            .unwrap();
        let skill = db
            .create_skill(&create_test_skill_request("cascade-s"))
            .unwrap();

        db.assign_mcp_to_project(proj_id, mcp.id).unwrap();
        db.conn()
            .execute(
                "INSERT INTO project_skills (project_id, skill_id) VALUES (?, ?)",
                rusqlite::params![proj_id, skill.id],
            )
            .unwrap();

        // Delete project
        db.conn()
            .execute("DELETE FROM projects WHERE id = ?", [proj_id])
            .unwrap();

        let mcp_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_mcps WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();
        let skill_count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM project_skills WHERE project_id = ?",
                [proj_id],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(mcp_count, 0);
        assert_eq!(skill_count, 0);
    }

    // =========================================================================
    // Spinner verb reorder with empty list
    // =========================================================================

    #[test]
    fn test_reorder_spinner_verbs_empty() {
        let db = setup_db();
        db.reorder_spinner_verbs(&[]).unwrap();
    }
}
