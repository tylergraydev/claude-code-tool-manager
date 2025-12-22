use crate::services::debug_logger;
use tauri::Manager;

/// Enable debug mode and return the log file path
#[tauri::command]
pub fn enable_debug_mode(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let log_path = debug_logger::enable_debug_logging(&app_data_dir)
        .map_err(|e| format!("Failed to enable debug logging: {}", e))?;

    // Persist the debug state so it survives restarts
    debug_logger::persist_debug_enabled(&app_data_dir, true)
        .map_err(|e| format!("Failed to persist debug state: {}", e))?;

    Ok(log_path.to_string_lossy().to_string())
}

/// Disable debug mode
#[tauri::command]
pub fn disable_debug_mode(app: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    debug_logger::disable_debug_logging()
        .map_err(|e| format!("Failed to disable debug logging: {}", e))?;

    // Remove the persistence flag so debug mode doesn't restart on next launch
    debug_logger::persist_debug_enabled(&app_data_dir, false)
        .map_err(|e| format!("Failed to persist debug state: {}", e))?;

    Ok(())
}

/// Check if debug mode is enabled
#[tauri::command]
pub fn is_debug_mode_enabled() -> bool {
    debug_logger::is_debug_enabled()
}

/// Get the current debug log file path
#[tauri::command]
pub fn get_debug_log_path() -> Option<String> {
    debug_logger::get_log_file_path().map(|p| p.to_string_lossy().to_string())
}

/// Open the logs folder in the system file explorer
#[tauri::command]
pub fn open_logs_folder(app: tauri::AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let logs_dir = debug_logger::get_logs_dir(&app_data_dir);

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&logs_dir)
        .map_err(|e| format!("Failed to create logs directory: {}", e))?;

    // Open in file explorer using the shell plugin
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(logs_dir.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(logs_dir.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(logs_dir.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

/// Write a log entry from the frontend
#[tauri::command]
pub fn write_frontend_log(
    level: String,
    message: String,
    context: Option<String>,
) -> Result<(), String> {
    debug_logger::write_log_with_context(
        &level,
        "frontend",
        &message,
        context.as_deref(),
    )
    .map_err(|e| format!("Failed to write log: {}", e))
}

/// Write an invoke log entry from the frontend
#[tauri::command]
pub fn write_invoke_log(
    command: String,
    duration_ms: f64,
    success: bool,
    args: Option<String>,
    error: Option<String>,
) -> Result<(), String> {
    let message = if success {
        format!("{} ({:.1}ms)", command, duration_ms)
    } else {
        format!("FAILED {} ({:.1}ms): {}", command, duration_ms, error.unwrap_or_default())
    };

    let level = if success { "INFO" } else { "ERROR" };

    debug_logger::write_log_with_context(level, "invoke", &message, args.as_deref())
        .map_err(|e| format!("Failed to write log: {}", e))
}
