# Data Model: Cybersecurity News Reader & Dashboard

**Feature Branch**: `001-news-reader-dashboard`
**Date**: 2026-02-28
**Source**: [spec.md](spec.md) Key Entities + Functional Requirements

---

## Entity Relationship Diagram

```text
┌───────────────┐        ┌───────────────────┐
│  news_sources │ 1────* │     articles       │
│───────────────│        │───────────────────│
│ id (PK)       │        │ id (PK)            │
│ name          │        │ source_id (FK)     │
│ feed_url      │        │ url (UNIQUE)       │
│ feed_type     │        │ title              │
│ default_tag   │        │ body_html          │
│ is_default    │        │ body_text          │
│ is_active     │        │ publish_date       │
│ created_at    │        │ category           │
│ updated_at    │        │ image_urls (JSON)  │
└───────────────┘        │ is_cached          │
                         │ cached_at          │
                         │ extraction_method  │
                         │ created_at         │
                         │ updated_at         │
                         └────────┬───────────┘
                                  │
                                  │ 1
                                  │
                                  ▼ *
                         ┌───────────────────┐
                         │ kql_suggestions    │
                         │───────────────────│
                         │ id (PK)            │
                         │ article_id (FK)    │
                         │ query_text         │
                         │ description        │
                         │ target_table       │
                         │ llm_endpoint       │
                         │ llm_model          │
                         │ created_at         │
                         └───────────────────┘

┌──────────────────────────────┐
│ articles_fts (FTS5 virtual)  │
│──────────────────────────────│
│ title                        │
│ body_text                    │
│ (content = articles)         │
└──────────────────────────────┘

┌───────────────────┐
│    app_settings    │
│───────────────────│
│ key (PK)           │
│ value              │
│ updated_at         │
└───────────────────┘
```

---

## Tables

### `news_sources`

Represents an aggregated feed origin (FR-033–FR-036).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique source ID |
| `name` | TEXT | NOT NULL | Human-readable source name (e.g., "BleepingComputer") |
| `feed_url` | TEXT | NOT NULL, UNIQUE | RSS/Atom feed URL |
| `feed_type` | TEXT | NOT NULL, DEFAULT 'rss' | Feed format: `rss`, `atom`, `json` |
| `default_tag` | TEXT | NOT NULL, DEFAULT 'general' | Category tag: `general`, `microsoft`, `threat_intel` |
| `is_default` | INTEGER | NOT NULL, DEFAULT 0 | 1 = shipped with app, 0 = user-added |
| `is_active` | INTEGER | NOT NULL, DEFAULT 1 | 1 = enabled, 0 = disabled by user (FR-034) |
| `created_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | ISO 8601 |
| `updated_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | ISO 8601, updated on change |

**Indexes**:
- `idx_news_sources_active` ON (`is_active`) WHERE `is_active = 1`
- `idx_news_sources_tag` ON (`default_tag`)

**Validation Rules**:
- `feed_url` must be a valid HTTP/HTTPS URL
- `feed_type` must be one of: `rss`, `atom`, `json`
- `default_tag` must be one of: `general`, `microsoft`, `threat_intel`
- On user add (FR-036): app validates URL is reachable and returns a parseable feed before INSERT

---

### `articles`

Represents a single news article (FR-001–FR-016, FR-037–FR-039).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique article ID |
| `source_id` | INTEGER | NOT NULL, FOREIGN KEY → `news_sources.id` | Originating source |
| `url` | TEXT | NOT NULL, UNIQUE | Article URL — deduplication key (FR-037) |
| `title` | TEXT | NOT NULL | Article headline |
| `body_html` | TEXT | | Extracted HTML content (with images, cleaned of ads) |
| `body_text` | TEXT | | Plain-text content (for FTS indexing and LLM prompt) |
| `publish_date` | TEXT | NOT NULL | ISO 8601 date from feed; used for dashboard filtering (FR-002–FR-004) |
| `category` | TEXT | NOT NULL, DEFAULT 'general' | Resolved category: `general`, `microsoft`, `threat_intel` |
| `image_urls` | TEXT | | JSON array of image URLs extracted from article body (FR-011) |
| `is_cached` | INTEGER | NOT NULL, DEFAULT 0 | 1 = full content cached locally (FR-016) |
| `cached_at` | TEXT | | ISO 8601 timestamp when content was cached; used for eviction (FR-038) |
| `extraction_method` | TEXT | | How content was obtained: `readability`, `scraper`, `webview`, `failed` |
| `created_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | ISO 8601 |
| `updated_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | ISO 8601 |

**Indexes**:
- `idx_articles_url` ON (`url`) — enforces UNIQUE constraint, fast dedup lookup
- `idx_articles_publish_date` ON (`publish_date` DESC) — dashboard date filtering
- `idx_articles_category` ON (`category`) — dashboard section queries
- `idx_articles_source` ON (`source_id`) — JOIN optimization
- `idx_articles_cached_at` ON (`cached_at`) WHERE `is_cached = 1` — cache eviction queries

**Validation Rules**:
- `url` must be a valid HTTP/HTTPS URL
- `publish_date` must be a valid ISO 8601 date string
- `category` must be one of: `general`, `microsoft`, `threat_intel`
- `extraction_method` must be one of: `readability`, `scraper`, `webview`, `failed`, or NULL (not yet extracted)

**State Transitions**:

```text
                ┌─────────────┐
                │  Ingested    │  article row created from feed
                │  (no body)   │  is_cached = 0, extraction_method = NULL
                └──────┬──────┘
                       │ user opens article
                       ▼
              ┌────────────────┐
              │  Extracting    │  readability crate attempts extraction
              └───┬───┬───┬───┘
                  │   │   │
     success ─────┘   │   └───── failure (empty body)
                      │
          ┌───────────┴───────────┐
          │                       │
          ▼                       ▼
  ┌──────────────┐      ┌──────────────────┐
  │   Cached     │      │ Scraper Fallback │
  │ is_cached=1  │      │ (site-specific)  │
  │ method=      │      └───┬──────┬───────┘
  │ 'readability'│          │      │
  └──────────────┘   success│      │failure
                          ┌─┘      └─┐
                          ▼          ▼
                 ┌──────────┐  ┌──────────────┐
                 │  Cached   │  │ WebView      │
                 │ method=   │  │ Fallback     │
                 │ 'scraper' │  │ (FR-039)     │
                 └──────────┘  └──┬──────┬────┘
                                  │      │
                           success│      │failure
                                  ▼      ▼
                          ┌────────┐  ┌─────────────┐
                          │ Cached  │  │ Failed      │
                          │method=  │  │ method=     │
                          │'webview'│  │ 'failed'    │
                          └────────┘  │ Show "Open   │
                                      │ in browser"  │
                                      │ (FR-015)     │
                                      └─────────────┘
```

**Cache Eviction** (FR-038):
- Background job runs on app launch and periodically
- Deletes rows where `is_cached = 1` AND `cached_at < (NOW - retention_period)`
- Retention period from `app_settings` key `cache_retention_days` (default: 30)
- Eviction sets `body_html = NULL`, `body_text = NULL`, `is_cached = 0`, `cached_at = NULL`
  (preserves article metadata for search index; FTS row is also updated)

---

### `kql_suggestions`

Represents a generated KQL query from the LLM service (FR-023–FR-030).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique suggestion ID |
| `article_id` | INTEGER | NOT NULL, FOREIGN KEY → `articles.id` ON DELETE CASCADE | Associated article |
| `query_text` | TEXT | NOT NULL | Full KQL query string (FR-028) |
| `description` | TEXT | NOT NULL | Plain-language description of what the query detects (FR-026) |
| `target_table` | TEXT | | Primary Sentinel table targeted (e.g., `SecurityEvent`, `SigninLogs`) |
| `llm_endpoint` | TEXT | NOT NULL | Endpoint URL used for generation (audit trail) |
| `llm_model` | TEXT | | Model name if provided |
| `created_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | ISO 8601 |

**Indexes**:
- `idx_kql_article` ON (`article_id`) — fetch suggestions for an article

**Validation Rules**:
- `query_text` must not be empty
- `description` must not be empty
- `article_id` must reference an existing article

---

### `articles_fts` (FTS5 Virtual Table)

Full-text search index for keyword search (FR-017–FR-022). Uses SQLite FTS5 extension.

```sql
CREATE VIRTUAL TABLE articles_fts USING fts5(
    title,
    body_text,
    content='articles',
    content_rowid='id',
    tokenize='porter unicode61'
);
```

**Configuration**:
- **Tokenizer**: `porter unicode61` — Porter stemming + Unicode normalization for English text
- **Content sync**: External content table (`articles`); requires triggers to keep FTS in sync:

```sql
-- Trigger: INSERT sync
CREATE TRIGGER articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, title, body_text)
    VALUES (new.id, new.title, new.body_text);
END;

-- Trigger: DELETE sync
CREATE TRIGGER articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, body_text)
    VALUES ('delete', old.id, old.title, old.body_text);
END;

-- Trigger: UPDATE sync
CREATE TRIGGER articles_au AFTER UPDATE OF title, body_text ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, body_text)
    VALUES ('delete', old.id, old.title, old.body_text);
    INSERT INTO articles_fts(rowid, title, body_text)
    VALUES (new.id, new.title, new.body_text);
END;
```

**Search Query Pattern** (FR-018, FR-019, FR-021):

```sql
SELECT
    a.id,
    a.title,
    a.url,
    a.source_id,
    a.publish_date,
    snippet(articles_fts, 1, '<mark>', '</mark>', '...', 32) AS excerpt,
    rank
FROM articles_fts
JOIN articles a ON a.id = articles_fts.rowid
WHERE articles_fts MATCH ?1
  AND a.publish_date BETWEEN ?2 AND ?3  -- date-range filter (FR-019)
ORDER BY rank                            -- BM25 relevance (FR-021)
LIMIT ?4 OFFSET ?5;
```

Column weights: title matches are weighted 10x higher than body_text matches:
```sql
-- Use bm25() with column weights
ORDER BY bm25(articles_fts, 10.0, 1.0)
```

---

### `app_settings`

Key-value store for user-configurable preferences (FR-032, FR-038).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `key` | TEXT | PRIMARY KEY | Setting identifier |
| `value` | TEXT | NOT NULL | Setting value (serialized as text) |
| `updated_at` | TEXT | NOT NULL, DEFAULT CURRENT_TIMESTAMP | ISO 8601 |

**Default Rows**:

| Key | Default Value | Description | FR |
|-----|--------------|-------------|-----|
| `llm_endpoint` | `""` (empty) | LLM service API URL | FR-032 |
| `llm_model` | `""` (empty) | Optional model name | FR-032 |
| `feed_refresh_interval_min` | `60` | Feed auto-refresh interval in minutes | FR-006 |
| `cache_retention_days` | `30` | Days before cached content is evicted | FR-038 |
| `theme` | `system` | UI theme: `system`, `light`, `dark` | — |

**Note**: The LLM API key is NOT stored in this table. Per FR-032, it is stored in the OS
keychain via the `keyring` crate. The keychain service name is `cyber-news-reader` and the
account name is `llm-api-key`.

---

## Rust Struct Mapping

These structs live in `src-tauri/src/models/` and map to the SQLite schema above.

### `Article` → `models/article.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: i64,
    pub source_id: i64,
    pub url: String,
    pub title: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub publish_date: String,         // ISO 8601
    pub category: ArticleCategory,
    pub image_urls: Vec<String>,      // deserialized from JSON column
    pub is_cached: bool,
    pub cached_at: Option<String>,
    pub extraction_method: Option<ExtractionMethod>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArticleCategory {
    General,
    Microsoft,
    ThreatIntel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    Readability,
    Scraper,
    Webview,
    Failed,
}
```

### `NewsSource` → `models/source.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSource {
    pub id: i64,
    pub name: String,
    pub feed_url: String,
    pub feed_type: FeedType,
    pub default_tag: ArticleCategory,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedType {
    Rss,
    Atom,
    Json,
}
```

### `KqlSuggestion` → `models/kql_suggestion.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KqlSuggestion {
    pub id: i64,
    pub article_id: i64,
    pub query_text: String,
    pub description: String,
    pub target_table: Option<String>,
    pub llm_endpoint: String,
    pub llm_model: Option<String>,
    pub created_at: String,
}
```

### `SearchResult` → `models/search.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub article_id: i64,
    pub title: String,
    pub url: String,
    pub source_name: String,
    pub publish_date: String,
    pub excerpt: String,        // FTS5 snippet with highlights
    pub relevance_rank: f64,    // BM25 score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub keywords: String,
    pub date_from: Option<String>,  // ISO 8601
    pub date_to: Option<String>,    // ISO 8601
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

### `AppSettings` → `models/settings.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub llm_endpoint: String,
    pub llm_model: String,
    pub feed_refresh_interval_min: u32,
    pub cache_retention_days: u32,
    pub theme: Theme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            llm_endpoint: String::new(),
            llm_model: String::new(),
            feed_refresh_interval_min: 60,
            cache_retention_days: 30,
            theme: Theme::System,
        }
    }
}
```

---

## Schema Migration (Initial)

File: `src-tauri/src/db/migrations/001_initial.sql`

```sql
-- news_sources table
CREATE TABLE IF NOT EXISTS news_sources (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    feed_url    TEXT    NOT NULL UNIQUE,
    feed_type   TEXT    NOT NULL DEFAULT 'rss',
    default_tag TEXT    NOT NULL DEFAULT 'general',
    is_default  INTEGER NOT NULL DEFAULT 0,
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- articles table
CREATE TABLE IF NOT EXISTS articles (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id         INTEGER NOT NULL REFERENCES news_sources(id),
    url               TEXT    NOT NULL UNIQUE,
    title             TEXT    NOT NULL,
    body_html         TEXT,
    body_text         TEXT,
    publish_date      TEXT    NOT NULL,
    category          TEXT    NOT NULL DEFAULT 'general',
    image_urls        TEXT,  -- JSON array
    is_cached         INTEGER NOT NULL DEFAULT 0,
    cached_at         TEXT,
    extraction_method TEXT,
    created_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at        TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- kql_suggestions table
CREATE TABLE IF NOT EXISTS kql_suggestions (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    article_id    INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    query_text    TEXT    NOT NULL,
    description   TEXT    NOT NULL,
    target_table  TEXT,
    llm_endpoint  TEXT    NOT NULL,
    llm_model     TEXT,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- app_settings key-value store
CREATE TABLE IF NOT EXISTS app_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- FTS5 virtual table for article search
CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
    title,
    body_text,
    content='articles',
    content_rowid='id',
    tokenize='porter unicode61'
);

-- FTS sync triggers
CREATE TRIGGER IF NOT EXISTS articles_ai AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, title, body_text)
    VALUES (new.id, new.title, new.body_text);
END;

CREATE TRIGGER IF NOT EXISTS articles_ad AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, body_text)
    VALUES ('delete', old.id, old.title, old.body_text);
END;

CREATE TRIGGER IF NOT EXISTS articles_au AFTER UPDATE OF title, body_text ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, body_text)
    VALUES ('delete', old.id, old.title, old.body_text);
    INSERT INTO articles_fts(rowid, title, body_text)
    VALUES (new.id, new.title, new.body_text);
END;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_news_sources_active ON news_sources(is_active) WHERE is_active = 1;
CREATE INDEX IF NOT EXISTS idx_news_sources_tag    ON news_sources(default_tag);
CREATE INDEX IF NOT EXISTS idx_articles_url         ON articles(url);
CREATE INDEX IF NOT EXISTS idx_articles_publish_date ON articles(publish_date DESC);
CREATE INDEX IF NOT EXISTS idx_articles_category    ON articles(category);
CREATE INDEX IF NOT EXISTS idx_articles_source      ON articles(source_id);
CREATE INDEX IF NOT EXISTS idx_articles_cached_at   ON articles(cached_at) WHERE is_cached = 1;
CREATE INDEX IF NOT EXISTS idx_kql_article          ON kql_suggestions(article_id);

-- Default settings
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('llm_endpoint', '');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('llm_model', '');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('feed_refresh_interval_min', '60');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('cache_retention_days', '30');
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('theme', 'system');
```
