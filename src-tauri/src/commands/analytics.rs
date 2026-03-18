use crate::services::stats_cache::{self, StatsCacheInfo};
use log::info;

/// Get usage stats from ~/.claude/stats-cache.json
#[tauri::command]
pub fn get_usage_stats() -> Result<StatsCacheInfo, String> {
    info!("[Analytics] Reading usage stats");
    stats_cache::read_stats_cache().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_usage_stats_returns_result() {
        // The function reads from the filesystem; in CI it may not exist.
        // We just verify the function is callable and returns a Result.
        let result = get_usage_stats();
        // Either Ok or Err is fine – we're testing that it doesn't panic.
        let _ = result;
    }
}
