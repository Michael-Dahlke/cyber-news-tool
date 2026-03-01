use readability::extractor;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;

/// Result of content extraction attempt.
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub body_html: String,
    pub body_text: String,
    pub image_urls: Vec<String>,
    pub method: ExtractionMethodUsed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtractionMethodUsed {
    Readability,
    Scraper,
}

impl ExtractionMethodUsed {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExtractionMethodUsed::Readability => "readability",
            ExtractionMethodUsed::Scraper => "scraper",
        }
    }
}

/// Extract article content. Tries readability first, then scraper fallback.
pub async fn extract_content(url: &str) -> Result<ExtractionResult, String> {
    // Try readability extraction first
    match extract_via_readability(url).await {
        Ok(result) if !result.body_html.trim().is_empty() => return Ok(result),
        Ok(_) => log::info!("Readability returned empty content for {url}, trying scraper"),
        Err(e) => log::info!("Readability failed for {url}: {e}, trying scraper"),
    }

    // Fallback to scraper-based extraction
    match extract_via_scraper(url).await {
        Ok(result) if !result.body_html.trim().is_empty() => Ok(result),
        Ok(_) => Err("Both extraction methods returned empty content".to_string()),
        Err(e) => Err(format!("All extraction methods failed: {e}")),
    }
}

/// Primary extraction using the readability crate.
async fn extract_via_readability(url: &str) -> Result<ExtractionResult, String> {
    let url_owned = url.to_string();

    // readability::extractor::scrape is blocking, so spawn on blocking thread
    let product = tokio::task::spawn_blocking(move || {
        extractor::scrape(&url_owned)
    })
    .await
    .map_err(|e| format!("Readability task failed: {e}"))?
    .map_err(|e| format!("Readability extraction error: {e}"))?;

    let body_html = product.content.clone();
    // Sanitize extracted HTML to prevent stored XSS
    let body_html = sanitize_html(&body_html);
    let body_text = strip_html_tags(&product.content);
    let image_urls = extract_image_urls_from_html(&product.content);

    Ok(ExtractionResult {
        body_html,
        body_text,
        image_urls,
        method: ExtractionMethodUsed::Readability,
    })
}

/// Fallback extraction using scraper with CSS selectors for common article structures.
async fn extract_via_scraper(url: &str) -> Result<ExtractionResult, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("CyberNewsReader/0.1")
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch {url}: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {} for {url}", response.status()));
    }

    let html_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    let document = Html::parse_document(&html_text);

    // Try common article content selectors in order of specificity
    let content_selectors = [
        "article .entry-content",
        "article .post-content",
        "article .article-body",
        ".post-body",
        ".entry-content",
        ".article-content",
        ".story-body",
        "article",
        "[role=\"main\"] .content",
        "main .content",
        "main",
    ];

    let mut body_html = String::new();

    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                body_html = element.inner_html();
                if !body_html.trim().is_empty() {
                    break;
                }
            }
        }
    }

    if body_html.trim().is_empty() {
        return Err("No article content found via CSS selectors".to_string());
    }

    // Clean up the scraped HTML - remove script/style tags, then sanitize
    let body_html = clean_html(&body_html);
    let body_html = sanitize_html(&body_html);
    let body_text = strip_html_tags(&body_html);
    let image_urls = extract_image_urls_from_html(&body_html);

    Ok(ExtractionResult {
        body_html,
        body_text,
        image_urls,
        method: ExtractionMethodUsed::Scraper,
    })
}

/// Extract image URLs from HTML content.
fn extract_image_urls_from_html(html: &str) -> Vec<String> {
    let document = Html::parse_fragment(html);
    let img_selector = Selector::parse("img[src]").unwrap_or_else(|_| {
        Selector::parse("img").expect("img selector should parse")
    });

    document
        .select(&img_selector)
        .filter_map(|el| el.value().attr("src").map(String::from))
        .filter(|src| {
            // Filter out tracking pixels and tiny images
            !src.contains("pixel") && !src.contains("1x1") && !src.contains("tracking")
        })
        .collect()
}

/// Remove HTML tags to produce plain text.
fn strip_html_tags(html: &str) -> String {
    let document = Html::parse_fragment(html);
    document
        .root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Clean HTML by removing script and style tags.
fn clean_html(html: &str) -> String {
    let document = Html::parse_fragment(html);

    let script_selector = Selector::parse("script").unwrap();
    let style_selector = Selector::parse("style").unwrap();

    let mut output = html.to_string();

    // Remove script elements
    for element in document.select(&script_selector) {
        let script_html = element.html();
        output = output.replace(&script_html, "");
    }

    // Remove style elements
    for element in document.select(&style_selector) {
        let style_html = element.html();
        output = output.replace(&style_html, "");
    }

    output
}

/// Sanitize HTML using an allowlist approach to prevent XSS.
/// Permits safe structural/formatting tags and strips everything else
/// (event handlers, script, iframe, object, embed, form, svg, etc.).
pub fn sanitize_html(html: &str) -> String {
    use std::collections::HashSet;

    let tags: HashSet<&str> = [
        "h1", "h2", "h3", "h4", "h5", "h6",
        "p", "br", "hr",
        "ul", "ol", "li",
        "blockquote", "pre", "code",
        "a", "img",
        "strong", "b", "em", "i", "u", "s", "del", "ins",
        "table", "thead", "tbody", "tr", "th", "td",
        "figure", "figcaption", "picture", "source",
        "div", "span", "section", "article", "aside",
        "mark", "sub", "sup", "abbr", "time", "details", "summary",
    ]
    .iter()
    .copied()
    .collect();

    let mut tag_attrs = std::collections::HashMap::new();
    tag_attrs.insert("a", ["href", "title"].iter().copied().collect::<HashSet<&str>>());
    tag_attrs.insert("img", ["src", "alt", "title", "loading", "width", "height"].iter().copied().collect());
    tag_attrs.insert("td", ["colspan", "rowspan"].iter().copied().collect());
    tag_attrs.insert("th", ["colspan", "rowspan"].iter().copied().collect());
    tag_attrs.insert("time", ["datetime"].iter().copied().collect());
    tag_attrs.insert("source", ["srcset", "type", "media"].iter().copied().collect());
    tag_attrs.insert("abbr", ["title"].iter().copied().collect());

    ammonia::Builder::new()
        .tags(tags)
        .link_rel(Some("noopener noreferrer"))
        .url_relative(ammonia::UrlRelative::Deny)
        .tag_attributes(tag_attrs)
        .clean(html)
        .to_string()
}

/// Sanitize search excerpts — only allow <mark> tags (used by FTS5 snippet).
pub fn sanitize_search_excerpt(excerpt: &str) -> String {
    let tags: std::collections::HashSet<&str> = ["mark"].iter().copied().collect();
    ammonia::Builder::new()
        .tags(tags)
        .clean(excerpt)
        .to_string()
}
