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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn setup() -> Connection {
        let conn = open_in_memory();
        conn.execute(
            "INSERT INTO news_sources (id, name, feed_url) VALUES (1, 'Test', 'https://feed.test/rss')",
            [],
        )
        .unwrap();
        conn
    }

    fn insert_cached(conn: &Connection, id: i64, days_ago: i64, size: i64) {
        let cached_at = (chrono::Utc::now() - chrono::Duration::days(days_ago))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        conn.execute(
            "INSERT INTO articles (id, source_id, url, title, body_html, body_text, publish_date,
                                   is_cached, cached_at, body_size_bytes, extraction_method)
             VALUES (?1, 1, ?2, 'Title', '<p>body</p>', 'body', '2026-01-01T00:00:00Z',
                     1, ?3, ?4, 'readability')",
            rusqlite::params![id, format!("https://a.test/{id}"), cached_at, size],
        )
        .unwrap();
    }

    fn is_cached(conn: &Connection, id: i64) -> bool {
        conn.query_row("SELECT is_cached FROM articles WHERE id = ?1", [id], |r| r.get::<_, i64>(0))
            .unwrap()
            == 1
    }

    #[test]
    fn evicts_only_articles_older_than_retention() {
        let conn = setup();
        insert_cached(&conn, 1, 40, 100);
        insert_cached(&conn, 2, 5, 100);

        let evicted = evict_expired_cache(&conn, 30).unwrap();

        assert_eq!(evicted, 1);
        assert!(!is_cached(&conn, 1));
        assert!(is_cached(&conn, 2));

        let (body, size): (Option<String>, i64) = conn
            .query_row("SELECT body_html, body_size_bytes FROM articles WHERE id = 1", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(body, None);
        assert_eq!(size, 0);
    }

    #[test]
    fn evicted_article_row_is_kept() {
        let conn = setup();
        insert_cached(&conn, 1, 40, 100);
        evict_expired_cache(&conn, 30).unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM articles", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn size_cap_evicts_oldest_first_until_under_limit() {
        let conn = setup();
        insert_cached(&conn, 1, 3, 400);
        insert_cached(&conn, 2, 2, 400);
        insert_cached(&conn, 3, 1, 400);

        let evicted = evict_by_size_cap(&conn, 500).unwrap();

        assert_eq!(evicted, 2);
        assert!(!is_cached(&conn, 1));
        assert!(!is_cached(&conn, 2));
        assert!(is_cached(&conn, 3));
    }

    #[test]
    fn size_cap_is_noop_when_under_limit() {
        let conn = setup();
        insert_cached(&conn, 1, 1, 100);
        assert_eq!(evict_by_size_cap(&conn, 1_000).unwrap(), 0);
        assert!(is_cached(&conn, 1));
    }

    #[test]
    fn evict_on_launch_uses_configured_retention() {
        let conn = setup();
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('cache_retention_days', '7')",
            [],
        )
        .unwrap();
        insert_cached(&conn, 1, 10, 100);
        insert_cached(&conn, 2, 3, 100);

        assert_eq!(evict_on_launch(&conn).unwrap(), 1);
        assert!(!is_cached(&conn, 1));
        assert!(is_cached(&conn, 2));
    }

    #[test]
    fn compute_body_size_sums_both_fields() {
        assert_eq!(compute_body_size(&Some("abc".into()), &Some("de".into())), 5);
        assert_eq!(compute_body_size(&None, &None), 0);
    }
}
