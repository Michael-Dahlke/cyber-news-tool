# Tauri IPC Contracts: Search

**Command Prefix**: `search::`
**Rust Module**: `src-tauri/src/commands/search.rs`

These commands handle full-text keyword search with date-range filtering (FR-017–FR-022).

---

## `search::query`

Performs a keyword search across all articles using SQLite FTS5.

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type SearchQueryParams = {
  keywords: string;           // FTS5 query string; supports AND/OR/NOT and prefix*
  date_from?: string;         // ISO 8601 date (inclusive); omit for no lower bound
  date_to?: string;           // ISO 8601 date (inclusive); omit for no upper bound
  limit?: number;             // Default: 30
  offset?: number;            // Default: 0
};
```

**Response**:

```typescript
type SearchQueryResponse = {
  results: SearchResult[];
  total_count: number;
  query_time_ms: number;
};

type SearchResult = {
  article_id: number;
  title: string;
  url: string;
  source_name: string;
  publish_date: string;         // ISO 8601
  excerpt: string;              // FTS5 snippet with <mark> highlights (FR-020)
  relevance_rank: number;       // BM25 score
};
```

**Errors**:
- `INVALID_QUERY` — FTS5 syntax error in keywords (e.g., unbalanced quotes)
- `QUERY_TOO_SHORT` — Query is empty or whitespace-only

**Behavior**:
- Uses FTS5 `MATCH` with BM25 ranking, title weighted 10x over body (FR-021)
- Date-range filter is applied via JOIN with `articles` table on `publish_date` (FR-019)
- Snippets are extracted via FTS5 `snippet()` function with `<mark>` delimiters (FR-020)
- Results sorted by BM25 relevance; among equal-relevance results, most recent first (FR-021)
- Returns empty array for no matches (not an error)

---

## `search::suggest`

Returns search term suggestions based on recent searches or popular terms (future enhancement).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type SuggestParams = {
  prefix: string;   // Partial input from the search bar
  limit?: number;   // Default: 5
};
```

**Response**:

```typescript
type SuggestResponse = {
  suggestions: string[];
};
```

**Note**: This command is a placeholder for future enhancement. Initial implementation may return an empty array. Included in the contract to reserve the IPC slot and avoid breaking changes later.
