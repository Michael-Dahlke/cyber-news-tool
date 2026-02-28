# Research: Cybersecurity News Reader & Dashboard

**Feature Branch**: `001-news-reader-dashboard`
**Date**: 2026-02-28

---

## R1 — Application Framework: Tauri v2 vs Electron

### Decision: Tauri v2

### Rationale

Tauri's native paradigm is TypeScript frontend + Rust backend — exactly the planned
architecture. It uses the system WebView (WKWebView on macOS, WebView2 on Windows) rather
than bundling Chromium, yielding a ~5 MB installer vs ~120 MB for Electron. Every requirement
maps to a mature Rust crate or first-party Tauri plugin with no capability gaps.

### Alternatives Considered

| Framework | Why Rejected |
|-----------|-------------|
| **Electron** | Bloated bundle (~120 MB), keychain library (`keytar`) archived since 2023, Rust is a second-class citizen requiring FFI workarounds |
| **Neutralinojs** | Immature plugin ecosystem, no keychain or SQLite support |
| **Wails** | Go backend — would abandon the superior Rust crate ecosystem for feed parsing and content extraction |
| **Flutter Desktop** | No TypeScript, weak RSS/content-extraction libraries in Dart |

### Key Dependencies (Tauri v2)

**Rust (Cargo)**:

- `tauri` v2 — app framework
- `rusqlite` with `bundled` + `fts5` features — SQLite + full-text search
- `feed-rs` — RSS/Atom/JSON Feed parser
- `reqwest` + `tokio` — async HTTP client
- `readability` — Mozilla Readability content extraction port
- `scraper` — CSS selector-based HTML extraction (supplement)
- `serde` + `serde_json` — serialization
- `tauri-plugin-os` — platform detection
- `keyring` — cross-platform OS keychain (macOS Keychain / Windows Credential Manager)

**TypeScript (npm)**:

- `react` + `react-dom` — UI framework
- `vite` + `@vitejs/plugin-react` — build tooling
- `@tauri-apps/api` — Tauri IPC bindings
- `@mozilla/readability` — JS fallback content extraction (loaded in WebView)

---

## R2 — UI Component Library

### Decision: shadcn/ui (Radix UI primitives + Tailwind CSS)

### Rationale

~30–60 KB gzipped total footprint, preserving Tauri's lightweight advantage. Components are
vendored into the project and fully customizable for native desktop feel on both macOS and
Windows. Radix UI provides industry-leading WAI-ARIA accessibility. CSS variable theming with
Tailwind's `dark:` variant enables trivial dark mode — critical for security professionals.
One gap (KQL syntax highlighting) is solved by adding `shiki` with its native Kusto grammar.

### Alternatives Considered

| Framework | Why Rejected |
|-----------|-------------|
| **Mantine** | Strongest runner-up — built-in code highlighting, but 2–4x larger bundle and less design flexibility for native desktop feel |
| **MUI** | Material Design looks like a Google product; ~150–250 KB bundle; DateRangePicker requires paid license |
| **Ant Design** | "Enterprise web app" aesthetic, largest bundle (~200–400 KB), China-focused ecosystem |
| **Chakra UI** | v2→v3 migration fractured ecosystem; missing DateRangePicker and Sidebar |

### Key Packages (npm)

- `tailwindcss` v4, `@tailwindcss/typography`
- `@radix-ui/react-*` (dialog, dropdown, scroll-area, tabs, tooltip)
- `class-variance-authority`, `clsx`, `tailwind-merge`
- `lucide-react` — icons
- `react-hook-form` + `zod` — form handling + validation
- `react-day-picker` — date range filter for search
- `shiki` — KQL syntax highlighting (Kusto grammar)
- `cmdk` — command palette / search bar

---

## R3 — E2E Testing Strategy

### Decision: Hybrid — Playwright (primary) + WebdriverIO/tauri-driver (native complement)

### Rationale

No single framework can E2E test a Tauri v2 app across all target platforms. macOS has no
WKWebView WebDriver (Apple doesn't provide one), so Tauri's `tauri-driver` only works on
Windows and Linux for testing the actual binary. The hybrid approach:

| Layer | Tool | Platforms | What It Tests |
|-------|------|-----------|---------------|
| Primary E2E | Playwright against Vite dev server | macOS, Windows, Linux | Navigation, rendering, forms, theme, search (~85–90%) |
| Native E2E | WebdriverIO + `tauri-driver` | Windows, Linux | Real binary, IPC, clipboard, keychain (~10–15% smoke) |
| Component | Vitest + `@tauri-apps/api/mocks` | All (jsdom) | Components, hooks, IPC call correctness |

Playwright's WebKit engine shares the same rendering core as macOS WKWebView, providing the
closest proxy for macOS rendering confidence.

### Alternatives Considered

| Framework | Why Rejected |
|-----------|-------------|
| **WebdriverIO only** | No macOS support; dealbreaker |
| **Cypress** | Can't control Tauri binary, no WebKit support, paid CI dashboard |
| **Appium 2** | Experimental, unreliable macOS support, no community tooling for Tauri |

### Key Packages

- **npm**: `@playwright/test`, `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`
- **Cargo**: `tauri-driver`
- **Already planned**: `vitest`, `@testing-library/react`

---

## R4 — Content Extraction Pipeline

### Decision: `readability` crate (primary) + `scraper` (site-specific overrides) + Readability.js in WebView (fallback)

### Rationale

The `readability` crate is a Rust port of Mozilla's Readability algorithm — the same algorithm
powering Firefox Reader View. It handles the majority of standard cybersecurity news sites with
zero per-site configuration. For the ~10–20% of edge cases (JS-rendered pages, non-standard
markup), `scraper` provides CSS-selector extraction for known problem sites, and Readability.js
running in Tauri's WebView handles the remaining JS-dependent pages.

### Extraction Pipeline

```text
reqwest (fetch HTML)
       │
       ▼
readability crate (primary extraction)
       │
       ├─ Success → clean article HTML → render in reader view
       │
       ▼ (empty/short body detected, <100 chars)
scraper-based site-specific rules (known domain overrides)
       │
       ├─ Success → clean article HTML → render in reader view
       │
       ▼ (still fails)
Tauri WebView + Readability.js (JS fallback, FR-039)
       │
       ├─ Success → extract via IPC → render in reader view
       │
       ▼ (still fails)
Show "Open in browser" fallback (FR-015)
```

### Site Compatibility Assessment

| Site | `readability` crate | Notes |
|------|-------------------|-------|
| BleepingComputer | Good | Standard article markup |
| The Hacker News | Good | Blog-style layout |
| Krebs on Security | Excellent | Simple WordPress layout |
| CISA.gov | Good | Clean semantic HTML |
| Microsoft Security Blog | Good | May need image URL resolution for relative paths |
| DarkReading | Fair | Heavy JS rendering; likely needs WebView fallback |

### Key Crates

```toml
readability = "0.3"    # Primary: Mozilla Readability algorithm port
scraper = "0.25"       # Supplement: CSS selector extraction for overrides
```

JS fallback: bundle `@mozilla/readability` in frontend assets for WebView execution path.

---

## R5 — LLM Integration for KQL Generation

### Decision: OpenAI-compatible Chat Completions API with user-configurable endpoint

### Rationale

The OpenAI Chat Completions API (`/v1/chat/completions`) has become the de facto standard
interface implemented by Azure OpenAI, OpenAI, Ollama, LM Studio, vLLM, and dozens of other
providers. By targeting this single API format, the app supports the widest range of providers
with one implementation. The user configures endpoint URL, API key, and optional model name
in Settings.

### Compatible Providers

| Provider | Endpoint Pattern | Notes |
|----------|-----------------|-------|
| Azure OpenAI | `https://{resource}.openai.azure.com/openai/deployments/{deployment}/chat/completions?api-version=2024-10-21` | Enterprise; requires Azure subscription |
| OpenAI | `https://api.openai.com/v1/chat/completions` | Direct access; pay-per-token |
| Ollama (local) | `http://localhost:11434/v1/chat/completions` | Free, private, offline-capable |
| LM Studio (local) | `http://localhost:1234/v1/chat/completions` | Free, private, offline-capable |
| Any OpenAI-compatible | User-specified URL | Covers vLLM, Together AI, Fireworks, etc. |

### Prompt Design (High Level)

The KQL generation prompt will:

1. Provide system context: role as a Microsoft Sentinel/Defender XDR KQL expert
2. Include the article text (truncated to model context window limits)
3. Request structured output: array of `{ query, description, table }`
4. Specify that queries should target common Sentinel tables (SecurityEvent, SigninLogs,
   DeviceProcessEvents, EmailEvents, etc.)

### Key Dependencies

**Rust**: `reqwest` (already included) + `serde_json` for constructing/parsing Chat Completions
request/response. No additional dedicated LLM client crate needed.

---

## R6 — SQLite Full-Text Search Strategy

### Decision: SQLite FTS5 via `rusqlite`

### Rationale

SQLite's FTS5 extension provides built-in full-text search with BM25 ranking, prefix queries,
and column weighting — all within the same database file already used for article storage.
No external search engine (e.g., Tantivy, MeiliSearch) is needed.

### Implementation Approach

- FTS5 virtual table indexed on `article_title` and `article_body_text`
- Column weighting: title matches ranked higher than body matches
- Date-range filtering via JOIN with the main articles table
- Search results return article ID, title, excerpt (FTS5 snippet function), and rank

### Key Configuration

```toml
[dependencies.rusqlite]
version = "0.32"
features = ["bundled", "fts5"]
```

---

## Summary of All Resolved Items

| Technical Context Item | Resolution |
|----------------------|------------|
| Framework: Tauri v2 vs Electron | **Tauri v2** — lightweight, native Rust support, system WebView |
| UI framework | **shadcn/ui** (Radix + Tailwind) — lightweight, customizable, dark mode |
| E2E testing | **Playwright (primary) + WebdriverIO/tauri-driver (native complement)** |
| Content extraction | **readability crate + scraper overrides + Readability.js WebView fallback** |
| LLM integration | **OpenAI-compatible Chat Completions API, user-configurable endpoint** |
| Full-text search | **SQLite FTS5 via rusqlite** |
