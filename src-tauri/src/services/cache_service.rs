use rusqlite::Connection;
use chrono;
// Note: chrono is imported for date computation in eviction cutoff

/// Evict cached article content older than the retention period.
/// Clears body_html, body_text, and resets cache flags for expired articles.
/// Returns the number of articles evicted.
pub fn evict_expired_cache(conn: &Connection, retention_days: i64) -> Result<i64, String> {
    // Compute the cutoff date in Rust to avoid SQL format-string injection.
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(retention_days))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let sql = "UPDATE articles
         SET body_html = NULL,
             body_text = NULL,
             image_urls = NULL,
             is_cached = 0,
             cached_at = NULL,
             body_size_bytes = 0,
             extraction_method = NULL,
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE is_cached = 1
           AND cached_at IS NOT NULL
           AND cached_at < ?1";

    let evicted = conn
        .execute(sql, rusqlite::params![cutoff])
        .map_err(|e| format!("Cache eviction SQL error: {e}"))?;

    if evicted > 0 {
        log::info!("Cache eviction: cleared {evicted} articles older than {retention_days} days");
    }

    Ok(evicted as i64)
}

/// Evict cached articles by size: delete oldest-cached articles until total
/// body_size_bytes drops below `limit_bytes`. Returns number of articles evicted.
pub fn evict_by_size_cap(conn: &Connection, limit_bytes: i64) -> Result<i64, String> {
    let total: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(body_size_bytes), 0) FROM articles WHERE is_cached = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Cache size query error: {e}"))?;

    if total <= limit_bytes {
        return Ok(0);
    }

    // Select oldest-cached articles to evict, in batches
    let mut evicted: i64 = 0;
    let mut remaining = total;

    loop {
        if remaining <= limit_bytes {
            break;
        }

        // Fetch a batch of 50 oldest-cached article IDs
        let mut stmt = conn
            .prepare(
                "SELECT id, body_size_bytes FROM articles
                 WHERE is_cached = 1 AND cached_at IS NOT NULL
                 ORDER BY cached_at ASC
                 LIMIT 50",
            )
            .map_err(|e| format!("Cache size eviction query error: {e}"))?;

        let batch: Vec<(i64, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Cache size eviction row error: {e}"))?
            .filter_map(|r| r.ok())
            .collect();

        if batch.is_empty() {
            break;
        }

        for (id, size) in &batch {
            conn.execute(
                "UPDATE articles SET body_html = NULL, body_text = NULL, image_urls = NULL,
                 is_cached = 0, cached_at = NULL, body_size_bytes = 0, extraction_method = NULL,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                 WHERE id = ?1",
                [id],
            )
            .map_err(|e| format!("Cache size eviction update error: {e}"))?;

            remaining -= size;
            evicted += 1;

            if remaining <= limit_bytes {
                break;
            }
        }
    }

    if evicted > 0 {
        log::info!("Cache size-cap eviction: cleared {evicted} articles to stay under {limit_bytes} bytes");
    }

    Ok(evicted)
}

/// Compute body_size_bytes from body_html + body_text for an article.
pub fn compute_body_size(body_html: &Option<String>, body_text: &Option<String>) -> i64 {
    let html_len = body_html.as_ref().map_or(0, |s| s.len() as i64);
    let text_len = body_text.as_ref().map_or(0, |s| s.len() as i64);
    html_len + text_len
}

/// Run cache eviction on app launch using the configured retention period
/// and size cap.
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

    let time_evicted = evict_expired_cache(conn, retention_days)?;

    // Size-cap eviction (FR-038)
    let limit_gb: i64 = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'article_cache_size_limit_gb'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let limit_bytes = limit_gb * 1_073_741_824;
    let size_evicted = evict_by_size_cap(conn, limit_bytes)?;

    Ok(time_evicted + size_evicted)
}
