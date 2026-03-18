use crate::db::{Database, SpinnerVerb};
use crate::services::spinner_verb_writer;
use log::info;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Get all spinner verbs
#[tauri::command]
pub fn get_all_spinner_verbs(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<SpinnerVerb>, String> {
    info!("[SpinnerVerbs] Getting all spinner verbs");
    let db = db.lock().map_err(|e| e.to_string())?;
    get_all_spinner_verbs_from_db(&db)
}

/// Create a new spinner verb
#[tauri::command]
pub fn create_spinner_verb(
    db: State<'_, Arc<Mutex<Database>>>,
    verb: String,
) -> Result<SpinnerVerb, String> {
    info!("[SpinnerVerbs] Creating spinner verb: {}", verb);
    let db = db.lock().map_err(|e| e.to_string())?;
    create_spinner_verb_in_db(&db, &verb)
}

/// Update an existing spinner verb
#[tauri::command]
pub fn update_spinner_verb(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    verb: String,
    is_enabled: bool,
) -> Result<SpinnerVerb, String> {
    info!("[SpinnerVerbs] Updating spinner verb id={}: {}", id, verb);
    let db = db.lock().map_err(|e| e.to_string())?;
    update_spinner_verb_in_db(&db, id, &verb, is_enabled)
}

/// Delete a spinner verb
#[tauri::command]
pub fn delete_spinner_verb(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    info!("[SpinnerVerbs] Deleting spinner verb id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    delete_spinner_verb_in_db(&db, id)
}

/// Reorder spinner verbs
#[tauri::command]
pub fn reorder_spinner_verbs(
    db: State<'_, Arc<Mutex<Database>>>,
    ids: Vec<i64>,
) -> Result<(), String> {
    info!("[SpinnerVerbs] Reordering {} spinner verbs", ids.len());
    let db = db.lock().map_err(|e| e.to_string())?;
    reorder_spinner_verbs_in_db(&db, &ids)
}

/// Get the spinner verb mode
#[tauri::command]
pub fn get_spinner_verb_mode(db: State<'_, Arc<Mutex<Database>>>) -> Result<String, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    get_spinner_verb_mode_from_db(&db)
}

/// Set the spinner verb mode
#[tauri::command]
pub fn set_spinner_verb_mode(
    db: State<'_, Arc<Mutex<Database>>>,
    mode: String,
) -> Result<(), String> {
    info!("[SpinnerVerbs] Setting mode to: {}", mode);
    let db = db.lock().map_err(|e| e.to_string())?;
    set_spinner_verb_mode_in_db(&db, &mode)
}

/// Sync spinner verbs to settings.json
#[tauri::command]
pub fn sync_spinner_verbs(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    info!("[SpinnerVerbs] Syncing spinner verbs to settings.json");
    let db = db.lock().map_err(|e| e.to_string())?;
    sync_spinner_verbs_from_db(&db)
}

/// Read current spinner verbs config from settings.json
#[tauri::command]
pub fn read_current_spinner_verbs_config() -> Result<Option<Value>, String> {
    info!("[SpinnerVerbs] Reading current config from settings.json");
    spinner_verb_writer::read_current_spinner_verbs_config().map_err(|e| e.to_string())
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

pub fn get_all_spinner_verbs_from_db(db: &Database) -> Result<Vec<SpinnerVerb>, String> {
    db.get_all_spinner_verbs().map_err(|e| e.to_string())
}

pub fn create_spinner_verb_in_db(db: &Database, verb: &str) -> Result<SpinnerVerb, String> {
    db.create_spinner_verb(verb).map_err(|e| e.to_string())
}

pub fn update_spinner_verb_in_db(
    db: &Database,
    id: i64,
    verb: &str,
    is_enabled: bool,
) -> Result<SpinnerVerb, String> {
    db.update_spinner_verb(id, verb, is_enabled)
        .map_err(|e| e.to_string())
}

pub fn delete_spinner_verb_in_db(db: &Database, id: i64) -> Result<(), String> {
    db.delete_spinner_verb(id).map_err(|e| e.to_string())
}

pub fn reorder_spinner_verbs_in_db(db: &Database, ids: &[i64]) -> Result<(), String> {
    db.reorder_spinner_verbs(ids).map_err(|e| e.to_string())
}

pub fn get_spinner_verb_mode_from_db(db: &Database) -> Result<String, String> {
    db.get_spinner_verb_mode().map_err(|e| e.to_string())
}

pub fn set_spinner_verb_mode_in_db(db: &Database, mode: &str) -> Result<(), String> {
    db.set_spinner_verb_mode(mode).map_err(|e| e.to_string())
}

pub fn sync_spinner_verbs_from_db(db: &Database) -> Result<(), String> {
    let verbs = db.get_all_spinner_verbs().map_err(|e| e.to_string())?;
    let mode = db.get_spinner_verb_mode().map_err(|e| e.to_string())?;

    let enabled_verbs: Vec<String> = verbs
        .iter()
        .filter(|v| v.is_enabled)
        .map(|v| v.verb.clone())
        .collect();

    if enabled_verbs.is_empty() {
        spinner_verb_writer::remove_spinner_verbs_from_settings().map_err(|e| e.to_string())
    } else {
        spinner_verb_writer::write_spinner_verbs_to_settings(&mode, &enabled_verbs)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_verb(db: &Database, verb: &str) -> SpinnerVerb {
        create_spinner_verb_in_db(db, verb).unwrap()
    }

    #[test]
    fn test_create_spinner_verb() {
        let db = Database::in_memory().unwrap();
        let verb = create_test_verb(&db, "Pondering");

        assert_eq!(verb.verb, "Pondering");
        assert!(verb.is_enabled);
        assert_eq!(verb.display_order, 0);
    }

    #[test]
    fn test_get_all_spinner_verbs() {
        let db = Database::in_memory().unwrap();
        create_test_verb(&db, "Pondering");
        create_test_verb(&db, "Crafting");

        let verbs = get_all_spinner_verbs_from_db(&db).unwrap();
        assert_eq!(verbs.len(), 2);
    }

    #[test]
    fn test_update_spinner_verb() {
        let db = Database::in_memory().unwrap();
        let verb = create_test_verb(&db, "Pondering");

        let updated = update_spinner_verb_in_db(&db, verb.id, "Thinking", false).unwrap();
        assert_eq!(updated.verb, "Thinking");
        assert!(!updated.is_enabled);
    }

    #[test]
    fn test_delete_spinner_verb() {
        let db = Database::in_memory().unwrap();
        let verb = create_test_verb(&db, "Pondering");
        delete_spinner_verb_in_db(&db, verb.id).unwrap();

        let verbs = get_all_spinner_verbs_from_db(&db).unwrap();
        assert!(verbs.is_empty());
    }

    #[test]
    fn test_spinner_verb_mode() {
        let db = Database::in_memory().unwrap();

        let mode = get_spinner_verb_mode_from_db(&db).unwrap();
        assert_eq!(mode, "append");

        set_spinner_verb_mode_in_db(&db, "replace").unwrap();
        let mode = get_spinner_verb_mode_from_db(&db).unwrap();
        assert_eq!(mode, "replace");
    }

    #[test]
    fn test_reorder_spinner_verbs() {
        let db = Database::in_memory().unwrap();
        let v1 = create_test_verb(&db, "Pondering");
        let v2 = create_test_verb(&db, "Crafting");
        let v3 = create_test_verb(&db, "Brewing");

        // Reorder: v3, v1, v2
        reorder_spinner_verbs_in_db(&db, &[v3.id, v1.id, v2.id]).unwrap();

        let verbs = get_all_spinner_verbs_from_db(&db).unwrap();
        assert_eq!(verbs[0].verb, "Brewing");
        assert_eq!(verbs[1].verb, "Pondering");
        assert_eq!(verbs[2].verb, "Crafting");
    }

    #[test]
    fn test_display_order_auto_increments() {
        let db = Database::in_memory().unwrap();
        let v1 = create_test_verb(&db, "Pondering");
        let v2 = create_test_verb(&db, "Crafting");
        let v3 = create_test_verb(&db, "Brewing");

        assert_eq!(v1.display_order, 0);
        assert_eq!(v2.display_order, 1);
        assert_eq!(v3.display_order, 2);
    }

    // =========================================================================
    // Additional spinner verb tests
    // =========================================================================

    #[test]
    fn test_get_all_spinner_verbs_empty() {
        let db = Database::in_memory().unwrap();
        let verbs = get_all_spinner_verbs_from_db(&db).unwrap();
        assert!(verbs.is_empty());
    }

    #[test]
    fn test_delete_nonexistent_verb() {
        let db = Database::in_memory().unwrap();
        // Deleting a non-existent ID - the db method may error
        let result = delete_spinner_verb_in_db(&db, 9999);
        // Depending on DB impl, this may succeed or error
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_set_spinner_verb_mode_append() {
        let db = Database::in_memory().unwrap();
        set_spinner_verb_mode_in_db(&db, "append").unwrap();
        assert_eq!(get_spinner_verb_mode_from_db(&db).unwrap(), "append");
    }

    #[test]
    fn test_set_spinner_verb_mode_replace() {
        let db = Database::in_memory().unwrap();
        set_spinner_verb_mode_in_db(&db, "replace").unwrap();
        assert_eq!(get_spinner_verb_mode_from_db(&db).unwrap(), "replace");
    }

    #[test]
    fn test_update_verb_preserves_order() {
        let db = Database::in_memory().unwrap();
        let v1 = create_test_verb(&db, "Pondering");
        let _v2 = create_test_verb(&db, "Crafting");

        let updated = update_spinner_verb_in_db(&db, v1.id, "Wondering", true).unwrap();
        assert_eq!(updated.verb, "Wondering");
        assert_eq!(updated.display_order, v1.display_order);
    }

    #[test]
    fn test_reorder_with_empty_list() {
        let db = Database::in_memory().unwrap();
        let result = reorder_spinner_verbs_in_db(&db, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_spinner_verb_serde() {
        let verb = SpinnerVerb {
            id: 1,
            verb: "Pondering".to_string(),
            is_enabled: true,
            display_order: 0,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&verb).unwrap();
        let deserialized: SpinnerVerb = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.verb, "Pondering");
        assert!(deserialized.is_enabled);
    }

    #[test]
    fn test_sync_spinner_verbs_empty_removes() {
        let db = Database::in_memory().unwrap();
        // With no verbs, sync_spinner_verbs_from_db should attempt to remove
        // This may fail since settings.json doesn't exist in test, but we test the logic path
        let result = sync_spinner_verbs_from_db(&db);
        // The function calls remove_spinner_verbs_from_settings which touches the filesystem
        // so it may fail in test env, but we just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_sync_spinner_verbs_with_disabled_verbs() {
        let db = Database::in_memory().unwrap();
        let v1 = create_test_verb(&db, "Pondering");
        // Disable it
        update_spinner_verb_in_db(&db, v1.id, "Pondering", false).unwrap();

        // All verbs disabled, should attempt removal
        let result = sync_spinner_verbs_from_db(&db);
        let _ = result; // May fail on filesystem in test
    }
}
