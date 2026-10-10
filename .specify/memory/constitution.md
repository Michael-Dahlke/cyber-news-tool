<!--
  Sync Impact Report
  ==================
  Version change: 1.0.0 → 1.0.1 (PATCH)
  Rationale: Windows dropped as a supported platform; macOS only.
  Modified principles: Security Requirements, credential store examples
    now list macOS Keychain only (requirement itself unchanged)
  Previous: (none) → 1.0.0, initial ratification
  Added sections:
    - Core Principles (4 principles)
    - Security Requirements
    - AI Usage Policy
    - Governance
  Removed sections: N/A
  Templates requiring updates:
    - specs/001-news-reader-dashboard/plan.md — ✅ updated (Constitution Check rewritten)
    - specs/001-news-reader-dashboard/contracts/kql.md — ✅ updated (user_confirmed param + USER_NOT_CONFIRMED error + Constitution II callout)
    - specs/001-news-reader-dashboard/spec.md — ✅ already aligned (FR-032, FR-029)
    - .specify/templates/tasks-template.md — ✅ no updates needed (generic template)
    - .specify/templates/plan-template.md — ✅ no updates needed (generic template)
    - .specify/templates/spec-template.md — ✅ no updates needed (generic template)
    - .github/agents/copilot-instructions.md — ✅ updated (constitution reference added)
  Follow-up TODOs: None
-->

# Cyber News Tool Constitution

## Core Principles

### I. Research-Only Purpose

This application is a **personal-use tool** for staying current on publicly
available cybersecurity news and cyber threat intelligence. It MUST only be
used for research purposes. The app MUST NOT be used for commercial
redistribution of aggregated content, automated threat response, or any
purpose beyond personal security research and awareness.

**Rationale**: Scoping the tool to personal research keeps the project
focused, avoids licensing complications with news sources, and ensures the
app remains a lightweight consumption tool rather than a platform.

### II. AI/LLM Governance (NON-NEGOTIABLE)

All access to external AI/LLM services (currently: KQL query generation)
MUST satisfy **both** of the following conditions before any request is sent:

1. **Authenticated access**: The LLM endpoint MUST require a valid API key
   or credential. The app MUST NOT send requests to unauthenticated AI
   endpoints. If no API key is configured, the app MUST block the AI
   feature and prompt the user to configure credentials.
2. **Explicit user confirmation**: The app MUST present a confirmation
   prompt to the user before every AI/LLM invocation. No AI request may be
   sent automatically, in the background, or without the user explicitly
   approving the specific action (e.g., "Generate KQL Queries — Send
   article content to [endpoint]? [Confirm / Cancel]").

**Rationale**: AI/LLM calls transmit article content to external services.
Requiring authentication prevents accidental leakage to open endpoints, and
explicit confirmation ensures the user maintains full control over what data
leaves the application.

### III. Data Containment

The results of any AI/LLM-generated content MUST only be returned to the
app. Generated content (e.g., KQL queries) MUST be stored locally in the
app's SQLite database and displayed within the app's UI. The app MUST NOT
forward, relay, or transmit AI-generated results to any third-party service,
external API, telemetry endpoint, or remote storage.

**Rationale**: As a personal research tool, all generated intelligence
artifacts stay local. This prevents unintended disclosure of detection
logic, search patterns, or inferred threat indicators to parties outside
the user's control.

### IV. Secret Security (NON-NEGOTIABLE)

No plaintext storage of secrets in the source code is acceptable. This
applies to **all** secret material including but not limited to:

- LLM API keys
- Authentication tokens
- Any credential or key used to access external services

Secrets MUST be stored exclusively in the operating system's native
credential store (macOS Keychain) via the
`keyring` crate. Secrets MUST NOT appear in:

- Source code files (`.rs`, `.ts`, `.tsx`, `.json`, `.toml`, etc.)
- Configuration files (`tauri.conf.json`, `.env`, etc.)
- Log output or debug traces
- Git history (committed or staged)

**Rationale**: Plaintext secrets in source code are the most common cause
of credential leakage in personal projects. OS keychain storage provides
encryption at rest and OS-level access control with zero additional
infrastructure.

## Security Requirements

The following security controls MUST be enforced across all implementation:

1. **Keychain-only secret storage**: All API keys and credentials MUST use
   the `keyring` crate with service name `cyber-news-reader`. No fallback
   to file-based storage is permitted.
2. **No secrets in logs**: Logging MUST redact or omit any field that could
   contain a secret. API keys MUST NOT appear in debug, info, or error
   log messages.
3. **No secrets in Git**: A `.gitignore` MUST exclude any file that could
   contain secrets (`.env`, `*.key`, etc.). Pre-commit hooks SHOULD scan
   for accidental secret commits.
4. **HTTPS-only for external calls**: All HTTP requests to external
   services (feed fetching, LLM API calls) MUST use HTTPS. Plain HTTP
   MUST NOT be used for any external communication except localhost
   development endpoints (e.g., Ollama at `http://localhost:*`).

## AI Usage Policy

These rules govern all current and future AI/LLM integrations in the app:

1. **Feature scope**: AI/LLM integration is currently limited to KQL query
   generation (FR-023–FR-030). Any future AI features MUST be documented
   in the feature spec with a `⚠ Gen AI Requirement` marker and MUST
   comply with all principles in this constitution.
2. **User-initiated only**: AI/LLM calls MUST only occur in response to an
   explicit user action (e.g., clicking "Generate KQL Queries"). The app
   MUST NOT make speculative, prefetched, or background AI calls.
3. **Confirmation dialog**: Before sending any data to an AI service, the
   app MUST display a confirmation dialog showing:
   - The target endpoint URL
   - A summary of the data being sent (e.g., "Article: [title]")
   - Confirm and Cancel actions
4. **Result containment**: AI-generated results MUST be stored only in the
   local SQLite database and displayed only in the app UI. No telemetry,
   analytics, or external reporting of AI results is permitted.
5. **Error transparency**: If an AI call fails, the error message MUST be
   shown to the user with actionable guidance (retry, check config). The
   app MUST NOT silently retry or suppress AI errors.

## Governance

This constitution is the highest-authority governance document for the
Cyber News Tool project. All implementation decisions, code reviews, and
feature additions MUST comply with the principles defined above.

- **Amendment procedure**: Any change to this constitution MUST be
  documented with a version bump, rationale, and updated Sync Impact
  Report. Principle removals or redefinitions require a MAJOR version
  bump. New principles or material expansions require a MINOR bump.
  Clarifications and wording fixes require a PATCH bump.
- **Versioning policy**: This constitution follows semantic versioning
  (MAJOR.MINOR.PATCH). The current version is tracked in the footer.
- **Compliance review**: The plan template's "Constitution Check" gate
  MUST evaluate all principles before Phase 0 research begins and again
  after Phase 1 design completes. Any violation MUST be documented in
  the plan's Complexity Tracking table with justification.
- **Agent compliance**: All AI coding agents working on this project MUST
  be aware of this constitution. The agent context file
  (`.github/agents/copilot-instructions.md`) MUST reference this document.

**Version**: 1.0.1 | **Ratified**: 2026-02-28 | **Last Amended**: 2026-10-10
