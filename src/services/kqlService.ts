import { invoke } from "@tauri-apps/api/core";
import type { KqlSuggestion } from "@/types";

// ─── Types ────────────────────────────────────────────────

export interface GenerateResponse {
  suggestions: KqlSuggestion[];
  llm_endpoint: string;
  llm_model: string | null;
}

export interface GetForArticleResponse {
  suggestions: KqlSuggestion[];
  generated_at: string | null;
}

export interface CopyToClipboardResponse {
  success: boolean;
}

export interface CheckConfigResponse {
  is_configured: boolean;
  endpoint: string;
  model: string | null;
  is_reachable: boolean;
  error: string | null;
}

// ─── Commands ─────────────────────────────────────────────

export async function generate(
  articleId: number,
  userConfirmed: boolean
): Promise<GenerateResponse> {
  return invoke<GenerateResponse>("kql_generate", {
    params: { article_id: articleId, user_confirmed: userConfirmed },
  });
}

export async function getForArticle(
  articleId: number
): Promise<GetForArticleResponse> {
  return invoke<GetForArticleResponse>("kql_get_for_article", {
    params: { article_id: articleId },
  });
}

export async function copyToClipboard(
  queryText: string
): Promise<CopyToClipboardResponse> {
  return invoke<CopyToClipboardResponse>("kql_copy_to_clipboard", {
    params: { query_text: queryText },
  });
}

export async function checkConfig(): Promise<CheckConfigResponse> {
  return invoke<CheckConfigResponse>("kql_check_config");
}
