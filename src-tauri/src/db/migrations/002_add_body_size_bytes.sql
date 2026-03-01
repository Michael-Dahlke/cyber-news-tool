-- 002_add_body_size_bytes.sql: Add body_size_bytes column to articles table
-- and new app_settings keys for the 2026-03-01 spec revision (FR-006, FR-038, FR-043)

-- Add body_size_bytes column for cache size cap accounting (FR-038)
ALTER TABLE articles ADD COLUMN body_size_bytes INTEGER NOT NULL DEFAULT 0;

-- Replace feed_refresh_interval_min with background_refresh_threshold_hours (FR-006)
DELETE FROM app_settings WHERE key = 'feed_refresh_interval_min';
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('background_refresh_threshold_hours', '4');

-- Add article cache size limit in GB (FR-038)
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('article_cache_size_limit_gb', '20');

-- Add reader font size in pixels (FR-043)
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('reader_font_size', '16');

-- Add last_refresh_at for background-focus threshold refresh (FR-006)
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('last_refresh_at', '');
