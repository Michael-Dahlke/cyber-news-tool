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
    "microsoft", "azure", "windows defender", "microsoft defender",
    "sentinel", "entra id", "entra ", "office 365", "m365",
    "exchange server", "sharepoint", "microsoft teams", "ms copilot",
    "active directory", "msrc", "patch tuesday", "azure ad",
    "microsoft security", "windows server", "intune",
];

/// Strong, unambiguous cybersecurity-specific threat intel keywords.
/// An article must contain at least one of these to be classified as threat_intel.
/// These terms are essentially never used outside of a security context.
const THREAT_INTEL_STRONG: &[&str] = &[
    "cve-20",          // CVE identifier (e.g. CVE-2024-...)
    "ransomware",
    "malware",
    "exploit",
    "zero-day",
    "0-day",
    "threat actor",
    "apt ",            // APT group (with trailing space to avoid "apt" in other words)
    "apt1", "apt28", "apt29", "apt41", // named APT groups
    "indicator of compromise",
    "ioc ",
    "ttps",
    "mitre att&ck",
    "command and control",
    "c2 server",
    "botnet",
    "backdoor",
    "rootkit",
    "spyware",
    "keylogger",
    "trojan",
    "worm ",           // malicious worm (trailing space)
    "nation-state",
    "cyber espionage",
    "cyberattack",
    "cyber attack",
    "threat campaign",
    "supply chain attack",
    "phishing campaign",
    "spear phishing",
];

/// Supplementary threat intel context keywords — these alone are NOT enough,
/// but if paired with a strong keyword via source context they reinforce classification.
const THREAT_INTEL_GENERAL: &[&str] = &[
    "vulnerability",
    "patch tuesday",
    "security advisory",
    "cisa ",
    "us-cert",
    "security bulletin",
    "security update",
    "remote code execution",
    "privilege escalation",
    "denial of service",
    "sql injection",
    "cross-site scripting",
    "authentication bypass",
];

/// Minimum number of THREAT_INTEL_GENERAL matches required when no strong indicator present.
const GENERAL_KEYWORD_THRESHOLD: usize = 2;

/// Classify an article based on source default tag and keyword matching.
///
/// Rules (in order):
/// 1. Non-general sources keep their tag unchanged.
/// 2. Title matches a strong Microsoft keyword → microsoft.
/// 3. Text contains at least 1 strong threat-intel indicator → threat_intel.
/// 4. Text contains ≥ GENERAL_KEYWORD_THRESHOLD general security keywords → threat_intel.
/// 5. Otherwise → general.
pub fn classify_article(
    source_tag: &ArticleCategory,
    title: &str,
    body_text: Option<&str>,
) -> ArticleCategory {
    // If source is specifically tagged (non-general), respect it.
    if *source_tag != ArticleCategory::General {
        return source_tag.clone();
    }

    // Only search the title for classification — body text from RSS summaries
    // is often short and may contain incidental matches. Using just the title
    // keeps classification focused on the article's actual subject.
    let title_lower = title.to_lowercase();
    // Use a short snippet of body text (first 500 chars) to limit false positives
    let body_snippet = body_text
        .unwrap_or("")
        .chars()
        .take(500)
        .collect::<String>()
        .to_lowercase();
    let combined = format!("{title_lower} {body_snippet}");

    // Check Microsoft keywords first (more specific)
    for kw in MICROSOFT_KEYWORDS {
        if combined.contains(kw) {
            return ArticleCategory::Microsoft;
        }
    }

    // Check for at least one strong, unambiguous cybersecurity indicator.
    let has_strong = THREAT_INTEL_STRONG.iter().any(|kw| combined.contains(kw));
    if has_strong {
        return ArticleCategory::ThreatIntel;
    }

    // Require multiple general security keywords to avoid false positives.
    let general_matches = THREAT_INTEL_GENERAL
        .iter()
        .filter(|kw| combined.contains(*kw))
        .count();
    if general_matches >= GENERAL_KEYWORD_THRESHOLD {
        return ArticleCategory::ThreatIntel;
    }

    ArticleCategory::General
}

/// Article fields derived from a single feed entry, ready for insertion.
#[derive(Debug, Clone, PartialEq)]
struct ParsedArticle {
    url: String,
    title: String,
    body_text: Option<String>,
    publish_date: String,
    category: ArticleCategory,
    image_urls_json: Option<String>,
}

/// Map a parsed feed entry to article fields. Returns `None` for entries
/// without a link, since the URL is the deduplication key.
fn entry_to_article(
    entry: &feed_rs::model::Entry,
    source_tag: &ArticleCategory,
) -> Option<ParsedArticle> {
    let url = entry
        .links
        .first()
        .map(|l| l.href.clone())
        .unwrap_or_default();

    if url.is_empty() {
        return None;
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

    let category = classify_article(source_tag, &title, body_text.as_deref());

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

    Some(ParsedArticle {
        url,
        title,
        body_text,
        publish_date,
        category,
        image_urls_json,
    })
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

/// Progress callback invoked as `(current, total, source_name)` per source.
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send>;

/// Refresh all active feeds and insert new articles into the database.
pub async fn refresh_all_feeds(
    db: Arc<Database>,
    progress_callback: Option<ProgressCallback>,
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

            let rows = stmt.query_map([], NewsSource::from_row)
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
                    let Some(article) = entry_to_article(&entry, &source_tag) else {
                        continue;
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
                            article.url,
                            article.title,
                            article.body_text,
                            article.publish_date,
                            article.category.as_str(),
                            article.image_urls_json,
                        ],
                    );

                    match insert_result {
                        Ok(0) => result.articles_deduplicated += 1,
                        Ok(_) => result.articles_added += 1,
                        Err(e) => {
                            log::warn!("Failed to insert article {}: {e}", article.url);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_fixture(bytes: &[u8], tag: ArticleCategory) -> Vec<ParsedArticle> {
        let feed = parser::parse(bytes).expect("fixture should parse");
        feed.entries
            .iter()
            .filter_map(|e| entry_to_article(e, &tag))
            .collect()
    }

    #[test]
    fn classify_respects_non_general_source_tag() {
        let category = classify_article(&ArticleCategory::Msrc, "Earnings call recap", None);
        assert_eq!(category, ArticleCategory::Msrc);
    }

    #[test]
    fn classify_microsoft_keyword_wins_over_threat_intel() {
        let category = classify_article(
            &ArticleCategory::General,
            "Ransomware abuses Azure storage",
            None,
        );
        assert_eq!(category, ArticleCategory::Microsoft);
    }

    #[test]
    fn classify_strong_threat_intel_keyword() {
        let category = classify_article(
            &ArticleCategory::General,
            "Researchers analyze CVE-2026-12345 exploitation",
            None,
        );
        assert_eq!(category, ArticleCategory::ThreatIntel);
    }

    #[test]
    fn classify_requires_two_general_keywords() {
        let one = classify_article(&ArticleCategory::General, "New vulnerability disclosed", None);
        assert_eq!(one, ArticleCategory::General);

        let two = classify_article(
            &ArticleCategory::General,
            "Vulnerability allows privilege escalation",
            None,
        );
        assert_eq!(two, ArticleCategory::ThreatIntel);
    }

    #[test]
    fn classify_only_considers_first_500_body_chars() {
        let body = format!("{} ransomware", "a".repeat(600));
        let category = classify_article(&ArticleCategory::General, "Weekly roundup", Some(&body));
        assert_eq!(category, ArticleCategory::General);
    }

    #[test]
    fn rss_entries_map_to_articles() {
        let articles = parse_fixture(
            include_bytes!("../../tests/fixtures/sample_rss.xml"),
            ArticleCategory::General,
        );

        // The entry without a link is dropped.
        assert_eq!(articles.len(), 2);

        let first = &articles[0];
        assert_eq!(first.url, "https://example.com/posts/ransomware-hospitals");
        assert_eq!(first.title, "New ransomware strain targets hospitals");
        assert_eq!(first.category, ArticleCategory::ThreatIntel);
        assert!(first.publish_date.starts_with("2026-10-06T14:30:00"));
        assert_eq!(
            first.image_urls_json.as_deref(),
            Some(r#"["https://example.com/img/ransomware.jpg"]"#)
        );

        let second = &articles[1];
        assert_eq!(second.category, ArticleCategory::General);
        assert_eq!(second.image_urls_json, None);
    }

    #[test]
    fn atom_entry_falls_back_to_content_and_updated_date() {
        let articles = parse_fixture(
            include_bytes!("../../tests/fixtures/sample_atom.xml"),
            ArticleCategory::General,
        );

        assert_eq!(articles.len(), 1);
        let article = &articles[0];
        assert_eq!(article.url, "https://example.org/exchange-advisory");
        assert_eq!(article.category, ArticleCategory::Microsoft);
        assert!(article.publish_date.starts_with("2026-10-08T12:00:00"));
        assert!(article
            .body_text
            .as_deref()
            .is_some_and(|b| b.contains("Admins should apply the update.")));
    }
}
