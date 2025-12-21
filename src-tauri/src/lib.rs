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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("mcp_library.db");
            let database = Database::new(&db_path)?;
            database.run_migrations()?;

            // Seed default repos
            if let Err(e) = services::repo_sync::seed_default_repos(&database) {
                log::error!("Failed to seed default repos: {}", e);
            }

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

            // Claude.json Commands
            commands::claude_json::get_claude_json_mcps,
            commands::claude_json::get_claude_json_projects,
            commands::claude_json::add_mcp_to_claude_json,
            commands::claude_json::remove_mcp_from_claude_json,
            commands::claude_json::toggle_mcp_in_claude_json,
            commands::claude_json::add_global_mcp_to_claude_json,
            commands::claude_json::remove_global_mcp_from_claude_json,

            // Skill Commands
            commands::skills::get_all_skills,
            commands::skills::create_skill,
            commands::skills::update_skill,
            commands::skills::delete_skill,
            commands::skills::get_global_skills,
            commands::skills::add_global_skill,
            commands::skills::remove_global_skill,
            commands::skills::toggle_global_skill,
            commands::skills::assign_skill_to_project,
            commands::skills::remove_skill_from_project,
            commands::skills::toggle_project_skill,
            commands::skills::get_project_skills,
            commands::skills::get_skill_files,
            commands::skills::create_skill_file,
            commands::skills::update_skill_file,
            commands::skills::delete_skill_file,

            // Sub-Agent Commands
            commands::subagents::get_all_subagents,
            commands::subagents::create_subagent,
            commands::subagents::update_subagent,
            commands::subagents::delete_subagent,
            commands::subagents::get_global_subagents,
            commands::subagents::add_global_subagent,
            commands::subagents::remove_global_subagent,
            commands::subagents::toggle_global_subagent,
            commands::subagents::assign_subagent_to_project,
            commands::subagents::remove_subagent_from_project,
            commands::subagents::toggle_project_subagent,
            commands::subagents::get_project_subagents,

            // Hook Commands
            commands::hooks::get_all_hooks,
            commands::hooks::get_hook_templates,
            commands::hooks::create_hook,
            commands::hooks::create_hook_from_template,
            commands::hooks::update_hook,
            commands::hooks::delete_hook,
            commands::hooks::get_global_hooks,
            commands::hooks::add_global_hook,
            commands::hooks::remove_global_hook,
            commands::hooks::toggle_global_hook,
            commands::hooks::get_project_hooks,
            commands::hooks::assign_hook_to_project,
            commands::hooks::remove_hook_from_project,
            commands::hooks::toggle_project_hook,
            commands::hooks::seed_hook_templates,

            // Repos (Marketplace) Commands
            commands::repos::get_all_repos,
            commands::repos::add_repo,
            commands::repos::remove_repo,
            commands::repos::toggle_repo,
            commands::repos::get_repo_items,
            commands::repos::get_all_repo_items,
            commands::repos::sync_repo,
            commands::repos::sync_all_repos,
            commands::repos::import_repo_item,
            commands::repos::get_github_rate_limit,
            commands::repos::seed_default_repos,
            commands::repos::reset_repos_to_defaults,

            // MCP Registry Commands
            commands::mcp_registry::search_mcp_registry,
            commands::mcp_registry::list_mcp_registry,
            commands::mcp_registry::get_mcp_from_registry,
            commands::mcp_registry::import_mcp_from_registry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
