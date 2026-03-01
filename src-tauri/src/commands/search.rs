use crate::db::Database;
use crate::services::search_service;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// Perform full-text keyword search across all articles.
#[tauri::command]
pub fn search_query(
    db: State<'_, Arc<Database>>,
    params: search_service::SearchQueryParams,
) -> Result<search_service::SearchQueryResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    search_service::search_articles(&conn, &params)
}

#[derive(Debug, Deserialize)]
pub struct SuggestParams {
    pub prefix: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SuggestResponse {
    pub suggestions: Vec<String>,
}

/// Search suggestions placeholder — returns empty array for now (future enhancement).
#[tauri::command]
pub fn search_suggest(
    _db: State<'_, Arc<Database>>,
    _params: SuggestParams,
) -> Result<SuggestResponse, String> {
    // Placeholder for future enhancement per contracts/search.md
    Ok(SuggestResponse {
        suggestions: Vec::new(),
    })
}
