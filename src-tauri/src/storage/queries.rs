//! Prepared SQL statements and execution functions for all database operations.
//!
//! All queries are parameterized to prevent SQL injection. The storage
//! writer uses these for batched inserts; the command handlers use them
//! for reads with LIMIT/OFFSET pagination.

use anyhow::Result;
use rusqlite::{Connection, params, types::Value};

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

/// Fetch a single crawl by ID.
pub const SELECT_CRAWL_BY_ID: &str = r#"
    SELECT id, start_url, config_json, status, started_at, completed_at,
           urls_crawled, urls_errored
    FROM crawls
    WHERE id = ?1
"#;

/// Base query for paginated issues.
pub const SELECT_ISSUES_BASE: &str = r#"
    SELECT id, crawl_id, page_id, rule_id, severity, category, message, detail_json
    FROM issues
    WHERE crawl_id = ?1
"#;

/// Base query for paginated links.
pub const SELECT_LINKS_BASE: &str = r#"
    SELECT id, crawl_id, source_page, target_url, anchor_text,
           link_type, is_internal, nofollow
    FROM links
    WHERE crawl_id = ?1
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

/// Find duplicate H1 headings across a crawl.
pub const SELECT_DUPLICATE_H1S: &str = r#"
    SELECT h1, GROUP_CONCAT(id) as page_ids, COUNT(*) as cnt
    FROM pages
    WHERE crawl_id = ?1 AND h1 IS NOT NULL AND h1 != ''
    GROUP BY h1
    HAVING cnt > 1
"#;

/// Find broken internal links (target page has 4xx/5xx status).
pub const SELECT_BROKEN_INTERNAL_LINKS: &str = r#"
    SELECT l.source_page, l.target_url, p_target.status_code
    FROM links l
    JOIN pages p_target ON p_target.crawl_id = l.crawl_id AND p_target.url = l.target_url
    WHERE l.crawl_id = ?1
      AND l.is_internal = 1
      AND p_target.status_code >= 400
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
        .collect::<std::result::Result<Vec<_>, _>>()?;
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
    Ok(rows.next().transpose()?)
}

/// Allowlisted sort column for pages.
fn pages_sort_col(sort_by: Option<&str>) -> &'static str {
    match sort_by {
        Some("url") => "url",
        Some("statusCode") => "status_code",
        Some("title") => "title",
        Some("responseTimeMs") => "response_time_ms",
        Some("bodySize") => "body_size",
        Some("depth") => "depth",
        Some("state") => "state",
        _ => "id",
    }
}

/// Build dynamic WHERE clauses and parameter values for page filters.
fn build_page_filters(
    url_search: Option<&str>,
    status_codes: Option<&[u16]>,
    content_type: Option<&str>,
) -> (String, Vec<Value>) {
    let mut clauses = String::new();
    let mut values: Vec<Value> = Vec::new();

    if let Some(search) = url_search {
        if !search.is_empty() {
            clauses.push_str(" AND url LIKE ?");
            values.push(Value::Text(format!("%{}%", search)));
        }
    }

    if let Some(codes) = status_codes {
        if !codes.is_empty() {
            let placeholders: Vec<&str> = codes.iter().map(|_| "?").collect();
            clauses.push_str(&format!(" AND status_code IN ({})", placeholders.join(",")));
            for &code in codes {
                values.push(Value::Integer(code as i64));
            }
        }
    }

    if let Some(ct) = content_type {
        if !ct.is_empty() {
            clauses.push_str(" AND content_type LIKE ?");
            values.push(Value::Text(format!("{}%", ct)));
        }
    }

    (clauses, values)
}

/// Fetch paginated pages with dynamic ordering and filtering.
#[allow(clippy::too_many_arguments)]
pub fn select_pages(
    conn: &Connection,
    crawl_id: &str,
    offset: i64,
    limit: i64,
    sort_by: Option<&str>,
    sort_desc: bool,
    url_search: Option<&str>,
    status_codes: Option<&[u16]>,
    content_type: Option<&str>,
) -> Result<Vec<PageRow>> {
    let order_col = pages_sort_col(sort_by);
    let dir = if sort_desc { "DESC" } else { "ASC" };
    let (filter_clauses, filter_values) =
        build_page_filters(url_search, status_codes, content_type);

    let query = format!(
        "{}{} ORDER BY {} {} LIMIT ? OFFSET ?",
        SELECT_PAGES_BASE, filter_clauses, order_col, dir
    );

    let mut param_values: Vec<Value> = vec![Value::Text(crawl_id.to_string())];
    param_values.extend(filter_values);
    param_values.push(Value::Integer(limit));
    param_values.push(Value::Integer(offset));

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(&param_values), row_to_page)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count pages with optional filters (for pagination total).
pub fn count_pages_filtered(
    conn: &Connection,
    crawl_id: &str,
    url_search: Option<&str>,
    status_codes: Option<&[u16]>,
    content_type: Option<&str>,
) -> Result<i64> {
    let (filter_clauses, filter_values) =
        build_page_filters(url_search, status_codes, content_type);

    let query = format!(
        "SELECT COUNT(*) FROM pages WHERE crawl_id = ?{}",
        filter_clauses
    );

    let mut param_values: Vec<Value> = vec![Value::Text(crawl_id.to_string())];
    param_values.extend(filter_values);

    let mut stmt = conn.prepare(&query)?;
    let count: i64 = stmt.query_row(rusqlite::params_from_iter(&param_values), |row| row.get(0))?;
    Ok(count)
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
        .collect::<std::result::Result<Vec<_>, _>>()?;
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
        .collect::<std::result::Result<Vec<_>, _>>()?;
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
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Fetch a single crawl by ID.
pub fn select_crawl_by_id(conn: &Connection, crawl_id: &str) -> Result<Option<CrawlRow>> {
    let mut stmt = conn.prepare(SELECT_CRAWL_BY_ID)?;
    let mut rows = stmt.query_map(params![crawl_id], row_to_crawl)?;
    Ok(rows.next().transpose()?)
}

/// Count issues grouped by severity for a crawl. Returns (errors, warnings, info).
pub fn count_issues_by_severity(conn: &Connection, crawl_id: &str) -> Result<(u64, u64, u64)> {
    let mut stmt = conn.prepare(COUNT_ISSUES_BY_SEVERITY)?;
    let rows = stmt.query_map(params![crawl_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    let mut errors: u64 = 0;
    let mut warnings: u64 = 0;
    let mut info: u64 = 0;

    for row in rows.flatten() {
        match row.0.as_str() {
            "error" => errors = row.1 as u64,
            "warning" => warnings = row.1 as u64,
            "info" => info = row.1 as u64,
            _ => {}
        }
    }

    Ok((errors, warnings, info))
}

/// Allowlisted sort column for issues.
fn issues_sort_col(sort_by: Option<&str>) -> &'static str {
    match sort_by {
        Some("severity") => "severity",
        Some("category") => "category",
        Some("ruleId") => "rule_id",
        Some("message") => "message",
        Some("pageId") => "page_id",
        _ => "id",
    }
}

/// Build dynamic WHERE clauses for issue filters.
fn build_issue_filters(
    severity: Option<&str>,
    category: Option<&str>,
    rule_id: Option<&str>,
) -> (String, Vec<Value>) {
    let mut clauses = String::new();
    let mut values: Vec<Value> = Vec::new();

    if let Some(sev) = severity {
        if !sev.is_empty() {
            clauses.push_str(" AND severity = ?");
            values.push(Value::Text(sev.to_string()));
        }
    }

    if let Some(cat) = category {
        if !cat.is_empty() {
            clauses.push_str(" AND category = ?");
            values.push(Value::Text(cat.to_string()));
        }
    }

    if let Some(rid) = rule_id {
        if !rid.is_empty() {
            clauses.push_str(" AND rule_id = ?");
            values.push(Value::Text(rid.to_string()));
        }
    }

    (clauses, values)
}

/// Fetch paginated issues with filtering and sorting.
#[allow(clippy::too_many_arguments)]
pub fn select_issues(
    conn: &Connection,
    crawl_id: &str,
    offset: i64,
    limit: i64,
    sort_by: Option<&str>,
    sort_desc: bool,
    severity: Option<&str>,
    category: Option<&str>,
    rule_id: Option<&str>,
) -> Result<Vec<IssueRow>> {
    let order_col = issues_sort_col(sort_by);
    let dir = if sort_desc { "DESC" } else { "ASC" };
    let (filter_clauses, filter_values) = build_issue_filters(severity, category, rule_id);

    let query = format!(
        "{}{} ORDER BY {} {} LIMIT ? OFFSET ?",
        SELECT_ISSUES_BASE, filter_clauses, order_col, dir
    );

    let mut param_values: Vec<Value> = vec![Value::Text(crawl_id.to_string())];
    param_values.extend(filter_values);
    param_values.push(Value::Integer(limit));
    param_values.push(Value::Integer(offset));

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(&param_values), row_to_issue)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count issues with optional filters.
pub fn count_issues(
    conn: &Connection,
    crawl_id: &str,
    severity: Option<&str>,
    category: Option<&str>,
    rule_id: Option<&str>,
) -> Result<i64> {
    let (filter_clauses, filter_values) = build_issue_filters(severity, category, rule_id);

    let query = format!(
        "SELECT COUNT(*) FROM issues WHERE crawl_id = ?{}",
        filter_clauses
    );

    let mut param_values: Vec<Value> = vec![Value::Text(crawl_id.to_string())];
    param_values.extend(filter_values);

    let mut stmt = conn.prepare(&query)?;
    let count: i64 = stmt.query_row(rusqlite::params_from_iter(&param_values), |row| row.get(0))?;
    Ok(count)
}

/// Allowlisted sort column for links.
fn links_sort_col(sort_by: Option<&str>) -> &'static str {
    match sort_by {
        Some("sourcePage") => "source_page",
        Some("targetUrl") => "target_url",
        Some("linkType") => "link_type",
        Some("anchorText") => "anchor_text",
        _ => "id",
    }
}

/// Build dynamic WHERE clauses for link filters.
fn build_link_filters(
    link_type: Option<&str>,
    is_internal: Option<bool>,
    is_broken: Option<bool>,
    anchor_text_missing: Option<bool>,
) -> (String, Vec<Value>) {
    let mut clauses = String::new();
    let mut values: Vec<Value> = Vec::new();

    if let Some(lt) = link_type {
        if !lt.is_empty() {
            clauses.push_str(" AND link_type = ?");
            values.push(Value::Text(lt.to_string()));
        }
    }

    if let Some(internal) = is_internal {
        clauses.push_str(" AND is_internal = ?");
        values.push(Value::Integer(if internal { 1 } else { 0 }));
    }

    if let Some(true) = anchor_text_missing {
        clauses.push_str(" AND (anchor_text IS NULL OR anchor_text = '')");
    }

    if let Some(broken) = is_broken {
        if broken {
            // Only internal links can be checked for broken status
            clauses.push_str(
                " AND is_internal = 1 AND target_url IN (\
                    SELECT url FROM pages WHERE crawl_id = links.crawl_id AND status_code >= 400\
                 )",
            );
        } else {
            clauses.push_str(
                " AND (is_internal = 0 OR target_url NOT IN (\
                    SELECT url FROM pages WHERE crawl_id = links.crawl_id AND status_code >= 400\
                 ))",
            );
        }
    }

    (clauses, values)
}

/// Fetch paginated links with filtering and sorting.
#[allow(clippy::too_many_arguments)]
pub fn select_links(
    conn: &Connection,
    crawl_id: &str,
    offset: i64,
    limit: i64,
    sort_by: Option<&str>,
    sort_desc: bool,
    link_type: Option<&str>,
    is_internal: Option<bool>,
    is_broken: Option<bool>,
    anchor_text_missing: Option<bool>,
) -> Result<Vec<LinkRow>> {
    let order_col = links_sort_col(sort_by);
    let dir = if sort_desc { "DESC" } else { "ASC" };
    let (filter_clauses, filter_values) =
        build_link_filters(link_type, is_internal, is_broken, anchor_text_missing);

    let query = format!(
        "{}{} ORDER BY {} {} LIMIT ? OFFSET ?",
        SELECT_LINKS_BASE, filter_clauses, order_col, dir
    );

    let mut param_values: Vec<Value> = vec![Value::Text(crawl_id.to_string())];
    param_values.extend(filter_values);
    param_values.push(Value::Integer(limit));
    param_values.push(Value::Integer(offset));

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(&param_values), row_to_link)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Count links with optional filters.
pub fn count_links(
    conn: &Connection,
    crawl_id: &str,
    link_type: Option<&str>,
    is_internal: Option<bool>,
    is_broken: Option<bool>,
    anchor_text_missing: Option<bool>,
) -> Result<i64> {
    let (filter_clauses, filter_values) =
        build_link_filters(link_type, is_internal, is_broken, anchor_text_missing);

    let query = format!(
        "SELECT COUNT(*) FROM links WHERE crawl_id = ?{}",
        filter_clauses
    );

    let mut param_values: Vec<Value> = vec![Value::Text(crawl_id.to_string())];
    param_values.extend(filter_values);

    let mut stmt = conn.prepare(&query)?;
    let count: i64 = stmt.query_row(rusqlite::params_from_iter(&param_values), |row| row.get(0))?;
    Ok(count)
}
