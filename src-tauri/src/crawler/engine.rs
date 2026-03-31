//! Crawl engine orchestrator.
//!
//! Coordinates the URL frontier, fetch workers, parse pipeline,
//! and storage writer into a cohesive crawl loop.

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use tokio::sync::{mpsc, watch, Semaphore};

use crate::commands::crawl::CrawlConfig;
use crate::crawler::frontier::UrlFrontier;
use crate::storage::db::Database;
use crate::CrawlState;

/// Handle to a running crawl. Used to control lifecycle from Tauri commands.
pub struct CrawlHandle {
    pub crawl_id: String,
    pub config: CrawlConfig,
    /// Sender to signal state transitions (pause, resume, stop).
    state_tx: watch::Sender<CrawlState>,
    /// Receiver to observe current state.
    state_rx: watch::Receiver<CrawlState>,
    /// When the crawl started.
    started_at: Instant,
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
}

/// Spawn the crawl orchestrator as a tokio task.
///
/// Returns a `CrawlHandle` for lifecycle control.
///
/// # Architecture
///
/// ```text
/// Orchestrator ──► Fetch Workers (mpsc) ──► Parse Pool (rayon) ──► Storage Writer (mpsc)
/// ```
pub async fn spawn_crawl(
    crawl_id: String,
    config: CrawlConfig,
    db: Arc<Database>,
    app_handle: tauri::AppHandle,
) -> Result<CrawlHandle> {
    let (state_tx, state_rx) = watch::channel(CrawlState::Running);

    let handle = CrawlHandle {
        crawl_id: crawl_id.clone(),
        config: config.clone(),
        state_tx,
        state_rx: state_rx.clone(),
        started_at: Instant::now(),
    };

    // Global concurrency semaphore.
    let global_semaphore = Arc::new(Semaphore::new(config.max_concurrency as usize));

    // TODO(phase-2): Implement the full orchestration loop:
    //
    // 1. Initialize the URL frontier with the seed URL.
    // 2. Spawn N fetch workers reading from a bounded mpsc channel.
    // 3. Spawn the rayon parse + analyze pipeline.
    // 4. Spawn the dedicated storage writer thread.
    // 5. Main loop:
    //    a. Check state_rx for pause/stop signals.
    //    b. Dequeue URL from frontier.
    //    c. Check robots.txt, enforce politeness delay.
    //    d. Acquire global + per-host semaphore permits.
    //    e. Send URL to fetch worker channel.
    //    f. Receive fetch results → dispatch to parse.
    //    g. Receive parse results → batch to storage writer.
    //    h. Emit crawl://progress event every 250ms or 50 URLs.
    // 6. On completion/stop: persist frontier, update crawl status.

    tokio::spawn(async move {
        tracing::info!(%crawl_id, "Crawl orchestrator started");
        // Orchestrator loop will be implemented in Phase 2.
    });

    Ok(handle)
}
