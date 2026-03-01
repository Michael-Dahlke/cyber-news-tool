use crate::db::Database;
use crate::models::{AppInfo, AppSettings, SettingsUpdate, Theme};
use crate::services::keychain;
use std::sync::Arc;
use tauri::State;

/// Retrieve all application settings.
#[tauri::command]
pub fn settings_get_all(db: State<'_, Arc<Database>>) -> Result<AppSettings, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    let mut settings = AppSettings::default();

    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| format!("DB error: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
            ))
        })
        .map_err(|e| format!("DB error: {e}"))?;

    for row in rows {
        let (key, value) = row.map_err(|e| format!("DB row error: {e}"))?;
        match key.as_str() {
            "llm_endpoint" => settings.llm_endpoint = value,
            "llm_model" => settings.llm_model = value,
            "feed_refresh_interval_min" => {
                settings.feed_refresh_interval_min = value.parse().unwrap_or(60);
            }
            "cache_retention_days" => {
                settings.cache_retention_days = value.parse().unwrap_or(30);
            }
            "theme" => settings.theme = Theme::from_str(&value),
            _ => {}
        }
    }

    settings.has_api_key = keychain::has_api_key();

    Ok(settings)
}

/// Update one or more settings.
#[tauri::command]
pub fn settings_update(
    db: State<'_, Arc<Database>>,
    update: SettingsUpdate,
) -> Result<AppSettings, String> {
    // Validate
    if let Some(interval) = update.feed_refresh_interval_min {
        if interval < 1 {
            return Err("INVALID_INTERVAL".to_string());
        }
    }
    if let Some(days) = update.cache_retention_days {
        if days < 1 {
            return Err("INVALID_RETENTION".to_string());
        }
    }

    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    let upsert_sql = "INSERT INTO app_settings (key, value, updated_at) VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at";

    if let Some(v) = &update.llm_endpoint {
        conn.execute(upsert_sql, rusqlite::params![&"llm_endpoint", v])
            .map_err(|e| format!("DB error: {e}"))?;
    }
    if let Some(v) = &update.llm_model {
        conn.execute(upsert_sql, rusqlite::params![&"llm_model", v])
            .map_err(|e| format!("DB error: {e}"))?;
    }
    if let Some(v) = update.feed_refresh_interval_min {
        conn.execute(upsert_sql, rusqlite::params![&"feed_refresh_interval_min", v.to_string()])
            .map_err(|e| format!("DB error: {e}"))?;
    }
    if let Some(v) = update.cache_retention_days {
        conn.execute(upsert_sql, rusqlite::params![&"cache_retention_days", v.to_string()])
            .map_err(|e| format!("DB error: {e}"))?;
    }
    if let Some(v) = &update.theme {
        // Validate theme value
        let _ = Theme::from_str(v);
        conn.execute(upsert_sql, rusqlite::params![&"theme", v])
            .map_err(|e| format!("DB error: {e}"))?;
    }

    drop(conn);

    // Return updated settings by re-reading from DB
    settings_get_all(db)
}

/// Store the LLM API key in the OS keychain.
#[tauri::command]
pub fn settings_set_api_key(api_key: String) -> Result<bool, String> {
    keychain::set_api_key(&api_key)?;
    Ok(true)
}

/// Remove the LLM API key from the OS keychain.
#[tauri::command]
pub fn settings_clear_api_key() -> Result<bool, String> {
    keychain::clear_api_key()?;
    Ok(true)
}

/// Return application metadata.
#[tauri::command]
pub fn settings_get_app_info(
    app_handle: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
) -> Result<AppInfo, String> {
    use crate::db::get_db_path;

    let db_path = get_db_path(&app_handle);
    let db_path_str = db_path.to_string_lossy().to_string();

    // Get version from Cargo package metadata (embedded at compile time)
    let version = env!("CARGO_PKG_VERSION").to_string();

    let os = std::env::consts::OS.to_string();

    // Verify DB is accessible
    let _conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    Ok(AppInfo {
        version,
        db_path: db_path_str,
        os,
    })
}
