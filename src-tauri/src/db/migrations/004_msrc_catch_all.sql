-- 004_msrc_catch_all.sql: The MSRC CVE feed is published under several names
-- (e.g. "MSRC Security Update Guide" at api.msrc.microsoft.com). Migration 003
-- only matched the exact blog source name, so high-volume CVE feeds added by the
-- user still landed on the dashboard. Match all MSRC sources by URL/name here.

-- Re-tag any MSRC source so future articles are classified as "msrc".
UPDATE news_sources
SET default_tag = 'msrc'
WHERE feed_url LIKE '%msrc.microsoft.com%'
   OR name LIKE '%MSRC%'
   OR name LIKE '%Security Response Center%'
   OR name LIKE '%Security Update Guide%';

-- Re-categorize existing articles belonging to any MSRC-tagged source.
UPDATE articles
SET category = 'msrc'
WHERE source_id IN (
    SELECT id FROM news_sources WHERE default_tag = 'msrc'
);
