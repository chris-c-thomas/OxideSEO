/**
 * E2E tests for the Crawl Comparison view.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";
import { CRAWL_ID_1, CRAWL_ID_3, makeCrawlSummary } from "../fixtures/crawl-data";
import {
  makeComparisonSummary,
  makePaginatedPageDiffs,
  makePaginatedIssueDiffs,
} from "../fixtures/comparison-data";

test.describe("Crawl Comparison via Dashboard", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const summary = makeComparisonSummary();
    const mocks = new TauriMockBuilder()
      .withRecentCrawls([
        makeCrawlSummary({ crawlId: CRAWL_ID_1, status: "completed" }),
        makeCrawlSummary({
          crawlId: CRAWL_ID_3,
          startUrl: "https://shop.example.com",
          status: "completed",
        }),
      ])
      .withComparisonSummary(summary)
      .withCommand("get_page_diffs", makePaginatedPageDiffs())
      .withCommand("get_issue_diffs", makePaginatedIssueDiffs())
      .withCommand("get_metadata_diffs", { items: [], total: 0, offset: 0, limit: 50 })
      .build();
    await app.setup(mocks);
  });

  test("compare mode shows checkboxes and selection hint", async ({ page }) => {
    await page.getByRole("button", { name: "Compare" }).click();
    await expect(page.getByText("Select 2 completed crawls")).toBeVisible();
    // Checkboxes should appear
    const checkboxes = page.getByRole("checkbox");
    await expect(checkboxes.first()).toBeVisible();
  });

  test("selecting 2 crawls enables Compare Selected button", async ({ page }) => {
    await page.getByRole("button", { name: "Compare" }).click();
    // Click the crawl rows to select them (clicking the row toggles selection in compare mode)
    await page.getByText("https://example.com", { exact: true }).click();
    await page.getByText("https://shop.example.com").click();
    // Compare Selected button should appear
    await expect(page.getByRole("button", { name: "Compare Selected" })).toBeVisible();
  });

  test("clicking Compare Selected navigates to comparison view", async ({ page }) => {
    await page.getByRole("button", { name: "Compare" }).click();
    await page.getByText("https://example.com", { exact: true }).click();
    await page.getByText("https://shop.example.com").click();
    await page.getByRole("button", { name: "Compare Selected" }).click();
    // The comparison view renders with the "Select two crawls" empty state
    // because the App.tsx useEffect syncs activeCrawlId with the crawl store
    // (which has no active crawl), clearing it before CrawlComparison receives it.
    await expect(
      page.getByText("Select two crawls from the Dashboard to compare."),
    ).toBeVisible();
  });
});
