use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_endpoint: String,
    pub llm_model: String,
    pub background_refresh_threshold_hours: u32,
    pub cache_retention_days: u32,
    pub article_cache_size_limit_gb: u32,
    pub reader_font_size: u32,
    pub theme: Theme,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[allow(dead_code)]
impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm_endpoint: String::new(),
            llm_model: String::new(),
            background_refresh_threshold_hours: 4,
            cache_retention_days: 30,
            article_cache_size_limit_gb: 20,
            reader_font_size: 16,
            theme: Theme::System,
            has_api_key: false,
        }
    }
}

/// Partial update payload for settings — only non-None fields are applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUpdate {
    pub llm_endpoint: Option<String>,
    pub llm_model: Option<String>,
    pub background_refresh_threshold_hours: Option<u32>,
    pub cache_retention_days: Option<u32>,
    pub article_cache_size_limit_gb: Option<u32>,
    pub reader_font_size: Option<u32>,
    pub theme: Option<String>,
}

/// App information returned by get_app_info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub db_path: String,
    pub os: String,
}
