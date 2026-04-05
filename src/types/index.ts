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
  redirectChain: RedirectHop[] | null;
}

export interface RedirectHop {
  url: string;
  statusCode: number;
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export interface AppSettings {
  defaultCrawlConfig: Record<string, unknown>;
  theme: "system" | "light" | "dark";
  defaultExportFormat: "csv" | "json" | "html" | "xlsx";
}

export interface RuleConfigOverride {
  ruleId: string;
  enabled: boolean | null;
  severity: string | null;
  params: Record<string, unknown> | null;
}

// ---------------------------------------------------------------------------
// Filters
// ---------------------------------------------------------------------------

export interface PageFilters {
  urlSearch: string | null;
  statusCodes: number[] | null;
  minSeverity: Severity | null;
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
