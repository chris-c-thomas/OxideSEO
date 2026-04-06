//! robots.txt fetcher, parser, and per-domain cache.
//!
//! Uses the `texting_robots` crate for RFC 9309 compliant parsing.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use texting_robots::Robot;

/// Cached robots.txt info for a single domain.
#[derive(Debug)]
struct RobotsEntry {
    /// Parsed robot rules (None if robots.txt was unavailable/invalid).
    robot: Option<Robot>,
    /// Crawl-delay directive value (if present), in seconds.
    crawl_delay: Option<f64>,
    /// Sitemap URLs declared in this robots.txt.
    sitemaps: Vec<String>,
    /// When this entry was cached.
    cached_at: Instant,
}

/// Cache and fetcher for robots.txt files.
pub struct RobotsCache {
    /// Cached entries per domain.
    cache: HashMap<String, RobotsEntry>,
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

    /// Check whether we have a cached (non-expired) entry for this domain.
    pub fn has_cached(&self, domain: &str) -> bool {
        self.cache
            .get(domain)
            .is_some_and(|e| e.cached_at.elapsed() < self.ttl)
    }

    /// Check if a URL is allowed by the domain's robots.txt.
    ///
    /// If the robots.txt hasn't been fetched yet, returns `true`
    /// (allow by default — actual fetch should be triggered first).
    pub fn is_allowed(&self, url: &str) -> bool {
        let domain = match url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
        {
            Some(d) => d,
            None => return true,
        };

        match self.cache.get(&domain) {
            Some(entry) if entry.cached_at.elapsed() < self.ttl => {
                match &entry.robot {
                    Some(robot) => robot.allowed(url),
                    None => true, // No valid robots.txt = allow all
                }
            }
            _ => true, // Not cached or expired
        }
    }

    /// Get sitemap URLs declared in a domain's robots.txt.
    pub fn sitemaps(&self, domain: &str) -> Vec<String> {
        self.cache
            .get(domain)
            .map(|entry| entry.sitemaps.clone())
            .unwrap_or_default()
    }

    /// Get the crawl delay for a domain (if specified in robots.txt).
    pub fn crawl_delay(&self, domain: &str) -> Option<Duration> {
        self.cache
            .get(domain)
            .and_then(|entry| entry.crawl_delay)
            .map(Duration::from_secs_f64)
    }

    /// Fetch and cache robots.txt for the given domain.
    ///
    /// Handles:
    /// - 200: parse with texting_robots and cache
    /// - 4xx: allow all (no restrictions)
    /// - 5xx/network error: allow all
    pub async fn fetch_and_cache(&mut self, domain: &str, client: &reqwest::Client) -> Result<()> {
        self.fetch_and_cache_with_scheme(domain, "https", client)
            .await
    }

    /// Fetch robots.txt with a specific URL scheme (http or https).
    pub async fn fetch_and_cache_with_scheme(
        &mut self,
        domain: &str,
        scheme: &str,
        client: &reqwest::Client,
    ) -> Result<()> {
        let robots_url = format!("{}://{}/robots.txt", scheme, domain);

        let response = match client.get(&robots_url).send().await {
            Ok(r) => r,
            Err(_) => {
                self.cache.insert(
                    domain.to_string(),
                    RobotsEntry {
                        robot: None,
                        crawl_delay: None,
                        sitemaps: Vec::new(),
                        cached_at: Instant::now(),
                    },
                );
                return Ok(());
            }
        };

        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        let entry = if status == 200 {
            match Robot::new(&self.user_agent, body.as_bytes()) {
                Ok(robot) => {
                    let crawl_delay = robot.delay.map(|d| d as f64);
                    let sitemaps: Vec<String> =
                        robot.sitemaps.iter().map(|s| s.to_string()).collect();
                    RobotsEntry {
                        robot: Some(robot),
                        crawl_delay,
                        sitemaps,
                        cached_at: Instant::now(),
                    }
                }
                Err(_) => RobotsEntry {
                    robot: None,
                    crawl_delay: None,
                    sitemaps: Vec::new(),
                    cached_at: Instant::now(),
                },
            }
        } else {
            // 4xx, 5xx, or other — allow all
            RobotsEntry {
                robot: None,
                crawl_delay: None,
                sitemaps: Vec::new(),
                cached_at: Instant::now(),
            }
        };

        self.cache.insert(domain.to_string(), entry);
        Ok(())
    }

    /// Insert robots.txt content directly (used in tests).
    pub fn insert_from_content(&mut self, domain: &str, content: &[u8]) {
        let entry = match Robot::new(&self.user_agent, content) {
            Ok(robot) => {
                let crawl_delay = robot.delay.map(|d| d as f64);
                let sitemaps: Vec<String> = robot.sitemaps.iter().map(|s| s.to_string()).collect();
                RobotsEntry {
                    robot: Some(robot),
                    crawl_delay,
                    sitemaps,
                    cached_at: Instant::now(),
                }
            }
            Err(_) => RobotsEntry {
                robot: None,
                crawl_delay: None,
                sitemaps: Vec::new(),
                cached_at: Instant::now(),
            },
        };
        self.cache.insert(domain.to_string(), entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] = include_bytes!("../../../tests/fixtures/robots.txt");

    #[test]
    fn test_allowed_paths_wildcard_agent() {
        let mut cache = RobotsCache::new("SomeBot", 3600);
        cache.insert_from_content("test.local", FIXTURE);

        assert!(cache.is_allowed("https://test.local/"));
        assert!(cache.is_allowed("https://test.local/about"));
        assert!(!cache.is_allowed("https://test.local/admin/"));
        assert!(!cache.is_allowed("https://test.local/admin/settings"));
        assert!(!cache.is_allowed("https://test.local/private/"));
        assert!(!cache.is_allowed("https://test.local/private/data"));
    }

    #[test]
    fn test_allowed_paths_oxideseo_agent() {
        let mut cache = RobotsCache::new("OxideSEO", 3600);
        cache.insert_from_content("test.local", FIXTURE);

        assert!(cache.is_allowed("https://test.local/"));
        assert!(cache.is_allowed("https://test.local/about"));
        assert!(!cache.is_allowed("https://test.local/admin/"));
    }

    #[test]
    fn test_crawl_delay_wildcard() {
        let mut cache = RobotsCache::new("SomeBot", 3600);
        cache.insert_from_content("test.local", FIXTURE);

        let delay = cache.crawl_delay("test.local");
        assert!(delay.is_some());
        assert_eq!(delay.unwrap(), Duration::from_secs(1));
    }

    #[test]
    fn test_crawl_delay_oxideseo() {
        let mut cache = RobotsCache::new("OxideSEO", 3600);
        cache.insert_from_content("test.local", FIXTURE);

        let delay = cache.crawl_delay("test.local");
        assert!(delay.is_some());
        assert_eq!(delay.unwrap(), Duration::from_millis(500));
    }

    #[test]
    fn test_uncached_domain_allows_all() {
        let cache = RobotsCache::new("OxideSEO", 3600);
        assert!(cache.is_allowed("https://uncached.example.com/anything"));
    }

    #[test]
    fn test_has_cached() {
        let mut cache = RobotsCache::new("OxideSEO", 3600);
        assert!(!cache.has_cached("test.local"));

        cache.insert_from_content("test.local", FIXTURE);
        assert!(cache.has_cached("test.local"));
    }

    #[test]
    fn test_invalid_robots_allows_all() {
        let mut cache = RobotsCache::new("OxideSEO", 3600);
        cache.insert_from_content("test.local", b"not valid robots content @#$%");

        // Invalid content should still result in a cached entry that allows all
        assert!(cache.is_allowed("https://test.local/admin/"));
    }

    #[test]
    fn test_empty_robots_allows_all() {
        let mut cache = RobotsCache::new("OxideSEO", 3600);
        cache.insert_from_content("test.local", b"");

        assert!(cache.is_allowed("https://test.local/anything"));
    }
}
