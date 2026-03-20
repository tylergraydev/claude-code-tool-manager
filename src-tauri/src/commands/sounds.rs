//! Sound management commands for Tauri.

use crate::services::sound_player::{self, CustomSound, SystemSound};
use log::info;

/// Get all available system sounds for the current OS
#[tauri::command]
pub fn get_system_sounds() -> Result<Vec<SystemSound>, String> {
    info!("[Sounds] Getting system sounds");
    sound_player::list_system_sounds()
}

/// Get all custom sounds from ~/.claude/sounds/
#[tauri::command]
pub fn get_custom_sounds() -> Result<Vec<CustomSound>, String> {
    info!("[Sounds] Getting custom sounds");
    sound_player::list_custom_sounds()
}

/// Preview a sound (play it)
#[tauri::command]
pub fn preview_sound(path: String) -> Result<(), String> {
    info!("[Sounds] Previewing sound: {}", path);
    sound_player::play_sound(&path)
}

/// Ensure the sounds directory exists and return its path
#[tauri::command]
pub fn ensure_sounds_directory() -> Result<String, String> {
    info!("[Sounds] Ensuring sounds directory exists");
    let path = sound_player::ensure_sounds_directory()?;
    Ok(path.to_string_lossy().to_string())
}

/// Upload a custom sound file
#[tauri::command]
pub fn upload_custom_sound(name: String, data: Vec<u8>) -> Result<CustomSound, String> {
    info!("[Sounds] Uploading custom sound: {}", name);
    sound_player::save_custom_sound(&name, &data)
}

/// Delete a custom sound file
#[tauri::command]
pub fn delete_custom_sound(name: String) -> Result<(), String> {
    info!("[Sounds] Deleting custom sound: {}", name);
    sound_player::delete_custom_sound(&name)
}

/// Generate a shell command for playing a sound
#[tauri::command]
pub fn generate_sound_hook_command(sound_path: String, method: String) -> Result<String, String> {
    info!(
        "[Sounds] Generating hook command for: {} (method: {})",
        sound_path, method
    );
    Ok(sound_player::generate_play_command(&sound_path, &method))
}

/// Deploy the Python notification script to ~/.claude/hooks/
#[tauri::command]
pub fn deploy_notification_script() -> Result<String, String> {
    info!("[Sounds] Deploying notification script");
    sound_player::deploy_notification_script()
}

/// Get the sounds directory path
#[tauri::command]
pub fn get_sounds_directory() -> Result<String, String> {
    let path = sound_player::get_custom_sounds_path();
    Ok(path.to_string_lossy().to_string())
}

/// Validate that a sound file path is valid
#[tauri::command]
pub fn validate_sound_file(path: String) -> Result<(), String> {
    sound_player::validate_sound_file(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sound_hook_command_python() {
        let result =
            generate_sound_hook_command("/path/to/sound.wav".to_string(), "python".to_string());
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.contains("notification-hook.py"));
    }

    #[test]
    fn test_generate_sound_hook_command_shell() {
        let result =
            generate_sound_hook_command("/path/to/sound.wav".to_string(), "shell".to_string());
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.contains("/path/to/sound.wav"));
    }

    #[test]
    fn test_validate_sound_file_nonexistent() {
        let result = validate_sound_file("/nonexistent/path/sound.wav".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_sounds_directory_returns_path() {
        let result = get_sounds_directory();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.contains("sounds"));
    }
}
