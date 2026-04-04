//! HTML parser: streaming extraction via lol_html with scraper fallback.
//!
//! Uses lol_html for the hot path (constant memory, streaming parse).
//! Falls back to scraper (full DOM) when lol_html fails on malformed HTML.

use anyhow::Result;
use url::Url;

use crate::crawler::{ExtractedImage, ExtractedLink, ParsedPage};

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
fn parse_with_lol_html(html_bytes: &[u8], page_url: &str, root_domain: &str) -> Result<ParsedPage> {
    use lol_html::{HtmlRewriter, Settings, element, text};
    use std::cell::RefCell;
    use std::rc::Rc;

    // --- Accumulators shared between handlers via Rc<RefCell<>> ---

    let title = Rc::new(RefCell::new(String::new()));
    let meta_description: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let meta_robots: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let viewport: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let canonical: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let base_url: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let h1s: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let h2s: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let h3s: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let h4s: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let h5s: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let h6s: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    // Heading text accumulator: (level, text_so_far)
    let heading_buf: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    let links: Rc<RefCell<Vec<ExtractedLink>>> = Rc::new(RefCell::new(Vec::new()));
    // Anchor text accumulator for current <a> tag
    let anchor_buf: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    // Current anchor metadata: (href, rel)
    #[allow(clippy::type_complexity)]
    let anchor_meta: Rc<RefCell<Option<(String, Option<String>)>>> = Rc::new(RefCell::new(None));

    let images: Rc<RefCell<Vec<ExtractedImage>>> = Rc::new(RefCell::new(Vec::new()));
    let scripts: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let stylesheets: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let body_text: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));

    // Capture copies for closures
    let pg_url = page_url.to_string();
    let rd = root_domain.to_string();

    // --- Build handler macro for each heading level ---
    macro_rules! heading_handlers {
        ($sel:expr, $vec:expr) => {{
            let buf = heading_buf.clone();
            let vec = $vec.clone();
            vec![
                element!($sel, move |_el| {
                    buf.borrow_mut().clear();
                    Ok(())
                }),
                text!($sel, {
                    let buf2 = heading_buf.clone();
                    let vec2 = vec.clone();
                    move |chunk| {
                        buf2.borrow_mut().push_str(chunk.as_str());
                        if chunk.last_in_text_node() {
                            let text = buf2.borrow().trim().to_string();
                            if !text.is_empty() {
                                vec2.borrow_mut().push(text);
                            }
                            buf2.borrow_mut().clear();
                        }
                        Ok(())
                    }
                }),
            ]
        }};
    }

    // --- Build element_content_handlers ---
    let mut handlers = Vec::new();

    // <title> — accumulate text
    {
        let t = title.clone();
        handlers.push(text!("title", move |chunk| {
            t.borrow_mut().push_str(chunk.as_str());
            Ok(())
        }));
    }

    // <meta name="description">
    {
        let md = meta_description.clone();
        handlers.push(element!("meta", move |el| {
            if let Some(name) = el.get_attribute("name") {
                if name.eq_ignore_ascii_case("description") {
                    *md.borrow_mut() = el.get_attribute("content");
                }
            }
            Ok(())
        }));
    }

    // <meta name="robots"> and <meta name="viewport"> — separate handler to avoid borrow conflicts
    {
        let mr = meta_robots.clone();
        let vp = viewport.clone();
        handlers.push(element!("meta", move |el| {
            if let Some(name) = el.get_attribute("name") {
                match name.to_lowercase().as_str() {
                    "robots" => {
                        *mr.borrow_mut() = el.get_attribute("content");
                    }
                    "viewport" => {
                        *vp.borrow_mut() = el.get_attribute("content");
                    }
                    _ => {}
                }
            }
            Ok(())
        }));
    }

    // <link rel="canonical">
    {
        let can = canonical.clone();
        let base = base_url.clone();
        let url = pg_url.clone();
        handlers.push(element!("link", move |el| {
            if let Some(rel) = el.get_attribute("rel") {
                let rel_lower = rel.to_lowercase();
                if rel_lower == "canonical" {
                    if let Some(href) = el.get_attribute("href") {
                        let resolved_base = base.borrow().clone().unwrap_or_else(|| url.clone());
                        *can.borrow_mut() = resolve_url(&href, &resolved_base);
                    }
                }
            }
            Ok(())
        }));
    }

    // <link rel="stylesheet">
    {
        let ss = stylesheets.clone();
        let base = base_url.clone();
        let url = pg_url.clone();
        handlers.push(element!("link", move |el| {
            if let Some(rel) = el.get_attribute("rel") {
                if rel.to_lowercase().contains("stylesheet") {
                    if let Some(href) = el.get_attribute("href") {
                        let resolved_base = base.borrow().clone().unwrap_or_else(|| url.clone());
                        if let Some(resolved) = resolve_url(&href, &resolved_base) {
                            ss.borrow_mut().push(resolved);
                        }
                    }
                }
            }
            Ok(())
        }));
    }

    // <base href="...">
    {
        let bu = base_url.clone();
        handlers.push(element!("base", move |el| {
            if let Some(href) = el.get_attribute("href") {
                *bu.borrow_mut() = Some(href);
            }
            Ok(())
        }));
    }

    // Headings h1-h6
    handlers.extend(heading_handlers!("h1", h1s));
    handlers.extend(heading_handlers!("h2", h2s));
    handlers.extend(heading_handlers!("h3", h3s));
    handlers.extend(heading_handlers!("h4", h4s));
    handlers.extend(heading_handlers!("h5", h5s));
    handlers.extend(heading_handlers!("h6", h6s));

    // <a href="..."> — element handler captures href/rel, text handler captures anchor text
    {
        let am = anchor_meta.clone();
        let ab = anchor_buf.clone();
        handlers.push(element!("a[href]", move |el| {
            ab.borrow_mut().clear();
            let href = el.get_attribute("href").unwrap_or_default();
            let rel = el.get_attribute("rel");
            *am.borrow_mut() = Some((href, rel));
            Ok(())
        }));
    }
    {
        let am = anchor_meta.clone();
        let ab = anchor_buf.clone();
        let lnks = links.clone();
        let base = base_url.clone();
        let url = pg_url.clone();
        let rd2 = rd.clone();
        handlers.push(text!("a[href]", move |chunk| {
            ab.borrow_mut().push_str(chunk.as_str());
            if chunk.last_in_text_node() {
                if let Some((href, rel)) = am.borrow_mut().take() {
                    let anchor_text = ab.borrow().trim().to_string();
                    let resolved_base = base.borrow().clone().unwrap_or_else(|| url.clone());
                    if let Some(resolved) = resolve_url(&href, &resolved_base) {
                        let is_nofollow = rel
                            .as_deref()
                            .map(|r| r.to_lowercase().contains("nofollow"))
                            .unwrap_or(false);
                        let internal = is_internal(&resolved, &rd2);
                        lnks.borrow_mut().push(ExtractedLink {
                            href: resolved,
                            anchor_text: if anchor_text.is_empty() {
                                None
                            } else {
                                Some(anchor_text)
                            },
                            rel,
                            is_internal: internal,
                            is_nofollow,
                        });
                    }
                    ab.borrow_mut().clear();
                }
            }
            Ok(())
        }));
    }

    // <img> — capture src, alt, srcset
    {
        let imgs = images.clone();
        let base = base_url.clone();
        let url = pg_url.clone();
        handlers.push(element!("img", move |el| {
            if let Some(src) = el.get_attribute("src") {
                let resolved_base = base.borrow().clone().unwrap_or_else(|| url.clone());
                let resolved_src = resolve_url(&src, &resolved_base).unwrap_or_else(|| src.clone());
                imgs.borrow_mut().push(ExtractedImage {
                    src: resolved_src,
                    alt: el.get_attribute("alt"),
                    has_srcset: el.get_attribute("srcset").is_some(),
                });
            }
            Ok(())
        }));
    }

    // <script src="...">
    {
        let scr = scripts.clone();
        let base = base_url.clone();
        let url = pg_url.clone();
        handlers.push(element!("script[src]", move |el| {
            if let Some(src) = el.get_attribute("src") {
                let resolved_base = base.borrow().clone().unwrap_or_else(|| url.clone());
                if let Some(resolved) = resolve_url(&src, &resolved_base) {
                    scr.borrow_mut().push(resolved);
                }
            }
            Ok(())
        }));
    }

    // Body text — accumulate all visible text for word count
    {
        let bt = body_text.clone();
        handlers.push(text!("body", move |chunk| {
            bt.borrow_mut().push_str(chunk.as_str());
            Ok(())
        }));
    }

    // --- Run the rewriter ---
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: handlers,
            ..Settings::new()
        },
        |_c: &[u8]| {}, // Discard output — we only extract
    );

    rewriter.write(html_bytes)?;
    rewriter.end()?;

    // --- Build result ---
    let title_text = title.borrow().trim().to_string();
    let base_resolved = base_url.borrow().clone();

    Ok(ParsedPage {
        url: page_url.to_string(),
        title: if title_text.is_empty() {
            None
        } else {
            Some(title_text)
        },
        meta_description: meta_description.borrow().clone(),
        meta_robots: meta_robots.borrow().clone(),
        canonical: canonical.borrow().clone(),
        viewport: viewport.borrow().clone(),
        h1s: h1s.borrow().clone(),
        h2s: h2s.borrow().clone(),
        h3s: h3s.borrow().clone(),
        h4s: h4s.borrow().clone(),
        h5s: h5s.borrow().clone(),
        h6s: h6s.borrow().clone(),
        links: links.borrow().clone(),
        images: images.borrow().clone(),
        scripts: scripts.borrow().clone(),
        stylesheets: stylesheets.borrow().clone(),
        word_count: count_words(&body_text.borrow()),
        base_url: base_resolved,
        parse_ok: true,
        body_size: None,
        response_time_ms: None,
    })
}

/// Fallback parser: scraper full DOM parse.
fn parse_with_scraper(html_bytes: &[u8], page_url: &str, root_domain: &str) -> Result<ParsedPage> {
    use scraper::{Html, Selector};

    let html_str = String::from_utf8_lossy(html_bytes);
    let document = Html::parse_document(&html_str);

    fn sel(s: &str) -> Selector {
        Selector::parse(s).unwrap()
    }

    let title_sel = sel("title");
    let title = document
        .select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .filter(|t| !t.is_empty());

    let meta_description = {
        let s = sel("meta[name='description']");
        document
            .select(&s)
            .next()
            .and_then(|el| el.value().attr("content").map(|v| v.to_string()))
    };
    let meta_robots = {
        let s = sel("meta[name='robots']");
        document
            .select(&s)
            .next()
            .and_then(|el| el.value().attr("content").map(|v| v.to_string()))
    };
    let viewport = {
        let s = sel("meta[name='viewport']");
        document
            .select(&s)
            .next()
            .and_then(|el| el.value().attr("content").map(|v| v.to_string()))
    };

    let canonical_sel = sel("link[rel='canonical']");
    let canonical = document
        .select(&canonical_sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .and_then(|href| resolve_url(href, page_url));

    let base_sel = sel("base[href]");
    let base_url = document
        .select(&base_sel)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|s| s.to_string());

    let resolve_base = base_url.as_deref().unwrap_or(page_url);

    fn extract_headings(document: &Html, tag: &str) -> Vec<String> {
        let s = Selector::parse(tag).unwrap();
        document
            .select(&s)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    }

    let a_sel = sel("a[href]");
    let links: Vec<ExtractedLink> = document
        .select(&a_sel)
        .filter_map(|el| {
            let href = el.value().attr("href")?;
            let resolved = resolve_url(href, resolve_base)?;
            let rel = el.value().attr("rel").map(|s| s.to_string());
            let is_nofollow = rel
                .as_deref()
                .map(|r| r.to_lowercase().contains("nofollow"))
                .unwrap_or(false);
            let anchor_text = el.text().collect::<String>().trim().to_string();
            Some(ExtractedLink {
                href: resolved.clone(),
                anchor_text: if anchor_text.is_empty() {
                    None
                } else {
                    Some(anchor_text)
                },
                rel,
                is_internal: is_internal(&resolved, root_domain),
                is_nofollow,
            })
        })
        .collect();

    let img_sel = sel("img");
    let images: Vec<ExtractedImage> = document
        .select(&img_sel)
        .filter_map(|el| {
            let src = el.value().attr("src")?;
            let resolved = resolve_url(src, resolve_base).unwrap_or_else(|| src.to_string());
            Some(ExtractedImage {
                src: resolved,
                alt: el.value().attr("alt").map(|s| s.to_string()),
                has_srcset: el.value().attr("srcset").is_some(),
            })
        })
        .collect();

    let script_sel = sel("script[src]");
    let scripts: Vec<String> = document
        .select(&script_sel)
        .filter_map(|el| {
            let src = el.value().attr("src")?;
            resolve_url(src, resolve_base)
        })
        .collect();

    let ss_sel = sel("link[rel='stylesheet']");
    let stylesheets: Vec<String> = document
        .select(&ss_sel)
        .filter_map(|el| {
            let href = el.value().attr("href")?;
            resolve_url(href, resolve_base)
        })
        .collect();

    let body_sel = sel("body");
    let body_text: String = document
        .select(&body_sel)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();

    Ok(ParsedPage {
        url: page_url.to_string(),
        title,
        meta_description,
        meta_robots,
        canonical,
        viewport,
        h1s: extract_headings(&document, "h1"),
        h2s: extract_headings(&document, "h2"),
        h3s: extract_headings(&document, "h3"),
        h4s: extract_headings(&document, "h4"),
        h5s: extract_headings(&document, "h5"),
        h6s: extract_headings(&document, "h6"),
        links,
        images,
        scripts,
        stylesheets,
        word_count: count_words(&body_text),
        base_url,
        parse_ok: true,
        body_size: None,
        response_time_ms: None,
    })
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

    // --- lol_html parser tests ---

    #[test]
    fn test_parse_fixture_lol_html() {
        let html = include_bytes!("../../../tests/fixtures/index.html");
        let page = parse_with_lol_html(html, "https://test.local/", "test.local").unwrap();

        assert_eq!(page.title.as_deref(), Some("Test Page - Home"));
        assert_eq!(
            page.meta_description.as_deref(),
            Some("This is the test homepage used for integration testing the OxideSEO crawler.")
        );
        assert_eq!(page.meta_robots.as_deref(), Some("index, follow"));
        assert_eq!(
            page.viewport.as_deref(),
            Some("width=device-width, initial-scale=1.0")
        );
        assert_eq!(page.canonical.as_deref(), Some("https://test.local/"));

        assert_eq!(page.h1s.len(), 1);
        assert_eq!(page.h1s[0], "Welcome to the Test Site");
        assert_eq!(page.h2s.len(), 2);
        assert_eq!(page.h3s.len(), 1);

        assert_eq!(page.links.len(), 7);
        let internal_count = page.links.iter().filter(|l| l.is_internal).count();
        assert_eq!(internal_count, 6);
        let nofollow_count = page.links.iter().filter(|l| l.is_nofollow).count();
        assert_eq!(nofollow_count, 1);

        // Check anchor text extraction
        let about_link = page
            .links
            .iter()
            .find(|l| l.href.contains("/about"))
            .unwrap();
        assert_eq!(about_link.anchor_text.as_deref(), Some("About Us"));

        assert_eq!(page.images.len(), 3);
        let with_alt = page.images.iter().filter(|i| i.alt.is_some()).count();
        assert_eq!(with_alt, 2); // one real alt, one empty alt=""

        assert_eq!(page.scripts.len(), 1);
        assert_eq!(page.stylesheets.len(), 1);
        assert!(page.word_count > 0);
        assert!(page.parse_ok);
    }

    // --- scraper fallback parser tests ---

    #[test]
    fn test_parse_fixture_scraper() {
        let html = include_bytes!("../../../tests/fixtures/index.html");
        let page = parse_with_scraper(html, "https://test.local/", "test.local").unwrap();

        assert_eq!(page.title.as_deref(), Some("Test Page - Home"));
        assert_eq!(page.meta_robots.as_deref(), Some("index, follow"));
        assert_eq!(page.canonical.as_deref(), Some("https://test.local/"));

        assert_eq!(page.h1s.len(), 1);
        assert_eq!(page.h2s.len(), 2);
        assert_eq!(page.links.len(), 7);
        assert_eq!(page.images.len(), 3);
        assert!(page.word_count > 0);
        assert!(page.parse_ok);
    }

    // --- Edge case tests ---

    #[test]
    fn test_parse_empty_html() {
        let page = parse_html(b"", "https://example.com/", "example.com");
        assert!(page.title.is_none());
        assert!(page.links.is_empty());
        assert!(page.parse_ok);
    }

    #[test]
    fn test_parse_minimal_html() {
        let html = b"<html><head><title>Hello</title></head><body><p>World</p></body></html>";
        let page = parse_html(html, "https://example.com/", "example.com");
        assert_eq!(page.title.as_deref(), Some("Hello"));
        assert_eq!(page.word_count, 1);
    }

    #[test]
    fn test_parse_missing_tags() {
        let html = b"<html><body><p>No head section at all</p></body></html>";
        let page = parse_html(html, "https://example.com/", "example.com");
        assert!(page.title.is_none());
        assert!(page.meta_description.is_none());
        assert!(page.canonical.is_none());
    }

    #[test]
    fn test_parse_base_url_resolution() {
        let html = br#"<html><head><base href="https://cdn.example.com/"></head>
        <body><a href="/page">Link</a></body></html>"#;
        let page = parse_html(html, "https://example.com/", "example.com");
        assert_eq!(page.base_url.as_deref(), Some("https://cdn.example.com/"));
        // Link should resolve against base URL
        assert!(!page.links.is_empty());
        assert!(page.links[0].href.contains("cdn.example.com"));
    }

    #[test]
    fn test_parse_public_api() {
        let html = include_bytes!("../../../tests/fixtures/index.html");
        let page = parse_html(html, "https://test.local/", "test.local");
        // Ensure the public API delegates correctly
        assert_eq!(page.title.as_deref(), Some("Test Page - Home"));
        assert!(page.parse_ok);
    }
}
