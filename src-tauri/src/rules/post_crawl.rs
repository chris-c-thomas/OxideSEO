//! Post-crawl cross-page analysis.
//!
//! Runs after all pages are crawled and stored. Detects issues that require
//! aggregate data: duplicate titles/descriptions/H1s, broken internal links,
//! and orphan pages. Produces `IssueRow` values for insertion via the
//! storage writer.

use anyhow::Result;
use rusqlite::params;

use crate::storage::db::Database;
use crate::storage::models::IssueRow;
use crate::storage::queries;
use crate::{RuleCategory, Severity};

/// Analyzes a completed crawl for cross-page SEO issues.
pub struct PostCrawlAnalyzer<'a> {
    db: &'a Database,
    crawl_id: &'a str,
}

impl<'a> PostCrawlAnalyzer<'a> {
    pub fn new(db: &'a Database, crawl_id: &'a str) -> Self {
        Self { db, crawl_id }
    }

    /// Run all post-crawl cross-page analyses. Returns issues to insert.
    pub fn analyze(&self) -> Result<Vec<IssueRow>> {
        let mut issues = Vec::new();
        issues.extend(self.find_duplicate_titles()?);
        issues.extend(self.find_duplicate_descriptions()?);
        issues.extend(self.find_duplicate_h1s()?);
        issues.extend(self.find_broken_internal_links()?);
        issues.extend(self.find_orphan_pages()?);
        Ok(issues)
    }

    /// Find pages sharing the same title.
    fn find_duplicate_titles(&self) -> Result<Vec<IssueRow>> {
        self.find_duplicates(
            queries::SELECT_DUPLICATE_TITLES,
            "meta.title_duplicate",
            RuleCategory::Meta,
            Severity::Warning,
            "title",
        )
    }

    /// Find pages sharing the same meta description.
    fn find_duplicate_descriptions(&self) -> Result<Vec<IssueRow>> {
        self.find_duplicates(
            queries::SELECT_DUPLICATE_DESCRIPTIONS,
            "meta.desc_duplicate",
            RuleCategory::Meta,
            Severity::Warning,
            "meta description",
        )
    }

    /// Find pages sharing the same H1 heading.
    fn find_duplicate_h1s(&self) -> Result<Vec<IssueRow>> {
        self.find_duplicates(
            queries::SELECT_DUPLICATE_H1S,
            "content.h1_duplicate",
            RuleCategory::Content,
            Severity::Warning,
            "H1",
        )
    }

    /// Generic duplicate finder for title/description/H1 queries.
    ///
    /// Each query returns rows of (value, page_ids_csv, count).
    fn find_duplicates(
        &self,
        sql: &str,
        rule_id: &str,
        category: RuleCategory,
        severity: Severity,
        field_name: &str,
    ) -> Result<Vec<IssueRow>> {
        let crawl_id = self.crawl_id.to_string();
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(sql)?;
            let mut rows = stmt.query(params![crawl_id])?;

            let mut issues = Vec::new();
            while let Some(row) = rows.next()? {
                let value: String = row.get(0)?;
                let page_ids_csv: String = row.get(1)?;
                let count: i64 = row.get(2)?;

                let page_ids: Vec<i64> = page_ids_csv
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();

                let truncated_value = if value.chars().count() > 80 {
                    let truncated: String = value.chars().take(77).collect();
                    format!("{truncated}...")
                } else {
                    value.clone()
                };

                for &page_id in &page_ids {
                    issues.push(IssueRow {
                        id: 0,
                        crawl_id: crawl_id.clone(),
                        page_id,
                        rule_id: rule_id.into(),
                        severity,
                        category,
                        message: format!(
                            "Duplicate {}: \"{}\" is shared by {} page(s).",
                            field_name, truncated_value, count
                        ),
                        detail_json: Some(
                            serde_json::json!({
                                "value": value,
                                "duplicate_page_ids": page_ids,
                                "count": count,
                            })
                            .to_string(),
                        ),
                    });
                }
            }
            Ok(issues)
        })
    }

    /// Find internal links pointing to pages with 4xx/5xx status codes.
    fn find_broken_internal_links(&self) -> Result<Vec<IssueRow>> {
        let crawl_id = self.crawl_id.to_string();
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(queries::SELECT_BROKEN_INTERNAL_LINKS)?;
            let mut rows = stmt.query(params![crawl_id])?;

            let mut issues = Vec::new();
            while let Some(row) = rows.next()? {
                let source_page: i64 = row.get(0)?;
                let target_url: String = row.get(1)?;
                let status_code: i32 = row.get(2)?;

                issues.push(IssueRow {
                    id: 0,
                    crawl_id: crawl_id.clone(),
                    page_id: source_page,
                    rule_id: "links.broken_internal".into(),
                    severity: Severity::Error,
                    category: RuleCategory::Links,
                    message: format!(
                        "Internal link to \"{}\" returned status {}.",
                        target_url, status_code
                    ),
                    detail_json: Some(
                        serde_json::json!({
                            "target_url": target_url,
                            "status_code": status_code,
                        })
                        .to_string(),
                    ),
                });
            }
            Ok(issues)
        })
    }

    /// Find pages with no inbound internal links (excluding the seed URL at depth 0).
    fn find_orphan_pages(&self) -> Result<Vec<IssueRow>> {
        let crawl_id = self.crawl_id.to_string();
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(queries::SELECT_ORPHAN_PAGES)?;
            let mut rows = stmt.query(params![crawl_id])?;

            let mut issues = Vec::new();
            while let Some(row) = rows.next()? {
                let page_id: i64 = row.get(0)?;
                let url: String = row.get(1)?;

                issues.push(IssueRow {
                    id: 0,
                    crawl_id: crawl_id.clone(),
                    page_id,
                    rule_id: "links.orphan_page".into(),
                    severity: Severity::Warning,
                    category: RuleCategory::Links,
                    message: format!("Page \"{}\" has no inbound internal links.", url),
                    detail_json: Some(
                        serde_json::json!({
                            "url": url,
                        })
                        .to_string(),
                    ),
                });
            }
            Ok(issues)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::models::{CrawlRow, LinkRow, PageRow};
    use crate::storage::queries;

    fn test_db() -> Database {
        Database::new_in_memory().unwrap()
    }

    fn insert_crawl(db: &Database, crawl_id: &str) {
        db.with_conn(|conn| {
            queries::insert_crawl(
                conn,
                &CrawlRow {
                    id: crawl_id.into(),
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
    }

    fn insert_page(db: &Database, _crawl_id: &str, url: &str, page: PageRow) -> i64 {
        let hash = blake3::hash(url.as_bytes()).as_bytes().to_vec();
        db.with_conn(|conn| queries::upsert_page(conn, &page, &hash))
            .unwrap()
    }

    fn make_page(crawl_id: &str, url: &str) -> PageRow {
        PageRow {
            id: 0,
            crawl_id: crawl_id.into(),
            url: url.into(),
            depth: 1,
            status_code: Some(200),
            content_type: Some("text/html".into()),
            response_time_ms: Some(100),
            body_size: Some(5000),
            title: Some("Default Title".into()),
            meta_desc: Some("Default description".into()),
            h1: Some("Default H1".into()),
            canonical: None,
            robots_directives: None,
            state: "analyzed".into(),
            fetched_at: Some("2026-04-03T00:00:00Z".into()),
            error_message: None,
        }
    }

    fn insert_link(db: &Database, link: &LinkRow) {
        db.with_conn(|conn| queries::insert_link(conn, link))
            .unwrap();
    }

    #[test]
    fn test_duplicate_titles() {
        let db = test_db();
        insert_crawl(&db, "c1");

        let mut p1 = make_page("c1", "https://example.com/a");
        p1.title = Some("Same Title".into());
        insert_page(&db, "c1", "https://example.com/a", p1);

        let mut p2 = make_page("c1", "https://example.com/b");
        p2.title = Some("Same Title".into());
        insert_page(&db, "c1", "https://example.com/b", p2);

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.find_duplicate_titles().unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().all(|i| i.rule_id == "meta.title_duplicate"));
    }

    #[test]
    fn test_duplicate_descriptions() {
        let db = test_db();
        insert_crawl(&db, "c1");

        let mut p1 = make_page("c1", "https://example.com/a");
        p1.meta_desc = Some("Same desc".into());
        insert_page(&db, "c1", "https://example.com/a", p1);

        let mut p2 = make_page("c1", "https://example.com/b");
        p2.meta_desc = Some("Same desc".into());
        insert_page(&db, "c1", "https://example.com/b", p2);

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.find_duplicate_descriptions().unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().all(|i| i.rule_id == "meta.desc_duplicate"));
    }

    #[test]
    fn test_duplicate_h1s() {
        let db = test_db();
        insert_crawl(&db, "c1");

        let mut p1 = make_page("c1", "https://example.com/a");
        p1.h1 = Some("Same H1".into());
        insert_page(&db, "c1", "https://example.com/a", p1);

        let mut p2 = make_page("c1", "https://example.com/b");
        p2.h1 = Some("Same H1".into());
        insert_page(&db, "c1", "https://example.com/b", p2);

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.find_duplicate_h1s().unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().all(|i| i.rule_id == "content.h1_duplicate"));
    }

    #[test]
    fn test_unique_titles_no_issues() {
        let db = test_db();
        insert_crawl(&db, "c1");

        let mut p1 = make_page("c1", "https://example.com/a");
        p1.title = Some("Title A".into());
        insert_page(&db, "c1", "https://example.com/a", p1);

        let mut p2 = make_page("c1", "https://example.com/b");
        p2.title = Some("Title B".into());
        insert_page(&db, "c1", "https://example.com/b", p2);

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.find_duplicate_titles().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_broken_internal_links() {
        let db = test_db();
        insert_crawl(&db, "c1");

        // Source page (200 OK)
        let p1 = make_page("c1", "https://example.com/a");
        let source_id = insert_page(&db, "c1", "https://example.com/a", p1);

        // Target page (404 Not Found)
        let mut p2 = make_page("c1", "https://example.com/broken");
        p2.status_code = Some(404);
        insert_page(&db, "c1", "https://example.com/broken", p2);

        // Internal link from source to broken target
        insert_link(
            &db,
            &LinkRow {
                id: 0,
                crawl_id: "c1".into(),
                source_page: source_id,
                target_url: "https://example.com/broken".into(),
                anchor_text: Some("Broken link".into()),
                link_type: "a".into(),
                is_internal: true,
                nofollow: false,
            },
        );

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.find_broken_internal_links().unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "links.broken_internal");
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].page_id, source_id);
    }

    #[test]
    fn test_orphan_pages() {
        let db = test_db();
        insert_crawl(&db, "c1");

        // Seed page (depth 0) — should NOT be flagged
        let mut seed = make_page("c1", "https://example.com/");
        seed.depth = 0;
        insert_page(&db, "c1", "https://example.com/", seed);

        // Orphan page (depth 1, no inbound links)
        let orphan = make_page("c1", "https://example.com/orphan");
        insert_page(&db, "c1", "https://example.com/orphan", orphan);

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.find_orphan_pages().unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "links.orphan_page");
        assert!(issues[0].message.contains("orphan"));
    }

    #[test]
    fn test_seed_not_orphan() {
        let db = test_db();
        insert_crawl(&db, "c1");

        // Only a seed page at depth 0 with no inbound links
        let mut seed = make_page("c1", "https://example.com/");
        seed.depth = 0;
        insert_page(&db, "c1", "https://example.com/", seed);

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.find_orphan_pages().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_full_analysis() {
        let db = test_db();
        insert_crawl(&db, "c1");

        // Two pages with duplicate titles
        let mut p1 = make_page("c1", "https://example.com/a");
        p1.title = Some("Dup Title".into());
        p1.meta_desc = Some("Unique A".into());
        p1.h1 = Some("Unique H1 A".into());
        p1.depth = 0;
        let id1 = insert_page(&db, "c1", "https://example.com/a", p1);

        let mut p2 = make_page("c1", "https://example.com/b");
        p2.title = Some("Dup Title".into());
        p2.meta_desc = Some("Unique B".into());
        p2.h1 = Some("Unique H1 B".into());
        insert_page(&db, "c1", "https://example.com/b", p2);

        // Broken internal link target
        let mut p3 = make_page("c1", "https://example.com/gone");
        p3.title = Some("Gone Page".into());
        p3.meta_desc = Some("Unique C".into());
        p3.h1 = Some("Unique H1 C".into());
        p3.status_code = Some(410);
        insert_page(&db, "c1", "https://example.com/gone", p3);

        insert_link(
            &db,
            &LinkRow {
                id: 0,
                crawl_id: "c1".into(),
                source_page: id1,
                target_url: "https://example.com/gone".into(),
                anchor_text: None,
                link_type: "a".into(),
                is_internal: true,
                nofollow: false,
            },
        );

        // Orphan page — no one links to it
        let mut p4 = make_page("c1", "https://example.com/orphan");
        p4.title = Some("Orphan Title".into());
        p4.meta_desc = Some("Unique D".into());
        p4.h1 = Some("Unique H1 D".into());
        insert_page(&db, "c1", "https://example.com/orphan", p4);

        // Link from p1 to p2 and p3 (so they are NOT orphaned)
        insert_link(
            &db,
            &LinkRow {
                id: 0,
                crawl_id: "c1".into(),
                source_page: id1,
                target_url: "https://example.com/b".into(),
                anchor_text: None,
                link_type: "a".into(),
                is_internal: true,
                nofollow: false,
            },
        );

        let analyzer = PostCrawlAnalyzer::new(&db, "c1");
        let issues = analyzer.analyze().unwrap();

        let dup_titles = issues
            .iter()
            .filter(|i| i.rule_id == "meta.title_duplicate")
            .count();
        let broken = issues
            .iter()
            .filter(|i| i.rule_id == "links.broken_internal")
            .count();
        let orphans = issues
            .iter()
            .filter(|i| i.rule_id == "links.orphan_page")
            .count();

        assert_eq!(dup_titles, 2); // Both pages with "Dup Title"
        assert_eq!(broken, 1); // Link to /gone (410)
        assert_eq!(orphans, 1); // /orphan has no inbound internal links
    }
}
