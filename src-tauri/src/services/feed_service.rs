use crate::db::Database;
use crate::models::{ArticleCategory, NewsSource};
use chrono::Utc;
use feed_rs::parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Result of refreshing all active feeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    pub sources_fetched: usize,
    pub articles_added: usize,
    pub articles_deduplicated: usize,
    pub errors: Vec<FeedError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedError {
    pub source_id: i64,
    pub source_name: String,
    pub error: String,
}

/// Microsoft-related keywords for category override (title + body matching).
const MICROSOFT_KEYWORDS: &[&str] = &[
    "microsoft", "azure", "windows", "defender", "sentinel", "entra",
    "office 365", "m365", "exchange", "sharepoint", "teams", "copilot",
    "active directory", "msrc", "patch tuesday",
];

/// Threat intelligence keywords for category override.
const THREAT_INTEL_KEYWORDS: &[&str] = &[
    "cve-", "vulnerability", "exploit", "zero-day", "0-day", "zero day",
    "ransomware", "malware", "apt", "threat actor", "campaign",
    "ioc", "indicator of compromise", "ttps", "mitre att&ck",
    "advisory", "cisa", "cert", "botnet", "backdoor", "phishing",
    "supply chain attack", "data breach",
];

/// Classify an article based on source default tag and keyword matching.
pub fn classify_article(
    source_tag: &ArticleCategory,
    title: &str,
    body_text: Option<&str>,
) -> ArticleCategory {
    // If source is specifically tagged (non-general), respect it
    if *source_tag != ArticleCategory::General {
        return source_tag.clone();
    }

    let text_lower = format!(
        "{} {}",
        title.to_lowercase(),
        body_text.unwrap_or("").to_lowercase()
    );

    // Check Microsoft keywords first (more specific)
    for kw in MICROSOFT_KEYWORDS {
        if text_lower.contains(kw) {
            return ArticleCategory::Microsoft;
        }
    }

    // Check threat intelligence keywords
    for kw in THREAT_INTEL_KEYWORDS {
        if text_lower.contains(kw) {
            return ArticleCategory::ThreatIntel;
        }
    }

    ArticleCategory::General
}

/// Fetch and parse a single feed, returning parsed entries.
async fn fetch_feed(
    client: &Client,
    source: &NewsSource,
) -> Result<Vec<feed_rs::model::Entry>, String> {
    let response = client
        .get(&source.feed_url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch {}: {e}", source.feed_url))?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {} for {}",
            response.status(),
            source.feed_url
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    let feed = parser::parse(&bytes[..])
        .map_err(|e| format!("Failed to parse feed {}: {e}", source.feed_url))?;

    Ok(feed.entries)
}

/// Validate that a URL is a reachable, parseable feed. Returns the feed title if detected.
pub async fn validate_feed_url(url: &str) -> Result<Option<String>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| "UNREACHABLE".to_string())?;

    if !response.status().is_success() {
        return Err("UNREACHABLE".to_string());
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|_| "UNREACHABLE".to_string())?;

    let feed = parser::parse(&bytes[..]).map_err(|_| "UNPARSEABLE_FEED".to_string())?;

    Ok(feed.title.map(|t| t.content))
}

/// Refresh all active feeds and insert new articles into the database.
pub async fn refresh_all_feeds(
    db: Arc<Database>,
    progress_callback: Option<Box<dyn Fn(usize, usize, &str) + Send>>,
) -> RefreshResult {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("CyberNewsReader/0.1")
        .build()
        .unwrap_or_default();

    // Load active sources
    let sources: Vec<NewsSource> = {
        let conn = match db.conn.lock() {
            Ok(c) => c,
            Err(_) => {
                return RefreshResult {
                    sources_fetched: 0,
                    articles_added: 0,
                    articles_deduplicated: 0,
                    errors: vec![FeedError {
                        source_id: 0,
                        source_name: "DB".to_string(),
                        error: "Failed to lock database".to_string(),
                    }],
                }
            }
        };

        let result: Result<Vec<NewsSource>, String> = (|| {
            let mut stmt = conn.prepare(
                "SELECT id, name, feed_url, feed_type, default_tag, is_default, is_active, created_at, updated_at
                 FROM news_sources WHERE is_active = 1",
            ).map_err(|e| format!("Query error: {e}"))?;

            let rows = stmt.query_map([], |row| NewsSource::from_row(row))
                .map_err(|e| format!("Query error: {e}"))?;
            Ok(rows.filter_map(|r| r.ok()).collect())
        })();

        match result {
            Ok(sources) => sources,
            Err(e) => {
                return RefreshResult {
                    sources_fetched: 0,
                    articles_added: 0,
                    articles_deduplicated: 0,
                    errors: vec![FeedError {
                        source_id: 0,
                        source_name: "DB".to_string(),
                        error: e,
                    }],
                }
            }
        }
    };

    let total = sources.len();
    let mut result = RefreshResult {
        sources_fetched: 0,
        articles_added: 0,
        articles_deduplicated: 0,
        errors: Vec::new(),
    };

    for (idx, source) in sources.iter().enumerate() {
        if let Some(ref cb) = progress_callback {
            cb(idx + 1, total, &source.name);
        }

        match fetch_feed(&client, source).await {
            Ok(entries) => {
                result.sources_fetched += 1;
                let source_tag = source.default_tag.clone();

                for entry in entries {
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
                        .or_else(|| {
                            entry
                                .content
                                .as_ref()
                                .and_then(|c| c.body.clone())
                        });

                    let publish_date = entry
                        .published
                        .or(entry.updated)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| Utc::now().to_rfc3339());

                    let category = classify_article(&source_tag, &title, body_text.as_deref());

                    // Extract image URLs from media content
                    let image_urls: Vec<String> = entry
                        .media
                        .iter()
                        .flat_map(|m| &m.content)
                        .filter_map(|c| {
                            c.url.as_ref().map(|u| u.to_string())
                        })
                        .collect();

                    let image_urls_json = if image_urls.is_empty() {
                        None
                    } else {
                        Some(serde_json::to_string(&image_urls).unwrap_or_default())
                    };

                    // Insert with URL deduplication (FR-037)
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
                result.errors.push(FeedError {
                    source_id: source.id,
                    source_name: source.name.clone(),
                    error: err,
                });
            }
        }
    }

    result
}
