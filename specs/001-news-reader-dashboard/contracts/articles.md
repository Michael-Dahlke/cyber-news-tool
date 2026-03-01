# Tauri IPC Contracts: Articles

**Command Prefix**: `articles::`
**Rust Module**: `src-tauri/src/commands/articles.rs`

These commands handle article retrieval, content extraction, and cache management.

---

## `articles::list_by_category`

Returns articles for a specific dashboard section (FR-001–FR-004).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type ListByCategoryParams = {
  category: "general" | "microsoft" | "threat_intel";
  days_back: number;  // 0 = today only (Today's News), 7 = past week (Microsoft/TI)
  limit?: number;     // Default: 50
  offset?: number;    // Default: 0
};
```

**Response**:

```typescript
type ListByCategoryResponse = {
  articles: ArticleSummary[];
  total_count: number;
};

type ArticleSummary = {
  id: number;
  title: string;
  url: string;
  source_id: number;
  source_name: string;
  publish_date: string;  // ISO 8601
  category: "general" | "microsoft" | "threat_intel";
  is_cached: boolean;
};
```

**Errors**: None expected (returns empty array when no articles match)

**Behavior**:
- `"general"` with `days_back: 0` → "Today's News" section (FR-002)
- `"microsoft"` with `days_back: 7` → "Microsoft News" section (FR-003)
- `"threat_intel"` with `days_back: 7` → "Threat Intelligence" section (FR-004)

---

## `articles::list_by_source`

Returns articles from a single feed source for the filtered source feed view (FR-040–FR-042).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type ListBySourceParams = {
  source_id: number;
  limit?: number;   // Default: 50
  offset?: number;  // Default: 0
};
```

**Response**:

```typescript
type ListBySourceResponse = {
  articles: ArticleSummary[];
  total_count: number;
  source_name: string;   // Convenience: name of the resolved source
};
```

**Errors**: None expected (returns empty array when source has no articles)

**Behavior**:
- Returns articles sorted newest-first (`publish_date DESC`)
- Includes articles from all time (no `days_back` filter)
- `source_id` must reference an active source in `news_sources`; if not found, returns empty array

---

## `articles::get_content`

Retrieves the full article content for the reader view (FR-009–FR-016, FR-039).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type GetContentParams = {
  article_id: number;
};
```

**Response**:

```typescript
type GetContentResponse = {
  article: ArticleDetail;
};

type ArticleDetail = {
  id: number;
  title: string;
  url: string;
  source_name: string;
  publish_date: string;         // ISO 8601
  body_html: string | null;     // Cleaned HTML for reader rendering (FR-010, FR-012)
  image_urls: string[];         // Extracted image URLs (FR-011)
  extraction_method: "readability" | "scraper" | "webview" | "failed" | null;
  is_cached: boolean;
  category: "general" | "microsoft" | "threat_intel";
};
```

**Errors**:
- `NOT_FOUND` — No article with the given ID
- `EXTRACTION_FAILED` — All extraction methods failed; frontend should show FR-015 fallback

**Behavior**: If `body_html` is NULL (article not yet extracted), triggers the content extraction pipeline:
1. `readability` crate extraction
2. `scraper` fallback for known domains
3. Returns with `extraction_method` set

If extraction fails entirely, returns `extraction_method: "failed"` and `body_html: null`. The frontend displays the FR-015 fallback ("Open in browser").

---

## `articles::extract_via_webview`

Called by the frontend when the standard extraction fails and the WebView fallback is needed (FR-039).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type ExtractViaWebviewParams = {
  article_id: number;
  extracted_html: string;  // HTML extracted by Readability.js running in the frontend WebView
};
```

**Response**:

```typescript
type ExtractViaWebviewResponse = {
  article: ArticleDetail;
};
```

**Errors**:
- `NOT_FOUND` — No article with the given ID

**Behavior**: Stores the WebView-extracted content, updates `extraction_method` to `"webview"`, marks as cached. The frontend orchestrates the WebView extraction and sends the result back via this command.

---

## `articles::get_cache_status`

Returns cache statistics for the Settings screen.

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type CacheStatusResponse = {
  cached_articles: number;
  total_articles: number;
  oldest_cached: string | null;       // ISO 8601
  body_cache_size_bytes: number;      // SUM of body_size_bytes for all cached articles
  cache_size_limit_gb: number;        // Current configured limit (from app_settings)
  size_cap_pct: number;               // body_cache_size_bytes / (cache_size_limit_gb * 1_073_741_824) * 100, rounded to 1 decimal
};
```

---

## `articles::evict_cache`

Manually triggers cache eviction based on the configured retention period (FR-038).

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type EvictCacheResponse = {
  evicted_count: number;
};
```
