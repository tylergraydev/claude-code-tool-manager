use crate::db::Database;
use crate::services::scanner as scanner_service;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn scan_claude_directory(db: State<'_, Mutex<Database>>) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let count = scanner_service::scan_and_import(&db).map_err(|e| e.to_string())?;
    Ok(count)
}
