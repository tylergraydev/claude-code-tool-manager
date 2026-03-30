use crate::services::session_explorer::{self, ProjectListInfo, SessionDetail, SessionListInfo};
use log::info;

/// List all discovered projects with session counts and aggregates
#[tauri::command]
pub async fn get_session_projects() -> Result<ProjectListInfo, String> {
    info!("[Sessions] Listing session projects");
    tokio::task::spawn_blocking(|| {
        session_explorer::list_projects().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// List all sessions for a given project folder
#[tauri::command]
pub async fn get_project_sessions(project_folder: String) -> Result<SessionListInfo, String> {
    info!(
        "[Sessions] Listing sessions for project: {}",
        project_folder
    );
    tokio::task::spawn_blocking(move || {
        session_explorer::list_sessions(&project_folder).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Get full session detail (transcript) for a specific session
#[tauri::command]
pub async fn get_session_detail(
    project_folder: String,
    session_id: String,
) -> Result<SessionDetail, String> {
    info!(
        "[Sessions] Getting session detail: {}/{}",
        project_folder, session_id
    );
    tokio::task::spawn_blocking(move || {
        session_explorer::get_session_detail(&project_folder, &session_id)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_session_projects_returns_result() {
        // Filesystem-dependent; just verify no panic.
        let _ = get_session_projects();
    }

    #[test]
    fn test_get_project_sessions_nonexistent_folder() {
        let result = get_project_sessions("/nonexistent/path/abc123".to_string());
        // Should return an error or empty result, not panic.
        let _ = result;
    }

    #[test]
    fn test_get_session_detail_nonexistent() {
        let result = get_session_detail(
            "/nonexistent/path".to_string(),
            "fake-session-id".to_string(),
        );
        // Should return an error, not panic.
        let _ = result;
    }
}
