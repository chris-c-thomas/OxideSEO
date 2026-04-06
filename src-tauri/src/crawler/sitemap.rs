//! Sitemap auto-discovery, parsing, and recursive fetching.
//!
//! Supports XML sitemaps, sitemap indexes, and gzip-compressed sitemaps.
//! Uses `quick-xml` for streaming XML parsing and `flate2` for decompression.

use std::io::Read as _;

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// A single entry from a `<urlset>` sitemap.
#[derive(Debug, Clone)]
pub struct SitemapEntry {
    pub url: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f64>,
}

/// Result of parsing a sitemap — either a list of URLs or a list of child sitemaps.
#[derive(Debug)]
pub enum SitemapContent {
    /// A `<sitemapindex>` containing references to other sitemaps.
    Index(Vec<String>),
    /// A `<urlset>` containing page entries.
    UrlSet(Vec<SitemapEntry>),
}

/// Discover sitemap URLs from multiple sources.
///
/// Checks the robots.txt `Sitemap:` directives plus well-known paths
/// (`/sitemap.xml`, `/sitemap_index.xml`).
pub async fn discover_sitemaps(
    domain: &str,
    scheme: &str,
    client: &reqwest::Client,
    robots_sitemaps: &[String],
) -> Vec<String> {
    let mut urls: Vec<String> = robots_sitemaps.to_vec();

    // Add well-known sitemap paths if not already in the list.
    let well_known = [
        format!("{}://{}/sitemap.xml", scheme, domain),
        format!("{}://{}/sitemap_index.xml", scheme, domain),
    ];

    for wk in &well_known {
        if !urls.iter().any(|u| u == wk) {
            urls.push(wk.clone());
        }
    }

    // Verify each URL exists with a HEAD request.
    let mut valid = Vec::new();
    for url in &urls {
        match client.head(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                valid.push(url.clone());
            }
            _ => {
                tracing::debug!(url = %url, "Sitemap URL not found or inaccessible");
            }
        }
    }

    valid
}

/// Parse sitemap XML bytes. Handles both `<urlset>` and `<sitemapindex>`.
///
/// If `is_gzipped` is true, decompresses the bytes first.
pub fn parse_sitemap(bytes: &[u8], is_gzipped: bool) -> Result<SitemapContent> {
    let data = if is_gzipped {
        let mut decoder = flate2::read::GzDecoder::new(bytes);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .context("Failed to decompress gzipped sitemap")?;
        decompressed
    } else {
        bytes.to_vec()
    };

    let mut reader = Reader::from_reader(data.as_slice());
    reader.config_mut().trim_text(true);

    // Peek at the root element to determine type.
    let mut buf = Vec::new();
    let mut is_index = false;
    let mut entries: Vec<SitemapEntry> = Vec::new();
    let mut index_urls: Vec<String> = Vec::new();

    // Current parsing state.
    let mut in_url = false;
    let mut in_sitemap = false;
    let mut current_loc: Option<String> = None;
    let mut current_lastmod: Option<String> = None;
    let mut current_changefreq: Option<String> = None;
    let mut current_priority: Option<f64> = None;
    let mut current_tag = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                match name {
                    "sitemapindex" => {
                        is_index = true;
                    }
                    "url" => {
                        in_url = true;
                        current_loc = None;
                        current_lastmod = None;
                        current_changefreq = None;
                        current_priority = None;
                    }
                    "sitemap" if is_index => {
                        in_sitemap = true;
                        current_loc = None;
                    }
                    "loc" | "lastmod" | "changefreq" | "priority" => {
                        current_tag = name.to_string();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }
                if in_url {
                    match current_tag.as_str() {
                        "loc" => current_loc = Some(text),
                        "lastmod" => current_lastmod = Some(text),
                        "changefreq" => current_changefreq = Some(text),
                        "priority" => current_priority = text.parse().ok(),
                        _ => {}
                    }
                } else if in_sitemap && current_tag == "loc" {
                    current_loc = Some(text);
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                let name = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
                match name {
                    "url" => {
                        if let Some(loc) = current_loc.take() {
                            entries.push(SitemapEntry {
                                url: loc,
                                lastmod: current_lastmod.take(),
                                changefreq: current_changefreq.take(),
                                priority: current_priority.take(),
                            });
                        }
                        in_url = false;
                    }
                    "sitemap" if is_index => {
                        if let Some(loc) = current_loc.take() {
                            index_urls.push(loc);
                        }
                        in_sitemap = false;
                    }
                    "loc" | "lastmod" | "changefreq" | "priority" => {
                        current_tag.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!(error = %e, "Error parsing sitemap XML");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    if is_index {
        Ok(SitemapContent::Index(index_urls))
    } else {
        Ok(SitemapContent::UrlSet(entries))
    }
}

/// Recursively fetch and parse all sitemaps starting from the given URLs.
///
/// Follows sitemap index references up to 3 levels deep to prevent infinite loops.
pub async fn fetch_all_sitemaps(urls: &[String], client: &reqwest::Client) -> Vec<SitemapEntry> {
    let mut all_entries = Vec::new();
    let mut pending: Vec<(String, u8)> = urls.iter().map(|u| (u.clone(), 0)).collect();

    while let Some((url, depth)) = pending.pop() {
        if depth > 3 {
            tracing::warn!(url = %url, "Sitemap index depth exceeded 3, skipping");
            continue;
        }

        let bytes = match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!(url = %url, error = %e, "Failed to read sitemap body");
                    continue;
                }
            },
            Ok(resp) => {
                tracing::debug!(url = %url, status = %resp.status(), "Sitemap fetch non-success");
                continue;
            }
            Err(e) => {
                tracing::warn!(url = %url, error = %e, "Failed to fetch sitemap");
                continue;
            }
        };

        let is_gzipped = url.ends_with(".gz");
        let content = match parse_sitemap(&bytes, is_gzipped) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(url = %url, error = %e, "Failed to parse sitemap");
                continue;
            }
        };

        match content {
            SitemapContent::Index(child_urls) => {
                tracing::info!(url = %url, count = child_urls.len(), "Parsed sitemap index");
                for child in child_urls {
                    pending.push((child, depth + 1));
                }
            }
            SitemapContent::UrlSet(entries) => {
                tracing::info!(url = %url, count = entries.len(), "Parsed sitemap urlset");
                all_entries.extend(entries);
            }
        }
    }

    all_entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITEMAP_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/</loc>
    <lastmod>2026-01-01</lastmod>
    <changefreq>daily</changefreq>
    <priority>1.0</priority>
  </url>
  <url>
    <loc>https://example.com/about</loc>
    <lastmod>2026-02-15</lastmod>
    <priority>0.8</priority>
  </url>
  <url>
    <loc>https://example.com/contact</loc>
  </url>
</urlset>"#;

    const SITEMAP_INDEX_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap>
    <loc>https://example.com/sitemap-pages.xml</loc>
    <lastmod>2026-03-01</lastmod>
  </sitemap>
  <sitemap>
    <loc>https://example.com/sitemap-blog.xml</loc>
  </sitemap>
</sitemapindex>"#;

    #[test]
    fn test_parse_standard_sitemap() {
        let content = parse_sitemap(SITEMAP_XML.as_bytes(), false).unwrap();
        match content {
            SitemapContent::UrlSet(entries) => {
                assert_eq!(entries.len(), 3);
                assert_eq!(entries[0].url, "https://example.com/");
                assert_eq!(entries[0].lastmod.as_deref(), Some("2026-01-01"));
                assert_eq!(entries[0].changefreq.as_deref(), Some("daily"));
                assert_eq!(entries[0].priority, Some(1.0));
                assert_eq!(entries[1].url, "https://example.com/about");
                assert_eq!(entries[1].priority, Some(0.8));
                assert_eq!(entries[2].url, "https://example.com/contact");
                assert!(entries[2].lastmod.is_none());
                assert!(entries[2].priority.is_none());
            }
            SitemapContent::Index(_) => panic!("Expected UrlSet, got Index"),
        }
    }

    #[test]
    fn test_parse_sitemap_index() {
        let content = parse_sitemap(SITEMAP_INDEX_XML.as_bytes(), false).unwrap();
        match content {
            SitemapContent::Index(urls) => {
                assert_eq!(urls.len(), 2);
                assert_eq!(urls[0], "https://example.com/sitemap-pages.xml");
                assert_eq!(urls[1], "https://example.com/sitemap-blog.xml");
            }
            SitemapContent::UrlSet(_) => panic!("Expected Index, got UrlSet"),
        }
    }

    #[test]
    fn test_parse_gzipped_sitemap() {
        use flate2::write::GzEncoder;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(SITEMAP_XML.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        let content = parse_sitemap(&compressed, true).unwrap();
        match content {
            SitemapContent::UrlSet(entries) => {
                assert_eq!(entries.len(), 3);
                assert_eq!(entries[0].url, "https://example.com/");
            }
            SitemapContent::Index(_) => panic!("Expected UrlSet"),
        }
    }

    #[test]
    fn test_parse_malformed_xml() {
        let malformed = b"<urlset><url><loc>https://example.com/</loc></url><broken";
        let content = parse_sitemap(malformed, false).unwrap();
        match content {
            SitemapContent::UrlSet(entries) => {
                // Should still capture the valid entry before the break.
                assert_eq!(entries.len(), 1);
            }
            SitemapContent::Index(_) => panic!("Expected UrlSet"),
        }
    }
}
