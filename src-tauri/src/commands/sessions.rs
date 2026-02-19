use crate::services::session_explorer::{self, ProjectListInfo, SessionDetail, SessionListInfo};
use log::info;

/// List all discovered projects with session counts and aggregates
#[tauri::command]
pub fn get_session_projects() -> Result<ProjectListInfo, String> {
    info!("[Sessions] Listing session projects");
    session_explorer::list_projects().map_err(|e| e.to_string())
}

/// List all sessions for a given project folder
#[tauri::command]
pub fn get_project_sessions(project_folder: String) -> Result<SessionListInfo, String> {
    info!(
        "[Sessions] Listing sessions for project: {}",
        project_folder
    );
    session_explorer::list_sessions(&project_folder).map_err(|e| e.to_string())
}

/// Get full session detail (transcript) for a specific session
#[tauri::command]
pub fn get_session_detail(
    project_folder: String,
    session_id: String,
) -> Result<SessionDetail, String> {
    info!(
        "[Sessions] Getting session detail: {}/{}",
        project_folder, session_id
    );
    session_explorer::get_session_detail(&project_folder, &session_id).map_err(|e| e.to_string())
}
