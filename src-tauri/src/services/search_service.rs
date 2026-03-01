use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub article_id: i64,
    pub title: String,
    pub url: String,
    pub source_name: String,
    pub publish_date: String,
    pub excerpt: String,
    pub relevance_rank: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchQueryResponse {
    pub results: Vec<SearchResult>,
    pub total_count: i64,
    pub query_time_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub keywords: String,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Execute a full-text search across articles using FTS5 with BM25 ranking.
/// Title is weighted 10x over body_text per FR-021.
pub fn search_articles(
    conn: &Connection,
    params: &SearchQueryParams,
) -> Result<SearchQueryResponse, String> {
    let keywords = params.keywords.trim();

    if keywords.is_empty() {
        return Err("QUERY_TOO_SHORT".to_string());
    }

    let limit = params.limit.unwrap_or(30);
    let offset = params.offset.unwrap_or(0);

    let start = Instant::now();

    // Build date filter conditions
    let mut date_conditions = Vec::new();
    let mut date_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref date_from) = params.date_from {
        date_conditions.push(format!("a.publish_date >= ?{}", date_params.len() + 2));
        date_params.push(Box::new(date_from.clone()));
    }
    if let Some(ref date_to) = params.date_to {
        date_conditions.push(format!("a.publish_date <= ?{}", date_params.len() + 2));
        date_params.push(Box::new(date_to.clone()));
    }

    let date_filter = if date_conditions.is_empty() {
        String::new()
    } else {
        format!("AND {}", date_conditions.join(" AND "))
    };

    // Count total matching results
    let count_sql = format!(
        "SELECT COUNT(*)
         FROM articles_fts fts
         JOIN articles a ON a.id = fts.rowid
         WHERE articles_fts MATCH ?1
         {date_filter}"
    );

    let mut count_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    count_params.push(Box::new(keywords.to_string()));
    for p in &date_params {
        count_params.push(Box::new(p.as_ref().to_sql().map_err(|e| format!("Param error: {e}"))?));
    }

    let count_param_refs: Vec<&dyn rusqlite::types::ToSql> = count_params.iter().map(|p| p.as_ref()).collect();

    let total_count: i64 = conn
        .prepare(&count_sql)
        .and_then(|mut stmt| stmt.query_row(count_param_refs.as_slice(), |row| row.get(0)))
        .map_err(|e| {
            if e.to_string().contains("fts5: syntax error") {
                "INVALID_QUERY".to_string()
            } else {
                format!("Search error: {e}")
            }
        })?;

    // Query with BM25 ranking: title weighted 10.0, body_text weighted 1.0
    // snippet() extracts highlighted excerpts with <mark> delimiters (FR-020)
    let query_sql = format!(
        "SELECT a.id, a.title, a.url, ns.name as source_name, a.publish_date,
                snippet(articles_fts, 1, '<mark>', '</mark>', '...', 40) as excerpt,
                bm25(articles_fts, 10.0, 1.0) as rank
         FROM articles_fts fts
         JOIN articles a ON a.id = fts.rowid
         JOIN news_sources ns ON a.source_id = ns.id
         WHERE articles_fts MATCH ?1
         {date_filter}
         ORDER BY rank, a.publish_date DESC
         LIMIT ?{} OFFSET ?{}",
        date_params.len() + 2,
        date_params.len() + 3
    );

    let mut query_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    query_params.push(Box::new(keywords.to_string()));
    for p in &date_params {
        query_params.push(Box::new(p.as_ref().to_sql().map_err(|e| format!("Param error: {e}"))?));
    }
    query_params.push(Box::new(limit));
    query_params.push(Box::new(offset));

    let query_param_refs: Vec<&dyn rusqlite::types::ToSql> = query_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&query_sql)
        .map_err(|e| format!("Search prepare error: {e}"))?;

    let results = stmt
        .query_map(query_param_refs.as_slice(), |row| {
            Ok(SearchResult {
                article_id: row.get(0)?,
                title: row.get(1)?,
                url: row.get(2)?,
                source_name: row.get(3)?,
                publish_date: row.get(4)?,
                excerpt: row.get::<_, String>(5).unwrap_or_default(),
                relevance_rank: row.get::<_, f64>(6).unwrap_or(0.0),
            })
        })
        .map_err(|e| {
            if e.to_string().contains("fts5: syntax error") {
                "INVALID_QUERY".to_string()
            } else {
                format!("Search query error: {e}")
            }
        })?
        .filter_map(|r| r.ok())
        .collect();

    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(SearchQueryResponse {
        results,
        total_count,
        query_time_ms,
    })
}
