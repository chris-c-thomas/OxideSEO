/**
 * Typed Tauri IPC command wrappers.
 *
 * Provides a type-safe bridge between the React frontend and the Rust backend.
 * Each function maps to a `#[tauri::command]` in `src-tauri/src/commands/`.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  AiAnalysisRow,
  AiCrawlSummaryRow,
  AiProviderConfig,
  AiProviderType,
  AiUsageRow,
  AnalysisType,
  AppSettings,
  BatchAnalysisFilter,
  BatchAnalysisResult,
  BatchCostEstimate,
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
  ExternalLinkRow,
  SitemapReportEntry,
  SiteTreeNode,
  CrawlComparisonSummary,
  PageDiffRow,
  PageDiffFilters,
  IssueDiffRow,
  IssueDiffFilters,
  MetadataDiffFilters,
  PluginInfo,
  PluginDetail,
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

/** Delete a crawl and all associated data. Stops it first if active. */
export function deleteCrawl(crawlId: string): Promise<void> {
  return invoke<void>("delete_crawl", { crawlId });
}

/** Re-run a crawl using the same configuration. Returns new crawl ID. */
export function rerunCrawl(crawlId: string): Promise<string> {
  return invoke<string>("rerun_crawl", { crawlId });
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

/** Fetch sitemap cross-reference report for a crawl. */
export function getSitemapReport(crawlId: string): Promise<SitemapReportEntry[]> {
  return invoke<SitemapReportEntry[]>("get_sitemap_report", { crawlId });
}

/** Fetch paginated external link check results. */
export function getExternalLinks(
  crawlId: string,
  pagination: PaginationParams,
  isBroken?: boolean,
): Promise<PaginatedResponse<ExternalLinkRow>> {
  return invoke<PaginatedResponse<ExternalLinkRow>>("get_external_links", {
    crawlId,
    pagination,
    isBroken: isBroken ?? null,
  });
}

/** Fetch hierarchical site tree for a crawl. */
export function getSiteTree(crawlId: string): Promise<SiteTreeNode[]> {
  return invoke<SiteTreeNode[]>("get_site_tree", { crawlId });
}

// ---------------------------------------------------------------------------
// Crawl comparison
// ---------------------------------------------------------------------------

/** Fetch comparison summary between two crawls. */
export function getComparisonSummary(
  baseCrawlId: string,
  compareCrawlId: string,
): Promise<CrawlComparisonSummary> {
  return invoke<CrawlComparisonSummary>("get_comparison_summary", {
    baseCrawlId,
    compareCrawlId,
  });
}

/** Fetch paginated page diffs between two crawls. */
export function getPageDiffs(
  baseCrawlId: string,
  compareCrawlId: string,
  pagination: PaginationParams,
  filters?: PageDiffFilters,
): Promise<PaginatedResponse<PageDiffRow>> {
  return invoke<PaginatedResponse<PageDiffRow>>("get_page_diffs", {
    baseCrawlId,
    compareCrawlId,
    pagination,
    filters: filters ?? null,
  });
}

/** Fetch paginated issue diffs between two crawls. */
export function getIssueDiffs(
  baseCrawlId: string,
  compareCrawlId: string,
  pagination: PaginationParams,
  filters?: IssueDiffFilters,
): Promise<PaginatedResponse<IssueDiffRow>> {
  return invoke<PaginatedResponse<IssueDiffRow>>("get_issue_diffs", {
    baseCrawlId,
    compareCrawlId,
    pagination,
    filters: filters ?? null,
  });
}

/** Fetch paginated metadata diffs between two crawls. */
export function getMetadataDiffs(
  baseCrawlId: string,
  compareCrawlId: string,
  pagination: PaginationParams,
  filters?: MetadataDiffFilters,
): Promise<PaginatedResponse<PageDiffRow>> {
  return invoke<PaginatedResponse<PageDiffRow>>("get_metadata_diffs", {
    baseCrawlId,
    compareCrawlId,
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

// ---------------------------------------------------------------------------
// AI analysis
// ---------------------------------------------------------------------------

/** Get AI provider configuration (without API key). */
export function getAiConfig(): Promise<AiProviderConfig> {
  return invoke<AiProviderConfig>("get_ai_config");
}

/** Save AI provider configuration. */
export function setAiConfig(config: AiProviderConfig): Promise<void> {
  return invoke<void>("set_ai_config", { config });
}

/** Store an API key in the OS keychain. */
export function setApiKey(provider: AiProviderType, key: string): Promise<void> {
  return invoke<void>("set_api_key", { provider, key });
}

/** Delete an API key from the OS keychain. */
export function deleteApiKey(provider: AiProviderType): Promise<void> {
  return invoke<void>("delete_api_key", { provider });
}

/** Check if a provider has a stored API key. */
export function hasApiKey(provider: AiProviderType): Promise<boolean> {
  return invoke<boolean>("has_api_key", { provider });
}

/** Test connectivity to the configured AI provider. Returns model info on success. */
export function testAiConnection(): Promise<string> {
  return invoke<string>("test_ai_connection");
}

/** Analyze a single page with the configured AI provider. */
export function analyzePage(
  crawlId: string,
  pageId: number,
  analysisTypes: AnalysisType[],
): Promise<AiAnalysisRow[]> {
  return invoke<AiAnalysisRow[]>("analyze_page", {
    crawlId,
    pageId,
    analysisTypes,
  });
}

/** Batch analyze pages for a crawl. */
export function batchAnalyzePages(
  crawlId: string,
  filter: BatchAnalysisFilter,
  analysisTypes: AnalysisType[],
): Promise<BatchAnalysisResult> {
  return invoke<BatchAnalysisResult>("batch_analyze_pages", {
    crawlId,
    filter,
    analysisTypes,
  });
}

/** Generate an AI summary for a completed crawl. Pass force=true to regenerate. */
export function generateCrawlSummary(
  crawlId: string,
  force?: boolean,
): Promise<AiCrawlSummaryRow> {
  return invoke<AiCrawlSummaryRow>("generate_crawl_summary", {
    crawlId,
    force: force ?? false,
  });
}

/** Get cached AI analyses for a page. */
export function getPageAnalyses(
  crawlId: string,
  pageId: number,
): Promise<AiAnalysisRow[]> {
  return invoke<AiAnalysisRow[]>("get_page_analyses", { crawlId, pageId });
}

/** Get AI usage/cost stats for a crawl. */
export function getAiUsage(crawlId: string): Promise<AiUsageRow[]> {
  return invoke<AiUsageRow[]>("get_ai_usage", { crawlId });
}

/** Get cached AI crawl summary. */
export function getCrawlAiSummary(crawlId: string): Promise<AiCrawlSummaryRow | null> {
  return invoke<AiCrawlSummaryRow | null>("get_crawl_ai_summary", { crawlId });
}

/** List models installed on an Ollama instance. */
export function listOllamaModels(endpoint: string): Promise<string[]> {
  return invoke<string[]>("list_ollama_models", { endpoint });
}

/** Estimate cost of a batch analysis before running it. */
export function estimateBatchCost(
  crawlId: string,
  filter: BatchAnalysisFilter,
  analysisTypes: AnalysisType[],
): Promise<BatchCostEstimate> {
  return invoke<BatchCostEstimate>("estimate_batch_cost", {
    crawlId,
    filter,
    analysisTypes,
  });
}

// ---------------------------------------------------------------------------
// Plugins
// ---------------------------------------------------------------------------

/** List all discovered plugins. */
export function listPlugins(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>("list_plugins");
}

/** Enable a plugin by name. */
export function enablePlugin(name: string): Promise<void> {
  return invoke<void>("enable_plugin", { name });
}

/** Disable a plugin by name. */
export function disablePlugin(name: string): Promise<void> {
  return invoke<void>("disable_plugin", { name });
}

/** Get detailed info about a specific plugin. */
export function getPluginDetail(name: string): Promise<PluginDetail> {
  return invoke<PluginDetail>("get_plugin_detail", { name });
}

/** Re-scan the plugins directory for new or updated plugins. */
export function reloadPlugins(): Promise<PluginInfo[]> {
  return invoke<PluginInfo[]>("reload_plugins");
}

/** Install a plugin from a directory chosen via file dialog. */
export function installPluginFromFile(): Promise<PluginInfo | null> {
  return invoke<PluginInfo | null>("install_plugin_from_file");
}

/** Uninstall a plugin by name. */
export function uninstallPlugin(name: string): Promise<void> {
  return invoke<void>("uninstall_plugin", { name });
}
