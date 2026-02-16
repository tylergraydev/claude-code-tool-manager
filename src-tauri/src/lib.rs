use std::sync::{Arc, Mutex};
use tauri::Manager;

mod commands;
mod db;
mod mcp_gateway;
mod mcp_server;
mod services;
mod utils;

use db::Database;
use mcp_gateway::server::{GatewayServerConfig, GatewayServerState, DEFAULT_GATEWAY_PORT};
use mcp_server::server::{McpServerConfig, McpServerState, DEFAULT_MCP_SERVER_PORT};
use services::mcp_session::McpSessionManager;

pub fn run() {
    env_logger::init();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());

    // Enable MCP bridge plugin for development/testing
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            // Restore debug mode early if it was persisted (before any other initialization)
            // This ensures we capture startup logs
            if let Err(e) = services::debug_logger::init_from_persisted(&app_data_dir) {
                log::warn!("Failed to restore debug mode: {}", e);
            }

            let db_path = app_data_dir.join("mcp_library.db");
            let database = Database::new(&db_path)?;
            database.run_migrations()?;

            // Seed default repos
            if let Err(e) = services::repo_sync::seed_default_repos(&database) {
                log::error!("Failed to seed default repos: {}", e);
            }

            // Wrap database in Arc<Mutex> for sharing with MCP server
            let database_arc = Arc::new(Mutex::new(database));
            app.manage(database_arc.clone());

            // Initialize session manager for MCP execution
            app.manage(Mutex::new(McpSessionManager::new()));

            // Initialize MCP server state with config from database
            let mcp_server_config = {
                let db = database_arc.lock().unwrap();
                let enabled = db.get_setting("mcp_server_enabled")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(true);
                let port = db.get_setting("mcp_server_port")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_MCP_SERVER_PORT);
                let auto_start = db.get_setting("mcp_server_auto_start")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(true);
                McpServerConfig { enabled, port, auto_start }
            };

            let mcp_server_state = Arc::new(McpServerState::with_config(mcp_server_config.clone()));
            app.manage(mcp_server_state.clone());

            // Initialize Gateway server state with config from database
            let gateway_config = {
                let db = database_arc.lock().unwrap();
                let enabled = db.get_setting("gateway_enabled")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(false);
                let port = db.get_setting("gateway_port")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_GATEWAY_PORT);
                let auto_start = db.get_setting("gateway_auto_start")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(false);
                GatewayServerConfig { enabled, port, auto_start }
            };

            let gateway_state = Arc::new(GatewayServerState::with_config(gateway_config.clone(), database_arc.clone()));
            app.manage(gateway_state.clone());

            // Run startup scan
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = services::scanner::run_startup_scan(&app_handle).await {
                    log::error!("Startup scan failed: {}", e);
                }
            });

            // Auto-start MCP server if configured
            if mcp_server_config.enabled && mcp_server_config.auto_start {
                let server_state = mcp_server_state.clone();
                let db_for_server = database_arc.clone();
                let db_for_library = database_arc.clone();
                tauri::async_runtime::spawn(async move {
                    // Small delay to let the app fully initialize
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    if let Err(e) = server_state.start(db_for_server).await {
                        log::error!("[McpServer] Auto-start failed: {}", e);
                    } else {
                        log::info!("[McpServer] Auto-started successfully on port {}", server_state.get_port());

                        // Auto-add the Tool Manager MCP to the library
                        if let Ok(db) = db_for_library.lock() {
                            let entry = mcp_server::server::generate_self_mcp_entry(server_state.get_port());
                            match db.get_mcp_by_name(&entry.name) {
                                Ok(Some(existing)) => {
                                    // Update URL, type, and source to ensure consistency
                                    let mut updated = existing.clone();
                                    updated.url = entry.url;
                                    updated.mcp_type = entry.mcp_type;
                                    updated.source = "system".to_string();
                                    if let Err(e) = db.update_mcp(&updated) {
                                        log::error!("[McpServer] Failed to update self MCP in library: {}", e);
                                    }
                                }
                                Ok(None) => {
                                    // Add new entry as a system MCP (readonly)
                                    if let Err(e) = db.create_system_mcp(&entry) {
                                        log::error!("[McpServer] Failed to add self MCP to library: {}", e);
                                    } else {
                                        log::info!("[McpServer] Tool Manager MCP added to library");
                                    }
                                }
                                Err(e) => {
                                    log::error!("[McpServer] Failed to check for existing self MCP: {}", e);
                                }
                            }
                        }
                    }
                });
            }

            // Auto-start Gateway server if configured
            if gateway_config.enabled && gateway_config.auto_start {
                let gw_state = gateway_state.clone();
                let db_for_gateway = database_arc.clone();
                tauri::async_runtime::spawn(async move {
                    // Delay a bit more than MCP server to avoid port conflicts during startup
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    if let Err(e) = gw_state.start().await {
                        log::error!("[Gateway] Auto-start failed: {}", e);
                    } else {
                        let status = gw_state.get_status().await;
                        log::info!(
                            "[Gateway] Auto-started successfully on port {} with {} backends and {} tools",
                            gw_state.get_port(),
                            status.connected_backends.len(),
                            status.total_tools
                        );

                        // Auto-add the Gateway MCP to the library as a system MCP
                        if let Ok(db) = db_for_gateway.lock() {
                            let entry = mcp_gateway::server::generate_gateway_mcp_entry(gw_state.get_port());
                            match db.get_mcp_by_name(&entry.name) {
                                Ok(Some(existing)) => {
                                    // Update URL, type, and source to ensure consistency
                                    let mut updated = existing.clone();
                                    updated.url = entry.url;
                                    updated.mcp_type = entry.mcp_type;
                                    updated.source = "system".to_string();
                                    if let Err(e) = db.update_mcp(&updated) {
                                        log::error!("[Gateway] Failed to update Gateway MCP in library: {}", e);
                                    }
                                }
                                Ok(None) => {
                                    // Add new entry as a system MCP (readonly)
                                    if let Err(e) = db.create_system_mcp(&entry) {
                                        log::error!("[Gateway] Failed to add Gateway MCP to library: {}", e);
                                    } else {
                                        log::info!("[Gateway] MCP Gateway added to library");
                                    }
                                }
                                Err(e) => {
                                    log::error!("[Gateway] Failed to check for existing Gateway MCP: {}", e);
                                }
                            }
                        }
                    }
                });
            }

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
            commands::mcp::toggle_mcp_favorite,
            // Project Commands
            commands::projects::get_all_projects,
            commands::projects::add_project,
            commands::projects::remove_project,
            commands::projects::browse_for_project,
            commands::projects::assign_mcp_to_project,
            commands::projects::remove_mcp_from_project,
            commands::projects::toggle_project_mcp,
            commands::projects::toggle_project_favorite,
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
            commands::skills::toggle_skill_favorite,
            // Slash Command Commands
            commands::commands::get_all_commands,
            commands::commands::create_command,
            commands::commands::update_command,
            commands::commands::delete_command,
            commands::commands::get_global_commands,
            commands::commands::add_global_command,
            commands::commands::remove_global_command,
            commands::commands::toggle_global_command,
            commands::commands::assign_command_to_project,
            commands::commands::remove_command_from_project,
            commands::commands::toggle_project_command,
            commands::commands::get_project_commands,
            commands::commands::toggle_command_favorite,
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
            commands::subagents::toggle_subagent_favorite,
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
            commands::hooks::export_hooks_to_json,
            commands::hooks::create_sound_notification_hooks,
            commands::hooks::duplicate_hook,
            // Sound Commands
            commands::sounds::get_system_sounds,
            commands::sounds::get_custom_sounds,
            commands::sounds::preview_sound,
            commands::sounds::ensure_sounds_directory,
            commands::sounds::upload_custom_sound,
            commands::sounds::delete_custom_sound,
            commands::sounds::generate_sound_hook_command,
            commands::sounds::deploy_notification_script,
            commands::sounds::get_sounds_directory,
            commands::sounds::validate_sound_file,
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
            // Settings Commands
            commands::settings::get_app_settings,
            commands::settings::update_app_settings,
            commands::settings::get_available_editors,
            commands::settings::get_opencode_paths_cmd,
            commands::settings::get_codex_paths_cmd,
            commands::settings::get_copilot_paths_cmd,
            commands::settings::get_cursor_paths_cmd,
            commands::settings::get_gemini_paths_cmd,
            commands::settings::toggle_editor,
            commands::settings::set_github_token,
            commands::settings::clear_github_token,
            commands::settings::has_github_token,
            // Profile Commands
            commands::profiles::get_all_profiles,
            commands::profiles::get_profile,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::profiles::capture_profile_from_current,
            commands::profiles::activate_profile,
            commands::profiles::deactivate_profile,
            commands::profiles::get_active_profile,
            // StatusLine Commands
            commands::statusline::get_all_statuslines,
            commands::statusline::get_statusline,
            commands::statusline::create_statusline,
            commands::statusline::update_statusline,
            commands::statusline::delete_statusline,
            commands::statusline::activate_statusline,
            commands::statusline::deactivate_statusline,
            commands::statusline::get_active_statusline,
            commands::statusline::fetch_statusline_gallery,
            commands::statusline::get_statusline_gallery_cache,
            commands::statusline::install_premade_statusline,
            commands::statusline::get_statusline_gallery_url,
            commands::statusline::set_statusline_gallery_url,
            commands::statusline::generate_statusline_preview,
            commands::statusline::read_current_statusline_config,
            // Spinner Verb Commands
            commands::spinner_verbs::get_all_spinner_verbs,
            commands::spinner_verbs::create_spinner_verb,
            commands::spinner_verbs::update_spinner_verb,
            commands::spinner_verbs::delete_spinner_verb,
            commands::spinner_verbs::reorder_spinner_verbs,
            commands::spinner_verbs::get_spinner_verb_mode,
            commands::spinner_verbs::set_spinner_verb_mode,
            commands::spinner_verbs::sync_spinner_verbs,
            commands::spinner_verbs::read_current_spinner_verbs_config,
            // Debug Commands
            commands::debug::enable_debug_mode,
            commands::debug::disable_debug_mode,
            commands::debug::is_debug_mode_enabled,
            commands::debug::get_debug_log_path,
            commands::debug::open_logs_folder,
            commands::debug::write_frontend_log,
            commands::debug::write_invoke_log,
            // MCP Test Commands
            commands::mcp_test::test_mcp,
            commands::mcp_test::test_mcp_config,
            // MCP Session Commands
            commands::mcp_session::start_mcp_session,
            commands::mcp_session::execute_tool,
            commands::mcp_session::end_mcp_session,
            commands::mcp_session::list_mcp_sessions,
            commands::mcp_session::get_mcp_session,
            commands::mcp_session::get_session_tools,
            commands::mcp_session::cleanup_idle_sessions,
            // MCP Server Commands
            commands::mcp_server::get_mcp_server_status,
            commands::mcp_server::get_mcp_server_config,
            commands::mcp_server::update_mcp_server_config,
            commands::mcp_server::start_mcp_server,
            commands::mcp_server::stop_mcp_server,
            commands::mcp_server::get_mcp_server_connection_config,
            commands::mcp_server::add_self_mcp_to_library,
            commands::mcp_server::remove_self_mcp_from_library,
            commands::mcp_server::is_self_mcp_in_library,
            // Permission Commands
            commands::permissions::get_all_permissions,
            commands::permissions::set_permission_rules,
            commands::permissions::add_permission_rule,
            commands::permissions::remove_permission_rule,
            commands::permissions::reorder_permission_rules,
            commands::permissions::set_default_mode,
            commands::permissions::set_additional_directories,
            commands::permissions::get_permission_templates,
            commands::permissions::seed_permission_templates,
            // Memory Commands
            commands::memory::get_all_memory_files,
            commands::memory::get_memory_file,
            commands::memory::save_memory_file,
            commands::memory::delete_memory_file,
            commands::memory::create_memory_file,
            commands::memory::detect_project_memory_location,
            commands::memory::render_markdown,
            // MCP Gateway Commands
            commands::mcp_gateway::get_gateway_status,
            commands::mcp_gateway::get_gateway_config,
            commands::mcp_gateway::update_gateway_config,
            commands::mcp_gateway::start_gateway,
            commands::mcp_gateway::stop_gateway,
            commands::mcp_gateway::get_gateway_connection_config,
            commands::mcp_gateway::get_gateway_mcps,
            commands::mcp_gateway::add_mcp_to_gateway,
            commands::mcp_gateway::remove_mcp_from_gateway,
            commands::mcp_gateway::toggle_gateway_mcp,
            commands::mcp_gateway::set_gateway_mcp_auto_restart,
            commands::mcp_gateway::is_mcp_in_gateway,
            commands::mcp_gateway::get_gateway_backends,
            commands::mcp_gateway::restart_gateway_backend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
