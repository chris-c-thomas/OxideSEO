//! Result query commands: paginated pages, issues, links, detail views.

use serde::{Deserialize, Serialize};
use tauri::State;

use std::sync::Arc;

use crate::storage::db::Database;
use crate::storage::models::*;
use crate::storage::queries;
use crate::{RuleCategory, Severity};

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
    pub min_severity: Option<Severity>,
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
    pub redirect_chain: Option<Vec<RedirectHop>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedirectHop {
    pub url: String,
    pub status_code: u16,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn severity_to_str(s: &Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn category_to_str(c: &RuleCategory) -> &'static str {
    match c {
        RuleCategory::Meta => "meta",
        RuleCategory::Content => "content",
        RuleCategory::Links => "links",
        RuleCategory::Images => "images",
        RuleCategory::Performance => "performance",
        RuleCategory::Security => "security",
        RuleCategory::Indexability => "indexability",
        RuleCategory::Structured => "structured",
        RuleCategory::International => "international",
    }
}

fn crawl_row_to_summary(
    crawl: &CrawlRow,
    issue_counts: IssueCounts,
) -> CrawlSummary {
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
            let (errors, warnings, info) =
                queries::count_issues_by_severity(conn, &crawl.id)?;
            summaries.push(crawl_row_to_summary(
                crawl,
                IssueCounts { errors, warnings, info },
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

        let total = queries::count_pages_filtered(
            conn, &crawl_id, url_search, status_codes, content_type,
        )?;
        let items = queries::select_pages(
            conn,
            &crawl_id,
            pagination.offset as i64,
            pagination.limit as i64,
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
            limit: pagination.limit,
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
        let (errors, warnings, info) =
            queries::count_issues_by_severity(conn, &crawl_id)?;
        Ok(crawl_row_to_summary(
            &crawl,
            IssueCounts { errors, warnings, info },
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
            redirect_chain: None,
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
        let severity_str = filters.as_ref()
            .and_then(|f| f.severity.as_ref())
            .map(severity_to_str);
        let category_str = filters.as_ref()
            .and_then(|f| f.category.as_ref())
            .map(category_to_str);
        let rule_id = filters.as_ref().and_then(|f| f.rule_id.as_deref());

        let total = queries::count_issues(
            conn, &crawl_id, severity_str, category_str, rule_id,
        )?;
        let items = queries::select_issues(
            conn,
            &crawl_id,
            pagination.offset as i64,
            pagination.limit as i64,
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
            limit: pagination.limit,
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
            conn, &crawl_id, link_type, is_internal, is_broken, anchor_text_missing,
        )?;
        let items = queries::select_links(
            conn,
            &crawl_id,
            pagination.offset as i64,
            pagination.limit as i64,
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
            limit: pagination.limit,
        })
    })
    .map_err(|e| e.to_string())
}
