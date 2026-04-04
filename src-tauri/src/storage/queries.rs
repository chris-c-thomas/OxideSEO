//! Prepared SQL statements and execution functions for all database operations.
//!
//! All queries are parameterized to prevent SQL injection. The storage
//! writer uses these for batched inserts; the command handlers use them
//! for reads with LIMIT/OFFSET pagination.

use anyhow::Result;
use rusqlite::{Connection, params};

use super::models::{CrawlRow, IssueRow, LinkRow, PageRow};

// ---------------------------------------------------------------------------
// SQL constants
// ---------------------------------------------------------------------------

/// Insert a new crawl record.
pub const INSERT_CRAWL: &str = r#"
    INSERT INTO crawls (id, start_url, config_json, status, started_at)
    VALUES (?1, ?2, ?3, ?4, datetime('now'))
"#;

/// Update crawl status and completion time.
pub const UPDATE_CRAWL_STATUS: &str = r#"
    UPDATE crawls SET status = ?2, completed_at = datetime('now')
    WHERE id = ?1
"#;

/// Update crawl counters (called per batch flush).
pub const UPDATE_CRAWL_STATS: &str = r#"
    UPDATE crawls
    SET urls_crawled = ?2, urls_errored = ?3
    WHERE id = ?1
"#;

/// Insert or update a page record.
/// Uses INSERT OR REPLACE keyed on (crawl_id, url_hash).
pub const UPSERT_PAGE: &str = r#"
    INSERT INTO pages (crawl_id, url, url_hash, depth, status_code, content_type,
                       response_time_ms, body_size, title, meta_desc, h1, canonical,
                       robots_directives, state, fetched_at, error_message)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
    ON CONFLICT(crawl_id, url_hash) DO UPDATE SET
        status_code = excluded.status_code,
        content_type = excluded.content_type,
        response_time_ms = excluded.response_time_ms,
        body_size = excluded.body_size,
        title = excluded.title,
        meta_desc = excluded.meta_desc,
        h1 = excluded.h1,
        canonical = excluded.canonical,
        robots_directives = excluded.robots_directives,
        state = excluded.state,
        fetched_at = excluded.fetched_at,
        error_message = excluded.error_message
"#;

/// Insert a link record.
pub const INSERT_LINK: &str = r#"
    INSERT INTO links (crawl_id, source_page, target_url, anchor_text, link_type,
                       is_internal, nofollow)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#;

/// Insert an issue record.
pub const INSERT_ISSUE: &str = r#"
    INSERT INTO issues (crawl_id, page_id, rule_id, severity, category, message, detail_json)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#;

/// Fetch recent crawls for dashboard, newest first.
pub const SELECT_RECENT_CRAWLS: &str = r#"
    SELECT id, start_url, config_json, status, started_at, completed_at,
           urls_crawled, urls_errored
    FROM crawls
    ORDER BY started_at DESC
    LIMIT ?1
"#;

/// Count pages for a crawl (used in pagination).
pub const COUNT_PAGES: &str = r#"
    SELECT COUNT(*) FROM pages WHERE crawl_id = ?1
"#;

/// Fetch paginated pages for a crawl.
/// The ORDER BY and WHERE clauses are appended dynamically based on
/// sort/filter parameters. This is the base query.
pub const SELECT_PAGES_BASE: &str = r#"
    SELECT id, crawl_id, url, depth, status_code, content_type,
           response_time_ms, body_size, title, meta_desc, h1, canonical,
           robots_directives, state, fetched_at, error_message
    FROM pages
    WHERE crawl_id = ?1
"#;

/// Fetch a single page by ID.
pub const SELECT_PAGE_BY_ID: &str = r#"
    SELECT id, crawl_id, url, depth, status_code, content_type,
           response_time_ms, body_size, title, meta_desc, h1, canonical,
           robots_directives, state, fetched_at, error_message
    FROM pages
    WHERE crawl_id = ?1 AND id = ?2
"#;

/// Fetch issues for a page.
pub const SELECT_ISSUES_FOR_PAGE: &str = r#"
    SELECT id, crawl_id, page_id, rule_id, severity, category, message, detail_json
    FROM issues
    WHERE crawl_id = ?1 AND page_id = ?2
    ORDER BY severity, category
"#;

/// Count issues for a crawl grouped by severity.
pub const COUNT_ISSUES_BY_SEVERITY: &str = r#"
    SELECT severity, COUNT(*) as cnt
    FROM issues
    WHERE crawl_id = ?1
    GROUP BY severity
"#;

/// Fetch inbound links to a page.
pub const SELECT_INBOUND_LINKS: &str = r#"
    SELECT l.id, l.crawl_id, l.source_page, l.target_url, l.anchor_text,
           l.link_type, l.is_internal, l.nofollow
    FROM links l
    JOIN pages p ON p.crawl_id = l.crawl_id AND p.url = l.target_url
    WHERE l.crawl_id = ?1 AND p.id = ?2
"#;

/// Fetch outbound links from a page.
pub const SELECT_OUTBOUND_LINKS: &str = r#"
    SELECT id, crawl_id, source_page, target_url, anchor_text,
           link_type, is_internal, nofollow
    FROM links
    WHERE crawl_id = ?1 AND source_page = ?2
"#;

/// Find duplicate titles across a crawl.
pub const SELECT_DUPLICATE_TITLES: &str = r#"
    SELECT title, GROUP_CONCAT(id) as page_ids, COUNT(*) as cnt
    FROM pages
    WHERE crawl_id = ?1 AND title IS NOT NULL AND title != ''
    GROUP BY title
    HAVING cnt > 1
"#;

/// Find duplicate meta descriptions across a crawl.
pub const SELECT_DUPLICATE_DESCRIPTIONS: &str = r#"
    SELECT meta_desc, GROUP_CONCAT(id) as page_ids, COUNT(*) as cnt
    FROM pages
    WHERE crawl_id = ?1 AND meta_desc IS NOT NULL AND meta_desc != ''
    GROUP BY meta_desc
    HAVING cnt > 1
"#;

/// Find orphan pages (no inbound internal links).
pub const SELECT_ORPHAN_PAGES: &str = r#"
    SELECT p.id, p.url
    FROM pages p
    WHERE p.crawl_id = ?1
      AND p.id NOT IN (
          SELECT DISTINCT p2.id
          FROM pages p2
          JOIN links l ON l.crawl_id = p2.crawl_id AND l.target_url = p2.url
          WHERE l.is_internal = 1 AND p2.crawl_id = ?1
      )
      AND p.depth > 0
"#;

// ---------------------------------------------------------------------------
// Write execution functions
// ---------------------------------------------------------------------------

/// Insert a new crawl record.
pub fn insert_crawl(conn: &Connection, crawl: &CrawlRow) -> Result<()> {
    conn.execute(
        INSERT_CRAWL,
        params![crawl.id, crawl.start_url, crawl.config_json, crawl.status],
    )?;
    Ok(())
}

/// Update crawl status (completed, stopped, error).
pub fn update_crawl_status(conn: &Connection, crawl_id: &str, status: &str) -> Result<()> {
    conn.execute(UPDATE_CRAWL_STATUS, params![crawl_id, status])?;
    Ok(())
}

/// Update crawl counters.
pub fn update_crawl_stats(
    conn: &Connection,
    crawl_id: &str,
    urls_crawled: i64,
    urls_errored: i64,
) -> Result<()> {
    conn.execute(
        UPDATE_CRAWL_STATS,
        params![crawl_id, urls_crawled, urls_errored],
    )?;
    Ok(())
}

/// Insert or update a page record. Returns the page row id.
pub fn upsert_page(conn: &Connection, page: &PageRow, url_hash: &[u8]) -> Result<i64> {
    conn.execute(
        UPSERT_PAGE,
        params![
            page.crawl_id,
            page.url,
            url_hash,
            page.depth,
            page.status_code,
            page.content_type,
            page.response_time_ms,
            page.body_size,
            page.title,
            page.meta_desc,
            page.h1,
            page.canonical,
            page.robots_directives,
            page.state,
            page.fetched_at,
            page.error_message,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Insert a single link record.
pub fn insert_link(conn: &Connection, link: &LinkRow) -> Result<()> {
    conn.execute(
        INSERT_LINK,
        params![
            link.crawl_id,
            link.source_page,
            link.target_url,
            link.anchor_text,
            link.link_type,
            link.is_internal,
            link.nofollow,
        ],
    )?;
    Ok(())
}

/// Insert a single issue record.
pub fn insert_issue(conn: &Connection, issue: &IssueRow) -> Result<()> {
    conn.execute(
        INSERT_ISSUE,
        params![
            issue.crawl_id,
            issue.page_id,
            issue.rule_id,
            issue.severity,
            issue.category,
            issue.message,
            issue.detail_json,
        ],
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Read execution functions
// ---------------------------------------------------------------------------

/// Helper to read a PageRow from a rusqlite Row.
fn row_to_page(row: &rusqlite::Row) -> rusqlite::Result<PageRow> {
    Ok(PageRow {
        id: row.get(0)?,
        crawl_id: row.get(1)?,
        url: row.get(2)?,
        depth: row.get(3)?,
        status_code: row.get(4)?,
        content_type: row.get(5)?,
        response_time_ms: row.get(6)?,
        body_size: row.get(7)?,
        title: row.get(8)?,
        meta_desc: row.get(9)?,
        h1: row.get(10)?,
        canonical: row.get(11)?,
        robots_directives: row.get(12)?,
        state: row.get(13)?,
        fetched_at: row.get(14)?,
        error_message: row.get(15)?,
    })
}

/// Helper to read a CrawlRow from a rusqlite Row.
fn row_to_crawl(row: &rusqlite::Row) -> rusqlite::Result<CrawlRow> {
    Ok(CrawlRow {
        id: row.get(0)?,
        start_url: row.get(1)?,
        config_json: row.get(2)?,
        status: row.get(3)?,
        started_at: row.get(4)?,
        completed_at: row.get(5)?,
        urls_crawled: row.get(6)?,
        urls_errored: row.get(7)?,
    })
}

/// Helper to read a LinkRow from a rusqlite Row.
fn row_to_link(row: &rusqlite::Row) -> rusqlite::Result<LinkRow> {
    Ok(LinkRow {
        id: row.get(0)?,
        crawl_id: row.get(1)?,
        source_page: row.get(2)?,
        target_url: row.get(3)?,
        anchor_text: row.get(4)?,
        link_type: row.get(5)?,
        is_internal: row.get(6)?,
        nofollow: row.get(7)?,
    })
}

/// Helper to read an IssueRow from a rusqlite Row.
fn row_to_issue(row: &rusqlite::Row) -> rusqlite::Result<IssueRow> {
    Ok(IssueRow {
        id: row.get(0)?,
        crawl_id: row.get(1)?,
        page_id: row.get(2)?,
        rule_id: row.get(3)?,
        severity: row.get(4)?,
        category: row.get(5)?,
        message: row.get(6)?,
        detail_json: row.get(7)?,
    })
}

/// Fetch recent crawls for dashboard.
pub fn select_recent_crawls(conn: &Connection, limit: u32) -> Result<Vec<CrawlRow>> {
    let mut stmt = conn.prepare(SELECT_RECENT_CRAWLS)?;
    let rows = stmt
        .query_map(params![limit], row_to_crawl)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Count total pages for a crawl.
pub fn count_pages(conn: &Connection, crawl_id: &str) -> Result<i64> {
    Ok(conn.query_row(COUNT_PAGES, params![crawl_id], |row| row.get(0))?)
}

/// Fetch a single page by crawl_id + page_id.
pub fn select_page_by_id(
    conn: &Connection,
    crawl_id: &str,
    page_id: i64,
) -> Result<Option<PageRow>> {
    let mut stmt = conn.prepare(SELECT_PAGE_BY_ID)?;
    let mut rows = stmt.query_map(params![crawl_id, page_id], row_to_page)?;
    Ok(rows.next().and_then(|r| r.ok()))
}

/// Fetch paginated pages with dynamic ordering.
pub fn select_pages(
    conn: &Connection,
    crawl_id: &str,
    offset: i64,
    limit: i64,
    sort_by: Option<&str>,
    sort_desc: bool,
) -> Result<Vec<PageRow>> {
    // Allowlist of sortable columns to prevent SQL injection.
    let order_col = match sort_by {
        Some("url") => "url",
        Some("statusCode") => "status_code",
        Some("title") => "title",
        Some("responseTimeMs") => "response_time_ms",
        Some("bodySize") => "body_size",
        Some("depth") => "depth",
        Some("state") => "state",
        _ => "id",
    };
    let dir = if sort_desc { "DESC" } else { "ASC" };
    let query = format!(
        "{} ORDER BY {} {} LIMIT ?2 OFFSET ?3",
        SELECT_PAGES_BASE, order_col, dir
    );

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt
        .query_map(params![crawl_id, limit, offset], row_to_page)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Fetch issues for a specific page.
pub fn select_issues_for_page(
    conn: &Connection,
    crawl_id: &str,
    page_id: i64,
) -> Result<Vec<IssueRow>> {
    let mut stmt = conn.prepare(SELECT_ISSUES_FOR_PAGE)?;
    let rows = stmt
        .query_map(params![crawl_id, page_id], row_to_issue)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Fetch outbound links from a page.
pub fn select_outbound_links(
    conn: &Connection,
    crawl_id: &str,
    page_id: i64,
) -> Result<Vec<LinkRow>> {
    let mut stmt = conn.prepare(SELECT_OUTBOUND_LINKS)?;
    let rows = stmt
        .query_map(params![crawl_id, page_id], row_to_link)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Fetch inbound links to a page.
pub fn select_inbound_links(
    conn: &Connection,
    crawl_id: &str,
    page_id: i64,
) -> Result<Vec<LinkRow>> {
    let mut stmt = conn.prepare(SELECT_INBOUND_LINKS)?;
    let rows = stmt
        .query_map(params![crawl_id, page_id], row_to_link)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}
