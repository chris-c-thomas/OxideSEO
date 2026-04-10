/**
 * Test fixture factories for issue-related types.
 */

import type { IssueRow, PaginatedResponse } from "@/types";
import { CRAWL_ID_1 } from "./crawl-data";

export function makeIssueRow(overrides?: Partial<IssueRow>): IssueRow {
  return {
    id: 1,
    crawlId: CRAWL_ID_1,
    pageId: 1,
    ruleId: "title-missing",
    severity: "error",
    category: "meta",
    message: "Page is missing a <title> tag.",
    detailJson: null,
    ...overrides,
  };
}

export const SAMPLE_ISSUES: IssueRow[] = [
  makeIssueRow({
    id: 1,
    ruleId: "title-missing",
    severity: "error",
    category: "meta",
    message: "Page is missing a <title> tag.",
  }),
  makeIssueRow({
    id: 2,
    pageId: 2,
    ruleId: "meta-description-length",
    severity: "warning",
    category: "meta",
    message:
      "Meta description is too short (32 characters). Recommended: 120-160 characters.",
  }),
  makeIssueRow({
    id: 3,
    pageId: 3,
    ruleId: "h1-missing",
    severity: "warning",
    category: "content",
    message: "Page is missing an <h1> heading.",
  }),
  makeIssueRow({
    id: 4,
    pageId: 1,
    ruleId: "image-alt-missing",
    severity: "warning",
    category: "images",
    message: "Image is missing alt text.",
  }),
  makeIssueRow({
    id: 5,
    pageId: 4,
    ruleId: "broken-link",
    severity: "error",
    category: "links",
    message: "Link target returns 404 status.",
  }),
  makeIssueRow({
    id: 6,
    pageId: 5,
    ruleId: "mixed-content",
    severity: "info",
    category: "security",
    message: "Page loads resources over HTTP on an HTTPS page.",
  }),
];

export function makePaginatedIssues(issues?: IssueRow[]): PaginatedResponse<IssueRow> {
  const items = issues ?? SAMPLE_ISSUES;
  return {
    items,
    total: items.length,
    offset: 0,
    limit: 50,
  };
}
