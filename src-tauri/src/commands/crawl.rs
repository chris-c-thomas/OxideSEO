//! Crawl lifecycle commands: start, pause, resume, stop, status.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::CrawlState;
use crate::storage::db::Database;

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

/// Configuration payload sent from the frontend to start a crawl.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlConfig {
    /// Seed URL to begin crawling from.
    pub start_url: String,
    /// Maximum link depth to follow (1-100, default 10).
    pub max_depth: u32,
    /// Global concurrent request cap (1-200, default 50).
    pub max_concurrency: u32,
    /// Number of async fetch workers (1-32, default 8).
    pub fetch_workers: u32,
    /// Number of rayon parse threads (1-N, default CPU_COUNT-2).
    pub parse_threads: u32,
    /// Memory budget in MB (128-16384, default 512).
    pub max_memory_mb: u32,
    /// Whether to obey robots.txt directives.
    pub respect_robots_txt: bool,
    /// URL patterns to include (regex strings).
    pub include_patterns: Vec<String>,
    /// URL patterns to exclude (regex strings).
    pub exclude_patterns: Vec<String>,
    /// Per-request timeout in seconds (5-120, default 30).
    pub request_timeout_secs: u32,
    /// Minimum delay between requests to the same host in ms (0-10000, default 500).
    pub crawl_delay_ms: u32,
    /// Custom User-Agent string (optional).
    pub user_agent: Option<String>,
    /// Custom HTTP headers as key-value pairs.
    pub custom_headers: Vec<(String, String)>,
    /// Per-host concurrent request limit (default 2).
    pub per_host_concurrency: u32,
    /// Maximum number of pages to crawl (0 = unlimited).
    pub max_pages: u32,
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            start_url: String::new(),
            max_depth: 10,
            max_concurrency: 50,
            fetch_workers: 8,
            parse_threads: num_cpus(),
            max_memory_mb: 512,
            respect_robots_txt: true,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            request_timeout_secs: 30,
            crawl_delay_ms: 500,
            user_agent: None,
            custom_headers: Vec::new(),
            per_host_concurrency: 2,
            max_pages: 0,
        }
    }
}

fn num_cpus() -> u32 {
    let n = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4);
    n.saturating_sub(2).max(1)
}

/// Real-time crawl status returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStatus {
    pub crawl_id: String,
    pub state: CrawlState,
    pub urls_crawled: u64,
    pub urls_queued: u64,
    pub urls_errored: u64,
    pub elapsed_ms: u64,
    pub current_rps: f64,
}

/// Progress event payload emitted via Tauri events at ~4Hz.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlProgress {
    pub crawl_id: String,
    pub urls_crawled: u64,
    pub urls_queued: u64,
    pub urls_errored: u64,
    pub current_rps: f64,
    pub elapsed_ms: u64,
    pub recent_urls: Vec<RecentUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentUrl {
    pub url: String,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u32>,
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Start a new crawl with the given configuration.
/// Returns the unique crawl ID.
#[tauri::command]
pub async fn start_crawl(
    config: CrawlConfig,
    _db: State<'_, Database>,
    _app: tauri::AppHandle,
) -> Result<String, String> {
    tracing::info!(url = %config.start_url, "Starting crawl");

    // TODO(phase-2): Validate config, create crawl record in DB,
    // spawn the crawl engine orchestrator task, return crawl_id.
    let _crawl_id = uuid::Uuid::new_v4().to_string();

    Err("Not yet implemented — Phase 2".into())
}

/// Pause an active crawl. In-flight requests will complete; the frontier
/// is persisted to SQLite for later resume.
#[tauri::command]
pub async fn pause_crawl(crawl_id: String) -> Result<(), String> {
    tracing::info!(%crawl_id, "Pausing crawl");
    // TODO(phase-2): Signal the orchestrator to stop dequeuing.
    Err("Not yet implemented — Phase 2".into())
}

/// Resume a previously paused crawl. Restores frontier from SQLite.
#[tauri::command]
pub async fn resume_crawl(crawl_id: String) -> Result<(), String> {
    tracing::info!(%crawl_id, "Resuming crawl");
    // TODO(phase-2): Restore frontier, restart orchestrator loop.
    Err("Not yet implemented — Phase 2".into())
}

/// Stop a crawl entirely. Cancels in-flight requests and marks crawl as stopped.
#[tauri::command]
pub async fn stop_crawl(crawl_id: String) -> Result<(), String> {
    tracing::info!(%crawl_id, "Stopping crawl");
    // TODO(phase-2): Cancel all tasks, persist final state.
    Err("Not yet implemented — Phase 2".into())
}

/// Get the current status of a crawl.
#[tauri::command]
pub async fn get_crawl_status(
    crawl_id: String,
    _db: State<'_, Database>,
) -> Result<CrawlStatus, String> {
    tracing::debug!(%crawl_id, "Getting crawl status");
    // TODO(phase-2): Query crawl state from DB + in-memory stats.
    Err("Not yet implemented — Phase 2".into())
}
