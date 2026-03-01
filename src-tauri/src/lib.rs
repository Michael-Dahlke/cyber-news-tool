mod commands;
mod db;
mod models;
mod services;

use db::Database;
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = db::get_db_path(&app.handle());
            log::info!("Database path: {}", db_path.display());

            let database = Database::new(&db_path)
                .expect("Failed to initialize database");

            // Seed default news sources on first run
            {
                let conn = database.conn.lock().expect("Failed to lock DB for seeding");
                db::seed::seed_default_sources(&conn)
                    .expect("Failed to seed default sources");
            }

            // Run cache eviction on launch (FR-038)
            {
                let conn = database.conn.lock().expect("Failed to lock DB for cache eviction");
                if let Err(e) = services::cache_service::evict_on_launch(&conn) {
                    log::warn!("Cache eviction on launch failed: {e}");
                }
            }

            // Read auto-refresh interval from settings
            let refresh_interval_min: u64 = {
                let conn = database.conn.lock().expect("Failed to lock DB for settings");
                conn.query_row(
                    "SELECT value FROM app_settings WHERE key = 'feed_refresh_interval_min'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60)
            };

            // Wrap in Arc for sharing with background task
            let db = Arc::new(database);

            // Manage the Arc<Database> — commands use State<'_, Arc<Database>>
            app.manage(db.clone());

            // Spawn auto-refresh background task
            let app_handle = app.handle().clone();
            let db_for_refresh = db.clone();
            tauri::async_runtime::spawn(async move {
                let interval = std::time::Duration::from_secs(refresh_interval_min * 60);
                loop {
                    tokio::time::sleep(interval).await;
                    log::info!("Auto-refresh: starting feed refresh");

                    let result = services::feed_service::refresh_all_feeds(db_for_refresh.clone(), None).await;

                    let timestamp = chrono::Utc::now().to_rfc3339();
                    let _ = app_handle.emit("feeds://auto-refresh-complete", serde_json::json!({
                        "articles_added": result.articles_added,
                        "timestamp": timestamp,
                    }));

                    log::info!(
                        "Auto-refresh complete: {} sources fetched, {} articles added, {} errors",
                        result.sources_fetched,
                        result.articles_added,
                        result.errors.len()
                    );
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::settings_get_all,
            commands::settings::settings_update,
            commands::settings::settings_set_api_key,
            commands::settings::settings_clear_api_key,
            commands::settings::settings_get_app_info,
            commands::feeds::feeds_list_sources,
            commands::feeds::feeds_add_source,
            commands::feeds::feeds_update_source,
            commands::feeds::feeds_remove_source,
            commands::feeds::feeds_refresh,
            commands::articles::articles_list_by_category,
            commands::articles::articles_get_content,
            commands::articles::articles_extract_via_webview,
            commands::articles::articles_get_cache_status,
            commands::articles::articles_evict_cache,
            commands::search::search_query,
            commands::search::search_suggest,
            commands::kql::kql_generate,
            commands::kql::kql_get_for_article,
            commands::kql::kql_copy_to_clipboard,
            commands::kql::kql_check_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
