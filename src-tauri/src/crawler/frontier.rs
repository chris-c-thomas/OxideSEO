//! URL frontier: priority queue with deduplication and SQLite backing store.
//!
//! The frontier manages the set of URLs to crawl. It provides:
//! - blake3 hash-based deduplication
//! - BFS-priority ordering (lower depth = higher priority)
//! - In-memory working set with SQLite overflow
//! - Persist/restore for pause/resume

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use url::Url;

use crate::storage::db::Database;

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
    /// Database handle for overflow/refill (None for in-memory-only mode).
    db: Option<Arc<Database>>,
    /// Crawl ID for scoping DB queries.
    crawl_id: String,
    /// Count of URLs overflowed to SQLite (not yet refilled).
    overflow_count: u64,
}

impl UrlFrontier {
    /// Create a new frontier with the given heap capacity (in-memory only).
    pub fn new(heap_capacity: usize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(heap_capacity),
            heap_capacity,
            refill_threshold: heap_capacity / 4,
            seen_hashes: HashSet::new(),
            domain_last_fetch: HashMap::new(),
            total_discovered: 0,
            total_dequeued: 0,
            db: None,
            crawl_id: String::new(),
            overflow_count: 0,
        }
    }

    /// Create a frontier backed by SQLite for overflow/refill.
    pub fn with_db(heap_capacity: usize, db: Arc<Database>, crawl_id: String) -> Self {
        Self {
            db: Some(db),
            crawl_id,
            ..Self::new(heap_capacity)
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
            // Overflow to SQLite backing store.
            self.overflow_to_db(&entry);
        }
        true
    }

    /// Dequeue the highest-priority URL. Returns `None` if empty.
    pub fn pop(&mut self) -> Option<FrontierEntry> {
        // Refill from SQLite when heap drops below threshold.
        if self.heap.len() < self.refill_threshold && self.overflow_count > 0 {
            self.refill_from_db();
        }

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

    /// True if both heap and overflow are empty.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty() && self.overflow_count == 0
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

    /// Total queued URLs (heap + overflow).
    pub fn total_queued(&self) -> u64 {
        self.heap.len() as u64 + self.overflow_count
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

    /// Write an overflow entry to the pages table with state='queued'.
    fn overflow_to_db(&mut self, entry: &FrontierEntry) {
        let Some(db) = &self.db else {
            // No DB — just push to heap anyway (exceeds capacity).
            self.heap.push(entry.clone());
            return;
        };

        let result = db.with_conn(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO pages (crawl_id, url, url_hash, depth, state)
                 VALUES (?1, ?2, ?3, ?4, 'queued')",
                rusqlite::params![
                    self.crawl_id,
                    entry.url,
                    entry.url_hash.as_slice(),
                    entry.depth,
                ],
            )?;
            Ok(())
        });

        match result {
            Ok(()) => self.overflow_count += 1,
            Err(e) => {
                tracing::warn!(url = %entry.url, error = %e, "Frontier overflow to DB failed");
                // Fallback: push to heap anyway.
                self.heap.push(entry.clone());
            }
        }
    }

    /// Refill the heap from SQLite overflow entries.
    fn refill_from_db(&mut self) {
        let Some(db) = &self.db else {
            return;
        };

        let batch_size = (self.heap_capacity / 2).max(1) as i64;

        let result = db.with_conn(|conn| {
            // Select entries to refill.
            let mut stmt = conn.prepare(
                "SELECT id, url, url_hash, depth FROM pages
                 WHERE crawl_id = ?1 AND state = 'queued'
                 ORDER BY depth ASC
                 LIMIT ?2",
            )?;

            let entries: Vec<(i64, FrontierEntry)> = stmt
                .query_map(rusqlite::params![self.crawl_id, batch_size], |row| {
                    let page_id: i64 = row.get(0)?;
                    let url: String = row.get(1)?;
                    let url_hash_blob: Vec<u8> = row.get(2)?;
                    let depth: u32 = row.get(3)?;

                    let mut url_hash = [0u8; 32];
                    if url_hash_blob.len() == 32 {
                        url_hash.copy_from_slice(&url_hash_blob);
                    }

                    Ok((
                        page_id,
                        FrontierEntry {
                            url,
                            url_hash,
                            depth,
                            priority: 100 - depth as i32,
                            source_page_id: None,
                        },
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            // Mark refilled entries as 'discovered' using their IDs.
            for (page_id, _) in &entries {
                conn.execute(
                    "UPDATE pages SET state = 'discovered' WHERE id = ?1",
                    rusqlite::params![page_id],
                )?;
            }

            Ok(entries)
        });

        match result {
            Ok(entries) => {
                let count = entries.len() as u64;
                for (_, entry) in entries {
                    self.heap.push(entry);
                }
                self.overflow_count = self.overflow_count.saturating_sub(count);
                tracing::debug!(count, "Frontier refilled from DB");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Frontier refill from DB failed");
            }
        }
    }

    /// Persist all in-memory heap entries to SQLite for pause/resume.
    pub fn persist(&self) -> Result<()> {
        let Some(db) = &self.db else {
            return Ok(());
        };

        db.with_conn_mut(|conn| {
            let tx = conn.transaction()?;
            for entry in self.heap.iter() {
                tx.execute(
                    "INSERT OR IGNORE INTO pages (crawl_id, url, url_hash, depth, state)
                     VALUES (?1, ?2, ?3, ?4, 'queued')",
                    rusqlite::params![
                        self.crawl_id,
                        entry.url,
                        entry.url_hash.as_slice(),
                        entry.depth,
                    ],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
    }

    /// Restore a frontier from SQLite (load all 'queued' entries).
    pub fn restore(db: Arc<Database>, crawl_id: &str, heap_capacity: usize) -> Result<Self> {
        let mut frontier = Self::with_db(heap_capacity, db.clone(), crawl_id.to_string());

        let entries = db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT url, url_hash, depth FROM pages
                 WHERE crawl_id = ?1 AND state = 'queued'
                 ORDER BY depth ASC",
            )?;

            let entries: Vec<FrontierEntry> = stmt
                .query_map(rusqlite::params![crawl_id], |row| {
                    let url: String = row.get(0)?;
                    let url_hash_blob: Vec<u8> = row.get(1)?;
                    let depth: u32 = row.get(2)?;

                    let mut url_hash = [0u8; 32];
                    if url_hash_blob.len() == 32 {
                        url_hash.copy_from_slice(&url_hash_blob);
                    }

                    Ok(FrontierEntry {
                        url,
                        url_hash,
                        depth,
                        priority: 100 - depth as i32,
                        source_page_id: None,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();
            Ok(entries)
        })?;

        let total = entries.len() as u64;
        for entry in entries {
            frontier.seen_hashes.insert(entry.url_hash);
            if frontier.heap.len() < heap_capacity {
                frontier.heap.push(entry);
            } else {
                frontier.overflow_count += 1;
            }
        }
        frontier.total_discovered = total;

        Ok(frontier)
    }
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

    // --- SQLite overflow/refill tests ---

    fn make_entry(path: &str, depth: u32) -> FrontierEntry {
        let url = format!("https://example.com{}", path);
        FrontierEntry {
            url_hash: hash_url(&url),
            url,
            depth,
            priority: 100 - depth as i32,
            source_page_id: None,
        }
    }

    #[test]
    fn test_overflow_and_refill() {
        let db = Arc::new(Database::new_in_memory().unwrap());

        // Insert a crawl record (pages FK requires it).
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO crawls (id, start_url, config_json, status) VALUES ('c1', 'https://example.com', '{}', 'running')",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        let mut frontier = UrlFrontier::with_db(5, db.clone(), "c1".into());

        // Push 10 entries — 5 fit in heap, 5 overflow to DB.
        for i in 0..10 {
            let path = format!("/page{}", i);
            frontier.push(make_entry(&path, i as u32));
        }

        assert_eq!(frontier.len(), 5); // heap has 5
        assert_eq!(frontier.overflow_count, 5); // DB has 5
        assert_eq!(frontier.total_discovered(), 10);

        // Pop all 10 — the frontier should refill from DB when heap runs low.
        let mut popped = Vec::new();
        while let Some(entry) = frontier.pop() {
            popped.push(entry.url);
        }

        assert_eq!(popped.len(), 10);
        assert!(frontier.is_empty());
    }

    #[test]
    fn test_persist_and_restore() {
        let db = Arc::new(Database::new_in_memory().unwrap());

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO crawls (id, start_url, config_json, status) VALUES ('c2', 'https://example.com', '{}', 'running')",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        // Create frontier with 3 entries in heap.
        let mut frontier = UrlFrontier::with_db(100, db.clone(), "c2".into());
        frontier.push(make_entry("/a", 0));
        frontier.push(make_entry("/b", 1));
        frontier.push(make_entry("/c", 2));
        assert_eq!(frontier.len(), 3);

        // Persist to DB.
        frontier.persist().unwrap();

        // Restore from DB into a new frontier.
        let mut restored = UrlFrontier::restore(db, "c2", 100).unwrap();
        assert_eq!(restored.total_discovered(), 3);

        // Pop and verify order (BFS: depth 0 first).
        let first = restored.pop().unwrap();
        assert!(first.url.contains("/a"));
    }

    #[test]
    fn test_dedup_across_overflow() {
        let db = Arc::new(Database::new_in_memory().unwrap());

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO crawls (id, start_url, config_json, status) VALUES ('c3', 'https://example.com', '{}', 'running')",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        let mut frontier = UrlFrontier::with_db(3, db, "c3".into());

        // Push 5 unique entries.
        for i in 0..5 {
            assert!(frontier.push(make_entry(&format!("/p{}", i), 0)));
        }

        // Try to re-push — all should be rejected by seen_hashes.
        for i in 0..5 {
            assert!(!frontier.push(make_entry(&format!("/p{}", i), 0)));
        }

        assert_eq!(frontier.total_discovered(), 5);
    }
}
