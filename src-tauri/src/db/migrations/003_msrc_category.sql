-- 003_msrc_category.sql: Separate the Microsoft Security Response Center (MSRC)
-- feed into its own "msrc" category so its high-volume CVE posts no longer
-- clutter the main dashboard sections and live on a dedicated MSRC tab.

-- Re-tag the MSRC source so future articles are classified as "msrc".
UPDATE news_sources
SET default_tag = 'msrc'
WHERE name = 'Microsoft Security Response Center';

-- Re-categorize existing articles already imported from the MSRC source.
UPDATE articles
SET category = 'msrc'
WHERE source_id IN (
    SELECT id FROM news_sources WHERE name = 'Microsoft Security Response Center'
);
