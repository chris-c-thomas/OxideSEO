//! External link checker: HEAD-only requests with deduplication and rate limiting.
//!
//! Runs as a separate pipeline from the main crawl. Receives external link
//! URLs via a channel, deduplicates them, and checks each with a HEAD request.
//! Results are sent back to the storage writer as `ExternalLinkRow` records.
//! Broken links (status >= 400 or network error) also produce issues.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;

use crate::storage::db::Database;
use crate::storage::models::{ExternalLinkRow, IssueRow, StorageCommand};
use crate::{RuleCategory, Severity};

/// An external link discovered during parsing, pending verification.
#[derive(Debug, Clone)]
pub struct ExternalLinkEntry {
    pub crawl_id: String,
    /// URL of the source page (resolved to page ID via DB lookup before insert).
    pub source_page_url: String,
    pub target_url: String,
}

/// Spawn the external link checker as a tokio task.
///
/// Receives entries via `rx`, deduplicates by target URL, performs HEAD requests,
/// and sends results to `storage_tx`. Returns when the channel is closed and all
/// in-flight checks complete.
pub async fn run_external_checker(
    mut rx: mpsc::Receiver<ExternalLinkEntry>,
    storage_tx: mpsc::Sender<StorageCommand>,
    db: Arc<Database>,
    concurrency: u32,
) {
    let client = match reqwest::Client::builder()
        .user_agent("OxideSEO/0.1 (external-link-check)")
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to build external checker HTTP client, using default");
            reqwest::Client::new()
        }
    };

    let semaphore = Arc::new(Semaphore::new(concurrency as usize));
    let mut seen: HashSet<String> = HashSet::new();

    // Per-domain scheduling: stores the earliest allowed time for the next request.
    // Using Instant-based scheduling ensures concurrent tasks for the same domain
    // serialize correctly with the intended 2s gap.
    let domain_schedule: Arc<tokio::sync::Mutex<std::collections::HashMap<String, Instant>>> =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    let mut tasks: JoinSet<()> = JoinSet::new();

    while let Some(entry) = rx.recv().await {
        // Deduplicate by target URL.
        if !seen.insert(entry.target_url.clone()) {
            continue;
        }

        let permit = semaphore.clone().acquire_owned().await;
        let Ok(permit) = permit else { break };

        let client = client.clone();
        let stx = storage_tx.clone();
        let schedule = domain_schedule.clone();
        let task_db = db.clone();

        tasks.spawn(async move {
            let _permit = permit;

            // Per-domain politeness: 2-second minimum between requests.
            // Compute sleep-until time while holding the lock, then release before sleeping.
            let domain = url::Url::parse(&entry.target_url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default();

            let sleep_until = {
                let mut map = schedule.lock().await;
                let now = Instant::now();
                let next_allowed = map
                    .get(&domain)
                    .copied()
                    .map(|t| t.max(now))
                    .unwrap_or(now);
                // Record when the NEXT task for this domain should run.
                map.insert(domain, next_allowed + Duration::from_secs(2));
                next_allowed
            };
            let now = Instant::now();
            if sleep_until > now {
                tokio::time::sleep(sleep_until - now).await;
            }

            // Resolve source page ID from the database by URL.
            let source_page_id = task_db
                .with_conn(|conn| {
                    let id: Option<i64> = conn
                        .query_row(
                            "SELECT id FROM pages WHERE crawl_id = ?1 AND url = ?2",
                            rusqlite::params![entry.crawl_id, entry.source_page_url],
                            |r| r.get(0),
                        )
                        .ok();
                    Ok(id)
                })
                .unwrap_or(None)
                .unwrap_or(0);

            if source_page_id == 0 {
                tracing::warn!(
                    source_url = %entry.source_page_url,
                    target_url = %entry.target_url,
                    "Could not resolve source page ID for external link, skipping"
                );
                return;
            }

            let start = Instant::now();
            let (status_code, error_message) = match client.head(&entry.target_url).send().await {
                Ok(resp) => (Some(resp.status().as_u16() as i32), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let response_time_ms = start.elapsed().as_millis() as i32;

            let is_broken = error_message.is_some() || status_code.is_some_and(|code| code >= 400);

            // Store the external link result.
            if let Err(e) = stx
                .send(StorageCommand::InsertExternalLinks(vec![ExternalLinkRow {
                    id: 0,
                    crawl_id: entry.crawl_id.clone(),
                    source_page: source_page_id,
                    target_url: entry.target_url.clone(),
                    status_code,
                    response_time_ms: Some(response_time_ms),
                    error_message: error_message.clone(),
                    checked_at: None, // Set by SQL datetime('now')
                }]))
                .await
            {
                tracing::error!(target_url = %entry.target_url, error = %e, "Failed to send external link result to storage");
            }

            // If broken, also report as an issue.
            if is_broken {
                let message = if let Some(ref err) = error_message {
                    format!("External link to \"{}\" failed: {}", entry.target_url, err)
                } else {
                    format!(
                        "External link to \"{}\" returned status {}.",
                        entry.target_url,
                        status_code.unwrap_or(0)
                    )
                };

                if let Err(e) = stx
                    .send(StorageCommand::InsertIssues(vec![IssueRow {
                        id: 0,
                        crawl_id: entry.crawl_id,
                        page_id: source_page_id,
                        rule_id: "links.broken_external".into(),
                        severity: Severity::Warning,
                        category: RuleCategory::Links,
                        message,
                        detail_json: Some(
                            serde_json::json!({
                                "target_url": entry.target_url,
                                "status_code": status_code,
                                "error": error_message,
                            })
                            .to_string(),
                        ),
                    }]))
                    .await
                {
                    tracing::error!(error = %e, "Failed to send broken external link issue to storage");
                }
            }
        });

        // Reap completed tasks to keep memory bounded.
        while let Some(result) = tasks.try_join_next() {
            if let Err(e) = result {
                tracing::error!(error = %e, "External link check task panicked");
            }
        }
    }

    // Wait for all remaining in-flight checks to complete.
    while let Some(result) = tasks.join_next().await {
        if let Err(e) = result {
            tracing::error!(error = %e, "External link check task panicked");
        }
    }
}
