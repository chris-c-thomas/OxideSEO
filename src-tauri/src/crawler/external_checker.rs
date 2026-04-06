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

use crate::storage::models::{ExternalLinkRow, IssueRow, StorageCommand};
use crate::{RuleCategory, Severity};

/// An external link discovered during parsing, pending verification.
#[derive(Debug, Clone)]
pub struct ExternalLinkEntry {
    pub crawl_id: String,
    pub source_page_id: i64,
    pub target_url: String,
}

/// Spawn the external link checker as a tokio task.
///
/// Receives entries via `rx`, deduplicates by target URL, performs HEAD requests,
/// and sends results to `storage_tx`. Returns when the channel is closed.
pub async fn run_external_checker(
    mut rx: mpsc::Receiver<ExternalLinkEntry>,
    storage_tx: mpsc::Sender<StorageCommand>,
    concurrency: u32,
) {
    let client = reqwest::Client::builder()
        .user_agent("OxideSEO/0.1 (external-link-check)")
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let semaphore = Arc::new(Semaphore::new(concurrency as usize));
    let mut seen: HashSet<String> = HashSet::new();

    // Per-domain last-request tracking for politeness (2s delay).
    let domain_delays: Arc<tokio::sync::Mutex<std::collections::HashMap<String, Instant>>> =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

    let mut tasks = Vec::new();

    while let Some(entry) = rx.recv().await {
        // Deduplicate by target URL.
        if !seen.insert(entry.target_url.clone()) {
            continue;
        }

        let permit = semaphore.clone().acquire_owned().await;
        let Ok(permit) = permit else { break };

        let client = client.clone();
        let stx = storage_tx.clone();
        let delays = domain_delays.clone();

        tasks.push(tokio::spawn(async move {
            let _permit = permit;

            // Per-domain politeness: 2-second minimum between requests.
            let domain = url::Url::parse(&entry.target_url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .unwrap_or_default();

            {
                let mut map = delays.lock().await;
                if let Some(last) = map.get(&domain) {
                    let elapsed = last.elapsed();
                    if elapsed < Duration::from_secs(2) {
                        tokio::time::sleep(Duration::from_secs(2) - elapsed).await;
                    }
                }
                map.insert(domain, Instant::now());
            }

            let start = Instant::now();
            let (status_code, error_message) = match client.head(&entry.target_url).send().await {
                Ok(resp) => (Some(resp.status().as_u16() as i32), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let response_time_ms = start.elapsed().as_millis() as i32;

            let is_broken = error_message.is_some() || status_code.is_some_and(|code| code >= 400);

            // Store the external link result.
            let _ = stx
                .send(StorageCommand::InsertExternalLinks(vec![ExternalLinkRow {
                    id: 0,
                    crawl_id: entry.crawl_id.clone(),
                    source_page: entry.source_page_id,
                    target_url: entry.target_url.clone(),
                    status_code,
                    response_time_ms: Some(response_time_ms),
                    error_message: error_message.clone(),
                    checked_at: None, // Set by SQL datetime('now')
                }]))
                .await;

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

                let _ = stx
                    .send(StorageCommand::InsertIssues(vec![IssueRow {
                        id: 0,
                        crawl_id: entry.crawl_id,
                        page_id: entry.source_page_id,
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
                    .await;
            }
        }));
    }

    // Wait for all in-flight checks to complete.
    for task in tasks {
        let _ = task.await;
    }
}
