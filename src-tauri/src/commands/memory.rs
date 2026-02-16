use crate::services::memory_writer::{self, AllMemoryFiles, MemoryFileInfo, MemoryScope};
use log::info;
use std::path::Path;

fn parse_scope(scope: &str) -> Result<MemoryScope, String> {
    match scope {
        "user" => Ok(MemoryScope::User),
        "project" => Ok(MemoryScope::Project),
        "local" => Ok(MemoryScope::Local),
        _ => Err(format!("Invalid scope: {}", scope)),
    }
}

/// Get all memory files from all three scopes
#[tauri::command]
pub fn get_all_memory_files(project_path: Option<String>) -> Result<AllMemoryFiles, String> {
    info!(
        "[Memory] Reading all memory files (project={:?})",
        project_path
    );
    let pp = project_path.as_deref().map(Path::new);
    memory_writer::read_all_memory_files(pp).map_err(|e| e.to_string())
}

/// Get a single memory file by scope
#[tauri::command]
pub fn get_memory_file(
    scope: String,
    project_path: Option<String>,
) -> Result<MemoryFileInfo, String> {
    info!(
        "[Memory] Reading memory file for scope={} (project={:?})",
        scope, project_path
    );
    let ms = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    memory_writer::read_memory_file(&ms, pp).map_err(|e| e.to_string())
}

/// Save content to a memory file
#[tauri::command]
pub fn save_memory_file(
    scope: String,
    project_path: Option<String>,
    content: String,
) -> Result<MemoryFileInfo, String> {
    info!(
        "[Memory] Saving memory file for scope={} ({} bytes)",
        scope,
        content.len()
    );
    let ms = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    memory_writer::write_memory_file(&ms, pp, &content).map_err(|e| e.to_string())
}

/// Delete a memory file
#[tauri::command]
pub fn delete_memory_file(
    scope: String,
    project_path: Option<String>,
) -> Result<(), String> {
    info!("[Memory] Deleting memory file for scope={}", scope);
    let ms = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    memory_writer::delete_memory_file(&ms, pp).map_err(|e| e.to_string())
}

/// Create a new memory file with optional initial content
#[tauri::command]
pub fn create_memory_file(
    scope: String,
    project_path: Option<String>,
    content: Option<String>,
) -> Result<MemoryFileInfo, String> {
    info!("[Memory] Creating memory file for scope={}", scope);
    let ms = parse_scope(&scope)?;
    let pp = project_path.as_deref().map(Path::new);
    let initial_content = content.unwrap_or_default();
    memory_writer::write_memory_file(&ms, pp, &initial_content).map_err(|e| e.to_string())
}

/// Detect which project memory location variant is in use
#[tauri::command]
pub fn detect_project_memory_location(
    project_path: String,
) -> Result<(String, String), String> {
    info!(
        "[Memory] Detecting project memory location for {:?}",
        project_path
    );
    let path = Path::new(&project_path);
    let (resolved_path, variant) =
        memory_writer::detect_project_memory_location(path).map_err(|e| e.to_string())?;
    Ok((resolved_path.to_string_lossy().to_string(), variant))
}

/// Render markdown content to HTML
#[tauri::command]
pub fn render_markdown(content: String) -> Result<String, String> {
    memory_writer::render_markdown(&content).map_err(|e| e.to_string())
}
