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

            -- Skills (Slash Commands and Agent Skills)
            CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                content TEXT NOT NULL,
                skill_type TEXT NOT NULL DEFAULT 'command' CHECK (skill_type IN ('command', 'skill')),
                allowed_tools TEXT,
                argument_hint TEXT,
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

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_mcps_type ON mcps(type);
            CREATE INDEX IF NOT EXISTS idx_mcps_source ON mcps(source);
            CREATE INDEX IF NOT EXISTS idx_project_mcps_project ON project_mcps(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_mcps_mcp ON project_mcps(mcp_id);
            CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
            CREATE INDEX IF NOT EXISTS idx_project_skills_project ON project_skills(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_subagents_project ON project_subagents(project_id);
            CREATE INDEX IF NOT EXISTS idx_repos_content_type ON repos(content_type);
            CREATE INDEX IF NOT EXISTS idx_repos_enabled ON repos(is_enabled);
            CREATE INDEX IF NOT EXISTS idx_repo_items_repo ON repo_items(repo_id);
            CREATE INDEX IF NOT EXISTS idx_repo_items_type ON repo_items(item_type);
            CREATE INDEX IF NOT EXISTS idx_repo_items_imported ON repo_items(is_imported);
            "#,
        )?;

        // Run migrations for existing databases
        self.run_schema_migrations()?;

        Ok(())
    }

    fn run_schema_migrations(&self) -> Result<()> {
        // Migration 1: Add new columns to skills table
        let has_skill_type: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('skills') WHERE name = 'skill_type'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !has_skill_type {
            self.conn.execute_batch(
                r#"
                ALTER TABLE skills ADD COLUMN skill_type TEXT NOT NULL DEFAULT 'command';
                ALTER TABLE skills ADD COLUMN allowed_tools TEXT;
                ALTER TABLE skills ADD COLUMN argument_hint TEXT;
                "#,
            )?;
        }

        // Migration 2: Rename agents tables to subagents
        let has_agents_table: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='agents'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

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

        Ok(())
    }
}

