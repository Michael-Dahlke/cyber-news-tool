# Cyber News Tool

A macOS desktop application for aggregating cybersecurity news and cyber threat intelligence into a single dashboard, with an in-app reader, full-text search, and AI-powered KQL query generation for Microsoft Sentinel and Defender XDR.

**Personal-use research tool**, built for my own cybersecurity research and shared as-is under the [MIT License](LICENSE).

![Platform](https://img.shields.io/badge/platform-macOS-blue)
![Stack](https://img.shields.io/badge/stack-Tauri%20v2%20%7C%20React%20%7C%20Rust-orange)
![Status](https://img.shields.io/badge/status-active%20(personal%20use)-green)
[![CI](https://github.com/Michael-Dahlke/cyber-news-tool/actions/workflows/ci.yml/badge.svg)](https://github.com/Michael-Dahlke/cyber-news-tool/actions/workflows/ci.yml)
[![Security audit](https://github.com/Michael-Dahlke/cyber-news-tool/actions/workflows/security.yml/badge.svg)](https://github.com/Michael-Dahlke/cyber-news-tool/actions/workflows/security.yml)

---

## Features

### Categorized News Dashboard
Curated sections updated from RSS/Atom feeds:
- **Today's News** — articles published today across all sources
- **Microsoft Security News** — Microsoft-related articles from the past 7 days
- **Threat Intelligence** — threat actor groups, malware families, and TTPs from the past 7 days
- **General** — general cybersecurity articles from the past 7 days

MSRC security advisories and CVEs are available in a dedicated tab. Feeds refresh on
launch and when the app regains focus after the configured background threshold
(4 hours by default); a manual refresh is always available. Works offline with
cached articles.

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
| Secret Storage | OS keychain via `keyring` crate (macOS Keychain) |
| LLM Integration | OpenAI-compatible Chat Completions API (user-configured) |

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.90+ (stable) | [rustup.rs](https://rustup.rs/) |
| Node.js | 20.19+ or 22.12+ | [nodejs.org](https://nodejs.org/) |
| pnpm | 12.x (CI uses 12.10.1) | `npm install -g pnpm` |
| Xcode Command Line Tools | — | `xcode-select --install` |

The Tauri CLI is a project dev dependency (`@tauri-apps/cli`), so `pnpm tauri …`
works after `pnpm install`; no global install is needed.

> If `pnpm tauri dev` fails with `failed to run 'cargo metadata'`, run
> `./scripts/fix-rust-path.sh` to repair broken rustup shims.

---

## Getting Started

```bash
# Clone
git clone https://github.com/Michael-Dahlke/cyber-news-tool.git
cd cyber-news-tool

# Install frontend dependencies
pnpm install

# Launch dev mode (Vite dev server + debug Rust build, live reload)
pnpm tauri dev
```

The SQLite database is auto-created on first launch at
`~/Library/Application Support/com.cyber-news-reader.app/cyber-news-reader.db`.
Dev and release builds share this database.

### Building and installing the app

```bash
pnpm tauri build
```

This produces a release build at
`src-tauri/target/release/bundle/macos/Cyber News Reader.app` (plus a `.dmg`).
Copy the `.app` into `/Applications`, replacing any previous version. The
installed app doesn't update itself, so rebuild after pulling new changes.

---

## Development

### Checks

Run the same checks CI runs before opening a PR:

```bash
pnpm build                                              # tsc + vite build
cd src-tauri
cargo clippy --all-targets --locked -- -D warnings      # lint, warnings are errors
cargo test --locked                                     # Rust unit tests
```

Rust unit tests live next to the code in `#[cfg(test)]` modules (sanitization,
feed parsing and classification, search, cache eviction, migrations, LLM
endpoint validation). Feed fixtures are in `src-tauri/tests/fixtures/`.

### Continuous integration

| Workflow | Runs | When |
|----------|------|------|
| **CI** → Lint & test (Linux) | Frontend build, clippy, tests | Every PR (**required** to merge), or manually |
| **CI** → Build & test (macOS) | Frontend build, tests on macOS | Monthly (first day, 14:00 UTC), or manually via *Actions → CI → Run workflow* |
| **Security audit** | `cargo audit`, `pnpm audit --prod` | PRs changing dependency manifests, lockfiles, or the audit workflow; weekly; or manually |

Merges to `main` do not trigger duplicate runs; use pull requests for changes.
Scheduled CI runs only the macOS job. Manual CI runs both jobs. Superseded
runs on the same ref are canceled in both workflows. CI still runs on docs-only
PRs so its required check can complete.

`main` is protected by a ruleset: changes go through pull requests, the Linux
check must pass, and force-pushes and deletion are blocked. Merged branches are
deleted automatically.

### Dependency policy

- **Dependabot** opens grouped update PRs monthly (Cargo, npm) with at most
  3 open per ecosystem; GitHub Actions updates monthly.
- **Supply-chain cooldown:** new versions must be public for **7 days**
  (14 for major versions) before adoption. Dependabot enforces this via
  `cooldown`, and pnpm via `minimumReleaseAge` in `pnpm-workspace.yaml`.
  Cargo has no stable equivalent yet, so check publish dates on crates.io for
  manual `cargo update`s.
- **Audit exceptions** go in `pnpm-workspace.yaml` (`auditConfig.ignoreGhsas`)
  with a reason and a removal condition.

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
├── tests/fixtures/            # Sample RSS/Atom feeds for unit tests
└── Cargo.toml

src/                           # React frontend
├── components/
│   ├── Dashboard/             # News sections, article cards
│   ├── Reader/                # Content renderer, WebView fallback
│   ├── Search/                # Search bar, filters, results
│   ├── Sources/               # Browse articles by source
│   ├── KQL/                   # KQL panel, query cards
│   ├── Settings/              # Source manager, LLM config
│   └── Layout/                # App shell, sidebar, error boundary
├── hooks/                     # Theme context and hooks
├── services/                  # Frontend invoke wrappers
├── types/                     # TypeScript type definitions
└── App.tsx

.github/
├── workflows/                 # CI and security audit
└── dependabot.yml             # Dependency updates + cooldown

specs/                         # Feature specifications
└── 001-news-reader-dashboard/ # Original build spec (historical; targeted macOS + Windows)
    ├── spec.md                # Requirements (45 FRs, 4 user stories)
    ├── plan.md                # Architecture & implementation plan
    ├── tasks.md               # 80 tasks across 8 phases (complete)
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

Additional safeguards in the code:
- Extracted article HTML is sanitized with an allowlist (`ammonia`) before rendering.
- LLM endpoints must use HTTPS; plain HTTP is allowed only for loopback hosts
  (`localhost`, `127.0.0.1`, `::1`) to support local models, and URLs with
  embedded credentials are rejected.

---

## App Experience

### Dashboard
<img width="1724" height="987" alt="image" src="https://github.com/user-attachments/assets/d26d5165-75c6-4a1a-9e06-77d11c626a72" />

### Browse by Source
<img width="1717" height="991" alt="image" src="https://github.com/user-attachments/assets/85ef1e97-76fc-4c81-a25c-c84b5cf61328" />

### Article View
<img width="1718" height="993" alt="image" src="https://github.com/user-attachments/assets/ba7a7d97-ba46-4edb-aa52-17ddb686a042" />

### KQL Generator
<img width="1724" height="997" alt="image" src="https://github.com/user-attachments/assets/279cf488-fcb9-4a30-8e63-58071ba7076f" />

---

## Security

To report a vulnerability, see [SECURITY.md](SECURITY.md). Please don't open public issues for security problems.

---

## License

[MIT](LICENSE)
