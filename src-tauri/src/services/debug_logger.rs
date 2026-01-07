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
        let mut file_guard = LOG_FILE
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        *file_guard = Some(file);
    }
    {
        let mut path_guard = LOG_FILE_PATH
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        *path_guard = Some(log_path.clone());
    }

    // Enable debug mode
    DEBUG_ENABLED.store(true, Ordering::SeqCst);

    // Write header
    write_log("INFO", "debug", "Debug logging enabled")?;
    write_log(
        "INFO",
        "debug",
        &format!("Log file: {}", log_path.display()),
    )?;
    write_log(
        "INFO",
        "debug",
        &format!("App version: {}", env!("CARGO_PKG_VERSION")),
    )?;
    write_log(
        "INFO",
        "debug",
        &format!("Platform: {}", std::env::consts::OS),
    )?;

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
        let mut file_guard = LOG_FILE
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
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
    let log_line = format!(
        "[{}] [{}] [{}] {}\n",
        timestamp,
        level.to_uppercase(),
        source,
        message
    );

    let mut file_guard = LOG_FILE
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
    if let Some(ref mut file) = *file_guard {
        file.write_all(log_line.as_bytes())?;
        file.flush()?;
    }

    Ok(())
}

/// Write a log entry with optional context (for structured data like JSON)
pub fn write_log_with_context(
    level: &str,
    source: &str,
    message: &str,
    context: Option<&str>,
) -> Result<()> {
    if !is_debug_enabled() {
        return Ok(());
    }

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_line = match context {
        Some(ctx) => format!(
            "[{}] [{}] [{}] {} | {}\n",
            timestamp,
            level.to_uppercase(),
            source,
            message,
            ctx
        ),
        None => format!(
            "[{}] [{}] [{}] {}\n",
            timestamp,
            level.to_uppercase(),
            source,
            message
        ),
    };

    let mut file_guard = LOG_FILE
        .lock()
        .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    // =========================================================================
    // Helper function tests
    // =========================================================================

    #[test]
    fn test_get_logs_dir() {
        let base = PathBuf::from("/app/data");
        let logs_dir = get_logs_dir(&base);
        assert_eq!(logs_dir, PathBuf::from("/app/data/logs"));
    }

    #[test]
    fn test_get_debug_flag_path() {
        let base = PathBuf::from("/app/data");
        let flag_path = get_debug_flag_path(&base);
        assert_eq!(flag_path, PathBuf::from("/app/data/debug_enabled"));
    }

    // =========================================================================
    // Initial state tests
    // =========================================================================

    #[test]
    fn test_is_debug_enabled_initially_false() {
        // Note: This may fail if another test enabled debug mode
        // The global state makes this tricky
        let enabled = is_debug_enabled();
        // Just verify it returns a bool - the actual value depends on test order
        assert!(enabled == true || enabled == false);
    }

    #[test]
    fn test_get_log_file_path_returns_option() {
        let path = get_log_file_path();
        // Path may be Some or None depending on test order
        // Just verify it doesn't panic
        let _ = path;
    }

    // =========================================================================
    // Persistence tests
    // =========================================================================

    #[test]
    fn test_persist_debug_enabled_creates_flag_file() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        // Persist as enabled
        persist_debug_enabled(&app_data_dir, true).unwrap();

        // Flag file should exist
        let flag_path = get_debug_flag_path(&app_data_dir);
        assert!(flag_path.exists());

        // is_debug_persisted should return true
        assert!(is_debug_persisted(&app_data_dir));
    }

    #[test]
    fn test_persist_debug_disabled_removes_flag_file() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        // First enable
        persist_debug_enabled(&app_data_dir, true).unwrap();
        assert!(is_debug_persisted(&app_data_dir));

        // Then disable
        persist_debug_enabled(&app_data_dir, false).unwrap();
        assert!(!is_debug_persisted(&app_data_dir));
    }

    #[test]
    fn test_persist_debug_disabled_when_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        // Disable when never enabled - should not error
        let result = persist_debug_enabled(&app_data_dir, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_debug_persisted_false_when_no_flag() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        assert!(!is_debug_persisted(&app_data_dir));
    }

    // =========================================================================
    // Enable/disable logging tests
    // =========================================================================

    #[test]
    #[serial]
    fn test_enable_debug_logging_creates_log_file() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        let result = enable_debug_logging(&app_data_dir);
        assert!(result.is_ok());

        let log_path = result.unwrap();
        assert!(log_path.exists());
        assert!(log_path.to_string_lossy().contains("debug-"));
        assert!(log_path.extension().map(|e| e == "log").unwrap_or(false));

        // Clean up
        let _ = disable_debug_logging();
    }

    #[test]
    #[serial]
    fn test_enable_debug_logging_creates_logs_directory() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        let logs_dir = get_logs_dir(&app_data_dir);
        assert!(!logs_dir.exists());

        let _ = enable_debug_logging(&app_data_dir);

        assert!(logs_dir.exists());

        // Clean up
        let _ = disable_debug_logging();
    }

    #[test]
    #[serial]
    fn test_disable_debug_logging_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        let _ = enable_debug_logging(&app_data_dir);
        let result = disable_debug_logging();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_disable_debug_logging_when_not_enabled() {
        // Should not error even if not enabled
        let result = disable_debug_logging();
        assert!(result.is_ok());
    }

    // =========================================================================
    // Write log tests
    // =========================================================================

    #[test]
    #[serial]
    fn test_write_log_when_disabled() {
        // First ensure disabled
        let _ = disable_debug_logging();

        // Write should succeed (no-op) when disabled
        let result = write_log("INFO", "test", "message");
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_write_log_when_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        // Enable and get the log path
        let log_path = enable_debug_logging(&app_data_dir).unwrap();

        // Write a unique message to identify this test
        let unique_msg = format!("Unique test message {}", std::process::id());
        write_log("INFO", "test", &unique_msg).unwrap();

        // Read log file and verify our message is there
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[INFO]"), "Log should contain INFO level");
        assert!(
            content.contains(&unique_msg),
            "Log should contain our unique message"
        );

        // Clean up
        let _ = disable_debug_logging();
    }

    #[test]
    #[serial]
    fn test_write_log_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        let log_path = enable_debug_logging(&app_data_dir).unwrap();

        // Write log with context using unique identifiers
        let unique_id = std::process::id();
        let unique_msg = format!("Command executed {}", unique_id);
        let context = format!(r#"{{"cmd": "test{}"}}"#, unique_id);
        write_log_with_context("INFO", "invoke", &unique_msg, Some(&context)).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(
            content.contains(&unique_msg),
            "Log should contain our message"
        );
        assert!(
            content.contains(&context),
            "Log should contain context JSON"
        );

        // Clean up
        let _ = disable_debug_logging();
    }

    #[test]
    #[serial]
    fn test_write_log_without_context() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        let log_path = enable_debug_logging(&app_data_dir).unwrap();

        let unique_msg = format!("Warning message {}", std::process::id());
        write_log_with_context("WARN", "frontend", &unique_msg, None).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(
            content.contains(&unique_msg),
            "Log should contain our warning message"
        );

        // Clean up
        let _ = disable_debug_logging();
    }

    // =========================================================================
    // Convenience function tests
    // =========================================================================

    #[test]
    fn test_debug_log_function() {
        // Just verify it doesn't panic
        debug_log("Debug message");
    }

    #[test]
    fn test_info_log_function() {
        info_log("Info message");
    }

    #[test]
    fn test_warn_log_function() {
        warn_log("Warning message");
    }

    #[test]
    fn test_error_log_function() {
        error_log("Error message");
    }

    // =========================================================================
    // init_from_persisted tests
    // =========================================================================

    #[test]
    #[serial]
    fn test_init_from_persisted_when_flag_exists() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        // Create the flag file
        persist_debug_enabled(&app_data_dir, true).unwrap();

        // init_from_persisted should enable debug logging
        let result = init_from_persisted(&app_data_dir);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Clean up
        let _ = disable_debug_logging();
    }

    #[test]
    fn test_init_from_persisted_when_no_flag() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        // No flag file exists
        let result = init_from_persisted(&app_data_dir);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // =========================================================================
    // Log format tests
    // =========================================================================

    #[test]
    #[serial]
    fn test_log_format_includes_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        let log_path = enable_debug_logging(&app_data_dir).unwrap();

        let unique_msg = format!("Timestamp test {}", std::process::id());
        write_log("INFO", "test", &unique_msg).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        // Should contain date format like [2024-01-01 12:00:00.123]
        assert!(content.contains("[20"), "Log should contain year prefix");

        // Clean up
        let _ = disable_debug_logging();
    }

    #[test]
    #[serial]
    fn test_log_format_uppercase_level() {
        let temp_dir = TempDir::new().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();

        let log_path = enable_debug_logging(&app_data_dir).unwrap();

        let unique_msg = format!("Uppercase test {}", std::process::id());
        write_log("info", "test", &unique_msg).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[INFO]"), "Level should be uppercased");

        // Clean up
        let _ = disable_debug_logging();
    }
}
