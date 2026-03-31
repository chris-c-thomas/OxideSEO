//! URL frontier: priority queue with deduplication and SQLite backing store.
//!
//! The frontier manages the set of URLs to crawl. It provides:
//! - blake3 hash-based deduplication
//! - BFS-priority ordering (lower depth = higher priority)
//! - In-memory working set with SQLite overflow
//! - Persist/restore for pause/resume

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Instant;

use anyhow::Result;
use url::Url;

/// A URL queued for crawling, with priority metadata.
#[derive(Debug, Clone)]
pub struct FrontierEntry {
    pub url: String,
    pub url_hash: [u8; 32],
    pub depth: u32,
    pub priority: i32,
    pub source_page_id: Option<i64>,
}

// BinaryHeap is a max-heap; we want *highest* priority (lowest depth) first.
impl PartialEq for FrontierEntry {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.depth == other.depth
    }
}

impl Eq for FrontierEntry {}

impl PartialOrd for FrontierEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FrontierEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority value = dequeued first.
        // For equal priority, prefer lower depth (BFS).
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.depth.cmp(&self.depth))
    }
}

/// The URL frontier manages discovered URLs for crawling.
pub struct UrlFrontier {
    /// In-memory working set (max `heap_capacity` entries).
    heap: BinaryHeap<FrontierEntry>,
    /// Max entries to keep in the in-memory heap.
    heap_capacity: usize,
    /// Refill threshold — when heap drops below this, pull from SQLite.
    refill_threshold: usize,
    /// Set of URL hashes already seen (for fast dedup without DB round-trip).
    seen_hashes: HashSet<[u8; 32]>,
    /// Per-domain last-fetch timestamp for politeness enforcement.
    domain_last_fetch: HashMap<String, Instant>,
    /// Total URLs discovered (including those in SQLite overflow).
    total_discovered: u64,
    /// Total URLs dequeued and sent for fetching.
    total_dequeued: u64,
}

impl UrlFrontier {
    /// Create a new frontier with the given heap capacity.
    pub fn new(heap_capacity: usize) -> Self {
        let refill_threshold = heap_capacity / 4;
        Self {
            heap: BinaryHeap::with_capacity(heap_capacity),
            heap_capacity,
            refill_threshold,
            seen_hashes: HashSet::new(),
            domain_last_fetch: HashMap::new(),
            total_discovered: 0,
            total_dequeued: 0,
        }
    }

    /// Add a URL to the frontier. Returns `false` if already seen (deduplicated).
    pub fn push(&mut self, entry: FrontierEntry) -> bool {
        if self.seen_hashes.contains(&entry.url_hash) {
            return false;
        }
        self.seen_hashes.insert(entry.url_hash);
        self.total_discovered += 1;

        if self.heap.len() < self.heap_capacity {
            self.heap.push(entry);
        } else {
            // TODO(phase-2): Overflow to SQLite backing store.
            // For now, just push if below capacity.
            self.heap.push(entry);
        }
        true
    }

    /// Dequeue the highest-priority URL. Returns `None` if empty.
    pub fn pop(&mut self) -> Option<FrontierEntry> {
        // TODO(phase-2): Refill from SQLite when heap is below threshold.
        let entry = self.heap.pop();
        if entry.is_some() {
            self.total_dequeued += 1;
        }
        entry
    }

    /// Number of URLs currently in the in-memory heap.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Check if a URL hash has already been seen.
    pub fn contains(&self, hash: &[u8; 32]) -> bool {
        self.seen_hashes.contains(hash)
    }

    pub fn total_discovered(&self) -> u64 {
        self.total_discovered
    }

    pub fn total_dequeued(&self) -> u64 {
        self.total_dequeued
    }

    /// Record the last fetch time for a domain (for politeness tracking).
    pub fn record_fetch(&mut self, domain: &str) {
        self.domain_last_fetch
            .insert(domain.to_string(), Instant::now());
    }

    /// Get the last fetch time for a domain.
    pub fn last_fetch_time(&self, domain: &str) -> Option<Instant> {
        self.domain_last_fetch.get(domain).copied()
    }

    // TODO(phase-2):
    // pub fn persist(&self, db: &Database) -> Result<()>
    // pub fn restore(db: &Database, heap_capacity: usize) -> Result<Self>
}

// ---------------------------------------------------------------------------
// URL normalization
// ---------------------------------------------------------------------------

/// Normalize a URL for consistent hashing and deduplication.
///
/// Normalization steps:
/// 1. Lowercase scheme and host.
/// 2. Remove fragments.
/// 3. Sort query parameters alphabetically.
/// 4. Remove default ports (80/443).
/// 5. Resolve dot segments.
/// 6. Strip trailing slash (configurable).
pub fn normalize_url(raw: &str, strip_trailing_slash: bool) -> Result<String> {
    let mut parsed = Url::parse(raw)?;

    // Remove fragment.
    parsed.set_fragment(None);

    // Remove default ports.
    if let Some(port) = parsed.port() {
        match (parsed.scheme(), port) {
            ("http", 80) | ("https", 443) => {
                let _ = parsed.set_port(None);
            }
            _ => {}
        }
    }

    // Sort query parameters.
    if let Some(query) = parsed.query() {
        if !query.is_empty() {
            let mut pairs: Vec<(String, String)> = parsed
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();
            pairs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

            let sorted_query: String = pairs
                .iter()
                .map(|(k, v)| {
                    if v.is_empty() {
                        k.clone()
                    } else {
                        format!("{}={}", k, v)
                    }
                })
                .collect::<Vec<_>>()
                .join("&");
            parsed.set_query(Some(&sorted_query));
        }
    }

    let mut result = parsed.to_string();

    // Strip trailing slash (optional, configurable).
    if strip_trailing_slash && result.ends_with('/') && parsed.path() != "/" {
        result.pop();
    }

    Ok(result)
}

/// Compute a blake3 hash for a normalized URL string.
pub fn hash_url(normalized_url: &str) -> [u8; 32] {
    blake3::hash(normalized_url.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_removes_fragment() {
        let result = normalize_url("https://example.com/page#section", false).unwrap();
        assert_eq!(result, "https://example.com/page");
    }

    #[test]
    fn test_normalize_removes_default_port() {
        let result = normalize_url("https://example.com:443/page", false).unwrap();
        assert_eq!(result, "https://example.com/page");
    }

    #[test]
    fn test_normalize_sorts_query_params() {
        let result = normalize_url("https://example.com/page?z=1&a=2&m=3", false).unwrap();
        assert_eq!(result, "https://example.com/page?a=2&m=3&z=1");
    }

    #[test]
    fn test_normalize_strips_trailing_slash() {
        let result = normalize_url("https://example.com/page/", true).unwrap();
        assert_eq!(result, "https://example.com/page");
    }

    #[test]
    fn test_normalize_preserves_root_slash() {
        let result = normalize_url("https://example.com/", true).unwrap();
        assert_eq!(result, "https://example.com/");
    }

    #[test]
    fn test_hash_url_deterministic() {
        let url = "https://example.com/page";
        assert_eq!(hash_url(url), hash_url(url));
    }

    #[test]
    fn test_hash_url_different_urls() {
        assert_ne!(
            hash_url("https://example.com/a"),
            hash_url("https://example.com/b")
        );
    }

    #[test]
    fn test_frontier_dedup() {
        let mut frontier = UrlFrontier::new(100);
        let hash = hash_url("https://example.com/");
        let entry = FrontierEntry {
            url: "https://example.com/".into(),
            url_hash: hash,
            depth: 0,
            priority: 100,
            source_page_id: None,
        };
        assert!(frontier.push(entry.clone()));
        assert!(!frontier.push(entry)); // duplicate rejected
    }

    #[test]
    fn test_frontier_priority_order() {
        let mut frontier = UrlFrontier::new(100);

        // Higher priority should be dequeued first.
        let low = FrontierEntry {
            url: "https://example.com/low".into(),
            url_hash: hash_url("https://example.com/low"),
            depth: 3,
            priority: 10,
            source_page_id: None,
        };
        let high = FrontierEntry {
            url: "https://example.com/high".into(),
            url_hash: hash_url("https://example.com/high"),
            depth: 1,
            priority: 100,
            source_page_id: None,
        };

        frontier.push(low);
        frontier.push(high);

        let first = frontier.pop().unwrap();
        assert_eq!(first.url, "https://example.com/high");
    }
}
