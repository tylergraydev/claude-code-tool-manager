use crate::services::keybindings_writer::{self, KeybindingsFile};
use log::info;

/// Get keybindings from ~/.claude/keybindings.json
#[tauri::command]
pub fn get_keybindings() -> Result<KeybindingsFile, String> {
    info!("[Keybindings] Reading keybindings file");
    keybindings_writer::read_keybindings().map_err(|e| e.to_string())
}

/// Save keybindings to ~/.claude/keybindings.json
#[tauri::command]
pub fn save_keybindings(keybindings: KeybindingsFile) -> Result<(), String> {
    info!(
        "[Keybindings] Saving keybindings ({} context blocks)",
        keybindings.bindings.len()
    );
    keybindings_writer::write_keybindings(&keybindings).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybindings_file_serde_empty() {
        let kf = KeybindingsFile {
            schema: None,
            bindings: vec![],
        };
        let json = serde_json::to_string(&kf).unwrap();
        let deser: KeybindingsFile = serde_json::from_str(&json).unwrap();
        assert!(deser.bindings.is_empty());
    }

    #[test]
    fn test_get_keybindings_returns_result() {
        // Filesystem-dependent; just verify no panic.
        let _ = get_keybindings();
    }
}
