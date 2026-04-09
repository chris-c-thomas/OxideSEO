//! Crawl lifecycle commands: start, pause, resume, stop, delete, re-run, and status.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::CrawlState;
use crate::crawler::engine::{CrawlHandle, TauriEmitter};
use crate::plugin::manager::PluginManager;
use crate::storage::db::Database;
use crate::storage::models::CrawlRow;
use crate::storage::queries;

/// State change event payload emitted to the frontend via `crawl://state`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStateChange {
    pub crawl_id: String,
    pub state: CrawlState,
}

/// Managed state type for the plugin manager.
pub type PluginManagerState = Arc<Mutex<PluginManager>>;

// ---------------------------------------------------------------------------
// Managed state type for active crawl handles
// ---------------------------------------------------------------------------

/// Map of active crawl handles, keyed by crawl_id.
pub type CrawlHandles = Arc<Mutex<HashMap<String, CrawlHandle>>>;

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

    // --- Phase 6: Advanced crawl features ---
    /// Enable JavaScript rendering for SPA detection.
    #[serde(default)]
    pub enable_js_rendering: bool,
    /// Maximum concurrent JS render webviews (default 2).
    #[serde(default = "default_js_render_max_concurrent")]
    pub js_render_max_concurrent: u32,
    /// URL patterns that should always be JS-rendered.
    #[serde(default)]
    pub js_render_patterns: Vec<String>,
    /// URL patterns that should never be JS-rendered.
    #[serde(default)]
    pub js_never_render_patterns: Vec<String>,
    /// Enable sitemap auto-discovery and parsing (default true).
    #[serde(default = "default_true")]
    pub enable_sitemap_discovery: bool,
    /// Enable external link checking via HEAD requests.
    #[serde(default)]
    pub enable_external_link_check: bool,
    /// Global concurrency for external link checks (default 5).
    #[serde(default = "default_external_link_concurrency")]
    pub external_link_concurrency: u32,
    /// URL rewrite rules as (regex, replacement) pairs.
    #[serde(default)]
    pub url_rewrite_rules: Vec<(String, String)>,
    /// Cookies to inject as (name, value) pairs.
    #[serde(default)]
    pub cookies: Vec<(String, String)>,
    /// Custom CSS selectors for data extraction as (label, selector) pairs.
    #[serde(default)]
    pub custom_css_selectors: Vec<(String, String)>,
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
            enable_js_rendering: false,
            js_render_max_concurrent: 2,
            js_render_patterns: Vec::new(),
            js_never_render_patterns: Vec::new(),
            enable_sitemap_discovery: true,
            enable_external_link_check: false,
            external_link_concurrency: 5,
            url_rewrite_rules: Vec::new(),
            cookies: Vec::new(),
            custom_css_selectors: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_js_render_max_concurrent() -> u32 {
    2
}

fn default_external_link_concurrency() -> u32 {
    5
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
    /// Resident set size in bytes (platform-dependent, may be None).
    pub memory_rss_bytes: Option<u64>,
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
    db: State<'_, Arc<Database>>,
    handles: State<'_, CrawlHandles>,
    plugin_manager: State<'_, PluginManagerState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // Validate config.
    if config.start_url.is_empty() {
        return Err("Start URL is required".into());
    }
    if url::Url::parse(&config.start_url).is_err() {
        return Err("Invalid start URL".into());
    }

    let crawl_id = uuid::Uuid::new_v4().to_string();
    tracing::info!(url = %config.start_url, %crawl_id, "Starting crawl");

    // Insert crawl record in DB.
    let config_json = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    db.with_conn(|conn| {
        queries::insert_crawl(
            conn,
            &CrawlRow {
                id: crawl_id.clone(),
                start_url: config.start_url.clone(),
                config_json,
                status: "running".into(),
                started_at: None, // Set by SQL datetime('now')
                completed_at: None,
                urls_crawled: 0,
                urls_errored: 0,
            },
        )
    })
    .map_err(|e| e.to_string())?;

    // Spawn the crawl engine.
    let emitter = Arc::new(TauriEmitter::new(app.clone()));
    let handle = crate::crawler::engine::spawn_crawl(
        crawl_id.clone(),
        config,
        db.inner().clone(),
        emitter,
        Some(app),
        Some(plugin_manager.inner().clone()),
    )
    .await
    .map_err(|e| e.to_string())?;

    // Store the handle for lifecycle control.
    handles.lock().await.insert(crawl_id.clone(), handle);

    Ok(crawl_id)
}

/// Pause an active crawl.
#[tauri::command]
pub async fn pause_crawl(
    crawl_id: String,
    handles: State<'_, CrawlHandles>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tracing::info!(%crawl_id, "Pausing crawl");
    let map = handles.lock().await;
    let handle = map.get(&crawl_id).ok_or("Crawl not found")?;
    handle.pause();
    emit_state_change(&app, &crawl_id, CrawlState::Paused);
    Ok(())
}

/// Resume a previously paused crawl.
#[tauri::command]
pub async fn resume_crawl(
    crawl_id: String,
    handles: State<'_, CrawlHandles>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tracing::info!(%crawl_id, "Resuming crawl");
    let map = handles.lock().await;
    let handle = map.get(&crawl_id).ok_or("Crawl not found")?;
    handle.resume();
    emit_state_change(&app, &crawl_id, CrawlState::Running);
    Ok(())
}

/// Stop a crawl entirely.
#[tauri::command]
pub async fn stop_crawl(
    crawl_id: String,
    handles: State<'_, CrawlHandles>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tracing::info!(%crawl_id, "Stopping crawl");
    let map = handles.lock().await;
    let handle = map.get(&crawl_id).ok_or("Crawl not found")?;
    handle.stop();
    emit_state_change(&app, &crawl_id, CrawlState::Stopped);
    Ok(())
}

/// Get the current status of a crawl.
#[tauri::command]
pub async fn get_crawl_status(
    crawl_id: String,
    handles: State<'_, CrawlHandles>,
    db: State<'_, Arc<Database>>,
) -> Result<CrawlStatus, String> {
    // Check for a live crawl handle first.
    let map = handles.lock().await;
    if let Some(handle) = map.get(&crawl_id) {
        let crawled = handle.stats.urls_crawled.load(Ordering::SeqCst);
        let errored = handle.stats.urls_errored.load(Ordering::SeqCst);
        let elapsed = handle.elapsed_ms();
        let queued = {
            let f = handle.frontier.lock().await;
            f.total_queued()
        };
        let elapsed_secs = elapsed as f64 / 1000.0;
        let rps = if elapsed_secs > 0.0 {
            crawled as f64 / elapsed_secs
        } else {
            0.0
        };

        return Ok(CrawlStatus {
            crawl_id,
            state: handle.state(),
            urls_crawled: crawled,
            urls_queued: queued,
            urls_errored: errored,
            elapsed_ms: elapsed,
            current_rps: rps,
        });
    }
    drop(map);

    // No live handle — query from DB (historical crawl).
    let crawl = db
        .with_conn(|conn| {
            let rows = queries::select_recent_crawls(conn, 1000)?;
            Ok(rows.into_iter().find(|r| r.id == crawl_id))
        })
        .map_err(|e| e.to_string())?
        .ok_or("Crawl not found")?;

    let state = match crawl.status.as_str() {
        "completed" => CrawlState::Completed,
        "stopped" => CrawlState::Stopped,
        "error" => CrawlState::Error,
        "paused" => CrawlState::Paused,
        "running" => CrawlState::Running,
        _ => CrawlState::Created,
    };

    Ok(CrawlStatus {
        crawl_id,
        state,
        urls_crawled: crawl.urls_crawled as u64,
        urls_queued: 0,
        urls_errored: crawl.urls_errored as u64,
        elapsed_ms: 0,
        current_rps: 0.0,
    })
}

/// Delete a crawl and all associated data.
///
/// If the crawl is currently active, it is stopped first.
#[tauri::command]
pub async fn delete_crawl(
    crawl_id: String,
    handles: State<'_, CrawlHandles>,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    tracing::info!(%crawl_id, "Deleting crawl");

    // Stop and remove from active handles if running.
    // Wait for the orchestrator and storage writer to fully shut down
    // before deleting data, to avoid racing with in-flight writes.
    {
        let mut map = handles.lock().await;
        if let Some(handle) = map.remove(&crawl_id) {
            handle.stop();
            drop(map);
            handle.wait_for_shutdown().await;
        }
    }

    db.with_conn_mut(|conn| queries::delete_crawl(conn, &crawl_id))
        .map_err(|e| format!("{e:#}"))?;

    Ok(())
}

/// Re-run a crawl using the same configuration.
///
/// Reads the config from a previous crawl and starts a fresh crawl with it.
#[tauri::command]
pub async fn rerun_crawl(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
    handles: State<'_, CrawlHandles>,
    plugin_manager: State<'_, PluginManagerState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    tracing::info!(%crawl_id, "Re-running crawl");

    let config_json = db
        .with_conn(|conn| queries::select_crawl_config(conn, &crawl_id))
        .map_err(|e| format!("{e:#}"))?
        .ok_or_else(|| format!("Crawl not found: {crawl_id}"))?;

    let config: CrawlConfig =
        serde_json::from_str(&config_json).map_err(|e| format!("Invalid config: {e}"))?;

    // Reuse start_crawl logic.
    start_crawl(config, db, handles, plugin_manager, app).await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Emit a `crawl://state` event so the frontend can detect state changes.
fn emit_state_change(app: &tauri::AppHandle, crawl_id: &str, state: CrawlState) {
    use tauri::Emitter;
    if let Err(e) = app.emit(
        "crawl://state",
        CrawlStateChange {
            crawl_id: crawl_id.to_owned(),
            state,
        },
    ) {
        tracing::warn!(%crawl_id, error = %e, "Failed to emit crawl state change event");
    }
}
