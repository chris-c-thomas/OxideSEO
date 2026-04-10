/**
 * E2E tests for the Crawl Monitor view.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";
import { emitCrawlProgress, emitCrawlState } from "../helpers/event-emitter";
import { CRAWL_ID_1, makeCrawlProgress } from "../fixtures/crawl-data";

test.describe("Crawl Monitor Empty State", () => {
  test("shows empty state when no crawlId is set", async ({ page }) => {
    const app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("Monitor");

    await expect(page.getByText("No active crawl")).toBeVisible();
  });
});

test.describe("Crawl Monitor Active Crawl", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withStartCrawlId(CRAWL_ID_1)
      .build();
    await app.setup(mocks);

    // Navigate to crawl config, start a crawl to get a crawlId
    await app.navigateTo("New Crawl");
    await page
      .getByPlaceholder("https://example.com")
      .fill("https://test.com");
    await page.getByRole("button", { name: "Start Crawl" }).click();

    // Should now be on crawl monitor
    await expect(
      page.getByRole("heading", { name: "Crawl Monitor" }),
    ).toBeVisible();
  });

  test("displays Crawl Monitor heading", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: "Crawl Monitor" }),
    ).toBeVisible();
  });

  test("shows metric cards", async ({ page }) => {
    await expect(page.getByText("Crawled")).toBeVisible();
    await expect(page.getByText("Queued")).toBeVisible();
    await expect(page.getByText("Errors")).toBeVisible();
    await expect(page.getByText("Speed")).toBeVisible();
  });

  test("shows Recent URLs table header", async ({ page }) => {
    await expect(page.getByText("Recent URLs")).toBeVisible();
    await expect(page.getByText("Waiting for crawl data...")).toBeVisible();
  });

  test("Pause and Stop buttons are visible for running crawl", async ({
    page,
  }) => {
    await expect(
      page.getByRole("button", { name: "Pause" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Stop" }),
    ).toBeVisible();
  });

  test("Stop button shows confirmation dialog", async ({ page }) => {
    await page.getByRole("button", { name: "Stop" }).click();
    await expect(page.getByText("Stop Crawl")).toBeVisible();
    await expect(
      page.getByText("In-flight requests will complete"),
    ).toBeVisible();
  });
});

test.describe("Crawl Monitor Live Updates", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withStartCrawlId(CRAWL_ID_1)
      .build();
    await app.setup(mocks);

    // Start a crawl
    await app.navigateTo("New Crawl");
    await page
      .getByPlaceholder("https://example.com")
      .fill("https://test.com");
    await page.getByRole("button", { name: "Start Crawl" }).click();
    await expect(
      page.getByRole("heading", { name: "Crawl Monitor" }),
    ).toBeVisible();
  });

  test("progress event updates metrics and recent URLs", async ({ page }) => {
    const progress = makeCrawlProgress({
      crawlId: CRAWL_ID_1,
      urlsCrawled: 42,
      urlsQueued: 100,
      urlsErrored: 3,
    });

    await emitCrawlProgress(page, progress);

    // Wait for the recent URL to appear
    await expect(
      page.getByText("https://example.com/page-1"),
    ).toBeVisible();
  });

  test("completed state shows View Results button", async ({ page }) => {
    await emitCrawlState(page, {
      crawlId: CRAWL_ID_1,
      state: "completed",
    });

    await expect(
      page.getByRole("button", { name: "View Results" }),
    ).toBeVisible();
    // Pause/Stop should be hidden
    await expect(
      page.getByRole("button", { name: "Pause" }),
    ).toBeHidden();
  });

  test("stopped state shows View Results button", async ({ page }) => {
    await emitCrawlState(page, {
      crawlId: CRAWL_ID_1,
      state: "stopped",
    });

    await expect(
      page.getByRole("button", { name: "View Results" }),
    ).toBeVisible();
  });
});
