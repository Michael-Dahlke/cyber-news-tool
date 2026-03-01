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
