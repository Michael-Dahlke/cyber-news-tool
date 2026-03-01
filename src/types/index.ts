// ─── Settings ─────────────────────────────────────────────

export type Theme = "system" | "light" | "dark";

export interface AppSettings {
  llm_endpoint: string;
  llm_model: string;
  has_api_key: boolean;
  feed_refresh_interval_min: number;
  cache_retention_days: number;
  theme: Theme;
}

export interface SettingsUpdate {
  llm_endpoint?: string;
  llm_model?: string;
  feed_refresh_interval_min?: number;
  cache_retention_days?: number;
  theme?: Theme;
}

export interface AppInfo {
  version: string;
  db_path: string;
  os: string;
}

// ─── News Sources ─────────────────────────────────────────

export type FeedType = "rss" | "atom" | "json";

export type ArticleCategory = "general" | "microsoft" | "threat_intel";

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
  extraction_method: ExtractionMethod | null;
  created_at: string;
  updated_at: string;
}

export interface ArticleListItem {
  id: number;
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
