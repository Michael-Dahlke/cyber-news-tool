import { invoke } from "@tauri-apps/api/core";
import type { ArticleCategory } from "@/types";

// ─── Article Summary (matches Rust ArticleSummary) ────────

export interface ArticleSummary {
  id: number;
  title: string;
  url: string;
  source_name: string;
  publish_date: string;
  category: string;
  is_cached: boolean;
}

export interface ListByCategoryResponse {
  articles: ArticleSummary[];
  total_count: number;
}

// ─── Articles Commands ────────────────────────────────────

export async function listByCategory(params: {
  category: ArticleCategory;
  days_back: number;
  limit?: number;
  offset?: number;
}): Promise<ListByCategoryResponse> {
  return invoke<ListByCategoryResponse>("articles_list_by_category", {
    params,
  });
}

// ─── Article Detail Types ─────────────────────────────────

export interface ArticleDetail {
  id: number;
  title: string;
  url: string;
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
  cache_size_bytes: number;
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
