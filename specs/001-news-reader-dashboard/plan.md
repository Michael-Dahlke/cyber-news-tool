# Implementation Plan: Cybersecurity News Reader & Dashboard

**Branch**: `001-news-reader-dashboard` | **Date**: 2026-02-28 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-news-reader-dashboard/spec.md`

## Summary

Cross-platform desktop application (macOS + Windows) that aggregates cybersecurity news from
publicly available RSS/Atom feeds into a categorized dashboard (Today's News, Microsoft News,
Threat Intelligence). Includes an in-app reader with content extraction and WebView fallback,
full-text keyword search with date-range filtering, and an AI-powered KQL query generator that
uses a user-configured LLM service to produce Microsoft Sentinel / Defender XDR detection queries
from article context.

## Technical Context

**Language/Version**: TypeScript 5.x (frontend), Rust (backend/core via Tauri v2)  
**Primary Dependencies**: Tauri v2, React 19, shadcn/ui (Radix UI + Tailwind CSS), SQLite via `rusqlite` (bundled + FTS5), `feed-rs` (Rust RSS/Atom parser), `readability` + `scraper` (content extraction), `reqwest` + `tokio` (async HTTP), `keyring` (OS keychain)  
**Storage**: SQLite local database for articles, sources, and settings; OS keychain for API key  
**Testing**: Vitest (frontend unit), `cargo test` (Rust unit), Playwright (primary E2E against Vite dev server — all platforms including WebKit for macOS proxy), WebdriverIO + `tauri-driver` (native E2E — Windows + Linux only)  
**Target Platform**: macOS (Apple Silicon + Intel), Windows 10/11  
**Project Type**: desktop-app  
**Performance Goals**: Dashboard load <10s, reader render <5s, search <3s, feed refresh <30s (from SC-001–SC-004)  
**Constraints**: Offline-capable (cached articles), single-user, no server component, API key stored in OS keychain  
**Scale/Scope**: Single user, ~20–50 RSS sources, ~1000s of articles in local cache, 5 primary screens (dashboard, reader, search, settings, KQL panel)

> **⚠ Gen AI Requirement**: The KQL generator (FR-023–FR-030) requires an external LLM service.
> The app MUST support configuration of any OpenAI-compatible API endpoint. Compatible providers include:
> - **Azure OpenAI Service** — recommended for enterprises already using Azure
> - **OpenAI API** — direct access to GPT models
> - **Any OpenAI-compatible endpoint** — e.g., local models via Ollama, LM Studio, or other providers
>
> The user configures the endpoint URL, API key, and optionally a model name in Settings.
> The app sends article text as a prompt and receives KQL query suggestions in the response.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Constitution**: v1.0.0 (ratified 2026-02-28) — 4 principles, 2 policy sections.

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Research-Only Purpose | **PASS** | App is personal-use cybersecurity news/CTI tool. No commercial redistribution features. Spec scoped to personal dashboard + reader + search + KQL gen. |
| II. AI/LLM Governance (NON-NEGOTIABLE) | **PASS** | FR-029 requires API key before AI use. FR-032 mandates keychain storage. KQL contract `kql::generate` requires `user_confirmed: true` parameter. `NOT_CONFIGURED` error blocks unconfigured use. `kql::generate` behavior step 1 validates credentials before any request. |
| III. Data Containment | **PASS** | KQL suggestions stored in local SQLite `kql_suggestions` table only. No telemetry, analytics, or external forwarding in any contract. All IPC commands return results to frontend UI only. |
| IV. Secret Security (NON-NEGOTIABLE) | **PASS** | FR-032 specifies OS keychain via `keyring` crate. `settings::set_api_key` stores in keychain with service `cyber-news-reader`. `settings::get_all` returns `has_api_key: boolean` — never the key itself. No `.env` files or plaintext config for secrets in project structure. |

| Policy Section | Status | Evidence |
|---------------|--------|----------|
| Security Requirements | **PASS** | Keychain-only storage (contracts/settings.md). No secret logging (implementation constraint). HTTPS-only for external calls except localhost (reqwest default + config). |
| AI Usage Policy | **PASS** | AI limited to KQL generation (FR-023–FR-030). User-initiated only ("Generate KQL Queries" button). Confirmation dialog required (constitution II + `user_confirmed` param). Result containment (local DB only). Error transparency (FR-030 + `LLM_ERROR`/`LLM_TIMEOUT` errors). |

**Pre-Phase 0 status**: PASS (vacuously — constitution not yet ratified).
**Post-Phase 1 re-check (2026-02-28)**: PASS — all 4 principles and 2 policy sections satisfied.

> **No violations. No Complexity Tracking entries required.**

## Project Structure

### Documentation (this feature)

```text
specs/001-news-reader-dashboard/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src-tauri/
├── src/
│   ├── main.rs              # Tauri entry point
│   ├── commands/             # Tauri command handlers (IPC bridge)
│   │   ├── feeds.rs          # Feed fetch, refresh, source management
│   │   ├── articles.rs       # Article retrieval, caching, content extraction
│   │   ├── search.rs         # Full-text search logic
│   │   ├── kql.rs            # LLM integration for KQL generation
│   │   └── settings.rs       # App settings + keychain access
│   ├── models/               # Rust data structures
│   │   ├── article.rs
│   │   ├── source.rs
│   │   ├── kql_suggestion.rs
│   │   └── settings.rs
│   ├── services/             # Core business logic
│   │   ├── feed_service.rs   # RSS/Atom parsing, dedup, classification
│   │   ├── content_extractor.rs  # HTML → reader content
│   │   ├── search_service.rs # SQLite FTS queries
│   │   ├── kql_service.rs    # LLM prompt construction + response parsing
│   │   ├── cache_service.rs  # Cache eviction, offline support
│   │   └── keychain.rs       # OS keychain abstraction
│   └── db/
│       ├── mod.rs            # SQLite connection pool
│       ├── seed.rs           # Default source seeding
│       └── migrations/       # Schema migrations
├── Cargo.toml
└── tauri.conf.json

src/                          # React frontend
├── components/
│   ├── Dashboard/
│   │   ├── DashboardView.tsx
│   │   ├── NewsSection.tsx
│   │   └── ArticleCard.tsx
│   ├── Reader/
│   │   ├── ReaderView.tsx
│   │   ├── ContentRenderer.tsx
│   │   └── WebViewFallback.tsx
│   ├── Search/
│   │   ├── SearchBar.tsx
│   │   ├── SearchResults.tsx
│   │   └── DateRangeFilter.tsx
│   ├── KQL/
│   │   ├── KQLPanel.tsx
│   │   └── KQLQueryCard.tsx
│   ├── Settings/
│   │   ├── SettingsView.tsx
│   │   ├── SourceManager.tsx
│   │   └── LLMConfig.tsx
│   └── Layout/
│       ├── AppShell.tsx
│       ├── Sidebar.tsx
│       ├── ErrorBoundary.tsx
│       └── OfflineIndicator.tsx
├── hooks/                    # React hooks for Tauri IPC
├── services/                 # Frontend service layer (Tauri invoke wrappers)
├── types/                    # TypeScript type definitions
├── App.tsx
└── main.tsx

tests/
├── unit/                     # Vitest unit tests (frontend)
├── backend/                  # Rust unit tests (in src-tauri/src/)
└── e2e/                      # Playwright E2E tests
```

**Structure Decision**: Tauri v2 project with Rust backend and React/TypeScript frontend.
The Rust backend handles all I/O-intensive work (feed fetching, content extraction, SQLite,
keychain, LLM API calls). The React frontend handles UI rendering and communicates with the
backend via Tauri's IPC command system. This structure maps directly to Tauri's conventions.

## Complexity Tracking

> No constitution gates exist, so no violations to justify. Included for completeness.

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| Dual-language (Rust + TypeScript) | Required by Tauri architecture | Single-language alternatives (pure Electron/JS) trade app size and performance; Tauri's system WebView satisfies FR-039 natively |
| WebView fallback (FR-039) | Built into Tauri | No additional complexity — Tauri apps already run in a WebView; the fallback simply loads the URL directly |
| OS keychain integration | `keyring` crate | Required by FR-032; platform-native, no custom crypto needed |
