use crate::db::Database;
use crate::services::{cache_service, content_extractor};
use chrono;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct ArticleSummary {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub source_id: i64,
    pub source_name: String,
    pub publish_date: String,
    pub category: String,
    pub is_cached: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListByCategoryParams {
    pub category: String,
    pub days_back: i64,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    /// When true, excludes articles published today (used so category sections
    /// don't duplicate articles already shown in the "Today" section).
    pub exclude_today: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ListByCategoryResponse {
    pub articles: Vec<ArticleSummary>,
    pub total_count: i64,
}

/// List articles by category with date filtering.
#[tauri::command]
pub fn articles_list_by_category(
    db: State<'_, Arc<Database>>,
    params: ListByCategoryParams,
) -> Result<ListByCategoryResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    // Build date filter using parameter binding to prevent SQL injection.
    // Compute the cutoff date in Rust and pass as a bound parameter.
    let cutoff_date = if params.days_back == 0 {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    } else {
        (chrono::Utc::now() - chrono::Duration::days(params.days_back))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string()
    };

    let (date_filter, date_param_idx) = if params.days_back == 0 {
        ("date(a.publish_date) = date('now')".to_string(), true)
    } else {
        ("a.publish_date >= ?2".to_string(), true)
    };
    let _ = date_param_idx; // used to track parameter numbering

    // Optionally exclude today's articles so they don't duplicate the Today section
    let today_exclusion = if params.exclude_today.unwrap_or(false) && params.days_back > 0 {
        " AND date(a.publish_date) != date('now')"
    } else {
        ""
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM articles a WHERE a.category = ?1 AND {date_filter}{today_exclusion}"
    );

    let total_count: i64 = conn
        .prepare(&count_sql)
        .map_err(|e| format!("DB error: {e}"))?
        .query_row(rusqlite::params![&params.category, &cutoff_date], |row| row.get(0))
        .map_err(|e| format!("DB error: {e}"))?;

    let query_sql = format!(
        "SELECT a.id, a.title, a.url, ns.name as source_name, a.publish_date, a.category, a.is_cached
         FROM articles a
         JOIN news_sources ns ON a.source_id = ns.id
         WHERE a.category = ?1 AND {date_filter}{today_exclusion}
         ORDER BY a.publish_date DESC
         LIMIT ?3 OFFSET ?4"
    );

    let mut stmt = conn
        .prepare(&query_sql)
        .map_err(|e| format!("DB error: {e}"))?;

    let articles = stmt
        .query_map(rusqlite::params![params.category, cutoff_date, limit, offset], |row| {
            Ok(ArticleSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                source_id: 0, // not fetched in category queries
                source_name: row.get(3)?,
                publish_date: row.get(4)?,
                category: row.get(5)?,
                is_cached: row.get::<_, i32>(6).map(|v| v != 0)?,
            })
        })
        .map_err(|e| format!("DB error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ListByCategoryResponse {
        articles,
        total_count,
    })
}

// ─── Today's Articles (all categories) ─────────────────────

#[derive(Debug, Deserialize)]
pub struct ListTodayParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List ALL articles published today, regardless of category (FR-002 revised).
#[tauri::command]
pub fn articles_list_today(
    db: State<'_, Arc<Database>>,
    params: ListTodayParams,
) -> Result<ListByCategoryResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or(0);

    let total_count: i64 = conn
        .prepare("SELECT COUNT(*) FROM articles a WHERE date(a.publish_date) = date('now') AND a.category != 'msrc'")
        .map_err(|e| format!("DB error: {e}"))?
        .query_row([], |row| row.get(0))
        .map_err(|e| format!("DB error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.title, a.url, ns.name as source_name, a.publish_date, a.category, a.is_cached
             FROM articles a
             JOIN news_sources ns ON a.source_id = ns.id
             WHERE date(a.publish_date) = date('now') AND a.category != 'msrc'
             ORDER BY a.publish_date DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let articles = stmt
        .query_map(rusqlite::params![limit, offset], |row| {
            Ok(ArticleSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                source_id: 0,
                source_name: row.get(3)?,
                publish_date: row.get(4)?,
                category: row.get(5)?,
                is_cached: row.get::<_, i32>(6).map(|v| v != 0)?,
            })
        })
        .map_err(|e| format!("DB error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ListByCategoryResponse {
        articles,
        total_count,
    })
}

// ─── Browse by Source (FR-033) ────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListBySourceParams {
    pub source_id: i64,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SourceInfo {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct ListBySourceResponse {
    pub source: SourceInfo,
    pub articles: Vec<ArticleSummary>,
    pub total: i64,
}

/// List articles by source with pagination (FR-033).
#[tauri::command]
pub fn articles_list_by_source(
    db: State<'_, Arc<Database>>,
    params: ListBySourceParams,
) -> Result<ListBySourceResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    // Fetch source info
    let source = conn
        .query_row(
            "SELECT id, name, feed_url, default_tag FROM news_sources WHERE id = ?1",
            [params.source_id],
            |row| {
                Ok(SourceInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    category: row.get(3)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "SOURCE_NOT_FOUND".to_string(),
            _ => format!("DB error: {e}"),
        })?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM articles WHERE source_id = ?1",
            [params.source_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.title, a.url, a.source_id, ns.name, a.publish_date, a.category, a.is_cached
             FROM articles a
             JOIN news_sources ns ON a.source_id = ns.id
             WHERE a.source_id = ?1
             ORDER BY a.publish_date DESC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let articles = stmt
        .query_map(rusqlite::params![params.source_id, limit, offset], |row| {
            Ok(ArticleSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                source_id: row.get(3)?,
                source_name: row.get(4)?,
                publish_date: row.get(5)?,
                category: row.get(6)?,
                is_cached: row.get::<_, i32>(7).map(|v| v != 0)?,
            })
        })
        .map_err(|e| format!("DB error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ListBySourceResponse {
        source,
        articles,
        total,
    })
}

// ─── Article Detail Types ─────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ArticleDetail {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub source_id: i64,
    pub source_name: String,
    pub publish_date: String,
    pub body_html: Option<String>,
    pub image_urls: Vec<String>,
    pub extraction_method: Option<String>,
    pub is_cached: bool,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct GetContentParams {
    pub article_id: i64,
}

#[derive(Debug, Serialize)]
pub struct GetContentResponse {
    pub article: ArticleDetail,
}

/// Get full article content, triggering extraction if not yet cached.
#[tauri::command]
pub async fn articles_get_content(
    db: State<'_, Arc<Database>>,
    params: GetContentParams,
) -> Result<GetContentResponse, String> {
    // First, check if article exists and already has content
    let article_row = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT a.id, a.title, a.url, a.source_id, ns.name as source_name, a.publish_date,
                        a.body_html, a.image_urls, a.extraction_method, a.is_cached, a.category
                 FROM articles a
                 JOIN news_sources ns ON a.source_id = ns.id
                 WHERE a.id = ?1",
            )
            .map_err(|e| format!("DB error: {e}"))?;

        let result = stmt
            .query_row([params.article_id], |row| {
                let image_urls_json: Option<String> = row.get(7)?;
                let image_urls: Vec<String> = image_urls_json
                    .and_then(|json| serde_json::from_str(&json).ok())
                    .unwrap_or_default();

                Ok(ArticleDetail {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    source_id: row.get(3)?,
                    source_name: row.get(4)?,
                    publish_date: row.get(5)?,
                    body_html: row.get(6)?,
                    image_urls,
                    extraction_method: row.get(8)?,
                    is_cached: row.get::<_, i32>(9).map(|v| v != 0)?,
                    category: row.get(10)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => "NOT_FOUND".to_string(),
                _ => format!("DB error: {e}"),
            })?;

        result
    };

    // If already extracted, return immediately
    if article_row.body_html.is_some() {
        return Ok(GetContentResponse {
            article: article_row,
        });
    }

    // Not yet extracted — run extraction pipeline
    let url = article_row.url.clone();
    match content_extractor::extract_content(&url).await {
        Ok(extraction) => {
            let method_str = extraction.method.as_str().to_string();
            let image_urls_json =
                serde_json::to_string(&extraction.image_urls).unwrap_or_else(|_| "[]".to_string());

            // Store extraction results in DB
            {
                let body_size = cache_service::compute_body_size(
                    &Some(extraction.body_html.clone()),
                    &Some(extraction.body_text.clone()),
                );
                let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
                conn.execute(
                    "UPDATE articles SET body_html = ?1, body_text = ?2, image_urls = ?3,
                     extraction_method = ?4, is_cached = 1, cached_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
                     body_size_bytes = ?6,
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     WHERE id = ?5",
                    rusqlite::params![
                        extraction.body_html,
                        extraction.body_text,
                        image_urls_json,
                        method_str,
                        params.article_id,
                        body_size
                    ],
                )
                .map_err(|e| format!("DB update error: {e}"))?;
            }

            Ok(GetContentResponse {
                article: ArticleDetail {
                    body_html: Some(extraction.body_html),
                    image_urls: extraction.image_urls,
                    extraction_method: Some(method_str),
                    is_cached: true,
                    ..article_row
                },
            })
        }
        Err(_) => {
            // Mark extraction as failed
            {
                let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
                conn.execute(
                    "UPDATE articles SET extraction_method = 'failed',
                     updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
                     WHERE id = ?1",
                    [params.article_id],
                )
                .map_err(|e| format!("DB update error: {e}"))?;
            }

            Ok(GetContentResponse {
                article: ArticleDetail {
                    extraction_method: Some("failed".to_string()),
                    ..article_row
                },
            })
        }
    }
}

// ─── WebView Fallback Extraction ──────────────────────────

#[derive(Debug, Deserialize)]
pub struct ExtractViaWebviewParams {
    pub article_id: i64,
    pub extracted_html: String,
}

#[derive(Debug, Serialize)]
pub struct ExtractViaWebviewResponse {
    pub article: ArticleDetail,
}

/// Store WebView-extracted content for an article.
#[tauri::command]
pub fn articles_extract_via_webview(
    db: State<'_, Arc<Database>>,
    params: ExtractViaWebviewParams,
) -> Result<ExtractViaWebviewResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    // Sanitize the incoming HTML to prevent stored XSS
    let sanitized_html = content_extractor::sanitize_html(&params.extracted_html);

    // Strip HTML to get plain text for FTS
    let body_text = {
        let doc = scraper::Html::parse_fragment(&sanitized_html);
        doc.root_element()
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };

    // Extract image URLs
    let image_urls: Vec<String> = {
        let doc = scraper::Html::parse_fragment(&sanitized_html);
        let selector = scraper::Selector::parse("img[src]").unwrap();
        doc.select(&selector)
            .filter_map(|el| el.value().attr("src").map(String::from))
            .collect()
    };

    let image_urls_json =
        serde_json::to_string(&image_urls).unwrap_or_else(|_| "[]".to_string());

    let body_size = cache_service::compute_body_size(
        &Some(sanitized_html.clone()),
        &Some(body_text.clone()),
    );

    conn.execute(
        "UPDATE articles SET body_html = ?1, body_text = ?2, image_urls = ?3,
         extraction_method = 'webview', is_cached = 1,
         cached_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now'),
         body_size_bytes = ?5,
         updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
         WHERE id = ?4",
        rusqlite::params![sanitized_html, body_text, image_urls_json, params.article_id, body_size],
    )
    .map_err(|e| format!("DB update error: {e}"))?;

    // Re-read the article to return complete data
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.title, a.url, a.source_id, ns.name as source_name, a.publish_date,
                    a.body_html, a.image_urls, a.extraction_method, a.is_cached, a.category
             FROM articles a
             JOIN news_sources ns ON a.source_id = ns.id
             WHERE a.id = ?1",
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let article = stmt
        .query_row([params.article_id], |row| {
            let img_json: Option<String> = row.get(7)?;
            let imgs: Vec<String> = img_json
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

            Ok(ArticleDetail {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                source_id: row.get(3)?,
                source_name: row.get(4)?,
                publish_date: row.get(5)?,
                body_html: row.get(6)?,
                image_urls: imgs,
                extraction_method: row.get(8)?,
                is_cached: row.get::<_, i32>(9).map(|v| v != 0)?,
                category: row.get(10)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "NOT_FOUND".to_string(),
            _ => format!("DB error: {e}"),
        })?;

    Ok(ExtractViaWebviewResponse { article })
}

// ─── Cache Management ─────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CacheStatusResponse {
    pub cached_articles: i64,
    pub total_articles: i64,
    pub oldest_cached: Option<String>,
    pub body_cache_size_bytes: i64,
    pub cache_size_limit_gb: f64,
    pub size_cap_pct: f64,
}

/// Get cache statistics for the Settings screen.
#[tauri::command]
pub fn articles_get_cache_status(
    db: State<'_, Arc<Database>>,
) -> Result<CacheStatusResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    let total_articles: i64 = conn
        .query_row("SELECT COUNT(*) FROM articles", [], |row| row.get(0))
        .map_err(|e| format!("DB error: {e}"))?;

    let cached_articles: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM articles WHERE is_cached = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let oldest_cached: Option<String> = conn
        .query_row(
            "SELECT MIN(cached_at) FROM articles WHERE is_cached = 1 AND cached_at IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .ok();

    // Use the pre-computed body_size_bytes column
    let body_cache_size_bytes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(body_size_bytes), 0) FROM articles WHERE is_cached = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let cache_size_limit_gb: f64 = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'article_cache_size_limit_gb'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20.0);

    let limit_bytes = cache_size_limit_gb * 1_073_741_824.0;
    let size_cap_pct = if limit_bytes > 0.0 {
        ((body_cache_size_bytes as f64 / limit_bytes) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    Ok(CacheStatusResponse {
        cached_articles,
        total_articles,
        oldest_cached,
        body_cache_size_bytes,
        cache_size_limit_gb,
        size_cap_pct,
    })
}

/// Manually trigger cache eviction based on configured retention period.
#[tauri::command]
pub fn articles_evict_cache(
    db: State<'_, Arc<Database>>,
) -> Result<EvictCacheResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    // Read retention days from settings
    let retention_days: i64 = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'cache_retention_days'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    let evicted_count = crate::services::cache_service::evict_expired_cache(&conn, retention_days)
        .map_err(|e| format!("Cache eviction error: {e}"))?;

    Ok(EvictCacheResponse { evicted_count })
}

#[derive(Debug, Serialize)]
pub struct EvictCacheResponse {
    pub evicted_count: i64,
}
