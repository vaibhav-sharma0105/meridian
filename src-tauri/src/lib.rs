pub mod ai;
pub mod audit;
pub mod commands;
pub mod connectors;
pub mod crypto;
pub mod daemon;
pub mod db;
pub mod documents;
pub mod drafts;
pub mod integrations;
pub mod models;
pub mod patterns;
pub mod plans;
pub mod sensitive;
pub mod skills;
pub mod suggestions;
pub mod utils;
pub mod vectors;

use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let conn = db::connection::init_db().expect("Failed to initialize database");

            // Seed builtin skills on first run
            if let Err(e) = skills::builtin::load_builtin_skills(&conn) {
                eprintln!("Warning: Failed to load builtin skills: {}", e);
            }

            app.manage(AppState {
                db: Mutex::new(conn),
            });
            app.manage(commands::embedding_worker::WorkerState::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Projects
            commands::projects::get_projects,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::archive_project,
            commands::projects::get_archived_projects,
            commands::projects::unarchive_project,
            // Meetings
            commands::meetings::ingest_meeting,
            commands::meetings::ingest_meeting_from_file,
            commands::meetings::get_meetings_for_project,
            commands::meetings::get_meeting,
            commands::meetings::delete_meeting,
            commands::meetings::force_delete_meeting,
            commands::meetings::unarchive_meeting,
            commands::meetings::rename_meeting,
            commands::meetings::count_moveable_tasks,
            commands::meetings::move_meeting_to_project,
            // Tasks
            commands::tasks::get_tasks_for_project,
            commands::tasks::get_all_tasks,
            commands::tasks::create_task,
            commands::tasks::update_task,
            commands::tasks::bulk_update_tasks,
            commands::tasks::reorder_tasks,
            commands::tasks::delete_task,
            commands::tasks::archive_task,
            commands::tasks::unarchive_task,
            commands::tasks::move_task_to_project,
            // Documents
            commands::documents::upload_document,
            commands::documents::upload_text,
            commands::documents::get_documents_for_project,
            commands::documents::delete_document,
            commands::documents::get_document_content,
            commands::documents::find_orphaned_documents,
            commands::documents::recover_orphaned_document,
            commands::documents::get_document_embedding_status,
            commands::documents::retry_document_embedding,
            commands::documents::get_embedding_migration_status,
            commands::documents::queue_embedding_migration,
            // AI
            commands::ai::verify_ai_connection,
            commands::ai::fetch_available_models,
            commands::ai::save_ai_settings,
            commands::ai::get_ai_settings,
            commands::ai::extract_tasks_from_transcript,
            commands::ai::chat_with_project,
            commands::ai::check_ollama_status,
            commands::ai::embed_document_chunks,
            commands::ai::search_documents,
            commands::ai::hybrid_search_documents,
            commands::ai::generate_output,
            // Settings
            commands::settings::get_app_settings,
            commands::settings::set_app_setting,
            commands::settings::get_prompt_templates,
            commands::settings::save_prompt_template,
            commands::settings::get_mcp_permissions,
            commands::settings::set_mcp_permissions,
            // Export / Import
            commands::export::export_project,
            commands::export::export_all,
            commands::import::import_project,
            // Notifications
            commands::notifications::get_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_read,
            commands::notifications::create_notification,
            commands::notifications::create_notification_with_options,
            commands::notifications::check_notification_permission,
            commands::notifications::request_notification_permission,
            // Updater
            commands::updater::check_for_updates,
            commands::updater::backup_database,
            // Connections
            commands::connections::connect_zoom,
            commands::connections::connect_gmail,
            commands::connections::get_connection,
            commands::connections::disconnect_provider,
            commands::connections::sync_connections,
            commands::connections::get_pending_imports,
            commands::connections::count_pending_imports,
            commands::connections::approve_import,
            commands::connections::dismiss_import,
            commands::connections::open_url,
            // Sheets Relay
            commands::connections::save_sheets_relay_config,
            commands::connections::test_sheets_relay,
            commands::connections::reset_sheets_relay_sync,
            // Audit
            commands::audit::get_audit_log,
            commands::audit::export_audit_log,
            commands::audit::prune_old_audit_logs,
            commands::audit::get_audit_log_stats,
            // Encryption
            commands::encryption::get_encryption_status,
            commands::encryption::check_password_strength,
            // Daemon
            commands::daemon::get_daemon_status,
            commands::daemon::start_daemon,
            commands::daemon::stop_daemon,
            commands::daemon::daemon_health_check,
            // Migration
            commands::migration::get_migration_status,
            commands::migration::migrate_database,
            commands::migration::list_backups,
            commands::migration::cleanup_old_backups,
            commands::migration::restore_from_backup,
            commands::migration::get_safe_backup_dir_path,
            commands::migration::list_safe_backups_cmd,
            commands::migration::restore_safe_backup,
            // Scheduler
            commands::scheduler::get_scheduler_status,
            commands::scheduler::enable_system_scheduler,
            commands::scheduler::disable_system_scheduler,
            // Embedding Worker
            commands::embedding_worker::start_embedding_worker,
            commands::embedding_worker::stop_embedding_worker,
            commands::embedding_worker::get_indexing_status,
            commands::embedding_worker::process_pending_embeddings,
            // Patterns
            commands::patterns::get_pattern_summaries,
            commands::patterns::get_pattern_model,
            commands::patterns::get_workflow_suggestions,
            commands::patterns::dismiss_workflow_suggestion,
            commands::patterns::get_smart_defaults,
            commands::patterns::get_communication_style,
            commands::patterns::record_draft_edit,
            commands::patterns::export_learning_data,
            commands::patterns::import_learning_data,
            commands::patterns::reset_pattern_category,
            commands::patterns::reset_all_learning,
            // Suggestions
            commands::suggestions::get_pending_suggestions,
            commands::suggestions::accept_suggestion,
            commands::suggestions::dismiss_suggestion,
            commands::suggestions::stop_suggesting,
            commands::suggestions::create_suggestion,
            commands::suggestions::get_suggestion_count_today,
            // Drafts
            commands::drafts::get_drafts_for_task,
            commands::drafts::generate_draft,
            commands::drafts::update_draft,
            commands::drafts::delete_draft,
            commands::drafts::scan_draft,
            // Plans
            commands::plans::evaluate_task_plan,
            commands::plans::get_task_plan,
            commands::plans::accept_plan,
            commands::plans::record_plan_correction,
            // Skills
            commands::skills::create_skill,
            commands::skills::get_skill,
            commands::skills::list_skills,
            commands::skills::update_skill,
            commands::skills::delete_skill,
            commands::skills::toggle_skill_enabled,
            commands::skills::run_skill_manually,
            commands::skills::test_run_skill,
            commands::skills::get_skill_runs,
            commands::skills::get_skill_run,
            commands::skills::approve_skill_run,
            commands::skills::reject_skill_run,
            commands::skills::clone_skill,
            commands::skills::export_skill,
            commands::skills::export_skill_to_directory,
            commands::skills::import_skill,
            commands::skills::get_skill_stats,
            commands::skills::record_skill_output_edit,
            commands::skills::extract_skill_from_chat,
            commands::skills::initialize_builtin_skills,
            commands::skills::reset_builtin_skills,
            commands::skills::pick_folder_dialog,
            commands::skills::list_skill_folders,
            commands::skills::get_skill_folder,
            commands::skills::install_skill_folder,
            commands::skills::delete_skill_folder,
            commands::skills::read_skill_file,
            commands::skills::execute_skill_script,
            commands::skills::toggle_folder_skill_enabled,
            // Integrations
            commands::integrations::list_integrations,
            commands::integrations::get_integration,
            commands::integrations::get_available_integrations,
            commands::integrations::create_integration,
            commands::integrations::update_integration,
            commands::integrations::delete_integration,
            commands::integrations::start_oauth_flow,
            commands::integrations::handle_oauth_callback,
            commands::integrations::refresh_integration_token,
            commands::integrations::sync_integration,
            commands::integrations::get_sync_status,
            commands::integrations::clear_integration_cache,
            commands::integrations::get_cached_items,
            commands::integrations::create_integration_link,
            commands::integrations::get_links_for_task,
            commands::integrations::get_links_for_meeting,
            commands::integrations::unlink_integration_item,
            commands::integrations::get_slack_socket_status,
            commands::integrations::detect_slack_action_items,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
