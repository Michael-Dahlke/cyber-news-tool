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

/// Map SQLite errors caused by malformed FTS5 MATCH syntax (unbalanced quotes,
/// dangling operators, unknown column filters) to INVALID_QUERY so the UI can
/// show a friendly message; anything else is reported with `context`.
fn map_fts_error(e: rusqlite::Error, context: &str) -> String {
    let msg = e.to_string();
    if msg.contains("fts5: syntax error")
        || msg.contains("unterminated string")
        || msg.contains("no such column")
    {
        "INVALID_QUERY".to_string()
    } else {
        format!("{context}: {e}")
    }
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
        .map_err(|e| map_fts_error(e, "Search error"))?;

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
                excerpt: {
                    let raw: String = row.get::<_, String>(5).unwrap_or_default();
                    crate::services::content_extractor::sanitize_search_excerpt(&raw)
                },
                relevance_rank: row.get::<_, f64>(6).unwrap_or(0.0),
            })
        })
        .map_err(|e| map_fts_error(e, "Search query error"))?
        .filter_map(|r| r.ok())
        .collect();

    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(SearchQueryResponse {
        results,
        total_count,
        query_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn setup() -> Connection {
        let conn = open_in_memory();
        conn.execute(
            "INSERT INTO news_sources (id, name, feed_url) VALUES (1, 'Test Source', 'https://feed.test/rss')",
            [],
        )
        .unwrap();
        let rows = [
            (1, "Ransomware gang hits retailer", "Attackers deployed ransomware across stores.", "2026-10-01T00:00:00Z"),
            (2, "Weekly security roundup", "Mentions of ransomware among other topics.", "2026-10-05T00:00:00Z"),
            (3, "Cloud pricing update", "Nothing about security here.", "2026-10-06T00:00:00Z"),
        ];
        for (id, title, body, date) in rows {
            conn.execute(
                "INSERT INTO articles (id, source_id, url, title, body_text, publish_date)
                 VALUES (?1, 1, ?2, ?3, ?4, ?5)",
                rusqlite::params![id, format!("https://a.test/{id}"), title, body, date],
            )
            .unwrap();
        }
        conn
    }

    fn params(keywords: &str) -> SearchQueryParams {
        SearchQueryParams {
            keywords: keywords.to_string(),
            date_from: None,
            date_to: None,
            limit: None,
            offset: None,
        }
    }

    #[test]
    fn empty_query_is_rejected() {
        let conn = setup();
        assert_eq!(search_articles(&conn, &params("   ")).unwrap_err(), "QUERY_TOO_SHORT");
    }

    #[test]
    fn title_matches_rank_above_body_matches() {
        let conn = setup();
        let response = search_articles(&conn, &params("ransomware")).unwrap();

        assert_eq!(response.total_count, 2);
        let ids: Vec<i64> = response.results.iter().map(|r| r.article_id).collect();
        assert_eq!(ids, vec![1, 2]);
        assert_eq!(response.results[0].source_name, "Test Source");
    }

    #[test]
    fn porter_stemming_matches_word_variants() {
        let conn = setup();
        let response = search_articles(&conn, &params("attacker")).unwrap();
        assert_eq!(response.total_count, 1);
    }

    #[test]
    fn excerpts_highlight_matches() {
        let conn = setup();
        let response = search_articles(&conn, &params("ransomware")).unwrap();
        assert!(response.results[0].excerpt.contains("<mark>ransomware</mark>"));
    }

    #[test]
    fn date_range_filters_results() {
        let conn = setup();
        let mut p = params("ransomware");
        p.date_from = Some("2026-10-03T00:00:00Z".to_string());
        let response = search_articles(&conn, &p).unwrap();

        assert_eq!(response.total_count, 1);
        assert_eq!(response.results[0].article_id, 2);

        let mut p = params("ransomware");
        p.date_to = Some("2026-10-03T00:00:00Z".to_string());
        let response = search_articles(&conn, &p).unwrap();
        assert_eq!(response.results[0].article_id, 1);
    }

    #[test]
    fn limit_and_offset_paginate() {
        let conn = setup();
        let mut p = params("ransomware");
        p.limit = Some(1);
        p.offset = Some(1);
        let response = search_articles(&conn, &p).unwrap();

        assert_eq!(response.total_count, 2);
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].article_id, 2);
    }

    #[test]
    fn malformed_fts_syntax_reports_invalid_query() {
        let conn = setup();
        for bad in ["\"unterminated", "ransomware AND", "nosuchcol:foo"] {
            assert_eq!(search_articles(&conn, &params(bad)).unwrap_err(), "INVALID_QUERY", "{bad}");
        }
    }
}
