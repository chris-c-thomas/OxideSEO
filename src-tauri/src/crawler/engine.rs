//! Crawl engine orchestrator.
//!
//! Coordinates the URL frontier, fetch workers, parse pipeline,
//! and storage writer into a cohesive crawl loop.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use regex::Regex;
use tokio::sync::{Mutex, Semaphore, watch};
use url::Url;

use crate::CrawlState;
use crate::commands::crawl::{CrawlConfig, CrawlProgress, RecentUrl};
use crate::crawler::fetcher::Fetcher;
use crate::crawler::frontier::{FrontierEntry, UrlFrontier, hash_url, normalize_url};
use crate::crawler::parser;
use crate::crawler::politeness::PolitenessController;
use crate::crawler::robots::RobotsCache;
use crate::rules::engine::RuleRegistry;
use crate::rules::rule::CrawlContext;
use crate::storage::db::Database;
use crate::storage::models::{IssueRow, LinkRow, PageRow, SitemapUrlRow, StorageCommand};
use crate::storage::writer::spawn_storage_writer;

/// Handle to a running crawl. Used to control lifecycle from Tauri commands.
pub struct CrawlHandle {
    pub crawl_id: String,
    pub config: CrawlConfig,
    /// Sender to signal state transitions (pause, resume, stop).
    /// Wrapped in Arc so both the handle and the orchestrator can access it.
    state_tx: Arc<watch::Sender<CrawlState>>,
    /// Receiver to observe current state.
    state_rx: watch::Receiver<CrawlState>,
    /// Shared stats for status queries.
    pub stats: Arc<CrawlStats>,
    /// Frontier for queued count queries.
    pub frontier: Arc<Mutex<UrlFrontier>>,
    /// When the crawl started.
    started_at: Instant,
}

/// Shared atomic crawl statistics.
pub struct CrawlStats {
    pub urls_crawled: AtomicU64,
    pub urls_errored: AtomicU64,
}

impl CrawlHandle {
    pub fn state(&self) -> CrawlState {
        *self.state_rx.borrow()
    }

    pub fn pause(&self) {
        let _ = self.state_tx.send(CrawlState::Paused);
    }

    pub fn resume(&self) {
        let _ = self.state_tx.send(CrawlState::Running);
    }

    pub fn stop(&self) {
        let _ = self.state_tx.send(CrawlState::Stopped);
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
}

/// Trait for progress event emission — allows test mocking.
pub trait ProgressEmitter: Send + Sync + 'static {
    fn emit_progress(&self, progress: &CrawlProgress);
}

/// Real emitter using Tauri's AppHandle.
pub struct TauriEmitter {
    app_handle: tauri::AppHandle,
}

impl TauriEmitter {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

impl ProgressEmitter for TauriEmitter {
    fn emit_progress(&self, progress: &CrawlProgress) {
        use tauri::Emitter;
        let _ = self.app_handle.emit("crawl://progress", progress);
    }
}

/// No-op emitter for tests.
pub struct NoopEmitter;

impl ProgressEmitter for NoopEmitter {
    fn emit_progress(&self, _progress: &CrawlProgress) {}
}

/// Spawn the crawl orchestrator as a tokio task.
///
/// Returns a `CrawlHandle` for lifecycle control.
pub async fn spawn_crawl(
    crawl_id: String,
    config: CrawlConfig,
    db: Arc<Database>,
    emitter: Arc<dyn ProgressEmitter>,
) -> Result<CrawlHandle> {
    let (state_tx, state_rx) = watch::channel(CrawlState::Running);

    let stats = Arc::new(CrawlStats {
        urls_crawled: AtomicU64::new(0),
        urls_errored: AtomicU64::new(0),
    });

    // Initialize components.
    let fetcher = Arc::new(Fetcher::new(&config)?);
    let frontier = Arc::new(Mutex::new(UrlFrontier::with_db(
        10_000,
        db.clone(),
        crawl_id.clone(),
    )));
    let politeness = Arc::new(PolitenessController::new(
        config.crawl_delay_ms as u64,
        config.per_host_concurrency as usize,
    ));
    let robots_cache = Arc::new(Mutex::new(RobotsCache::new(
        config
            .user_agent
            .as_deref()
            .unwrap_or("OxideSEO/0.1 (+https://github.com/oxide-seo/oxide-seo)"),
        3600,
    )));

    let mut rule_registry = RuleRegistry::new();
    rule_registry.register_builtins();
    let rule_registry = Arc::new(rule_registry);

    // Extract root domain and scheme from seed URL.
    let seed_url = normalize_url(&config.start_url, true)?;
    let parsed_seed = Url::parse(&seed_url)?;
    let root_domain = parsed_seed.host_str().unwrap_or("").to_string();
    let seed_scheme = parsed_seed.scheme().to_string();

    // Compile URL patterns for filtering and rewriting.
    let compiled_patterns =
        Arc::new(CompiledPatterns::from_config(&config).context("Failed to compile URL patterns")?);

    // Seed the frontier.
    {
        let mut f = frontier.lock().await;
        f.push(FrontierEntry {
            url: seed_url,
            url_hash: hash_url(&normalize_url(&config.start_url, true)?),
            depth: 0,
            priority: 100,
            source_page_id: None,
        });
    }

    // Storage writer channel + thread.
    let (storage_tx, storage_rx) = tokio::sync::mpsc::channel::<StorageCommand>(5000);
    let _writer_handle = spawn_storage_writer(db.clone(), storage_rx, 200);

    // External link checker channel (only active when enabled).
    let external_checker_tx: Option<
        tokio::sync::mpsc::Sender<crate::crawler::external_checker::ExternalLinkEntry>,
    > = if config.enable_external_link_check {
        let (ext_tx, ext_rx) = tokio::sync::mpsc::channel(5000);
        let ext_storage_tx = storage_tx.clone();
        let ext_concurrency = config.external_link_concurrency;
        tokio::spawn(async move {
            crate::crawler::external_checker::run_external_checker(
                ext_rx,
                ext_storage_tx,
                ext_concurrency,
            )
            .await;
            tracing::info!("External link checker finished");
        });
        Some(ext_tx)
    } else {
        None
    };

    let global_semaphore = Arc::new(Semaphore::new(config.max_concurrency as usize));
    let in_flight = Arc::new(AtomicU64::new(0));

    let state_tx = Arc::new(state_tx);
    let handle = CrawlHandle {
        crawl_id: crawl_id.clone(),
        config: config.clone(),
        state_tx: state_tx.clone(),
        state_rx: state_rx.clone(),
        stats: stats.clone(),
        frontier: frontier.clone(),
        started_at: Instant::now(),
    };

    // Spawn the orchestrator loop.
    let orchestrator_state_rx = state_rx.clone();
    let started_at = handle.started_at;
    tokio::spawn(async move {
        tracing::info!(%crawl_id, "Crawl orchestrator started");

        // --- Sitemap discovery (before main crawl loop) ---
        if config.enable_sitemap_discovery {
            // Ensure robots.txt is fetched so we can extract Sitemap: directives.
            {
                let mut rc = robots_cache.lock().await;
                if !rc.has_cached(&root_domain) {
                    rc.fetch_and_cache_with_scheme(&root_domain, &seed_scheme, fetcher.client())
                        .await
                        .ok();
                }
            }

            let robots_sitemaps = {
                let rc = robots_cache.lock().await;
                rc.sitemaps(&root_domain)
            };

            let sitemap_urls = crate::crawler::sitemap::discover_sitemaps(
                &root_domain,
                &seed_scheme,
                fetcher.client(),
                &robots_sitemaps,
            )
            .await;

            if !sitemap_urls.is_empty() {
                let entries =
                    crate::crawler::sitemap::fetch_all_sitemaps(&sitemap_urls, fetcher.client())
                        .await;

                tracing::info!(count = entries.len(), "Sitemap entries discovered");

                // Store sitemap URLs in the database.
                let sitemap_rows: Vec<SitemapUrlRow> = entries
                    .iter()
                    .map(|e| SitemapUrlRow {
                        id: 0,
                        crawl_id: crawl_id.clone(),
                        url: e.url.clone(),
                        lastmod: e.lastmod.clone(),
                        changefreq: e.changefreq.clone(),
                        priority: e.priority,
                        source: "sitemap_xml".into(),
                    })
                    .collect();

                if !sitemap_rows.is_empty() {
                    let _ = storage_tx
                        .send(StorageCommand::InsertSitemapUrls(sitemap_rows))
                        .await;
                }

                // Seed frontier with sitemap URLs at elevated priority.
                let mut f = frontier.lock().await;
                for entry in &entries {
                    if let Ok(normalized) = normalize_url(&entry.url, true) {
                        // Apply include/exclude filters to sitemap URLs too.
                        if let Some(final_url) = compiled_patterns.filter_and_rewrite(&normalized) {
                            let hash = hash_url(&final_url);
                            f.push(FrontierEntry {
                                url: final_url,
                                url_hash: hash,
                                depth: 0,
                                priority: 150, // Higher than default 100 - depth
                                source_page_id: None,
                            });
                        }
                    }
                }
            }
        }

        let mut last_emit = Instant::now();
        let mut last_emit_count = 0u64;
        let mut recent_urls: Vec<RecentUrl> = Vec::new();

        loop {
            // Check for stop signal.
            let current_state = *orchestrator_state_rx.borrow();
            if current_state == CrawlState::Stopped {
                tracing::info!(%crawl_id, "Crawl stopped by user");
                break;
            }

            // Handle pause — wait until state changes.
            if current_state == CrawlState::Paused {
                tracing::info!(%crawl_id, "Crawl paused");
                let mut rx = orchestrator_state_rx.clone();
                loop {
                    if rx.changed().await.is_err() {
                        break;
                    }
                    let s = *rx.borrow();
                    if s != CrawlState::Paused {
                        break;
                    }
                }
                if *orchestrator_state_rx.borrow() == CrawlState::Stopped {
                    break;
                }
                tracing::info!(%crawl_id, "Crawl resumed");
                continue;
            }

            // Check max_pages limit.
            if config.max_pages > 0
                && stats.urls_crawled.load(Ordering::SeqCst) >= config.max_pages as u64
            {
                tracing::info!(%crawl_id, "Max pages limit reached");
                break;
            }

            // Dequeue from frontier.
            let entry = {
                let mut f = frontier.lock().await;
                f.pop()
            };

            let entry = match entry {
                Some(e) => e,
                None => {
                    if in_flight.load(Ordering::SeqCst) == 0 {
                        tracing::info!(%crawl_id, "Frontier empty, crawl complete");
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }
            };

            // Mark as in-flight immediately to prevent premature completion.
            in_flight.fetch_add(1, Ordering::SeqCst);

            // Check robots.txt for this domain (skip fetch entirely if disabled).
            let domain = extract_domain(&entry.url);
            if config.respect_robots_txt {
                let mut rc = robots_cache.lock().await;
                if !rc.has_cached(&domain) {
                    rc.fetch_and_cache_with_scheme(&domain, &seed_scheme, fetcher.client())
                        .await
                        .ok();
                    if let Some(delay) = rc.crawl_delay(&domain) {
                        politeness.set_domain_delay(&domain, delay).await;
                    }
                }
                if !rc.is_allowed(&entry.url) {
                    in_flight.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }
            }

            // Wait for politeness delay.
            politeness.wait_for_politeness(&domain).await;

            // Acquire global concurrency permit.
            let permit = global_semaphore.clone().acquire_owned().await;
            let Ok(permit) = permit else {
                in_flight.fetch_sub(1, Ordering::SeqCst);
                break; // Semaphore closed
            };

            // Acquire per-host permit.
            let host_sem = politeness.acquire_host_permit(&domain).await;
            let host_permit = host_sem.acquire_owned().await;
            let Ok(host_permit) = host_permit else {
                in_flight.fetch_sub(1, Ordering::SeqCst);
                continue;
            };

            // Spawn a task for this URL: fetch → parse → store.
            let task_fetcher = fetcher.clone();
            let task_frontier = frontier.clone();
            let task_storage_tx = storage_tx.clone();
            let task_rule_registry = rule_registry.clone();
            let task_crawl_id = crawl_id.clone();
            let task_root_domain = root_domain.clone();
            let task_config = config.clone();
            let task_in_flight = in_flight.clone();
            let task_stats = stats.clone();
            let task_patterns = compiled_patterns.clone();
            let task_ext_tx = external_checker_tx.clone();

            tokio::spawn(async move {
                let _permit = permit;
                let _host_permit = host_permit;

                // --- Fetch ---
                let fetch_result = match task_fetcher.fetch(&entry.url).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!(url = %entry.url, error = %e, "Fetch failed");
                        task_stats.urls_errored.fetch_add(1, Ordering::SeqCst);

                        let url_hash = entry.url_hash.to_vec();
                        let _ = task_storage_tx
                            .send(StorageCommand::UpsertPage {
                                page: Box::new(PageRow {
                                    id: 0,
                                    crawl_id: task_crawl_id.clone(),
                                    url: entry.url.clone(),
                                    depth: entry.depth as i32,
                                    status_code: None,
                                    content_type: None,
                                    response_time_ms: None,
                                    body_size: None,
                                    title: None,
                                    meta_desc: None,
                                    h1: None,
                                    canonical: None,
                                    robots_directives: None,
                                    state: "errored".into(),
                                    fetched_at: Some(chrono::Utc::now().to_rfc3339()),
                                    error_message: Some(e.to_string()),
                                    custom_extractions: None,
                                }),
                                url_hash,
                            })
                            .await;

                        task_in_flight.fetch_sub(1, Ordering::SeqCst);
                        return;
                    }
                };

                task_stats.urls_crawled.fetch_add(1, Ordering::SeqCst);

                let is_html = fetch_result
                    .content_type
                    .as_deref()
                    .map(|ct| ct.contains("text/html"))
                    .unwrap_or(false);

                let status_code = fetch_result.status_code;

                if is_html && (200..300).contains(&(status_code as i32)) {
                    // --- Parse on rayon ---
                    let body = fetch_result.body_bytes.clone();
                    let final_url = fetch_result.final_url.clone();
                    let fetch_body_size = fetch_result.body_size;
                    let fetch_response_time_ms = fetch_result.response_time_ms;
                    let rd = task_root_domain.clone();
                    let registry = task_rule_registry.clone();

                    let (tx, rx) = tokio::sync::oneshot::channel();
                    rayon::spawn(move || {
                        let mut page = parser::parse_html(&body, &final_url, &rd);
                        page.body_size = Some(fetch_body_size);
                        page.response_time_ms = Some(fetch_response_time_ms);
                        let ctx = CrawlContext {
                            root_domain: rd,
                            cross_page_available: false,
                        };
                        let issues = registry.evaluate_page(&page, &ctx);
                        let _ = tx.send((page, issues));
                    });

                    let Ok((parsed_page, issues)) = rx.await else {
                        task_in_flight.fetch_sub(1, Ordering::SeqCst);
                        return;
                    };

                    // --- Enqueue discovered links ---
                    if entry.depth < task_config.max_depth {
                        let mut f = task_frontier.lock().await;
                        for link in &parsed_page.links {
                            if link.is_internal && !link.is_nofollow {
                                if let Ok(normalized) = normalize_url(&link.href, true) {
                                    // Apply URL rewrite rules, then include/exclude filters.
                                    let url_to_enqueue =
                                        task_patterns.filter_and_rewrite(&normalized);
                                    if let Some(final_url) = url_to_enqueue {
                                        let hash = hash_url(&final_url);
                                        f.push(FrontierEntry {
                                            url: final_url,
                                            url_hash: hash,
                                            depth: entry.depth + 1,
                                            priority: 100i32.saturating_sub(entry.depth as i32 + 1),
                                            source_page_id: None,
                                        });
                                    }
                                }
                            }
                        }
                    }

                    // --- Custom CSS extraction (second parse, only when selectors configured) ---
                    let custom_extractions = if !task_config.custom_css_selectors.is_empty() {
                        let selectors = task_config.custom_css_selectors.clone();
                        let body_for_css = fetch_result.body_bytes.clone();
                        let (css_tx, css_rx) = tokio::sync::oneshot::channel();
                        rayon::spawn(move || {
                            let result = parser::extract_custom_css(&body_for_css, &selectors);
                            let _ = css_tx.send(result);
                        });
                        css_rx.await.ok().map(|v| v.to_string())
                    } else {
                        None
                    };

                    // --- Send to storage ---
                    let url_hash = entry.url_hash.to_vec();
                    let page_row = PageRow {
                        id: 0,
                        crawl_id: task_crawl_id.clone(),
                        url: entry.url.clone(),
                        depth: entry.depth as i32,
                        status_code: Some(fetch_result.status_code as i32),
                        content_type: fetch_result.content_type.clone(),
                        response_time_ms: Some(fetch_result.response_time_ms as i32),
                        body_size: Some(fetch_result.body_size as i64),
                        title: parsed_page.title.clone(),
                        meta_desc: parsed_page.meta_description.clone(),
                        h1: parsed_page.h1s.first().cloned(),
                        canonical: parsed_page.canonical.clone(),
                        robots_directives: parsed_page.meta_robots.clone(),
                        state: "analyzed".into(),
                        fetched_at: Some(chrono::Utc::now().to_rfc3339()),
                        error_message: None,
                        custom_extractions,
                    };

                    // Build link rows.
                    let link_rows: Vec<LinkRow> = parsed_page
                        .links
                        .iter()
                        .map(|l| LinkRow {
                            id: 0,
                            crawl_id: task_crawl_id.clone(),
                            source_page: 0, // Resolved by storage writer
                            target_url: l.href.clone(),
                            anchor_text: l.anchor_text.clone(),
                            link_type: "a".into(),
                            is_internal: l.is_internal,
                            nofollow: l.is_nofollow,
                        })
                        .collect();

                    // Build issue rows.
                    let issue_rows: Vec<IssueRow> = issues
                        .iter()
                        .map(|i| IssueRow {
                            id: 0,
                            crawl_id: task_crawl_id.clone(),
                            page_id: 0, // Resolved by storage writer
                            rule_id: i.rule_id.clone(),
                            severity: i.severity,
                            category: i.category,
                            message: i.message.clone(),
                            detail_json: i.detail.as_ref().map(|d| d.to_string()),
                        })
                        .collect();

                    // Send external links to the checker (if enabled).
                    if let Some(ref ext_tx) = task_ext_tx {
                        for link in &parsed_page.links {
                            if !link.is_internal {
                                let _ = ext_tx
                                    .send(crate::crawler::external_checker::ExternalLinkEntry {
                                        crawl_id: task_crawl_id.clone(),
                                        source_page_id: 0, // Will be resolved by checker after page insert
                                        target_url: link.href.clone(),
                                    })
                                    .await;
                            }
                        }
                    }

                    // Send page + links + issues together so writer can resolve page ID.
                    let _ = task_storage_tx
                        .send(StorageCommand::StorePage {
                            page: Box::new(page_row),
                            url_hash,
                            links: link_rows,
                            issues: issue_rows,
                        })
                        .await;
                } else {
                    // Non-HTML resource or error status — record metadata only.
                    let url_hash = entry.url_hash.to_vec();
                    let _ = task_storage_tx
                        .send(StorageCommand::UpsertPage {
                            page: Box::new(PageRow {
                                id: 0,
                                crawl_id: task_crawl_id.clone(),
                                url: entry.url.clone(),
                                depth: entry.depth as i32,
                                status_code: Some(fetch_result.status_code as i32),
                                content_type: fetch_result.content_type.clone(),
                                response_time_ms: Some(fetch_result.response_time_ms as i32),
                                body_size: Some(fetch_result.body_size as i64),
                                title: None,
                                meta_desc: None,
                                h1: None,
                                canonical: None,
                                robots_directives: None,
                                state: "fetched".into(),
                                fetched_at: Some(chrono::Utc::now().to_rfc3339()),
                                error_message: None,
                                custom_extractions: None,
                            }),
                            url_hash,
                        })
                        .await;
                }

                task_in_flight.fetch_sub(1, Ordering::SeqCst);
            });

            // --- Throttled progress emission ---
            let now = Instant::now();
            let current_crawled = stats.urls_crawled.load(Ordering::SeqCst);
            if now.duration_since(last_emit) >= Duration::from_millis(250)
                || current_crawled.saturating_sub(last_emit_count) >= 50
            {
                let queued = {
                    let f = frontier.lock().await;
                    f.total_queued()
                };
                let elapsed = started_at.elapsed().as_millis() as u64;
                let elapsed_secs = elapsed as f64 / 1000.0;
                let rps = if elapsed_secs > 0.0 {
                    current_crawled as f64 / elapsed_secs
                } else {
                    0.0
                };

                let progress = CrawlProgress {
                    crawl_id: crawl_id.clone(),
                    urls_crawled: current_crawled,
                    urls_queued: queued,
                    urls_errored: stats.urls_errored.load(Ordering::SeqCst),
                    current_rps: rps,
                    elapsed_ms: elapsed,
                    recent_urls: std::mem::take(&mut recent_urls),
                };
                emitter.emit_progress(&progress);

                last_emit = now;
                last_emit_count = current_crawled;
            }
        }

        // --- Crawl complete ---
        // Close the external checker channel so it drains remaining work.
        drop(external_checker_tx);

        // Persist frontier for potential resume.
        {
            let f = frontier.lock().await;
            if let Err(e) = f.persist() {
                tracing::warn!(error = %e, "Failed to persist frontier");
            }
        }

        // Final stats flush.
        let final_crawled = stats.urls_crawled.load(Ordering::SeqCst);
        let final_errored = stats.urls_errored.load(Ordering::SeqCst);

        if let Err(e) = storage_tx
            .send(StorageCommand::UpdateCrawlStats {
                crawl_id: crawl_id.clone(),
                urls_crawled: final_crawled as i64,
                urls_errored: final_errored as i64,
            })
            .await
        {
            tracing::error!(error = %e, "Failed to send crawl stats to storage writer");
        }

        // Flush all pending writes and wait for confirmation before post-crawl analysis.
        let flush_ok = {
            let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
            if let Err(e) = storage_tx.send(StorageCommand::FlushAck(ack_tx)).await {
                tracing::error!(error = %e, "Failed to send FlushAck to storage writer");
                false
            } else {
                ack_rx.await.is_ok()
            }
        };

        // Post-crawl cross-page analysis. Runs on spawn_blocking because
        // PostCrawlAnalyzer performs synchronous SQLite reads via Database::with_conn.
        if flush_ok {
            let post_db = db.clone();
            let post_crawl_id = crawl_id.clone();
            let post_result = tokio::task::spawn_blocking(move || {
                let analyzer = crate::rules::PostCrawlAnalyzer::new(&post_db, &post_crawl_id);
                analyzer.analyze()
            })
            .await;

            match post_result {
                Ok(Ok(issues)) if !issues.is_empty() => {
                    tracing::info!(count = issues.len(), "Post-crawl analysis found issues");
                    if let Err(e) = storage_tx.send(StorageCommand::InsertIssues(issues)).await {
                        tracing::error!(error = %e, "Failed to persist post-crawl issues");
                    }
                    if let Err(e) = storage_tx.send(StorageCommand::Flush).await {
                        tracing::error!(error = %e, "Failed to flush post-crawl issues");
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!(error = %e, "Post-crawl analysis failed");
                }
                Err(e) => {
                    tracing::error!(error = %e, "Post-crawl analysis task panicked");
                }
                _ => {}
            }
        } else {
            tracing::error!("Skipping post-crawl analysis — storage flush did not complete");
        }

        let final_state = match *orchestrator_state_rx.borrow() {
            CrawlState::Stopped => CrawlState::Stopped,
            _ => CrawlState::Completed,
        };
        let final_status = match final_state {
            CrawlState::Stopped => "stopped",
            _ => "completed",
        };

        // Update the watch channel so handle.state() reflects completion.
        let _ = state_tx.send(final_state);

        if let Err(e) = storage_tx
            .send(StorageCommand::CompleteCrawl {
                crawl_id: crawl_id.clone(),
                status: final_status.into(),
            })
            .await
        {
            tracing::error!(error = %e, crawl_id = %crawl_id, "Failed to mark crawl as complete");
        }

        if let Err(e) = storage_tx.send(StorageCommand::Flush).await {
            tracing::error!(error = %e, "Failed to send final flush to storage writer");
        }
        if let Err(e) = storage_tx.send(StorageCommand::Shutdown).await {
            tracing::error!(error = %e, "Failed to send shutdown to storage writer");
        }

        // Emit final progress.
        let queued = {
            let f = frontier.lock().await;
            f.total_queued()
        };
        let elapsed = started_at.elapsed().as_millis() as u64;
        emitter.emit_progress(&CrawlProgress {
            crawl_id: crawl_id.clone(),
            urls_crawled: final_crawled,
            urls_queued: queued,
            urls_errored: final_errored,
            current_rps: 0.0,
            elapsed_ms: elapsed,
            recent_urls: vec![],
        });

        tracing::info!(
            %crawl_id,
            urls_crawled = final_crawled,
            urls_errored = final_errored,
            status = final_status,
            "Crawl orchestrator finished"
        );
    });

    Ok(handle)
}

/// Pre-compiled regex patterns for URL filtering and rewriting.
#[derive(Clone)]
struct CompiledPatterns {
    include: Vec<Regex>,
    exclude: Vec<Regex>,
    rewrite_rules: Vec<(Regex, String)>,
}

impl CompiledPatterns {
    /// Compile patterns from config. Logs warnings for invalid regexes and skips them.
    fn from_config(config: &CrawlConfig) -> Result<Self> {
        let include = config
            .include_patterns
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!(pattern = %p, error = %e, "Invalid include pattern, skipping");
                    None
                }
            })
            .collect();

        let exclude = config
            .exclude_patterns
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!(pattern = %p, error = %e, "Invalid exclude pattern, skipping");
                    None
                }
            })
            .collect();

        let rewrite_rules = config
            .url_rewrite_rules
            .iter()
            .filter_map(|(pattern, replacement)| match Regex::new(pattern) {
                Ok(r) => Some((r, replacement.clone())),
                Err(e) => {
                    tracing::warn!(pattern = %pattern, error = %e, "Invalid rewrite pattern, skipping");
                    None
                }
            })
            .collect();

        Ok(Self {
            include,
            exclude,
            rewrite_rules,
        })
    }

    /// Apply rewrite rules to a URL, then check include/exclude filters.
    /// Returns `Some(rewritten_url)` if the URL should be crawled, `None` if filtered out.
    fn filter_and_rewrite(&self, url: &str) -> Option<String> {
        // Apply rewrite rules in order.
        let mut result = url.to_string();
        for (regex, replacement) in &self.rewrite_rules {
            result = regex
                .replace_all(&result, replacement.as_str())
                .into_owned();
        }

        // Check include patterns: if non-empty, URL must match at least one.
        if !self.include.is_empty() && !self.include.iter().any(|r| r.is_match(&result)) {
            return None;
        }

        // Check exclude patterns: URL must not match any.
        if self.exclude.iter().any(|r| r.is_match(&result)) {
            return None;
        }

        Some(result)
    }
}

/// Extract the hostname from a URL string.
fn extract_domain(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default()
}
