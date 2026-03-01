use crate::db::Database;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct ArticleSummary {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub source_name: String,
    pub publish_date: String,
    pub category: String,
    pub is_cached: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListByCategoryParams {
    pub category: String,
    pub days_back: i64,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListByCategoryResponse {
    pub articles: Vec<ArticleSummary>,
    pub total_count: i64,
}

/// List articles by category with date filtering.
#[tauri::command]
pub fn articles_list_by_category(
    db: State<'_, Arc<Database>>,
    params: ListByCategoryParams,
) -> Result<ListByCategoryResponse, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    // Build date filter: days_back=0 means today only, days_back=7 means past week
    let date_filter = if params.days_back == 0 {
        "date(a.publish_date) = date('now')".to_string()
    } else {
        format!(
            "a.publish_date >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-{} days')",
            params.days_back
        )
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM articles a WHERE a.category = ?1 AND {date_filter}"
    );

    let total_count: i64 = conn
        .prepare(&count_sql)
        .map_err(|e| format!("DB error: {e}"))?
        .query_row([&params.category], |row| row.get(0))
        .map_err(|e| format!("DB error: {e}"))?;

    let query_sql = format!(
        "SELECT a.id, a.title, a.url, ns.name as source_name, a.publish_date, a.category, a.is_cached
         FROM articles a
         JOIN news_sources ns ON a.source_id = ns.id
         WHERE a.category = ?1 AND {date_filter}
         ORDER BY a.publish_date DESC
         LIMIT ?2 OFFSET ?3"
    );

    let mut stmt = conn
        .prepare(&query_sql)
        .map_err(|e| format!("DB error: {e}"))?;

    let articles = stmt
        .query_map(rusqlite::params![params.category, limit, offset], |row| {
            Ok(ArticleSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                source_name: row.get(3)?,
                publish_date: row.get(4)?,
                category: row.get(5)?,
                is_cached: row.get::<_, i32>(6).map(|v| v != 0)?,
            })
        })
        .map_err(|e| format!("DB error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ListByCategoryResponse {
        articles,
        total_count,
    })
}
