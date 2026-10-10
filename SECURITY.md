# Security Policy

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Report privately through GitHub instead: go to the
[**Security** tab](https://github.com/Michael-Dahlke/cyber-news-tool/security) and
choose **Report a vulnerability**. Include steps to reproduce, the affected version or
commit, and the impact you expect.

This is a personal project maintained in spare time. I'll acknowledge reports as soon as
I can and coordinate a fix and disclosure with you.

## Supported versions

Only the latest commit on `main` is supported. There are no release branches.

## Scope

In scope, for example:

- Script injection or XSS through article content in the in-app reader
- Leaking the LLM API key (it must only ever live in the macOS Keychain)
- Bypassing the LLM endpoint HTTPS requirement, or sending data to the LLM without the
  explicit confirmation step
- SQL or FTS5 injection through search or feed data

Out of scope: vulnerabilities in third-party feeds or LLM services themselves, and issues
that require an already-compromised local machine.

## How this project manages security

- Extracted article HTML is sanitized with an allowlist before it is rendered.
- Secrets are stored only in the macOS Keychain.
- Dependencies are audited in CI (`cargo audit`, `pnpm audit`) on every dependency change
  and weekly. Updates must be public for at least 7 days before they are adopted.
- GitHub secret scanning with push protection is enabled.
