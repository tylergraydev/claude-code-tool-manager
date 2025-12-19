use crate::db::Database;
use crate::services::config_parser;
use crate::utils::paths::get_claude_paths;
use anyhow::Result;
use rusqlite::params;
use walkdir::WalkDir;

pub async fn run_startup_scan(app: &tauri::AppHandle) -> Result<()> {
    let db = app.state::<std::sync::Mutex<Database>>();
    let db = db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

    let count = scan_and_import(&db)?;
    log::info!("Startup scan found {} MCPs", count);

    Ok(())
}

pub fn scan_and_import(db: &Database) -> Result<usize> {
    let paths = get_claude_paths()?;
    let mut count = 0;

    // Scan plugins/marketplaces directory
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
                                let args_json = mcp.args.as_ref().map(|a| serde_json::to_string(a).unwrap());
                                let headers_json = mcp.headers.as_ref().map(|h| serde_json::to_string(h).unwrap());
                                let env_json = mcp.env.as_ref().map(|e| serde_json::to_string(e).unwrap());

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
