import { invoke } from "@tauri-apps/api/core";
import type { ArticleCategory } from "@/types";

// ─── Article Summary (matches Rust ArticleSummary) ────────

export interface ArticleSummary {
  id: number;
  title: string;
  url: string;
  source_id: number;
  source_name: string;
  publish_date: string;
  category: string;
  is_cached: boolean;
}

export interface ListByCategoryResponse {
  articles: ArticleSummary[];
  total_count: number;
}

// ─── Browse by Source (FR-033) ────────────────────────────

export interface SourceInfo {
  id: number;
  name: string;
  url: string;
  category: string;
}

export interface ListBySourceResponse {
  source: SourceInfo;
  articles: ArticleSummary[];
  total: number;
}

// ─── Articles Commands ────────────────────────────────────

export async function listByCategory(params: {
  category: ArticleCategory;
  days_back: number;
  limit?: number;
  offset?: number;
  /** Exclude today's articles (so they appear only in the Today section) */
  exclude_today?: boolean;
}): Promise<ListByCategoryResponse> {
  return invoke<ListByCategoryResponse>("articles_list_by_category", {
    params,
  });
}

/** List ALL articles from today regardless of category. */
export async function listToday(params?: {
  limit?: number;
  offset?: number;
}): Promise<ListByCategoryResponse> {
  return invoke<ListByCategoryResponse>("articles_list_today", {
    params: params ?? {},
  });
}

/** List articles for a single source (FR-033). */
export async function listBySource(params: {
  source_id: number;
  limit?: number;
  offset?: number;
}): Promise<ListBySourceResponse> {
  return invoke<ListBySourceResponse>("articles_list_by_source", { params });
}

// ─── Article Detail Types ─────────────────────────────────

export interface ArticleDetail {
  id: number;
  title: string;
  url: string;
  source_id: number;
  source_name: string;
  publish_date: string;
  body_html: string | null;
  image_urls: string[];
  extraction_method: string | null;
  is_cached: boolean;
  category: string;
}

export interface GetContentResponse {
  article: ArticleDetail;
}

export interface ExtractViaWebviewResponse {
  article: ArticleDetail;
}

export interface CacheStatusResponse {
  cached_articles: number;
  total_articles: number;
  oldest_cached: string | null;
  body_cache_size_bytes: number;
  cache_size_limit_gb: number;
  size_cap_pct: number;
}

export interface EvictCacheResponse {
  evicted_count: number;
}

// ─── Content Commands ─────────────────────────────────────

export async function getContent(articleId: number): Promise<GetContentResponse> {
  return invoke<GetContentResponse>("articles_get_content", {
    params: { article_id: articleId },
  });
}

export async function extractViaWebview(
  articleId: number,
  extractedHtml: string
): Promise<ExtractViaWebviewResponse> {
  return invoke<ExtractViaWebviewResponse>("articles_extract_via_webview", {
    params: { article_id: articleId, extracted_html: extractedHtml },
  });
}

// ─── Cache Commands ───────────────────────────────────────

export async function getCacheStatus(): Promise<CacheStatusResponse> {
  return invoke<CacheStatusResponse>("articles_get_cache_status");
}

export async function evictCache(): Promise<EvictCacheResponse> {
  return invoke<EvictCacheResponse>("articles_evict_cache");
}
