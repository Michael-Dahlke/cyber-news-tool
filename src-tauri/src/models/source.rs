use serde::{Deserialize, Serialize};
use super::article::ArticleCategory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSource {
    pub id: i64,
    pub name: String,
    pub feed_url: String,
    pub feed_type: FeedType,
    pub default_tag: ArticleCategory,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedType {
    Rss,
    Atom,
    Json,
}

#[allow(dead_code)]
impl FeedType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeedType::Rss => "rss",
            FeedType::Atom => "atom",
            FeedType::Json => "json",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "atom" => FeedType::Atom,
            "json" => FeedType::Json,
            _ => FeedType::Rss,
        }
    }
}

impl NewsSource {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let feed_type_str: String = row.get("feed_type")?;
        let default_tag_str: String = row.get("default_tag")?;
        let is_default_int: i32 = row.get("is_default")?;
        let is_active_int: i32 = row.get("is_active")?;

        Ok(NewsSource {
            id: row.get("id")?,
            name: row.get("name")?,
            feed_url: row.get("feed_url")?,
            feed_type: FeedType::from_str(&feed_type_str),
            default_tag: ArticleCategory::from_str(&default_tag_str),
            is_default: is_default_int != 0,
            is_active: is_active_int != 0,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
