/**
 * Shared TypeScript types for OxideSEO.
 *
 * These types mirror the Rust serde-serialized structs used in Tauri IPC.
 * Keep in sync with the Rust definitions in `src-tauri/src/`.
 */

// ---------------------------------------------------------------------------
// Crawl configuration & control
// ---------------------------------------------------------------------------

export interface CrawlConfig {
  startUrl: string;
  maxDepth: number;
  maxConcurrency: number;
  fetchWorkers: number;
  parseThreads: number;
  maxMemoryMb: number;
  respectRobotsTxt: boolean;
  includePatterns: string[];
  excludePatterns: string[];
  requestTimeoutSecs: number;
  crawlDelayMs: number;
  userAgent: string | null;
  customHeaders: [string, string][];
  perHostConcurrency: number;
  maxPages: number;
  // Phase 6: Advanced crawl features
  enableJsRendering: boolean;
  jsRenderMaxConcurrent: number;
  jsRenderPatterns: string[];
  jsNeverRenderPatterns: string[];
  enableSitemapDiscovery: boolean;
  enableExternalLinkCheck: boolean;
  externalLinkConcurrency: number;
  urlRewriteRules: [string, string][];
  cookies: [string, string][];
  customCssSelectors: [string, string][];
}

export type CrawlState =
  | "created"
  | "running"
  | "paused"
  | "completed"
  | "stopped"
  | "error";

export interface CrawlStatus {
  crawlId: string;
  state: CrawlState;
  urlsCrawled: number;
  urlsQueued: number;
  urlsErrored: number;
  elapsedMs: number;
  currentRps: number;
}

export interface CrawlProgress {
  crawlId: string;
  urlsCrawled: number;
  urlsQueued: number;
  urlsErrored: number;
  currentRps: number;
  elapsedMs: number;
  recentUrls: RecentUrl[];
}

export interface RecentUrl {
  url: string;
  statusCode: number | null;
  responseTimeMs: number | null;
}

// ---------------------------------------------------------------------------
// Result data
// ---------------------------------------------------------------------------

export interface PaginationParams {
  offset: number;
  limit: number;
  sortBy: string | null;
  sortDir: "asc" | "desc" | null;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  offset: number;
  limit: number;
}

export interface PageRow {
  id: number;
  crawlId: string;
  url: string;
  depth: number;
  statusCode: number | null;
  contentType: string | null;
  responseTimeMs: number | null;
  bodySize: number | null;
  title: string | null;
  metaDesc: string | null;
  h1: string | null;
  canonical: string | null;
  robotsDirectives: string | null;
  state: string;
  fetchedAt: string | null;
  errorMessage: string | null;
  customExtractions: string | null;
  isJsRendered: boolean;
  bodyText: string | null;
}

export interface LinkRow {
  id: number;
  crawlId: string;
  sourcePage: number;
  targetUrl: string;
  anchorText: string | null;
  linkType: string;
  isInternal: boolean;
  nofollow: boolean;
}

export interface IssueRow {
  id: number;
  crawlId: string;
  pageId: number;
  ruleId: string;
  severity: Severity;
  category: RuleCategory;
  message: string;
  detailJson: string | null;
}

export interface SitemapUrlRow {
  id: number;
  crawlId: string;
  url: string;
  lastmod: string | null;
  changefreq: string | null;
  priority: number | null;
  source: string;
}

export interface ExternalLinkRow {
  id: number;
  crawlId: string;
  sourcePage: number;
  targetUrl: string;
  statusCode: number | null;
  responseTimeMs: number | null;
  errorMessage: string | null;
  checkedAt: string | null;
}

export interface SitemapReportEntry {
  url: string;
  status: string;
  pageStatusCode: number | null;
}

export type Severity = "error" | "warning" | "info";

export type RuleCategory =
  | "meta"
  | "content"
  | "links"
  | "images"
  | "performance"
  | "security"
  | "indexability"
  | "structured"
  | "international";

// ---------------------------------------------------------------------------
// Summaries
// ---------------------------------------------------------------------------

export interface CrawlSummary {
  crawlId: string;
  startUrl: string;
  status: string;
  startedAt: string | null;
  completedAt: string | null;
  urlsCrawled: number;
  urlsErrored: number;
  issueCounts: IssueCounts;
}

export interface IssueCounts {
  errors: number;
  warnings: number;
  info: number;
}

export interface PageDetail {
  page: PageRow;
  issues: IssueRow[];
  inboundLinks: LinkRow[];
  outboundLinks: LinkRow[];
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export interface AppSettings {
  defaultCrawlConfig: Record<string, unknown>;
  theme: "system" | "light" | "dark";
  defaultExportFormat: "csv" | "json" | "html";
}

export interface RuleConfigOverride {
  ruleId: string;
  enabled: boolean | null;
  severity: Severity | null;
  params: Record<string, unknown> | null;
}

// ---------------------------------------------------------------------------
// Filters
// ---------------------------------------------------------------------------

export interface PageFilters {
  urlSearch: string | null;
  statusCodes: number[] | null;
  contentType: string | null;
}

export interface IssueFilters {
  severity: Severity | null;
  category: RuleCategory | null;
  ruleId: string | null;
}

export interface LinkFilters {
  linkType: string | null;
  isInternal: boolean | null;
  isBroken: boolean | null;
  anchorTextMissing: boolean | null;
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

export type ExportDataType =
  | "pages"
  | "issues"
  | "links"
  | "images"
  | "ai_analyses"
  | "full_report";

export interface ExportRequest {
  crawlId: string;
  format: "csv" | "json" | "html";
  dataType: ExportDataType;
  columns: string[] | null;
}

export interface ExportResult {
  filePath: string;
  rowsExported: number;
}

// ---------------------------------------------------------------------------
// AI analysis (Phase 7)
// ---------------------------------------------------------------------------

export type AiProviderType = "open_ai" | "anthropic" | "ollama";

export interface AiProviderConfig {
  providerType: AiProviderType;
  model: string;
  ollamaEndpoint: string | null;
  maxTokensPerCrawl: number;
  isConfigured: boolean;
}

export type AnalysisType = "content_score" | "meta_desc" | "title_tag";

export interface AiAnalysisRow {
  id: number;
  crawlId: string;
  pageId: number;
  analysisType: AnalysisType;
  provider: string;
  model: string;
  resultJson: string;
  inputTokens: number;
  outputTokens: number;
  costUsd: number;
  latencyMs: number;
  createdAt: string;
}

export interface AiUsageRow {
  id: number;
  crawlId: string;
  provider: string;
  model: string;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCostUsd: number;
  requestCount: number;
  updatedAt: string;
}

export interface AiCrawlSummaryRow {
  id: number;
  crawlId: string;
  provider: string;
  model: string;
  summaryJson: string;
  inputTokens: number;
  outputTokens: number;
  costUsd: number;
  createdAt: string;
}

export interface BatchAnalysisFilter {
  onlyWithIssues: boolean;
  onlyMissingMeta: boolean;
  maxPages: number;
}

export interface BatchAnalysisResult {
  pagesAnalyzed: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCostUsd: number;
  errors: number;
  budgetExhausted: boolean;
}

export interface BatchCostEstimate {
  eligiblePages: number;
  estimatedInputTokens: number;
  estimatedOutputTokens: number;
  estimatedCostUsd: number;
}

export interface BatchProgress {
  completed: number;
  total: number;
  currentUrl: string;
  tokensUsed: number;
  budgetRemaining: number;
}

/** Parsed result from content quality scoring. */
export interface ContentScoreResult {
  overallScore: number;
  relevanceScore: number;
  readabilityScore: number;
  depthScore: number;
  reasoning: string;
  suggestions: string[];
}

/** Parsed result from meta description generation. */
export interface MetaDescResult {
  suggested: string;
  charCount: number;
  reasoning: string;
}

/** Parsed result from title tag suggestion. */
export interface TitleTagResult {
  suggestions: { title: string; charCount: number }[];
  reasoning: string;
}

/** Parsed result from crawl summary generation. */
export interface CrawlSummaryResult {
  summary: string;
  topActions: string[];
  overallHealth: "good" | "fair" | "poor";
  criticalIssuesCount: number;
  keyFindings: string[];
}

// ---------------------------------------------------------------------------
// Plugins (Phase 8)
// ---------------------------------------------------------------------------

/** The type of extension a plugin provides. */
export type PluginKind = "rule" | "exporter" | "post_processor" | "ui_extension";

/** Summary info about an installed plugin. */
export interface PluginInfo {
  name: string;
  version: string;
  description: string;
  kind: PluginKind;
  enabled: boolean;
  isNative: boolean;
  /** Last load error, if any. Set when the plugin fails to load during a crawl. */
  loadError: string | null;
}

/** Detailed info about a specific plugin. */
export interface PluginDetail {
  name: string;
  version: string;
  description: string;
  author: string | null;
  license: string | null;
  kind: PluginKind;
  capabilities: string[];
  enabled: boolean;
  isNative: boolean;
  config: Record<string, unknown> | null;
  installedAt: string;
}
