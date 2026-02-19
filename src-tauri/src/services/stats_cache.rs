use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Top-level response with file metadata + optional parsed stats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsCacheInfo {
    pub file_path: String,
    pub exists: bool,
    pub stats: Option<StatsCache>,
}

/// The full stats-cache.json schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsCache {
    pub version: Option<u32>,
    pub last_computed_date: Option<String>,
    #[serde(default)]
    pub daily_activity: Vec<DailyActivity>,
    #[serde(default)]
    pub daily_model_tokens: Vec<DailyModelTokens>,
    #[serde(default)]
    pub model_usage: HashMap<String, ModelUsageDetail>,
    pub total_sessions: Option<u64>,
    pub total_messages: Option<u64>,
    pub longest_session: Option<LongestSession>,
    pub first_session_date: Option<String>,
    #[serde(default)]
    pub hour_counts: HashMap<String, u64>,
    pub total_speculation_time_saved_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyActivity {
    pub date: String,
    pub message_count: u64,
    pub session_count: u64,
    pub tool_call_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyModelTokens {
    pub date: String,
    #[serde(default)]
    pub tokens_by_model: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsageDetail {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub web_search_requests: u64,
    #[serde(default)]
    pub cost_usd: f64,
    #[serde(default)]
    pub context_window: u64,
    #[serde(default)]
    pub max_output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongestSession {
    pub session_id: String,
    pub duration: u64,
    pub message_count: u64,
    pub timestamp: String,
}

/// Get the path to ~/.claude/stats-cache.json
pub fn stats_cache_path() -> PathBuf {
    if let Some(base) = directories::BaseDirs::new() {
        base.home_dir().join(".claude").join("stats-cache.json")
    } else {
        PathBuf::from("~/.claude/stats-cache.json")
    }
}

/// Read stats cache from the default path
pub fn read_stats_cache() -> Result<StatsCacheInfo> {
    let path = stats_cache_path();
    read_stats_cache_from_path(&path)
}

/// Read stats cache from a given path (testable variant)
pub fn read_stats_cache_from_path(path: &Path) -> Result<StatsCacheInfo> {
    let file_path = path.to_string_lossy().to_string();

    if !path.exists() {
        return Ok(StatsCacheInfo {
            file_path,
            exists: false,
            stats: None,
        });
    }

    let content = std::fs::read_to_string(path)?;
    let stats: StatsCache = serde_json::from_str(&content)?;
    Ok(StatsCacheInfo {
        file_path,
        exists: true,
        stats: Some(stats),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonexistent_file_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stats-cache.json");

        let info = read_stats_cache_from_path(&path).unwrap();
        assert!(!info.exists);
        assert!(info.stats.is_none());
        assert!(!info.file_path.is_empty());
    }

    #[test]
    fn test_valid_file_parses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stats-cache.json");
        std::fs::write(
            &path,
            r#"{
                "version": 2,
                "lastComputedDate": "2026-02-16",
                "dailyActivity": [
                    { "date": "2026-02-16", "messageCount": 100, "sessionCount": 5, "toolCallCount": 30 }
                ],
                "dailyModelTokens": [
                    { "date": "2026-02-16", "tokensByModel": { "claude-opus-4-6": 50000 } }
                ],
                "modelUsage": {
                    "claude-opus-4-6": {
                        "inputTokens": 1000,
                        "outputTokens": 2000,
                        "cacheReadInputTokens": 5000,
                        "cacheCreationInputTokens": 3000,
                        "webSearchRequests": 0,
                        "costUSD": 0,
                        "contextWindow": 0,
                        "maxOutputTokens": 0
                    }
                },
                "totalSessions": 10,
                "totalMessages": 500,
                "longestSession": {
                    "sessionId": "abc-123",
                    "duration": 3600000,
                    "messageCount": 50,
                    "timestamp": "2026-02-16T10:00:00Z"
                },
                "firstSessionDate": "2026-01-01T00:00:00Z",
                "hourCounts": { "10": 5, "14": 3 },
                "totalSpeculationTimeSavedMs": 0
            }"#,
        )
        .unwrap();

        let info = read_stats_cache_from_path(&path).unwrap();
        assert!(info.exists);

        let s = info.stats.unwrap();
        assert_eq!(s.version, Some(2));
        assert_eq!(s.total_sessions, Some(10));
        assert_eq!(s.total_messages, Some(500));
        assert_eq!(s.daily_activity.len(), 1);
        assert_eq!(s.daily_activity[0].message_count, 100);
        assert_eq!(s.daily_model_tokens.len(), 1);
        assert!(s.model_usage.contains_key("claude-opus-4-6"));
        assert!(s.longest_session.is_some());
        assert_eq!(s.hour_counts.get("10"), Some(&5));
    }

    #[test]
    fn test_minimal_file_parses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stats-cache.json");
        std::fs::write(&path, "{}").unwrap();

        let info = read_stats_cache_from_path(&path).unwrap();
        assert!(info.exists);

        let s = info.stats.unwrap();
        assert!(s.version.is_none());
        assert!(s.total_sessions.is_none());
        assert!(s.daily_activity.is_empty());
        assert!(s.model_usage.is_empty());
    }

    #[test]
    fn test_path_function_returns_non_empty() {
        let path = stats_cache_path();
        assert!(!path.to_string_lossy().is_empty());
    }
}
