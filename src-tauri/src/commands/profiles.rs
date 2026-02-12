use crate::commands::config::sync_global_config_from_db;
use crate::db::{CreateProfileRequest, Database, Profile, ProfileWithItems};
use log::info;
use rusqlite::params;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Get all profiles
#[tauri::command]
pub fn get_all_profiles(db: State<'_, Arc<Mutex<Database>>>) -> Result<Vec<Profile>, String> {
    info!("[Profiles] Getting all profiles");
    let db = db.lock().map_err(|e| e.to_string())?;
    get_all_profiles_from_db(&db)
}

/// Get a single profile with its items
#[tauri::command]
pub fn get_profile(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
) -> Result<ProfileWithItems, String> {
    info!("[Profiles] Getting profile id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    get_profile_from_db(&db, id)
}

/// Create a new profile
#[tauri::command]
pub fn create_profile(
    db: State<'_, Arc<Mutex<Database>>>,
    request: CreateProfileRequest,
) -> Result<Profile, String> {
    info!("[Profiles] Creating profile: {}", request.name);
    let db = db.lock().map_err(|e| e.to_string())?;
    create_profile_in_db(&db, &request)
}

/// Update an existing profile
#[tauri::command]
pub fn update_profile(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    request: CreateProfileRequest,
) -> Result<Profile, String> {
    info!("[Profiles] Updating profile id={}: {}", id, request.name);
    let db = db.lock().map_err(|e| e.to_string())?;
    update_profile_in_db(&db, id, &request)
}

/// Delete a profile
#[tauri::command]
pub fn delete_profile(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    info!("[Profiles] Deleting profile id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    delete_profile_in_db(&db, id)
}

/// Capture the current global configuration into a profile
#[tauri::command]
pub fn capture_profile_from_current(
    db: State<'_, Arc<Mutex<Database>>>,
    profile_id: i64,
) -> Result<ProfileWithItems, String> {
    info!(
        "[Profiles] Capturing current config into profile id={}",
        profile_id
    );
    let db = db.lock().map_err(|e| e.to_string())?;
    capture_profile_from_current_in_db(&db, profile_id)
}

/// Activate a profile (replace global config with profile items)
#[tauri::command]
pub fn activate_profile(db: State<'_, Arc<Mutex<Database>>>, id: i64) -> Result<(), String> {
    info!("[Profiles] Activating profile id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    activate_profile_in_db(&db, id)?;

    // Sync config to disk
    sync_global_config_from_db(&db)?;

    Ok(())
}

/// Deactivate the current active profile (just marks none as active)
#[tauri::command]
pub fn deactivate_profile(db: State<'_, Arc<Mutex<Database>>>) -> Result<(), String> {
    info!("[Profiles] Deactivating all profiles");
    let db = db.lock().map_err(|e| e.to_string())?;
    deactivate_all_profiles(&db)
}

/// Get the currently active profile
#[tauri::command]
pub fn get_active_profile(db: State<'_, Arc<Mutex<Database>>>) -> Result<Option<Profile>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    get_active_profile_from_db(&db)
}

// ============================================================================
// Testable helper functions (no Tauri State dependency)
// ============================================================================

pub fn get_all_profiles_from_db(db: &Database) -> Result<Vec<Profile>, String> {
    let mut stmt = db
        .conn()
        .prepare(
            "SELECT id, name, description, icon, is_active, created_at, updated_at
             FROM profiles ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let profiles = stmt
        .query_map([], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(profiles)
}

fn get_profile_by_id(db: &Database, id: i64) -> Result<Profile, String> {
    db.conn()
        .query_row(
            "SELECT id, name, description, icon, is_active, created_at, updated_at
             FROM profiles WHERE id = ?",
            params![id],
            |row| {
                Ok(Profile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    is_active: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())
}

fn get_profile_items_grouped(db: &Database, profile_id: i64) -> Result<ProfileWithItems, String> {
    let profile = get_profile_by_id(db, profile_id)?;

    let mut stmt = db
        .conn()
        .prepare("SELECT item_type, item_id FROM profile_items WHERE profile_id = ?")
        .map_err(|e| e.to_string())?;

    let rows: Vec<(String, i64)> = stmt
        .query_map(params![profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut mcps = Vec::new();
    let mut skills = Vec::new();
    let mut commands = Vec::new();
    let mut subagents = Vec::new();
    let mut hooks = Vec::new();

    for (item_type, item_id) in rows {
        match item_type.as_str() {
            "mcp" => mcps.push(item_id),
            "skill" => skills.push(item_id),
            "command" => commands.push(item_id),
            "subagent" => subagents.push(item_id),
            "hook" => hooks.push(item_id),
            _ => {}
        }
    }

    Ok(ProfileWithItems {
        profile,
        mcps,
        skills,
        commands,
        subagents,
        hooks,
    })
}

pub fn get_profile_from_db(db: &Database, id: i64) -> Result<ProfileWithItems, String> {
    get_profile_items_grouped(db, id)
}

pub fn create_profile_in_db(
    db: &Database,
    request: &CreateProfileRequest,
) -> Result<Profile, String> {
    db.conn()
        .execute(
            "INSERT INTO profiles (name, description, icon) VALUES (?, ?, ?)",
            params![request.name, request.description, request.icon],
        )
        .map_err(|e| e.to_string())?;

    let id = db.conn().last_insert_rowid();
    get_profile_by_id(db, id)
}

pub fn update_profile_in_db(
    db: &Database,
    id: i64,
    request: &CreateProfileRequest,
) -> Result<Profile, String> {
    db.conn()
        .execute(
            "UPDATE profiles SET name = ?, description = ?, icon = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![request.name, request.description, request.icon, id],
        )
        .map_err(|e| e.to_string())?;

    get_profile_by_id(db, id)
}

pub fn delete_profile_in_db(db: &Database, id: i64) -> Result<(), String> {
    db.conn()
        .execute("DELETE FROM profiles WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn capture_profile_from_current_in_db(
    db: &Database,
    profile_id: i64,
) -> Result<ProfileWithItems, String> {
    // Clear existing items for this profile
    db.conn()
        .execute(
            "DELETE FROM profile_items WHERE profile_id = ?",
            params![profile_id],
        )
        .map_err(|e| e.to_string())?;

    // Capture current global MCPs
    db.conn()
        .execute(
            "INSERT INTO profile_items (profile_id, item_type, item_id)
             SELECT ?, 'mcp', mcp_id FROM global_mcps",
            params![profile_id],
        )
        .map_err(|e| e.to_string())?;

    // Capture current global skills
    db.conn()
        .execute(
            "INSERT INTO profile_items (profile_id, item_type, item_id)
             SELECT ?, 'skill', skill_id FROM global_skills",
            params![profile_id],
        )
        .map_err(|e| e.to_string())?;

    // Capture current global commands
    db.conn()
        .execute(
            "INSERT INTO profile_items (profile_id, item_type, item_id)
             SELECT ?, 'command', command_id FROM global_commands",
            params![profile_id],
        )
        .map_err(|e| e.to_string())?;

    // Capture current global subagents
    db.conn()
        .execute(
            "INSERT INTO profile_items (profile_id, item_type, item_id)
             SELECT ?, 'subagent', subagent_id FROM global_subagents",
            params![profile_id],
        )
        .map_err(|e| e.to_string())?;

    // Capture current global hooks
    db.conn()
        .execute(
            "INSERT INTO profile_items (profile_id, item_type, item_id)
             SELECT ?, 'hook', hook_id FROM global_hooks",
            params![profile_id],
        )
        .map_err(|e| e.to_string())?;

    // Update profile timestamp
    db.conn()
        .execute(
            "UPDATE profiles SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            params![profile_id],
        )
        .map_err(|e| e.to_string())?;

    get_profile_items_grouped(db, profile_id)
}

pub fn activate_profile_in_db(db: &Database, id: i64) -> Result<(), String> {
    // Use a transaction
    db.conn()
        .execute_batch("BEGIN TRANSACTION")
        .map_err(|e| e.to_string())?;

    let result = (|| -> Result<(), String> {
        // Clear all global assignment tables
        db.conn()
            .execute("DELETE FROM global_mcps", [])
            .map_err(|e| e.to_string())?;
        db.conn()
            .execute("DELETE FROM global_skills", [])
            .map_err(|e| e.to_string())?;
        db.conn()
            .execute("DELETE FROM global_commands", [])
            .map_err(|e| e.to_string())?;
        db.conn()
            .execute("DELETE FROM global_subagents", [])
            .map_err(|e| e.to_string())?;
        db.conn()
            .execute("DELETE FROM global_hooks", [])
            .map_err(|e| e.to_string())?;

        // Populate from profile_items (INSERT OR IGNORE for safety with deleted items)
        db.conn()
            .execute(
                "INSERT OR IGNORE INTO global_mcps (mcp_id)
                 SELECT pi.item_id FROM profile_items pi
                 INNER JOIN mcps m ON m.id = pi.item_id
                 WHERE pi.profile_id = ? AND pi.item_type = 'mcp'",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        db.conn()
            .execute(
                "INSERT OR IGNORE INTO global_skills (skill_id)
                 SELECT pi.item_id FROM profile_items pi
                 INNER JOIN skills s ON s.id = pi.item_id
                 WHERE pi.profile_id = ? AND pi.item_type = 'skill'",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        db.conn()
            .execute(
                "INSERT OR IGNORE INTO global_commands (command_id)
                 SELECT pi.item_id FROM profile_items pi
                 INNER JOIN commands c ON c.id = pi.item_id
                 WHERE pi.profile_id = ? AND pi.item_type = 'command'",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        db.conn()
            .execute(
                "INSERT OR IGNORE INTO global_subagents (subagent_id)
                 SELECT pi.item_id FROM profile_items pi
                 INNER JOIN subagents sa ON sa.id = pi.item_id
                 WHERE pi.profile_id = ? AND pi.item_type = 'subagent'",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        db.conn()
            .execute(
                "INSERT OR IGNORE INTO global_hooks (hook_id)
                 SELECT pi.item_id FROM profile_items pi
                 INNER JOIN hooks h ON h.id = pi.item_id
                 WHERE pi.profile_id = ? AND pi.item_type = 'hook'",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        // Set active profile
        db.conn()
            .execute("UPDATE profiles SET is_active = 0", [])
            .map_err(|e| e.to_string())?;
        db.conn()
            .execute(
                "UPDATE profiles SET is_active = 1 WHERE id = ?",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    })();

    if result.is_err() {
        let _ = db.conn().execute_batch("ROLLBACK");
        return result;
    }

    db.conn()
        .execute_batch("COMMIT")
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn deactivate_all_profiles(db: &Database) -> Result<(), String> {
    db.conn()
        .execute("UPDATE profiles SET is_active = 0", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_active_profile_from_db(db: &Database) -> Result<Option<Profile>, String> {
    let result = db.conn().query_row(
        "SELECT id, name, description, icon, is_active, created_at, updated_at
         FROM profiles WHERE is_active = 1",
        [],
        |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                icon: row.get(3)?,
                is_active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    );

    match result {
        Ok(profile) => Ok(Some(profile)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile(db: &Database, name: &str) -> Profile {
        let request = CreateProfileRequest {
            name: name.to_string(),
            description: Some("Test profile".to_string()),
            icon: Some("🔧".to_string()),
        };
        create_profile_in_db(db, &request).unwrap()
    }

    #[test]
    fn test_create_profile() {
        let db = Database::in_memory().unwrap();
        let profile = create_test_profile(&db, "Work");

        assert_eq!(profile.name, "Work");
        assert_eq!(profile.description, Some("Test profile".to_string()));
        assert!(!profile.is_active);
    }

    #[test]
    fn test_get_all_profiles() {
        let db = Database::in_memory().unwrap();
        create_test_profile(&db, "Work");
        create_test_profile(&db, "Personal");

        let profiles = get_all_profiles_from_db(&db).unwrap();
        assert_eq!(profiles.len(), 2);
    }

    #[test]
    fn test_update_profile() {
        let db = Database::in_memory().unwrap();
        let profile = create_test_profile(&db, "Old Name");

        let request = CreateProfileRequest {
            name: "New Name".to_string(),
            description: Some("Updated".to_string()),
            icon: None,
        };
        let updated = update_profile_in_db(&db, profile.id, &request).unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[test]
    fn test_delete_profile() {
        let db = Database::in_memory().unwrap();
        let profile = create_test_profile(&db, "Temp");
        delete_profile_in_db(&db, profile.id).unwrap();

        let profiles = get_all_profiles_from_db(&db).unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_capture_and_activate_profile() {
        let db = Database::in_memory().unwrap();

        // Create an MCP and add it globally
        db.conn()
            .execute(
                "INSERT INTO mcps (name, type, source) VALUES ('test-mcp', 'stdio', 'manual')",
                [],
            )
            .unwrap();
        let mcp_id = db.conn().last_insert_rowid();
        db.conn()
            .execute(
                "INSERT INTO global_mcps (mcp_id) VALUES (?)",
                params![mcp_id],
            )
            .unwrap();

        // Create a profile and capture current state
        let profile = create_test_profile(&db, "Test Profile");
        let with_items = capture_profile_from_current_in_db(&db, profile.id).unwrap();
        assert_eq!(with_items.mcps.len(), 1);
        assert_eq!(with_items.mcps[0], mcp_id);

        // Clear global MCPs
        db.conn().execute("DELETE FROM global_mcps", []).unwrap();
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM global_mcps", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        // Activate profile - should restore global MCPs
        activate_profile_in_db(&db, profile.id).unwrap();
        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM global_mcps", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Profile should be marked active
        let active = get_active_profile_from_db(&db).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, profile.id);
    }

    #[test]
    fn test_deactivate_profile() {
        let db = Database::in_memory().unwrap();
        let profile = create_test_profile(&db, "Active");

        // Mark active
        db.conn()
            .execute(
                "UPDATE profiles SET is_active = 1 WHERE id = ?",
                params![profile.id],
            )
            .unwrap();

        deactivate_all_profiles(&db).unwrap();
        let active = get_active_profile_from_db(&db).unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn test_activate_handles_deleted_items() {
        let db = Database::in_memory().unwrap();

        // Create an MCP and capture it
        db.conn()
            .execute(
                "INSERT INTO mcps (name, type, source) VALUES ('ephemeral', 'stdio', 'manual')",
                [],
            )
            .unwrap();
        let mcp_id = db.conn().last_insert_rowid();
        db.conn()
            .execute(
                "INSERT INTO global_mcps (mcp_id) VALUES (?)",
                params![mcp_id],
            )
            .unwrap();

        let profile = create_test_profile(&db, "Profile");
        capture_profile_from_current_in_db(&db, profile.id).unwrap();

        // Delete the MCP entirely
        db.conn().execute("DELETE FROM global_mcps", []).unwrap();
        db.conn()
            .execute("DELETE FROM mcps WHERE id = ?", params![mcp_id])
            .unwrap();

        // Activate should succeed without error (INSERT OR IGNORE with JOIN filters out missing items)
        activate_profile_in_db(&db, profile.id).unwrap();

        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM global_mcps", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0); // The deleted MCP is not restored
    }
}
