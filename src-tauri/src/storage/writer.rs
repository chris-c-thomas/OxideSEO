//! Dedicated storage writer thread.
//!
//! Receives [`StorageCommand`] messages via a channel and batches them
//! into SQLite transactions for write throughput. All database writes
//! flow through this single thread to avoid WAL contention.

use std::sync::Arc;

use tracing;

use super::db::Database;
use super::models::StorageCommand;
use super::queries;

/// Spawn the storage writer on a dedicated OS thread.
///
/// Uses `tokio::sync::mpsc::Receiver` with `blocking_recv()` so the
/// thread blocks efficiently while waiting for commands.
///
/// Returns a `JoinHandle` for the writer thread.
pub fn spawn_storage_writer(
    db: Arc<Database>,
    mut rx: tokio::sync::mpsc::Receiver<StorageCommand>,
    batch_size: usize,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut batch: Vec<StorageCommand> = Vec::with_capacity(batch_size);

        loop {
            // Block waiting for the first message.
            match rx.blocking_recv() {
                Some(StorageCommand::Shutdown) => {
                    flush_batch(&db, &mut batch);
                    tracing::info!("Storage writer shutting down");
                    break;
                }
                Some(StorageCommand::Flush) => {
                    flush_batch(&db, &mut batch);
                }
                Some(cmd) => {
                    batch.push(cmd);

                    // Drain remaining available messages without blocking.
                    while batch.len() < batch_size {
                        match rx.try_recv() {
                            Ok(StorageCommand::Shutdown) => {
                                flush_batch(&db, &mut batch);
                                tracing::info!("Storage writer shutting down");
                                return;
                            }
                            Ok(StorageCommand::Flush) => {
                                flush_batch(&db, &mut batch);
                                break;
                            }
                            Ok(cmd) => batch.push(cmd),
                            Err(_) => break, // No more messages available
                        }
                    }

                    // Flush if batch is full.
                    if batch.len() >= batch_size {
                        flush_batch(&db, &mut batch);
                    }
                }
                None => {
                    // Channel closed — flush remaining and exit.
                    flush_batch(&db, &mut batch);
                    break;
                }
            }
        }
    })
}

/// Flush all accumulated commands in a single transaction.
fn flush_batch(db: &Database, batch: &mut Vec<StorageCommand>) {
    if batch.is_empty() {
        return;
    }

    let count = batch.len();
    let result = db.with_conn_mut(|conn| {
        let tx = conn.transaction()?;

        for cmd in batch.drain(..) {
            match cmd {
                StorageCommand::UpsertPage { page, url_hash } => {
                    if let Err(e) = queries::upsert_page(&tx, &page, &url_hash) {
                        tracing::warn!(url = %page.url, error = %e, "Failed to upsert page");
                    }
                }
                StorageCommand::StorePage {
                    page,
                    url_hash,
                    links,
                    issues,
                } => match queries::upsert_page(&tx, &page, &url_hash) {
                    Ok(page_id) => {
                        for mut link in links {
                            link.source_page = page_id;
                            if let Err(e) = queries::insert_link(&tx, &link) {
                                tracing::warn!(error = %e, "Failed to insert link");
                            }
                        }
                        for mut issue in issues {
                            issue.page_id = page_id;
                            if let Err(e) = queries::insert_issue(&tx, &issue) {
                                tracing::warn!(error = %e, "Failed to insert issue");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(url = %page.url, error = %e, "Failed to upsert page");
                    }
                },
                StorageCommand::InsertLinks(links) => {
                    for link in &links {
                        if let Err(e) = queries::insert_link(&tx, link) {
                            tracing::warn!(
                                target_url = %link.target_url,
                                error = %e,
                                "Failed to insert link"
                            );
                        }
                    }
                }
                StorageCommand::InsertIssues(issues) => {
                    for issue in &issues {
                        if let Err(e) = queries::insert_issue(&tx, issue) {
                            tracing::warn!(
                                rule_id = %issue.rule_id,
                                error = %e,
                                "Failed to insert issue"
                            );
                        }
                    }
                }
                StorageCommand::UpdateCrawlStats {
                    crawl_id,
                    urls_crawled,
                    urls_errored,
                } => {
                    if let Err(e) =
                        queries::update_crawl_stats(&tx, &crawl_id, urls_crawled, urls_errored)
                    {
                        tracing::warn!(crawl_id = %crawl_id, error = %e, "Failed to update stats");
                    }
                }
                StorageCommand::CompleteCrawl { crawl_id, status } => {
                    if let Err(e) = queries::update_crawl_status(&tx, &crawl_id, &status) {
                        tracing::warn!(
                            crawl_id = %crawl_id,
                            error = %e,
                            "Failed to update crawl status"
                        );
                    }
                }
                StorageCommand::Flush | StorageCommand::Shutdown => {
                    // Already handled at the receive level; shouldn't appear in batch.
                }
            }
        }

        tx.commit()?;
        Ok(())
    });

    match result {
        Ok(()) => {
            tracing::debug!(count, "Storage writer flushed batch");
        }
        Err(e) => {
            tracing::error!(error = %e, count, "Storage writer batch flush failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::models::{IssueRow, LinkRow, PageRow};

    fn test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    fn make_page(crawl_id: &str, url: &str) -> PageRow {
        PageRow {
            id: 0,
            crawl_id: crawl_id.to_string(),
            url: url.to_string(),
            depth: 0,
            status_code: Some(200),
            content_type: Some("text/html".into()),
            response_time_ms: Some(100),
            body_size: Some(5000),
            title: Some("Test Page".into()),
            meta_desc: Some("A test page".into()),
            h1: Some("Hello".into()),
            canonical: None,
            robots_directives: None,
            state: "analyzed".into(),
            fetched_at: Some("2026-04-03T00:00:00Z".into()),
            error_message: None,
        }
    }

    #[tokio::test]
    async fn test_writer_upsert_and_flush() {
        let db = Arc::new(test_db());
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Insert a crawl record first (pages have FK to crawls).
        db.with_conn(|conn| {
            queries::insert_crawl(
                conn,
                &crate::storage::models::CrawlRow {
                    id: "test-1".into(),
                    start_url: "https://example.com".into(),
                    config_json: "{}".into(),
                    status: "running".into(),
                    started_at: None,
                    completed_at: None,
                    urls_crawled: 0,
                    urls_errored: 0,
                },
            )
        })
        .unwrap();

        let handle = spawn_storage_writer(db.clone(), rx, 100);

        // Send page upserts
        let hash1 = blake3::hash(b"https://example.com/").as_bytes().to_vec();
        let hash2 = blake3::hash(b"https://example.com/about")
            .as_bytes()
            .to_vec();

        tx.send(StorageCommand::UpsertPage {
            page: Box::new(make_page("test-1", "https://example.com/")),
            url_hash: hash1,
        })
        .await
        .unwrap();

        tx.send(StorageCommand::UpsertPage {
            page: Box::new(make_page("test-1", "https://example.com/about")),
            url_hash: hash2,
        })
        .await
        .unwrap();

        // Flush and shut down
        tx.send(StorageCommand::Flush).await.unwrap();
        tx.send(StorageCommand::Shutdown).await.unwrap();
        handle.join().unwrap();

        // Verify pages were written
        let count = db
            .with_conn(|conn| queries::count_pages(conn, "test-1"))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_writer_links_and_issues() {
        let db = Arc::new(test_db());
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // Insert crawl + page
        db.with_conn(|conn| {
            queries::insert_crawl(
                conn,
                &crate::storage::models::CrawlRow {
                    id: "test-2".into(),
                    start_url: "https://example.com".into(),
                    config_json: "{}".into(),
                    status: "running".into(),
                    started_at: None,
                    completed_at: None,
                    urls_crawled: 0,
                    urls_errored: 0,
                },
            )?;
            let hash = blake3::hash(b"https://example.com/").as_bytes().to_vec();
            queries::upsert_page(conn, &make_page("test-2", "https://example.com/"), &hash)?;
            Ok(())
        })
        .unwrap();

        let handle = spawn_storage_writer(db.clone(), rx, 100);

        // Send links
        tx.send(StorageCommand::InsertLinks(vec![LinkRow {
            id: 0,
            crawl_id: "test-2".into(),
            source_page: 1,
            target_url: "https://example.com/about".into(),
            anchor_text: Some("About".into()),
            link_type: "a".into(),
            is_internal: true,
            nofollow: false,
        }]))
        .await
        .unwrap();

        // Send issues
        tx.send(StorageCommand::InsertIssues(vec![IssueRow {
            id: 0,
            crawl_id: "test-2".into(),
            page_id: 1,
            rule_id: "meta.title_length".into(),
            severity: "warning".into(),
            category: "meta".into(),
            message: "Title too short".into(),
            detail_json: None,
        }]))
        .await
        .unwrap();

        // Update stats
        tx.send(StorageCommand::UpdateCrawlStats {
            crawl_id: "test-2".into(),
            urls_crawled: 1,
            urls_errored: 0,
        })
        .await
        .unwrap();

        tx.send(StorageCommand::Shutdown).await.unwrap();
        handle.join().unwrap();

        // Verify
        db.with_conn(|conn| {
            let links: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM links WHERE crawl_id = 'test-2'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(links, 1);

            let issues: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM issues WHERE crawl_id = 'test-2'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(issues, 1);

            let crawled: i64 = conn
                .query_row(
                    "SELECT urls_crawled FROM crawls WHERE id = 'test-2'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(crawled, 1);

            Ok(())
        })
        .unwrap();
    }

    #[tokio::test]
    async fn test_writer_shutdown_flushes_remaining() {
        let db = Arc::new(test_db());
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        db.with_conn(|conn| {
            queries::insert_crawl(
                conn,
                &crate::storage::models::CrawlRow {
                    id: "test-3".into(),
                    start_url: "https://example.com".into(),
                    config_json: "{}".into(),
                    status: "running".into(),
                    started_at: None,
                    completed_at: None,
                    urls_crawled: 0,
                    urls_errored: 0,
                },
            )
        })
        .unwrap();

        let handle = spawn_storage_writer(db.clone(), rx, 500); // Large batch size

        // Send a page but NO explicit Flush — Shutdown should flush it.
        let hash = blake3::hash(b"https://example.com/").as_bytes().to_vec();
        tx.send(StorageCommand::UpsertPage {
            page: Box::new(make_page("test-3", "https://example.com/")),
            url_hash: hash,
        })
        .await
        .unwrap();

        tx.send(StorageCommand::Shutdown).await.unwrap();
        handle.join().unwrap();

        let count = db
            .with_conn(|conn| queries::count_pages(conn, "test-3"))
            .unwrap();
        assert_eq!(count, 1);
    }
}
