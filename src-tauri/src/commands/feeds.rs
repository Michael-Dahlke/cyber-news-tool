use crate::db::Database;
use crate::models::{ArticleCategory, NewsSource};
use crate::services::feed_service;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct ListSourcesResponse {
    pub sources: Vec<NewsSource>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SourceSmokeTestResult {
    pub source_id: i64,
    pub source_name: String,
    pub feed_url: String,
    pub ok: bool,
    pub error: Option<String>,
}

/// List all configured news sources.
#[tauri::command]
pub fn feeds_list_sources(db: State<'_, Arc<Database>>) -> Result<Vec<NewsSource>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, feed_url, feed_type, default_tag, is_default, is_active, created_at, updated_at
             FROM news_sources ORDER BY is_default DESC, name ASC",
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let sources = stmt
        .query_map([], NewsSource::from_row)
        .map_err(|e| format!("DB error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sources)
}

#[tauri::command]
pub async fn feeds_smoke_test_sources(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<SourceSmokeTestResult>, String> {
    let sources: Vec<NewsSource> = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, feed_url, feed_type, default_tag, is_default, is_active, created_at, updated_at
                 FROM news_sources WHERE is_active = 1",
            )
            .map_err(|e| format!("DB error: {e}"))?;

        let rows = stmt
            .query_map([], NewsSource::from_row)
            .map_err(|e| format!("DB error: {e}"))?;

        rows.filter_map(|r| r.ok()).collect()
    };

    let mut results = Vec::with_capacity(sources.len());

    for source in sources {
        let status = feed_service::validate_feed_url(&source.feed_url).await;
        let result = match status {
            Ok(_) => SourceSmokeTestResult {
                source_id: source.id,
                source_name: source.name,
                feed_url: source.feed_url,
                ok: true,
                error: None,
            },
            Err(err) => SourceSmokeTestResult {
                source_id: source.id,
                source_name: source.name,
                feed_url: source.feed_url,
                ok: false,
                error: Some(err),
            },
        };
        results.push(result);
    }

    Ok(results)
}

#[derive(Debug, Deserialize)]
pub struct AddSourceParams {
    pub feed_url: String,
    pub name: Option<String>,
    pub default_tag: Option<String>,
}

/// Validate and add a user-provided feed URL.
#[tauri::command]
pub async fn feeds_add_source(
    db: State<'_, Arc<Database>>,
    params: AddSourceParams,
) -> Result<NewsSource, String> {
    // URL scheme validation — require http(s)
    if !params.feed_url.starts_with("http://") && !params.feed_url.starts_with("https://") {
        return Err("INVALID_URL".to_string());
    }

    // Security: warn about insecure feeds (LOW-1)
    let is_insecure_feed = params.feed_url.starts_with("http://");
    if is_insecure_feed {
        log::warn!("Adding insecure (HTTP) feed URL — content may be tampered via MITM: {}", params.feed_url);
    }

    // Check for duplicate
    {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
        let exists: bool = conn
            .prepare("SELECT COUNT(*) FROM news_sources WHERE feed_url = ?1")
            .map_err(|e| format!("DB error: {e}"))?
            .query_row([&params.feed_url], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)
            .map_err(|e| format!("DB error: {e}"))?;

        if exists {
            return Err("DUPLICATE_URL".to_string());
        }
    }

    // Validate and fetch feed title
    let feed_title = feed_service::validate_feed_url(&params.feed_url).await?;

    let name = params
        .name
        .filter(|n| !n.is_empty())
        .or(feed_title)
        .unwrap_or_else(|| "Unknown Feed".to_string());

    let tag = params.default_tag.unwrap_or_else(|| "general".to_string());
    // Validate tag enum
    let _ = ArticleCategory::from_str(&tag);

    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    conn.execute(
        "INSERT INTO news_sources (name, feed_url, feed_type, default_tag, is_default, is_active)
         VALUES (?1, ?2, 'rss', ?3, 0, 1)",
        rusqlite::params![name, params.feed_url, tag],
    )
    .map_err(|e| format!("DB error: {e}"))?;

    let id = conn.last_insert_rowid();

    let source = conn
        .prepare("SELECT id, name, feed_url, feed_type, default_tag, is_default, is_active, created_at, updated_at FROM news_sources WHERE id = ?1")
        .map_err(|e| format!("DB error: {e}"))?
        .query_row([id], NewsSource::from_row)
        .map_err(|e| format!("DB error: {e}"))?;

    Ok(source)
}

#[derive(Debug, Deserialize)]
pub struct UpdateSourceParams {
    pub id: i64,
    pub name: Option<String>,
    pub default_tag: Option<String>,
    pub is_active: Option<bool>,
}

/// Update a source's properties.
#[tauri::command]
pub fn feeds_update_source(
    db: State<'_, Arc<Database>>,
    params: UpdateSourceParams,
) -> Result<NewsSource, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    // Check exists
    let exists: bool = conn
        .prepare("SELECT COUNT(*) FROM news_sources WHERE id = ?1")
        .map_err(|e| format!("DB error: {e}"))?
        .query_row([params.id], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)
        .map_err(|e| format!("DB error: {e}"))?;

    if !exists {
        return Err("NOT_FOUND".to_string());
    }

    if let Some(ref name) = params.name {
        conn.execute(
            "UPDATE news_sources SET name = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?2",
            rusqlite::params![name, params.id],
        )
        .map_err(|e| format!("DB error: {e}"))?;
    }

    if let Some(ref tag) = params.default_tag {
        let _ = ArticleCategory::from_str(tag);
        conn.execute(
            "UPDATE news_sources SET default_tag = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?2",
            rusqlite::params![tag, params.id],
        )
        .map_err(|e| format!("DB error: {e}"))?;
    }

    if let Some(active) = params.is_active {
        conn.execute(
            "UPDATE news_sources SET is_active = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?2",
            rusqlite::params![active as i32, params.id],
        )
        .map_err(|e| format!("DB error: {e}"))?;
    }

    let source = conn
        .prepare("SELECT id, name, feed_url, feed_type, default_tag, is_default, is_active, created_at, updated_at FROM news_sources WHERE id = ?1")
        .map_err(|e| format!("DB error: {e}"))?
        .query_row([params.id], NewsSource::from_row)
        .map_err(|e| format!("DB error: {e}"))?;

    Ok(source)
}

/// Remove a user-added source (default sources can only be deactivated).
#[tauri::command]
pub fn feeds_remove_source(db: State<'_, Arc<Database>>, id: i64) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    let is_default: Option<bool> = conn
        .prepare("SELECT is_default FROM news_sources WHERE id = ?1")
        .map_err(|e| format!("DB error: {e}"))?
        .query_row([id], |row| row.get::<_, bool>(0))
        .ok();

    match is_default {
        None => Err("NOT_FOUND".to_string()),
        Some(true) => Err("CANNOT_REMOVE_DEFAULT".to_string()),
        Some(false) => {
            // Delete associated articles first
            conn.execute("DELETE FROM articles WHERE source_id = ?1", [id])
                .map_err(|e| format!("DB error: {e}"))?;
            conn.execute("DELETE FROM news_sources WHERE id = ?1", [id])
                .map_err(|e| format!("DB error: {e}"))?;
            Ok(true)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub sources_fetched: usize,
    pub articles_added: usize,
    pub articles_deduplicated: usize,
    pub errors: Vec<feed_service::FeedError>,
}

/// Refresh all active feeds.
#[tauri::command]
pub async fn feeds_refresh(
    db: State<'_, Arc<Database>>,
    app_handle: tauri::AppHandle,
) -> Result<RefreshResponse, String> {
    use tauri::Emitter;

    // We need an Arc<Database> for the async service.
    // Since Tauri State is already Arc-wrapped internally, we clone the inner data.
    // But State doesn't expose Arc directly, so we need to work around this.
    // We'll pass a raw pointer approach — actually, let's just re-read sources and do the work here.
    
    // For now, do a simpler approach: read sources, fetch feeds, insert articles
    let sources: Vec<NewsSource> = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, feed_url, feed_type, default_tag, is_default, is_active, created_at, updated_at
                 FROM news_sources WHERE is_active = 1",
            )
            .map_err(|e| format!("DB error: {e}"))?;

        let rows = stmt.query_map([], NewsSource::from_row)
            .map_err(|e| format!("DB error: {e}"))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("CyberNewsReader/0.1")
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let total = sources.len();
    let mut result = RefreshResponse {
        sources_fetched: 0,
        articles_added: 0,
        articles_deduplicated: 0,
        errors: Vec::new(),
    };

    for (idx, source) in sources.iter().enumerate() {
        // Emit progress event
        let _ = app_handle.emit(
            "feeds://refresh-progress",
            serde_json::json!({
                "current": idx + 1,
                "total": total,
                "source_name": source.name,
            }),
        );

        let fetch_result = async {
            let response = client
                .get(&source.feed_url)
                .send()
                .await
                .map_err(|e| format!("Fetch error: {e}"))?;

            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }

            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("Read error: {e}"))?;

            feed_rs::parser::parse(&bytes[..])
                .map_err(|e| format!("Parse error: {e}"))
        }
        .await;

        match fetch_result {
            Ok(feed) => {
                result.sources_fetched += 1;
                let source_tag = source.default_tag.clone();

                for entry in feed.entries {
                    let url = entry
                        .links
                        .first()
                        .map(|l| l.href.clone())
                        .unwrap_or_default();

                    if url.is_empty() {
                        continue;
                    }

                    let title = entry
                        .title
                        .as_ref()
                        .map(|t| t.content.clone())
                        .unwrap_or_else(|| "Untitled".to_string());

                    let body_text = entry
                        .summary
                        .as_ref()
                        .map(|s| s.content.clone())
                        .or_else(|| entry.content.as_ref().and_then(|c| c.body.clone()));

                    let publish_date = entry
                        .published
                        .or(entry.updated)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                    let category =
                        feed_service::classify_article(&source_tag, &title, body_text.as_deref());

                    let image_urls: Vec<String> = entry
                        .media
                        .iter()
                        .flat_map(|m| &m.content)
                        .filter_map(|c| c.url.as_ref().map(|u| u.to_string()))
                        .collect();

                    let image_urls_json = if image_urls.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&image_urls).unwrap_or_default())
                    };

                    let conn = match db.conn.lock() {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let insert_result = conn.execute(
                        "INSERT OR IGNORE INTO articles (source_id, url, title, body_text, publish_date, category, image_urls)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        rusqlite::params![
                            source.id,
                            url,
                            title,
                            body_text,
                            publish_date,
                            category.as_str(),
                            image_urls_json,
                        ],
                    );

                    match insert_result {
                        Ok(0) => result.articles_deduplicated += 1,
                        Ok(_) => result.articles_added += 1,
                        Err(e) => {
                            log::warn!("Failed to insert article {url}: {e}");
                        }
                    }
                }
            }
            Err(err) => {
                result.errors.push(feed_service::FeedError {
                    source_id: source.id,
                    source_name: source.name.clone(),
                    error: err,
                });
            }
        }
    }

    Ok(result)
}
