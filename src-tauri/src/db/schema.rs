use anyhow::Result;
use rusqlite::{Connection, params};
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

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_mcps_type ON mcps(type);
            CREATE INDEX IF NOT EXISTS idx_mcps_source ON mcps(source);
            CREATE INDEX IF NOT EXISTS idx_project_mcps_project ON project_mcps(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_mcps_mcp ON project_mcps(mcp_id);
            CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
            "#,
        )?;

        Ok(())
    }
}

// Helper trait for JSON serialization in SQLite
pub trait JsonValue {
    fn to_json_string(&self) -> Option<String>;
}

impl<T: serde::Serialize> JsonValue for Option<T> {
    fn to_json_string(&self) -> Option<String> {
        self.as_ref().map(|v| serde_json::to_string(v).ok()).flatten()
    }
}
