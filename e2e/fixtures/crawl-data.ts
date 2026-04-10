/**
 * Test fixture factories for crawl-related types.
 */

import type {
  CrawlSummary,
  CrawlProgress,
  CrawlConfig,
  CrawlStatus,
  RecentUrl,
} from "@/types";

export const CRAWL_ID_1 = "e2e-crawl-001";
export const CRAWL_ID_2 = "e2e-crawl-002";
export const CRAWL_ID_3 = "e2e-crawl-003";

export function makeCrawlSummary(
  overrides?: Partial<CrawlSummary>,
): CrawlSummary {
  return {
    crawlId: CRAWL_ID_1,
    startUrl: "https://example.com",
    status: "completed",
    startedAt: "2026-04-08T10:00:00Z",
    completedAt: "2026-04-08T10:05:00Z",
    urlsCrawled: 150,
    urlsErrored: 3,
    issueCounts: { errors: 5, warnings: 12, info: 8 },
    ...overrides,
  };
}

export function makeCrawlStatus(
  overrides?: Partial<CrawlStatus>,
): CrawlStatus {
  return {
    crawlId: CRAWL_ID_1,
    state: "running",
    urlsCrawled: 75,
    urlsQueued: 200,
    urlsErrored: 2,
    elapsedMs: 30000,
    currentRps: 12.5,
    ...overrides,
  };
}

export function makeCrawlProgress(
  overrides?: Partial<CrawlProgress>,
): CrawlProgress {
  return {
    crawlId: CRAWL_ID_1,
    urlsCrawled: 75,
    urlsQueued: 200,
    urlsErrored: 2,
    currentRps: 12.5,
    elapsedMs: 30000,
    recentUrls: SAMPLE_RECENT_URLS,
    memoryRssBytes: 256 * 1024 * 1024,
    ...overrides,
  };
}

export function makeCrawlConfig(
  overrides?: Partial<CrawlConfig>,
): CrawlConfig {
  return {
    startUrl: "https://example.com",
    maxDepth: 10,
    maxConcurrency: 50,
    fetchWorkers: 8,
    parseThreads: 4,
    maxMemoryMb: 512,
    respectRobotsTxt: true,
    includePatterns: [],
    excludePatterns: [],
    requestTimeoutSecs: 30,
    crawlDelayMs: 0,
    userAgent: null,
    customHeaders: [],
    perHostConcurrency: 10,
    maxPages: 0,
    enableJsRendering: false,
    jsRenderMaxConcurrent: 4,
    jsRenderPatterns: [],
    jsNeverRenderPatterns: [],
    enableSitemapDiscovery: true,
    enableExternalLinkCheck: false,
    externalLinkConcurrency: 10,
    urlRewriteRules: [],
    cookies: [],
    customCssSelectors: [],
    ...overrides,
  };
}

export const SAMPLE_RECENT_URLS: RecentUrl[] = [
  { url: "https://example.com/page-1", statusCode: 200, responseTimeMs: 150 },
  { url: "https://example.com/page-2", statusCode: 301, responseTimeMs: 80 },
  { url: "https://example.com/page-3", statusCode: 404, responseTimeMs: 200 },
  { url: "https://example.com/page-4", statusCode: 200, responseTimeMs: 120 },
  { url: "https://example.com/page-5", statusCode: 500, responseTimeMs: 300 },
];

export const SAMPLE_CRAWL_LIST: CrawlSummary[] = [
  makeCrawlSummary({ crawlId: CRAWL_ID_1, status: "completed" }),
  makeCrawlSummary({
    crawlId: CRAWL_ID_2,
    startUrl: "https://blog.example.com",
    status: "running",
    completedAt: null,
    urlsCrawled: 42,
    urlsErrored: 0,
    issueCounts: { errors: 0, warnings: 3, info: 1 },
  }),
  makeCrawlSummary({
    crawlId: CRAWL_ID_3,
    startUrl: "https://shop.example.com",
    status: "stopped",
    urlsCrawled: 80,
    urlsErrored: 1,
    issueCounts: { errors: 2, warnings: 5, info: 3 },
  }),
];
