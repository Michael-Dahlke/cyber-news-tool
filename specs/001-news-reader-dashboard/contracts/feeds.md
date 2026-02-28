# Tauri IPC Contracts: Feeds

**Command Prefix**: `feeds::`
**Rust Module**: `src-tauri/src/commands/feeds.rs`

These commands manage RSS/Atom feed sources and trigger feed refresh operations.

---

## `feeds::list_sources`

Returns all configured news sources.

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type ListSourcesResponse = {
  sources: NewsSource[];
};

type NewsSource = {
  id: number;
  name: string;
  feed_url: string;
  feed_type: "rss" | "atom" | "json";
  default_tag: "general" | "microsoft" | "threat_intel";
  is_default: boolean;
  is_active: boolean;
  created_at: string;   // ISO 8601
  updated_at: string;   // ISO 8601
};
```

**Errors**: None expected (returns empty array if no sources)

---

## `feeds::add_source`

Validates and adds a user-provided RSS/Atom feed URL (FR-035, FR-036).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type AddSourceParams = {
  feed_url: string;         // URL to validate and add
  name?: string;            // Optional display name; auto-detected from feed if omitted
  default_tag?: "general" | "microsoft" | "threat_intel";  // Default: "general"
};
```

**Response**:

```typescript
type AddSourceResponse = {
  source: NewsSource;
};
```

**Errors**:
- `INVALID_URL` — URL is not a valid HTTP/HTTPS URL
- `UNREACHABLE` — URL could not be fetched (timeout, DNS failure, non-2xx status)
- `UNPARSEABLE_FEED` — Response is not a valid RSS/Atom/JSON feed
- `DUPLICATE_URL` — A source with this URL already exists

**Behavior**: Fetches the URL, attempts to parse it with `feed-rs`, extracts the feed title for `name` if not provided, and inserts into `news_sources`. Returns error before saving if validation fails.

---

## `feeds::update_source`

Updates a source's properties (FR-034).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type UpdateSourceParams = {
  id: number;
  name?: string;
  default_tag?: "general" | "microsoft" | "threat_intel";
  is_active?: boolean;
};
```

**Response**:

```typescript
type UpdateSourceResponse = {
  source: NewsSource;
};
```

**Errors**:
- `NOT_FOUND` — No source with the given ID

---

## `feeds::remove_source`

Removes a user-added source (FR-034). Default sources can only be deactivated, not removed.

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type RemoveSourceParams = {
  id: number;
};
```

**Response**:

```typescript
type RemoveSourceResponse = {
  success: boolean;
};
```

**Errors**:
- `NOT_FOUND` — No source with the given ID
- `CANNOT_REMOVE_DEFAULT` — Source is a default source; use `update_source` to deactivate

---

## `feeds::refresh`

Triggers a feed refresh for all active sources (FR-006).

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type RefreshResponse = {
  sources_fetched: number;
  articles_added: number;
  articles_deduplicated: number;  // FR-037: skipped because URL already exists
  errors: FeedError[];
};

type FeedError = {
  source_id: number;
  source_name: string;
  error: string;
};
```

**Errors**: None at the command level — per-source errors are returned in the `errors` array.

**Behavior**: Fetches all active sources concurrently, parses feeds, deduplicates by URL, classifies articles by source tag + keyword matching, and inserts new articles. Emits `feeds://refresh-progress` event during execution.

---

## Events (Backend → Frontend)

### `feeds://refresh-progress`

Emitted during `feeds::refresh` to report progress.

```typescript
type RefreshProgressEvent = {
  current: number;     // Sources processed so far
  total: number;       // Total active sources
  source_name: string; // Currently processing
};
```

### `feeds://auto-refresh-complete`

Emitted when the background auto-refresh timer fires and completes (FR-006).

```typescript
type AutoRefreshCompleteEvent = {
  articles_added: number;
  timestamp: string;   // ISO 8601 — updates "last updated" display (FR-007)
};
```
