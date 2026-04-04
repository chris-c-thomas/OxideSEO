//! Data models corresponding to the SQLite schema.
//!
//! These structs are used for both database serialization (via rusqlite)
//! and IPC serialization (via serde) to the frontend.

use serde::{Deserialize, Serialize};

/// Row in the `crawls` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlRow {
    pub id: String,
    pub start_url: String,
    pub config_json: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub urls_crawled: i64,
    pub urls_errored: i64,
}

/// Row in the `pages` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRow {
    pub id: i64,
    pub crawl_id: String,
    pub url: String,
    pub depth: i32,
    pub status_code: Option<i32>,
    pub content_type: Option<String>,
    pub response_time_ms: Option<i32>,
    pub body_size: Option<i64>,
    pub title: Option<String>,
    pub meta_desc: Option<String>,
    pub h1: Option<String>,
    pub canonical: Option<String>,
    pub robots_directives: Option<String>,
    pub state: String,
    pub fetched_at: Option<String>,
    pub error_message: Option<String>,
}

/// Row in the `links` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkRow {
    pub id: i64,
    pub crawl_id: String,
    pub source_page: i64,
    pub target_url: String,
    pub anchor_text: Option<String>,
    pub link_type: String,
    pub is_internal: bool,
    pub nofollow: bool,
}

/// Row in the `issues` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueRow {
    pub id: i64,
    pub crawl_id: String,
    pub page_id: i64,
    pub rule_id: String,
    pub severity: String,
    pub category: String,
    pub message: String,
    pub detail_json: Option<String>,
}

/// Batch of items to write to the database.
///
/// The storage writer thread receives these via a channel and flushes
/// them inside a single SQLite transaction.
#[derive(Debug)]
pub enum StorageCommand {
    /// Insert or update a page record (includes url_hash for dedup key).
    UpsertPage {
        page: Box<PageRow>,
        url_hash: Vec<u8>,
    },
    /// Insert a page with its associated links and issues.
    /// The writer resolves the page ID after upsert and sets it on links/issues.
    StorePage {
        page: Box<PageRow>,
        url_hash: Vec<u8>,
        links: Vec<LinkRow>,
        issues: Vec<IssueRow>,
    },
    /// Insert one or more link records.
    InsertLinks(Vec<LinkRow>),
    /// Insert one or more issue records.
    InsertIssues(Vec<IssueRow>),
    /// Update crawl counters.
    UpdateCrawlStats {
        crawl_id: String,
        urls_crawled: i64,
        urls_errored: i64,
    },
    /// Mark the crawl as completed.
    CompleteCrawl { crawl_id: String, status: String },
    /// Flush current batch and commit transaction.
    Flush,
    /// Shutdown the storage writer.
    Shutdown,
}
