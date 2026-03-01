use crate::db::Database;
use crate::models::KqlSuggestion;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ─── Types ────────────────────────────────────────────────

/// Result of a KQL generation call.
#[derive(Debug, Clone, Serialize)]
pub struct KqlGenerationResult {
    pub suggestions: Vec<KqlSuggestion>,
    pub llm_endpoint: String,
    pub llm_model: Option<String>,
}

/// A single suggestion parsed from the LLM response JSON.
#[derive(Debug, Deserialize)]
struct ParsedSuggestion {
    query: String,
    description: String,
    table: Option<String>,
}

/// OpenAI-compatible Chat Completions response.
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChoice {
    message: ChatCompletionMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionMessage {
    content: Option<String>,
}

// ─── Constants ────────────────────────────────────────────

const LLM_TIMEOUT_SECS: u64 = 60;
const MAX_ARTICLE_CHARS: usize = 16_000; // ~4000 tokens

const SYSTEM_PROMPT: &str = r#"You are a cybersecurity KQL (Kusto Query Language) expert specializing in Microsoft Sentinel and Defender XDR detection queries.

Given an article about a cybersecurity threat, vulnerability, or attack technique, generate KQL detection queries that a SOC analyst could use to hunt for related indicators or behaviors in their environment.

Target tables to consider:
- SecurityEvent (Windows Security Events)
- SigninLogs (Azure AD sign-in activity)
- AuditLogs (Azure AD audit logs)
- DeviceProcessEvents (Defender for Endpoint process events)
- DeviceNetworkEvents (Defender for Endpoint network events)
- DeviceFileEvents (Defender for Endpoint file events)
- EmailEvents (Defender for Office 365 email events)
- CommonSecurityLog (CEF/syslog data)
- Syslog (Linux syslog)
- ThreatIntelligenceIndicator (Threat intelligence feeds)
- OfficeActivity (Office 365 activity)
- AzureActivity (Azure resource management)

Rules:
1. Each query must be valid KQL syntax
2. Include comments in queries explaining the detection logic
3. Focus on actionable detection, not just data retrieval
4. Use time-scoped queries (e.g., "| where TimeGenerated > ago(7d)")
5. If specific IOCs (IPs, domains, hashes) are mentioned in the article, include them
6. Generate 1-5 queries depending on the article content

Respond ONLY with a JSON array of objects. Each object must have:
- "query": the full KQL query string
- "description": a plain-language description of what the query detects
- "table": the primary Sentinel table used (e.g., "SecurityEvent"), or null if multiple

Example response:
[
  {
    "query": "SecurityEvent\n| where TimeGenerated > ago(7d)\n| where EventID == 4688\n| where CommandLine contains \"mimikatz\"",
    "description": "Detects processes with mimikatz in the command line, indicating potential credential theft",
    "table": "SecurityEvent"
  }
]

If the article does not contain actionable threat intelligence or indicators for detection queries, respond with an empty JSON array: []"#;

// ─── Core Functions ───────────────────────────────────────

/// Generate KQL suggestions for an article using the configured LLM service.
pub async fn generate_kql(
    db: Arc<Database>,
    article_id: i64,
) -> Result<KqlGenerationResult, String> {
    // 1. Read LLM config from settings + keychain
    let (endpoint, model) = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
        let endpoint: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'llm_endpoint'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("DB error: {e}"))?;
        let model: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'llm_model'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("DB error: {e}"))?;
        (endpoint, model)
    };

    if endpoint.is_empty() {
        return Err("NOT_CONFIGURED".to_string());
    }

    let api_key = crate::services::keychain::get_api_key()
        .map_err(|e| format!("KEYCHAIN_ERROR: {e}"))?
        .ok_or_else(|| "NOT_CONFIGURED".to_string())?;

    // 2. Read article body_text from DB
    let body_text: String = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
        conn.query_row(
            "SELECT body_text FROM articles WHERE id = ?1",
            [article_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .map_err(|e| {
            if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                "ARTICLE_NOT_FOUND".to_string()
            } else {
                format!("DB error: {e}")
            }
        })?
        .ok_or_else(|| "ARTICLE_NO_CONTENT".to_string())?
    };

    if body_text.trim().is_empty() {
        return Err("ARTICLE_NO_CONTENT".to_string());
    }

    // 3. Truncate article text if too long (~4000 tokens ≈ 16000 chars)
    let truncated_text = if body_text.len() > MAX_ARTICLE_CHARS {
        format!(
            "{}\n\n[Article truncated for LLM processing]",
            &body_text[..MAX_ARTICLE_CHARS]
        )
    } else {
        body_text
    };

    // 4. Build Chat Completions request
    let model_name = if model.is_empty() {
        None
    } else {
        Some(model.clone())
    };

    let request_body = build_chat_completions_body(&truncated_text, model_name.as_deref());

    // 5. Send POST to LLM endpoint
    let chat_url = build_chat_url(&endpoint);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(LLM_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let response = client
        .post(&chat_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "LLM_TIMEOUT".to_string()
            } else {
                format!("LLM_ERROR: {e}")
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!(
            "LLM_ERROR: HTTP {status} — {body}",
        ));
    }

    // 6. Parse the LLM response
    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("LLM_ERROR: Failed to parse response — {e}"))?;

    let content = completion
        .choices
        .first()
        .and_then(|c| c.message.content.as_deref())
        .ok_or_else(|| "LLM_ERROR: Empty response from LLM".to_string())?;

    let parsed = parse_suggestions(content)?;

    if parsed.is_empty() {
        return Err("LLM_NO_SUGGESTIONS".to_string());
    }

    // 7. Store suggestions in DB
    let suggestions = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

        // Delete any previous suggestions for this article (re-generation)
        conn.execute(
            "DELETE FROM kql_suggestions WHERE article_id = ?1",
            [article_id],
        )
        .map_err(|e| format!("DB error: {e}"))?;

        let mut stored: Vec<KqlSuggestion> = Vec::new();
        for p in &parsed {
            conn.execute(
                "INSERT INTO kql_suggestions (article_id, query_text, description, target_table, llm_endpoint, llm_model)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    article_id,
                    &p.query,
                    &p.description,
                    &p.table,
                    &endpoint,
                    &model_name,
                ],
            )
            .map_err(|e| format!("DB error: {e}"))?;

            let id = conn.last_insert_rowid();
            let suggestion = conn
                .query_row(
                    "SELECT id, article_id, query_text, description, target_table, llm_endpoint, llm_model, created_at
                     FROM kql_suggestions WHERE id = ?1",
                    [id],
                    KqlSuggestion::from_row,
                )
                .map_err(|e| format!("DB error: {e}"))?;
            stored.push(suggestion);
        }
        stored
    };

    Ok(KqlGenerationResult {
        suggestions,
        llm_endpoint: endpoint,
        llm_model: model_name,
    })
}

/// Retrieve previously generated KQL suggestions for an article.
pub fn get_suggestions_for_article(
    db: &Database,
    article_id: i64,
) -> Result<(Vec<KqlSuggestion>, Option<String>), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;

    // Verify article exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM articles WHERE id = ?1",
            [article_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("DB error: {e}"))?;

    if !exists {
        return Err("ARTICLE_NOT_FOUND".to_string());
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, article_id, query_text, description, target_table, llm_endpoint, llm_model, created_at
             FROM kql_suggestions WHERE article_id = ?1 ORDER BY id ASC",
        )
        .map_err(|e| format!("DB error: {e}"))?;

    let suggestions: Vec<KqlSuggestion> = stmt
        .query_map([article_id], KqlSuggestion::from_row)
        .map_err(|e| format!("DB error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    let generated_at = suggestions.first().map(|s| s.created_at.clone());

    Ok((suggestions, generated_at))
}

/// Check LLM config with database access.
pub async fn check_config_with_db(db: Arc<Database>) -> Result<CheckConfigResult, String> {
    let (endpoint, model) = {
        let conn = db.conn.lock().map_err(|e| format!("DB lock error: {e}"))?;
        let endpoint: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'llm_endpoint'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();
        let model: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'llm_model'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_default();
        (endpoint, model)
    };

    let has_api_key = crate::services::keychain::has_api_key();
    let is_configured = !endpoint.is_empty() && has_api_key;

    if !is_configured {
        return Ok(CheckConfigResult {
            is_configured: false,
            endpoint: if endpoint.is_empty() {
                "(not set)".to_string()
            } else {
                mask_endpoint(&endpoint)
            },
            model: if model.is_empty() { None } else { Some(model) },
            is_reachable: false,
            error: Some(if endpoint.is_empty() {
                "LLM endpoint not configured".to_string()
            } else {
                "API key not set".to_string()
            }),
        });
    }

    let api_key = crate::services::keychain::get_api_key()
        .map_err(|e| format!("KEYCHAIN_ERROR: {e}"))?
        .unwrap_or_default();

    // Send a minimal test request to validate connectivity
    let chat_url = build_chat_url(&endpoint);
    let test_body = serde_json::json!({
        "messages": [
            {"role": "user", "content": "test"}
        ],
        "max_tokens": 1
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    match client
        .post(&chat_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&test_body)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 400 => {
            // 400 is acceptable — means the endpoint is reachable but rejected our minimal request
            Ok(CheckConfigResult {
                is_configured: true,
                endpoint: mask_endpoint(&endpoint),
                model: if model.is_empty() { None } else { Some(model) },
                is_reachable: true,
                error: None,
            })
        }
        Ok(resp) => {
            let status = resp.status();
            Ok(CheckConfigResult {
                is_configured: true,
                endpoint: mask_endpoint(&endpoint),
                model: if model.is_empty() { None } else { Some(model) },
                is_reachable: false,
                error: Some(format!("HTTP {status}")),
            })
        }
        Err(e) => Ok(CheckConfigResult {
            is_configured: true,
            endpoint: mask_endpoint(&endpoint),
            model: if model.is_empty() { None } else { Some(model) },
            is_reachable: false,
            error: Some(format!("{e}")),
        }),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckConfigResult {
    pub is_configured: bool,
    pub endpoint: String,
    pub model: Option<String>,
    pub is_reachable: bool,
    pub error: Option<String>,
}

// ─── Helpers ──────────────────────────────────────────────

/// Build the Chat Completions JSON body.
fn build_chat_completions_body(
    article_text: &str,
    model: Option<&str>,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "messages": [
            {
                "role": "system",
                "content": SYSTEM_PROMPT
            },
            {
                "role": "user",
                "content": article_text
            }
        ],
        "temperature": 0.3
    });

    if let Some(m) = model {
        body["model"] = serde_json::Value::String(m.to_string());
    }

    body
}

/// Build the chat completions URL, appending `/chat/completions` if needed.
fn build_chat_url(endpoint: &str) -> String {
    let trimmed = endpoint.trim_end_matches('/');
    if trimmed.ends_with("/chat/completions") {
        trimmed.to_string()
    } else if trimmed.ends_with("/v1") {
        format!("{trimmed}/chat/completions")
    } else {
        // Try appending /v1/chat/completions if it looks like a base URL
        // But if it already has a path beyond the host, just append /chat/completions
        if trimmed.contains("/v1/") || trimmed.ends_with("/openai") {
            format!("{trimmed}/chat/completions")
        } else {
            format!("{trimmed}/v1/chat/completions")
        }
    }
}

/// Parse the LLM response content into structured suggestions.
fn parse_suggestions(content: &str) -> Result<Vec<ParsedSuggestion>, String> {
    // The LLM might wrap JSON in markdown code fences — strip them
    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let parsed: Vec<ParsedSuggestion> = serde_json::from_str(cleaned)
        .map_err(|e| format!("LLM_ERROR: Failed to parse suggestions JSON — {e}"))?;

    // Validate each suggestion has non-empty required fields
    let valid: Vec<ParsedSuggestion> = parsed
        .into_iter()
        .filter(|s| !s.query.trim().is_empty() && !s.description.trim().is_empty())
        .collect();

    Ok(valid)
}

/// Mask an endpoint URL for display (show scheme + host only).
fn mask_endpoint(endpoint: &str) -> String {
    // Simple string-based masking without url crate
    if let Some(after_scheme) = endpoint.strip_prefix("https://").or_else(|| endpoint.strip_prefix("http://")) {
        let scheme = if endpoint.starts_with("https") { "https" } else { "http" };
        let host = after_scheme.split('/').next().unwrap_or(after_scheme);
        return format!("{scheme}://{host}/...");
    }
    // Fallback: show first 30 chars
    if endpoint.len() > 30 {
        format!("{}...", &endpoint[..30])
    } else {
        endpoint.to_string()
    }
}
