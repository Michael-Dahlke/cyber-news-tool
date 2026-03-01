# cyber-news Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-28

## Active Technologies
- SQLite local database for articles, sources, and settings; OS keychain for API key (001-news-reader-dashboard)

- TypeScript 5.x (frontend), Rust (backend/core via Tauri v2) + Tauri v2, React 19, shadcn/ui (Radix UI + Tailwind CSS), SQLite via `rusqlite` (bundled + FTS5), `feed-rs` (Rust RSS/Atom parser), `readability` + `scraper` (content extraction), `reqwest` + `tokio` (async HTTP), `keyring` (OS keychain) (001-news-reader-dashboard)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

TypeScript 5.x (frontend), Rust (backend/core via Tauri v2): Follow standard conventions

## Recent Changes
- 001-news-reader-dashboard: Added TypeScript 5.x (frontend), Rust (backend/core via Tauri v2) + Tauri v2, React 19, shadcn/ui (Radix UI + Tailwind CSS), SQLite via `rusqlite` (bundled + FTS5), `feed-rs` (Rust RSS/Atom parser), `readability` + `scraper` (content extraction), `reqwest` + `tokio` (async HTTP), `keyring` (OS keychain)

- 001-news-reader-dashboard: Added TypeScript 5.x (frontend), Rust (backend/core via Tauri v2) + Tauri v2, React 19, shadcn/ui (Radix UI + Tailwind CSS), SQLite via `rusqlite` (bundled + FTS5), `feed-rs` (Rust RSS/Atom parser), `readability` + `scraper` (content extraction), `reqwest` + `tokio` (async HTTP), `keyring` (OS keychain)

<!-- MANUAL ADDITIONS START -->

## Constitution

This project is governed by `.specify/memory/constitution.md` (v1.0.0). All code changes MUST comply with:

1. **Research-Only Purpose** — personal-use cybersecurity research tool only
2. **AI/LLM Governance (NON-NEGOTIABLE)** — all LLM calls require authenticated access AND explicit user confirmation before sending
3. **Data Containment** — AI-generated results stored locally only, never forwarded to external services
4. **Secret Security (NON-NEGOTIABLE)** — no plaintext secrets in source code; API keys in OS keychain only via `keyring` crate

When implementing AI/LLM features: always require `user_confirmed: true` parameter, validate credentials before requests, and store results in local SQLite only.

<!-- MANUAL ADDITIONS END -->
