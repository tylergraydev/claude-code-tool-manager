use crate::db::models::{CreateStatusLineRequest, SegmentsPayload, StatusLine, StatusLineGalleryEntry, StatusLineSegment};
use crate::db::schema::Database;
use crate::services::{statusline_gallery, statusline_writer};
use log::info;
use std::sync::{Arc, Mutex};
use tauri::State;

// ============================================================================
// CRUD Operations
// ============================================================================

#[tauri::command]
pub fn get_all_statuslines(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<StatusLine>, String> {
    info!("[StatusLine] Getting all statuslines");
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_all_statuslines().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
) -> Result<Option<StatusLine>, String> {
    info!("[StatusLine] Getting statusline id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_statusline_by_id(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
    request: CreateStatusLineRequest,
) -> Result<StatusLine, String> {
    info!("[StatusLine] Creating statusline: {}", request.name);
    let db = db.lock().map_err(|e| e.to_string())?;
    db.create_statusline(&request).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
    request: CreateStatusLineRequest,
) -> Result<StatusLine, String> {
    info!("[StatusLine] Updating statusline id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;

    let existing = db
        .get_statusline_by_id(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "StatusLine not found".to_string())?;

    let tags_json = request.tags.as_ref().map(|t| serde_json::to_string(t).unwrap());
    let _ = tags_json; // tags handled by update_statusline

    let updated = StatusLine {
        id: existing.id,
        name: request.name,
        description: request.description,
        statusline_type: request.statusline_type,
        package_name: request.package_name,
        install_command: request.install_command,
        run_command: request.run_command,
        raw_command: request.raw_command,
        padding: request.padding.unwrap_or(existing.padding),
        is_active: existing.is_active,
        segments_json: request.segments_json,
        generated_script: request.generated_script,
        icon: request.icon,
        author: request.author,
        homepage_url: request.homepage_url,
        tags: request.tags,
        source: existing.source,
        created_at: existing.created_at,
        updated_at: existing.updated_at,
    };

    db.update_statusline(&updated).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
) -> Result<(), String> {
    info!("[StatusLine] Deleting statusline id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;

    // If this was active, remove from settings
    if let Ok(Some(sl)) = db.get_statusline_by_id(id) {
        if sl.is_active {
            statusline_writer::remove_statusline_from_settings()
                .map_err(|e| e.to_string())?;
        }
    }

    db.delete_statusline(id).map_err(|e| e.to_string())
}

// ============================================================================
// Activation
// ============================================================================

#[tauri::command]
pub fn activate_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
    id: i64,
) -> Result<StatusLine, String> {
    info!("[StatusLine] Activating statusline id={}", id);
    let db = db.lock().map_err(|e| e.to_string())?;

    let sl = db
        .get_statusline_by_id(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "StatusLine not found".to_string())?;

    // Determine the command to write to settings.json
    let command = match sl.statusline_type.as_str() {
        "custom" => {
            // Generate script from segments and write to disk
            let payload = sl
                .segments_json
                .as_ref()
                .map(|s| SegmentsPayload::parse(s))
                .unwrap_or_else(|| SegmentsPayload { theme: "default".to_string(), segments: vec![] });

            let script = statusline_writer::generate_script_from_segments_with_theme(&payload.segments, &payload.theme);

            // Write script to ~/.claude/statusline.py
            let script_path = statusline_writer::write_statusline_script(&script)
                .map_err(|e| e.to_string())?;

            // Also save the generated script in the DB
            let mut updated = sl.clone();
            updated.generated_script = Some(script);
            let _ = db.update_statusline(&updated);

            let python_cmd = if cfg!(target_os = "windows") { "python" } else { "python3" };
            format!("{} {}", python_cmd, script_path.display())
        }
        "premade" => {
            sl.run_command
                .clone()
                .ok_or_else(|| "Premade statusline has no run_command".to_string())?
        }
        "raw" => {
            sl.raw_command
                .clone()
                .ok_or_else(|| "Raw statusline has no raw_command".to_string())?
        }
        _ => return Err(format!("Unknown statusline type: {}", sl.statusline_type)),
    };

    // Write to settings.json
    statusline_writer::write_statusline_to_settings(&command, sl.padding)
        .map_err(|e| e.to_string())?;

    // Set as active in DB
    db.set_active_statusline(id).map_err(|e| e.to_string())?;

    db.get_statusline_by_id(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to retrieve activated statusline".to_string())
}

#[tauri::command]
pub fn deactivate_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<(), String> {
    info!("[StatusLine] Deactivating statusline");
    let db = db.lock().map_err(|e| e.to_string())?;

    statusline_writer::remove_statusline_from_settings()
        .map_err(|e| e.to_string())?;

    db.deactivate_all_statuslines().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_active_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Option<StatusLine>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_active_statusline().map_err(|e| e.to_string())
}

// ============================================================================
// Gallery
// ============================================================================

#[tauri::command]
pub async fn fetch_statusline_gallery(
    db: State<'_, Arc<Mutex<Database>>>,
    url: Option<String>,
) -> Result<Vec<StatusLineGalleryEntry>, String> {
    info!("[StatusLine] Fetching gallery");

    let gallery_url = {
        let db = db.lock().map_err(|e| e.to_string())?;
        url.unwrap_or_else(|| statusline_gallery::get_gallery_url(&db))
    };

    let github_token = {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.get_setting("github_token")
    };

    match statusline_gallery::fetch_gallery_from_url(
        &gallery_url,
        github_token.as_deref(),
    )
    .await
    {
        Ok(entries) => {
            // Cache the results
            let db = db.lock().map_err(|e| e.to_string())?;
            let _ = statusline_gallery::cache_gallery(&db, &entries);
            Ok(entries)
        }
        Err(e) => {
            info!("[StatusLine] Gallery fetch failed, using seed entries: {}", e);
            // Fall back to seed entries
            Ok(statusline_gallery::get_seed_gallery_entries())
        }
    }
}

#[tauri::command]
pub fn get_statusline_gallery_cache(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<Vec<StatusLineGalleryEntry>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    Ok(statusline_gallery::get_cached_gallery(&db)
        .unwrap_or_else(statusline_gallery::get_seed_gallery_entries))
}

#[tauri::command]
pub fn install_premade_statusline(
    db: State<'_, Arc<Mutex<Database>>>,
    entry: StatusLineGalleryEntry,
) -> Result<StatusLine, String> {
    info!("[StatusLine] Installing premade: {}", entry.name);
    let db = db.lock().map_err(|e| e.to_string())?;

    let request = CreateStatusLineRequest {
        name: entry.name,
        description: entry.description,
        statusline_type: "premade".to_string(),
        package_name: entry.package_name,
        install_command: entry.install_command,
        run_command: entry.run_command,
        raw_command: None,
        padding: None,
        segments_json: None,
        generated_script: None,
        icon: entry.icon,
        author: entry.author,
        homepage_url: entry.homepage_url,
        tags: entry.tags,
    };

    db.create_statusline(&request).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_statusline_gallery_url(
    db: State<'_, Arc<Mutex<Database>>>,
) -> Result<String, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    Ok(statusline_gallery::get_gallery_url(&db))
}

#[tauri::command]
pub fn set_statusline_gallery_url(
    db: State<'_, Arc<Mutex<Database>>>,
    url: String,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    statusline_gallery::set_gallery_url(&db, &url).map_err(|e| e.to_string())
}

// ============================================================================
// Preview & Import
// ============================================================================

#[tauri::command]
pub fn generate_statusline_preview(
    segments: Vec<StatusLineSegment>,
    theme: Option<String>,
) -> Result<String, String> {
    let theme = theme.as_deref().unwrap_or("default");
    Ok(statusline_writer::generate_script_from_segments_with_theme(&segments, theme))
}

#[tauri::command]
pub fn read_current_statusline_config() -> Result<Option<serde_json::Value>, String> {
    statusline_writer::read_current_statusline_config().map_err(|e| e.to_string())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::db::models::{CreateStatusLineRequest, StatusLineSegment};
    use crate::db::Database;
    use crate::services::statusline_writer;

    fn make_custom_request(name: &str) -> CreateStatusLineRequest {
        CreateStatusLineRequest {
            name: name.to_string(),
            description: Some("Test custom status line".to_string()),
            statusline_type: "custom".to_string(),
            package_name: None,
            install_command: None,
            run_command: None,
            raw_command: None,
            padding: Some(1),
            segments_json: Some(serde_json::to_string(&vec![
                StatusLineSegment {
                    id: "seg1".to_string(),
                    segment_type: "model".to_string(),
                    enabled: true,
                    label: None,
                    format: Some("short".to_string()),
                    color: Some("cyan".to_string()),
                    bg_color: None,
                    separator_char: None,
                    custom_text: None,
                    position: 0,
                },
                StatusLineSegment {
                    id: "seg2".to_string(),
                    segment_type: "separator".to_string(),
                    enabled: true,
                    label: None,
                    format: None,
                    color: Some("gray".to_string()),
                    bg_color: None,
                    separator_char: Some("|".to_string()),
                    custom_text: None,
                    position: 1,
                },
                StatusLineSegment {
                    id: "seg3".to_string(),
                    segment_type: "cost".to_string(),
                    enabled: true,
                    label: None,
                    format: Some("$0.00".to_string()),
                    color: Some("green".to_string()),
                    bg_color: None,
                    separator_char: None,
                    custom_text: None,
                    position: 2,
                },
            ]).unwrap()),
            generated_script: None,
            icon: None,
            author: None,
            homepage_url: None,
            tags: Some(vec!["custom".to_string()]),
        }
    }

    fn make_raw_request(name: &str) -> CreateStatusLineRequest {
        CreateStatusLineRequest {
            name: name.to_string(),
            description: Some("Test raw status line".to_string()),
            statusline_type: "raw".to_string(),
            package_name: None,
            install_command: None,
            run_command: None,
            raw_command: Some("echo 'hello'".to_string()),
            padding: None,
            segments_json: None,
            generated_script: None,
            icon: None,
            author: None,
            homepage_url: None,
            tags: None,
        }
    }

    fn make_premade_request(name: &str) -> CreateStatusLineRequest {
        CreateStatusLineRequest {
            name: name.to_string(),
            description: Some("Test premade status line".to_string()),
            statusline_type: "premade".to_string(),
            package_name: Some("claude-limitline".to_string()),
            install_command: Some("npm install -g claude-limitline".to_string()),
            run_command: Some("claude-limitline".to_string()),
            raw_command: None,
            padding: Some(0),
            segments_json: None,
            generated_script: None,
            icon: Some("⚡".to_string()),
            author: Some("tylergraydev".to_string()),
            homepage_url: Some("https://github.com/tylergraydev/claude-limitline".to_string()),
            tags: Some(vec!["premade".to_string(), "powerline".to_string()]),
        }
    }

    #[test]
    fn test_create_statusline_custom() {
        let db = Database::in_memory().unwrap();
        let req = make_custom_request("My Custom SL");
        let sl = db.create_statusline(&req).unwrap();

        assert_eq!(sl.name, "My Custom SL");
        assert_eq!(sl.statusline_type, "custom");
        assert_eq!(sl.padding, 1);
        assert!(!sl.is_active);
        assert!(sl.segments_json.is_some());

        // Verify segments can be parsed back
        let segments: Vec<StatusLineSegment> =
            serde_json::from_str(sl.segments_json.as_ref().unwrap()).unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].segment_type, "model");
        assert_eq!(segments[1].segment_type, "separator");
        assert_eq!(segments[2].segment_type, "cost");
    }

    #[test]
    fn test_create_statusline_raw() {
        let db = Database::in_memory().unwrap();
        let req = make_raw_request("My Raw SL");
        let sl = db.create_statusline(&req).unwrap();

        assert_eq!(sl.name, "My Raw SL");
        assert_eq!(sl.statusline_type, "raw");
        assert_eq!(sl.raw_command, Some("echo 'hello'".to_string()));
    }

    #[test]
    fn test_create_statusline_premade() {
        let db = Database::in_memory().unwrap();
        let req = make_premade_request("claude-limitline");
        let sl = db.create_statusline(&req).unwrap();

        assert_eq!(sl.name, "claude-limitline");
        assert_eq!(sl.statusline_type, "premade");
        assert_eq!(sl.package_name, Some("claude-limitline".to_string()));
        assert_eq!(sl.run_command, Some("claude-limitline".to_string()));
        assert_eq!(sl.author, Some("tylergraydev".to_string()));
    }

    #[test]
    fn test_get_all_statuslines() {
        let db = Database::in_memory().unwrap();
        db.create_statusline(&make_custom_request("SL1")).unwrap();
        db.create_statusline(&make_raw_request("SL2")).unwrap();
        db.create_statusline(&make_premade_request("SL3")).unwrap();

        let all = db.get_all_statuslines().unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_get_statusline_by_id() {
        let db = Database::in_memory().unwrap();
        let created = db.create_statusline(&make_custom_request("FindMe")).unwrap();

        let found = db.get_statusline_by_id(created.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "FindMe");

        // Non-existent ID
        let not_found = db.get_statusline_by_id(9999).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_update_statusline() {
        let db = Database::in_memory().unwrap();
        let created = db.create_statusline(&make_custom_request("Original")).unwrap();

        let mut updated = created.clone();
        updated.name = "Updated Name".to_string();
        updated.description = Some("Updated description".to_string());
        updated.padding = 3;

        let result = db.update_statusline(&updated).unwrap();
        assert_eq!(result.name, "Updated Name");
        assert_eq!(result.description, Some("Updated description".to_string()));
        assert_eq!(result.padding, 3);
    }

    #[test]
    fn test_delete_statusline() {
        let db = Database::in_memory().unwrap();
        let created = db.create_statusline(&make_custom_request("DeleteMe")).unwrap();

        db.delete_statusline(created.id).unwrap();
        let found = db.get_statusline_by_id(created.id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_set_active_statusline() {
        let db = Database::in_memory().unwrap();
        let sl1 = db.create_statusline(&make_custom_request("SL1")).unwrap();
        let sl2 = db.create_statusline(&make_raw_request("SL2")).unwrap();

        // Activate SL1
        db.set_active_statusline(sl1.id).unwrap();
        let active = db.get_active_statusline().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, sl1.id);

        // Activate SL2 — should deactivate SL1
        db.set_active_statusline(sl2.id).unwrap();
        let active = db.get_active_statusline().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, sl2.id);

        // Verify SL1 is no longer active
        let sl1_fresh = db.get_statusline_by_id(sl1.id).unwrap().unwrap();
        assert!(!sl1_fresh.is_active);
    }

    #[test]
    fn test_deactivate_all_statuslines() {
        let db = Database::in_memory().unwrap();
        let sl = db.create_statusline(&make_custom_request("Active SL")).unwrap();
        db.set_active_statusline(sl.id).unwrap();

        // Verify it's active
        assert!(db.get_active_statusline().unwrap().is_some());

        // Deactivate all
        db.deactivate_all_statuslines().unwrap();
        assert!(db.get_active_statusline().unwrap().is_none());
    }

    #[test]
    fn test_unique_name_constraint() {
        let db = Database::in_memory().unwrap();
        db.create_statusline(&make_custom_request("UniqueName")).unwrap();

        let result = db.create_statusline(&make_custom_request("UniqueName"));
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_script_full_segments() {
        let segments = vec![
            StatusLineSegment {
                id: "1".to_string(),
                segment_type: "model".to_string(),
                enabled: true,
                label: None,
                format: Some("short".to_string()),
                color: Some("cyan".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 0,
            },
            StatusLineSegment {
                id: "2".to_string(),
                segment_type: "separator".to_string(),
                enabled: true,
                label: None,
                format: None,
                color: Some("gray".to_string()),
                bg_color: None,
                separator_char: Some("|".to_string()),
                custom_text: None,
                position: 1,
            },
            StatusLineSegment {
                id: "3".to_string(),
                segment_type: "cost".to_string(),
                enabled: true,
                label: Some("Cost: ".to_string()),
                format: Some("$0.00".to_string()),
                color: Some("green".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 2,
            },
            StatusLineSegment {
                id: "4".to_string(),
                segment_type: "separator".to_string(),
                enabled: true,
                label: None,
                format: None,
                color: Some("gray".to_string()),
                bg_color: None,
                separator_char: Some("|".to_string()),
                custom_text: None,
                position: 3,
            },
            StatusLineSegment {
                id: "5".to_string(),
                segment_type: "context".to_string(),
                enabled: true,
                label: None,
                format: Some("percentage".to_string()),
                color: Some("yellow".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 4,
            },
            StatusLineSegment {
                id: "6".to_string(),
                segment_type: "separator".to_string(),
                enabled: true,
                label: None,
                format: None,
                color: Some("gray".to_string()),
                bg_color: None,
                separator_char: Some("|".to_string()),
                custom_text: None,
                position: 5,
            },
            StatusLineSegment {
                id: "7".to_string(),
                segment_type: "cwd".to_string(),
                enabled: true,
                label: None,
                format: Some("basename".to_string()),
                color: Some("blue".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: None,
                position: 6,
            },
            StatusLineSegment {
                id: "8".to_string(),
                segment_type: "custom_text".to_string(),
                enabled: true,
                label: None,
                format: None,
                color: Some("white".to_string()),
                bg_color: None,
                separator_char: None,
                custom_text: Some("🚀".to_string()),
                position: 7,
            },
        ];

        let script = statusline_writer::generate_script_from_segments(&segments);

        // Verify it's valid Python
        assert!(script.contains("#!/usr/bin/env python3"));
        assert!(script.contains("import sys"));
        assert!(script.contains("import json"));
        assert!(script.contains("def main():"));

        // Verify segment handlers are present
        assert!(script.contains("model"));
        assert!(script.contains("cost"));
        assert!(script.contains("context_window"));
        assert!(script.contains("cwd"));
        assert!(script.contains("🚀")); // custom_text

        // Verify ANSI color codes
        assert!(script.contains("\\033[")); // ANSI escape sequences
    }

    #[test]
    fn test_tags_serialization() {
        let db = Database::in_memory().unwrap();
        let req = make_premade_request("TaggedSL");
        let sl = db.create_statusline(&req).unwrap();

        assert_eq!(sl.tags, Some(vec!["premade".to_string(), "powerline".to_string()]));
    }
}
