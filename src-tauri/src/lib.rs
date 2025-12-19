use tauri::Manager;
use std::sync::Mutex;

mod commands;
mod db;
mod services;
mod utils;

use db::Database;

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("mcp_library.db");
            let database = Database::new(&db_path)?;
            database.run_migrations()?;

            app.manage(Mutex::new(database));

            // Run startup scan
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = services::scanner::run_startup_scan(&app_handle).await {
                    log::error!("Startup scan failed: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // MCP Commands
            commands::mcp::get_all_mcps,
            commands::mcp::get_mcp,
            commands::mcp::create_mcp,
            commands::mcp::update_mcp,
            commands::mcp::delete_mcp,
            commands::mcp::duplicate_mcp,
            commands::mcp::toggle_global_mcp,

            // Project Commands
            commands::projects::get_all_projects,
            commands::projects::add_project,
            commands::projects::remove_project,
            commands::projects::browse_for_project,
            commands::projects::assign_mcp_to_project,
            commands::projects::remove_mcp_from_project,
            commands::projects::toggle_project_mcp,
            commands::projects::sync_project_config,

            // Global Settings Commands
            commands::config::get_global_mcps,
            commands::config::add_global_mcp,
            commands::config::remove_global_mcp,
            commands::config::toggle_global_mcp_assignment,
            commands::config::sync_global_config,
            commands::config::get_claude_paths,
            commands::config::open_config_file,
            commands::config::backup_configs,

            // Scanner Commands
            commands::scanner::scan_claude_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
