use anyhow::Result;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Top-level response for the insights report HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsReportInfo {
    pub file_path: String,
    pub exists: bool,
    pub html_content: Option<String>,
}

/// Top-level response for all session facets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFacetsInfo {
    pub dir_path: String,
    pub exists: bool,
    pub facets: Vec<SessionFacet>,
}

/// A single session's quality facets.
/// JSON files use snake_case keys; we serialize as camelCase for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFacet {
    #[serde(alias = "session_id")]
    pub session_id: String,

    #[serde(alias = "underlying_goal", default)]
    pub underlying_goal: String,

    #[serde(alias = "goal_categories", default)]
    pub goal_categories: HashMap<String, u64>,

    #[serde(default)]
    pub outcome: String,

    #[serde(alias = "user_satisfaction_counts", default)]
    pub user_satisfaction_counts: HashMap<String, u64>,

    #[serde(alias = "claude_helpfulness", default)]
    pub claude_helpfulness: String,

    #[serde(alias = "session_type", default)]
    pub session_type: String,

    #[serde(alias = "friction_counts", default)]
    pub friction_counts: HashMap<String, u64>,

    #[serde(alias = "friction_detail", default)]
    pub friction_detail: String,

    #[serde(alias = "primary_success", default)]
    pub primary_success: String,

    #[serde(alias = "brief_summary", default)]
    pub brief_summary: String,
}

/// Get the path to ~/.claude/usage-data/
fn usage_data_dir() -> PathBuf {
    if let Some(base) = directories::BaseDirs::new() {
        base.home_dir().join(".claude").join("usage-data")
    } else {
        PathBuf::from("~/.claude/usage-data")
    }
}

/// Read the insights report HTML from the default location
pub fn read_insights_report() -> Result<InsightsReportInfo> {
    let dir = usage_data_dir();
    let path = dir.join("report.html");
    read_insights_report_from_path(&path)
}

/// Read the insights report HTML from a given path (testable variant)
pub fn read_insights_report_from_path(path: &Path) -> Result<InsightsReportInfo> {
    let file_path = path.to_string_lossy().to_string();

    if !path.exists() {
        return Ok(InsightsReportInfo {
            file_path,
            exists: false,
            html_content: None,
        });
    }

    let content = std::fs::read_to_string(path)?;
    Ok(InsightsReportInfo {
        file_path,
        exists: true,
        html_content: Some(content),
    })
}

/// Read all session facets from the default location
pub fn read_all_facets() -> Result<SessionFacetsInfo> {
    let dir = usage_data_dir();
    let facets_dir = dir.join("facets");
    read_all_facets_from_dir(&facets_dir)
}

/// Read all session facets from a given directory (testable variant)
pub fn read_all_facets_from_dir(dir: &Path) -> Result<SessionFacetsInfo> {
    let dir_path = dir.to_string_lossy().to_string();

    if !dir.exists() {
        return Ok(SessionFacetsInfo {
            dir_path,
            exists: false,
            facets: Vec::new(),
        });
    }

    let mut facets = Vec::new();

    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<SessionFacet>(&content) {
                Ok(facet) => facets.push(facet),
                Err(e) => {
                    warn!(
                        "[Insights] Skipping unparseable facet file {}: {}",
                        path.display(),
                        e
                    );
                }
            },
            Err(e) => {
                warn!(
                    "[Insights] Skipping unreadable facet file {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    facets.sort_by(|a, b| a.session_id.cmp(&b.session_id));

    Ok(SessionFacetsInfo {
        dir_path,
        exists: true,
        facets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonexistent_report_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("report.html");

        let info = read_insights_report_from_path(&path).unwrap();
        assert!(!info.exists);
        assert!(info.html_content.is_none());
    }

    #[test]
    fn test_valid_report_reads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("report.html");
        std::fs::write(&path, "<html><body>Test</body></html>").unwrap();

        let info = read_insights_report_from_path(&path).unwrap();
        assert!(info.exists);
        assert_eq!(info.html_content.unwrap(), "<html><body>Test</body></html>");
    }

    #[test]
    fn test_nonexistent_facets_dir_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let facets_dir = dir.path().join("facets");

        let info = read_all_facets_from_dir(&facets_dir).unwrap();
        assert!(!info.exists);
        assert!(info.facets.is_empty());
    }

    #[test]
    fn test_valid_facet_parses() {
        let dir = tempfile::tempdir().unwrap();
        let facets_dir = dir.path().join("facets");
        std::fs::create_dir(&facets_dir).unwrap();

        std::fs::write(
            facets_dir.join("abc-123.json"),
            r#"{
                "session_id": "abc-123",
                "underlying_goal": "Fix a bug",
                "goal_categories": { "bug_fix": 1 },
                "outcome": "fully_achieved",
                "user_satisfaction_counts": { "satisfied": 3 },
                "claude_helpfulness": "essential",
                "session_type": "single_task",
                "friction_counts": { "tool_failures": 1 },
                "friction_detail": "One tool call failed",
                "primary_success": "true",
                "brief_summary": "Fixed the login bug"
            }"#,
        )
        .unwrap();

        let info = read_all_facets_from_dir(&facets_dir).unwrap();
        assert!(info.exists);
        assert_eq!(info.facets.len(), 1);

        let f = &info.facets[0];
        assert_eq!(f.session_id, "abc-123");
        assert_eq!(f.underlying_goal, "Fix a bug");
        assert_eq!(f.outcome, "fully_achieved");
        assert_eq!(f.claude_helpfulness, "essential");
        assert_eq!(f.brief_summary, "Fixed the login bug");
        assert_eq!(f.goal_categories.get("bug_fix"), Some(&1));
        assert_eq!(f.friction_counts.get("tool_failures"), Some(&1));
    }

    #[test]
    fn test_unparseable_facets_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let facets_dir = dir.path().join("facets");
        std::fs::create_dir(&facets_dir).unwrap();

        // Valid facet
        std::fs::write(
            facets_dir.join("good.json"),
            r#"{ "session_id": "good-1", "outcome": "fully_achieved" }"#,
        )
        .unwrap();

        // Invalid JSON
        std::fs::write(facets_dir.join("bad.json"), "not json").unwrap();

        // Non-JSON file (should be skipped)
        std::fs::write(facets_dir.join("readme.txt"), "ignore me").unwrap();

        let info = read_all_facets_from_dir(&facets_dir).unwrap();
        assert!(info.exists);
        assert_eq!(info.facets.len(), 1);
        assert_eq!(info.facets[0].session_id, "good-1");
    }

    #[test]
    fn test_facets_sorted_by_session_id() {
        let dir = tempfile::tempdir().unwrap();
        let facets_dir = dir.path().join("facets");
        std::fs::create_dir(&facets_dir).unwrap();

        std::fs::write(facets_dir.join("z.json"), r#"{ "session_id": "zzz" }"#).unwrap();
        std::fs::write(facets_dir.join("a.json"), r#"{ "session_id": "aaa" }"#).unwrap();

        let info = read_all_facets_from_dir(&facets_dir).unwrap();
        assert_eq!(info.facets[0].session_id, "aaa");
        assert_eq!(info.facets[1].session_id, "zzz");
    }
}
