//! Plugin post-processor trait and SQL safety validation.
//!
//! Post-processors run after the core `PostCrawlAnalyzer` and can generate
//! additional issues from aggregate crawl data.

use anyhow::Result;

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

/// Validate that a SQL statement is read-only.
///
/// Returns `true` if the statement appears safe for WASM plugin execution.
/// Rejects statements containing write operations or schema modifications.
pub fn validate_read_only_sql(sql: &str) -> bool {
    let upper = sql.trim().to_uppercase();

    // Must start with SELECT.
    if !upper.starts_with("SELECT") {
        return false;
    }

    // Reject any write or DDL keywords.
    const FORBIDDEN: &[&str] = &[
        "INSERT", "UPDATE", "DELETE", "DROP", "ALTER", "CREATE", "ATTACH", "DETACH", "REPLACE",
        "PRAGMA",
    ];
    for keyword in FORBIDDEN {
        if upper.contains(keyword) {
            return false;
        }
    }

    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_select() {
        assert!(validate_read_only_sql(
            "SELECT * FROM pages WHERE crawl_id = ?1"
        ));
        assert!(validate_read_only_sql(
            "SELECT COUNT(*) FROM issues WHERE rule_id LIKE 'plugin.%'"
        ));
        assert!(validate_read_only_sql("  SELECT url FROM pages  "));
    }

    #[test]
    fn test_rejects_write_operations() {
        assert!(!validate_read_only_sql("INSERT INTO pages VALUES (1)"));
        assert!(!validate_read_only_sql("UPDATE pages SET title = 'x'"));
        assert!(!validate_read_only_sql("DELETE FROM pages WHERE id = 1"));
        assert!(!validate_read_only_sql(
            "SELECT * FROM pages; DELETE FROM pages"
        ));
    }

    #[test]
    fn test_rejects_ddl() {
        assert!(!validate_read_only_sql("DROP TABLE pages"));
        assert!(!validate_read_only_sql("ALTER TABLE pages ADD COLUMN x"));
        assert!(!validate_read_only_sql("CREATE TABLE evil (id INT)"));
        assert!(!validate_read_only_sql("ATTACH DATABASE 'x' AS y"));
        assert!(!validate_read_only_sql("DETACH DATABASE y"));
    }

    #[test]
    fn test_rejects_pragma() {
        assert!(!validate_read_only_sql("PRAGMA table_info(pages)"));
    }

    #[test]
    fn test_rejects_non_select() {
        assert!(!validate_read_only_sql("REPLACE INTO pages VALUES (1)"));
        assert!(!validate_read_only_sql(""));
        assert!(!validate_read_only_sql("EXPLAIN SELECT * FROM pages"));
    }

    #[test]
    fn test_case_insensitive() {
        assert!(!validate_read_only_sql(
            "select * from pages; insert into pages values(1)"
        ));
        assert!(validate_read_only_sql("select * from pages where id = 1"));
    }
}
