# Feature Specification: Cybersecurity News Reader & Dashboard

**Feature Branch**: `001-news-reader-dashboard`  
**Created**: 2026-02-28  
**Status**: Implemented
**Input**: User description: "One-stop cybersecurity news and CTI app with dashboard, in-app reader, keyword search, and AI-powered KQL generator — macOS and Windows"

## Assumptions

- News is aggregated from publicly available RSS/Atom feeds and open APIs; no paywalled sources are included.
- The dashboard includes Today's News, Microsoft News, Threat Intelligence, and General sections populated from the same aggregated source pool. MSRC/CVE advisories are presented in a dedicated view. Each NewsSource carries a default category tag (General, Microsoft, Threat Intelligence, or MSRC). Articles inherit their source's category tag; keyword matching on title and body can additionally assign or override the category for articles from General-tagged sources.
- The KQL generator sends article text to an external LLM service configured by the user; the app does not bundle an AI model.
- KQL queries are intended for use in Microsoft Sentinel and Microsoft Defender XDR.
- The app is a single-user desktop application; no accounts or authentication are required.
- Article content is cached locally to support offline reading of previously opened articles.
- The reader view uses a content-extraction approach (strip ads/navigation, keep body text and inline images).
- The app targets English-language articles; multi-language support is out of scope for this version.
- Users can add and remove sources from a default curated list. The app ships with a pre-configured set of reputable cybersecurity news sources. A Settings screen allows users to disable default sources or add their own RSS/Atom feed URLs. The app validates that added URLs are reachable and return a parseable feed before saving them.

## Clarifications

### Session 2026-02-28

- Q: How should the app handle duplicate articles appearing from multiple sources? → A: Deduplicate by exact URL — the same URL ingested from multiple feeds is stored and displayed once.
- Q: How are articles classified into dashboard categories (Microsoft News, Threat Intelligence)? → A: Both source-level tags and keyword matching — each feed carries a default category tag set at configuration time; keyword matching on title and body can additionally assign or override category for articles from general-purpose feeds.
- Q: How should the LLM API key be stored? → A: OS keychain — macOS Keychain on macOS, Windows Credential Manager on Windows.
- Q: What is the default article cache retention period? → A: 30 days — cached articles are automatically evicted after 30 days; configurable by the user in Settings.
- Q: How should the app handle articles that require JavaScript rendering? → A: Fallback to in-app WebView — attempt HTTP extraction first; if the body is empty or unparseable, render the page in an embedded WebView with ad-blocking applied.

### Session 2026-03-01

- Q: What does the 20GB storage limit cover, and how does it interact with the existing 30-day eviction rule? → A: The 20GB cap applies to cached article bodies only (not feed metadata or the index). When the cap is reached, the oldest-cached article bodies are purged first regardless of their age relative to the 30-day window; both rules operate simultaneously.
- Q: How should browsing articles by source be accessible? → A: Both — the source name is a tappable link on every article card and in the reader view header (opens a filtered "articles from this source" view), AND a dedicated Sources browser page lists all active sources with the same filtered view on tap.
- Q: When should feed refresh be triggered? → A: On cold launch (process start) and when the app resumes from background after a user-configurable idle threshold (default: 4 hours). Navigating back to the dashboard from the article reader or any other in-app view MUST NOT trigger a refresh. A manual refresh button is always available in the dashboard toolbar.
- Q: Should the app support non-RSS/Atom feed sources? → A: Deferred — non-RSS source integration (e.g., HTML scraping) is out of scope for this version and will be addressed in a future specification. Note: the Microsoft Security Blog Threat Intelligence topic feed (`https://www.microsoft.com/en-us/security/blog/topic/threat-intelligence/feed/`) is a valid RSS 2.0 feed and is added to the built-in default source list.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Categorized News Dashboard (Priority: P1)

A user opens the app and immediately sees four curated sections on the dashboard:
**Today's Cybersecurity News** (articles published today), **Microsoft Security News** (Microsoft-related
articles from the past 7 days), and **Threat Intelligence** (articles covering threat actor groups and TTPs
from the past 7 days), plus General cybersecurity news. Each section shows article headlines, source name, and publish date. The user can
scroll each section independently and click any headline to open the article.

**Why this priority**: This is the core value proposition — a single pane of glass for daily security news
awareness. Without this, the app has no reason to exist.

**Independent Test**: Can be fully tested by launching the app with a working internet connection and
verifying that all four sections display articles with correct date filtering. Delivers immediate,
standalone value as a news aggregator dashboard.

**Acceptance Scenarios**:

1. **Given** the app is opened with internet access, **When** the dashboard loads, **Then** the "Today's News" section displays at least one article published on the current calendar day.
2. **Given** the app is opened, **When** the dashboard loads, **Then** the "Microsoft News" section displays only articles related to Microsoft published within the past 7 days.
3. **Given** the app is opened, **When** the dashboard loads, **Then** the "Threat Intelligence" section displays only articles mentioning threat actor groups or TTPs published within the past 7 days.
4. **Given** the dashboard is displaying articles, **When** a section has no matching articles for the period, **Then** the section shows a "No articles found" placeholder rather than an error.
5. **Given** the app was last opened more than 1 hour ago, **When** the user opens the app, **Then** article feeds are automatically refreshed before display.
6. **Given** the app has no internet access, **When** the dashboard loads, **Then** the most recently cached articles are displayed with a visible "offline" indicator and the last-updated timestamp.

---

### User Story 2 - In-App Article Reader (Priority: P2)

A user clicks any article headline in the dashboard or search results. Instead of opening a browser, the
article opens inside the app in a clean reader view: article title, publication date, source name, the full
article body text, and any images embedded in the article are rendered. Navigation/menu elements, banner
ads, sidebar widgets, and tracking pixels are stripped from the view.

**Why this priority**: Opening articles in-app in a clean reader is a key differentiator from a simple
link list. Without it the app is just an RSS reader; with it, the user never leaves the app context.

**Independent Test**: Can be fully tested by clicking any article and verifying that the reader view loads
readable content with images and no ads, delivering a standalone in-app reading experience.

**Acceptance Scenarios**:

1. **Given** a user is on the dashboard or search results, **When** they click an article headline, **Then** the article opens in a reader view inside the app without launching an external browser.
2. **Given** the article reader is open, **When** the article contains inline images, **Then** those images are displayed within the reader view.
3. **Given** the article reader is open, **When** the source page contains advertisements, **Then** no advertisement content is rendered in the reader view.
4. **Given** the article reader is open, **When** a user scrolls to the bottom, **Then** a "Back to Dashboard" or navigation control returns them to their previous context.
5. **Given** the article was previously opened and cached, **When** the user opens it without internet access, **Then** the cached version is displayed with an indicator that it may not be the latest version.
6. **Given** the article reader is open, **When** the content extraction fails to parse the article body, **Then** the user is shown a fallback message with a link to open the article in their default browser.

---

### User Story 3 - Keyword Search (Priority: P3)

A user types one or more keywords into a search bar and receives a list of articles matching those terms
from across all aggregated sources. Results show the article title, source, publish date, and a brief
excerpt. The user can filter results by date range. Clicking any result opens the article in the in-app
reader.

**Why this priority**: Search enables users to investigate specific topics, threat actors, or CVEs on
demand, extending the value of the aggregated content beyond the fixed dashboard categories.

**Independent Test**: Can be fully tested by entering a known keyword that appears in aggregated content
and verifying relevant articles are returned. Requires the dashboard feed to be populated (US1 dependency),
but delivers independent search-workflow value.

**Acceptance Scenarios**:

1. **Given** the search bar is available, **When** a user types a keyword and submits, **Then** a results list is shown containing articles whose title or body contains that keyword.
2. **Given** search results are displayed, **When** the user applies a date-range filter, **Then** results are narrowed to only articles within the selected date range.
3. **Given** a user searches for a term with no matches, **When** results are returned, **Then** a "No results found" message is displayed rather than an empty list or error.
4. **Given** search results are displayed, **When** a user clicks a result, **Then** the article opens in the in-app reader (User Story 2 flow).
5. **Given** a user performs a search, **When** results are returned, **Then** they are sorted by relevance with the most recent articles ranked higher among equal-relevance results.

---

### User Story 4 - KQL Query Generator (Priority: P4)

While reading an article in the reader view, a user selects "Generate KQL Queries." The app sends the
article's content to a configured AI service, which returns a set of KQL query suggestions relevant to
detecting the threats, behaviors, or indicators described in the article. The queries are displayed in a
panel within the reader view, formatted and ready to copy into Microsoft Sentinel or Defender XDR.

> **⚠ Gen AI Requirement**: This user story requires integration with an external Large Language Model
> (LLM) service. The app must allow the user to configure which AI service and API key to use (e.g.,
> Azure OpenAI, OpenAI, or a compatible endpoint). The AI service selection and configuration must be
> documented in the implementation plan so the user can choose their preferred provider.

**Why this priority**: The KQL generator is a high-value differentiator for security practitioners but
depends on the reader view (US2) and an external AI service. It enhances rather than enables the core
experience.

**Independent Test**: Can be fully tested by opening any article in the reader view, triggering the KQL
generator with a configured AI service, and verifying that KQL queries relevant to the article's threat
content are returned and displayed.

**Acceptance Scenarios**:

1. **Given** an article is open in the reader view, **When** the user selects "Generate KQL Queries," **Then** a panel appears showing one or more KQL query suggestions derived from the article content.
2. **Given** the KQL panel is shown, **When** the user clicks "Copy" on a query, **Then** the query text is copied to the clipboard.
3. **Given** the KQL panel is shown, **When** a query is displayed, **Then** it is labeled with a brief description of what threat behavior or indicator it detects.
4. **Given** the KQL generator is triggered, **When** no AI service is configured, **Then** the user is shown a prompt to configure their AI service endpoint and API key in Settings.
5. **Given** the AI service call fails or times out, **When** the user triggers KQL generation, **Then** an error message explains the failure and offers a retry option.
6. **Given** an article with no detectable threat indicators is open, **When** KQL generation is triggered, **Then** the AI service response informs the user that no specific detections were identified rather than producing irrelevant queries.

---

### Edge Cases

- What happens when an article's source page has changed structure and content extraction produces garbled or empty output?
- How does the system handle articles that require JavaScript rendering and cannot be fetched via simple HTTP request? The app first attempts HTTP fetch + content extraction; if the extracted body is empty or unparseable, it falls back to rendering the page in an embedded WebView with ad-blocking applied.
- When the same article URL is ingested from multiple feeds, it is stored and displayed once (deduplicated by exact URL). Different sources covering the same story via distinct URLs each appear as separate entries.
- How does the reader handle articles that are extremely long (100,000+ characters)?
- What happens when the configured AI service returns malformed or non-KQL text?
- How does the app behave during the first launch when no feeds have been populated yet?
- What if an article URL redirects through multiple hops before reaching the content?
- What happens when the article body cache approaches but has not yet reached the 20 GB cap — should the user receive a warning?
- Integration of non-RSS/Atom sources (e.g., HTML scraping of pages without a feed) is **deferred** to a future specification. The Microsoft Security Blog Threat Intelligence topic page is already covered by its RSS feed (`https://www.microsoft.com/en-us/security/blog/topic/threat-intelligence/feed/`).

## Requirements *(mandatory)*

### Functional Requirements

#### Dashboard

- **FR-001**: The app MUST display a dashboard with four named sections: "Today's News," "Microsoft News," "Threat Intelligence," and "General." MSRC/CVE advisories MUST be available in a dedicated view.
- **FR-002**: "Today's News" MUST display only articles published on the current calendar day.
- **FR-003**: "Microsoft News" MUST display only articles classified under the Microsoft category from the past 7 days. Classification occurs via: (a) the article's source having a Microsoft category tag, or (b) keyword matching identifying Microsoft products, services, or security advisories in the article title or body.
- **FR-004**: "Threat Intelligence" MUST display only articles classified under the Threat Intelligence category from the past 7 days. Classification occurs via: (a) the article's source having a Threat Intelligence category tag, or (b) keyword matching identifying named threat actor groups, malware families, or TTPs in the article title or body.
- **FR-005**: The app MUST only surface articles from free, publicly accessible sources; no paywalled or subscription-gated content may be included.
- **FR-006**: The dashboard MUST auto-refresh article feeds on cold launch (process start) and when the app resumes from background after the user-configured idle threshold (default: 4 hours). Navigating back to the dashboard from the reader or any other in-app screen MUST NOT trigger a refresh. A visible manual refresh button MUST always be available in the dashboard toolbar.
- **FR-007**: The app MUST display a visible "last updated" timestamp on each dashboard section.
- **FR-008**: When no internet connection is available, the app MUST display the most recently cached articles with an offline indicator.

#### In-App Reader

- **FR-009**: The app MUST open all articles in an in-app reader view; no article MUST open in an external browser by default.
- **FR-010**: The reader view MUST render the full article body text.
- **FR-011**: The reader view MUST render inline images that are part of the article content.
- **FR-012**: The reader view MUST strip advertisements, navigation menus, sidebars, and non-editorial content from the displayed view.
- **FR-013**: The reader view MUST display the article title, source name, and publish date prominently.
- **FR-014**: The reader view MUST provide a navigation control to return to the previous dashboard or search context.
- **FR-015**: When content extraction fails, the app MUST display a fallback message and offer a control to open the article in the user's default browser.
- **FR-016**: Previously opened articles MUST be cached locally so they can be re-read without an internet connection.

#### Search

- **FR-017**: The app MUST provide a search interface accessible from any screen.
- **FR-018**: Search MUST accept one or more keywords and return articles whose title or body contains those terms.
- **FR-019**: Search results MUST be filterable by a user-selected date range.
- **FR-020**: Search results MUST display article title, source name, publish date, and a brief excerpt.
- **FR-021**: Search results MUST be sorted with the most recent articles first among equal-relevance matches.
- **FR-022**: Clicking a search result MUST open the article in the in-app reader (FR-009).

#### KQL Generator

> **⚠ Gen AI Requirement — Action Required**: FR-023 through FR-028 require an external LLM service.
> The implementation plan MUST document the LLM service integration point and list compatible providers
> (e.g., Azure OpenAI, OpenAI API, or any OpenAI-compatible endpoint) so the user can select and
> configure their preferred service before this feature can function.

- **FR-023**: The reader view MUST provide a "Generate KQL Queries" action on every open article.
- **FR-024**: When triggered, the app MUST send the article's content to a user-configured LLM service and receive KQL query suggestions in response.
- **FR-025**: The KQL generator MUST display returned queries in a panel within the reader view.
- **FR-026**: Each displayed KQL query MUST be accompanied by a brief plain-language description of what threat behavior or indicator it detects.
- **FR-027**: Each displayed KQL query MUST include a one-click "Copy to Clipboard" action.
- **FR-028**: Generated KQL queries MUST be formatted for use in Microsoft Sentinel and/or Microsoft Defender XDR.
- **FR-029**: If no LLM service is configured, the app MUST prompt the user to configure a service endpoint and API key in Settings before attempting generation.
- **FR-030**: If the LLM service call fails or times out, the app MUST display an explanatory error message and offer a retry option.

#### Platform & Settings

- **FR-031**: The app MUST run natively on macOS (Apple Silicon and Intel) and Windows 10/11.
- **FR-032**: The app MUST provide a Settings screen where the user can configure: LLM service endpoint, API key, and feed refresh interval. The LLM API key MUST be stored in the OS keychain (macOS Keychain on macOS; Windows Credential Manager on Windows) and MUST NOT be written to any plaintext file.
- **FR-033**: The app MUST ship with a default curated list of reputable cybersecurity news sources pre-configured and active. The default list MUST include the Microsoft Security Blog Threat Intelligence topic feed (`https://www.microsoft.com/en-us/security/blog/topic/threat-intelligence/feed/`) tagged with the Threat Intelligence category.
- **FR-034**: Users MUST be able to disable or remove any source from the default curated list.
- **FR-035**: Users MUST be able to add custom news sources by providing an RSS/Atom feed URL.
- **FR-036**: When a user adds a custom source, the app MUST validate that the URL is reachable and returns a parseable feed before saving it, and MUST display an error if validation fails.
- **FR-037**: During feed ingestion, the app MUST deduplicate articles by exact URL — if the same article URL arrives from more than one source, only one copy MUST be stored and displayed.
- **FR-038**: The app MUST automatically evict cached article content that is older than the configured cache retention period (default: 30 days). The retention period MUST be user-configurable in Settings. Additionally, the app MUST enforce a maximum size cap on the article body cache (default: 20 GB). When the cap is reached, the app MUST purge cached article bodies in oldest-first order until the cache size is below the cap. Both the time-based eviction and size-based cap operate simultaneously; whichever condition triggers first causes eviction. The size cap MUST be user-configurable in Settings.
- **FR-039**: The app MUST first attempt article content extraction via HTTP fetch and HTML parsing. If the extracted body is empty or unparseable (e.g., the page requires JavaScript rendering), the app MUST fall back to displaying the article in an embedded WebView with ad-blocking rules applied. The WebView fallback MUST NOT require the user to take any action to trigger it.

#### Source Browsing

- **FR-040**: Every article card displayed on the dashboard, in search results, and in the reader view header MUST render the source name as a tappable link. Tapping the source name MUST open a filtered feed view showing only articles from that source.
- **FR-041**: The app MUST provide a dedicated Sources browser page accessible from the sidebar or main navigation. The Sources page MUST list all active news sources. Tapping a source on this page MUST open the same filtered source feed view described in FR-040.
- **FR-042**: The filtered source feed view MUST display articles from the selected source sorted by publish date (newest first) and MUST provide a navigation control to return to the previous context.

#### Accessibility & Readability

- **FR-043**: The reader view MUST provide a font size control allowing the user to increase or decrease the article body text size. The selected font size MUST persist across sessions.
- **FR-044**: The reader view MUST allow the user to tap or click any inline image to open it in a full-window lightbox view that scales the image to fill the available app window. The lightbox MUST provide a dismiss control to return to the article reader view.
- **FR-045**: The lightbox image view MUST support pinch-to-zoom (on trackpad) and scroll-to-pan interactions when the image is zoomed beyond the window bounds.

### Key Entities

- **Article**: Represents a single news item. Key attributes: title, source name, source URL, article URL, publish date, body text, extracted images, category tags (today/microsoft/threat-intel), cached view.
- **NewsSource**: Represents an aggregated feed origin. Key attributes: name, feed URL, feed type (RSS/Atom/API), default category tag (General / Microsoft / Threat Intelligence), active/inactive status.
- **SearchResult**: Represents a matched article from a search query. Attributes: article reference, matched keywords, excerpt, relevance rank.
- **KQLSuggestion**: Represents a generated KQL query. Key attributes: article reference, query text, detection description, generation timestamp, LLM service used.
- **AppSettings**: Stores user-configurable preferences. Attributes: LLM endpoint, API key (stored in OS keychain — macOS Keychain / Windows Credential Manager), feed background-resume refresh threshold (default: 4 hours), cache retention period (default: 30 days), article body cache size cap (default: 20 GB), reader font size preference.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The dashboard loads and displays at least one article per populated section within 10 seconds of app launch on a standard broadband connection.
- **SC-002**: The dashboard feed refresh completes in under 30 seconds for a typical source list.
- **SC-003**: The in-app reader renders a full article (text and images) within 5 seconds of the user clicking the headline.
- **SC-004**: Keyword search returns results within 3 seconds of the user submitting the query.
- **SC-005**: The KQL generator returns at least one query suggestion per article that contains identifiable threat content (named threat actors, CVEs, malware families, or explicit TTPs).
- **SC-006**: The app launches and runs without errors on both macOS and Windows with no platform-specific workarounds required by the user.
- **SC-007**: The reader view displays zero visible advertisement elements for articles from the top 10 most frequently aggregated sources.
- **SC-008**: Previously cached articles remain accessible in the reader view when the device has no internet connection.
