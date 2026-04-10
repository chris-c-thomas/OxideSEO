/**
 * E2E tests for the standalone Issues view (sidebar navigation target).
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";
import { CRAWL_ID_1 } from "../fixtures/crawl-data";
import { makePaginatedIssues, SAMPLE_ISSUES } from "../fixtures/issue-data";

test.describe("Issues View Empty State", () => {
  test("shows empty state when no crawl is selected", async ({ page }) => {
    const app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("Issues");

    await expect(page.getByText("No crawl selected")).toBeVisible();
  });
});

test.describe("Issues View with Data", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withStartCrawlId(CRAWL_ID_1)
      .withIssues(makePaginatedIssues())
      .build();
    await app.setup(mocks);
    // Start a crawl to establish active crawlId
    await app.navigateTo("New Crawl");
    await page.getByPlaceholder("https://example.com").fill("https://test.com");
    await page.getByRole("button", { name: "Start Crawl" }).click();
    await expect(page.getByRole("heading", { name: "Crawl Monitor" })).toBeVisible();
    await app.navigateTo("Issues");
  });

  test("shows Issues heading", async ({ page }) => {
    await expect(page.getByRole("heading", { name: "Issues" })).toBeVisible();
  });

  test("displays issue data from mock", async ({ page }) => {
    // Should show at least one issue message from the fixtures
    await expect(
      page.getByText(/missing a <title> tag|Meta description is too short/i).first(),
    ).toBeVisible();
  });

  test("shows filter bar", async ({ page }) => {
    // The IssueFilterBar should be visible
    await expect(page.getByText(/Severity|Category/i).first()).toBeVisible();
  });
});
