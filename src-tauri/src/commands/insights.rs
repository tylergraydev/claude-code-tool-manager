use crate::services::insights::{self, InsightsReportInfo, SessionFacetsInfo};
use log::info;

/// Get the insights report HTML from ~/.claude/usage-data/report.html
#[tauri::command]
pub fn get_insights_report() -> Result<InsightsReportInfo, String> {
    info!("[Insights] Reading insights report");
    insights::read_insights_report().map_err(|e| e.to_string())
}

/// Get all session facets from ~/.claude/usage-data/facets/
#[tauri::command]
pub fn get_session_facets() -> Result<SessionFacetsInfo, String> {
    info!("[Insights] Reading session facets");
    insights::read_all_facets().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_insights_report_returns_result() {
        // Filesystem-dependent; just verify no panic.
        let _ = get_insights_report();
    }

    #[test]
    fn test_get_session_facets_returns_result() {
        let _ = get_session_facets();
    }
}
