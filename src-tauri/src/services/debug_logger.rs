use anyhow::Result;
use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

// Global state for debug logging
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
static LOG_FILE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

/// Check if debug logging is enabled
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::SeqCst)
}

/// Get the current log file path (if debug is enabled)
pub fn get_log_file_path() -> Option<PathBuf> {
    LOG_FILE_PATH.lock().ok()?.clone()
}

/// Enable debug logging, creating a new log file
pub fn enable_debug_logging(app_data_dir: &PathBuf) -> Result<PathBuf> {
    // Create logs directory
    let logs_dir = app_data_dir.join("logs");
    fs::create_dir_all(&logs_dir)?;

    // Generate timestamped filename
    let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
    let log_filename = format!("debug-{}.log", timestamp);
    let log_path = logs_dir.join(&log_filename);

    // Create and open log file
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Store file handle and path
    {
        let mut file_guard = LOG_FILE.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        *file_guard = Some(file);
    }
    {
        let mut path_guard = LOG_FILE_PATH.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        *path_guard = Some(log_path.clone());
    }

    // Enable debug mode
    DEBUG_ENABLED.store(true, Ordering::SeqCst);

    // Write header
    write_log("INFO", "debug", "Debug logging enabled")?;
    write_log("INFO", "debug", &format!("Log file: {}", log_path.display()))?;
    write_log("INFO", "debug", &format!("App version: {}", env!("CARGO_PKG_VERSION")))?;
    write_log("INFO", "debug", &format!("Platform: {}", std::env::consts::OS))?;

    Ok(log_path)
}

/// Disable debug logging
pub fn disable_debug_logging() -> Result<()> {
    if is_debug_enabled() {
        write_log("INFO", "debug", "Debug logging disabled")?;
    }

    DEBUG_ENABLED.store(false, Ordering::SeqCst);

    // Close file handle
    {
        let mut file_guard = LOG_FILE.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        if let Some(mut file) = file_guard.take() {
            let _ = file.flush();
        }
    }

    // Clear path (but keep it for reference until next enable)
    // We don't clear the path so users can still see where the last log was

    Ok(())
}

/// Write a log entry to the debug log file
pub fn write_log(level: &str, source: &str, message: &str) -> Result<()> {
    if !is_debug_enabled() {
        return Ok(());
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_line = format!("[{}] [{}] [{}] {}\n", timestamp, level.to_uppercase(), source, message);

    let mut file_guard = LOG_FILE.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
    if let Some(ref mut file) = *file_guard {
        file.write_all(log_line.as_bytes())?;
        file.flush()?;
    }

    Ok(())
}

/// Write a log entry with optional context (for structured data like JSON)
pub fn write_log_with_context(level: &str, source: &str, message: &str, context: Option<&str>) -> Result<()> {
    if !is_debug_enabled() {
        return Ok(());
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_line = match context {
        Some(ctx) => format!("[{}] [{}] [{}] {} | {}\n", timestamp, level.to_uppercase(), source, message, ctx),
        None => format!("[{}] [{}] [{}] {}\n", timestamp, level.to_uppercase(), source, message),
    };

    let mut file_guard = LOG_FILE.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
    if let Some(ref mut file) = *file_guard {
        file.write_all(log_line.as_bytes())?;
        file.flush()?;
    }

    Ok(())
}

/// Get the logs directory path
pub fn get_logs_dir(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("logs")
}

/// Get the path to the debug persistence flag file
fn get_debug_flag_path(app_data_dir: &PathBuf) -> PathBuf {
    app_data_dir.join("debug_enabled")
}

/// Persist debug mode setting to disk
pub fn persist_debug_enabled(app_data_dir: &PathBuf, enabled: bool) -> Result<()> {
    let flag_path = get_debug_flag_path(app_data_dir);
    if enabled {
        // Create the flag file
        File::create(&flag_path)?;
    } else {
        // Remove the flag file if it exists
        if flag_path.exists() {
            fs::remove_file(&flag_path)?;
        }
    }
    Ok(())
}

/// Check if debug mode was persisted (for startup)
pub fn is_debug_persisted(app_data_dir: &PathBuf) -> bool {
    get_debug_flag_path(app_data_dir).exists()
}

/// Initialize debug mode from persisted state (call early in startup)
pub fn init_from_persisted(app_data_dir: &PathBuf) -> Result<Option<PathBuf>> {
    if is_debug_persisted(app_data_dir) {
        let log_path = enable_debug_logging(app_data_dir)?;
        write_log("INFO", "debug", "Debug mode restored from persisted state")?;
        Ok(Some(log_path))
    } else {
        Ok(None)
    }
}

/// Convenience function to log from Rust code when debug is enabled
#[allow(dead_code)]
pub fn debug_log(message: &str) {
    let _ = write_log("DEBUG", "rust", message);
}

#[allow(dead_code)]
pub fn info_log(message: &str) {
    let _ = write_log("INFO", "rust", message);
}

#[allow(dead_code)]
pub fn warn_log(message: &str) {
    let _ = write_log("WARN", "rust", message);
}

#[allow(dead_code)]
pub fn error_log(message: &str) {
    let _ = write_log("ERROR", "rust", message);
}
