//! Sound player service for cross-platform sound playback and management.

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents a sound file that can be played
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSound {
    pub name: String,
    pub path: String,
    pub category: String, // "system" or "custom"
}

/// Represents a custom uploaded sound
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSound {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created_at: String,
}

/// Get the system sounds directory path based on the current OS
pub fn get_system_sounds_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from("/System/Library/Sounds"))
    }

    #[cfg(target_os = "linux")]
    {
        // Try common Linux sound directories
        let paths = ["/usr/share/sounds/freedesktop/stereo", "/usr/share/sounds"];
        for path in paths {
            if Path::new(path).exists() {
                return Some(PathBuf::from(path));
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(r"C:\Windows\Media"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

/// Get the custom sounds directory (~/.claude/sounds/)
pub fn get_custom_sounds_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("sounds")
}

/// Get the hooks directory (~/.claude/hooks/)
pub fn get_hooks_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("hooks")
}

/// Ensure the custom sounds directory exists
pub fn ensure_sounds_directory() -> Result<PathBuf, String> {
    let path = get_custom_sounds_path();
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create sounds directory: {}", e))?;
    Ok(path)
}

/// Ensure the hooks directory exists
pub fn ensure_hooks_directory() -> Result<PathBuf, String> {
    let path = get_hooks_path();
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create hooks directory: {}", e))?;
    Ok(path)
}

/// List all system sounds available on the current OS
pub fn list_system_sounds() -> Result<Vec<SystemSound>, String> {
    let sounds_path = match get_system_sounds_path() {
        Some(path) => path,
        None => return Ok(vec![]),
    };

    if !sounds_path.exists() {
        return Ok(vec![]);
    }

    let mut sounds = Vec::new();

    let entries = std::fs::read_dir(&sounds_path)
        .map_err(|e| format!("Failed to read system sounds directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            // Filter for audio files
            if matches!(ext.as_str(), "aiff" | "wav" | "mp3" | "ogg" | "oga" | "m4a") {
                if let Some(name) = path.file_stem() {
                    sounds.push(SystemSound {
                        name: name.to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        category: "system".to_string(),
                    });
                }
            }
        }
    }

    sounds.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    info!("[SoundPlayer] Found {} system sounds", sounds.len());
    Ok(sounds)
}

/// List all custom sounds in ~/.claude/sounds/
pub fn list_custom_sounds() -> Result<Vec<CustomSound>, String> {
    let sounds_path = get_custom_sounds_path();

    if !sounds_path.exists() {
        return Ok(vec![]);
    }

    let mut sounds = Vec::new();

    let entries = std::fs::read_dir(&sounds_path)
        .map_err(|e| format!("Failed to read custom sounds directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            // Filter for audio files
            if matches!(ext.as_str(), "aiff" | "wav" | "mp3" | "ogg" | "oga" | "m4a") {
                if let Ok(metadata) = path.metadata() {
                    let created_at = metadata
                        .created()
                        .or_else(|_| metadata.modified())
                        .map(|t| {
                            chrono::DateTime::<chrono::Utc>::from(t)
                                .format("%Y-%m-%dT%H:%M:%SZ")
                                .to_string()
                        })
                        .unwrap_or_else(|_| "unknown".to_string());

                    sounds.push(CustomSound {
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        path: path.to_string_lossy().to_string(),
                        size: metadata.len(),
                        created_at,
                    });
                }
            }
        }
    }

    sounds.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    info!("[SoundPlayer] Found {} custom sounds", sounds.len());
    Ok(sounds)
}

/// Play a sound file (non-blocking)
pub fn play_sound(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(format!("Sound file not found: {}", path.display()));
    }

    let path_str = path.to_string_lossy().to_string();
    info!("[SoundPlayer] Playing sound: {}", path_str);

    #[cfg(target_os = "macos")]
    {
        std::thread::spawn(move || {
            let result = Command::new("afplay").arg(&path_str).output();
            if let Err(e) = result {
                error!("[SoundPlayer] Failed to play sound: {}", e);
            }
        });
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(move || {
            // Try paplay first (PulseAudio), then aplay (ALSA)
            let result = Command::new("paplay")
                .arg(&path_str)
                .output()
                .or_else(|_| Command::new("aplay").arg(&path_str).output());
            if let Err(e) = result {
                error!("[SoundPlayer] Failed to play sound: {}", e);
            }
        });
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || {
            let script = format!(
                "(New-Object Media.SoundPlayer '{}').PlaySync()",
                path_str.replace("'", "''")
            );
            let result = Command::new("powershell").args(["-c", &script]).output();
            if let Err(e) = result {
                error!("[SoundPlayer] Failed to play sound: {}", e);
            }
        });
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Sound playback not supported on this platform".to_string())
    }
}

/// Validate that a file is a valid audio format
pub fn validate_sound_file(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if !matches!(
        extension.as_str(),
        "aiff" | "wav" | "mp3" | "ogg" | "oga" | "m4a"
    ) {
        return Err(format!("Unsupported audio format: {}", extension));
    }

    Ok(())
}

/// Generate a shell command for playing a sound based on the platform
pub fn generate_play_command(sound_path: &str, method: &str) -> String {
    if method == "python" {
        // Use the Python notification script
        let python_cmd = if cfg!(target_os = "windows") {
            "python"
        } else {
            "python3"
        };
        return format!("{} ~/.claude/hooks/notification-hook.py", python_cmd);
    }

    // Direct shell command approach
    #[cfg(target_os = "macos")]
    {
        format!("afplay \"{}\"", sound_path)
    }

    #[cfg(target_os = "linux")]
    {
        format!("paplay \"{}\" || aplay \"{}\"", sound_path, sound_path)
    }

    #[cfg(target_os = "windows")]
    {
        format!(
            "powershell -c \"(New-Object Media.SoundPlayer '{}').PlaySync()\"",
            sound_path.replace("'", "''")
        )
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        format!(
            "# Sound playback not supported on this platform: {}",
            sound_path
        )
    }
}

/// The Python notification script template
pub const NOTIFICATION_SCRIPT: &str = r#"#!/usr/bin/env python3
"""
Notification hook for Claude Code that plays different sounds based on event type.
"""
import json
import sys
import subprocess
import os

# Hook event sounds (PermissionRequest, Stop, SubagentStop, etc.)
EVENT_SOUNDS = {
    "PermissionRequest": "~/.claude/sounds/permission.wav",
    "Stop": "~/.claude/sounds/success.wav",
    "SubagentStop": "~/.claude/sounds/success.wav",
}

# Notification type sounds (for Notification hook events)
NOTIFICATION_SOUNDS = {
    "permission_prompt": "~/.claude/sounds/permission.wav",
    "idle_prompt": "~/.claude/sounds/idle.wav",
    "auth_success": "~/.claude/sounds/success.wav",
    "elicitation_dialog": "~/.claude/sounds/input.wav",
}

DEFAULT_SOUND = "~/.claude/sounds/notification.wav"

def play_sound(sound_path: str) -> bool:
    sound_path = os.path.expanduser(sound_path)
    if not os.path.exists(sound_path):
        return False

    try:
        if sys.platform == "darwin":
            subprocess.run(["afplay", sound_path], check=True, capture_output=True)
        elif sys.platform.startswith("linux"):
            try:
                subprocess.run(["paplay", sound_path], check=True, capture_output=True)
            except FileNotFoundError:
                subprocess.run(["aplay", sound_path], check=True, capture_output=True)
        elif sys.platform == "win32":
            subprocess.run([
                "powershell", "-c",
                f"(New-Object Media.SoundPlayer '{sound_path}').PlaySync()"
            ], check=True, capture_output=True)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False

def main():
    try:
        input_data = json.load(sys.stdin)
    except json.JSONDecodeError:
        sys.exit(1)

    hook_event = input_data.get("hook_event_name", "")
    notification_type = input_data.get("notification_type", "")

    if hook_event in EVENT_SOUNDS:
        sound_file = EVENT_SOUNDS[hook_event]
    elif notification_type in NOTIFICATION_SOUNDS:
        sound_file = NOTIFICATION_SOUNDS[notification_type]
    else:
        sound_file = DEFAULT_SOUND

    if play_sound(sound_file):
        print(json.dumps({"suppressOutput": True}))
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()
"#;

/// Deploy the Python notification script to ~/.claude/hooks/
pub fn deploy_notification_script() -> Result<String, String> {
    let hooks_dir = ensure_hooks_directory()?;
    let script_path = hooks_dir.join("notification-hook.py");

    std::fs::write(&script_path, NOTIFICATION_SCRIPT)
        .map_err(|e| format!("Failed to write notification script: {}", e))?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms)
            .map_err(|e| format!("Failed to set script permissions: {}", e))?;
    }

    info!(
        "[SoundPlayer] Deployed notification script to: {}",
        script_path.display()
    );
    Ok(script_path.to_string_lossy().to_string())
}

/// Save a custom sound file
pub fn save_custom_sound(name: &str, data: &[u8]) -> Result<CustomSound, String> {
    let sounds_dir = ensure_sounds_directory()?;
    let file_path = sounds_dir.join(name);

    // Validate the filename has a valid extension
    let extension = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if !matches!(
        extension.as_str(),
        "aiff" | "wav" | "mp3" | "ogg" | "oga" | "m4a"
    ) {
        return Err(format!("Unsupported audio format: {}", extension));
    }

    std::fs::write(&file_path, data).map_err(|e| format!("Failed to write sound file: {}", e))?;

    let metadata = std::fs::metadata(&file_path)
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    info!("[SoundPlayer] Saved custom sound: {}", file_path.display());

    Ok(CustomSound {
        name: name.to_string(),
        path: file_path.to_string_lossy().to_string(),
        size: metadata.len(),
        created_at,
    })
}

/// Delete a custom sound file
pub fn delete_custom_sound(name: &str) -> Result<(), String> {
    let sounds_dir = get_custom_sounds_path();
    let file_path = sounds_dir.join(name);

    if !file_path.exists() {
        return Err(format!("Sound file not found: {}", name));
    }

    // Safety check: only delete from the sounds directory
    if !file_path.starts_with(&sounds_dir) {
        return Err("Cannot delete files outside sounds directory".to_string());
    }

    std::fs::remove_file(&file_path).map_err(|e| format!("Failed to delete sound file: {}", e))?;

    info!("[SoundPlayer] Deleted custom sound: {}", name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_play_command_shell() {
        let cmd = generate_play_command("/path/to/sound.wav", "shell");
        #[cfg(target_os = "macos")]
        assert!(cmd.contains("afplay"));
        #[cfg(target_os = "linux")]
        assert!(cmd.contains("paplay"));
    }

    #[test]
    fn test_generate_play_command_python() {
        let cmd = generate_play_command("/path/to/sound.wav", "python");
        assert!(cmd.contains("python"));
        assert!(cmd.contains("notification-hook.py"));
    }

    #[test]
    fn test_validate_sound_file_valid_extensions() {
        // This test would need actual files to pass, but we can at least verify
        // the function handles missing files correctly
        let result = validate_sound_file("/nonexistent/sound.wav");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_sound_file_missing_extension() {
        // Create a temp file without audio extension
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("sound.txt");
        std::fs::write(&file_path, b"not audio").unwrap();

        let result = validate_sound_file(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported"));
    }

    #[test]
    fn test_validate_sound_file_valid_wav() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.wav");
        std::fs::write(&file_path, b"fake wav data").unwrap();

        let result = validate_sound_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sound_file_valid_mp3() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.mp3");
        std::fs::write(&file_path, b"fake mp3 data").unwrap();

        let result = validate_sound_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sound_file_valid_aiff() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.aiff");
        std::fs::write(&file_path, b"fake aiff data").unwrap();

        let result = validate_sound_file(file_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_play_sound_nonexistent_file() {
        let result = play_sound("/nonexistent/path/sound.wav");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_custom_sounds_path() {
        let path = get_custom_sounds_path();
        assert!(path.to_string_lossy().contains(".claude"));
        assert!(path.to_string_lossy().contains("sounds"));
    }

    #[test]
    fn test_get_hooks_path() {
        let path = get_hooks_path();
        assert!(path.to_string_lossy().contains(".claude"));
        assert!(path.to_string_lossy().contains("hooks"));
    }

    #[test]
    fn test_save_and_delete_custom_sound() {
        // Override the sounds path for testing isn't possible directly,
        // but we can at least test the validation part
        let result = save_custom_sound("test.txt", b"not audio");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported"));
    }

    #[test]
    fn test_generate_play_command_shell_contains_path() {
        let cmd = generate_play_command("/test/sound.wav", "shell");
        assert!(cmd.contains("/test/sound.wav") || cmd.contains("sound.wav"));
    }
}
