mod commands;
mod db;
mod models;
mod services;

use db::Database;
use std::sync::Arc;
use tauri::{Emitter, Manager};

/// Read the `last_refresh_at` timestamp from app_settings, returning None if absent.
fn read_last_refresh_at(db: &Database) -> Option<chrono::DateTime<chrono::Utc>> {
    let conn = db.conn.lock().ok()?;
    let ts: String = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'last_refresh_at'",
            [],
            |row| row.get(0),
        )
        .ok()?;
    chrono::DateTime::parse_from_rfc3339(&ts)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

/// Write (upsert) `last_refresh_at` with the current UTC timestamp.
fn write_last_refresh_at(db: &Database) {
    if let Ok(conn) = db.conn.lock() {
        let now = chrono::Utc::now().to_rfc3339();
        let _ = conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('last_refresh_at', ?1, ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            [&now],
        );
    }
}

/// Returns true when `now - last_refresh_at > threshold_hours`, or when no previous refresh exists.
fn should_refresh(db: &Database, threshold_hours: u64) -> bool {
    match read_last_refresh_at(db) {
        None => true, // never refreshed
        Some(last) => {
            let elapsed = chrono::Utc::now().signed_duration_since(last);
            elapsed.num_seconds() > (threshold_hours as i64) * 3600
        }
    }
}

/// Perform a refresh, emit event, and record the timestamp.
async fn do_refresh(db: Arc<Database>, app_handle: tauri::AppHandle) {
    log::info!("Feed refresh: starting");
    let result = services::feed_service::refresh_all_feeds(db.clone(), None).await;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let _ = app_handle.emit(
        "feeds://auto-refresh-complete",
        serde_json::json!({
            "articles_added": result.articles_added,
            "timestamp": timestamp,
        }),
    );

    write_last_refresh_at(&db);

    log::info!(
        "Feed refresh complete: {} sources fetched, {} articles added, {} errors",
        result.sources_fetched,
        result.articles_added,
        result.errors.len()
    );
}

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

            // Read background refresh threshold from settings (FR-006)
            let refresh_threshold_hours: u64 = {
                let conn = database.conn.lock().expect("Failed to lock DB for settings");
                conn.query_row(
                    "SELECT value FROM app_settings WHERE key = 'background_refresh_threshold_hours'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4)
            };

            // Wrap in Arc for sharing with background task
            let db = Arc::new(database);

            // Manage the Arc<Database> — commands use State<'_, Arc<Database>>
            app.manage(db.clone());

            // ── FR-006: Cold-launch refresh ──────────────────────────
            // On every app start, refresh feeds if threshold exceeded (or if
            // no previous refresh exists). This replaces the old periodic timer.
            if should_refresh(&db, refresh_threshold_hours) {
                let db_launch = db.clone();
                let handle_launch = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    do_refresh(db_launch, handle_launch).await;
                });
            }

            // ── FR-006: Window-focus refresh ─────────────────────────
            // When the window regains focus after being in the background for
            // longer than the threshold, trigger a single refresh.
            let db_focus = db.clone();
            let handle_focus = app.handle().clone();
            let main_window = app
                .get_webview_window("main")
                .expect("main window must exist");

            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(true) = event {
                    if should_refresh(&db_focus, refresh_threshold_hours) {
                        let db_inner = db_focus.clone();
                        let handle_inner = handle_focus.clone();
                        tauri::async_runtime::spawn(async move {
                            do_refresh(db_inner, handle_inner).await;
                        });
                    }
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
            commands::articles::articles_list_by_source,
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
