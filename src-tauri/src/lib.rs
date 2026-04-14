pub mod ai;
pub mod commands;
pub mod connectors;
pub mod db;
pub mod models;
pub mod utils;

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
            // Documents
            commands::documents::upload_document,
            commands::documents::upload_text,
            commands::documents::get_documents_for_project,
            commands::documents::delete_document,
            commands::documents::get_document_content,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
