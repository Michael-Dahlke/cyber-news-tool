// ─── Settings ─────────────────────────────────────────────

export type Theme = "system" | "light" | "dark";

export interface AppSettings {
  llm_endpoint: string;
  llm_model: string;
  has_api_key: boolean;
  background_refresh_threshold_hours: number;
  cache_retention_days: number;
  article_cache_size_limit_gb: number;
  reader_font_size: number;
  theme: Theme;
}

export interface SettingsUpdate {
  llm_endpoint?: string;
  llm_model?: string;
  background_refresh_threshold_hours?: number;
  cache_retention_days?: number;
  article_cache_size_limit_gb?: number;
  reader_font_size?: number;
  theme?: Theme;
}

export interface AppInfo {
  version: string;
  db_path: string;
  os: string;
}

// ─── News Sources ─────────────────────────────────────────

export type FeedType = "rss" | "atom" | "json";

export type ArticleCategory = "general" | "microsoft" | "threat_intel" | "msrc";

export interface NewsSource {
  id: number;
  name: string;
  feed_url: string;
  feed_type: FeedType;
  default_tag: ArticleCategory;
  is_default: boolean;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

// ─── Articles ─────────────────────────────────────────────

export type ExtractionMethod = "readability" | "scraper" | "webview" | "failed";

export interface Article {
  id: number;
  source_id: number;
  url: string;
  title: string;
  body_html: string | null;
  body_text: string | null;
  publish_date: string;
  category: ArticleCategory;
  image_urls: string[];
  is_cached: boolean;
  cached_at: string | null;
  body_size_bytes: number;
  extraction_method: ExtractionMethod | null;
  created_at: string;
  updated_at: string;
}

export interface ArticleListItem {
  id: number;
  source_id: number;
  url: string;
  title: string;
  publish_date: string;
  category: ArticleCategory;
  source_name: string;
  is_cached: boolean;
}

// ─── Search ───────────────────────────────────────────────

export interface SearchQuery {
  keywords: string;
  date_from?: string;
  date_to?: string;
  limit?: number;
  offset?: number;
}

export interface SearchResult {
  id: number;
  title: string;
  url: string;
  source_name: string;
  publish_date: string;
  excerpt: string;
  relevance_rank: number;
}

// ─── KQL ──────────────────────────────────────────────────

export interface KqlSuggestion {
  id: number;
  article_id: number;
  query_text: string;
  description: string;
  target_table: string | null;
  llm_endpoint: string;
  llm_model: string | null;
  created_at: string;
}

// ─── Cache Status ─────────────────────────────────────────

export interface CacheStatusResponse {
  cached_articles: number;
  total_articles: number;
  oldest_cached: string | null;
  body_cache_size_bytes: number;
  cache_size_limit_gb: number;
  size_cap_pct: number;
}

// ─── Source Browsing ──────────────────────────────────────

export interface ListBySourceResponse {
  source: NewsSource;
  articles: ArticleListItem[];
  total: number;
}
