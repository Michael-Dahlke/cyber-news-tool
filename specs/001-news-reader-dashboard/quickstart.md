# Quickstart: Cybersecurity News Reader & Dashboard

**Feature Branch**: `001-news-reader-dashboard`

---

## Prerequisites

| Tool | Version | Purpose | Install |
|------|---------|---------|---------|
| **Rust** | 1.75+ (stable) | Backend compilation | [rustup.rs](https://rustup.rs/) |
| **Node.js** | 20 LTS+ | Frontend build | [nodejs.org](https://nodejs.org/) |
| **pnpm** | 9+ | Package manager (recommended for Tauri) | `npm install -g pnpm` |
| **Tauri CLI** | 2.x | App build & development | `cargo install tauri-cli --version "^2"` |

### Platform-Specific

**macOS**:
- Xcode Command Line Tools: `xcode-select --install`
- WKWebView ships with macOS — no additional WebView install needed

**Windows**:
- [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload
- [Rust for Windows](https://www.rust-lang.org/tools/install) — install via `rustup-init.exe`

---

## Project Setup

### 1. Clone the Repository

```bash
git clone https://github.com/Michael-Dahlke/cyber-news-tool.git
cd cyber-news-tool
```

### 2. Install Frontend Dependencies

```bash
pnpm install
```

### 3. Verify Rust Backend Compiles

```bash
cd src-tauri
cargo check
cd ..
```

### 4. Initialize the Database

The SQLite database is auto-created on first launch at the Tauri app data directory:
- **macOS**: `~/Library/Application Support/com.cyber-news-reader.app/cyber-news.db`
- **Windows**: `%APPDATA%\com.cyber-news-reader.app\cyber-news.db`

Schema migrations in `src-tauri/src/db/migrations/` run automatically on startup.

---

## Development

### Start Dev Server (Frontend + Backend Hot Reload)

```bash
pnpm tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420` (frontend HMR)
- Tauri Rust backend with cargo watch (recompiles on Rust file changes)
- App opens in a native window using the system WebView

### Frontend Only (for quick UI iteration)

```bash
pnpm dev
```

Opens the React app in a browser at `http://localhost:1420`. Tauri IPC calls will fail (no backend), but UI layout and component work is possible.

### Rust Backend Only (compile check)

```bash
cd src-tauri
cargo build
```

---

## Testing

### Available Checks

```bash
pnpm build             # TypeScript check and Vite production build
```

### Rust Unit Tests

```bash
cd src-tauri
cargo test
```

### Rust Compile Check

```bash
cd src-tauri
cargo check
cd ..
```

Frontend unit-test, Playwright, and native WebDriver scripts are not currently
configured in this repository.

---

## Build for Distribution

### macOS

```bash
pnpm tauri build
```

Output: `src-tauri/target/release/bundle/dmg/` (.dmg installer)

### Windows

```powershell
pnpm tauri build
```

Output: `src-tauri\target\release\bundle\nsis\` (.exe installer) or `msi\` (.msi installer)

---

## Configuration

### LLM Service (Required for KQL Generator)

1. Open the app → Settings
2. Enter your LLM endpoint URL:
   - Azure OpenAI: `https://{resource}.openai.azure.com/openai/deployments/{deployment}/chat/completions?api-version=2024-10-21`
   - OpenAI: `https://api.openai.com/v1/chat/completions`
   - Ollama (local): `http://localhost:11434/v1/chat/completions`
   - LM Studio (local): `http://localhost:1234/v1/chat/completions`
3. Enter your API key (stored in OS keychain — never written to disk)
4. Optionally specify a model name

### News Sources

The app ships with a curated default source list. To customize:
- Settings → Sources → toggle any default source on/off
- Settings → Sources → "Add Source" → paste an RSS/Atom feed URL

---

## Project Structure Overview

```text
cyber-news/
├── src/                     # React frontend (TypeScript)
│   ├── components/          # UI components (shadcn/ui + custom)
│   ├── hooks/               # React hooks for Tauri IPC
│   ├── services/            # Frontend service layer
│   ├── types/               # TypeScript type definitions
│   ├── App.tsx              # Root component
│   └── main.tsx             # Entry point
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/        # Tauri IPC command handlers
│   │   ├── models/          # Data structures
│   │   ├── services/        # Business logic
│   │   └── db/              # SQLite connection + migrations
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── tests/
│   ├── unit/                # Vitest frontend tests
│   └── e2e/                 # Playwright E2E tests
├── specs/                   # Feature specifications
│   └── 001-news-reader-dashboard/
├── package.json
├── vite.config.ts
└── tailwind.config.ts
```

---

## Troubleshooting

| Issue | Solution |
|-------|---------|
| `pnpm tauri dev` fails with missing WebView | **macOS**: Update macOS. **Windows**: Install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| Rust compile error on `rusqlite` | Ensure `bundled` feature is enabled in `Cargo.toml` — this builds SQLite from source, avoiding system dependency issues |
| Hot reload not working | Vite HMR requires the dev server URL in `tauri.conf.json` → `build.devUrl` to be `http://localhost:1420` |
| API key prompt on every launch | OS keychain access may require user approval on first use (macOS Keychain Access prompt) |
| FTS5 not found | Ensure `fts5` feature is enabled for `rusqlite` in `Cargo.toml` |
