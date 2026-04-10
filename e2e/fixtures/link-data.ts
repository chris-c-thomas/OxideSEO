/**
 * Test fixture factories for link-related types.
 */

import type { LinkRow, PaginatedResponse } from "@/types";
import { CRAWL_ID_1 } from "./crawl-data";

export function makeLinkRow(overrides?: Partial<LinkRow>): LinkRow {
  return {
    id: 1,
    crawlId: CRAWL_ID_1,
    sourcePage: 1,
    targetUrl: "https://example.com/about",
    anchorText: "About Us",
    linkType: "a",
    isInternal: true,
    nofollow: false,
    ...overrides,
  };
}

export const SAMPLE_LINKS: LinkRow[] = [
  makeLinkRow({
    id: 1,
    targetUrl: "https://example.com/about",
    anchorText: "About Us",
  }),
  makeLinkRow({
    id: 2,
    targetUrl: "https://example.com/contact",
    anchorText: "Contact",
  }),
  makeLinkRow({
    id: 3,
    targetUrl: "https://external.com/resource",
    anchorText: "External Resource",
    isInternal: false,
  }),
  makeLinkRow({
    id: 4,
    targetUrl: "https://example.com/blog",
    anchorText: null,
    nofollow: true,
  }),
];

export function makePaginatedLinks(
  links?: LinkRow[],
): PaginatedResponse<LinkRow> {
  const items = links ?? SAMPLE_LINKS;
  return {
    items,
    total: items.length,
    offset: 0,
    limit: 50,
  };
}
