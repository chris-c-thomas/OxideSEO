/**
 * E2E tests for the Export Dialog.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";
import { CRAWL_ID_1, makeCrawlSummary } from "../fixtures/crawl-data";
import { makePaginatedPages } from "../fixtures/page-data";

async function openExportDialog(app: AppHelper) {
  const mocks = new TauriMockBuilder()
    .withStartCrawlId(CRAWL_ID_1)
    .withCrawlSummary(makeCrawlSummary({ crawlId: CRAWL_ID_1 }))
    .withCrawlResults(makePaginatedPages(5))
    .withExportResult({ filePath: "/tmp/export.csv", rowsExported: 150 })
    .build();
  await app.setup(mocks);
  // Start a crawl to get a crawlId
  await app.navigateTo("New Crawl");
  await app.page.getByPlaceholder("https://example.com").fill("https://test.com");
  await app.page.getByRole("button", { name: "Start Crawl" }).click();
  await expect(app.page.getByRole("heading", { name: "Crawl Monitor" })).toBeVisible();
  await app.navigateTo("Results");
  await expect(app.page.getByRole("heading", { name: "Results" })).toBeVisible();
  // Open export dialog
  await app.page.getByRole("button", { name: "Export" }).click();
  await expect(app.page.getByRole("dialog")).toBeVisible();
}

test.describe("Export Dialog Format Selection", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    await openExportDialog(app);
  });

  test("renders format options", async ({ page }) => {
    await expect(page.getByText("CSV")).toBeVisible();
    await expect(page.getByText("JSON Lines")).toBeVisible();
    await expect(page.getByText("Excel")).toBeVisible();
    await expect(page.getByText("PDF Report")).toBeVisible();
    await expect(page.getByText("HTML Report")).toBeVisible();
  });

  test("renders data type options in dialog", async ({ page }) => {
    const dialog = page.getByRole("dialog");
    await expect(dialog.getByText("Data Type", { exact: true })).toBeVisible();
    await expect(dialog.getByText("Pages", { exact: true })).toBeVisible();
    await expect(dialog.getByText("Issues", { exact: true })).toBeVisible();
  });

  test("Export button is present", async ({ page }) => {
    await expect(
      page.getByRole("button", { name: /Export/i }).last(),
    ).toBeVisible();
  });
});

test.describe("Export Dialog Column Selection", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    await openExportDialog(app);
  });

  test("shows column checkboxes for CSV format", async ({ page }) => {
    // CSV is the default format, and Pages is the default data type
    // Column checkboxes should be visible
    await expect(page.getByText("URL", { exact: true }).first()).toBeVisible();
    await expect(page.getByText("Status Code").first()).toBeVisible();
    await expect(page.getByText("Title", { exact: true }).first()).toBeVisible();
  });
});
