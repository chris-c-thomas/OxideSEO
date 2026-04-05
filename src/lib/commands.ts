/**
 * Typed Tauri IPC command wrappers.
 *
 * Provides a type-safe bridge between the React frontend and the Rust backend.
 * Each function maps to a `#[tauri::command]` in `src-tauri/src/commands/`.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  CrawlConfig,
  CrawlStatus,
  CrawlSummary,
  ExportRequest,
  ExportResult,
  IssueFilters,
  IssueRow,
  LinkFilters,
  LinkRow,
  PageDetail,
  PageFilters,
  PageRow,
  PaginatedResponse,
  PaginationParams,
  RuleConfigOverride,
} from "@/types";

// ---------------------------------------------------------------------------
// Crawl lifecycle
// ---------------------------------------------------------------------------

/** Start a new crawl. Returns the unique crawl ID. */
export function startCrawl(config: CrawlConfig): Promise<string> {
  return invoke<string>("start_crawl", { config });
}

/** Pause an active crawl. In-flight requests will complete. */
export function pauseCrawl(crawlId: string): Promise<void> {
  return invoke<void>("pause_crawl", { crawlId });
}

/** Resume a paused crawl. */
export function resumeCrawl(crawlId: string): Promise<void> {
  return invoke<void>("resume_crawl", { crawlId });
}

/** Stop a crawl entirely. */
export function stopCrawl(crawlId: string): Promise<void> {
  return invoke<void>("stop_crawl", { crawlId });
}

/** Get real-time status of a crawl. */
export function getCrawlStatus(crawlId: string): Promise<CrawlStatus> {
  return invoke<CrawlStatus>("get_crawl_status", { crawlId });
}

// ---------------------------------------------------------------------------
// Result queries
// ---------------------------------------------------------------------------

/** Fetch recent crawls for the dashboard. */
export function getRecentCrawls(limit: number = 20): Promise<CrawlSummary[]> {
  return invoke<CrawlSummary[]>("get_recent_crawls", { limit });
}

/** Fetch paginated page results with optional sorting and filtering. */
export function getCrawlResults(
  crawlId: string,
  pagination: PaginationParams,
  filters?: PageFilters,
): Promise<PaginatedResponse<PageRow>> {
  return invoke<PaginatedResponse<PageRow>>("get_crawl_results", {
    crawlId,
    pagination,
    filters: filters ?? null,
  });
}

/** Fetch summary stats for a crawl. */
export function getCrawlSummary(crawlId: string): Promise<CrawlSummary> {
  return invoke<CrawlSummary>("get_crawl_summary", { crawlId });
}

/** Fetch full detail for a single page. */
export function getPageDetail(crawlId: string, pageId: number): Promise<PageDetail> {
  return invoke<PageDetail>("get_page_detail", { crawlId, pageId });
}

/** Fetch paginated issues for a crawl. */
export function getIssues(
  crawlId: string,
  pagination: PaginationParams,
  filters?: IssueFilters,
): Promise<PaginatedResponse<IssueRow>> {
  return invoke<PaginatedResponse<IssueRow>>("get_issues", {
    crawlId,
    pagination,
    filters: filters ?? null,
  });
}

/** Fetch paginated links for a crawl. */
export function getLinks(
  crawlId: string,
  pagination: PaginationParams,
  filters?: LinkFilters,
): Promise<PaginatedResponse<LinkRow>> {
  return invoke<PaginatedResponse<LinkRow>>("get_links", {
    crawlId,
    pagination,
    filters: filters ?? null,
  });
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/** Get application settings. */
export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

/** Save application settings. */
export function setSettings(settings: AppSettings): Promise<void> {
  return invoke<void>("set_settings", { settings });
}

/** Get all rule configuration overrides. */
export function getRuleConfig(): Promise<RuleConfigOverride[]> {
  return invoke<RuleConfigOverride[]>("get_rule_config");
}

/** Save rule configuration overrides. */
export function setRuleConfig(overrides: RuleConfigOverride[]): Promise<void> {
  return invoke<void>("set_rule_config", { overrides });
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/** Export crawl data in the requested format. */
export function exportData(request: ExportRequest): Promise<ExportResult> {
  return invoke<ExportResult>("export_data", { request });
}

/** Save a crawl to a standalone .seocrawl file. Returns file path or null if cancelled. */
export function saveCrawlFile(crawlId: string): Promise<string | null> {
  return invoke<string | null>("save_crawl_file", { crawlId });
}

/** Open a .seocrawl file and import its data. Returns the imported crawl ID or null if cancelled. */
export function openCrawlFile(): Promise<string | null> {
  return invoke<string | null>("open_crawl_file");
}
