use crate::db::Database;
use crate::services::scanner as scanner_service;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn scan_claude_directory(db: State<'_, Mutex<Database>>) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;

    // Scan claude.json for projects and MCPs
    let claude_json_count = scanner_service::scan_claude_json(&db).map_err(|e| e.to_string())?;

    // Scan plugins directory
    let plugin_count = scanner_service::scan_plugins(&db).map_err(|e| e.to_string())?;

    Ok(claude_json_count + plugin_count)
}
