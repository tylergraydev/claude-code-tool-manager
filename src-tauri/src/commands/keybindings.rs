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
