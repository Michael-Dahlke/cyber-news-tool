use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: i64,
    pub source_id: i64,
    pub url: String,
    pub title: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub publish_date: String,
    pub category: ArticleCategory,
    pub image_urls: Vec<String>,
    pub is_cached: bool,
    pub cached_at: Option<String>,
    pub extraction_method: Option<ExtractionMethod>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArticleCategory {
    General,
    Microsoft,
    ThreatIntel,
}

impl ArticleCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArticleCategory::General => "general",
            ArticleCategory::Microsoft => "microsoft",
            ArticleCategory::ThreatIntel => "threat_intel",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "microsoft" => ArticleCategory::Microsoft,
            "threat_intel" => ArticleCategory::ThreatIntel,
            _ => ArticleCategory::General,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    Readability,
    Scraper,
    Webview,
    Failed,
}

impl ExtractionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtractionMethod::Readability => "readability",
            ExtractionMethod::Scraper => "scraper",
            ExtractionMethod::Webview => "webview",
            ExtractionMethod::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "readability" => Some(ExtractionMethod::Readability),
            "scraper" => Some(ExtractionMethod::Scraper),
            "webview" => Some(ExtractionMethod::Webview),
            "failed" => Some(ExtractionMethod::Failed),
            _ => None,
        }
    }
}

/// Helper to parse an Article from a rusqlite Row.
impl Article {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let image_urls_json: Option<String> = row.get("image_urls")?;
        let image_urls: Vec<String> = image_urls_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let category_str: String = row.get("category")?;
        let extraction_str: Option<String> = row.get("extraction_method")?;
        let is_cached_int: i32 = row.get("is_cached")?;

        Ok(Article {
            id: row.get("id")?,
            source_id: row.get("source_id")?,
            url: row.get("url")?,
            title: row.get("title")?,
            body_html: row.get("body_html")?,
            body_text: row.get("body_text")?,
            publish_date: row.get("publish_date")?,
            category: ArticleCategory::from_str(&category_str),
            image_urls,
            is_cached: is_cached_int != 0,
            cached_at: row.get("cached_at")?,
            extraction_method: extraction_str.and_then(|s| ExtractionMethod::from_str(&s)),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
