mod commands;
mod db;
mod models;
mod services;

use db::Database;
use tauri::Manager;

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

            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::settings_get_all,
            commands::settings::settings_update,
            commands::settings::settings_set_api_key,
            commands::settings::settings_clear_api_key,
            commands::settings::settings_get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
