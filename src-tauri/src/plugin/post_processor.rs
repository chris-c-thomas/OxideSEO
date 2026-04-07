//! Plugin post-processor trait and SQL safety validation.
//!
//! Post-processors run after the core `PostCrawlAnalyzer` and can generate
//! additional issues from aggregate crawl data.

use anyhow::Result;
use rusqlite::Connection;

use crate::storage::db::Database;
use crate::storage::models::IssueRow;

/// Callback type for WASM post-processors to execute read-only SQL.
pub type QueryCallback<'a> = Box<dyn Fn(&str) -> Result<serde_json::Value> + 'a>;

/// Context provided to post-processor plugins.
pub enum PostProcessorContext<'a> {
    /// WASM plugins get a callback that validates and executes read-only SQL.
    QueryFn(QueryCallback<'a>),
    /// Native plugins get direct database access.
    Database(&'a Database),
}

/// Trait for plugin-provided post-crawl processors.
pub trait PluginPostProcessor: Send + Sync {
    /// Post-processor name for logging and UI display.
    fn name(&self) -> &str;

    /// Run post-crawl analysis and return any issues found.
    fn process(&self, crawl_id: &str, context: PostProcessorContext<'_>) -> Result<Vec<IssueRow>>;
}

/// Validate that a SQL statement is read-only using SQLite's own analysis.
///
/// Prepares the statement and checks `stmt.readonly()`, which is the
/// authoritative way to determine if a statement modifies the database.
/// Falls back to keyword rejection if the statement fails to prepare.
pub fn validate_read_only_sql(conn: &Connection, sql: &str) -> bool {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Reject multi-statement strings (semicolons outside the final position).
    // SQLite's prepare() only parses the first statement, so a trailing
    // write statement would be invisible to readonly().
    let without_trailing = trimmed.trim_end_matches(';').trim();
    if without_trailing.contains(';') {
        return false;
    }

    // SQLite considers ATTACH, DETACH, and some PRAGMAs as "readonly" even
    // though they can modify state. Block these explicitly.
    let upper = trimmed.to_uppercase();
    if upper.starts_with("ATTACH") || upper.starts_with("DETACH") || upper.starts_with("PRAGMA") {
        return false;
    }

    match conn.prepare(trimmed) {
        Ok(stmt) => stmt.readonly(),
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE pages (id INTEGER PRIMARY KEY, url TEXT, updated_at TEXT, title TEXT);
             CREATE TABLE issues (id INTEGER PRIMARY KEY, rule_id TEXT, severity TEXT);",
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_valid_select() {
        let conn = test_conn();
        assert!(validate_read_only_sql(
            &conn,
            "SELECT * FROM pages WHERE id = ?1"
        ));
        assert!(validate_read_only_sql(
            &conn,
            "SELECT COUNT(*) FROM issues WHERE rule_id LIKE 'plugin.%'"
        ));
        assert!(validate_read_only_sql(&conn, "  SELECT url FROM pages  "));
    }

    #[test]
    fn test_column_names_with_keywords_allowed() {
        let conn = test_conn();
        // These were false positives with the old substring approach.
        assert!(validate_read_only_sql(
            &conn,
            "SELECT updated_at FROM pages"
        ));
        assert!(validate_read_only_sql(
            &conn,
            "SELECT id, title FROM pages WHERE updated_at IS NOT NULL"
        ));
    }

    #[test]
    fn test_rejects_write_operations() {
        let conn = test_conn();
        assert!(!validate_read_only_sql(
            &conn,
            "INSERT INTO pages VALUES (1, 'x', 'y', 'z')"
        ));
        assert!(!validate_read_only_sql(
            &conn,
            "UPDATE pages SET title = 'x'"
        ));
        assert!(!validate_read_only_sql(
            &conn,
            "DELETE FROM pages WHERE id = 1"
        ));
    }

    #[test]
    fn test_rejects_multi_statement_injection() {
        let conn = test_conn();
        assert!(!validate_read_only_sql(
            &conn,
            "SELECT * FROM pages; DELETE FROM pages"
        ));
        assert!(!validate_read_only_sql(&conn, "SELECT 1; DROP TABLE pages"));
    }

    #[test]
    fn test_rejects_ddl() {
        let conn = test_conn();
        assert!(!validate_read_only_sql(&conn, "DROP TABLE pages"));
        assert!(!validate_read_only_sql(&conn, "CREATE TABLE evil (id INT)"));
    }

    #[test]
    fn test_rejects_pragma() {
        let conn = test_conn();
        // Most PRAGMAs are not readonly in SQLite.
        assert!(!validate_read_only_sql(&conn, "PRAGMA table_info(pages)"));
    }

    #[test]
    fn test_rejects_empty_and_invalid() {
        let conn = test_conn();
        assert!(!validate_read_only_sql(&conn, ""));
        assert!(!validate_read_only_sql(&conn, "   "));
        assert!(!validate_read_only_sql(&conn, "NOT VALID SQL AT ALL"));
    }

    #[test]
    fn test_case_insensitive() {
        let conn = test_conn();
        assert!(validate_read_only_sql(
            &conn,
            "select * from pages where id = 1"
        ));
        assert!(!validate_read_only_sql(
            &conn,
            "select * from pages; insert into pages values(1, 'a', 'b', 'c')"
        ));
    }

    #[test]
    fn test_with_cte() {
        let conn = test_conn();
        assert!(validate_read_only_sql(
            &conn,
            "WITH cte AS (SELECT id FROM pages) SELECT * FROM cte"
        ));
    }

    #[test]
    fn test_rejects_attach() {
        let conn = test_conn();
        assert!(!validate_read_only_sql(
            &conn,
            "ATTACH DATABASE ':memory:' AS evil"
        ));
    }
}
