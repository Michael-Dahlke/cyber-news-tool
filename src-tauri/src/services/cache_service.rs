use rusqlite::Connection;

/// Evict cached article content older than the retention period.
/// Clears body_html, body_text, and resets cache flags for expired articles.
/// Returns the number of articles evicted.
pub fn evict_expired_cache(conn: &Connection, retention_days: i64) -> Result<i64, String> {
    let cutoff = format!(
        "strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-{} days')",
        retention_days
    );

    let sql = format!(
        "UPDATE articles
         SET body_html = NULL,
             body_text = NULL,
             image_urls = NULL,
             is_cached = 0,
             cached_at = NULL,
             extraction_method = NULL,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE is_cached = 1
           AND cached_at IS NOT NULL
           AND cached_at < {cutoff}"
    );

    let evicted = conn
        .execute(&sql, [])
        .map_err(|e| format!("Cache eviction SQL error: {e}"))?;

    if evicted > 0 {
        log::info!("Cache eviction: cleared {evicted} articles older than {retention_days} days");
    }

    Ok(evicted as i64)
}

/// Run cache eviction on app launch using the configured retention period.
pub fn evict_on_launch(conn: &Connection) -> Result<i64, String> {
    let retention_days: i64 = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'cache_retention_days'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    evict_expired_cache(conn, retention_days)
}
