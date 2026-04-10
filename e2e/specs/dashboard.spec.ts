/**
 * E2E tests for the Dashboard view.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";
import {
  SAMPLE_CRAWL_LIST,
  CRAWL_ID_1,
  CRAWL_ID_3,
  makeCrawlSummary,
} from "../fixtures/crawl-data";

test.describe("Dashboard Empty State", () => {
  test("shows empty state when no crawls exist", async ({ page }) => {
    const app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);

    await expect(page.getByText("No crawls yet")).toBeVisible();
    await expect(page.getByText("Configure Crawl")).toBeVisible();
  });

  test("Configure Crawl CTA navigates to crawl config", async ({ page }) => {
    const app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);

    await page.getByRole("button", { name: "Configure Crawl" }).click();
    await expect(
      page.getByRole("heading", { name: /Configure Crawl/i }).first(),
    ).toBeVisible();
  });
});

test.describe("Dashboard Crawl List", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withRecentCrawls(SAMPLE_CRAWL_LIST)
      .build();
    await app.setup(mocks);
  });

  test("displays crawl entries with URLs and status badges", async ({
    page,
  }) => {
    await expect(page.getByText("https://example.com")).toBeVisible();
    await expect(page.getByText("https://blog.example.com")).toBeVisible();
    await expect(page.getByText("https://shop.example.com")).toBeVisible();
    // Status badges
    await expect(page.getByText("completed")).toBeVisible();
    await expect(page.getByText("running")).toBeVisible();
    await expect(page.getByText("stopped")).toBeVisible();
  });

  test("shows metric cards with aggregate data", async ({ page }) => {
    await expect(page.getByText("Total URLs")).toBeVisible();
    await expect(page.getByText("Errors")).toBeVisible();
    await expect(page.getByText("Warnings")).toBeVisible();
  });

  test("shows page counts for crawls", async ({ page }) => {
    await expect(page.getByText("150 pages")).toBeVisible();
    await expect(page.getByText("42 pages")).toBeVisible();
  });

  test("clicking a completed crawl row navigates away from dashboard", async ({
    page,
  }) => {
    // Click the first crawl row (example.com - completed)
    await page.getByText("https://example.com", { exact: true }).click();
    // Should navigate away from dashboard — the heading "Dashboard" should no longer be visible
    await expect(
      page.getByRole("heading", { name: "Dashboard" }),
    ).toBeHidden();
  });
});

test.describe("Dashboard Actions", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    // Use only completed and stopped crawls for action tests (no running crawl to avoid ambiguity)
    const mocks = new TauriMockBuilder()
      .withRecentCrawls([
        makeCrawlSummary({ crawlId: CRAWL_ID_1, status: "completed" }),
        makeCrawlSummary({
          crawlId: CRAWL_ID_3,
          startUrl: "https://shop.example.com",
          status: "stopped",
        }),
      ])
      .build();
    await app.setup(mocks);
  });

  test("completed crawl dropdown shows Re-run and View Results", async ({
    page,
  }) => {
    // Click the dropdown trigger for the first crawl row
    const dropdownTriggers = page.locator('[aria-label="Crawl actions"]');
    await dropdownTriggers.first().click();

    await expect(page.getByRole("menuitem", { name: "Re-run" })).toBeVisible();
    await expect(
      page.getByRole("menuitem", { name: "View Results" }),
    ).toBeVisible();
    await expect(
      page.getByRole("menuitem", { name: "Delete" }),
    ).toBeVisible();
  });

  test("delete shows confirmation dialog", async ({ page }) => {
    // Open dropdown for the second crawl (shop.example.com)
    const dropdownTriggers = page.locator('[aria-label="Crawl actions"]');
    await dropdownTriggers.last().click();

    await page.getByRole("menuitem", { name: "Delete" }).click();
    // Confirm dialog
    await expect(page.getByText("Delete Crawl")).toBeVisible();
    await expect(
      page.getByText("permanently delete", { exact: false }),
    ).toBeVisible();
  });

  test("canceling delete keeps crawl in list", async ({ page }) => {
    const dropdownTriggers = page.locator('[aria-label="Crawl actions"]');
    await dropdownTriggers.last().click();

    await page.getByRole("menuitem", { name: "Delete" }).click();
    // Cancel the dialog
    await page.getByRole("button", { name: "Cancel" }).click();
    // Crawl should still be visible
    await expect(page.getByText("https://shop.example.com")).toBeVisible();
  });
});

test.describe("Dashboard Compare Mode", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withRecentCrawls(SAMPLE_CRAWL_LIST)
      .build();
    await app.setup(mocks);
  });

  test("Compare button toggles compare mode", async ({ page }) => {
    await page.getByRole("button", { name: "Compare" }).click();
    // Should show cancel text and selection hint
    await expect(page.getByText("Select 2 completed crawls")).toBeVisible();
    await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible();
  });

  test("checkboxes appear in compare mode for selectable crawls", async ({
    page,
  }) => {
    await page.getByRole("button", { name: "Compare" }).click();
    // Checkboxes should appear in compare mode
    const checkboxes = page.getByRole("checkbox");
    await expect(checkboxes.first()).toBeVisible();
  });
});

test.describe("Dashboard Error States", () => {
  test("shows error when getRecentCrawls fails", async ({ page }) => {
    const app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withCommandError("get_recent_crawls", "Database connection failed")
      .build();
    await app.setup(mocks);

    await expect(page.getByText("Database connection failed")).toBeVisible();
  });
});
