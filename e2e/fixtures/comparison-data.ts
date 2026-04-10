/**
 * Test fixture factories for crawl comparison types.
 */

import type {
  CrawlComparisonSummary,
  PageDiffRow,
  IssueDiffRow,
  PaginatedResponse,
} from "@/types";
import { makeCrawlSummary, CRAWL_ID_1, CRAWL_ID_2 } from "./crawl-data";

export function makeComparisonSummary(
  overrides?: Partial<CrawlComparisonSummary>,
): CrawlComparisonSummary {
  return {
    baseCrawl: makeCrawlSummary({ crawlId: CRAWL_ID_1 }),
    compareCrawl: makeCrawlSummary({
      crawlId: CRAWL_ID_2,
      startUrl: "https://example.com",
      startedAt: "2026-04-09T10:00:00Z",
      completedAt: "2026-04-09T10:06:00Z",
      urlsCrawled: 165,
      urlsErrored: 1,
    }),
    newPages: 15,
    removedPages: 3,
    changedStatusCode: 5,
    changedTitle: 8,
    changedMetaDesc: 4,
    newIssues: 7,
    resolvedIssues: 10,
    ...overrides,
  };
}

export function makePageDiffRow(
  overrides?: Partial<PageDiffRow>,
): PageDiffRow {
  return {
    url: "https://example.com/new-page",
    diffType: "new",
    baseStatusCode: null,
    compareStatusCode: 200,
    baseTitle: null,
    compareTitle: "New Page - Example",
    baseMetaDesc: null,
    compareMetaDesc: "A new page on the site.",
    ...overrides,
  };
}

export function makeIssueDiffRow(
  overrides?: Partial<IssueDiffRow>,
): IssueDiffRow {
  return {
    url: "https://example.com/about",
    ruleId: "title-missing",
    severity: "error",
    category: "meta",
    message: "Page is missing a <title> tag.",
    diffType: "resolved",
    ...overrides,
  };
}

export const SAMPLE_PAGE_DIFFS: PageDiffRow[] = [
  makePageDiffRow({ url: "https://example.com/new-1", diffType: "new" }),
  makePageDiffRow({
    url: "https://example.com/old-page",
    diffType: "removed",
    baseStatusCode: 200,
    compareStatusCode: null,
    baseTitle: "Old Page",
    compareTitle: null,
  }),
  makePageDiffRow({
    url: "https://example.com/changed",
    diffType: "status_code_changed",
    baseStatusCode: 200,
    compareStatusCode: 301,
    baseTitle: "Changed Page",
    compareTitle: "Changed Page",
  }),
];

export const SAMPLE_ISSUE_DIFFS: IssueDiffRow[] = [
  makeIssueDiffRow({ diffType: "resolved" }),
  makeIssueDiffRow({
    url: "https://example.com/blog",
    ruleId: "h1-missing",
    severity: "warning",
    category: "content",
    message: "Page is missing an <h1> heading.",
    diffType: "new",
  }),
];

export function makePaginatedPageDiffs(
  diffs?: PageDiffRow[],
): PaginatedResponse<PageDiffRow> {
  const items = diffs ?? SAMPLE_PAGE_DIFFS;
  return { items, total: items.length, offset: 0, limit: 50 };
}

export function makePaginatedIssueDiffs(
  diffs?: IssueDiffRow[],
): PaginatedResponse<IssueDiffRow> {
  const items = diffs ?? SAMPLE_ISSUE_DIFFS;
  return { items, total: items.length, offset: 0, limit: 50 };
}
