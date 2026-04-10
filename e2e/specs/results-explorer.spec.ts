/**
 * E2E tests for the Results Explorer view.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";
import { CRAWL_ID_1, makeCrawlSummary } from "../fixtures/crawl-data";
import { makePaginatedPages, SAMPLE_PAGE_DETAIL } from "../fixtures/page-data";
import { makePaginatedIssues } from "../fixtures/issue-data";
import { makePaginatedLinks } from "../fixtures/link-data";

/**
 * Helper to set up the Results Explorer with a crawl loaded.
 * Starts a crawl from the config page, then navigates to results via the sidebar.
 */
async function setupWithResults(app: AppHelper, mocks: ReturnType<TauriMockBuilder["build"]>) {
  await app.setup(mocks);
  // Start a crawl to get an active crawlId in the app state
  await app.navigateTo("New Crawl");
  await app.page.getByPlaceholder("https://example.com").fill("https://test.com");
  await app.page.getByRole("button", { name: "Start Crawl" }).click();
  // Wait for monitor, then navigate to results
  await expect(app.page.getByRole("heading", { name: "Crawl Monitor" })).toBeVisible();
  await app.navigateTo("Results");
}

test.describe("Results Explorer with Data", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withStartCrawlId(CRAWL_ID_1)
      .withCrawlSummary(
        makeCrawlSummary({
          crawlId: CRAWL_ID_1,
          urlsCrawled: 150,
          issueCounts: { errors: 5, warnings: 12, info: 8 },
        }),
      )
      .withCrawlResults(makePaginatedPages(10, 150))
      .withIssues(makePaginatedIssues())
      .withLinks(makePaginatedLinks())
      .withPageDetail(SAMPLE_PAGE_DETAIL)
      .build();
    await setupWithResults(app, mocks);
  });

  test("shows Results heading", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: "Results" }),
    ).toBeVisible();
  });

  test("renders all 8 tab triggers", async ({ page }) => {
    const tabLabels = [
      "All Pages",
      "Issues",
      "Links",
      "Images",
      "Sitemap",
      "External",
      "AI Insights",
      "Site Tree",
    ];
    for (const label of tabLabels) {
      await expect(page.getByRole("tab", { name: label })).toBeVisible();
    }
  });

  test("shows summary bar with page count", async ({ page }) => {
    await expect(page.getByText("150 pages")).toBeVisible();
  });

  test("Export button is visible", async ({ page }) => {
    await expect(
      page.getByRole("button", { name: "Export" }),
    ).toBeVisible();
  });

  test("clicking Export opens the export dialog", async ({ page }) => {
    await page.getByRole("button", { name: "Export" }).click();
    await expect(page.getByRole("dialog")).toBeVisible();
    await expect(page.getByText("Export Crawl Data")).toBeVisible();
  });

  test("switching to Issues tab shows issues content", async ({ page }) => {
    await page.getByRole("tab", { name: "Issues" }).click();
    // Wait for issues content to load
    await expect(
      page.getByText(/title-missing|meta-description-length|h1-missing/i).first(),
    ).toBeVisible();
  });
});

test.describe("Results Explorer Empty State", () => {
  test("shows empty state when no crawl is selected", async ({ page }) => {
    const app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("Results");
    await expect(page.getByText("No crawl selected")).toBeVisible();
  });
});
