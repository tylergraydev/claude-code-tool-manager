use crate::services::stats_cache::{self, StatsCacheInfo};
use log::info;

/// Get usage stats from ~/.claude/stats-cache.json
#[tauri::command]
pub fn get_usage_stats() -> Result<StatsCacheInfo, String> {
    info!("[Analytics] Reading usage stats");
    stats_cache::read_stats_cache().map_err(|e| e.to_string())
}
