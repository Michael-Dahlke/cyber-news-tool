# Cyber News Tool

A cross-platform desktop application for aggregating cybersecurity news and cyber threat intelligence into a single dashboard, with an in-app reader, full-text search, and AI-powered KQL query generation for Microsoft Sentinel and Defender XDR.

**Personal-use research tool** — not intended for commercial redistribution.

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)
![Stack](https://img.shields.io/badge/stack-Tauri%20v2%20%7C%20React%20%7C%20Rust-orange)
![Status](https://img.shields.io/badge/status-pre--development-yellow)

---

## Features

### Categorized News Dashboard
Three curated sections updated from RSS/Atom feeds:
- **Today's News** — articles published today across all sources
- **Microsoft Security News** — Microsoft-related articles from the past 7 days
- **Threat Intelligence** — threat actor groups, malware families, and TTPs from the past 7 days

Auto-refreshes on launch and every 60 minutes. Works offline with cached articles.

### In-App Article Reader
Click any headline to read the full article inside the app — no browser needed. Content extraction strips ads, navigation, and tracking. Falls back to an embedded WebView for JavaScript-heavy pages.

### Full-Text Search
Keyword search across all aggregated articles with date-range filtering, BM25 relevance ranking, and highlighted excerpts. Powered by SQLite FTS5.

### AI-Powered KQL Generator
Generate KQL detection queries from article content using any OpenAI-compatible LLM service (Azure OpenAI, OpenAI, Ollama, LM Studio). Queries are formatted for Microsoft Sentinel and Defender XDR with one-click copy to clipboard.

> Every AI call requires explicit user confirmation before any data is sent — no background or automatic AI requests.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri v2](https://v2.tauri.app/) — Rust backend + system WebView |
| Frontend | React 19, TypeScript, [shadcn/ui](https://ui.shadcn.com/) + Tailwind CSS v4 |
| Database | SQLite with FTS5 via `rusqlite` |
| Content Extraction | `readability` + `scraper` (Rust) → Readability.js WebView fallback |
| Feed Parsing | `feed-rs` (RSS/Atom/JSON) |
| Secret Storage | OS keychain via `keyring` crate (macOS Keychain / Windows Credential Manager) |
| LLM Integration | OpenAI-compatible Chat Completions API (user-configured) |

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.75+ (stable) | [rustup.rs](https://rustup.rs/) |
| Node.js | 20 LTS+ | [nodejs.org](https://nodejs.org/) |
| pnpm | 9+ | `npm install -g pnpm` |
| Tauri CLI | 2.x | `cargo install tauri-cli --version "^2"` |

**macOS**: Xcode Command Line Tools (`xcode-select --install`)

**Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload, [Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

---

## Getting Started

```bash
# Clone
git clone https://github.com/Michael-Dahlke/cyber-news-tool.git
cd cyber-news-tool

# Install frontend dependencies
pnpm install

# Verify Rust compiles
cd src-tauri && cargo check && cd ..

# Launch dev mode
pnpm tauri dev
```

The SQLite database is auto-created on first launch:
- **macOS**: `~/Library/Application Support/com.cyber-news-reader.app/cyber-news.db`
- **Windows**: `%APPDATA%\com.cyber-news-reader.app\cyber-news.db`

---

## Project Structure

```
src-tauri/                    # Rust backend
├── src/
│   ├── main.rs               # Tauri entry point
│   ├── commands/              # IPC command handlers
│   ├── models/                # Data structures
│   ├── services/              # Business logic
│   └── db/                    # SQLite + migrations
└── Cargo.toml

src/                           # React frontend
├── components/
│   ├── Dashboard/             # News sections, article cards
│   ├── Reader/                # Content renderer, WebView fallback
│   ├── Search/                # Search bar, filters, results
│   ├── KQL/                   # KQL panel, query cards
│   ├── Settings/              # Source manager, LLM config
│   └── Layout/                # App shell, sidebar, error boundary
├── hooks/                     # React hooks for Tauri IPC
├── services/                  # Frontend invoke wrappers
├── types/                     # TypeScript type definitions
└── App.tsx

specs/                         # Feature specifications
└── 001-news-reader-dashboard/
    ├── spec.md                # Requirements (39 FRs, 4 user stories)
    ├── plan.md                # Architecture & implementation plan
    ├── tasks.md               # 59 tasks across 7 phases
    ├── data-model.md          # SQLite schema & Rust models
    ├── research.md            # Technology decisions
    ├── quickstart.md          # Developer setup guide
    └── contracts/             # Tauri IPC API contracts
```

---

## Security & Governance

This project follows a [constitution](.specify/memory/constitution.md) with four core principles:

1. **Research-Only Purpose** — Personal cybersecurity research tool only
2. **AI/LLM Governance** *(non-negotiable)* — Authenticated access + explicit user confirmation before every AI call
3. **Data Containment** — AI-generated results stored locally only, never forwarded externally
4. **Secret Security** *(non-negotiable)* — All secrets in OS keychain only, never in plaintext files or logs

---

## License

[MIT](LICENSE)
