//! Prepared SQL statements for all database operations.
//!
//! All queries are parameterized to prevent SQL injection. The storage
//! writer uses these for batched inserts; the command handlers use them
//! for reads with LIMIT/OFFSET pagination.

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
