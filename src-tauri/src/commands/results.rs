//! Result query commands: paginated pages, issues, links, detail views.

use serde::{Deserialize, Serialize};
use tauri::State;

use std::sync::Arc;

use crate::storage::db::Database;
use crate::storage::models::*;
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
// Tauri commands
// ---------------------------------------------------------------------------

/// Fetch recent crawls for the dashboard.
#[tauri::command]
pub async fn get_recent_crawls(
    _limit: Option<u32>,
    _db: State<'_, Arc<Database>>,
) -> Result<Vec<CrawlSummary>, String> {
    // TODO(phase-4): Query crawls table ordered by started_at DESC.
    Ok(Vec::new())
}

/// Fetch paginated crawl results (pages table) with sorting and filtering.
#[tauri::command]
pub async fn get_crawl_results(
    _crawl_id: String,
    pagination: PaginationParams,
    _filters: Option<PageFilters>,
    _db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<PageRow>, String> {
    // TODO(phase-4): Build SQL query with LIMIT/OFFSET, ORDER BY, WHERE clauses.
    Ok(PaginatedResponse {
        items: Vec::new(),
        total: 0,
        offset: pagination.offset,
        limit: pagination.limit,
    })
}

/// Fetch summary statistics for a single crawl.
#[tauri::command]
pub async fn get_crawl_summary(
    _crawl_id: String,
    _db: State<'_, Arc<Database>>,
) -> Result<CrawlSummary, String> {
    // TODO(phase-4): Aggregate from crawls + issues tables.
    Err("Not yet implemented — Phase 4".into())
}

/// Fetch full detail for a single page including issues and links.
#[tauri::command]
pub async fn get_page_detail(
    _crawl_id: String,
    _page_id: i64,
    _db: State<'_, Arc<Database>>,
) -> Result<PageDetail, String> {
    // TODO(phase-4): Join pages + issues + links tables.
    Err("Not yet implemented — Phase 4".into())
}

/// Fetch paginated issues for a crawl.
#[tauri::command]
pub async fn get_issues(
    _crawl_id: String,
    pagination: PaginationParams,
    _filters: Option<IssueFilters>,
    _db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<IssueRow>, String> {
    Ok(PaginatedResponse {
        items: Vec::new(),
        total: 0,
        offset: pagination.offset,
        limit: pagination.limit,
    })
}

/// Fetch paginated links for a crawl.
#[tauri::command]
pub async fn get_links(
    _crawl_id: String,
    pagination: PaginationParams,
    _filters: Option<LinkFilters>,
    _db: State<'_, Arc<Database>>,
) -> Result<PaginatedResponse<LinkRow>, String> {
    Ok(PaginatedResponse {
        items: Vec::new(),
        total: 0,
        offset: pagination.offset,
        limit: pagination.limit,
    })
}
