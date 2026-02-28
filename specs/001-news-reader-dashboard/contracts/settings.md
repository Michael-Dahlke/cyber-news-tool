# Tauri IPC Contracts: Settings

**Command Prefix**: `settings::`
**Rust Module**: `src-tauri/src/commands/settings.rs`

These commands manage application settings and OS keychain operations (FR-032, FR-038).

---

## `settings::get_all`

Retrieves all current application settings.

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type GetAllResponse = {
  settings: AppSettings;
};

type AppSettings = {
  llm_endpoint: string;           // May be empty if not configured
  llm_model: string;              // May be empty
  has_api_key: boolean;           // True if API key exists in keychain (never returns the key itself)
  feed_refresh_interval_min: number;
  cache_retention_days: number;
  theme: "system" | "light" | "dark";
};
```

**Note**: The API key is NEVER returned to the frontend. Only `has_api_key` indicates whether one is stored.

---

## `settings::update`

Updates one or more settings (FR-032).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type UpdateParams = {
  llm_endpoint?: string;
  llm_model?: string;
  feed_refresh_interval_min?: number;
  cache_retention_days?: number;
  theme?: "system" | "light" | "dark";
};
```

**Response**:

```typescript
type UpdateResponse = {
  settings: AppSettings;
};
```

**Errors**:
- `INVALID_INTERVAL` — `feed_refresh_interval_min` is less than 1
- `INVALID_RETENTION` — `cache_retention_days` is less than 1

**Behavior**: Only provided fields are updated. Omitted fields remain unchanged. After updating `feed_refresh_interval_min`, the background refresh timer is rescheduled.

---

## `settings::set_api_key`

Stores the LLM API key in the OS keychain (FR-032).

**Direction**: Frontend → Backend
**Parameters**:

```typescript
type SetApiKeyParams = {
  api_key: string;
};
```

**Response**:

```typescript
type SetApiKeyResponse = {
  success: boolean;
};
```

**Errors**:
- `KEYCHAIN_ERROR` — OS keychain access denied or unavailable
- `EMPTY_KEY` — Provided key is empty

**Behavior**: Stores in OS keychain with service name `cyber-news-reader`, account `llm-api-key`. Overwrites any existing value.

---

## `settings::clear_api_key`

Removes the LLM API key from the OS keychain.

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type ClearApiKeyResponse = {
  success: boolean;
};
```

**Errors**:
- `KEYCHAIN_ERROR` — OS keychain access failed

---

## `settings::get_app_info`

Returns application metadata for the Settings / About screen.

**Direction**: Frontend → Backend
**Parameters**: None

**Response**:

```typescript
type AppInfoResponse = {
  version: string;         // App version from Cargo.toml
  platform: string;        // "macos" | "windows"
  arch: string;            // "aarch64" | "x86_64"
  db_path: string;         // Path to SQLite database file
  db_size_bytes: number;   // Current DB file size
};
```
