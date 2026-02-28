# Tauri IPC Contracts: KQL Generator

**Command Prefix**: `kql::`
**Rust Module**: `src-tauri/src/commands/kql.rs`

These commands handle AI-powered KQL query generation from article content (FR-023–FR-030).

> **⚠ Gen AI Requirement**: All commands in this module require a configured LLM endpoint and
> API key. The app delegates AI inference to an external OpenAI-compatible API — no model is
> bundled.

---

## `kql::generate`

Sends article content to the configured LLM service and returns KQL query suggestions (FR-023–FR-028).

> **Constitution II — AI/LLM Governance**: The frontend MUST display a confirmation dialog
> before calling this command. The dialog MUST show the target endpoint URL and article title.
> The `user_confirmed` parameter proves the user explicitly approved this invocation.

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type GenerateParams = {
  article_id: number;
  user_confirmed: boolean;  // MUST be true — proof of explicit user consent
};
```

**Response**:

```typescript
type GenerateResponse = {
  suggestions: KqlSuggestion[];
  llm_endpoint: string;       // Endpoint used (for audit display)
  llm_model: string | null;   // Model used, if specified
};

type KqlSuggestion = {
  id: number;
  query_text: string;           // Full KQL query (FR-028)
  description: string;          // Plain-language detection description (FR-026)
  target_table: string | null;  // Primary Sentinel table (e.g., "SecurityEvent")
  created_at: string;           // ISO 8601
};
```

**Errors**:
- `USER_NOT_CONFIRMED` — `user_confirmed` is false or missing; frontend must show confirmation dialog
- `NOT_CONFIGURED` — LLM endpoint or API key not set (FR-029); frontend should redirect to Settings
- `ARTICLE_NOT_FOUND` — No article with the given ID
- `ARTICLE_NO_CONTENT` — Article body not extracted yet; trigger extraction first
- `LLM_TIMEOUT` — LLM request timed out (default: 60 seconds)
- `LLM_ERROR` — LLM returned a non-2xx status or unparseable response (FR-030)
- `LLM_NO_SUGGESTIONS` — LLM indicated no detectable threat indicators in the article

**Behavior**:
1. Validates `user_confirmed == true` (Constitution II — rejects if false)
2. Validates LLM endpoint + API key are configured (FR-029)
3. Retrieves article `body_text` from DB
4. Constructs Chat Completions request:
   - System prompt: KQL expert context, target Sentinel/Defender XDR tables
   - User prompt: article text (truncated to ~4000 tokens if needed)
   - Response format instruction: JSON array of `{ query, description, table }`
5. Sends POST to configured endpoint with `Authorization: Bearer {api_key}` header
6. Parses response, validates each suggestion, stores in `kql_suggestions` table (local only — Constitution III)
7. Returns stored suggestions

---

## `kql::get_for_article`

Retrieves previously generated KQL suggestions for an article (avoids re-generating).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type GetForArticleParams = {
  article_id: number;
};
```

**Response**:

```typescript
type GetForArticleResponse = {
  suggestions: KqlSuggestion[];
  generated_at: string | null;  // Timestamp of last generation, null if never generated
};
```

**Errors**:
- `ARTICLE_NOT_FOUND` — No article with the given ID

---

## `kql::copy_to_clipboard`

Copies a KQL query to the system clipboard (FR-027).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type CopyToClipboardParams = {
  query_text: string;
};
```

**Response**:

```typescript
type CopyToClipboardResponse = {
  success: boolean;
};
```

**Errors**:
- `CLIPBOARD_ERROR` — OS clipboard access failed

**Note**: This could also be handled entirely in the frontend via the Clipboard API, but routing through Tauri ensures consistent cross-platform behavior. Implementation may choose either approach.

---

## `kql::check_config`

Quick check whether the LLM service is configured and reachable.

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type CheckConfigResponse = {
  is_configured: boolean;     // endpoint + API key both set
  endpoint: string;           // Configured endpoint (masked if sensitive)
  model: string | null;       // Configured model name
  is_reachable: boolean;      // True if a test request succeeded
  error: string | null;       // Error message if unreachable
};
```

**Behavior**: If configured, sends a minimal test request (e.g., empty prompt with `max_tokens: 1`) to validate connectivity. Does not consume meaningful tokens.
