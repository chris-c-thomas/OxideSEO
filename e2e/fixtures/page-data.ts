/**
 * Test fixture factories for page-related types.
 */

import type { PageRow, PageDetail, PaginatedResponse, IssueRow, LinkRow } from "@/types";
import { CRAWL_ID_1 } from "./crawl-data";

export function makePageRow(overrides?: Partial<PageRow>): PageRow {
  return {
    id: 1,
    crawlId: CRAWL_ID_1,
    url: "https://example.com/",
    depth: 0,
    statusCode: 200,
    contentType: "text/html",
    responseTimeMs: 150,
    bodySize: 45000,
    title: "Example Domain - Home",
    metaDesc: "Welcome to Example Domain, the premier example website.",
    h1: "Welcome to Example",
    canonical: "https://example.com/",
    robotsDirectives: null,
    state: "completed",
    fetchedAt: "2026-04-08T10:00:01Z",
    errorMessage: null,
    customExtractions: null,
    isJsRendered: false,
    bodyText: null,
    ...overrides,
  };
}

export function makePageDetail(overrides?: Partial<PageDetail>): PageDetail {
  const page = makePageRow(overrides?.page);
  return {
    page,
    issues: overrides?.issues ?? [],
    inboundLinks: overrides?.inboundLinks ?? [],
    outboundLinks: overrides?.outboundLinks ?? [],
  };
}

export function makePaginatedPages(
  count: number,
  total?: number,
): PaginatedResponse<PageRow> {
  const items: PageRow[] = [];
  for (let i = 0; i < count; i++) {
    items.push(
      makePageRow({
        id: i + 1,
        url: `https://example.com/page-${i + 1}`,
        title: `Page ${i + 1} - Example`,
        depth: Math.min(i, 5),
        statusCode: i % 10 === 0 ? 404 : 200,
        responseTimeMs: 100 + i * 10,
        bodySize: 30000 + i * 1000,
      }),
    );
  }
  return {
    items,
    total: total ?? count,
    offset: 0,
    limit: 50,
  };
}

export const SAMPLE_PAGE_DETAIL: PageDetail = makePageDetail({
  page: makePageRow({
    id: 1,
    url: "https://example.com/about",
    title: "About Us - Example",
    metaDesc: "Learn more about Example Domain.",
    h1: "About Us",
    statusCode: 200,
    responseTimeMs: 180,
    bodySize: 52000,
  }),
  issues: [
    {
      id: 1,
      crawlId: CRAWL_ID_1,
      pageId: 1,
      ruleId: "meta-description-length",
      severity: "warning",
      category: "meta",
      message:
        "Meta description is too short (32 characters). Recommended: 120-160 characters.",
      detailJson: null,
    },
  ] satisfies IssueRow[],
  inboundLinks: [
    {
      id: 1,
      crawlId: CRAWL_ID_1,
      sourcePage: 2,
      targetUrl: "https://example.com/about",
      anchorText: "About Us",
      linkType: "a",
      isInternal: true,
      nofollow: false,
    },
  ] satisfies LinkRow[],
  outboundLinks: [
    {
      id: 2,
      crawlId: CRAWL_ID_1,
      sourcePage: 1,
      targetUrl: "https://example.com/contact",
      anchorText: "Contact",
      linkType: "a",
      isInternal: true,
      nofollow: false,
    },
  ] satisfies LinkRow[],
});
