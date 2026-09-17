# Tasks: Cybersecurity News Reader & Dashboard

**Input**: Design documents from `/specs/001-news-reader-dashboard/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Not included — tests were not explicitly requested in the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Rust backend**: `src-tauri/src/`
- **React frontend**: `src/`
- **Tests**: `tests/`
- Structure derived from plan.md Project Structure section

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Tauri v2 project with all required tooling and dependencies.

- [X] T001 Initialize Tauri v2 project with React + TypeScript template using `pnpm create tauri-app` in project root
- [X] T002 Configure Rust dependencies in src-tauri/Cargo.toml (tauri v2, rusqlite with bundled+fts5, feed-rs, readability, scraper, reqwest, tokio, keyring, serde, serde_json)
- [X] T003 [P] Configure frontend dependencies in package.json (react 19, react-dom, @tauri-apps/api, @tauri-apps/plugin-shell, tailwindcss v4, @tailwindcss/typography, lucide-react, class-variance-authority, clsx, tailwind-merge, react-hook-form, zod, react-day-picker, shiki, cmdk)
- [X] T004 [P] Configure Tailwind CSS v4 with shadcn/ui theme variables and dark mode in tailwind.config.ts and src/globals.css
- [X] T005 [P] Configure TypeScript strict mode in tsconfig.json and Vite build settings in vite.config.ts

**Checkpoint**: `pnpm tauri dev` launches an empty app window. `cargo check` passes in src-tauri/.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core database, models, keychain, settings, layout shell, and shared TypeScript types that ALL user stories depend on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T006 Create SQLite database module with connection pool and migration runner in src-tauri/src/db/mod.rs
- [X] T007 Create initial schema migration with all tables, FTS5 virtual table, triggers, indexes, and default settings rows in src-tauri/src/db/migrations/001_initial.sql
- [X] T008 [P] Create Article model with ArticleCategory and ExtractionMethod enums in src-tauri/src/models/article.rs
- [X] T009 [P] Create NewsSource model with FeedType enum in src-tauri/src/models/source.rs
- [X] T010 [P] Create AppSettings model with Theme enum and Default impl in src-tauri/src/models/settings.rs
- [X] T011 Create models module barrel file with re-exports in src-tauri/src/models/mod.rs
- [X] T012 [P] Implement keychain service for OS credential storage (macOS Keychain / Windows Credential Manager) with service name `cyber-news-reader` in src-tauri/src/services/keychain.rs
- [X] T013 [P] Create services module barrel file in src-tauri/src/services/mod.rs
- [X] T014 Implement settings Tauri commands (get_all, update, set_api_key, clear_api_key, get_app_info) per contracts/settings.md in src-tauri/src/commands/settings.rs
- [X] T015 Create commands module barrel file in src-tauri/src/commands/mod.rs
- [X] T016 Configure Tauri app entry point with module declarations, command registration, database initialization, and managed state in src-tauri/src/main.rs
- [X] T017 [P] Create shared TypeScript type definitions matching all contract types in src/types/ (article.ts, source.ts, search.ts, kql.ts, settings.ts, index.ts)
- [X] T018 [P] Create frontend Tauri invoke wrapper services in src/services/ (feedService.ts, articleService.ts, searchService.ts, kqlService.ts, settingsService.ts, index.ts)
- [X] T019 [P] Create AppShell layout component with sidebar navigation and content area in src/components/Layout/AppShell.tsx
- [X] T020 [P] Create Sidebar navigation component with route links (Dashboard, Search, Settings) in src/components/Layout/Sidebar.tsx
- [X] T021 Create app entry point with React Router and route definitions in src/App.tsx and src/main.tsx
- [X] T022 Seed default cybersecurity news sources (BleepingComputer, Krebs on Security, The Hacker News, CISA, Microsoft Security Blog, DarkReading, etc.) in src-tauri/src/db/seed.rs

**Checkpoint**: App launches with sidebar navigation, settings screen saves/loads preferences, API key stores in OS keychain. Database creates all tables on first run.

---

## Phase 3: User Story 1 — Categorized News Dashboard (Priority: P1) 🎯 MVP

**Goal**: Dashboard with four curated sections (Today's News, Microsoft News, Threat Intelligence, and General) plus a dedicated MSRC/CVE view, populated from RSS/Atom feeds with auto-refresh and offline support.

**Independent Test**: Launch app with internet → all four dashboard sections display articles with correct date filtering. Open the MSRC/CVE view to verify advisories are available. Kill internet → cached articles shown with offline indicator.

### Implementation for User Story 1

- [X] T023 [US1] Implement feed fetching and RSS/Atom parsing service using feed-rs with concurrent source fetching in src-tauri/src/services/feed_service.rs
- [X] T024 [US1] Implement article category classification logic with source-level tags and keyword matching for Microsoft and Threat Intelligence categories in src-tauri/src/services/feed_service.rs
- [X] T025 [US1] Implement URL deduplication logic (FR-037) — skip insert when article URL already exists in src-tauri/src/services/feed_service.rs
- [X] T026 [US1] Implement feeds Tauri commands (list_sources, add_source, update_source, remove_source, refresh) per contracts/feeds.md in src-tauri/src/commands/feeds.rs
- [X] T027 [US1] Implement articles list_by_category command with date filtering per contracts/articles.md in src-tauri/src/commands/articles.rs
- [X] T028 [US1] Implement launch/focus auto-refresh using the configurable background threshold and emit feeds://refresh-progress and feeds://auto-refresh-complete events in src-tauri/src/lib.rs
- [X] T029 [P] [US1] Create ArticleCard component displaying title, source name, publish date, and category badge in src/components/Dashboard/ArticleCard.tsx
- [X] T030 [P] [US1] Create NewsSection component with section header, last-updated timestamp (FR-007), scrollable article list, and "No articles found" placeholder in src/components/Dashboard/NewsSection.tsx
- [X] T031 [US1] Create DashboardView component assembling three NewsSection instances (Today days_back=0, Microsoft days_back=7, Threat Intel days_back=7) in src/components/Dashboard/DashboardView.tsx
- [X] T032 [US1] Create OfflineIndicator component that detects connectivity status and displays cached-data warning in src/components/Layout/OfflineIndicator.tsx
- [X] T033 [US1] Create SettingsView component with tabbed layout for general settings in src/components/Settings/SettingsView.tsx
- [X] T034 [P] [US1] Create SourceManager component with default source toggle, custom source add/validate, and remove controls in src/components/Settings/SourceManager.tsx

**Checkpoint**: Dashboard loads three populated sections. Feed refresh works. Sources can be managed in Settings. Offline mode shows cached articles with indicator.

---

## Phase 4: User Story 2 — In-App Article Reader (Priority: P2)

**Goal**: Click any article headline to open a clean reader view with extracted content, images, no ads, and fallback to WebView or browser when extraction fails.

**Independent Test**: Click any article → reader shows clean content with images and no ads. Disconnect internet → cached article renders. Force extraction failure → "Open in browser" fallback shown.

### Implementation for User Story 2

- [X] T035 [US2] Implement content extraction service with readability crate (primary), scraper CSS-selector overrides (secondary), and extraction status tracking in src-tauri/src/services/content_extractor.rs
- [X] T036 [US2] Implement articles get_content command with extraction pipeline trigger per contracts/articles.md in src-tauri/src/commands/articles.rs
- [X] T037 [US2] Implement extract_via_webview command for WebView fallback HTML storage in src-tauri/src/commands/articles.rs
- [X] T038 [US2] Implement cache service with eviction logic (FR-038, configurable retention, background eviction on launch) in src-tauri/src/services/cache_service.rs
- [X] T039 [US2] Implement get_cache_status and evict_cache commands in src-tauri/src/commands/articles.rs
- [X] T040 [P] [US2] Create ContentRenderer component that renders cleaned HTML with inline images and typography styles in src/components/Reader/ContentRenderer.tsx
- [X] T041 [P] [US2] Create WebViewFallback component that loads article URL in iframe, runs Readability.js extraction, and sends result back via extract_via_webview command in src/components/Reader/WebViewFallback.tsx
- [X] T042 [US2] Create ReaderView component with article header (title, source, date), content area, extraction-failed fallback (FR-015), and "Back" navigation control (FR-014) in src/components/Reader/ReaderView.tsx

**Checkpoint**: Clicking any article opens the reader with cleaned content and images. Cached articles readable offline. Failed extraction shows "Open in browser" link.

---

## Phase 5: User Story 3 — Keyword Search (Priority: P3)

**Goal**: Full-text keyword search across all articles with date-range filtering, BM25 relevance ranking, and highlighted excerpts. Clicking a result opens the reader.

**Independent Test**: Enter a keyword known to exist in aggregated articles → results returned with excerpts and highlighting. Apply date filter → results narrow correctly.

### Implementation for User Story 3

- [X] T043 [US3] Implement search service with FTS5 MATCH queries, BM25 ranking (title weighted 10x), date-range filtering, and snippet extraction in src-tauri/src/services/search_service.rs
- [X] T044 [US3] Implement search Tauri commands (query, suggest) per contracts/search.md in src-tauri/src/commands/search.rs
- [X] T045 [P] [US3] Create SearchBar component with keyword input and submit using cmdk in src/components/Search/SearchBar.tsx
- [X] T046 [P] [US3] Create DateRangeFilter component with date pickers using react-day-picker in src/components/Search/DateRangeFilter.tsx
- [X] T047 [US3] Create SearchResults component with result cards showing title, source, date, highlighted excerpt, and click-to-open-reader in src/components/Search/SearchResults.tsx

**Checkpoint**: Search returns relevant results with highlighted excerpts. Date filter works. Clicking a result opens the reader view.

---

## Phase 6: User Story 4 — KQL Query Generator (Priority: P4)

**Goal**: Generate KQL detection queries from article content via a user-configured LLM service, with explicit user confirmation before each AI call (Constitution II), and display formatted queries with copy-to-clipboard.

**Independent Test**: Open any article → click "Generate KQL Queries" → confirmation dialog shown → approve → KQL queries returned and displayed with descriptions. Copy button works. No LLM configured → redirected to Settings.

### Implementation for User Story 4

- [X] T048 [US4] Create KqlSuggestion model in src-tauri/src/models/kql_suggestion.rs and add to models/mod.rs
- [X] T049 [US4] Implement KQL service with Chat Completions prompt construction (system: KQL expert + Sentinel tables, user: article text), response parsing, suggestion validation, and local storage in src-tauri/src/services/kql_service.rs
- [X] T050 [US4] Implement kql Tauri commands (generate with user_confirmed validation, get_for_article, copy_to_clipboard, check_config) per contracts/kql.md in src-tauri/src/commands/kql.rs
- [X] T051 [P] [US4] Create KQLQueryCard component with query text (syntax-highlighted via shiki with Kusto grammar), description, target table label, and copy-to-clipboard button in src/components/KQL/KQLQueryCard.tsx
- [X] T052 [US4] Create KQLPanel component with confirmation dialog (shows endpoint URL + article title per Constitution II), generation trigger, loading state, error/retry, and "no suggestions" message in src/components/KQL/KQLPanel.tsx
- [X] T053 [US4] Create LLMConfig component for endpoint URL, model name input, API key set/clear, and connection test in src/components/Settings/LLMConfig.tsx
- [X] T054 [US4] Integrate KQL panel into ReaderView as a toggleable side panel triggered by "Generate KQL Queries" button in src/components/Reader/ReaderView.tsx

**Checkpoint**: KQL generation works end-to-end with confirmation dialog. Queries display with syntax highlighting. Copy to clipboard works. Settings allow LLM configuration.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Theme, error handling, performance, security hardening, and final validation.

- [X] T055 [P] Add dark mode theme toggle and system theme detection using Tailwind dark variant and app_settings theme preference in src/components/Layout/AppShell.tsx
- [X] T056 [P] Add React error boundary with user-friendly error display and recovery in src/components/Layout/ErrorBoundary.tsx
- [X] T057 Performance and quality validation: verify dashboard load <10s (SC-001), feed refresh <30s (SC-002), reader render <5s (SC-003), search <3s (SC-004), KQL returns ≥1 query for threat-content articles (SC-005), reader shows zero ads from top 10 sources (SC-007)
- [X] T058 Security hardening: validate HTTPS-only for external calls (except localhost LLM endpoints), audit that no secrets appear in logs or source, verify keychain-only storage
- [X] T059 Run quickstart.md validation end-to-end on macOS and verify all development workflows (dev server, build, test commands)

---

## Phase 8: Spec Revision 2026-03-01 — New Features & Updates

**Purpose**: Tasks from the 2026-03-01 spec session. Covers the refresh policy revision (FR-006), 20 GB cache cap (FR-038), reader accessibility — font size (FR-043) and image lightbox (FR-044–045) — and the new Source Browsing user story US5 (FR-040–042). Requires a new DB migration and model/type updates before any story work begins.

### Data Model & Type Updates (prerequisite for all Phase 8 implementation)

- [X] T060 Add `body_size_bytes INTEGER NOT NULL DEFAULT 0` column to articles table via new migration in src-tauri/src/db/migrations/002_add_body_size_bytes.sql
- [X] T061 [P] Update Article Rust struct to add `body_size_bytes: i64` field in src-tauri/src/models/article.rs
- [X] T062 [P] Update AppSettings Rust struct — replace `feed_refresh_interval_min` with `background_refresh_threshold_hours: i64`; add `article_cache_size_limit_gb: i64` and `reader_font_size: i32` in src-tauri/src/models/settings.rs
- [X] T063 [P] Update shared TypeScript types — add `source_id: number` to `ArticleSummary`; replace `feed_refresh_interval_min` with `background_refresh_threshold_hours`, add `article_cache_size_limit_gb` and `reader_font_size` to `AppSettings`; update `CacheStatusResponse` to use `body_cache_size_bytes`, `cache_size_limit_gb`, `size_cap_pct` in src/types/index.ts

---

### US1 Revision — Refresh Policy (FR-006)

- [X] T064 [US1] Replace auto-refresh timer in src-tauri/src/main.rs with cold-launch refresh; write `last_refresh_at` to app_settings after each successful refresh; subscribe to `WindowEvent::Focused` and trigger refresh only when `NOW - last_refresh_at > background_refresh_threshold_hours * 3600s` in src-tauri/src/main.rs

---

### US2 Revision — Cache Size Cap + Reader Accessibility (FR-038, FR-043–045)

- [X] T065 [US2] Update cache_service.rs — record `body_size_bytes = body_html.len() + body_text.len()` when writing cached content; add size-cap eviction loop (`SELECT SUM(body_size_bytes)` → oldest-`cached_at` batch DELETE until sum drops below `article_cache_size_limit_gb * 1_073_741_824`) in src-tauri/src/services/cache_service.rs
- [X] T066 [US2] Update articles::get_cache_status response to return `body_cache_size_bytes`, `cache_size_limit_gb`, `size_cap_pct` (1 decimal) per updated contracts/articles.md in src-tauri/src/commands/articles.rs
- [X] T067 [P] [US2] Update SettingsView.tsx — add cache size progress bar showing `size_cap_pct`% and a cache size limit config input (GB) alongside the existing retention-days field in src/components/Settings/SettingsView.tsx
- [X] T068 [P] [US2] Implement FontSizeControl.tsx — increment/decrement stepper (12–28 px range, step 2), applies `--reader-font-size` CSS variable on reader root div, persists value via `settingsService.update({ reader_font_size })` in src/components/Reader/FontSizeControl.tsx
- [X] T069 [US2] Implement ImageLightbox.tsx — Radix UI `<Dialog>` rendered full-screen (`position: fixed, inset: 0`), CSS `transform: scale()` driven by pointer events for pinch-to-zoom on trackpad, `overflow: auto` on dialog content with image at natural size for scroll-to-pan when zoomed, dismiss button returns to reader in src/components/Reader/ImageLightbox.tsx
- [X] T070 [US2] Update ContentRenderer.tsx — set `--reader-font-size` CSS variable on root div from settings; wrap each `<img>` element with an `onClick` handler that opens ImageLightbox with the clicked image src in src/components/Reader/ContentRenderer.tsx
- [X] T071 [US2] Update ReaderView.tsx — render `<FontSizeControl />` in the reader toolbar alongside the back-nav control in src/components/Reader/ReaderView.tsx

**Checkpoint**: Back-navigation never triggers a refresh. Cache size enforces 20 GB cap with oldest-first eviction. Reader font size persists across sessions. Tapping an image opens full-window lightbox with zoom and pan.

---

### US5 (New Story) — Source Browsing (FR-040–042) 🆕

**Goal**: Every article card and reader header shows a tappable source name that opens a filtered source feed view showing only articles from that source. A dedicated Sources page in the sidebar lists all active sources.

**Independent Test**: Tap the source chip on any article card → filtered article list for that source appears sorted newest-first with back navigation. Open Sources page from sidebar → all active sources listed → tap any → same filtered view.

- [X] T072 [US5] Implement articles::list_by_source IPC command — `source_id` parameter, newest-first `ORDER BY publish_date DESC`, pagination via `limit`/`offset`, convenience `source_name` in response per contracts/articles.md in src-tauri/src/commands/articles.rs
- [X] T073 [P] [US5] Add `listBySource` wrapper function to src/services/articleService.ts
- [X] T074 [P] [US5] Implement SourcesView.tsx — calls `feedService.listSources()`, renders a card per active source with name and tap-to-navigate to `/sources/:id` in src/components/Sources/SourcesView.tsx
- [X] T075 [US5] Implement SourceFeedView.tsx — reads `source_id` from route param, calls `articleService.listBySource()`, renders `ArticleCard` list newest-first with scroll pagination and a back-navigation control in src/components/Sources/SourceFeedView.tsx
- [X] T076 [US5] Update ArticleCard.tsx source chip to render as `<Link to={\`/sources/${article.source_id}\`}>` using React Router in src/components/Dashboard/ArticleCard.tsx
- [X] T077 [US5] Update ReaderView.tsx article header — wrap source name in `<Link to={\`/sources/${article.source_id}\`}>` in src/components/Reader/ReaderView.tsx
- [X] T078 [US5] Register `/sources` and `/sources/:id` routes in src/App.tsx; add Sources navigation entry (icon + label) to Sidebar.tsx in src/components/Layout/Sidebar.tsx

**Checkpoint**: All article cards show tappable source chip. Tapping navigates to filtered source feed. Sources page accessible from sidebar. Back navigation returns to previous context.

---

### US6 Revision — Settings IPC Update (FR-032, FR-038 revised)

- [X] T079 [US6] Update settings::update IPC — handle `background_refresh_threshold_hours`, `article_cache_size_limit_gb`, `reader_font_size` fields; remove `INVALID_INTERVAL` error; add `INVALID_THRESHOLD` (< 1), `INVALID_CACHE_LIMIT` (< 1), `INVALID_FONT_SIZE` (outside 12–28) errors per updated contracts/settings.md in src-tauri/src/commands/settings.rs
- [X] T080 [P] [US6] Update settingsService.ts to use revised `AppSettings` type and new `UpdateParams` fields in src/services/settingsService.ts

**Checkpoint**: Settings IPC correctly validates and persists all new fields. Updating threshold takes effect on next focus event.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3–6)**: All depend on Foundational phase completion
  - US1 (P1): Can start after Foundational — no dependencies on other stories
  - US2 (P2): Can start after Foundational — no runtime dependency on US1 (articles exist from feed refresh)
  - US3 (P3): Can start after Foundational — requires FTS5 triggers (created in migration) and articles in DB (populated by US1)
  - US4 (P4): Can start after Foundational — integrates into ReaderView (US2), so US2 should complete first
- **Polish (Phase 7)**: Depends on all user stories being complete
- **Phase 8 (Revision)**: T060–T063 (data model updates) must complete first — BLOCKS all Phase 8 story tasks
  - T064 (US1 refresh revision): Can start after T060–T063
  - T065–T071 (US2 cache + accessibility): Can start after T060–T063; T068–T071 only depend on T063
  - T072–T078 (US5 source browsing): T072 depends on T060–T061; T073–T080 can start after T063
  - T079–T080 (US6 settings revision): Can start after T062–T063

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational — No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational — Independent of US1 structurally, but needs articles in DB to test (US1 populates them)
- **User Story 3 (P3)**: Can start after Foundational — Needs articles in DB and FTS5 index populated (US1 feeds create articles, triggers populate FTS)
- **User Story 4 (P4)**: Depends on US2 — KQL panel integrates into ReaderView (T054)
- **User Story 5 (P5 — new)**: Depends on T060–T063 (data model updates); T075 depends on T072 (IPC command); T076–T077 depend on T063 (type update)

### Recommended Execution Order

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundational)
    ↓
Phase 3 (US1 — Dashboard) ← MVP delivery point
    ↓
Phase 4 (US2 — Reader)
    ↓
Phase 5 (US3 — Search)  ←─── can run in parallel with US4 if US2 is done
Phase 6 (US4 — KQL)     ←─┘
    ↓
Phase 7 (Polish)
    ↓
Phase 8 — Data model updates (T060–T063) first
    ├── US1 revision (T064)
    ├── US2 revision (T065–T071)
    ├── US5 source browsing (T072–T078)
    └── US6 settings revision (T079–T080)
```

### Within Each User Story

- Backend services before commands
- Commands before frontend components
- Shared/reusable components ([P]) before composite views
- Core implementation before integration

### Parallel Opportunities

**Setup**: T003, T004, T005 can run in parallel (different config files)

**Foundational**: T008, T009, T010 can run in parallel (independent model files); T012, T013 can run in parallel; T017, T018, T019, T020 can run in parallel (frontend files independent of backend)

**US1**: T029, T030, T034 can run in parallel (independent components)

**US2**: T040, T041 can run in parallel (independent components)

**US3**: T045, T046 can run in parallel (independent components)

**US4**: T051 can run in parallel with other [P]-marked tasks

**Phase 8 data model**: T061, T062, T063 can run in parallel (independent files) after T060

**Phase 8 US2 revision**: T068 (FontSizeControl) can run in parallel with T065–T066; T067 is independent frontend change

**Phase 8 US5**: T073, T074 can run in parallel after T072 completes; T076, T077 can run in parallel after T063

---

## Parallel Example: User Story 1

```text
# Launch all parallelizable US1 frontend components together:
Task T029: "Create ArticleCard component in src/components/Dashboard/ArticleCard.tsx"
Task T030: "Create NewsSection component in src/components/Dashboard/NewsSection.tsx"
Task T034: "Create SourceManager component in src/components/Settings/SourceManager.tsx"

# Then compose them:
Task T031: "Create DashboardView component (depends on T029, T030)"
Task T033: "Create SettingsView component (depends on T034)"
```

## Parallel Example: Foundational Models

```text
# Launch all model files together:
Task T008: "Create Article model in src-tauri/src/models/article.rs"
Task T009: "Create NewsSource model in src-tauri/src/models/source.rs"
Task T010: "Create AppSettings model in src-tauri/src/models/settings.rs"

# Then create barrel file:
Task T011: "Create models module barrel file in src-tauri/src/models/mod.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 — Categorized News Dashboard
4. **STOP and VALIDATE**: Dashboard loads with three populated sections, feeds refresh, sources manageable
5. This delivers the core value proposition — a single pane of glass for daily security news

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → **MVP!** (functional news dashboard)
3. Add User Story 2 → Test independently → In-app reader (no more leaving to browser)
4. Add User Story 3 → Test independently → Search across all articles
5. Add User Story 4 → Test independently → AI-powered KQL generation
6. Each story adds value without breaking previous stories

### Single Developer Strategy

Since this is a personal-use project:

1. Complete Setup + Foundational sequentially
2. Work through stories in priority order: US1 → US2 → US3 → US4
3. Within each story: backend first (services → commands), then frontend (components → view)
4. Validate each story independently at its checkpoint before moving on

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable at its checkpoint
- Constitution II compliance: KQL generation (T050, T052) MUST validate `user_confirmed == true` and show confirmation dialog
- Constitution IV compliance: API key MUST only be stored via keychain service (T012), never in plaintext
- Phase 8 prerequisite: T060–T063 (migration + model + type updates) MUST complete before any Phase 8 story task
- US5 source browsing (T072–T078) is a new user story added in the 2026-03-01 spec revision
- Back-navigation MUST NOT trigger feed refresh (FR-006 revised) — only `WindowEvent::Focused` elapsed-time check triggers it
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- No test tasks included — add them later if TDD approach is desired
