use crate::services::managed_settings::{self, ManagedSettingsInfo};
use log::info;

/// Get managed settings (enterprise/admin deployed read-only config)
#[tauri::command]
pub fn get_managed_settings() -> Result<ManagedSettingsInfo, String> {
    info!("[ManagedSettings] Reading managed settings file");
    managed_settings::read_managed_settings().map_err(|e| e.to_string())
}
