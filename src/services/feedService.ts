import { invoke } from "@tauri-apps/api/core";
import type { NewsSource, ArticleCategory } from "@/types";

// ─── Feed Error ───────────────────────────────────────────

export interface FeedError {
  source_id: number;
  source_name: string;
  error: string;
}

// ─── Refresh Response ─────────────────────────────────────

export interface RefreshResponse {
  sources_fetched: number;
  articles_added: number;
  articles_deduplicated: number;
  errors: FeedError[];
}

// ─── Feeds Commands ───────────────────────────────────────

export async function listSources(): Promise<NewsSource[]> {
  return invoke<NewsSource[]>("feeds_list_sources");
}

export async function addSource(params: {
  feed_url: string;
  name?: string;
  default_tag?: ArticleCategory;
}): Promise<NewsSource> {
  return invoke<NewsSource>("feeds_add_source", { params });
}

export async function updateSource(params: {
  id: number;
  name?: string;
  default_tag?: ArticleCategory;
  is_active?: boolean;
}): Promise<NewsSource> {
  return invoke<NewsSource>("feeds_update_source", { params });
}

export async function removeSource(id: number): Promise<boolean> {
  return invoke<boolean>("feeds_remove_source", { id });
}

export async function refreshFeeds(): Promise<RefreshResponse> {
  return invoke<RefreshResponse>("feeds_refresh");
}
