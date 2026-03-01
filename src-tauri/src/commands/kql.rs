use crate::db::Database;
use crate::models::KqlSuggestion;
use crate::services::kql_service;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

// ─── generate ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GenerateParams {
    pub article_id: i64,
    pub user_confirmed: bool,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub suggestions: Vec<KqlSuggestion>,
    pub llm_endpoint: String,
    pub llm_model: Option<String>,
}

/// Generate KQL queries from article content via the configured LLM service.
/// Requires `user_confirmed: true` per Constitution II — AI/LLM Governance.
#[tauri::command]
pub async fn kql_generate(
    db: State<'_, Arc<Database>>,
    params: GenerateParams,
) -> Result<GenerateResponse, String> {
    if !params.user_confirmed {
        return Err("USER_NOT_CONFIRMED".to_string());
    }

    let result = kql_service::generate_kql(db.inner().clone(), params.article_id).await?;

    Ok(GenerateResponse {
        suggestions: result.suggestions,
        llm_endpoint: result.llm_endpoint,
        llm_model: result.llm_model,
    })
}

// ─── get_for_article ──────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetForArticleParams {
    pub article_id: i64,
}

#[derive(Debug, Serialize)]
pub struct GetForArticleResponse {
    pub suggestions: Vec<KqlSuggestion>,
    pub generated_at: Option<String>,
}

/// Retrieve previously generated KQL suggestions for an article.
#[tauri::command]
pub fn kql_get_for_article(
    db: State<'_, Arc<Database>>,
    params: GetForArticleParams,
) -> Result<GetForArticleResponse, String> {
    let (suggestions, generated_at) =
        kql_service::get_suggestions_for_article(db.inner(), params.article_id)?;

    Ok(GetForArticleResponse {
        suggestions,
        generated_at,
    })
}

// ─── copy_to_clipboard ────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CopyToClipboardParams {
    pub query_text: String,
}

#[derive(Debug, Serialize)]
pub struct CopyToClipboardResponse {
    pub success: bool,
}

/// Copy a KQL query to the system clipboard.
#[tauri::command]
pub fn kql_copy_to_clipboard(
    params: CopyToClipboardParams,
) -> Result<CopyToClipboardResponse, String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("CLIPBOARD_ERROR: {e}"))?;

    clipboard
        .set_text(&params.query_text)
        .map_err(|e| format!("CLIPBOARD_ERROR: {e}"))?;

    Ok(CopyToClipboardResponse { success: true })
}

// ─── check_config ─────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CheckConfigResponse {
    pub is_configured: bool,
    pub endpoint: String,
    pub model: Option<String>,
    pub is_reachable: bool,
    pub error: Option<String>,
}

/// Check whether the LLM service is configured and reachable.
#[tauri::command]
pub async fn kql_check_config(
    db: State<'_, Arc<Database>>,
) -> Result<CheckConfigResponse, String> {
    let result = kql_service::check_config_with_db(db.inner().clone()).await?;

    Ok(CheckConfigResponse {
        is_configured: result.is_configured,
        endpoint: result.endpoint,
        model: result.model,
        is_reachable: result.is_reachable,
        error: result.error,
    })
}
