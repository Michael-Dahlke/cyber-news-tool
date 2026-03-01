-- 001_initial.sql: Initial schema for Cyber News Reader
-- Creates all tables, FTS5 virtual table, triggers, indexes, and default settings

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
