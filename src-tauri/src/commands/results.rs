//! Result query commands: paginated pages, issues, links, detail views,
//! site tree, and crawl comparison.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::storage::db::Database;
use crate::storage::models::*;
use crate::storage::queries;
use crate::{RuleCategory, Severity};

/// Maximum items per page to prevent excessive memory usage from buggy clients.
const MAX_PAGE_LIMIT: u64 = 1000;

fn clamp_limit(limit: u64) -> u64 {
    limit.clamp(1, MAX_PAGE_LIMIT)
}

// ---------------------------------------------------------------------------
// Pagination & filtering types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    pub offset: u64,
    pub limit: u64,
    pub sort_by: Option<String>,
    pub sort_dir: Option<SortDirection>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageFilters {
    pub url_search: Option<String>,
    pub status_codes: Option<Vec<u16>>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueFilters {
    pub severity: Option<Severity>,
    pub category: Option<RuleCategory>,
    pub rule_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkFilters {
    pub link_type: Option<String>,
    pub is_internal: Option<bool>,
    pub is_broken: Option<bool>,
    pub anchor_text_missing: Option<bool>,
}

// ---------------------------------------------------------------------------
// Response wrappers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlSummary {
    pub crawl_id: String,
    pub start_url: String,
    pub status: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub urls_crawled: u64,
    pub urls_errored: u64,
    pub issue_counts: IssueCounts,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCounts {
    pub errors: u64,
    pub warnings: u64,
    pub info: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageDetail {
    pub page: PageRow,
    pub issues: Vec<IssueRow>,
    pub inbound_links: Vec<LinkRow>,
    pub outbound_links: Vec<LinkRow>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn crawl_row_to_summary(crawl: &CrawlRow, issue_counts: IssueCounts) -> CrawlSummary {
    CrawlSummary {
        crawl_id: crawl.id.clone(),
        start_url: crawl.start_url.clone(),
        status: crawl.status.clone(),
        started_at: crawl.started_at.clone(),
        completed_at: crawl.completed_at.clone(),
        urls_crawled: crawl.urls_crawled as u64,
        urls_errored: crawl.urls_errored as u64,
        issue_counts,
    }
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Fetch recent crawls for the dashboard.
#[tauri::command]
pub async fn get_recent_crawls(
    limit: Option<u32>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<CrawlSummary>, String> {
    db.with_conn(|conn| {
        let crawls = queries::select_recent_crawls(conn, limit.unwrap_or(20))?;
        let mut summaries = Vec::with_capacity(crawls.len());
        for crawl in &crawls {
            let (errors, warnings, info) = queries::count_issues_by_severity(conn, &crawl.id)?;
            summaries.push(crawl_row_to_summary(
                crawl,
                IssueCounts {
                    errors,
                    warnings,
                    info,
                },
            ));
        }
        Ok(summaries)
    })
    .map_err(|e| e.to_string())
}

/// Fetch paginated crawl results (pages table) with sorting and filtering.
#[tauri::command]
pub async fn get_crawl_results(
    crawl_id: String,
    pagination: PaginationParams,
    filters: Option<PageFilters>,
    db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<PageRow>, String> {
    let sort_desc = matches!(pagination.sort_dir, Some(SortDirection::Desc));

    db.with_conn(|conn| {
        let url_search = filters.as_ref().and_then(|f| f.url_search.as_deref());
        let status_codes_vec = filters.as_ref().and_then(|f| f.status_codes.clone());
        let status_codes = status_codes_vec.as_deref();
        let content_type = filters.as_ref().and_then(|f| f.content_type.as_deref());

        let total =
            queries::count_pages_filtered(conn, &crawl_id, url_search, status_codes, content_type)?;
        let items = queries::select_pages(
            conn,
            &crawl_id,
            pagination.offset as i64,
            clamp_limit(pagination.limit) as i64,
            pagination.sort_by.as_deref(),
            sort_desc,
            url_search,
            status_codes,
            content_type,
        )?;

        Ok(PaginatedResponse {
            items,
            total: total as u64,
            offset: pagination.offset,
            limit: clamp_limit(pagination.limit),
        })
    })
    .map_err(|e| e.to_string())
}

/// Fetch summary statistics for a single crawl.
#[tauri::command]
pub async fn get_crawl_summary(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<CrawlSummary, String> {
    db.with_conn(|conn| {
        let crawl = queries::select_crawl_by_id(conn, &crawl_id)?
            .ok_or_else(|| anyhow::anyhow!("Crawl not found: {}", crawl_id))?;
        let (errors, warnings, info) = queries::count_issues_by_severity(conn, &crawl_id)?;
        Ok(crawl_row_to_summary(
            &crawl,
            IssueCounts {
                errors,
                warnings,
                info,
            },
        ))
    })
    .map_err(|e| e.to_string())
}

/// Fetch full detail for a single page including issues and links.
#[tauri::command]
pub async fn get_page_detail(
    crawl_id: String,
    page_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<PageDetail, String> {
    db.with_conn(|conn| {
        let page = queries::select_page_by_id(conn, &crawl_id, page_id)?
            .ok_or_else(|| anyhow::anyhow!("Page not found: {}", page_id))?;
        let issues = queries::select_issues_for_page(conn, &crawl_id, page_id)?;
        let inbound_links = queries::select_inbound_links(conn, &crawl_id, page_id)?;
        let outbound_links = queries::select_outbound_links(conn, &crawl_id, page_id)?;

        Ok(PageDetail {
            page,
            issues,
            inbound_links,
            outbound_links,
        })
    })
    .map_err(|e| e.to_string())
}

/// Fetch paginated issues for a crawl.
#[tauri::command]
pub async fn get_issues(
    crawl_id: String,
    pagination: PaginationParams,
    filters: Option<IssueFilters>,
    db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<IssueRow>, String> {
    let sort_desc = matches!(pagination.sort_dir, Some(SortDirection::Desc));

    db.with_conn(|conn| {
        let severity_owned = filters
            .as_ref()
            .and_then(|f| f.severity.as_ref())
            .map(|s| s.to_string());
        let category_owned = filters
            .as_ref()
            .and_then(|f| f.category.as_ref())
            .map(|c| c.to_string());
        let severity_str = severity_owned.as_deref();
        let category_str = category_owned.as_deref();
        let rule_id = filters.as_ref().and_then(|f| f.rule_id.as_deref());

        let total = queries::count_issues(conn, &crawl_id, severity_str, category_str, rule_id)?;
        let items = queries::select_issues(
            conn,
            &crawl_id,
            pagination.offset as i64,
            clamp_limit(pagination.limit) as i64,
            pagination.sort_by.as_deref(),
            sort_desc,
            severity_str,
            category_str,
            rule_id,
        )?;

        Ok(PaginatedResponse {
            items,
            total: total as u64,
            offset: pagination.offset,
            limit: clamp_limit(pagination.limit),
        })
    })
    .map_err(|e| e.to_string())
}

/// Fetch paginated links for a crawl.
#[tauri::command]
pub async fn get_links(
    crawl_id: String,
    pagination: PaginationParams,
    filters: Option<LinkFilters>,
    db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<LinkRow>, String> {
    let sort_desc = matches!(pagination.sort_dir, Some(SortDirection::Desc));

    db.with_conn(|conn| {
        let link_type = filters.as_ref().and_then(|f| f.link_type.as_deref());
        let is_internal = filters.as_ref().and_then(|f| f.is_internal);
        let is_broken = filters.as_ref().and_then(|f| f.is_broken);
        let anchor_text_missing = filters.as_ref().and_then(|f| f.anchor_text_missing);

        let total = queries::count_links(
            conn,
            &crawl_id,
            link_type,
            is_internal,
            is_broken,
            anchor_text_missing,
        )?;
        let items = queries::select_links(
            conn,
            &crawl_id,
            pagination.offset as i64,
            clamp_limit(pagination.limit) as i64,
            pagination.sort_by.as_deref(),
            sort_desc,
            link_type,
            is_internal,
            is_broken,
            anchor_text_missing,
        )?;

        Ok(PaginatedResponse {
            items,
            total: total as u64,
            offset: pagination.offset,
            limit: clamp_limit(pagination.limit),
        })
    })
    .map_err(|e| e.to_string())
}

/// Sitemap cross-reference report entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SitemapReportEntry {
    pub url: String,
    /// "in_sitemap_not_crawled", "crawled_not_in_sitemap", or "ok"
    pub status: String,
    pub page_status_code: Option<i32>,
}

/// Fetch sitemap cross-reference report for a crawl.
#[tauri::command]
pub async fn get_sitemap_report(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<SitemapReportEntry>, String> {
    use rusqlite::params;

    db.with_conn(|conn| {
        let mut report = Vec::new();

        // URLs in sitemap but not crawled (or non-200).
        {
            let mut stmt = conn.prepare(queries::SELECT_SITEMAP_NOT_CRAWLED)?;
            let mut rows = stmt.query(params![crawl_id])?;
            while let Some(row) = rows.next()? {
                let url: String = row.get(0)?;
                let status_code: Option<i32> = row.get(1)?;
                report.push(SitemapReportEntry {
                    url,
                    status: "in_sitemap_not_crawled".into(),
                    page_status_code: status_code,
                });
            }
        }

        // Pages crawled but not in sitemap.
        {
            let mut stmt = conn.prepare(queries::SELECT_CRAWLED_NOT_IN_SITEMAP)?;
            let mut rows = stmt.query(params![crawl_id])?;
            while let Some(row) = rows.next()? {
                let _page_id: i64 = row.get(0)?;
                let url: String = row.get(1)?;
                report.push(SitemapReportEntry {
                    url,
                    status: "crawled_not_in_sitemap".into(),
                    page_status_code: Some(200),
                });
            }
        }

        Ok(report)
    })
    .map_err(|e| e.to_string())
}

/// Fetch paginated external link check results.
#[tauri::command]
pub async fn get_external_links(
    crawl_id: String,
    pagination: PaginationParams,
    is_broken: Option<bool>,
    db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<ExternalLinkRow>, String> {
    db.with_conn(|conn| {
        let total = queries::count_external_links(conn, &crawl_id, is_broken)?;
        let items = queries::select_external_links(
            conn,
            &crawl_id,
            pagination.offset as i64,
            clamp_limit(pagination.limit) as i64,
            is_broken,
        )?;

        Ok(PaginatedResponse {
            items,
            total: total as u64,
            offset: pagination.offset,
            limit: clamp_limit(pagination.limit),
        })
    })
    .map_err(|e| format!("{e:#}"))
}

// ---------------------------------------------------------------------------
// Site tree
// ---------------------------------------------------------------------------

/// A node in the hierarchical site tree built from crawled URL paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteTreeNode {
    /// Path segment label (e.g. "blog", "about", or the hostname for root).
    pub label: String,
    /// Full URL if this node corresponds to a crawled page.
    pub url: Option<String>,
    /// Page ID for navigating to page detail.
    pub page_id: Option<i64>,
    /// HTTP status code of the page (None if this path segment was not itself a crawled page).
    pub status_code: Option<i32>,
    /// Number of descendant pages (including self if it is a page).
    pub page_count: u32,
    /// Child nodes sorted alphabetically.
    pub children: Vec<SiteTreeNode>,
}

/// Build a hierarchical site tree from all crawled pages.
#[tauri::command]
pub async fn get_site_tree(
    crawl_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<SiteTreeNode>, String> {
    db.with_conn(|conn| {
        let pages = queries::select_page_tree_data(conn, &crawl_id)?;
        Ok(build_site_tree(&pages))
    })
    .map_err(|e| format!("{e:#}"))
}

/// Build a forest of site tree nodes from flat page data.
///
/// Groups pages by host, then splits each URL path into segments to
/// create a nested tree. Each host becomes a root node.
fn build_site_tree(pages: &[PageTreeEntry]) -> Vec<SiteTreeNode> {
    let mut by_host: HashMap<String, Vec<&PageTreeEntry>> = HashMap::new();
    for page in pages {
        let host = url::Url::parse(&page.url)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_else(|| "unknown".into());
        by_host.entry(host).or_default().push(page);
    }

    let mut roots: Vec<SiteTreeNode> = Vec::new();

    for (host, host_pages) in &by_host {
        let mut root = SiteTreeNode {
            label: host.clone(),
            url: None,
            page_id: None,
            status_code: None,
            page_count: 0,
            children: Vec::new(),
        };

        for page in host_pages {
            let path = url::Url::parse(&page.url)
                .map(|u| u.path().to_string())
                .unwrap_or_else(|_| "/".into());

            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            let mut current_children = &mut root.children;
            for (i, &segment) in segments.iter().enumerate() {
                let is_leaf = i == segments.len() - 1;
                let pos = current_children.iter().position(|n| n.label == segment);
                let idx = match pos {
                    Some(idx) => idx,
                    None => {
                        current_children.push(SiteTreeNode {
                            label: segment.to_string(),
                            url: None,
                            page_id: None,
                            status_code: None,
                            page_count: 0,
                            children: Vec::new(),
                        });
                        current_children.len() - 1
                    }
                };
                if is_leaf {
                    current_children[idx].url = Some(page.url.clone());
                    current_children[idx].page_id = Some(page.page_id);
                    current_children[idx].status_code = page.status_code;
                }
                current_children = &mut current_children[idx].children;
            }

            // Handle root path "/" — assign to the host node itself.
            if segments.is_empty() {
                root.url = Some(page.url.clone());
                root.page_id = Some(page.page_id);
                root.status_code = page.status_code;
            }
        }

        compute_page_counts(&mut root);
        sort_tree(&mut root);
        roots.push(root);
    }

    roots.sort_by(|a, b| a.label.cmp(&b.label));
    roots
}

/// Recursively compute the page_count for each node.
fn compute_page_counts(node: &mut SiteTreeNode) -> u32 {
    let mut count: u32 = if node.page_id.is_some() { 1 } else { 0 };
    for child in &mut node.children {
        count += compute_page_counts(child);
    }
    node.page_count = count;
    count
}

/// Recursively sort children alphabetically.
fn sort_tree(node: &mut SiteTreeNode) {
    node.children.sort_by(|a, b| a.label.cmp(&b.label));
    for child in &mut node.children {
        sort_tree(child);
    }
}

// ---------------------------------------------------------------------------
// Crawl comparison
// ---------------------------------------------------------------------------

/// Summary of differences between two crawls.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlComparisonSummary {
    pub base_crawl: CrawlSummary,
    pub compare_crawl: CrawlSummary,
    pub new_pages: u64,
    pub removed_pages: u64,
    pub changed_status_code: u64,
    pub changed_title: u64,
    pub changed_meta_desc: u64,
    pub new_issues: u64,
    pub resolved_issues: u64,
}

/// Fetch a summary of differences between two crawls.
#[tauri::command]
pub async fn get_comparison_summary(
    base_crawl_id: String,
    compare_crawl_id: String,
    db: State<'_, Arc<Database>>,
) -> Result<CrawlComparisonSummary, String> {
    db.with_conn(|conn| {
        let base = queries::select_crawl_by_id(conn, &base_crawl_id)?
            .ok_or_else(|| anyhow::anyhow!("Base crawl not found: {base_crawl_id}"))?;
        let compare = queries::select_crawl_by_id(conn, &compare_crawl_id)?
            .ok_or_else(|| anyhow::anyhow!("Compare crawl not found: {compare_crawl_id}"))?;

        let (be, bw, bi) = queries::count_issues_by_severity(conn, &base_crawl_id)?;
        let (ce, cw, ci) = queries::count_issues_by_severity(conn, &compare_crawl_id)?;

        let counts = queries::count_comparison_summary(conn, &base_crawl_id, &compare_crawl_id)?;

        Ok(CrawlComparisonSummary {
            base_crawl: crawl_row_to_summary(
                &base,
                IssueCounts {
                    errors: be,
                    warnings: bw,
                    info: bi,
                },
            ),
            compare_crawl: crawl_row_to_summary(
                &compare,
                IssueCounts {
                    errors: ce,
                    warnings: cw,
                    info: ci,
                },
            ),
            new_pages: counts.new_pages,
            removed_pages: counts.removed_pages,
            changed_status_code: counts.changed_status,
            changed_title: counts.changed_title,
            changed_meta_desc: counts.changed_meta,
            new_issues: counts.new_issues,
            resolved_issues: counts.resolved_issues,
        })
    })
    .map_err(|e| format!("{e:#}"))
}

/// Fetch paginated page diff between two crawls.
#[tauri::command]
pub async fn get_page_diffs(
    base_crawl_id: String,
    compare_crawl_id: String,
    pagination: PaginationParams,
    filters: Option<PageDiffFilters>,
    db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<PageDiffRow>, String> {
    db.with_conn(|conn| {
        let diff_type = filters.as_ref().and_then(|f| f.diff_type);
        let url_search = filters.as_ref().and_then(|f| f.url_search.clone());

        let total = queries::count_page_diffs(
            conn,
            &base_crawl_id,
            &compare_crawl_id,
            diff_type,
            url_search.as_deref(),
        )?;
        let items = queries::select_page_diffs(
            conn,
            &base_crawl_id,
            &compare_crawl_id,
            pagination.offset as i64,
            clamp_limit(pagination.limit) as i64,
            diff_type,
            url_search.as_deref(),
        )?;

        Ok(PaginatedResponse {
            items,
            total: total as u64,
            offset: pagination.offset,
            limit: clamp_limit(pagination.limit),
        })
    })
    .map_err(|e| format!("{e:#}"))
}

/// Fetch paginated issue diff between two crawls.
#[tauri::command]
pub async fn get_issue_diffs(
    base_crawl_id: String,
    compare_crawl_id: String,
    pagination: PaginationParams,
    filters: Option<IssueDiffFilters>,
    db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<IssueDiffRow>, String> {
    db.with_conn(|conn| {
        let diff_type = filters.as_ref().and_then(|f| f.diff_type);

        let total = queries::count_issue_diffs(conn, &base_crawl_id, &compare_crawl_id, diff_type)?;
        let items = queries::select_issue_diffs(
            conn,
            &base_crawl_id,
            &compare_crawl_id,
            pagination.offset as i64,
            clamp_limit(pagination.limit) as i64,
            diff_type,
        )?;

        Ok(PaginatedResponse {
            items,
            total: total as u64,
            offset: pagination.offset,
            limit: clamp_limit(pagination.limit),
        })
    })
    .map_err(|e| format!("{e:#}"))
}

/// Fetch paginated metadata diff (title/meta_desc changes) between two crawls.
#[tauri::command]
pub async fn get_metadata_diffs(
    base_crawl_id: String,
    compare_crawl_id: String,
    pagination: PaginationParams,
    filters: Option<MetadataDiffFilters>,
    db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<PageDiffRow>, String> {
    db.with_conn(|conn| {
        let url_search = filters.as_ref().and_then(|f| f.url_search.clone());

        let total = queries::count_metadata_diffs(
            conn,
            &base_crawl_id,
            &compare_crawl_id,
            url_search.as_deref(),
        )?;
        let items = queries::select_metadata_diffs(
            conn,
            &base_crawl_id,
            &compare_crawl_id,
            pagination.offset as i64,
            clamp_limit(pagination.limit) as i64,
            url_search.as_deref(),
        )?;

        Ok(PaginatedResponse {
            items,
            total: total as u64,
            offset: pagination.offset,
            limit: clamp_limit(pagination.limit),
        })
    })
    .map_err(|e| format!("{e:#}"))
}
