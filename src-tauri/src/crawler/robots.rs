//! robots.txt fetcher, parser, and per-domain cache.
//!
//! Uses the `texting_robots` crate for RFC 9309 compliant parsing.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;

/// Cached robots.txt info for a single domain.
#[derive(Debug, Clone)]
pub struct RobotsInfo {
    /// Whether the robots.txt was successfully fetched and parsed.
    pub is_valid: bool,
    /// The raw robots.txt content.
    pub content: String,
    /// Crawl-delay directive value (if present), in seconds.
    pub crawl_delay: Option<f64>,
    /// When this entry was cached.
    pub cached_at: Instant,
}

/// Cache and fetcher for robots.txt files.
pub struct RobotsCache {
    /// Cached robots.txt per domain.
    cache: HashMap<String, RobotsInfo>,
    /// Cache TTL — entries older than this are re-fetched.
    ttl: Duration,
    /// User-agent string to match against in robots.txt directives.
    user_agent: String,
}

impl RobotsCache {
    pub fn new(user_agent: &str, ttl_secs: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
            user_agent: user_agent.to_string(),
        }
    }

    /// Check if a URL is allowed by the domain's robots.txt.
    ///
    /// If the robots.txt hasn't been fetched yet, returns `true`
    /// (allow by default — actual fetch should be triggered first).
    pub fn is_allowed(&self, url: &str) -> bool {
        let domain = match url::Url::parse(url).ok().and_then(|u| u.host_str().map(|h| h.to_string())) {
            Some(d) => d,
            None => return true,
        };

        match self.cache.get(&domain) {
            Some(info) if info.cached_at.elapsed() < self.ttl => {
                if !info.is_valid {
                    return true; // No valid robots.txt = allow all.
                }
                // TODO(phase-2): Use texting_robots to check if `url` is allowed
                // for `self.user_agent`.
                true
            }
            _ => true, // Not cached or expired — allow (will be fetched).
        }
    }

    /// Get the crawl delay for a domain (if specified in robots.txt).
    pub fn crawl_delay(&self, domain: &str) -> Option<Duration> {
        self.cache
            .get(domain)
            .and_then(|info| info.crawl_delay)
            .map(|secs| Duration::from_secs_f64(secs))
    }

    /// Fetch and cache robots.txt for the given domain.
    ///
    /// Handles:
    /// - 200: parse and cache
    /// - 4xx: allow all (no restrictions)
    /// - 5xx: retry once, then allow all
    /// - Network error: allow all
    pub async fn fetch_and_cache(
        &mut self,
        domain: &str,
        client: &reqwest::Client,
    ) -> Result<()> {
        let robots_url = format!("https://{}/robots.txt", domain);

        let response = match client.get(&robots_url).send().await {
            Ok(r) => r,
            Err(_) => {
                // Network error — cache as invalid (allow all).
                self.cache.insert(domain.to_string(), RobotsInfo {
                    is_valid: false,
                    content: String::new(),
                    crawl_delay: None,
                    cached_at: Instant::now(),
                });
                return Ok(());
            }
        };

        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        let info = match status {
            200 => {
                // TODO(phase-2): Parse with texting_robots, extract Crawl-delay.
                RobotsInfo {
                    is_valid: true,
                    content: body,
                    crawl_delay: None, // Extract from parsed rules.
                    cached_at: Instant::now(),
                }
            }
            400..=499 => RobotsInfo {
                is_valid: false,
                content: String::new(),
                crawl_delay: None,
                cached_at: Instant::now(),
            },
            _ => {
                // 5xx or other — allow all for now.
                RobotsInfo {
                    is_valid: false,
                    content: String::new(),
                    crawl_delay: None,
                    cached_at: Instant::now(),
                }
            }
        };

        self.cache.insert(domain.to_string(), info);
        Ok(())
    }
}
