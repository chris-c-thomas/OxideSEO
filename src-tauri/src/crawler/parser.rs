//! HTML parser: streaming extraction via lol_html with scraper fallback.
//!
//! Uses lol_html for the hot path (constant memory, streaming parse).
//! Falls back to scraper (full DOM) when lol_html fails on malformed HTML.

use anyhow::Result;
use url::Url;

use crate::crawler::ParsedPage;

/// Parse an HTML document and extract all SEO-relevant data.
///
/// # Arguments
/// - `html_bytes`: Raw HTML response body.
/// - `page_url`: The URL this HTML was fetched from (for resolving relative URLs).
/// - `root_domain`: The crawl's root domain (for classifying internal vs external links).
pub fn parse_html(html_bytes: &[u8], page_url: &str, root_domain: &str) -> ParsedPage {
    match parse_with_lol_html(html_bytes, page_url, root_domain) {
        Ok(page) => page,
        Err(e) => {
            tracing::warn!(
                url = %page_url,
                error = %e,
                "lol_html parse failed, falling back to scraper"
            );
            parse_with_scraper(html_bytes, page_url, root_domain).unwrap_or_else(|_| ParsedPage {
                url: page_url.to_string(),
                parse_ok: false,
                ..Default::default()
            })
        }
    }
}

/// Primary parser: lol_html streaming parse.
///
/// Registers element content handlers for all SEO-relevant tags:
/// - `<title>`, `<meta>`, `<link rel="canonical">`, `<base>`
/// - `<h1>` through `<h6>`
/// - `<a>`, `<img>`, `<script>`, `<link rel="stylesheet">`
fn parse_with_lol_html(
    _html_bytes: &[u8],
    page_url: &str,
    _root_domain: &str,
) -> Result<ParsedPage> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let page = Rc::new(RefCell::new(ParsedPage {
        url: page_url.to_string(),
        parse_ok: true,
        ..Default::default()
    }));

    // TODO(phase-2): Implement full lol_html extraction.
    //
    // Register element content handlers for:
    //
    // 1. <title> — capture text content
    // 2. <meta name="description"> — capture content attribute
    // 3. <meta name="robots"> — capture content attribute
    // 4. <meta name="viewport"> — capture content attribute
    // 5. <link rel="canonical"> — capture href attribute
    // 6. <base href="..."> — capture for URL resolution
    // 7. <h1> through <h6> — capture text content
    // 8. <a href="..."> — extract href, rel, anchor text; resolve relative;
    //    classify internal vs external
    // 9. <img src="..."> — extract src, alt, check srcset
    // 10. <script src="..."> — external script URLs
    // 11. <link rel="stylesheet"> — external stylesheet URLs
    //
    // After parse: compute word count by stripping tags and counting
    // whitespace-delimited tokens.

    let result = page.borrow().clone();
    Ok(result)
}

/// Fallback parser: scraper full DOM parse.
fn parse_with_scraper(html_bytes: &[u8], page_url: &str, _root_domain: &str) -> Result<ParsedPage> {
    let html_str = String::from_utf8_lossy(html_bytes);
    let _document = scraper::Html::parse_document(&html_str);

    let page = ParsedPage {
        url: page_url.to_string(),
        parse_ok: true,
        ..Default::default()
    };

    // TODO(phase-2): Implement scraper-based extraction as fallback.
    // Same data points as lol_html, but using CSS selectors.

    Ok(page)
}

/// Resolve a potentially relative URL against a base URL.
pub fn resolve_url(href: &str, base: &str) -> Option<String> {
    match Url::parse(href) {
        Ok(absolute) => Some(absolute.to_string()),
        Err(url::ParseError::RelativeUrlWithoutBase) => Url::parse(base)
            .ok()
            .and_then(|b| b.join(href).ok())
            .map(|u| u.to_string()),
        Err(_) => None,
    }
}

/// Check if a URL belongs to the same root domain as the crawl.
pub fn is_internal(url: &str, root_domain: &str) -> bool {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .map(|host| host == root_domain || host.ends_with(&format!(".{}", root_domain)))
        .unwrap_or(false)
}

/// Count words in text content (tags stripped).
pub fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_url() {
        let result = resolve_url("https://example.com/page", "https://example.com/");
        assert_eq!(result, Some("https://example.com/page".into()));
    }

    #[test]
    fn test_resolve_relative_url() {
        let result = resolve_url("/about", "https://example.com/page");
        assert_eq!(result, Some("https://example.com/about".into()));
    }

    #[test]
    fn test_is_internal_same_domain() {
        assert!(is_internal("https://example.com/page", "example.com"));
    }

    #[test]
    fn test_is_internal_subdomain() {
        assert!(is_internal("https://blog.example.com/post", "example.com"));
    }

    #[test]
    fn test_is_internal_external() {
        assert!(!is_internal("https://other.com/page", "example.com"));
    }

    #[test]
    fn test_word_count() {
        assert_eq!(count_words("hello world foo bar"), 4);
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("   spaced   out   "), 2);
    }
}
