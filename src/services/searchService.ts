import { invoke } from "@tauri-apps/api/core";

// ─── Search Types ─────────────────────────────────────────

export interface SearchResult {
  article_id: number;
  title: string;
  url: string;
  source_name: string;
  publish_date: string;
  excerpt: string;
  relevance_rank: number;
}

export interface SearchQueryResponse {
  results: SearchResult[];
  total_count: number;
  query_time_ms: number;
}

export interface SuggestResponse {
  suggestions: string[];
}

// ─── Search Commands ──────────────────────────────────────

export async function searchQuery(params: {
  keywords: string;
  date_from?: string;
  date_to?: string;
  limit?: number;
  offset?: number;
}): Promise<SearchQueryResponse> {
  return invoke<SearchQueryResponse>("search_query", { params });
}

export async function searchSuggest(params: {
  prefix: string;
  limit?: number;
}): Promise<SuggestResponse> {
  return invoke<SuggestResponse>("search_suggest", { params });
}
