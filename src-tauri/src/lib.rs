pub mod ai;
pub mod audit;
pub mod commands;
pub mod connectors;
pub mod crypto;
pub mod db;
pub mod models;
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
            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Projects
            commands::projects::get_projects,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::archive_project,
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
            commands::ai::generate_output,
            // Settings
            commands::settings::get_app_settings,
            commands::settings::set_app_setting,
            commands::settings::get_prompt_templates,
            commands::settings::save_prompt_template,
            // Export / Import
            commands::export::export_project,
            commands::export::export_all,
            commands::import::import_project,
            // Notifications
            commands::notifications::get_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_read,
            commands::notifications::create_notification,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
